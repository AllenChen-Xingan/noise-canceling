"""生成 ESTOI 对拍参考值：用上游 pystoi 在公式造的确定性信号上算 ESTOI，写 estoi-pystoi.json。

信号造法必须和 `tests/estoi_parity.rs` 的 `signal()` 一字不差（两边各写一份公式，参考值由本脚本算出后进仓）。
跑法（主目录下有 exp002 的 Python 环境，带 pystoi 0.4.1）：
    <主目录>/_tmp/exp002-venv/bin/python src/eval/tests/fixtures/gen_estoi_ref.py
"""
import json, math, pathlib
import numpy as np
import pystoi

SR = 16000
M64 = (1 << 64) - 1

def lcg(seed):
    s = seed & M64
    while True:
        s = (s * 6364136223846793005 + 1442695040888963407) & M64
        yield ((s >> 11) / float(1 << 53)) * 2.0 - 1.0

def clean(n):
    out = np.zeros(n)
    for i in range(n):
        t = i / SR
        on = 1.0 if int(t * 2.5) % 2 == 0 else 0.0
        f0 = 120.0 * (1.0 + 0.08 * math.sin(2 * math.pi * 0.7 * t))
        v = 0.0
        for k in range(1, 25):
            v += math.sin(2 * math.pi * k * f0 * t) / k
        am = 0.6 + 0.4 * math.sin(2 * math.pi * 4.0 * t)
        out[i] = 0.1 * v * am * on
    return out

def case(kind, n, seed):
    x = clean(n)
    g = lcg(seed)
    if kind.startswith("noise"):
        a = float(kind.split("-")[1])
        y = np.array([x[i] + a * next(g) for i in range(n)])
    elif kind == "zero":
        # 输出整段数字零（降噪器把人声全删掉的极端情形）
        y = np.zeros(n)
    elif kind == "gate":
        y = np.array([x[i] * (0.1 if (i // 1600) % 3 == 0 else 1.0) for i in range(n)])
    else:
        y = x.copy()
    return x, y

CASES = [("same", 1), ("noise-0.02", 2), ("noise-0.1", 3), ("noise-0.3", 4), ("gate", 5), ("zero", 6)]
N = SR * 4
res = []
for kind, seed in CASES:
    x, y = case(kind, N, seed)
    res.append({"case": kind, "seed": seed,
                "estoi": float(pystoi.stoi(x, y, SR, extended=True)),
                "stoi": float(pystoi.stoi(x, y, SR, extended=False))})
out = {"doc": "pystoi 0.4.1 在公式信号上的参考值（gen_estoi_ref.py 生成）；对拍要求 |Rust − 上游| ≤ 1e-4",
       "pystoi": pystoi.__version__ if hasattr(pystoi, "__version__") else "0.4.1",
       "sr": SR, "n": N, "cases": res}
p = pathlib.Path(__file__).with_name("estoi-pystoi.json")
p.write_text(json.dumps(out, ensure_ascii=False, indent=1) + "\n")
print(json.dumps(res, indent=1))
