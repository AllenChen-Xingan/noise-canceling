"""exp003 第四步：生成对比页（_tmp/20261030-exp003/index.html + manifest.js + 检验题音频）。

- 候选 = CANDS（默认 DPDFNet-2 · GTCRN · FastEnhancer-M；--cands a,b,c 可改），一屏同时给出 A/B/C… + 原声
- 每次生成一个新 build：页面写进 _tmp/20261030-exp003/builds/<build>/，**不覆盖、不删任何旧文件**（音频共用 _tmp/20261030-exp003/audio/）；
  揭盲表 unblind-key.json、manifest.js、files.json 同时复制进 docs/experiments/exp003-offline-ab/builds/<build>/ 进版本库（人定：对照表都要提交）
- 两层随机：生成时每段的候选随机排进槽位 s0、s1…（secrets 随机）；页面运行时再按拉丁方轮换分到 A/B/C…，
  使每个候选在每个字母位置出现的次数平衡
- 揭盲表以 base64 存在 manifest.js 里（只防「一眼看到」，不防有意去解）；音频文件名是随机串，不含模型名
- 检验题两道（借 P.808 陷阱题思路）：
  · trap-identical：所有槽位是同一个文件 → 期望「最清楚」「最自然」都答「差不多」
  · trap-degraded：一个槽位是某候选输出低通 1 kHz 再加 5 dB 白噪声（响度拉回相同），其余槽位是各候选原样
    → 期望「最清楚」「最自然」都不选劣化版（答「分不出」不扣）
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/build_page.py
"""
import base64
import json
import secrets
import shutil
import sys
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
# FastEnhancer-M：窗 512（32 ms）、前瞻 0，同 GTCRN 口径（README「FastEnhancer 计时」）
INFO = {
    "dpdfnet2": {"name": "DPDFNet-2", "latency": "端到端约 66–70 ms（≤ 75 ms 档）"},
    "gtcrn": {"name": "GTCRN", "latency": "端到端约 38–42 ms（≤ 45 ms 档）"},
    "fastenhancer_m": {"name": "FastEnhancer-M", "latency": "端到端约 38–42 ms（≤ 45 ms 档）"},
    "voxlock_given": {"name": "VoxLock（给定 3 s 样本）", "latency": "单线程跟不上实时；攒 40 ms 一块约 70–74 ms 起"},
    "voxlock_live": {"name": "VoxLock（自动锁定）", "latency": "同上"},
}
CANDS = ["dpdfnet2", "gtcrn", "fastenhancer_m"]


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
    models = sys.argv[sys.argv.index("--cands") + 1].split(",") if "--cands" in sys.argv else CANDS
    assert all({"noisy", *models} <= set(files[c["id"]]["versions"]) for c in clips), "files.json 里缺候选，先跑 enhance.py"
    # 「显示原文」：LibriSpeech 段用人工转写，其余用干净参考的 Qwen3-ASR 转写（标「机器转写」）
    sc = json.loads((HERE / "scores.json").read_text())["per_clip"] if (HERE / "scores.json").exists() else {}
    def text_of(c):
        if c.get("target", {}).get("text"):
            return c["target"]["text"].capitalize()
        r = sc.get(c["id"], {}).get("asr_ref")
        return f"{r}（机器转写）" if r else None
    build = datetime.now().strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    page_dir = OUT / "builds" / build
    page_dir.mkdir(parents=True, exist_ok=False)
    up = lambda f: "../../" + f  # 页面在 builds/<build>/ 下，音频在 _tmp/20261030-exp003/audio/
    rng = secrets.SystemRandom()
    key, mclips = {}, []
    for c in clips:
        cid = c["id"]
        order = models[:]
        rng.shuffle(order)
        key[cid] = order
        v = files[cid]["versions"]
        mclips.append({"id": cid, "desc": c["desc"], "text": text_of(c), "ref": up(v["noisy"]["file"]),
                       "cands": [up(v[m]["file"]) for m in order]})

    traps = []
    c1, c2 = rng.sample([c for c in clips if c["kind"] == "synthetic"], 2)
    m1 = rng.choice(models)
    f1 = files[c1["id"]]["versions"][m1]["file"]
    traps.append({"id": "trap-identical", "clip": c1["id"], "desc": c1["desc"],
                  "label": "所有候选其实是同一个文件（应答「差不多」）",
                  "ref": up(files[c1["id"]]["versions"]["noisy"]["file"]), "cands": [up(f1)] * len(models),
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
                  "ref": up(files[c2["id"]]["versions"]["noisy"]["file"]), "cands": [up(f) for _, f in slots],
                  "names": [n for n, _ in slots], "expect": {"not": ["degraded"]}, "source_model": m2})

    manifest = {"build": build, "n_cands": len(models), "clips": mclips,
                "models": {m: INFO.get(m, {"name": m, "latency": "?"}) for m in models},
                "traps": [{k: v for k, v in t.items() if k not in ("source_model", "names")} for t in traps],
                "key": base64.b64encode(json.dumps({"clips": key, "traps": {t["id"]: t["names"] for t in traps}})
                                        .encode()).decode()}
    (page_dir / "manifest.js").write_text("window.EXP003 = " + json.dumps(manifest, ensure_ascii=False) + ";\n")
    shutil.copyfile(HERE / "ab_page.html", page_dir / "index.html")
    (page_dir / "unblind-key.json").write_text(json.dumps({"build": build, "models": models, "key": key, "traps": traps},
                                                          ensure_ascii=False, indent=1))
    shutil.copyfile(OUT / "files.json", page_dir / "files.json")
    # 对照表进版本库（人定：每个 build 的 unblind-key / manifest / files.json 都要提交）
    keep = HERE / "builds" / build
    keep.mkdir(parents=True, exist_ok=False)
    for f in ["unblind-key.json", "manifest.js", "files.json"]:
        shutil.copyfile(page_dir / f, keep / f)
    print("build", build, "候选", models, "→", page_dir / "index.html")


if __name__ == "__main__":
    main()
