"""候选噪声片段的诊断（只读）：逐秒电平 · 频谱峰 · 平稳度 · 单音。
回答「这段底噪为什么不平稳 / 有没有哼声」。

- 平稳度不另写一份：`level_std_db` 直接调 `screen.level_std_db`，与筛查用同一个判据、同一个下限
- 频谱峰只用来找人耳听得出来、混在底噪里的声音（哼声、嗡声、音乐）：Welch 平均功率谱上的局部极大，
  两两至少隔 20 Hz（更近的只留高的那个），按电平降序取前 k 个
- 逐秒单音（`--tone`）：频率、带宽、参考带、算「在」的阈值全在 `screen_rules.json` 的 `tone_check`，
  代码里不写这些数；回答「这个单音只在开头那一秒，还是后面每一秒都有」
- 只读：不改任何文件，输出写 `--out` 指的新 jsonl

用法：
    python3 diagnose.py --root /workspace/train-raw --list candidates.txt --out diagnose.jsonl
    python3 diagnose.py ... --tone        # 多写一项逐秒单音
清单一行一个相对路径（相对 `--root`），`#` 开头是注释。
"""
import argparse
import json
import sys
from collections import Counter
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import screen  # noqa: E402  level_std_db 与 dBFS 下限的正本；清单读法也复用它

NPERSEG = 8192  # Welch 每段长度；片段更短就用整段
PEAK_MIN_SEP_HZ = 20.0  # 两个峰至少隔这么远，否则只留高的那个
FMIN_HZ, FMAX_HZ = 20.0, 8000.0
FLOOR_DBFS = 20.0 * np.log10(screen.DBFS_FLOOR)  # 与逐秒电平同一个下限，避免 -inf 写不进 JSON


def _x(x):
    return np.asarray(x, np.float64).ravel()


def per_second_dbfs(x, sr, sec=1.0):
    """逐秒 RMS 电平（dBFS）的列表；末尾不足一秒的片段丢掉，整个片段不足一秒就报错。

    整秒静音压到 -140 dBFS（`screen.DBFS_FLOOR`，与 level_std_db 同一个下限），避免 -inf 写不进 JSON。
    """
    x = _x(x)
    n = int(round(sec * sr))
    if n < 1:
        raise ValueError("采样率太低，切不出 1 秒的块")
    blocks = x.size // n
    if blocks < 1:
        raise ValueError("片段不足 1 秒，算不出逐秒电平")
    rms = np.sqrt(np.mean(x[: blocks * n].reshape(blocks, n) ** 2, axis=1))
    rms = np.maximum(rms, screen.DBFS_FLOOR)
    return [float(v) for v in 20.0 * np.log10(rms)]


