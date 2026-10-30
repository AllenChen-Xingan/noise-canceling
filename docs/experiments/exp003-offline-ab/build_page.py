"""exp003 第四步：生成对比页（_tmp/20261030-exp003/index.html + manifest.js + 检验题音频）。

- 候选模型 = files.json 里除 noisy 外的所有版本（DPDFNet-2、GTCRN，接好后加 VoxLock），一屏同时给出 A/B/C… + 原声
- 两层随机：生成时每段的候选随机排进槽位 s0、s1…（secrets 随机）；页面运行时再按拉丁方轮换分到 A/B/C…，
  使每个候选在每个字母位置出现的次数平衡
- 揭盲表以 base64 存在 manifest.js 里（只防「一眼看到」，不防有意去解）；音频文件名是随机串，不含模型名
- 检验题两道（借 P.808 陷阱题思路）：
  · trap-identical：所有槽位是同一个文件 → 期望「最清楚」「最自然」都答「差不多」
  · trap-degraded：一个槽位是某候选输出低通 1 kHz 再加 5 dB 白噪声（响度拉回相同），其余槽位是各候选原样
    → 期望「最清楚」「最自然」都不选劣化版、也不答「差不多」
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/build_page.py
"""
import base64
import json
import secrets
import shutil
from datetime import datetime

import numpy as np
import pyloudnorm
import soundfile as sf
from scipy.signal import butter, sosfilt

from _paths import ROOT

HERE = ROOT / "docs/experiments/exp003-offline-ab"
OUT = ROOT / "_tmp/20261030-exp003"
SR = 16000
# 名字与端到端延迟说明（页面揭盲后显示）；延迟出处见 docs/research/002_model-baseline/decision.md
INFO = {
    "dpdfnet2": {"name": "DPDFNet-2", "latency": "端到端约 66–70 ms"},
    "gtcrn": {"name": "GTCRN", "latency": "端到端约 38–42 ms"},
    "voxlock_given": {"name": "VoxLock（给定 3 s 样本）", "latency": "待 exp003 计时"},
    "voxlock_live": {"name": "VoxLock（自动锁定）", "latency": "待 exp003 计时"},
}


def degrade(y, meter):
    sos = butter(6, 1000, "low", fs=SR, output="sos")
    d = sosfilt(sos, y).astype(np.float32)
    noise = np.random.default_rng().standard_normal(len(d)).astype(np.float32)
    d = d + noise * np.sqrt(np.mean(d ** 2) / np.mean(noise ** 2)) * 10 ** (-5 / 20)
    d = d * 10 ** ((meter.integrated_loudness(y) - meter.integrated_loudness(d)) / 20)
    return d / max(1.0, float(np.max(np.abs(d))) / 0.89)


def main():
    clips = json.loads((HERE / "clips.json").read_text())["clips"]
    files = json.loads((OUT / "files.json").read_text())
    models = [m for m in next(iter(files.values()))["versions"] if m != "noisy"]
    assert all(set(files[c["id"]]["versions"]) == {"noisy", *models} for c in clips), "各段的候选不一致"
    rng = secrets.SystemRandom()
    key, mclips = {}, []
    for c in clips:
        cid = c["id"]
        order = models[:]
        rng.shuffle(order)
        key[cid] = order
        v = files[cid]["versions"]
        mclips.append({"id": cid, "desc": c["desc"], "ref": v["noisy"]["file"], "cands": [v[m]["file"] for m in order]})

    traps = []
    c1, c2 = rng.sample([c for c in clips if c["kind"] == "synthetic"], 2)
    m1 = rng.choice(models)
    f1 = files[c1["id"]]["versions"][m1]["file"]
    traps.append({"id": "trap-identical", "clip": c1["id"], "desc": c1["desc"],
                  "label": "所有候选其实是同一个文件（应答「差不多」）",
                  "ref": files[c1["id"]]["versions"]["noisy"]["file"], "cands": [f1] * len(models),
                  "names": ["same"] * len(models), "expect": {"best": ["same"]}, "source_model": m1})
    m2 = rng.choice(models)
    f2 = files[c2["id"]]["versions"][m2]["file"]
    y, _ = sf.read(OUT / f2, dtype="float32")
    fd = f"audio/{c2['id']}_{secrets.token_hex(3)}.wav"
    sf.write(OUT / fd, degrade(y, pyloudnorm.Meter(SR)), SR, subtype="PCM_16")
    others = [m for m in models if m != m2]
    rng.shuffle(others)
    slots = [(m2, f2), ("degraded", fd)] + [(m, files[c2["id"]]["versions"][m]["file"]) for m in others[: len(models) - 2]]
    rng.shuffle(slots)
    traps.append({"id": "trap-degraded", "clip": c2["id"], "desc": c2["desc"],
                  "label": "其中一份是明显劣化版（最清楚、最自然都不该选它，也不该答「差不多」）",
                  "ref": files[c2["id"]]["versions"]["noisy"]["file"], "cands": [f for _, f in slots],
                  "names": [n for n, _ in slots], "expect": {"not": ["degraded", "same"]}, "source_model": m2})

    build = datetime.now().strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    manifest = {"build": build, "n_cands": len(models), "clips": mclips,
                "models": {m: INFO.get(m, {"name": m, "latency": "?"}) for m in models},
                "traps": [{k: v for k, v in t.items() if k not in ("source_model", "names")} for t in traps],
                "key": base64.b64encode(json.dumps({"clips": key, "traps": {t["id"]: t["names"] for t in traps}})
                                        .encode()).decode()}
    (OUT / "manifest.js").write_text("window.EXP003 = " + json.dumps(manifest, ensure_ascii=False) + ";\n")
    shutil.copyfile(HERE / "ab_page.html", OUT / "index.html")
    # 留一份揭盲表在 _tmp 下（不进仓、页面不读），方便事后核对
    (OUT / "unblind-key.json").write_text(json.dumps({"build": build, "key": key, "traps": traps}, ensure_ascii=False, indent=1))
    print("build", build, "候选", models, "→", OUT / "index.html")


if __name__ == "__main__":
    main()
