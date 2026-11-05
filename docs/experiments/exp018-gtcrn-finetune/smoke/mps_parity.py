"""exp018 冒烟：GTCRN 在 MPS 上算得对不对——和 CPU 比前向输出、梯度、连续训练 N 步的损失与权重。

起因：PyTorch issue #94691（MPS 上 nn.GRU 静默算错 / 出 NaN，2025-01 标「correctness (silent)」，
LSTM 已修、GRU 未修；见调研 013 `notes/D-finetune-skeptic.md`）。GTCRN 有 14 个 GRU 实例。
单步耗时快不代表算得对，所以单独核。

判据（本脚本的默认阈值，写进输出 JSON）：
- 前向输出相对误差（‖mps−cpu‖ / ‖cpu‖）≤ 1e-3
- 整个模型的梯度向量相对误差 ≤ 1e-3；每个参数的梯度误差按「占整个梯度的比例」算
  （‖Δg_i‖ / ‖g_全部‖）≤ 1e-3。不用「每个参数自己的相对误差」：BatchNorm 前面那层卷积的
  bias 理论梯度为 0，数值上只剩噪声，自己除自己会到 10 倍量级（2026-11-05 实测，误报）
- N 步训练后：两边损失都有限；每步损失相对差 ≤ 1e-2；最终完整状态（参数 + BatchNorm 运行统计）相对差 ≤ 1e-2
- 导出口径：device 上训出的状态装进**没打绕法**的原模块，CPU eval 前向与 CPU 训出的模型相对差 ≤ 1e-2

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

TOL = {"forward": 1e-3, "grad": 1e-3, "train_loss": 1e-2, "train_weight": 1e-2, "export_eval": 1e-2}


def rel(a: torch.Tensor, b: torch.Tensor) -> float:
    """‖a−b‖ / ‖b‖，两边先挪到 CPU float64。"""
    a, b = a.detach().cpu().double(), b.detach().cpu().double()
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
    gc, gd = res["cpu"][2], res["dev"][2]
    all_c = torch.cat([gc[k].flatten() for k in gc]).double()
    all_d = torch.cat([gd[k].flatten() for k in gc]).double()
    total = float(all_c.norm()) + 1e-12
    out["grad_rel_model"] = rel(all_d, all_c)
    share = {k: float((gd[k].double() - gc[k].double()).norm()) / total for k in gc}
    worst = sorted(share.items(), key=lambda kv: -kv[1])[:5]
    out["grad_share_max"] = worst[0][1] if worst else 0.0
    out["grad_share_worst5"] = dict(worst)
    self_rel = sorted(((k, rel(gd[k], g)) for k, g in gc.items()), key=lambda kv: -kv[1])[:5]
    out["grad_self_rel_worst5_info_only"] = dict(self_rel)
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
    # 4) 训练后的完整状态：参数 + BatchNorm 运行统计（codex 027 第 1 轮 #6）
    sd_dev, sd_cpu = m_dev.state_dict(), m_cpu.state_dict()
    fkeys = [k for k, v in sd_cpu.items() if v.is_floating_point()]
    w_dev = torch.cat([sd_dev[k].detach().cpu().flatten() for k in fkeys])
    w_cpu = torch.cat([sd_cpu[k].detach().cpu().flatten() for k in fkeys])
    out["train_weight_rel"] = rel(w_dev, w_cpu)
    out["train_state_keys"] = len(fkeys)
    # 5) 导出口径：把 device 上训出的 state_dict 装进一个**没打绕法**的新模型，CPU eval 前向，
    #    与 CPU 上训出的模型 eval 前向比（导出给引擎时用的是原模块）
    fresh = type(m_cpu)()
    fresh.load_state_dict({k: v.detach().cpu() for k, v in sd_dev.items()})
    fresh.eval()
    m_cpu.eval()
    noisy, _ = batch(n_batch, seconds, seed=999)
    x = st.to_real(st.stft(noisy))
    with torch.no_grad():
        out["export_eval_rel"] = rel(fresh(x), m_cpu(x))
    out["export_unpatched"] = not any("forward" in vars(mod) for mod in fresh.modules())
    return out


def verdict(r: dict) -> dict:
    checks = {
        "forward": r["forward_rel"] <= TOL["forward"],
        "grad": (r["grad_rel_model"] <= TOL["grad"] and r["grad_share_max"] <= TOL["grad"]
                 and not r["grad_any_nan_dev"]),
        "train_loss": r["train_loss_finite"] and r["train_loss_rel_max"] <= TOL["train_loss"],
        "train_weight": r["train_weight_rel"] <= TOL["train_weight"],
        "export_eval": r["export_unpatched"] and r["export_eval_rel"] <= TOL["export_eval"],
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
    ap.add_argument("--mps-fix", action="store_true", help="两边都换上 mps_fix 的等价卷积")
    ap.add_argument("--out", default=None)
    a = ap.parse_args(argv)
    torch.manual_seed(0)
    GTCRN, HybridLoss = st.load_official(a.gtcrn_dir)
    model = GTCRN()
    if a.ckpt:
        model.load_state_dict(torch.load(a.ckpt, map_location="cpu", weights_only=False)["model"])
    patched = []
    if a.mps_fix:
        import mps_fix as fx  # 同目录；deepcopy 会带上换过的 forward
        patched = fx.patch(model)
    r = compare(model, a.device, HybridLoss, a.batch, a.seconds, a.steps, a.lr)
    r.update(verdict(r))
    r.update(mps_fix_patched=patched, batch=a.batch, seconds_per_clip=a.seconds, lr=a.lr,
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
