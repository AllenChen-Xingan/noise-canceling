"""开训前冒烟（014 第 3 步）：一个命令跑完三件事 + 测每步耗时，三件全过才进下一步。

    python smoke.py --config configs/smoke-cpu.yaml [--set …] [--out 报告.json]

1. **初始损失合理**（034 修正：微调时看「等于原检查点的损失」，不看随机初始化的理论值）：
   B 零初始化时与 A 在同一批上的损失相同（相对差 ≤ 1e-5），且低于随机初始化模型的损失
2. **单批能过拟合**（判据第 3 版，事后改判据：AI 提、大总管定，2026-11-25）：开发份第 0 批前 4 段反复训 overfit_steps 步，
   把「初值到这 4 段 oracle 损失（掩码族能达到的损失，见 oracle.py）」的距离走完 ≥ 一半，SI-SNR 提高 ≥ 6 dB；
   B 的注意力层要动过。oracle 不可用时判「判据不可用」（退出码 3），不判「不过」；整批 32 段的读数只作参考
3. **原检查点读数复现**：起点权重在官方样例 `mix.wav` 上的输出与官方 `enh.wav` 最大差 ≤ 2 个 16 位量化步（2/32768）；
   正式数据另用 `--known-dev` 比开发份已知读数；参照由 `--write-known-dev` 在 CPU 机上用同一份代码算
   （plan-h100-smoke.md 第三节，大总管定），文件带数据版本 meta，对不上就判不过
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
import gate_v4
import losses
import oracle
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


NEAR_PERFECT_SNR_DB = 40.0


def near_perfect_loss(crit, clean, fr, snr_db: float = NEAR_PERFECT_SNR_DB, seed: int = 0) -> float:
    """「近乎完美」输出（目标 + 低 snr_db 的白噪声）的损失，当过拟合判据的下限。

    不用「输出恰好等于目标」：那时 SI-SNR 项被官方的 1e-8 分母下限顶成约 −8，下限约 −10，任何训练都到不了
    （2026-11-24 本机冒烟实测：下限 −10.04，150 步走到 −0.92 只占 37%）。40 dB 是达得到的参照（AI 定）。"""
    g = torch.Generator(device="cpu").manual_seed(seed)
    noise = torch.randn(clean.shape, generator=g).to(clean.device)
    p_c = clean.pow(2).mean(dim=1, keepdim=True)
    noise = noise * torch.sqrt(p_c / 10 ** (snr_db / 10)) / noise.pow(2).mean(dim=1, keepdim=True).sqrt()
    return crit(losses.stft(clean + noise), clean, fr)[0].item()


def overfit_pass(first: float, last: float, floor: float) -> bool:
    """单批过拟合判据：把「初值到近乎完美输出（40 dB）损失」的距离走完至少一半，即 (first − last) / (first − floor) ≥ 0.5。
    不按 |初值| 归一：SI-SNR 项是负对数，总损失会跨零（自审 P2）。"""
    span = first - floor
    return bool(np.isfinite(last) and span > 0 and (first - last) / span >= 0.5)


SI_SNR_GAIN_DB = 6.0  # 过拟合后这一批 SI-SNR 至少提高这么多（大总管 2026-11-24 要的不依赖下限的读数；阈值 AI 定，云上再核）


@torch.no_grad()
def batch_si_snr(m, batch, device) -> float:
    """这一批的平均 SI-SNR（dB）：模型输出过 istft 后对干净目标。"""
    noisy, clean = batch["noisy"].to(device), batch["clean"].to(device)
    out = losses.istft(m(losses.stft(noisy), train.cond_for(m, noisy.shape[0], device)), clean.shape[1])
    return float(losses.si_snr_db(out, clean).mean())


OVERFIT_N = 4  # 过线只用开发份第 0 批的前 4 段（判据第 3 版，大总管 2026-11-25）
EXIT_CRITERION_UNUSABLE = 3  # 判据本身不可用：区别于「不过」的退出码 1
OVERFIT_RULE = ("事后改判据：AI 提、大总管定，2026-11-25（第 3 版）。只看开发份第 0 批前 4 段："
                "(初值 − 终值) / (初值 − 这 4 段的 oracle 损失) ≥ 0.5；这 4 段 SI-SNR 提高 ≥ 6 dB；"
                "B 的注意力输出层权重要动过。oracle 不是有限数或不比初值低 → 判据不可用。"
                "原 32 段读数只作参考。前两版：40 dB 下限（真数据上低于 oracle，到不了）、输出等于目标（−10.04，到不了）")


def _slice(batch: dict, a: int, b: int) -> dict:
    out = dict(batch)
    for k in ("noisy", "clean", "replay", "near_silent", "owner_ivs", "interf_ivs"):
        out[k] = batch[k][a:b]
    return out


def head(batch: dict, n: int) -> dict:
    """批里前 n 段（每段的字段一起切）。不够 n 段就报错。"""
    B = batch["noisy"].shape[0]
    if n > B:
        raise ValueError(f"批里只有 {B} 段，取不了前 {n} 段")
    return _slice(batch, 0, n)


@torch.no_grad()
def per_segment(m, crit, batch, device) -> list[dict]:
    """逐段的损失与 SI-SNR（只记录；批归一化冻结，单段与整批同口径）。"""
    out = []
    for i in range(int(batch["noisy"].shape[0])):
        s = _slice(batch, i, i + 1)
        out.append({"index": i, "loss": train.forward_loss(m, crit, s, device)[0].item(),
                    "si_snr_db": batch_si_snr(m, s, device)})
    return out


def overfit_verdict(first: float, last: float, oracle_loss: float, gain_db: float, moved) -> str:
    """pass / fail / criterion_unusable。oracle 不可用（非有限、或不比初值低）时判据本身作废，不判「不过」。"""
    if not (np.isfinite(first) and np.isfinite(oracle_loss)) or first - oracle_loss <= 0:
        return "criterion_unusable"
    ok = (overfit_pass(first, last, oracle_loss) and np.isfinite(gain_db) and gain_db >= SI_SNR_GAIN_DB
          and (moved is None or moved > 0))
    return "pass" if ok else "fail"


def _overfit_run(cfg, batch, device, steps: int, lr: float) -> dict:
    """同一批反复训，只出读数不判：初值 · 终值 · 40 dB 下限 · oracle · SI-SNR 前后 · 注意力输出层变化。"""
    m = _model(cfg, device)
    train.set_train(m, cfg)
    crit = losses.TrainLoss(losses.LossCfg()).to(device)
    with torch.no_grad():
        clean = batch["clean"].to(device)
        fr = train.frames_mask(batch, clean.shape[1]).to(device)
        floor40 = near_perfect_loss(crit, clean, fr)
    o = oracle.oracle_loss(crit, batch, device)
    attn0 = m.attn.out.weight.detach().clone() if getattr(m, "attn", None) is not None else None
    sisnr0 = batch_si_snr(m, batch, device)
    seg0 = per_segment(m, crit, batch, device)
    opt = torch.optim.Adam([p for p in m.parameters() if p.requires_grad], lr=lr)
    traj, grads1 = [], None
    for _ in range(steps):
        loss, logs = train.forward_loss(m, crit, batch, device)
        opt.zero_grad(set_to_none=True)
        loss.backward()
        if grads1 is None:
            grads1 = gate_v4.grad_norms(m)
        opt.step()
        traj.append(logs["total"])
    moved = None if attn0 is None else float((m.attn.out.weight.detach() - attn0).abs().max())
    sisnr1 = batch_si_snr(m, batch, device)
    seg1 = per_segment(m, crit, batch, device)
    with torch.no_grad():  # 终值在最后一次更新之后重算，与 SI-SNR、逐段同一权重状态（codex gate-v4 #5）
        last_eval = train.forward_loss(m, crit, batch, device)[0].item()
    segs = [{"index": a["index"], "loss_before": a["loss"], "loss_after": b["loss"], "si_snr_db_before": a["si_snr_db"],
             "si_snr_db_after": b["si_snr_db"], "si_snr_gain_db": b["si_snr_db"] - a["si_snr_db"]}
            for a, b in zip(seg0, seg1)]
    first, last = traj[0], last_eval
    return {"last_traj": traj[-1], "n_segments": int(batch["noisy"].shape[0]), "first": first, "last": last, "steps": steps, "lr": lr,
            "floor_40db": floor40, "drop_frac_of_40db_span": oracle.frac_of_span(first, last, floor40),
            "floor_40db_reachable": oracle.floor_reachable(floor40, o["loss"]),
            "oracle_loss": o["loss"], "oracle_si_snr_db": o["si_snr_db"],
            "drop_frac_of_oracle_span": oracle.frac_of_span(first, last, o["loss"]),
            "attn_out_max_change": moved, "si_snr_db_before": sisnr0, "si_snr_db_after": sisnr1,
            "si_snr_gain_db": sisnr1 - sisnr0, "drop_frac": gate_v4.drop_frac(first, last),
            "grad_norms_first_step": grads1, "per_segment": segs}


def check_overfit(cfg, batch, device, steps: int, lr: float, reference: bool = True) -> dict:
    """单批过拟合（判据第 4 版，见 gate_v4.RULE；第 3 版的判法记在 verdict_v3，只记录）。
    B 另要求注意力输出层的权重动过（零初始化，只靠基座也能降，自审 P2）。
    条件口子本次喂全零（D98），W·0 对 W 的梯度恒为 0，它不动是预期，不作要求。
    reference=True 时另在整批上跑同样步数，读数记进 reference，不影响过线。"""
    B = int(batch["noisy"].shape[0])
    short = B < OVERFIT_N
    g = _overfit_run(cfg, head(batch, min(OVERFIT_N, B)), device, steps, lr)
    verdict = "criterion_unusable" if short else gate_v4.verdict(
        g["first"], g["last"], g["si_snr_gain_db"], g["grad_norms_first_step"], g["attn_out_max_change"])
    v3 = "criterion_unusable" if short else overfit_verdict(
        g["first"], g["last"], g["oracle_loss"], g["si_snr_gain_db"], g["attn_out_max_change"])
    out = {**g, "verdict": verdict, "verdict_v3": v3,
           "backbone_grads_ok": gate_v4.backbone_grads_ok(g["grad_norms_first_step"]), "pass": verdict == "pass", "floor": g["oracle_loss"], "floor_kind": "oracle",
           "floor_reachable": oracle.floor_reachable(g["oracle_loss"], g["oracle_loss"]),
           "drop_frac_of_span": g["drop_frac_of_oracle_span"],
           "si_snr_gain_ok": bool(np.isfinite(g["si_snr_gain_db"]) and g["si_snr_gain_db"] >= SI_SNR_GAIN_DB),
           "rule": gate_v4.RULE, "rule_v3": OVERFIT_RULE}
    if short:
        out["unusable_reason"] = f"批里只有 {B} 段，不够 {OVERFIT_N} 段"
    if reference and B > OVERFIT_N:
        out["reference"] = _overfit_run(cfg, batch, device, steps, lr)
    return out


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


# 参照读数文件里必须和本次一致的数据字段：开发份换了任何一项，读数就不可比
KNOWN_META_KEYS = ("kind", "build_id", "dev_n", "dev_seed", "seg_s", "batch", "dev_batches")


def known_meta(cfg, build_id, device) -> dict:
    d = cfg.data
    return {"kind": d.kind, "build_id": build_id, "dev_n": d.dev_n, "dev_seed": d.dev_seed, "seg_s": d.seg_s,
            "batch": d.batch, "dev_batches": d.dev_batches, "device": str(device), "torch": torch.__version__}


def dev_reading(cfg, dev, device) -> dict:
    """起点模型（A，官方权重）在开发份上的读数；--write-known-dev 与 --known-dev 共用这一个函数。"""
    m = _model(cfg, device, variant="A").eval()
    return train.evaluate(m, losses.TrainLoss(losses.LossCfg()).to(device), dev, device, cfg)


def check_known_dev(cfg, dev, device, known: dict, tol: float, build_id=None) -> dict:
    """开发份已知读数（键 = 损失日志名，如 hybrid）与起点模型实测比对。
    新格式 {"readings": {…}, "meta": {…}}：meta 的 KNOWN_META_KEYS 要和本次逐项相同。
    旧的扁平格式 {"hybrid": x} 没法核数据版本，只在合成数据上认；正式数据（loader）用它判不过。"""
    readings, meta = (known.get("readings") or {}, known.get("meta")) if "readings" in known else (known, None)
    want = known_meta(cfg, build_id, device)
    meta_mismatch = {} if meta is None else {k: (meta.get(k), want[k]) for k in KNOWN_META_KEYS
                                             if meta.get(k) != want[k]}
    if meta is not None and meta.get("device") != "cpu":  # 参照必须是 CPU 算的，否则 GPU 自比也能过（codex 第 3 轮 #2）
        meta_mismatch["device"] = (meta.get("device"), "cpu")
    meta_missing = meta is None and cfg.data.kind == "loader"
    got = dev_reading(cfg, dev, device)
    bad = {}
    for k, v in readings.items():
        g = got.get(k)
        # 实测、已知、容差都必须是有限数；NaN 时 abs(NaN − v) > tol 为假，会误判通过（codex 第 4 轮 #2）
        ok = (g is not None and all(np.isfinite(x) for x in (g, v, tol)) and abs(g - v) <= tol)
        if not ok:
            bad[k] = (g, v)
    ok = bool(readings) and not bad and not meta_mismatch and not meta_missing
    return {"pass": ok, "got": got, "known": known, "tol": tol, "mismatch": bad,
            "meta_mismatch": meta_mismatch, "meta_missing": meta_missing}


def timing(cfg, source, device, steps: int) -> dict:
    m = _model(cfg, device)
    train.set_train(m, cfg)  # 与正式训练同一模式
    crit = losses.TrainLoss(losses.LossCfg(w_lvl=1.0, w_on=1.0)).to(device)
    opt = torch.optim.Adam([p for p in m.parameters() if p.requires_grad], lr=1e-5)
    it = source.train_batches(0, device=device)  # 显卡上走 GPU 混音，与正式训练同一条路（codex 第 1 轮 #7）
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
        known_dev: dict | None, known_tol: float, overfit_ref: bool = True) -> dict:
    device = train.resolve_device(cfg.device)
    source = make_source(cfg)
    dev = source.dev_batches()
    batch = dev[0]
    rep = {"device": str(device), "power": power_state(), "torch": torch.__version__,
           "build_id": source.build_id, "config": cfgmod.to_dict(cfg),
           "started_at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    rep["init_loss"] = check_init(cfg, batch, device)
    rep["overfit"] = check_overfit(cfg, batch, device, overfit_steps, overfit_lr, reference=overfit_ref)
    rep["reproduce"] = check_reproduce(device)
    if known_dev:
        rep["known_dev"] = check_known_dev(cfg, dev, device, known_dev, known_tol, build_id=source.build_id)
    elif cfg.data.kind == "loader":
        # 014 第 3 步要「原检查点在开发份上读数等于已知成绩」：正式数据没给基准就不算通过（codex 第 4 轮 #3）
        rep["known_dev"] = {"pass": False, "incomplete": True,
                            "rule": "正式数据冒烟必须用 --known-dev 给开发份已知读数（等 evalcmp / cloudrun 出数）"}
    rep["timing"] = timing(cfg, source, device, timing_steps)
    checks = [k for k in ("init_loss", "overfit", "reproduce", "known_dev") if k in rep]
    rep["pass"] = all(rep[k]["pass"] for k in checks)
    others_ok = all(rep[k]["pass"] for k in checks if k != "overfit")
    rep["verdict"] = ("pass" if rep["pass"] else
                      "criterion_unusable" if others_ok and rep["overfit"]["verdict"] == "criterion_unusable" else "fail")
    return rep


def write_known_dev(cfg, out: Path) -> int:
    """CPU 参照（plan-h100-smoke.md 第三节）：读数非有限、或文件已存在，都不写、退出码非 0。"""
    if out.exists():
        print(f"拒绝覆盖已有文件：{out}", file=sys.stderr)
        return 2
    out.parent.mkdir(parents=True, exist_ok=True)  # 先建目录，免得算完写不出（codex 第 1 轮 #2）
    device = train.resolve_device(cfg.device)
    if device.type != "cpu":
        print(f"参照读数必须在 CPU 上算（加 --set device=cpu），现在是 {device}", file=sys.stderr)
        return 2
    source = make_source(cfg)
    got = dev_reading(cfg, source.dev_batches(), device)
    h = None if got is None else got.get("hybrid")
    if h is None or not np.isfinite(h):
        print(f"开发份读数不是有限数，不写：{got}", file=sys.stderr)
        return 1
    rec = {"readings": {"hybrid": float(h)}, "meta": known_meta(cfg, source.build_id, device),
           "written_at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    with open(out, "x") as f:  # 独占新建：与上面的存在检查之间被别人抢先写了也不覆盖
        f.write(json.dumps(rec, ensure_ascii=False, indent=2))
    print(json.dumps(rec, ensure_ascii=False, indent=2))
    return 0


def overfit_diag(cfg, out: Path, steps: int, lr: float) -> int:
    """过拟合排查（大总管 2026-11-25）：与冒烟同一批（开发份第 0 批）、同一个过拟合，另算 oracle 损失。
    报告给三个比例：走完「40 dB 下限」距离 · 走完「oracle」距离；并判 40 dB 下限是否可达。只记录，不判过不过。"""
    if out.exists():
        print(f"拒绝覆盖已有文件：{out}", file=sys.stderr)
        return 2
    out.parent.mkdir(parents=True, exist_ok=True)
    device = train.resolve_device(cfg.device)
    source = make_source(cfg)
    batch = source.dev_batches()[0]
    ov = _overfit_run(cfg, batch, device, steps, lr)  # 整批、40 dB 下限口径（与 2026-11-25 云上排查同一读法）
    if not (np.isfinite(ov["oracle_loss"]) and np.isfinite(ov["last"])):
        print(f"读数不是有限数，不写：oracle {ov['oracle_loss']}，last {ov['last']}", file=sys.stderr)
        return 1
    rep = {**ov, "floor": ov["floor_40db"], "drop_frac_of_span": ov["drop_frac_of_40db_span"],
           "floor_reachable": ov["floor_40db_reachable"],
           "batch_index": 0, "batch": cfg.data.batch, "seg_s": cfg.data.seg_s, "build_id": source.build_id,
           "device": str(device), "torch": torch.__version__, "config": cfgmod.to_dict(cfg),
           "note": "排查用，不判过不过；oracle 是掩码族里的一个可行点（见 oracle.py），真正的最好损失 ≤ 它",
           "written_at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
    text = json.dumps(rep, ensure_ascii=False, indent=2, default=str)
    with open(out, "x") as f:
        f.write(text)
    print(text)
    return 0


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
    ap.add_argument("--write-known-dev", help="只算起点模型的开发份读数写到这里（不覆盖已有文件），不跑三件事")
    ap.add_argument("--skip-overfit-ref", action="store_true", help="不跑整批 32 段的过拟合参考（只记录用，不影响过线）")
    ap.add_argument("--overfit-probe", help="过拟合慢的诊断（学习曲线 · 参数与梯度 · 学习率）写到这里，见 overfit_probe.py")
    ap.add_argument("--probe-steps", type=int, default=2000, help="--overfit-probe 的学习曲线步数")
    ap.add_argument("--probe-sweep-steps", type=int, default=300, help="--overfit-probe 每个对照学习率的步数")
    ap.add_argument("--overfit-diag", help="只做过拟合排查（同一批：过拟合 + oracle 对照）写到这里，不判过不过")
    a = ap.parse_args(argv)
    cfg = cfgmod.load(a.config, train.parse_set(a.set))
    if a.write_known_dev:
        return write_known_dev(cfg, Path(a.write_known_dev))
    if a.overfit_probe:
        import overfit_probe  # 只在诊断时导入（它反过来用本模块的函数）
        return overfit_probe.run_probe(cfg, Path(a.overfit_probe), a.probe_steps, a.probe_sweep_steps)
    if a.overfit_diag:
        return overfit_diag(cfg, Path(a.overfit_diag), a.overfit_steps, a.overfit_lr)
    known = json.loads(Path(a.known_dev).read_text()) if a.known_dev else None
    rep = run(cfg, overfit_steps=a.overfit_steps, overfit_lr=a.overfit_lr, timing_steps=a.timing_steps,
              known_dev=known, known_tol=a.known_tol, overfit_ref=not a.skip_overfit_ref)
    text = json.dumps(rep, ensure_ascii=False, indent=2, default=str)
    if a.out:
        Path(a.out).write_text(text)
    print(text)
    return {"pass": 0, "criterion_unusable": EXIT_CRITERION_UNUSABLE}.get(rep["verdict"], 1)


if __name__ == "__main__":
    sys.exit(main())
