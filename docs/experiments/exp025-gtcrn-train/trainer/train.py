"""训练脚本（014 第 4–5 步的代码）。

用法（在本目录）：
    python train.py --config configs/default.yaml --set run.name=exp025-B-s0 --set data.shard_root=… --set data.recipe=…
同一个 run.name 再跑一次 = 从最新检查点续训（配置只许改 `runrec.RESUMABLE_CHANGES` 里那几项）。

- 优化器 Adam，梯度裁剪，学习率前 warmup_steps 步线性升到 lr、之后不变（027 3.1）
- 设备 auto：cuda > mps（打 exp018 的绕法，导出时用原模块）> cpu
- 到 max_steps、计费时长 ≥ max_hours、计费时长 × 单价 ≥ max_usd、收到 SIGTERM / SIGINT、取数超时或数据用完，
  都存检查点（并补一次开发评估）停下，原因写进 run.json。计费时长含开跑准备，续训时按心跳文件补上被抢占丢的那段
- 损失不是有限数：不存坏权重，记 `stop.reason = nonfinite` 停下；之后拒绝在同一 run.name 上续训（014 意外矩阵：训崩记进清单不删）
- 批归一化默认冻结统计量（`model.bn = frozen`），训练与推理一致
"""
from __future__ import annotations

import argparse
import importlib.util
import math
import signal
import sys
import threading
import time
from pathlib import Path

import torch
import yaml

import config as cfgmod
import losses
import models
from data import frames_mask, make_source
from official import load_start_state
from runrec import EMA, RunDir, check_resume, env_info, start_record

HERE = Path(__file__).resolve().parent
MPS_FIX = HERE.parents[1] / "exp018-gtcrn-finetune" / "smoke" / "mps_fix.py"


def resolve_device(name: str) -> torch.device:
    if name != "auto":
        return torch.device(name)
    if torch.cuda.is_available():
        return torch.device("cuda")
    if torch.backends.mps.is_available():
        return torch.device("mps")
    return torch.device("cpu")


def mps_patch(model: torch.nn.Module) -> list[str]:
    spec = importlib.util.spec_from_file_location("exp018_mps_fix", MPS_FIX)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.patch(model)


def build_model(cfg, device: torch.device) -> torch.nn.Module:
    m = cfg.model
    kw = {} if m.variant == "A" else dict(k_frames=m.k_frames, heads=m.heads, cond_dim=m.cond_dim,
                                          attn=m.attn, film=m.film)
    torch.manual_seed(cfg.run.seed)  # B 的 q / kv 随机初始化由种子定
    model = models.build(m.variant, **kw)
    if m.init == "official":
        model.load_base(load_start_state())
    model = model.to(device)
    if device.type == "mps":
        mps_patch(model)
    return model


def cond_for(model, batch_size: int, device) -> torch.Tensor | None:
    """本次条件输入一律全零（D98）；以后喂场景向量 / 声纹改这里。"""
    film = getattr(model, "film", None)
    return None if film is None else torch.zeros(batch_size, film.d, device=device)


def to_dev(batch: dict, device) -> tuple:
    n = batch["clean"].shape[1]
    return (batch["noisy"].to(device), batch["clean"].to(device),
            frames_mask(batch, n).to(device), batch["replay"].to(device))


def forward_loss(model, crit, batch, device, *, sp_ref=None, teacher=None):
    noisy, clean, fr, replay = to_dev(batch, device)
    spec = losses.stft(noisy)
    pred = model(spec, cond_for(model, noisy.shape[0], device))
    t_spec = None
    if teacher is not None and replay.any():
        with torch.no_grad():
            t_spec = teacher(spec, cond_for(teacher, noisy.shape[0], device))
    return crit(pred, clean, fr, model=model, sp_ref=sp_ref, teacher_spec=t_spec, replay=replay)


def loss_cfg(cfg, w_lvl: float, w_on: float) -> losses.LossCfg:
    lc = cfg.loss
    return losses.LossCfg(w_lvl=w_lvl, w_on=w_on, lvl_margin_db=lc.lvl_margin_db, sp=lc.sp, w_sp=lc.w_sp,
                          kd=lc.kd, w_kd=lc.w_kd, level_norm=lc.level_norm, level_floor=lc.level_floor)


