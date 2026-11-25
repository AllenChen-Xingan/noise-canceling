"""过拟合慢的诊断（大总管 2026-11-25）：`smoke.py --overfit-probe <文件>`。只诊断，不判过不过、不改判据。

同一小批（开发份第 0 批前 4 段，与判据第 3 版同一批）上做三件事：
1. 学习曲线：lr 1e-3 跑到 probe_steps 步，在 150 · 300 · 500 · 1000 · 2000 步记损失、走完 oracle 距离的比例、SI-SNR
2. 哪些参数在动：可训练参数数、按层分组；批归一化是否全在评估模式、缩放偏移是否照训；第 1 步与最后 1 步各组梯度范数
3. 学习率：3e-4 与 3e-3 各跑 sweep_steps 步（默认 300），同样记录
4–6. 数据侧（`data_probe.py`）：输入与目标的对齐 · 目标本身（静音、削波、电平）与 wav · 逐个关配方旋钮各 150 步
跑的顺序按便宜的先来：数据检查 → 关旋钮 → 学习曲线 → 学习率。
每记一个点就重写报告（原子改名），中途被杀也留下已跑完的部分（complete 为 False）。
"""
from __future__ import annotations

import json
import os
import time
from pathlib import Path

import torch

import config as cfgmod
import data_probe
import losses
import oracle
import smoke
import train
from data import import_loader, make_source

MARKS = (150, 300, 500, 1000, 2000)
SWEEP_LRS = (3e-4, 3e-3)


def marks_for(steps: int) -> list[int]:
    ms = [k for k in MARKS if k < steps]
    return ms + [steps]


def _group(name: str) -> str:
    return ".".join(name.split(".")[:2])


def param_report(model) -> dict:
    groups: dict[str, dict] = {}
    for n, p in model.named_parameters():
        g = groups.setdefault(_group(n), {"params": 0, "trainable": 0})
        g["params"] += p.numel()
        g["trainable"] += p.numel() if p.requires_grad else 0
    bns = [m for m in model.modules() if isinstance(m, torch.nn.modules.batchnorm._BatchNorm)]
    return {"trainable": sum(g["trainable"] for g in groups.values()),
            "total": sum(g["params"] for g in groups.values()), "groups": groups,
            "frozen_param_names": [n for n, p in model.named_parameters() if not p.requires_grad],
            "bn_layers": len(bns), "bn_in_train_mode": sum(m.training for m in bns),
            "bn_affine_trainable": all(p.requires_grad for m in bns for p in m.parameters())}


def grad_norms(model) -> dict:
    """各组（参数名前两段）的梯度 L2 范数；没有梯度的可训练参数按 0 计。"""
    acc: dict[str, float] = {}
    for n, p in model.named_parameters():
        if not p.requires_grad:
            continue
        g = 0.0 if p.grad is None else float(p.grad.detach().double().pow(2).sum())
        acc[_group(n)] = acc.get(_group(n), 0.0) + g
    return {k: v ** 0.5 for k, v in acc.items()}


@torch.no_grad()
def _eval(m, crit, batch, device) -> tuple[float, float]:
    return train.forward_loss(m, crit, batch, device)[0].item(), smoke.batch_si_snr(m, batch, device)


def run_curve(cfg, batch, device, lr: float, steps: int, oracle_loss: float, on_mark=lambda r: None) -> dict:
    m = smoke._model(cfg, device)
    train.set_train(m, cfg)
    crit = losses.TrainLoss(losses.LossCfg()).to(device)
    opt = torch.optim.Adam([p for p in m.parameters() if p.requires_grad], lr=lr)
    loss0, snr0 = _eval(m, crit, batch, device)
    marks = set(marks_for(steps))
    rec = {"lr": lr, "steps": steps, "loss0": loss0, "si_snr_db0": snr0, "marks": [], "traj_every_10": []}
    t0 = time.monotonic()
    for k in range(1, steps + 1):
        loss, logs = train.forward_loss(m, crit, batch, device)
        opt.zero_grad(set_to_none=True)
        loss.backward()
        if k == 1:
            rec["grad_norms_first_step"] = grad_norms(m)
        if k == steps:
            rec["grad_norms_last_step"] = grad_norms(m)
        opt.step()
        if k % 10 == 0:
            rec["traj_every_10"].append(logs["total"])
        if k in marks:
            lk, sk = _eval(m, crit, batch, device)
            rec["marks"].append({"step": k, "loss": lk, "drop_frac_of_oracle_span": oracle.frac_of_span(loss0, lk, oracle_loss),
                                 "si_snr_db": sk, "si_snr_gain_db": sk - snr0, "elapsed_s": time.monotonic() - t0})
            on_mark(rec)
    return rec


