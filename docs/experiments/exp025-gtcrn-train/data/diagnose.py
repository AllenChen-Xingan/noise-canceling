"""候选噪声片段的诊断（只读）：逐秒电平 · 频谱峰 · 平稳度。回答「这段底噪为什么不平稳 / 有没有哼声」。

- 平稳度不另写一份：`level_std_db` 直接调 `screen.level_std_db`，与筛查用同一个判据、同一个下限
- 频谱峰只用来找人耳听得出来、混在底噪里的声音（哼声、嗡声、音乐）：Welch 平均功率谱上的局部极大，
  两两至少隔 20 Hz（更近的只留高的那个），按电平降序取前 k 个
- 只读：不改任何文件，输出写 `--out` 指的新 jsonl

用法：
    python3 diagnose.py --root /workspace/train-raw --list candidates.txt --out diagnose.jsonl
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


def diagnose(x, sr, k=3):
    """一条片段的诊断记录。不足 2 秒时 `level_std_db` 为 null（screen 算不出，不编一个数）。"""
    x = _x(x)
    row = {"seconds": round(float(x.size) / float(sr), 3), "sr": int(sr),
           "per_second_dbfs": per_second_dbfs(x, sr),
           "peaks": spectral_peaks(x, sr, k=k)}
    try:
        row["level_std_db"] = screen.level_std_db(x, sr)
    except ValueError:
        row["level_std_db"] = None
    return row


def summary(rows):
    """一行汇总：条数 · 逐秒电平标准差的中位数 · 最常见的峰频（按 1 Hz 取整，平局取低频）。"""
    stds = [r["level_std_db"] for r in rows if r["level_std_db"] is not None]
    med = f"{float(np.median(stds)):.2f} dB" if stds else "未量化（没有 2 秒以上的片段）"
    peaks = Counter(int(round(f)) for r in rows for f, _ in r["peaks"])
    if peaks:
        hz, n = min(peaks.items(), key=lambda kv: (-kv[1], kv[0]))
        top = f"{hz} Hz（{n} 次）"
    else:
        top = "未量化（没有峰）"
    return f"共 {len(rows)} 条；逐秒电平标准差中位数 {med}；最常见的频谱峰 {top}"


def run(args):
    rows = []
    root = Path(args.root)
    with open(args.out, "w", encoding="utf-8") as out:
        for rel in screen.read_list(args.list):
            x, sr = screen.read_first_channel(root / rel)
            row = {"path": rel, **diagnose(x, sr)}
            out.write(json.dumps(row, ensure_ascii=False) + "\n")
            rows.append(row)
    print(summary(rows))


def main(argv=None):
    ap = argparse.ArgumentParser(description="候选噪声片段的诊断（只读：逐秒电平 · 频谱峰 · 平稳度）")
    ap.add_argument("--root", required=True, help="片段所在目录；清单里的路径相对它")
    ap.add_argument("--list", required=True, help="候选清单，一行一个相对路径，# 开头是注释")
    ap.add_argument("--out", required=True, help="输出 jsonl，一条片段一行")
    run(ap.parse_args(argv))


if __name__ == "__main__":
    main()