@torch.no_grad()
def calibrate(model, cfg, batches, device) -> dict:
    """027 3.2 权重第 1 步：起点模型在 calib_batches 个开发批上量各项大小，把 "auto" 的权重定成约占 HybridLoss 的 10%。"""
    pc = loss_cfg(cfg, 1.0, 1.0)
    pc.sp, pc.kd = False, False  # 标定只量 HybridLoss 与两项保护
    probe = losses.TrainLoss(pc).to(device)
    model.eval()
    sums = {"hybrid": 0.0, "lvl": 0.0, "on": 0.0}
    valid = {"lvl": 0, "on": 0}
    for b in batches:
        _, logs = forward_loss(model, probe, b, device)
        sums["hybrid"] += logs["hybrid"]
        for k in ("lvl", "on"):
            sums[k] += logs[k] * logs[f"{k}_valid"]
            valid[k] += logs[f"{k}_valid"]
    set_train(model, cfg)
    nb = len(batches)
    h = sums["hybrid"] / nb
    out = {"batches": nb, "hybrid_mean": h}
    for k, key in (("lvl", "w_lvl"), ("on", "w_on")):
        mean = sums[k] / valid[k] if valid[k] else 0.0
        v = getattr(cfg.loss, key)
        out[k] = {"mean_valid": mean, "valid_segments": valid[k],
                  "weight": losses.calibrate_weight(h, mean) if v == "auto" else float(v), "auto": v == "auto"}
    return out


@torch.no_grad()
def evaluate(model, crit, dev, device, cfg, **kw) -> dict:
    """开发份平均读数。每个键按「出现过的批数」平均（kd 只在有回放样本的批出现，不按总批数摊薄）。"""
    model.eval()
    acc, cnt = {}, {}
    for b in dev:
        _, logs = forward_loss(model, crit, b, device, **kw)
        for k, v in logs.items():
            acc[k] = acc.get(k, 0.0) + v
            cnt[k] = cnt.get(k, 0) + 1
    set_train(model, cfg)
    return {k: v / cnt[k] for k, v in acc.items()}


def set_train(model, cfg) -> None:
    """训练模式。bn = frozen（默认）时批归一化层留在评估模式：用起点的运行统计量、不更新，缩放与偏移照训。
    理由（自审 P1）：训练模式的批归一化用整段（含未来帧）与整批的统计量，和推理不一致；微调时还会冲掉 DNS3 的统计量。"""
    model.train()
    if cfg.model.bn == "frozen":
        for m in model.modules():
            if isinstance(m, torch.nn.modules.batchnorm._BatchNorm):
                m.eval()


def _lr_lambda(warmup: int):
    return lambda s: min(1.0, (s + 1) / warmup) if warmup > 0 else 1.0


class _Signals:
    """收到 SIGTERM / SIGINT（RunPod 抢占、人按 Ctrl-C）时置位，循环在本步结束后存检查点停下。"""

    def __init__(self):
        self.got = None
        self.old = {}

    def __enter__(self):
        if threading.current_thread() is threading.main_thread():
            for sig in (signal.SIGTERM, signal.SIGINT):
                self.old[sig] = signal.signal(sig, self._on)
        return self

    def _on(self, signum, frame):
        self.got = signal.Signals(signum).name

    def __exit__(self, *exc):
        for sig, h in self.old.items():
            signal.signal(sig, h)


HEARTBEAT_S = 30.0


class _Beat:
    """后台线程每 HEARTBEAT_S 秒写一次心跳（步数 · 计费秒 · 累计美元），取数卡住、算标定时也照写。
    被抢占时最多漏记一个间隔（≤ 30 s）；机器的真实账单归 cloudrun（codex 第 1 轮 #4）。心跳只由这个线程写。"""

    def __init__(self, rd, snap, every: float):
        self.rd, self.snap, self.every = rd, snap, every
        self.stop = threading.Event()
        self.th = threading.Thread(target=self._run, daemon=True)

    def _run(self):
        while not self.stop.wait(self.every):
            self.rd.beat(*self.snap())

    def __enter__(self):
        self.th.start()
        return self

    def __exit__(self, *exc):
        self.stop.set()
        self.th.join()
        self.rd.beat(*self.snap())


