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
- 自动保护（guards.py，方案第十二节）：不是有限数另存现场 crash-<步>.pt · 损失猛涨停 · 开发份最好的一次另存 best.pt ·
  前 500 步太慢停 · 中途卡住 900 s 强退。停下原因写进 run.json，退出码非 0
"""
from __future__ import annotations

import argparse
import importlib.util
import json
import math
import sys
import time
from pathlib import Path

import torch
import yaml

import config as cfgmod
import guards as G
import losses
import models
from data import frames_mask, make_source
from official import load_start_state
from runrec import EMA, RunDir, check_resume, env_info, git_info, start_record
from watchdog import HEARTBEAT_S, _Beat, _bounded, _Deadline, _hard_exit, _Signals  # noqa: F401  测试经 train 引用

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
                          kd=lc.kd, w_kd=lc.w_kd, level_norm=lc.level_norm, level_floor=lc.level_floor,
                          on_skip_leading=lc.on_skip_leading)


@torch.no_grad()
def calibrate(model, cfg, batches, device, should_stop=lambda: None) -> dict | None:
    """027 3.2 权重第 1 步：起点模型在 calib_batches 个开发批上量各项大小，把 "auto" 的权重定成约占 HybridLoss 的 10%。
    每批前问一次 should_stop（上限 / 信号）；要停就返回 None（codex 第 2 轮 #1）。
    某一批的读数不是有限数时不等下一批，立刻返回 {"nonfinite": True, "batch": 批号, "logs": logs}（codex guards 第 4 轮 #1）。"""
    pc = loss_cfg(cfg, 1.0, 1.0)
    pc.sp, pc.kd = False, False  # 标定只量 HybridLoss 与两项保护
    probe = losses.TrainLoss(pc).to(device)
    model.eval()
    sums = {"hybrid": 0.0, "lvl": 0.0, "on": 0.0}
    valid = {"lvl": 0, "on": 0}
    for i, b in enumerate(batches):
        if should_stop():
            set_train(model, cfg)
            return None
        _, logs = forward_loss(model, probe, b, device)
        if not all_finite(logs):  # 这一批已经读到非有限数：不再问 should_stop，也不算什么权重
            set_train(model, cfg)
            return {"nonfinite": True, "batch": i, "logs": logs}
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
def evaluate(model, crit, dev, device, cfg, should_stop=lambda: None, **kw) -> dict | None:
    """开发份平均读数。lvl / on 按有效段数加权（整批无效的不进分母，codex 第 3 轮 #3）；kd 等按出现过的批数平均。
    每批前问一次 should_stop，要停就返回 None（codex 第 3 轮 #1）。
    某一批的读数不是有限数时不等下一批，立刻返回 {"nonfinite": True, "batch": 批号, "logs": logs}（codex guards 第 3 轮 #1）。"""
    model.eval()
    acc, cnt = {}, {}
    try:
        for i, b in enumerate(dev):
            if should_stop():
                return None
            _, logs = forward_loss(model, crit, b, device, **kw)
            if not all_finite(logs):  # 这一批已经读到非有限数：不再问 should_stop（它可能先要停，把 NaN 掩掉）
                return {"nonfinite": True, "batch": i, "logs": logs}
            for k, v in logs.items():
                if k.endswith("_valid"):
                    continue
                w = logs.get(f"{k}_valid", 1) if k in ("lvl", "on") else 1
                acc[k] = acc.get(k, 0.0) + v * w
                cnt[k] = cnt.get(k, 0) + w
            for k in ("lvl", "on"):
                if f"{k}_valid" in logs:
                    cnt[f"{k}_valid"] = cnt.get(f"{k}_valid", 0) + logs[f"{k}_valid"]
    finally:
        set_train(model, cfg)
    out = {k: (v / cnt[k] if cnt[k] else 0.0) for k, v in acc.items()}
    out |= {k: v for k, v in cnt.items() if k.endswith("_valid")}
    return out


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


def _setup_stop(reason, step, billed_s, usd) -> dict:
    """准备期就停：不新存检查点、不改 run.json，账只在心跳里；step 是已落盘的进度（新运行为 0）。"""
    return {"reason": reason or "setup_stopped", "step": step, "phase": "setup",
            "billed_s": round(billed_s, 2), "usd": round(usd, 4)}


def deadline_s(lim) -> float:
    """兜底时限：时长上限与「美元上限 ÷ 单价」折成的时长取较小者，再加 hang_s（codex 第 9 轮 #1 的非故障部分）。"""
    hours = lim.max_hours if lim.usd_per_hour <= 0 else min(lim.max_hours, lim.max_usd / lim.usd_per_hour)
    return hours * 3600 + lim.hang_s


