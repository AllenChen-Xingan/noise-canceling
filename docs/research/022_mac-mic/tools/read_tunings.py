"""只读解码 macOS 内置麦调音文件里的标准模式（general）波束参数。

用途：给 022 `findings/tunings.md` 第三节出表。只读 /System/Library/Audio/Tunings，
不开麦、不改设置。用法：python3 -I read_tunings.py [调音目录]
"""
import glob
import math
import os
import plistlib
import struct
import sys

ROOT = "/System/Library/Audio/Tunings"
BIN_HZ = 50  # 481 点覆盖 0–24 kHz，每点 50 Hz
REPORT_HZ = (200, 500, 1000, 2000, 4000, 8000)


def f32_be(data):
    """plist 里的系数是大端 float32。"""
    if len(data) % 4:
        raise ValueError("长度不是 4 的倍数：%d" % len(data))
    return struct.unpack(">%df" % (len(data) // 4), data)


def db20(x):
    return 20.0 * math.log10(max(x, 1e-12))


def beam_stats(beam, hz):
    """返回 (范数 dB, 同相增益/范数 dB)；范数 = 波束器对各麦独立自噪声的放大。"""
    k = int(round(hz / BIN_HZ))
    mics = sorted(beam["RealCoefficients"])
    re = [f32_be(beam["RealCoefficients"][m])[k] for m in mics]
    im = [f32_be(beam["ImagCoefficients"][m])[k] for m in mics]
    norm = math.sqrt(sum(r * r + i * i for r, i in zip(re, im)))
    coherent = math.hypot(sum(re), sum(im))
    return db20(norm), db20(coherent / max(norm, 1e-12) / math.sqrt(len(mics)))


def graph_units(path):
    """列出 .dspg 里的苹果处理单元四字码，如 dccb、bmfd、sens、tmst。"""
    units = []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            parts = line.replace("(", " ").replace(")", " ").split()
            if len(parts) >= 5 and parts[0] == "box" and parts[2].startswith("au"):
                units.append(parts[3])
    return units


def aid_number(path):
    seg = [s for s in path.split(os.sep) if s.startswith("AID")][0]
    return int(seg[3:])


def scan(root=ROOT):
    rows = []
    pattern = os.path.join(root, "AID*", "DSP", "Strips",
                           "builtin_microphone_general_aufx_bmfd.plist")
    for p in sorted(glob.glob(pattern), key=aid_number):
        with open(p, "rb") as fh:
            d = plistlib.load(fh)
        beam = d["Beam0"]
        graph = p.replace("Strips", "Graphs").replace(
            "_aufx_bmfd.plist", ".dspg")
        units = graph_units(graph) if os.path.exists(graph) else []
        stats = [beam_stats(beam, hz) for hz in REPORT_HZ]
        rows.append({
            "aid": aid_number(p),
            "mics": d.get("nMicsUsed"),
            "units": units,
            "norm_db": [round(s[0], 1) for s in stats],
            "coherent_db": [round(s[1], 1) for s in stats],
        })
    return rows


def main(argv):
    root = argv[1] if len(argv) > 1 else ROOT
    print("AID\t麦数\t处理单元\t范数dB@%s\t同相dB" % (REPORT_HZ,))
    for r in scan(root):
        print("%d\t%s\t%s\t%s\t%s" % (r["aid"], r["mics"], ",".join(r["units"]),
                                      r["norm_db"], r["coherent_db"]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