def welch_power(x, sr, nperseg=NPERSEG):
    """Welch 平均功率谱（Hann 窗，50% 重叠）→ (频率 Hz, 功率)。

    功率的绝对标度没归一化：峰位只比相对大小，用不上绝对标度。
    """
    x = _x(x)
    n = min(int(nperseg), x.size)
    if n < 32:
        raise ValueError("片段太短，算不了频谱")
    win = np.hanning(n)
    step = max(1, n // 2)
    segs = [x[i:i + n] * win for i in range(0, x.size - n + 1, step)]
    if not segs:
        raise ValueError("片段太短，算不了频谱")
    power = np.mean([np.abs(np.fft.rfft(s)) ** 2 for s in segs], axis=0)
    return np.fft.rfftfreq(n, 1.0 / sr), power


def spectral_peaks(x, sr, k=3, fmin=FMIN_HZ, fmax=FMAX_HZ):
    """平均功率谱上前 k 个峰 → [(频率 Hz, 比中位数高多少 dB), …]，按电平降序。

    峰必须是局部极大（不比左右邻点低；平台只留其中一个），两两至少隔 `PEAK_MIN_SEP_HZ`。
    中位数在 [fmin, fmax] 这一段上取，所以第二个数读作「这个峰比这段频谱的典型水平高多少 dB」。
    """
    f, power = welch_power(x, sr)
    band = (f >= fmin) & (f <= fmax)
    if np.count_nonzero(band) < 3:
        raise ValueError(f"{fmin}–{fmax} Hz 里频点太少，算不出频谱峰")
    lvl = 10.0 * np.log10(np.maximum(power, 1e-300))
    median = float(np.median(lvl[band]))
    idx = [int(i) for i in np.flatnonzero(band)
           if 0 < i < lvl.size - 1 and lvl[i] >= lvl[i - 1] and lvl[i] >= lvl[i + 1]]
    idx.sort(key=lambda i: -lvl[i])
    picked = []
    for i in idx:
        hz = float(f[i])
        if all(abs(hz - p[0]) >= PEAK_MIN_SEP_HZ for p in picked):
            picked.append((hz, float(lvl[i] - median)))
        if len(picked) >= int(k):
            break
    return picked


def diagnose(x, sr, k=3, tone=None):
    """一条片段的诊断记录。不足 2 秒时 `level_std_db` 为 null（screen 算不出，不编一个数）。

    给了 `tone`（`load_tone_check()` 的那份判据）就多写一项逐秒单音。
    """
    x = _x(x)
    row = {"seconds": round(float(x.size) / float(sr), 3), "sr": int(sr),
           "per_second_dbfs": per_second_dbfs(x, sr),
           "peaks": spectral_peaks(x, sr, k=k)}
    try:
        row["level_std_db"] = screen.level_std_db(x, sr)
    except ValueError:
        row["level_std_db"] = None
    if tone is not None:
        rows = tone_per_second(x, sr, tone["hz"], tone["half_bw_hz"], tone["ref_band_hz"],
                               tone["ref_exclude_hz"])
        row["tone"] = {"hz": tone["hz"], "per_second": rows,
                       "verdict": tone_verdict(rows, tone["present_db"])}
    return row


def summary(rows, tone=False):
    """一行汇总：条数 · 逐秒电平标准差的中位数 · 最常见的峰频（按 1 Hz 取整，平局取低频）。

    `tone` 为真时（且每行带了单音判定）再多列三种结论各多少条。
    """
    stds = [r["level_std_db"] for r in rows if r["level_std_db"] is not None]
    med = f"{float(np.median(stds)):.2f} dB" if stds else "未量化（没有 2 秒以上的片段）"
    peaks = Counter(int(round(f)) for r in rows for f, _ in r["peaks"])
    if peaks:
        hz, n = min(peaks.items(), key=lambda kv: (-kv[1], kv[0]))
        top = f"{hz} Hz（{n} 次）"
    else:
        top = "未量化（没有峰）"
    line = f"共 {len(rows)} 条；逐秒电平标准差中位数 {med}；最常见的频谱峰 {top}"
    if tone:
        where = Counter(r["tone"]["verdict"]["where"] for r in rows if "tone" in r)
        parts = " · ".join(f"{w} {where.get(w, 0)} 条"
                           for w in ("none", "first_only", "later_too"))
        line += f"；单音逐秒判定 {parts}"
    return line


def load_tone_check(path=HERE / "screen_rules.json"):
    """单音判据（频率 · 带宽 · 参考带 · 算「在」要高出多少 dB）：正本在 json，代码里不写这些数。"""
    return json.loads(Path(path).read_text(encoding="utf-8"))["tone_check"]


def _sin_power(block, win):
    """Hann 窗 rfft 的逐频点正弦等效功率：2|X_k|² / (N·Σw²)。

    推导：窗内正弦（幅度 A）的窗能量 P = A²/2·Σw²；单边谱合计 Σ|X_k|² ≈ N·P/2（共轭对称）。
    按 N·Σw²/2 归一后，全部频点功率合计 = A²/2。
    满幅正弦（幅度 1.0）因此读 0.5，也就是 -3.01 dBFS。
    """
    return 2.0 * np.abs(np.fft.rfft(block * win)) ** 2 / (np.sum(win ** 2) * win.size)


def tone_per_second(x, sr, hz, half_bw_hz, ref_band_hz, ref_exclude_hz, sec=1.0):
    """逐秒量单音有多强：整秒块做 Hann 窗 rfft（1 秒块就是 1 Hz 一根频点）。

    每秒一行：`tone_dbfs` 是单音带内功率合计（满幅正弦 ≈ -3 dBFS，静音压到 `FLOOR_DBFS`），
    `prominence_db` 是带内平均频点功率比参考带中位数高多少 dB（参考带里离单音太近的频点不算，
    那一圈是窗的主瓣泄能）。末尾不足一秒的块丢掉，整个片段不足一秒就报错。
    """
    x = _x(x)
    n = int(round(sec * sr))
    if n < 1:
        raise ValueError("采样率太低，切不出 1 秒的块")
    blocks = x.size // n
    if blocks < 1:
        raise ValueError("片段不足 1 秒，算不出逐秒单音")
    win = np.hanning(n)
    f = np.fft.rfftfreq(n, 1.0 / sr)
    tone = np.abs(f - hz) <= half_bw_hz
    if np.count_nonzero(tone) < 1:
        raise ValueError(f"单音带 {hz}±{half_bw_hz} Hz 里一根频点都没有")
    ref = (f >= float(ref_band_hz[0])) & (f <= float(ref_band_hz[1])) & (np.abs(f - hz) > ref_exclude_hz)
    if np.count_nonzero(ref) < 1:
        raise ValueError(f"参考带 {ref_band_hz[0]}–{ref_band_hz[1]} Hz 里没有可用频点")
    rows = []
    for b in range(blocks):
        power = _sin_power(x[b * n:(b + 1) * n], win)
        tone_power = float(np.sum(power[tone]))
        tone_dbfs = 10.0 * np.log10(max(tone_power, 1e-300))
        prom = (10.0 * np.log10(max(float(np.mean(power[tone])), 1e-300))
                - 10.0 * np.log10(max(float(np.median(power[ref])), 1e-300)))
        rows.append({"tone_dbfs": float(max(tone_dbfs, FLOOR_DBFS)),
                     "prominence_db": float(prom)})
    return rows


def tone_verdict(rows, present_db):
    """单音在哪些秒出现：`none` 一秒都不在 · `first_only` 正好只在第 1 秒 ·
    `later_too` 第 2 秒及以后也有。秒号从 1 起。"""
    present = [i + 1 for i, row in enumerate(rows) if row["prominence_db"] >= present_db]
    if not present:
        where = "none"
    elif present == [1]:
        where = "first_only"
    else:
        where = "later_too"
    return {"present_seconds": present, "where": where}


def run(args):
    rows = []
    root = Path(args.root)
    tone = load_tone_check() if args.tone else None
    with open(args.out, "w", encoding="utf-8") as out:
        for rel in screen.read_list(args.list):
            x, sr = screen.read_first_channel(root / rel)
            row = {"path": rel, **diagnose(x, sr, tone=tone)}
            out.write(json.dumps(row, ensure_ascii=False) + "\n")
            rows.append(row)
    print(summary(rows, tone=args.tone))


def main(argv=None):
    ap = argparse.ArgumentParser(description="候选噪声片段的诊断（只读：逐秒电平 · 频谱峰 · 平稳度 · 单音）")
    ap.add_argument("--root", required=True, help="片段所在目录；清单里的路径相对它")
    ap.add_argument("--list", required=True, help="候选清单，一行一个相对路径，# 开头是注释")
    ap.add_argument("--out", required=True, help="输出 jsonl，一条片段一行")
    ap.add_argument("--tone", action="store_true",
                    help="多写一项逐秒单音（频率 · 带宽 · 判据取 screen_rules.json 的 tone_check）")
    run(ap.parse_args(argv))


if __name__ == "__main__":
    main()