def train(cfg, *, max_wall_s: float | None = None, _signals: _Signals | None = None,
          _beat_s: float = HEARTBEAT_S, _on_hang=None) -> dict:
    """跑到停下为止，返回停下记录。max_wall_s / _signals / _beat_s 只给测试用。

    计费（billed_s · usd）从本函数第一行算起；信号处理与心跳线程也从第一行起生效，覆盖建模型、取开发份、标定
    （codex 第 2 轮 #1 #2）。准备期就被杀、还没有检查点时，下次开跑从心跳接上账。续训时在检查点、心跳、上次停下
    记录三处取最大值。美元按段累计：每段只按本段单价乘本段时长，改单价不追溯（codex 第 1 轮 #6）。"""
    t_proc = time.monotonic()
    cfg_dict = cfgmod.to_dict(cfg)
    git_info()  # 拿不到提交号就在碰磁盘、花钱之前失败（大总管 2026-11-25）
    # 第一次碰磁盘之前先起「本段墙钟时限」看门狗：不读盘、不写盘，只看本进程已跑多久。建目录、加锁、读心跳
    # 卡在网络盘上时它照样出手（codex 第 8 轮 #1）。之后由 _Beat 按累计账判上限，这一道只兜底。
    deadline = _Deadline(deadline_s(cfg.limits), _on_hang)
    with deadline:
        rd = RunDir(Path(cfg.run.out_dir).expanduser() / cfg.run.name)
        rd.lock()  # 先占住目录再读任何状态
        deadline.rd = rd
        try:
            return _train_locked(cfg, rd, t_proc, cfg_dict, max_wall_s, _signals, _beat_s, _on_hang)
        finally:
            rd.unlock()


