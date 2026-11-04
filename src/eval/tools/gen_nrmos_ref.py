"""生成评分模型对拍参考值：用上游 Python 包（scoreq 1.0.1 · distillmos 0.9.1）给 6 段公式信号打分，
写 `tests/fixtures/nrmos-upstream.json`。信号公式必须和 `tests/nrmos_parity.rs` 的 `signal()` 一字不差。

调用方式照上游：SCOREQ 用 `Scoreq(data_domain="natural", mode="nr").predict(test_path=…)`（读 float32 wav）；
Distill-MOS 用 `ConvTransformerSQAModel()(x)`（默认在前向里分段）。音频写在临时目录，用完即删、不进仓。
跑法：<主目录>/_tmp/exp009-venv/bin/python -I src/eval/tools/gen_nrmos_ref.py
"""
import json, math, pathlib, tempfile
import numpy as np
import soundfile as sf
import torch
import scoreq
import distillmos

SR = 16000
M64 = (1 << 64) - 1

def lcg(seed):
    s = seed & M64
    while True:
        s = (s * 6364136223846793005 + 1442695040888963407) & M64
        yield ((s >> 11) / float(1 << 53)) * 2.0 - 1.0

def clean(n):
    t = np.arange(n) / SR
    on = ((t * 2.5).astype(np.int64) % 2 == 0).astype(np.float64)
    f0 = 120.0 * (1.0 + 0.08 * np.sin(2 * math.pi * 0.7 * t))
    v = np.zeros(n)
    for k in range(1, 25):
        v += np.sin(2 * math.pi * k * f0 * t) / k
    am = 0.6 + 0.4 * np.sin(2 * math.pi * 4.0 * t)
    return 0.1 * v * am * on

# (名字, 秒长对应的样本数, 噪声幅度, 种子)：覆盖短于一段、正好一段、不是 320 倍数、要分 4 段、20 s 长段
CASES = [("short-3s", 48000, 0.0, 1), ("seq-exact", 122880, 0.01, 2), ("odd-len", 81967, 0.03, 3),
         ("seg4-10.6s", 169600, 0.05, 4), ("long-20s-noisy", 320000, 0.1, 5), ("noise-only", 64000, -1.0, 6)]

def signal(n, a, seed):
    g = lcg(seed)
    if a < 0:  # 只有噪声
        return np.array([0.1 * next(g) for _ in range(n)])
    x = clean(n)
    if a > 0:
        x = x + a * np.array([next(g) for _ in range(n)])
    return x

sq = scoreq.Scoreq(data_domain="natural", mode="nr")
dm = distillmos.ConvTransformerSQAModel().eval()
res = []
with tempfile.TemporaryDirectory() as d:
    for name, n, a, seed in CASES:
        x = signal(n, a, seed).astype(np.float32)
        p = pathlib.Path(d) / f"{name}.wav"
        sf.write(p, x, SR, subtype="FLOAT")
        s1 = float(sq.predict(test_path=str(p), ref_path=None))
        with torch.no_grad():
            s2 = float(dm(torch.from_numpy(x)[None, :])[0])
        res.append({"case": name, "n": n, "noise": a, "seed": seed, "scoreq": s1, "distill": s2})
        print(name, round(s1, 4), round(s2, 4))
out = {"doc": "上游 scoreq 1.0.1（natural · NR，ONNX）与 distillmos 0.9.1 在公式信号上的参考分（gen_nrmos_ref.py 生成）；对拍要求 |Rust − 上游| ≤ 0.01",
       "sr": SR, "cases": res}
p = pathlib.Path(__file__).resolve().parents[1] / "tests/fixtures/nrmos-upstream.json"
p.write_text(json.dumps(out, ensure_ascii=False, indent=1) + "\n")
print("已写", p)