def train(cfg, *, max_wall_s: float | None = None, _signals: _Signals | None = None,
          _beat_s: float = HEARTBEAT_S) -> dict:
    """跑到停下为止，返回停下记录。max_wall_s / _signals / _beat_s 只给测试用。

    计费（billed_s · usd）从本函数开始算，含建模型、取开发份、标定；续训时接上次检查点与心跳里较大的那个。
    美元按段累计：每段只按本段单价乘本段时长，改单价不追溯（codex 第 1 轮 #6）。上限按这两个数判。"""
    t_proc = time.monotonic()
    device = resolve_device(cfg.device)
    cfg_dict = cfgmod.to_dict(cfg)
    rd = RunDir(Path(cfg.run.out_dir).expanduser() / cfg.run.name)
    source = make_source(cfg)
    dev = source.dev_batches()
    model = build_model(cfg, device)
    sp_ref = {k: v.detach().clone() for k, v in model.named_parameters()} if cfg.loss.sp else None
    teacher = None
    if cfg.loss.kd:
        teacher = build_model(cfg, device).eval()
        for p in teacher.parameters():
            p.requires_grad_(False)
    opt = torch.optim.Adam([p for p in model.parameters() if p.requires_grad], lr=cfg.optim.lr,
                           betas=tuple(cfg.optim.betas))
    sched = torch.optim.lr_scheduler.LambdaLR(opt, _lr_lambda(cfg.optim.warmup_steps))
    ema = EMA(model, cfg.ema.decay) if cfg.ema.enabled else None

    st, last = rd.load_latest(device)
    if st is None and (rd.run_json.exists() or any(rd.ckpt_dir.glob("step-*.pt"))):
        # 已有运行却一份检查点都读不出：不当成新运行（会清零花费、覆盖 run.json、跳过训崩检查，codex 第 1 轮 #5）
        raise RuntimeError(f"{rd.root} 已有运行记录，但没有可读的检查点（见 ckpt/bad_ckpt.txt）；请人工处理或换 run.name")
    if st is not None:
        rec = rd.read()
        why = check_resume(rec, cfg_dict, source.build_id)
        if why:
            raise RuntimeError("拒绝续训：" + "；".join(why))
        model.load_state_dict(st["model"])
        opt.load_state_dict(st["optim"])
        sched.load_state_dict(st["sched"])
        if ema is not None:
            ema.load_state_dict(st["ema"])
        if sp_ref is not None:
            sp_ref = {k: v.to(device) for k, v in st["sp_ref"].items()}
        step, calib = st["step"], st["calib"]
        hb, prev = rd.heartbeat(), rec.get("stop") or {}
        billed0 = max(st["billed_s"], hb.get("billed_s", 0.0), prev.get("billed_s", 0.0))  # 三处取最晚的
        usd0 = max(st.get("usd", 0.0), hb.get("usd", 0.0), prev.get("usd", 0.0))
        rec["config"] = cfg_dict
        rec["resumes"].append({"at": time.strftime("%Y-%m-%dT%H:%M:%S%z"), "from_step": step,
                               "from_ckpt": last.name, "billed_s_before": round(billed0, 2),
                               "env": env_info(), "prev_stop": rec.pop("stop", None)})
    else:
        calib = calibrate(model, cfg, source.dev_batches(cfg.data.calib_batches), device)
        step, billed0, usd0 = 0, 0.0, 0.0
        pcnt = models.param_counts(model)
        rec = start_record(cfg_dict, source.build_id, source.num_workers,
                           {"device": str(device), "params": pcnt["trainable"], "params_fixed": pcnt["fixed"],
                            "calib": calib, "env": env_info()})
    rd.write(rec)
    lim, sc = cfg.limits, cfg.schedule
    crit = losses.TrainLoss(loss_cfg(cfg, calib["lvl"]["weight"], calib["on"]["weight"])).to(device)
    kw = {"sp_ref": sp_ref, "teacher": teacher}

    def billed():
        return billed0 + time.monotonic() - t_proc

    def usd():
        return usd0 + (time.monotonic() - t_proc) / 3600 * lim.usd_per_hour

    def state():
        return {"model": model.state_dict(), "optim": opt.state_dict(), "sched": sched.state_dict(),
                "ema": ema.state_dict() if ema else None, "sp_ref": sp_ref, "step": step,
                "billed_s": billed(), "usd": usd(), "calib": calib, "config": cfg_dict,
                "variant": cfg.model.variant}

    def dev_eval():
        row = {"step": step, "dev": evaluate(model, crit, dev, device, cfg, **kw)}
        if ema is not None:  # 034 建议 7：EMA 与不用 EMA 在开发份上并排比
            raw = {k: v.clone() for k, v in model.state_dict().items()}
            model.load_state_dict(ema.state_dict())
            row["dev_ema"] = evaluate(model, crit, dev, device, cfg, **kw)
            model.load_state_dict(raw)
        rd.log(row)

    if st is None:
        rd.save(state(), 0, cfg.schedule.keep_last)
    t_start = time.monotonic()
    reason, last_eval = None, step
    set_train(model, cfg)
    batches = source.train_batches(start_batch=step, device=device)
    with (_signals or _Signals()) as sigs, _Beat(rd, lambda: (step, billed(), usd()), _beat_s):
        while True:
            if step >= sc.max_steps:
                reason = "max_steps"
            elif billed() / 3600 >= lim.max_hours:
                reason = "max_hours"
            elif usd() >= lim.max_usd:
                reason = "max_usd"
            elif sigs.got:
                reason = f"signal:{sigs.got}"
            elif max_wall_s is not None and time.monotonic() - t_start >= max_wall_s:
                reason = "interrupted"
            if reason:
                break
            try:
                batch = next(batches)
            except StopIteration:
                reason = "data_exhausted"
                break
            except RuntimeError as e:  # DataLoader 的 timeout：worker 卡住时不无限等（自审 P1）
                if "timed out" not in str(e).lower():
                    raise
                reason = "data_timeout"
                break
            if batch.get("batch_index", step) != step:  # 第 step 步必须拿第 step 批（续训对齐的交叉校验）
                raise RuntimeError(f"数据批次错位：第 {step} 步拿到第 {batch['batch_index']} 批")
            loss, logs = forward_loss(model, crit, batch, device, sp_ref=sp_ref, teacher=teacher)
            if not math.isfinite(loss.item()):
                reason = "nonfinite"
                rec["crash"] = {"step": step, "logs": logs}
                break
            opt.zero_grad(set_to_none=True)
            loss.backward()
            gn = torch.nn.utils.clip_grad_norm_(model.parameters(), cfg.optim.grad_clip)
            if not torch.isfinite(gn):  # 损失有限、梯度却不是：不更新，否则 Adam 会把 NaN 写进权重（codex 第 1 轮 #2）
                reason = "nonfinite"
                rec["crash"] = {"step": step, "logs": logs, "grad_norm": float(gn)}
                break
            opt.step()
            sched.step()
            if ema is not None:
                ema.update(model)
            step += 1
            if step % sc.log_every == 0 or step == 1:
                rd.log({"step": step, "lr": sched.get_last_lr()[0], "billed_s": round(billed(), 2),
                        "usd": round(usd(), 4), "grad_norm": float(gn), **logs})
            if step % sc.eval_every == 0:
                dev_eval()
                last_eval = step
            if step % sc.ckpt_every == 0:
                rd.save(state(), step, sc.keep_last)
        if reason != "nonfinite":
            rd.save(state(), step, sc.keep_last)  # 先存：被抢占时可能只剩几秒（codex 第 1 轮 #3）
            if reason == "max_steps" and step > last_eval:  # 只在正常跑完时补评估；上限 / 信号时不再花时间
                dev_eval()
    rec["stop"] = {"reason": reason, "step": step, "billed_s": round(billed(), 2), "usd": round(usd(), 4),
                   "at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    rd.write(rec)
    return rec["stop"]


def parse_set(items: list[str]) -> dict:
    out = {}
    for it in items:
        k, _, v = it.partition("=")
        out[k] = yaml.safe_load(v)
    return out


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="exp025 GTCRN 训练")
    ap.add_argument("--config", required=True)
    ap.add_argument("--set", action="append", default=[], help="覆盖配置，如 run.name=x（可多次）")
    a = ap.parse_args(argv)
    stop = train(cfgmod.load(a.config, parse_set(a.set)))
    print(stop)
    return 0 if stop["reason"] in ("max_steps", "max_hours", "max_usd") else 1


if __name__ == "__main__":
    sys.exit(main())
