"""exp016 咔哒诊断：V-CLICK 检出的点是「真咔哒」还是「残留 / 穿过来的声音的陡沿」。只读，不改评分。

V-CLICK 的 Python 复刻，照正本 `src/eval/src/verifiers/speech.rs` 的 `click_positions` 写。
它是第二处表示，所以有两道机器比对：
- 参数 k · resid_frac · floor_db 从 `src/eval/src/thresholds/table.rs` 读出来比（`params_from_rust`，测试 `tests/test_click_diag.py`）
- 每段的检出个数必须等于成绩单 `clips.jsonl` 里 V-CLICK 的 `count`，不等就退出（`main`）

输出 = 评测引擎打分时的那份输出：成绩单 run 目录 `qa/asr-wav/<片段>#…-<系统>-….wav`（浮点），
不是外部离线原始输出（评测引擎后面还接了一级保护处理，两者最大差 0.30，实测）。

用法：python click_diag.py <主目录> <清单 clips.jsonl> <片段,…> --run <系统>=<run 目录>:<clips.jsonl> … --out <新目录>
"""
from __future__ import annotations

import argparse
import glob
import json
import re
from pathlib import Path

import numpy as np
import soundfile as sf

SR = 16000
HERE = Path(__file__).resolve().parent
TABLE = HERE.parents[2] / "src/eval/src/thresholds/table.rs"
K, RESID_FRAC, FLOOR_DB = 8.0, 0.5, -34.0


def params_from_rust(text: str) -> dict[str, float]:
    """从 table.rs 的 `pub const CLICK` 块里取 p("名", Some(值), …)。"""
    blk = text[text.index("pub const CLICK"):]
    blk = blk[:blk.index("};")]
    return {m[0]: float(m[1]) for m in re.findall(r'p\(\s*"(\w+)",\s*Some\((-?[\d.]+)\)', blk)}


def outliers(x: np.ndarray, k: float) -> np.ndarray:
    d = np.abs(np.diff(x.astype(np.float32))).astype(np.float64)
    blk, f = int(0.02 * SR), np.zeros(len(d), bool)
    for s in range(0, len(d), blk):
        c = d[s:s + blk]
        med = np.quantile(c, .5)
        mad = np.quantile(np.abs(c - med), .5)
        f[s:s + blk] = c > med + k * max(mad, 1e-6)
    return f


def residual(x: np.ndarray, clean: np.ndarray) -> np.ndarray:
    """输出减去「逐 5 ms 最小二乘增益、块中心间线性插值 × 干净轨」。"""
    blk = int(0.005 * SR)
    n = min(len(x), len(clean))
    nb = n // blk
    if nb == 0:
        return x.copy()
    xs = x[:nb * blk].astype(np.float64).reshape(nb, blk)
    cs = clean[:nb * blk].astype(np.float64).reshape(nb, blk)
    cc = (cs * cs).sum(1)
    g = np.where(cc > 0, (xs * cs).sum(1) / np.where(cc > 0, cc, 1), 0.0)
    t = np.arange(n)
    b0 = np.clip(np.floor((t - blk / 2) / blk), 0, nb - 1).astype(int)
    b1 = np.minimum(b0 + 1, nb - 1)
    w = np.where(b1 == b0, 0.0, np.clip((t - (b0 * blk + blk / 2)) / blk, 0, 1))
    r = x.astype(np.float32).copy()
    r[:n] = r[:n] - ((g[b0] * (1 - w) + g[b1] * w) * clean[:n]).astype(np.float32)
    return r


def click_positions(out, mix, clean, bg) -> list[int]:
    peak = np.quantile(np.abs(clean.astype(np.float64)), .999)
    floor = np.float32(peak * 10 ** (FLOOR_DB / 20))
    r = residual(out, clean)
    fo, fm = outliers(out, K), outliers(mix, K) | outliers(bg, K)
    g, pos, last = int(0.002 * SR), [], None
    for i in np.nonzero(fo)[0]:
        if i + 1 >= len(r):
            continue
        do, dr = abs(out[i + 1] - out[i]), abs(r[i + 1] - r[i])
        if do < floor or dr < np.float32(RESID_FRAC) * do:
            continue
        if fm[max(i - g, 0):min(i + g + 1, len(fm))].any():
            continue
        if last is None or i > last + 5 * g // 2:
            pos.append(int(i))
        last = i
    return pos


