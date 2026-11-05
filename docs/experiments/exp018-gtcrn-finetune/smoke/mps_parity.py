"""exp018 冒烟：GTCRN 在 MPS 上算得对不对——和 CPU 比前向输出、梯度、连续训练 N 步的损失与权重。

起因：PyTorch issue #94691（MPS 上 nn.GRU 静默算错 / 出 NaN，2025-01 标「correctness (silent)」，
LSTM 已修、GRU 未修；见调研 013 `notes/D-finetune-skeptic.md`）。GTCRN 有 14 个 GRU 实例。
单步耗时快不代表算得对，所以单独核。

判据（本脚本的默认阈值，写进输出 JSON）：
- 前向输出相对误差（‖mps−cpu‖ / ‖cpu‖）≤ 1e-3
- 每个参数的梯度相对误差 ≤ 1e-2（GRU 反向在不同后端累加顺序不同，放宽一档）
- N 步训练后：两边损失都有限；每步损失相对差 ≤ 1e-2；最终权重相对差 ≤ 1e-2

用法（单次 ≤ 约 1 分钟）：
    python mps_parity.py --gtcrn-dir <resources/gtcrn> --ckpt <…/model_trained_on_dns3.tar> \
        --out <主目录>/_tmp/027/<run-id>/parity.json
"""
from __future__ import annotations

import argparse
import copy
import json
import os

os.environ.setdefault("PYTORCH_ENABLE_MPS_FALLBACK", "1")

import torch  # noqa: E402

import step_timing as st  # noqa: E402  （同目录：复用官方模型载入与 STFT，不重复写）

TOL = {"forward": 1e-3, "grad": 1e-2, "train_loss": 1e-2, "train_weight": 1e-2}


def rel(a: torch.Tensor, b: torch.Tensor) -> float:
    """‖a−b‖ / ‖b‖，两边先挪到 CPU float64。"""
    a, b = a.detach().double().cpu(), b.detach().double().cpu()
    return float((a - b).norm() / (b.norm() + 1e-12))


def batch(n_batch: int, seconds: float, seed: int) -> tuple[torch.Tensor, torch.Tensor]:
    """伪语音：带包络的低通噪声当「人声」，白噪当噪声；比纯白噪更像真实输入的动态范围。"""
    g = torch.Generator().manual_seed(seed)
    n = int(seconds * st.FS)
    src = torch.randn(n_batch, n, generator=g)
    k = torch.ones(1, 1, 15) / 15
    voice = torch.nn.functional.conv1d(src[:, None], k, padding="same")[:, 0]
    env = (torch.sin(torch.linspace(0, 12.0, n))[None] > 0).float() * 0.9 + 0.1
    clean = 0.3 * voice * env
    noisy = clean + 0.05 * torch.randn(n_batch, n, generator=g)
    return noisy, clean


