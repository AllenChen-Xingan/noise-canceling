"""exp018 冒烟：GTCRN 单个训练步（前向 + 反向 + Adam 更新）在 MPS / CPU 上的耗时。

只量速度，不训练：输入是随机噪声，权重从官方检查点载入（只读 import 官方代码，不抄）。
一步 = 波形 STFT → 模型 → 官方 HybridLoss → backward → Adam.step，和真实训练同一条路径。

用法（单次 ≤ 约 1 分钟，不占重活名额）：
    python step_timing.py --gtcrn-dir <resources/gtcrn> --device mps --batch 8 --seconds 4 \
        --out <主目录>/_tmp/027/<run-id>/timing-mps-b8.json
"""
from __future__ import annotations

import argparse
import json
import os
import platform
import statistics
import sys
import time
import warnings

# MPS 不支持的算子回落到 CPU，并由下面的 warnings 记下来；必须在 import torch 之前设置
os.environ.setdefault("PYTORCH_ENABLE_MPS_FALLBACK", "1")

import torch  # noqa: E402

N_FFT, HOP, WIN = 512, 256, 512  # 与官方 infer.py / loss.py 一致
FS = 16000


def load_official(gtcrn_dir: str):
    """从官方目录 import GTCRN 与 HybridLoss（只读引用，不复制代码）。"""
    sys.path.insert(0, gtcrn_dir)
    from gtcrn import GTCRN  # type: ignore
    from loss import HybridLoss  # type: ignore
    return GTCRN, HybridLoss


def stft(x: torch.Tensor) -> torch.Tensor:
    win = torch.hann_window(WIN, device=x.device).pow(0.5)
    return torch.stft(x, N_FFT, HOP, WIN, window=win, return_complex=True)


def to_real(spec_c: torch.Tensor) -> torch.Tensor:
    """复数谱 (B,F,T) → 官方模型要的 (B,F,T,2)。"""
    return torch.view_as_real(spec_c).contiguous()


def sync(device: torch.device) -> None:
    if device.type == "mps":
        torch.mps.synchronize()


def fallback_ops(caught: list) -> list[str]:
    """从 warnings 里挑出「MPS 不支持、回落 CPU」的算子名，去重保序。"""
    ops: list[str] = []
    for w in caught:
        msg = str(w.message)
        if "fall back" in msg or "MPS backend" in msg:
            name = msg.split("'")[1] if msg.count("'") >= 2 else msg[:120]
            if name not in ops:
                ops.append(name)
    return ops


def run(gtcrn_dir: str, device_name: str, batch: int, seconds: float,
        warmup: int, steps: int, ckpt: str | None, mps_fix: bool = False) -> dict:
    device = torch.device(device_name)
    GTCRN, HybridLoss = load_official(gtcrn_dir)
    model = GTCRN()
    if ckpt:
        state = torch.load(ckpt, map_location="cpu", weights_only=False)
        model.load_state_dict(state["model"])
    patched = []
    if mps_fix:
        import mps_fix as fx  # 同目录
        patched = fx.patch(model)
    model = model.to(device).train()
    loss_fn = HybridLoss()
    opt = torch.optim.Adam(model.parameters(), lr=1e-4)
    n_params = sum(p.numel() for p in model.parameters())

    g = torch.Generator().manual_seed(0)
    n = int(seconds * FS)
    clean = (0.1 * torch.randn(batch, n, generator=g)).to(device)
    noisy = clean + (0.1 * torch.randn(batch, n, generator=g)).to(device)

    times: list[float] = []
    losses: list[float] = []
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        for i in range(warmup + steps):
            sync(device)
            t0 = time.perf_counter()
            x = to_real(stft(noisy))
            y = to_real(stft(clean))
            pred = model(x)
            loss = loss_fn(pred, y)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            opt.step()
            sync(device)
            dt = time.perf_counter() - t0
            if i >= warmup:
                times.append(dt)
                losses.append(float(loss.detach().cpu()))
    audio_s = batch * seconds
    med = statistics.median(times)
    return {
        "device": device_name,
        "batch": batch,
        "seconds_per_clip": seconds,
        "warmup": warmup,
        "steps": steps,
        "checkpoint": os.path.basename(ckpt) if ckpt else None,
        "mps_fix_patched": patched,
        "params": n_params,
        "step_s_median": med,
        "step_s_min": min(times),
        "step_s_max": max(times),
        "audio_s_per_step": audio_s,
        "audio_hours_per_wall_hour": audio_s / med,
        "loss_finite": all(map(lambda v: v == v and abs(v) != float("inf"), losses)),
        "mps_fallback_ops": fallback_ops(caught) if device.type == "mps" else [],
        "torch": torch.__version__,
        "machine": platform.machine(),
        "mac_ver": platform.mac_ver()[0],
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--gtcrn-dir", required=True)
    ap.add_argument("--device", default="cpu", choices=["cpu", "mps"])
    ap.add_argument("--batch", type=int, default=8)
    ap.add_argument("--seconds", type=float, default=4.0)
    ap.add_argument("--warmup", type=int, default=3)
    ap.add_argument("--steps", type=int, default=10)
    ap.add_argument("--ckpt", default=None, help="官方检查点；不给则随机初始化")
    ap.add_argument("--mps-fix", action="store_true", help="换掉 MPS 上算错的空洞逐通道卷积（mps_fix.py）")
    ap.add_argument("--out", default=None, help="写 JSON 的路径；不给则只打印")
    a = ap.parse_args(argv)
    res = run(a.gtcrn_dir, a.device, a.batch, a.seconds, a.warmup, a.steps, a.ckpt, a.mps_fix)
    text = json.dumps(res, ensure_ascii=False, indent=2)
    if a.out:
        os.makedirs(os.path.dirname(a.out) or ".", exist_ok=True)
        if os.path.exists(a.out):
            raise SystemExit(f"拒绝覆盖已有文件：{a.out}")
        with open(a.out, "w", encoding="utf-8") as f:
            f.write(text + "\n")
    print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