def _train_locked(cfg, rd, t_proc, cfg_dict, max_wall_s, _signals, _beat_s, _on_hang) -> dict:
    lim, sc = cfg.limits, cfg.schedule
    hb0 = rd.heartbeat()
    acct = {"billed0": hb0.get("billed_s", 0.0), "usd0": hb0.get("usd", 0.0), "step": hb0.get("step", 0)}

    def billed():
        return acct["billed0"] + time.monotonic() - t_proc

    def usd():
        return acct["usd0"] + (time.monotonic() - t_proc) / 3600 * lim.usd_per_hour

    acct["t_progress"] = time.monotonic()
    sigs = _signals or _Signals()

    def tick_stop():  # 准备期与评估里每批问一次：顺带报进展，免得被「卡住」看门狗误杀（guards ⑤）
        acct["t_progress"] = time.monotonic()
        return stop_reason()

    def stop_reason():
        if billed() / 3600 >= lim.max_hours:
            return "max_hours"
        if usd() >= lim.max_usd:
            return "max_usd"
        if sigs.got:
            return f"signal:{sigs.got}"
        return None

    beat = _Beat(rd, lambda: (acct["step"], billed(), usd()), _beat_s, want_stop=stop_reason,
                 last_progress=lambda: acct["t_progress"], hang_s=lim.hang_s, on_hang=_on_hang)
    with sigs, beat:
        device = resolve_device(cfg.device)
        source = make_source(cfg)
        dev = source.dev_batches(should_stop=tick_stop)
        if dev is None:
            return _setup_stop(stop_reason(), acct["step"], billed(), usd())
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
        guard = G.Guard(cfg.limits.expect_s_step)
        if st is not None and st.get("guards") is not None:
            guard.load_state_dict(st["guards"])
        if st is not None:  # 最好的一次取三处里最好的：检查点 · run.json · best.pt 自带的（与权重一起原子存，codex guards 第 1、2 轮）
            for b in (rd.read().get("best"), best_in_file(rd.ckpt_dir / "best.pt", device)):
                if b is not None and (guard.best is None or b["hybrid"] < guard.best["hybrid"]):
                    guard.best, guard.bad = dict(b), 0
        init_only = st is None and rd.read().get("phase") == "init" and not any(rd.ckpt_dir.glob("step-*.pt"))
        if init_only:
            # 上次写了 run.json、还没存下第 0 步就被杀（codex 第 3 轮 #2）：配置与数据一致就重建，账从心跳接上
            why = check_resume(rd.read(), cfg_dict, source.build_id)
            if why:
                raise RuntimeError("拒绝重建：" + "；".join(why))
        elif st is None and (rd.run_json.exists() or any(rd.ckpt_dir.glob("step-*.pt"))):
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
            prev = rec.get("stop") or {}
            # 三处取最晚的（本段准备期的秒数由 billed() / usd() 从 t_proc 起自动计入）
            acct["billed0"] = max(st["billed_s"], hb0.get("billed_s", 0.0), prev.get("billed_s", 0.0))
            acct["usd0"] = max(st.get("usd", 0.0), hb0.get("usd", 0.0), prev.get("usd", 0.0))
            rec["config"] = cfg_dict
            rec["resumes"].append({"at": time.strftime("%Y-%m-%dT%H:%M:%S%z"), "from_step": step,
                                   "from_ckpt": last.name, "billed_s_before": round(acct["billed0"], 2),
                                   "env": env_info(), "prev_stop": rec.pop("stop", None)})
        else:
            cal_dev = source.calib_batches(cfg.data.calib_batches, should_stop=tick_stop)
            calib = None if cal_dev is None else calibrate(model, cfg, cal_dev, device, tick_stop)
            if isinstance(calib, dict) and calib.get("nonfinite"):  # 标定读到非有限数：准备期就停，只留一份现场（第 4 轮 #2）
                crash = {"reason": "nonfinite", "phase": "calibrate", "batch": calib["batch"],
                         "logs": calib["logs"], "at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
                (rd.root / "setup_crash.json").write_text(json.dumps(crash, ensure_ascii=False))
                return {**_setup_stop("nonfinite", acct["step"], billed(), usd()), "crash": crash}
            if calib is None:  # 准备期就到上限 / 收到信号：没有检查点可存，只留心跳里的账
                return _setup_stop(stop_reason(), acct["step"], billed(), usd())
            step = 0
            pcnt = models.param_counts(model)
            rec = start_record(cfg_dict, source.build_id, source.num_workers,
                               {"device": str(device), "params": pcnt["trainable"], "params_fixed": pcnt["fixed"],
                                "calib": calib, "env": env_info(), "billed_s_before": round(acct["billed0"], 2),
                                "phase": "init"})
        if guard.best is not None:
            rec["best"] = guard.best  # 续训时把三处对齐后的最好成绩写回 run.json
        acct["step"] = step
        rd.write(rec)
        crit = losses.TrainLoss(loss_cfg(cfg, calib["lvl"]["weight"], calib["on"]["weight"])).to(device)
        kw = {"sp_ref": sp_ref, "teacher": teacher}

        def state():
            return {"model": model.state_dict(), "optim": opt.state_dict(), "sched": sched.state_dict(),
                    "ema": ema.state_dict() if ema else None, "sp_ref": sp_ref, "step": step,
                    "billed_s": billed(), "usd": usd(), "calib": calib, "config": cfg_dict,
                    "variant": cfg.model.variant, "guards": guard.state_dict()}

        def guard_event(kind, **detail):
            ev = {"kind": kind, "step": step, "at": time.strftime("%Y-%m-%dT%H:%M:%S%z"), **detail}
            rec.setdefault("guards", []).append(ev)
            rd.log({"step": step, "guard": ev})
            rd.write(rec)

        def crash(**info):
            """①：先把「训崩」写进 run.json（续训据此拒绝，不依赖现场存没存成），再有时限地尽力存现场：
            出事前的权重与优化器状态（还没被这一步更新），不进 step-* 轮换（codex guards 第 1 轮 #3）。"""
            rec["crash"] = {"step": step, **info, "snapshot": None}
            rec["stop"] = {"reason": "nonfinite", "step": step, "billed_s": round(billed(), 2), "usd": round(usd(), 4),
                           "at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
            rd.write(rec)
            name, err = f"crash-{step:07d}.pt", {}

            def save():
                try:
                    rd.save_as(state(), name)
                except Exception as e:  # noqa: BLE001 —— 现场存不成也要停
                    err["e"] = f"{type(e).__name__}: {e}"
                    raise
            if _bounded(save, CRASH_SAVE_S):
                rec["crash"]["snapshot"] = name
            else:
                rec["crash"]["snapshot_error"] = err.get("e", f"{CRASH_SAVE_S:.0f} s 内没存完")
            rd.write(rec)

        def dev_eval():
            """评估一次；读数不是有限数时返回 "nonfinite"（①），否则 None。"""
            row = {"step": step, "dev": evaluate(model, crit, dev, device, cfg, tick_stop, **kw)}
            # 原模型或 EMA 中途读到非有限数：evaluate 会直接交回来（codex guards 第 3 轮 #1）。
            # 这个判断必须排在「row["dev"] is None」之前：那种情况下一批的 should_stop 会把 NaN 掩成「要停」。
            got = row["dev"] if isinstance(row["dev"], dict) else {}
            if ema is not None and row["dev"] is not None and not got.get("nonfinite"):  # 034 建议 7：EMA 与不用 EMA 在开发份上并排比
                raw = {k: v.clone() for k, v in model.state_dict().items()}
                model.load_state_dict(ema.state_dict())
                try:
                    row["dev_ema"] = evaluate(model, crit, dev, device, cfg, tick_stop, **kw)
                finally:
                    model.load_state_dict(raw)
            ema_got = row.get("dev_ema") if isinstance(row.get("dev_ema"), dict) else {}
            if got.get("nonfinite") or ema_got.get("nonfinite"):  # 原模型已经是非有限就不再跑 EMA（省时间，也不存 best.pt）
                crash(dev=row)
                return "nonfinite"
            if row["dev"] is None:  # 评估中途要停：这一行不记（半份读数会误导比较）
                return None
            rd.log(row)
            g = guard.on_eval(step, row["dev"].get("hybrid"))
            if g["nonfinite"] or not all_finite(row):  # 原模型与 EMA 的全部读数（codex guards 第 1 轮 #1）
                crash(dev=row)
                return "nonfinite"
            if g["best"]:  # ③：最好的一次另存，不参与轮换；成绩单默认用它
                rd.save_as(state(), "best.pt")
                rec["best"] = guard.best
                rd.write(rec)
            if g["warn"]:
                guard_event("dev_plateau", message=g["warn"], best=guard.best)
            return None

        if st is None:
            rd.save(state(), 0, sc.keep_last)
            rec["phase"] = "training"  # 第 0 步已落盘，之后才算正式开训
            rd.write(rec)
        t_start = time.monotonic()
        reason, last_eval = None, step
        set_train(model, cfg)
        batches = source.train_batches(start_batch=step, device=device)
        while True:
            reason = stop_reason() or ("max_steps" if step >= sc.max_steps else None)  # 上限 / 信号优先（第 3 轮 #1）
            if reason is None and max_wall_s is not None and time.monotonic() - t_start >= max_wall_s:
                reason = "interrupted"
            if reason:
                break
            t_iter = time.monotonic()
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
                crash(logs=logs)
                break
            opt.zero_grad(set_to_none=True)
            loss.backward()
            gn = torch.nn.utils.clip_grad_norm_(model.parameters(), cfg.optim.grad_clip)
            if not torch.isfinite(gn):  # 损失有限、梯度却不是：不更新，否则 Adam 会把 NaN 写进权重（codex 第 1 轮 #2）
                reason = "nonfinite"
                crash(logs=logs, grad_norm=float(gn))
                break
            opt.step()
            sched.step()
            if ema is not None:
                ema.update(model)
            step += 1
            acct["step"] = step
            acct["t_progress"] = time.monotonic()
            if step % sc.log_every == 0 or step == 1:
                rd.log({"step": step, "lr": sched.get_last_lr()[0], "billed_s": round(billed(), 2),
                        "usd": round(usd(), 4), "grad_norm": float(gn), **logs})
            why = guard.on_train_step(step, logs.get("total", loss.item()), time.monotonic() - t_iter)
            if why:  # ② 损失猛涨 / ④ 太慢：只停本运行（每个运行各自判）
                reason = why
                guard_event(why, **(guard.spike_detail() if why == "loss_spike" else guard.slow_detail()))
                break
            if stop_reason():  # 本步算完时已到上限 / 收到信号：不评估，回到循环顶上存盘停下（codex 第 2 轮 #1）
                continue
            if step % sc.eval_every == 0:
                last_eval = step
                if dev_eval() == "nonfinite":
                    reason = "nonfinite"
                    break
            if step % sc.ckpt_every == 0:
                rd.save(state(), step, sc.keep_last)
        if reason != "nonfinite":
            rd.save(state(), step, sc.keep_last)  # 先存：被抢占时可能只剩几秒（codex 第 1 轮 #3）
            if reason == "max_steps" and step > last_eval:  # 只在正常跑完时补评估；上限 / 信号时不再花时间
                if dev_eval() == "nonfinite":
                    reason = "nonfinite"
                else:  # 补评估改了保护状态（最好一次、连续变差次数）：重存这一步，续训才接得上（codex guards 第 2 轮 #2）
                    rd.save(state(), step, sc.keep_last)
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
    return exit_code(stop)


CRASH_SAVE_S = 120.0  # 存现场最多等这么久；存不成也照样停（codex guards 第 1 轮 #3）


def best_in_file(path: Path, device) -> dict | None:
    """best.pt 里与权重一起存下的最好成绩；文件没有或读坏了返回 None。"""
    if not path.exists():
        return None
    try:
        g = torch.load(path, map_location=device, weights_only=False).get("guards") or {}
    except Exception:  # noqa: BLE001 —— 读坏了就不用它
        return None
    return g.get("best")


def all_finite(x) -> bool:
    """字典 / 列表里所有数都是有限数（bool 与 None 不算数）。"""
    if isinstance(x, dict):
        return all(all_finite(v) for v in x.values())
    if isinstance(x, (list, tuple)):
        return all(all_finite(v) for v in x)
    if isinstance(x, float):
        return math.isfinite(x)
    return True


def exit_code(stop: dict) -> int:
    """正常停（跑满 / 到时长或花费上限）0；其余（自动保护、取数超时、信号等）1，job-wait 据此报警。"""
    return 0 if stop["reason"] in ("max_steps", "max_hours", "max_usd") else 1


if __name__ == "__main__":
    sys.exit(main())
