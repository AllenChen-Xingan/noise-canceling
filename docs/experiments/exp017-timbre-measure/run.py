"""exp017：对干净人声做已知劣化，三种量法各读一遍。

用法：python -I run.py <exp013 build 目录> <输出目录>
只读 exp013 正式 build 的 owner.wav（干净参考）；劣化音频写进输出目录（主目录 _tmp/026/<run-id>/），
读数写 readings.csv。模拟的是「干净人声单独过一遍」那一路：输出 = 劣化(参考)。
"""

import csv
import hashlib
import json
import sys
from pathlib import Path

import numpy as np
import soundfile as sf
from scipy import signal

sys.path.insert(0, str(Path(__file__).resolve().parent))
import measures as M  # noqa: E402

SR = 16000
SEED = 17
# 写出音频、供 Rust 对拍（tests/timbre_parity.rs）的劣化
PARITY = ("ref", "lp4k", "hs+6", "ls+6", "notch1-2k-12", "white20")


def lowpass(x, fc):
    return signal.sosfiltfilt(signal.butter(8, fc, "low", fs=SR, output="sos"), x)


def shelf(x, fc, gain_db, kind):
    """RBJ 搁架滤波（S = 1）。"""
    a = 10 ** (gain_db / 40)
    w = 2 * np.pi * fc / SR
    al = np.sin(w) / 2 * np.sqrt(2)
    cw, sa = np.cos(w), 2 * np.sqrt(a) * al
    if kind == "high":
        b = [a * ((a + 1) + (a - 1) * cw + sa), -2 * a * ((a - 1) + (a + 1) * cw), a * ((a + 1) + (a - 1) * cw - sa)]
        d = [(a + 1) - (a - 1) * cw + sa, 2 * ((a - 1) - (a + 1) * cw), (a + 1) - (a - 1) * cw - sa]
    else:
        b = [a * ((a + 1) - (a - 1) * cw + sa), 2 * a * ((a - 1) - (a + 1) * cw), a * ((a + 1) - (a - 1) * cw - sa)]
        d = [(a + 1) + (a - 1) * cw + sa, -2 * ((a - 1) + (a + 1) * cw), (a + 1) + (a - 1) * cw - sa]
    return signal.lfilter(b, d, x)


def notch(x, lo, hi, depth_db):
    bp = signal.sosfiltfilt(signal.butter(4, [lo, hi], "band", fs=SR, output="sos"), x)
    return x + (10 ** (depth_db / 20) - 1) * bp


def tilt(x, db_per_oct):
    """整体谱倾斜：以 1 kHz 为轴，每倍频程 db_per_oct dB（零相位，FFT 域）。"""
    f = np.fft.rfftfreq(len(x), 1 / SR)
    g = 10 ** (db_per_oct * np.log2(np.maximum(f, 50) / 1000) / 20)
    return np.fft.irfft(np.fft.rfft(x) * g, len(x))


def add_noise(x, owner, snr_db, kind, rng):
    n = rng.standard_normal(len(x))
    if kind == "pink":
        f = np.fft.rfftfreq(len(x), 1 / SR)
        sp = np.fft.rfft(n) / np.sqrt(np.maximum(f, 20))
        n = np.fft.irfft(sp, len(x))
    act = np.repeat(owner, M.MASK_FRAME)[: len(x)]
    ps = np.mean(x[: len(act)][act] ** 2)
    return x + n * np.sqrt(ps / np.mean(n**2) / 10 ** (snr_db / 10))


DEGS = [("ref", lambda x, o, r: x)]
DEGS += [(f"gain{g:+d}", (lambda g: lambda x, o, r: x * 10 ** (g / 20))(g)) for g in (-10, 6)]
DEGS += [(f"lp{f // 1000}k", (lambda f: lambda x, o, r: lowpass(x, f))(f)) for f in (4000, 2000, 1000)]
DEGS += [(f"hs{g:+d}", (lambda g: lambda x, o, r: shelf(x, 3000, g, "high"))(g)) for g in (-6, -3, 3, 6)]
DEGS += [(f"ls{g:+d}", (lambda g: lambda x, o, r: shelf(x, 250, g, "low"))(g)) for g in (-6, 6)]
DEGS += [(f"notch2-4k{d:d}", (lambda d: lambda x, o, r: notch(x, 2000, 4000, d))(d)) for d in (-6, -12)]
DEGS += [(f"notch{lo / 1000:g}-{hi / 1000:g}k-12", (lambda lo, hi: lambda x, o, r: notch(x, lo, hi, -12))(lo, hi)) for lo, hi in ((500, 1000), (1000, 2000))]
DEGS += [("peak2-4k+6", lambda x, o, r: notch(x, 2000, 4000, 6))]
DEGS += [(f"tilt{t:+g}", (lambda t: lambda x, o, r: tilt(x, t))(t)) for t in (-2, -1)]
DEGS += [(f"{k}{s}", (lambda k, s: lambda x, o, r: add_noise(x, o, s, k, r))(k, s)) for k in ("white", "pink") for s in (30, 20, 10)]


def clips(build):
    seen, out = set(), []
    for d in sorted(Path(build).iterdir()):
        scene, dev = d.name.rsplit("-", 2)[0], d.name.rsplit("-", 1)[1]
        if (scene, dev) not in seen and (d / "owner.wav").exists():
            seen.add((scene, dev))
            out.append(d)
    return out


def main(build, outdir):
    outdir = Path(outdir)
    outdir.mkdir(parents=True, exist_ok=False)
    rng = np.random.default_rng(SEED)
    rows, hashes = [], {}
    for d in clips(build):
        ref, sr = sf.read(d / "owner.wav", dtype="float64")
        assert sr == SR
        hashes[d.name] = hashlib.sha256((d / "owner.wav").read_bytes()).hexdigest()
        owner = M.truth_from_clean(ref)
        noise_only = ~M.dilate(owner, 3)
        for name, fn in DEGS:
            y = fn(ref, owner, rng)
            if name in PARITY:
                sf.write(outdir / f"{d.name}__{name}.wav", y.astype(np.float32), SR, subtype="FLOAT")
            code, cdev = M.code_timbre(y, ref, owner)
            code_n, _ = M.code_timbre(y, ref, owner, noise_only)
            doc, ddev = M.doc_timbre(y, ref, owner)
            tdev, _ = M.third_oct_dev(y, ref, owner)
            rows.append(dict(
                clip=d.name, deg=name,
                code_max=abs(code), code_signed=code, code_noisy_path=abs(code_n),
                doc_rms=doc, third_rms=float(np.sqrt(np.mean(tdev**2))), third_max=float(np.max(np.abs(tdev))),
                **{f"code_b{i}": v for i, v in enumerate(cdev)},
                **{f"doc_b{i}": v for i, v in enumerate(ddev)},
            ))
    with open(outdir / "readings.csv", "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)
    (outdir / "inputs.json").write_text(json.dumps({"seed": SEED, "owner_sha256": hashes}, indent=1))
    print(f"{len(hashes)} clips × {len(DEGS)} degradations → {outdir / 'readings.csv'}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