def _write(out: Path, rep: dict) -> None:
    tmp = out.with_name(out.name + f".{os.getpid()}.tmp")
    tmp.write_text(json.dumps(rep, ensure_ascii=False, indent=2, default=str))
    os.replace(tmp, out)


def run_probe(cfg, out: Path, steps: int, sweep_steps: int) -> int:
    if out.exists():
        print(f"拒绝覆盖已有文件：{out}")
        return 2
    out.parent.mkdir(parents=True, exist_ok=True)
    device = train.resolve_device(cfg.device)
    source = make_source(cfg)
    batch = smoke.head(source.dev_batches()[0], smoke.OVERFIT_N)
    crit = losses.TrainLoss(losses.LossCfg()).to(device)
    o = oracle.oracle_loss(crit, batch, device)
    m = smoke._model(cfg, device)
    train.set_train(m, cfg)
    rep = {"complete": False, "n_segments": smoke.OVERFIT_N, "batch_index": 0, "build_id": source.build_id,
           "device": str(device), "torch": torch.__version__, "config": cfgmod.to_dict(cfg),
           "oracle_loss": o["loss"], "oracle_si_snr_db": o["si_snr_db"], "params": param_report(m),
           "note": "诊断用，不判过不过、不改判据（大总管 2026-11-25）", "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    del m
    _write(out, rep)

    # 数据侧：对齐 · 目标 · wav（4 段）
    maker, recipe, metas = None, None, None
    if cfg.data.kind == "loader":
        recipe = json.loads(Path(cfg.data.recipe).read_text())
        maker = data_probe.loader_batch_maker(import_loader(), cfg, out.parent / f"{out.stem}-recipes", smoke.OVERFIT_N)
        b0 = maker(recipe, "baseline")
        metas = b0["metas"]
        rep["baseline_rebuild_max_abs_diff"] = float((b0["noisy"] - batch["noisy"]).abs().max())
    rep["data_checks"] = {"alignment": data_probe.alignment(batch["noisy"], batch["clean"]),
                          "targets": data_probe.target_stats(batch["clean"], batch["noisy"], metas, recipe),
                          "wavs": data_probe.write_wavs(out.parent / f"{out.stem}-wav", batch["noisy"], batch["clean"])}
    _write(out, rep)
    if maker is None:
        rep["ablation"] = {"skipped": "合成数据没有配方"}
    else:
        rep["ablation"] = {}

        def save_abl(name, r):
            rep["ablation"][name] = r
            _write(out, rep)
        data_probe.run_ablations(cfg, device, recipe, maker, 150, run_curve, on_entry=save_abl)
    _write(out, rep)

    def save_curve(r):
        rep["curve"] = r
        _write(out, rep)
    rep["curve"] = run_curve(cfg, batch, device, 1e-3, steps, o["loss"], on_mark=save_curve)
    _write(out, rep)
    rep["lr_sweep"] = []
    for lr in SWEEP_LRS:
        i = len(rep["lr_sweep"])
        rep["lr_sweep"].append({"lr": lr, "steps": sweep_steps, "marks": []})  # 占位：跑完前被杀也看得出跑到哪个

        def save_sweep(r, _i=i):
            rep["lr_sweep"][_i] = r
            _write(out, rep)
        rep["lr_sweep"][i] = run_curve(cfg, batch, device, lr, sweep_steps, o["loss"], on_mark=save_sweep)
        _write(out, rep)
    rep["complete"] = True
    rep["finished_at"] = time.strftime("%Y-%m-%dT%H:%M:%S%z")
    _write(out, rep)
    print(json.dumps({k: rep[k] for k in ("oracle_loss", "complete")}, ensure_ascii=False))
    return 0
