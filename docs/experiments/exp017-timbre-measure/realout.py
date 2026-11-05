"""exp017 旁证：在 GTCRN 真实输出（带噪那一路）上，代码版与 ERB 候选各读一遍。

用法：python -I realout.py <exp013 build 目录> <成绩单 run 的 qa/asr-wav 目录> <输出 csv>
素材是 exp014 v2 run 给识别服务存下的 16 位 WAV（带噪输入过 GTCRN 的输出，不是干净人声那一路）。
停顿掩码 = 主讲人与旁人（各按干净轨求区间）都不在、外扩 30 ms，同 runner.rs 的 guard。
"""

import csv
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

sys.path.insert(0, str(Path(__file__).resolve().parent))
import measures as M  # noqa: E402


def main(build, asr_dir, out_csv):
    rows = []
    for p in sorted(Path(asr_dir).glob("*-gtcrn-*.wav")):
        clip = p.name.split("#")[0]
        d = Path(build) / clip
        ref, sr = sf.read(d / "owner.wav", dtype="float64")
        y, sr2 = sf.read(p, dtype="float64")
        assert sr == sr2 == 16000
        n = min(len(ref), len(y))
        ref, y = ref[:n], y[:n]
        owner = M.truth_from_clean(ref)
        talk = owner.copy()
        if (d / "interf.wav").exists():
            itf, _ = sf.read(d / "interf.wav", dtype="float64")
            if np.any(itf):
                talk |= M.truth_from_clean(itf[:n])
        noise_only = ~M.dilate(talk, 3)
        code, _ = M.code_timbre(y, ref, owner, noise_only)
        erb, _ = M.erb_shape(y, ref, owner, noise_only)
        rows.append(dict(clip=clip, code_max=abs(code), erb_std=erb["std"], erb_slope=erb["slope"], erb_max=erb["max"]))
    with open(out_csv, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    a = {k: np.array([r[k] for r in rows]) for k in ("code_max", "erb_std")}
    print(f"{len(rows)} 段")
    for k, v in a.items():
        print(k, "中位数", round(float(np.median(v)), 2), "p90", round(float(np.percentile(v, 90)), 2))
    print("相关（Spearman 近似：秩的皮尔逊）", round(float(np.corrcoef(np.argsort(np.argsort(a["code_max"])), np.argsort(np.argsort(a["erb_std"])))[0, 1]), 2))


if __name__ == "__main__":
    main(*sys.argv[1:4])
