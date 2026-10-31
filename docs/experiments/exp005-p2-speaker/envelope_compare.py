"""exp005 附：在 exp004 的同一批 12 段上，比「快开慢关」能量版（p1）与 VAD 版（本 crate）。

输入：exp004 重生成的素材与 Rust GTCRN 原样输出（<主目录>/_tmp/20261030-exp004/{src,rs/raw}），一行不改
指标（与 exp004 README「实测效果」同口径）：
- 判为有声（含 hangover）的 hop 比例：全段 / 目标说话人停顿里 / 目标说话人说话时（停顿 = exp003 `vad_mask` 按干净参考判的无声）
- 均值 SI-SDR（对干净参考）：只开下限（−12 dB）· 能量版 · VAD 版
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp005-p2-speaker/envelope_compare.py <vdn-speaker-lab 可执行文件>
输出：results/envelope_compare.json
"""
import json
import subprocess
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
from _common import vad_mask  # noqa: E402
from _paths import MAIN  # noqa: E402

E4 = MAIN / "_tmp/20261030-exp004"
TMP = MAIN / "_tmp/20261031-exp005/env_cmp"
HOP = 256


def si_sdr(est, ref):
    ref = ref - ref.mean()
    est = est - est.mean()
    a = np.dot(est, ref) / (np.dot(ref, ref) + 1e-12)
    t = a * ref
    return float(10 * np.log10(np.sum(t ** 2) / (np.sum((est - t) ** 2) + 1e-12)))


def main():
    exe = sys.argv[1]
    TMP.mkdir(parents=True, exist_ok=True)
    clips = json.loads((E4 / "clips.json").read_text())["clips"]
    rows = []
    for c in clips:
        cid = c["id"]
        noisy = E4 / "src" / f"{cid}_noisy.wav"
        wet = E4 / "rs/raw" / f"{cid}_gtcrn.wav"
        cleanp = E4 / "src" / f"{cid}_clean.wav"
        if not wet.exists():
            continue
        pre = TMP / cid
        subprocess.run([exe, "envelope", str(noisy), str(wet), str(pre)], check=True)
        j = json.loads(Path(f"{pre}.env.json").read_text())
        n = len(j["vad_open"]) * HOP
        x = sf.read(str(noisy), dtype="float32")[0][:n]
        w = sf.read(str(wet), dtype="float32")[0][:n]
        floor = (1 - 10 ** (-12 / 20)) * w + 10 ** (-12 / 20) * x
        r = {"id": cid, "desc": c.get("desc", "")}
        for k in ("energy", "vad"):
            r[f"open_{k}"] = float(np.mean(j[f"{k}_open"]))
        if cleanp.exists():
            ref = sf.read(str(cleanp), dtype="float32")[0][:n]
            m = vad_mask(ref)[0]
            m = np.pad(m, (0, n))[:n]
            hop_pause = ~m.reshape(-1, HOP).any(axis=1)
            for k in ("energy", "vad"):
                o = np.array(j[f"{k}_open"], bool)
                r[f"open_{k}_pause"] = float(o[hop_pause].mean()) if hop_pause.any() else None
                r[f"open_{k}_speech"] = float(o[~hop_pause].mean())
            r["sisdr_floor"] = si_sdr(floor, ref)
            for k in ("energy", "vad"):
                r[f"sisdr_{k}"] = si_sdr(sf.read(f"{pre}.{k}.wav", dtype="float32")[0][:n], ref)
        rows.append(r)
        print(cid, {k: round(v, 3) for k, v in r.items() if isinstance(v, float)})
    keys = [k for k in rows[0] if k.startswith(("open", "sisdr"))]
    summ = {k: float(np.mean([r[k] for r in rows if r.get(k) is not None])) for k in keys}
    out = {"source": "exp004 src + rs/raw (GTCRN)", "params": "open −6 dB · floor −12 dB · hangover 150 ms · release 300 ms · VAD 滞回 0.5/0.35",
           "per_clip": rows, "mean": summ}
    (HERE / "results/envelope_compare.json").write_text(json.dumps(out, ensure_ascii=False, indent=1) + "\n")
    print("mean", {k: round(v, 3) for k, v in summ.items()})


if __name__ == "__main__":
    main()
