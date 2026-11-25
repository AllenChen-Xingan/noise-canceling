"""单批过拟合判据第 4 版（事后改判据，2026-11-25，大总管定，技术题，人未参与）：查「流程能学」，不查「走多远」。

过线（开发份第 0 批前 4 段，150 步，lr 1e-3）：
1. 训练损失相对下降 (初值 − 终值) / 初值 ≥ 15%；初值不是正的有限数 → 判据不可用
2. 这 4 段 SI-SNR 提高 > 0 dB
3. 主干四块（REQUIRED_BACKBONE）都在，且 base.* 各组第 1 步梯度范数都非零、有限
4. B 的注意力输出层权重动过（A 没有这一层，不查）
只记录、不过线：走完 oracle 距离的比例、SI-SNR +6 dB（第 3 版的两条）。
依据（大总管原话摘要）：2000 步曲线变平在约 30%，换 3 个学习率、关 5 组配方旋钮都改不了，说明对预训练过的
2.5 万参数因果小模型这两条门槛不适用，不是流程坏了。前三版的失败报告都留在 smoke_reports/。
「训练有没有用」另由 H100 乙段 300 步的开发份读数判（dev_gate.py）。
"""
from __future__ import annotations

import math

DROP_FRAC_MIN = 0.15
BACKBONE_PREFIX = "base."
REQUIRED_BACKBONE = ("base.encoder", "base.dpgrnn1", "base.dpgrnn2", "base.decoder")  # GTCRN 主干四块，缺一块就不过（codex #1）
RULE = ("事后改判据，2026-11-25（第 4 版，大总管定，人未参与）。开发份第 0 批前 4 段："
        "(初值 − 终值) / 初值 ≥ 15%；SI-SNR 提高 > 0 dB；主干各层第 1 步梯度非零、有限；B 的注意力输出层动过。"
        "初值不是正的有限数 → 判据不可用。走完 oracle 距离 ≥ 50%、SI-SNR +6 dB 降为只记录。"
        "前三版：输出等于目标（到不了）· 40 dB 下限（到不了）· oracle 下限 50%（2000 步约 30%，曲线变平）")


def _fin(x) -> bool:
    return isinstance(x, (int, float)) and not isinstance(x, bool) and math.isfinite(x)  # bool 不算读数（codex #4）


def drop_frac(first, last):
    """相对下降；初值不是正的有限数、或终值非有限时返回 None。"""
    if not (_fin(first) and _fin(last)) or first <= 0:
        return None
    return (first - last) / first


def grad_norms(model) -> dict:
    """各组（参数名前两段）的梯度 L2 范数；没有梯度的可训练参数按 0 计。"""
    acc: dict[str, float] = {}
    for n, p in model.named_parameters():
        if not p.requires_grad:
            continue
        k = ".".join(n.split(".")[:2])
        g = 0.0 if p.grad is None else float(p.grad.detach().double().pow(2).sum())
        acc[k] = acc.get(k, 0.0) + g
    return {k: v ** 0.5 for k, v in acc.items()}


def backbone_grads_ok(grads) -> bool:
    if not grads:
        return False
    bb = [v for k, v in grads.items() if k.startswith(BACKBONE_PREFIX)]
    return (all(k in grads for k in REQUIRED_BACKBONE) and bool(bb)
            and all(_fin(v) and v > 0 for v in bb))


def verdict(first, last, gain_db, grads, moved) -> str:
    """pass / fail / criterion_unusable（初值不是正的有限数时相对下降没有意义）。"""
    if not _fin(first) or first <= 0:
        return "criterion_unusable"
    d = drop_frac(first, last)
    ok = (d is not None and d >= DROP_FRAC_MIN and _fin(gain_db) and gain_db > 0
          and backbone_grads_ok(grads) and (moved is None or (_fin(moved) and moved > 0)))
    return "pass" if ok else "fail"


def rejudge(probe: dict, gate3: dict) -> dict:
    """按第 4 版重判 2026-11-25 那次（同一起点、同一批、150 步、lr 1e-3）：初值与终值取诊断报告学习曲线
    （终值是第 150 次更新之后重算的，与第 4 版 smoke 同口径，codex #5），SI-SNR 同；第 1 步梯度取诊断报告；
    注意力输出层变化诊断没记，取第 3 版报告（同一次设置）。"""
    ov = gate3["overfit"]
    c = probe["curve"]
    m150 = next(m for m in c["marks"] if m["step"] == ov["steps"])
    first, last, gain = c["loss0"], m150["loss"], m150["si_snr_gain_db"]
    grads = c["grad_norms_first_step"]
    v = verdict(first, last, gain, grads, ov["attn_out_max_change"])
    others = {k: bool(gate3[k]["pass"]) for k in ("init_loss", "reproduce", "known_dev")}
    return {"rule": RULE, "gate3_verdict": gate3["overfit"]["verdict"],
            "overfit": {"first": first, "last": last, "drop_frac": drop_frac(first, last),
                        "si_snr_gain_db": gain, "gate3_last_traj": ov["last"], "attn_out_max_change": ov["attn_out_max_change"],
                        "grad_norms_first_step": grads, "backbone_grads_ok": backbone_grads_ok(grads),
                        "drop_frac_of_oracle_span": ov["drop_frac_of_oracle_span"], "verdict": v},
            "others": others, "verdict": v if all(others.values()) else "fail",
            "sources": {"gate3": "smoke_reports/20261125-cpu-gate3-FAILED.json",
                        "probe": "smoke_reports/20261125-overfit-probe-cloud-cpu.json"}}


if __name__ == "__main__":
    import json
    import sys
    from pathlib import Path
    if len(sys.argv) != 4:
        sys.exit("用法：python gate_v4.py <诊断报告.json> <第 3 版报告.json> <输出.json>")
    out = Path(sys.argv[3])
    r = rejudge(json.loads(Path(sys.argv[1]).read_text()), json.loads(Path(sys.argv[2]).read_text()))
    with open(out, "x") as f:
        f.write(json.dumps(r, ensure_ascii=False, indent=2))
    print(json.dumps(r, ensure_ascii=False, indent=2))
