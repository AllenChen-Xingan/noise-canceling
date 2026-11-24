"""开训前冒烟（014 第 3 步）：一个命令跑完三件事 + 测每步耗时，三件全过才进下一步。

    python smoke.py --config configs/smoke-cpu.yaml [--set …] [--out 报告.json]

1. **初始损失合理**（034 修正：微调时看「等于原检查点的损失」，不看随机初始化的理论值）：
   B 零初始化时与 A 在同一批上的损失相同（相对差 ≤ 1e-5），且低于随机初始化模型的损失
2. **单批能过拟合**：同一批反复训 overfit_steps 步，把「初值到完美输出损失」的距离走完 ≥ 一半；B 的注意力层要动过
3. **原检查点读数复现**：起点权重在官方样例 `mix.wav` 上的输出与官方 `enh.wav` 最大差 ≤ 2 个 16 位量化步（2/32768）；
   云上另可用 `--known-dev` 给出开发份已知读数比对（evalcmp 出数后接）
4. **耗时**：正式配置下每步墙钟与 CPU 时间、进程 CPU 占用（不判过线，只记；D96 要看 CPU 是不是瓶颈）
报告带电源状态与设备；本机跑请套 `caffeinate -dims`（电池下会睡眠降频，见本目录 README）。
"""
from __future__ import annotations

import argparse
import json
import platform
import resource
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf
import torch

import config as cfgmod
import losses
import train
from data import make_source
from official import gtcrn_dir

LSB16 = 1 / 32768


def power_state() -> str:
    if platform.system() != "Darwin":
        return "n/a"
    r = subprocess.run(["pmset", "-g", "batt"], capture_output=True, text=True)
    return "ac" if "AC Power" in r.stdout else "battery"


def _model(cfg, device, **over):
    c = cfgmod.from_dict({**cfgmod.to_dict(cfg), "model": {**cfgmod.to_dict(cfg)["model"], **over}})
    return train.build_model(c, device)


def check_init(cfg, batch, device) -> dict:
    crit = losses.TrainLoss(losses.LossCfg()).to(device)
    vals = {}
    with torch.no_grad():
        for tag, over in (("A", {"variant": "A"}), ("B", {"variant": "B"}), ("random", {"init": "random"})):
            m = _model(cfg, device, **over)
            train.set_train(m, cfg)  # 与训练同一模式：bn = frozen 时三者都用起点统计量
            vals[tag] = train.forward_loss(m, crit, batch, device)[0].item()
    rel = abs(vals["B"] - vals["A"]) / max(abs(vals["A"]), 1e-9)
    ok = rel <= 1e-5 and np.isfinite(vals["A"]) and vals["A"] < vals["random"]
    return {"pass": bool(ok), "loss": vals, "rel_diff_B_vs_A": rel,
            "rule": "B 与 A 相对差 ≤ 1e-5，且起点损失 < 随机初始化"}


def overfit_pass(first: float, last: float, floor: float) -> bool:
    """单批过拟合判据：把「初值到完美输出的距离」走完至少一半，即 (first − last) / (first − floor) ≥ 0.5。
    floor = 输出恰好等于目标时的损失。不按 |初值| 归一：SI-SNR 项是负对数，总损失会跨零（自审 P2）。"""
    span = first - floor
    return bool(np.isfinite(last) and span > 0 and (first - last) / span >= 0.5)


def check_overfit(cfg, batch, device, steps: int, lr: float) -> dict:
    """同一批反复训。B 另要求注意力输出层的权重动过（零初始化，只靠基座也能降一半，自审 P2）。
    条件口子本次喂全零（D98），W·0 对 W 的梯度恒为 0，它不动是预期，不作要求。"""
    m = _model(cfg, device)
    train.set_train(m, cfg)
    crit = losses.TrainLoss(losses.LossCfg()).to(device)
    with torch.no_grad():
        clean = batch["clean"].to(device)
        fr = train.frames_mask(batch, clean.shape[1]).to(device)
        floor = crit(losses.stft(clean), clean, fr)[0].item()
    attn0 = m.attn.out.weight.detach().clone() if getattr(m, "attn", None) is not None else None
    opt = torch.optim.Adam([p for p in m.parameters() if p.requires_grad], lr=lr)
    traj = []
    for _ in range(steps):
        loss, logs = train.forward_loss(m, crit, batch, device)
        opt.zero_grad(set_to_none=True)
        loss.backward()
        opt.step()
        traj.append(logs["total"])
    moved = None if attn0 is None else float((m.attn.out.weight.detach() - attn0).abs().max())
    ok = overfit_pass(traj[0], traj[-1], floor) and (moved is None or moved > 0)
    span = traj[0] - floor
    return {"pass": ok, "first": traj[0], "last": traj[-1], "floor": floor,
            "drop_frac_of_span": (traj[0] - traj[-1]) / span if span > 0 else None,
            "attn_out_max_change": moved, "steps": steps, "lr": lr,
            "rule": "(初值 − 终值) / (初值 − 完美输出的损失) ≥ 0.5；B 的注意力输出层权重要动过"}


