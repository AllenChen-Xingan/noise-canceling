"""exp003 第四步：生成对比页（_tmp/20261030-exp003/index.html + manifest.js + 检验题音频）。

- 每段的两个处理版随机放进槽位 x / y（secrets 随机），页面运行时再随机分到 A / B，两层都随机
- 揭盲表以 base64 存在 manifest.js 里（只防「一眼看到」，不防有意去解）；音频文件名是随机串，不含模型名
- 检验题两道（借 P.808 陷阱题思路）：
  · trap-identical：A、B 是同一个文件 → 期望两问都答「差不多」
  · trap-degraded：A、B 一个是处理版原样，一个是同一文件低通 1 kHz 再加 5 dB 白噪声（响度拉回相同）→ 期望两问都选原样那份
用法：_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/build_page.py
"""
import base64
import json
import secrets
import shutil
from datetime import datetime
from pathlib import Path

import numpy as np
import pyloudnorm
import soundfile as sf
from scipy.signal import butter, sosfilt

from _paths import ROOT
HERE = Path(__file__).resolve().parent
OUT = ROOT / "_tmp/20261030-exp003"
SR = 16000
MODELS = ["dpdfnet2", "gtcrn"]


def main():
    clips = json.loads((HERE / "clips.json").read_text())["clips"]
    files = json.loads((OUT / "files.json").read_text())
    rng = secrets.SystemRandom()
    key, mclips = {}, []
    for c in clips:
        cid = c["id"]
        order = MODELS[:]
        rng.shuffle(order)
        key[cid] = {"x": order[0], "y": order[1]}
        v = files[cid]["versions"]
        mclips.append({"id": cid, "desc": c["desc"], "ref": v["noisy"]["file"],
                       "x": v[order[0]]["file"], "y": v[order[1]]["file"]})

    # 检验题
    traps = []
    c1, c2 = rng.sample([c for c in clips if c["kind"] == "synthetic"], 2)
    m1 = rng.choice(MODELS)
    f1 = files[c1["id"]]["versions"][m1]["file"]
    traps.append({"id": "trap-identical", "clip": c1["id"], "desc": c1["desc"], "label": "A、B 其实是同一个文件（应答「差不多」）",
                  "ref": files[c1["id"]]["versions"]["noisy"]["file"], "s1": f1, "s2": f1,
                  "names": {"s1": "same", "s2": "same"}, "expect": {"clearer": ["same"], "natural": ["same"]},
                  "source_model": m1})
    m2 = rng.choice(MODELS)
    f2 = files[c2["id"]]["versions"][m2]["file"]
    y, _ = sf.read(OUT / f2, dtype="float32")
    sos = butter(6, 1000, "low", fs=SR, output="sos")
    d = sosfilt(sos, y).astype(np.float32)
    noise = np.random.default_rng().standard_normal(len(d)).astype(np.float32)
    d = d + noise * np.sqrt(np.mean(d ** 2) / np.mean(noise ** 2)) * 10 ** (-5 / 20)
    meter = pyloudnorm.Meter(SR)
    d = d * 10 ** ((meter.integrated_loudness(y) - meter.integrated_loudness(d)) / 20)
    d = d / max(1.0, float(np.max(np.abs(d))) / 0.89)
    fd = f"audio/{c2['id']}_{secrets.token_hex(3)}.wav"
    sf.write(OUT / fd, d, SR, subtype="PCM_16")
    traps.append({"id": "trap-degraded", "clip": c2["id"], "desc": c2["desc"], "label": "一份是明显劣化版（应选原样那份）",
                  "ref": files[c2["id"]]["versions"]["noisy"]["file"], "s1": f2, "s2": fd,
                  "names": {"s1": "intact", "s2": "degraded"}, "expect": {"clearer": ["intact"], "natural": ["intact"]},
                  "source_model": m2})

    build = datetime.now().strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    manifest = {"build": build, "clips": mclips,
                "traps": [{k: v for k, v in t.items() if k != "source_model"} for t in traps],
                "key": base64.b64encode(json.dumps(key).encode()).decode()}
    (OUT / "manifest.js").write_text("window.EXP003 = " + json.dumps(manifest, ensure_ascii=False) + ";\n")
    shutil.copyfile(HERE / "ab_page.html", OUT / "index.html")
    # 留一份揭盲表在 _tmp 外层（不进仓、不给页面读），方便事后核对
    (OUT / "unblind-key.json").write_text(json.dumps({"build": build, "key": key, "traps": traps}, ensure_ascii=False, indent=1))
    print("build", build, "→", OUT / "index.html")


if __name__ == "__main__":
    main()
