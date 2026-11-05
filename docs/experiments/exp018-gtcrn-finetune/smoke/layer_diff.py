"""exp018 冒烟：GTCRN 逐层比 CPU 与 MPS 的输出，找出差从哪一层开始。

起因：mps_parity.py 判前向不过（相对差 0.61）。本脚本给每个叶子模块挂前向钩子，
同一份权重、同一份输入（先在 CPU 上算好 STFT，再拷到两边，排除 STFT 本身的差），
逐层记录相对差，按执行顺序列出，并标出「第一个超过阈值的层」。

用法（几秒）：
    python layer_diff.py --gtcrn-dir <resources/gtcrn> --ckpt <…tar> --mode eval --out <…json>
"""
from __future__ import annotations

import argparse
import copy
import json
import os

os.environ.setdefault("PYTORCH_ENABLE_MPS_FALLBACK", "1")

import torch  # noqa: E402

import mps_parity as mp  # noqa: E402
import step_timing as st  # noqa: E402


def first_tensor(x):
    """模块输出可能是张量或 (输出, 隐状态) 元组；取第一个张量。"""
    if isinstance(x, torch.Tensor):
        return x
    if isinstance(x, (tuple, list)):
        for v in x:
            t = first_tensor(v)
            if t is not None:
                return t
    return None


def record(model: torch.nn.Module) -> tuple[list, list]:
    """给叶子模块挂钩子，按执行顺序记 (名字, 类型, 输出)。"""
    log: list = []
    hooks = []
    for name, mod in model.named_modules():
        if len(list(mod.children())) == 0 or isinstance(mod, torch.nn.GRU):
            def hook(m, inp, out, name=name):
                t = first_tensor(out)
                if t is not None:
                    log.append((name, type(m).__name__, t.detach().cpu()))
            hooks.append(mod.register_forward_hook(hook))
    return log, hooks


def diff(gtcrn_dir: str, ckpt: str | None, device: str, mode: str, seconds: float,
         threshold: float = 1e-3) -> dict:
    torch.manual_seed(0)
    GTCRN, _ = st.load_official(gtcrn_dir)
    m_cpu = GTCRN()
    if ckpt:
        m_cpu.load_state_dict(torch.load(ckpt, map_location="cpu", weights_only=False)["model"])
    m_dev = copy.deepcopy(m_cpu).to(device)
    for m in (m_cpu, m_dev):
        m.train() if mode == "train" else m.eval()
    noisy, _ = mp.batch(1, seconds, seed=0)
    x = st.to_real(st.stft(noisy))  # STFT 只在 CPU 上算一次
    log_c, hc = record(m_cpu)
    log_d, hd = record(m_dev)
    with torch.no_grad():
        y_c = m_cpu(x)
        y_d = m_dev(x.to(device))
    for h in hc + hd:
        h.remove()
    if len(log_c) != len(log_d):
        raise RuntimeError(f"两边钩子数不同：{len(log_c)} vs {len(log_d)}")
    layers = []
    first_bad = None
    for (n, t, a), (_, _, b) in zip(log_c, log_d):
        r = mp.rel(b, a)
        layers.append({"name": n, "type": t, "rel": r})
        if first_bad is None and r > threshold:
            first_bad = {"name": n, "type": t, "rel": r}
    return {
        "device": device, "mode": mode, "seconds": seconds, "threshold": threshold,
        "output_rel": mp.rel(y_d, y_c), "first_bad": first_bad,
        "worst": sorted(layers, key=lambda d: -d["rel"])[:8], "layers": layers,
        "torch": torch.__version__,
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--gtcrn-dir", required=True)
    ap.add_argument("--ckpt", default=None)
    ap.add_argument("--device", default="mps")
    ap.add_argument("--mode", default="eval", choices=["eval", "train"])
    ap.add_argument("--seconds", type=float, default=0.5)
    ap.add_argument("--out", default=None)
    a = ap.parse_args(argv)
    r = diff(a.gtcrn_dir, a.ckpt, a.device, a.mode, a.seconds)
    if a.out:
        os.makedirs(os.path.dirname(a.out) or ".", exist_ok=True)
        if os.path.exists(a.out):
            raise SystemExit(f"拒绝覆盖已有文件：{a.out}")
        with open(a.out, "w", encoding="utf-8") as f:
            json.dump(r, f, ensure_ascii=False, indent=2)
    print(json.dumps({k: r[k] for k in ("device", "mode", "output_rel", "first_bad", "worst")},
                     ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