def compare(model_cpu: torch.nn.Module, device: str, loss_cls, n_batch: int, seconds: float,
            steps: int, lr: float) -> dict:
    """在 CPU 和 device 上从同一份权重出发，逐项比对。"""
    m_dev = copy.deepcopy(model_cpu).to(device)
    m_cpu = model_cpu
    loss_fn = loss_cls()
    out: dict = {"device": device}

    # 1) 前向 + 2) 梯度（同一批输入）
    noisy, clean = batch(n_batch, seconds, seed=0)
    res = {}
    for name, m, dev in (("cpu", m_cpu, "cpu"), ("dev", m_dev, device)):
        m.train()
        m.zero_grad(set_to_none=True)
        x = st.to_real(st.stft(noisy.to(dev)))
        y = st.to_real(st.stft(clean.to(dev)))
        pred = m(x)
        loss = loss_fn(pred, y)
        loss.backward()
        res[name] = (pred.detach().cpu(), float(loss.detach().cpu()),
                     {k: p.grad.detach().cpu() for k, p in m.named_parameters() if p.grad is not None})
    out["forward_rel"] = rel(res["dev"][0], res["cpu"][0])
    out["loss_cpu"], out["loss_dev"] = res["cpu"][1], res["dev"][1]
    grad_rel = {k: rel(res["dev"][2][k], g) for k, g in res["cpu"][2].items()}
    worst = sorted(grad_rel.items(), key=lambda kv: -kv[1])[:5]
    out["grad_rel_max"] = worst[0][1] if worst else 0.0
    out["grad_rel_worst5"] = dict(worst)
    out["grad_any_nan_dev"] = any(not torch.isfinite(g).all() for g in res["dev"][2].values())

    # 3) 连续训练 N 步，每步同一批数据
    opts = {n: torch.optim.Adam(m.parameters(), lr=lr) for n, m in (("cpu", m_cpu), ("dev", m_dev))}
    traj: dict[str, list[float]] = {"cpu": [], "dev": []}
    for s in range(steps):
        noisy, clean = batch(n_batch, seconds, seed=100 + s)
        for name, m, dev in (("cpu", m_cpu, "cpu"), ("dev", m_dev, device)):
            x = st.to_real(st.stft(noisy.to(dev)))
            y = st.to_real(st.stft(clean.to(dev)))
            loss = loss_fn(m(x), y)
            opts[name].zero_grad(set_to_none=True)
            loss.backward()
            opts[name].step()
            traj[name].append(float(loss.detach().cpu()))
    out["train_steps"] = steps
    out["train_loss_cpu"] = traj["cpu"]
    out["train_loss_dev"] = traj["dev"]
    out["train_loss_rel_max"] = max(abs(a - b) / (abs(b) + 1e-12) for a, b in zip(traj["dev"], traj["cpu"]))
    out["train_loss_finite"] = all(v == v and abs(v) != float("inf") for v in traj["dev"] + traj["cpu"])
    w_dev = torch.cat([p.detach().cpu().flatten() for p in m_dev.parameters()])
    w_cpu = torch.cat([p.detach().cpu().flatten() for p in m_cpu.parameters()])
    out["train_weight_rel"] = rel(w_dev, w_cpu)
    return out


def verdict(r: dict) -> dict:
    checks = {
        "forward": r["forward_rel"] <= TOL["forward"],
        "grad": r["grad_rel_max"] <= TOL["grad"] and not r["grad_any_nan_dev"],
        "train_loss": r["train_loss_finite"] and r["train_loss_rel_max"] <= TOL["train_loss"],
        "train_weight": r["train_weight_rel"] <= TOL["train_weight"],
    }
    return {"tolerances": TOL, "checks": checks, "pass": all(checks.values())}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--gtcrn-dir", required=True)
    ap.add_argument("--ckpt", default=None)
    ap.add_argument("--device", default="mps", help="和 CPU 比的那一边；测试里可传 cpu 自比")
    ap.add_argument("--batch", type=int, default=8)
    ap.add_argument("--seconds", type=float, default=4.0)
    ap.add_argument("--steps", type=int, default=20)
    ap.add_argument("--lr", type=float, default=1e-4)
    ap.add_argument("--out", default=None)
    a = ap.parse_args(argv)
    torch.manual_seed(0)
    GTCRN, HybridLoss = st.load_official(a.gtcrn_dir)
    model = GTCRN()
    if a.ckpt:
        model.load_state_dict(torch.load(a.ckpt, map_location="cpu", weights_only=False)["model"])
    r = compare(model, a.device, HybridLoss, a.batch, a.seconds, a.steps, a.lr)
    r.update(verdict(r))
    r.update(batch=a.batch, seconds_per_clip=a.seconds, lr=a.lr,
             checkpoint=os.path.basename(a.ckpt) if a.ckpt else None, torch=torch.__version__)
    text = json.dumps(r, ensure_ascii=False, indent=2)
    if a.out:
        os.makedirs(os.path.dirname(a.out) or ".", exist_ok=True)
        if os.path.exists(a.out):
            raise SystemExit(f"拒绝覆盖已有文件：{a.out}")
        with open(a.out, "w", encoding="utf-8") as f:
            f.write(text + "\n")
    print(text)
    return 0 if r["pass"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