def check_reproduce(device) -> dict:
    d = gtcrn_dir() / "stream" / "test_wavs"
    mix, fs = sf.read(d / "mix.wav", dtype="float32")
    ref, _ = sf.read(d / "enh.wav", dtype="float32")
    assert fs == 16000
    cfg = cfgmod.from_dict({"model": {"variant": "A"}, "device": "cpu"})
    m = train.build_model(cfg, torch.device("cpu")).eval()  # 复现在 CPU 上做：比的是权重，不是设备
    x = torch.as_tensor(mix)[None]
    with torch.no_grad():
        y = losses.istft(m(losses.stft(x)), x.shape[1])[0].numpy()
    n = min(len(y), len(ref))
    diff = float(np.abs(y[:n] - ref[:n]).max())
    return {"pass": diff <= 2 * LSB16, "max_abs_diff": diff, "lsb16": LSB16, "samples": n,
            "rule": "最大差 ≤ 2 个 16 位量化步"}


def check_known_dev(cfg, dev, device, known: dict, tol: float) -> dict:
    """开发份已知读数（键 = 损失日志名，如 hybrid）与起点模型实测比对。"""
    m = _model(cfg, device, variant="A").eval()
    got = train.evaluate(m, losses.TrainLoss(losses.LossCfg()).to(device), dev, device, cfg)
    bad = {k: (got.get(k), v) for k, v in known.items() if abs(got.get(k, float("inf")) - v) > tol}
    return {"pass": not bad, "got": got, "known": known, "tol": tol, "mismatch": bad}


def timing(cfg, source, device, steps: int) -> dict:
    m = _model(cfg, device).train()
    crit = losses.TrainLoss(losses.LossCfg(w_lvl=1.0, w_on=1.0)).to(device)
    opt = torch.optim.Adam([p for p in m.parameters() if p.requires_grad], lr=1e-5)
    it = source.train_batches(0)
    t_data, t_step = [], []
    ru0 = resource.getrusage(resource.RUSAGE_SELF)
    w0 = time.perf_counter()
    for _ in range(steps):
        a = time.perf_counter()
        b = next(it)
        t_data.append(time.perf_counter() - a)
        a = time.perf_counter()
        loss, _ = train.forward_loss(m, crit, b, device)
        opt.zero_grad(set_to_none=True)
        loss.backward()
        opt.step()
        if device.type == "cuda":
            torch.cuda.synchronize()
        t_step.append(time.perf_counter() - a)
    wall = time.perf_counter() - w0
    ru1 = resource.getrusage(resource.RUSAGE_SELF)
    cpu = (ru1.ru_utime - ru0.ru_utime) + (ru1.ru_stime - ru0.ru_stime)
    return {"steps": steps, "batch": cfg.data.batch, "seg_s": cfg.data.seg_s,
            "data_s_median": float(np.median(t_data)), "step_s_median": float(np.median(t_step[1:] or t_step)),
            "wall_s": wall, "cpu_s_main_process": cpu, "cpu_util_main_process": cpu / wall,
            "note": "DataLoader 子进程的 CPU 不在此数内；含两项保护损失"}


def run(cfg, *, overfit_steps: int, overfit_lr: float, timing_steps: int,
        known_dev: dict | None, known_tol: float) -> dict:
    device = train.resolve_device(cfg.device)
    source = make_source(cfg)
    dev = source.dev_batches()
    batch = dev[0]
    rep = {"device": str(device), "power": power_state(), "torch": torch.__version__,
           "build_id": source.build_id, "config": cfgmod.to_dict(cfg),
           "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    rep["init_loss"] = check_init(cfg, batch, device)
    rep["overfit"] = check_overfit(cfg, batch, device, overfit_steps, overfit_lr)
    rep["reproduce"] = check_reproduce(device)
    if known_dev:
        rep["known_dev"] = check_known_dev(cfg, dev, device, known_dev, known_tol)
    rep["timing"] = timing(cfg, source, device, timing_steps)
    checks = [k for k in ("init_loss", "overfit", "reproduce", "known_dev") if k in rep]
    rep["pass"] = all(rep[k]["pass"] for k in checks)
    return rep


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="exp025 开训前冒烟")
    ap.add_argument("--config", required=True)
    ap.add_argument("--set", action="append", default=[])
    ap.add_argument("--overfit-steps", type=int, default=150)
    ap.add_argument("--overfit-lr", type=float, default=1e-3)
    ap.add_argument("--timing-steps", type=int, default=10)
    ap.add_argument("--known-dev", help="开发份已知读数 JSON 文件，如 {\"hybrid\": 1.23}")
    ap.add_argument("--known-tol", type=float, default=1e-3)
    ap.add_argument("--out", help="报告写到这里（JSON）")
    a = ap.parse_args(argv)
    cfg = cfgmod.load(a.config, train.parse_set(a.set))
    known = json.loads(Path(a.known_dev).read_text()) if a.known_dev else None
    rep = run(cfg, overfit_steps=a.overfit_steps, overfit_lr=a.overfit_lr, timing_steps=a.timing_steps,
              known_dev=known, known_tol=a.known_tol)
    text = json.dumps(rep, ensure_ascii=False, indent=2, default=str)
    if a.out:
        Path(a.out).write_text(text)
    print(text)
    return 0 if rep["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
