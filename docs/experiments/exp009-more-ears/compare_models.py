"""exp009 · 作者听过的 12 段里，GTCRN 和 DPDFNet-2（及 FastEnhancer）在信号上差在哪。纯信号计算，不跑模型。

输入：regen_exp003.py 生成的 _tmp/20261031-exp009/exp003-regen/raw/（模型原始输出，未做响度统一）与 src/cNN_clean.wav（干净参考，
各模型与参考的延迟都为 0，见 align.json）。16 kHz，帧长 32 ms、帧移 16 ms。
帧分类（按干净参考）：说话帧 = 能量高于本段最大帧 −35 dB；停顿帧 = 低于 −50 dB。
量（都与整体音量无关，作者听的是响度统一后的版本）：
  pause_vs_noisy_db   停顿处残留噪声比原声低多少 dB（越负 = 背景压得越狠）
  contrast_db         同一输出里「说话帧电平 − 停顿帧电平」（越大 = 背景越安静）
  si_sdr_db           与干净参考的尺度无关信噪比（越大 = 越接近干净人声，残留噪声与人声损伤都扣分）
  hf_ratio_db         说话帧里 4–8 kHz 能量占比，相对干净参考（负 = 高频被削，声音发闷）
  lf_ratio_db         说话帧里 < 300 Hz 能量占比，相对干净参考
  lsd_speech_db       说话帧对数谱距离（先按最小二乘对齐增益；越小 = 人声音色越接近）
输出：results/compare_models.json
用法：<主目录>/_tmp/exp009-venv/bin/python docs/experiments/exp009-more-ears/compare_models.py
"""
import json
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
R = ROOT / "_tmp/20261031-exp009/exp003-regen"
MODELS = ["noisy", "dpdfnet2", "gtcrn", "fastenhancer_m"]
SR, N, H = 16000, 512, 256


def frames(x):
    n = 1 + (len(x) - N) // H
    idx = np.arange(N)[None, :] + H * np.arange(n)[:, None]
    return x[idx] * np.hanning(N)


def spec(x):
    return np.abs(np.fft.rfft(frames(x), axis=1)) ** 2


def db(v):
    return 10 * np.log10(np.maximum(v, 1e-20))


def si_sdr(y, s):
    a = np.dot(y, s) / np.dot(s, s)
    e = y - a * s
    return float(db(np.sum((a * s) ** 2) / np.sum(e ** 2)))


def measure(y, s, noisy):
    L = min(len(y), len(s), len(noisy))
    y, s, noisy = y[:L], s[:L], noisy[:L]
    Es = frames(s).__pow__(2).sum(1)
    top = db(Es.max())
    sp, pa = db(Es) > top - 35, db(Es) < top - 50
    Ey, En = frames(y).__pow__(2).sum(1), frames(noisy).__pow__(2).sum(1)
    Y, S = spec(y), spec(s)
    f = np.fft.rfftfreq(N, 1 / SR)
    hf, lf = (f >= 4000) & (f <= 8000), f < 300

    def ratio(P, band):
        return db(P[sp][:, band].sum() / P[sp].sum())

    g = np.sum(Y[sp] * S[sp]) / np.sum(Y[sp] ** 2)  # 增益对齐
    lsd = np.sqrt(np.mean((db(g * Y[sp] + 1e-12) - db(S[sp] + 1e-12)) ** 2, axis=1)).mean()
    return {
        "pause_vs_noisy_db": round(float(db(Ey[pa].mean()) - db(En[pa].mean())), 2) if pa.any() else None,
        "contrast_db": round(float(db(Ey[sp].mean()) - db(Ey[pa].mean())), 2) if pa.any() else None,
        "si_sdr_db": round(si_sdr(y, s), 2),
        "hf_ratio_db": round(float(ratio(Y, hf) - ratio(S, hf)), 2),
        "lf_ratio_db": round(float(ratio(Y, lf) - ratio(S, lf)), 2),
        "lsd_speech_db": round(float(lsd), 2),
        "n_speech_frames": int(sp.sum()), "n_pause_frames": int(pa.sum()),
    }


def main():
    clips = json.loads((HERE.parent / "exp003-offline-ab/clips.json").read_text())["clips"]
    out = {"per_clip": {}, "median": {}, "gtcrn_minus_dpdfnet2": {}}
    for c in clips:
        cid = c["id"]
        s, sr = sf.read(R / "src" / f"{cid}_clean.wav", dtype="float64")
        assert sr == SR
        noisy, _ = sf.read(R / "raw" / f"{cid}_noisy.wav", dtype="float64")
        out["per_clip"][cid] = {"desc": c["desc"], "lang": c.get("lang")}
        for m in MODELS:
            y, sr2 = sf.read(R / "raw" / f"{cid}_{m}.wav", dtype="float64")
            assert sr2 == SR
            out["per_clip"][cid][m] = measure(y, s, noisy)
    keys = ["pause_vs_noisy_db", "contrast_db", "si_sdr_db", "hf_ratio_db", "lf_ratio_db", "lsd_speech_db"]
    for m in MODELS:
        out["median"][m] = {k: round(float(np.median([v[m][k] for v in out["per_clip"].values() if v[m][k] is not None])), 2) for k in keys}
    for k in keys:
        d = [v["gtcrn"][k] - v["dpdfnet2"][k] for v in out["per_clip"].values() if v["gtcrn"][k] is not None and v["dpdfnet2"][k] is not None]
        out["gtcrn_minus_dpdfnet2"][k] = {"median": round(float(np.median(d)), 2), "n_gtcrn_higher": int(sum(x > 0 for x in d)), "n": len(d)}
    (HERE / "results/compare_models.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps({"median": out["median"], "gtcrn_minus_dpdfnet2": out["gtcrn_minus_dpdfnet2"]}, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