def describe(i: int, out: np.ndarray, iv) -> dict:
    """一处检出的样子：跳变 ÷ 前后 ±20 ms（去掉 ±1 ms）一阶差分的均方根；落不落在主讲人说话段；在 10 ms 帧里的相位。"""
    dd = np.abs(np.diff(out))
    w = np.concatenate([dd[max(i - 320, 0):max(i - 16, 0)], dd[i + 16:i + 320]])
    return {"t_s": round(i / SR, 3), "jump": float(dd[i]), "ratio": float(dd[i] / np.sqrt(np.mean(w ** 2))),
            "voiced": any(s <= i < e for s, e in iv), "phase160": i % 160}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("main_root", type=Path)
    ap.add_argument("manifest", type=Path)
    ap.add_argument("clips")
    ap.add_argument("--run", action="append", required=True, help="<系统>=<run 目录>:<clips.jsonl>")
    ap.add_argument("--out", type=Path, required=True)
    a = ap.parse_args()
    got = params_from_rust(TABLE.read_text())
    if (got["k"], got["resid_frac"], got["floor_db"]) != (K, RESID_FRAC, FLOOR_DB):
        raise SystemExit(f"参数与 table.rs 不一致：{got}")
    if a.out.exists():
        raise SystemExit(f"{a.out} 已存在，不覆盖")
    a.out.mkdir(parents=True)
    man = {json.loads(l)["id"]: json.loads(l) for l in a.manifest.open()}
    rd = lambda p: sf.read(p, dtype="float32")[0]
    rows, figs = [], []
    for spec in a.run:
        sysn, rest = spec.split("=", 1)
        rdir, cj = rest.split(":", 1)
        want = {}
        for l in open(cj):
            r = json.loads(l)
            if r["system"] == sysn:
                want[r["clip"]] = next(int(s["extra"]["count"]) for s in r["scores"] if s["v"] == "V-CLICK")
        for cid in a.clips.split(","):
            au = man[cid]["audio"]
            mix, clean = rd(a.main_root / au["mix"]), rd(a.main_root / au["owner"])
            bg = rd(a.main_root / au["noise"]) + rd(a.main_root / au["interf"])
            out = rd(glob.glob(f"{rdir}/qa/asr-wav/{cid}#*-{sysn}-*.wav")[0])
            pos = click_positions(out, mix, clean, bg)
            if len(pos) != want[cid]:
                raise SystemExit(f"{sysn} {cid}：复刻检出 {len(pos)} 个，成绩单 {want[cid]} 个，复刻与正本不一致")
            iv = [t for tg in man[cid]["targets"] for t in tg["iv"]]
            rows += [{"clip": cid, "system": sysn, **describe(i, out, iv)} for i in pos]
            dd = np.abs(np.diff(out))
            figs += [(sysn, cid, i, out, mix, clean) for i in sorted(pos, key=lambda i: -dd[i])[:2]]
    summ = {}
    for s in sorted({r["system"] for r in rows}):
        q = np.array([r["ratio"] for r in rows if r["system"] == s])
        ph = [r["phase160"] for r in rows if r["system"] == s]
        summ[s] = {"n": len(q), "voiced": sum(r["voiced"] for r in rows if r["system"] == s),
                   "ratio_median": round(float(np.median(q)), 2), "ratio_p10_p90": np.round(np.quantile(q, [.1, .9]), 2).tolist(),
                   "ratio_gt_20": int((q > 20).sum()), "phase_hist8": np.histogram(ph, bins=8, range=(0, 160))[0].tolist()}
    (a.out / "clicks.json").write_text(json.dumps({"summary": summ, "rows": rows}, ensure_ascii=False, indent=1) + "\n")
    print(json.dumps(summ, ensure_ascii=False))
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        return
    figs = [f for f in figs if f[0] == figs[0][0]]
    fig, ax = plt.subplots(len(figs), 1, figsize=(10, 2.4 * len(figs)))
    for k, (sysn, cid, i, out, mix, clean) in enumerate(figs):
        lo, hi = max(i - 480, 0), i + 480
        t = (np.arange(lo, hi) - i) / SR * 1000
        ax[k].plot(t, mix[lo:hi], color="0.75", lw=.6, label="mix")
        ax[k].plot(t, clean[lo:hi], color="g", lw=.6, label="clean")
        ax[k].plot(t, out[lo:hi], color="r", lw=.8, label=f"{sysn} out")
        ax[k].axvline(0, color="k", ls=":")
        ax[k].set_title(f"{cid} @ {i / SR:.3f}s (ms relative to click)", fontsize=8)
        ax[k].legend(fontsize=6, loc="upper right")
    plt.tight_layout()
    plt.savefig(a.out / "waveforms.png", dpi=90)


if __name__ == "__main__":
    main()
