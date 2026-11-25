"""训练时的自动保护（方案第十二节，大总管定 2026-11-25）：① 不是有限数存现场 · ② 损失猛涨 · ③ 最好的检查点与警告 ·
④ 前 500 步太慢（每个运行各自判）· ⑤ 中途卡住。阈值正本在 guards.py。"""
import json
import threading
import time

import pytest
import torch

import config as cfgmod
import guards as G
import losses
import train
from test_train import _cfg, _read


# ---------- ② 损失猛涨 ----------

def _feed(g, values, start=1):
    out = None
    for i, v in enumerate(values, start):
        out = g.on_train_step(i, v, 0.1) or out
    return out


def test_spike_threshold_is_3x_of_base():
    assert G.SPIKE_K == 3.0 and G.SPIKE_BASE_STEPS == 20 and G.SPIKE_WINDOW == 50
    assert _feed(G.Guard(), [1.0] * 20 + [3.1] * 50) == "loss_spike"
    assert _feed(G.Guard(), [1.0] * 20 + [2.9] * 50) is None


def test_spike_works_across_zero():
    """基准为负（含负对数项）：m − b > 2·|b| 才算，等于 3 倍的意思，跨零也成立。"""
    assert _feed(G.Guard(), [-1.0] * 20 + [1.1] * 50) == "loss_spike"
    assert _feed(G.Guard(), [-1.0] * 20 + [0.9] * 50) is None


def test_spike_uses_median_so_one_bad_batch_does_not_stop():
    vals = [1.0] * 20 + [1.0] * 49 + [1e6]
    assert _feed(G.Guard(), vals) is None


def test_spike_needs_full_window():
    assert _feed(G.Guard(), [1.0] * 20 + [100.0] * 49) is None


def test_nonfinite_train_value_is_not_judged_as_spike():
    g = G.Guard()
    assert g.on_train_step(1, float("nan"), 0.1) is None  # 非有限交给 ① 处理


# ---------- ④ 太慢 ----------

def test_too_slow_judged_once_at_step_500_over_steps_50_to_500():
    assert (G.SLOW_FACTOR, G.SPEED_FROM, G.SPEED_TO) == (1.2, 50, 500)
    for dt, want in ((0.46, None), (0.48, "too_slow")):
        g = G.Guard(expect_s_step=0.39)
        got = [g.on_train_step(k, 1.0, 5.0 if k < 50 else dt) for k in range(1, 501)]
        assert got[-1] == want and all(x is None for x in got[:-1]), dt  # 前 50 步慢不算；到第 500 步才判


def test_too_slow_off_without_expectation():
    g = G.Guard(expect_s_step=None)
    assert all(g.on_train_step(k, 1.0, 99.0) is None for k in range(1, 501))


# ---------- ③ 最好的检查点 · 警告 ----------

def test_best_and_patience_warning():
    assert (G.PATIENCE, G.BEST_TOL) == (3, 1e-3)
    g = G.Guard()
    r = [g.on_eval(s, h) for s, h in ((100, 3.90), (200, 3.80), (300, 3.8005), (400, 3.85), (500, 3.86), (600, 3.87))]
    assert [x["best"] for x in r] == [True, True, False, False, False, False]
    assert [x["warn"] is not None for x in r] == [False, False, False, False, False, True]  # 3.8005 不算更差
    assert g.best == {"step": 200, "hybrid": 3.80}
    assert g.on_eval(700, 3.79)["best"] and g.bad == 0


def test_eval_nonfinite_flagged():
    assert G.Guard().on_eval(100, float("nan"))["nonfinite"]


def test_state_roundtrip_keeps_base_window_best_and_speed():
    g = G.Guard(expect_s_step=0.39)
    for k in range(1, 80):
        g.on_train_step(k, 1.0, 0.4)
    g.on_eval(50, 3.9)
    h = G.Guard(expect_s_step=0.39)
    h.load_state_dict(json.loads(json.dumps(g.state_dict())))
    assert h.state_dict() == g.state_dict()


# ---------- ⑤ 中途卡住 ----------

def test_watchdog_fires_on_stall_even_without_stop_wanted(tmp_path, monkeypatch):
    assert G.STALL_S == 900.0
    rd = train.RunDir(tmp_path / "w")
    fired = []
    t0 = time.monotonic()
    b = train._Beat(rd, lambda: (0, 1.0, 0.0), 0.01, want_stop=lambda: None, last_progress=lambda: t0 - 950,
                    hang_s=5.0, on_hang=fired.append)
    with b:
        time.sleep(0.1)
    assert fired and "900" in fired[0]
    fired.clear()
    b = train._Beat(rd, lambda: (0, 1.0, 0.0), 0.01, want_stop=lambda: None, last_progress=lambda: t0 - 800,
                    hang_s=5.0, on_hang=fired.append)
    with b:
        time.sleep(0.1)
    assert not fired


# ---------- 接进训练循环 ----------

def _patch_train_loss(monkeypatch, change):
    """只改训练步（模型在训练模式）的损失；标定与评估（评估模式）不动。change(第几次训练步, loss, logs) → (loss, logs)。"""
    real = train.forward_loss
    n = {"k": 0}

    def wrapped(model, crit, batch, device, **k):
        loss, logs = real(model, crit, batch, device, **k)
        if model.training:
            n["k"] += 1
            loss, logs = change(n["k"], loss, logs)
        return loss, logs
    monkeypatch.setattr(train, "forward_loss", wrapped)


def _nan_at(monkeypatch, step_to_break):
    _patch_train_loss(monkeypatch, lambda k, l, g: (l * float("nan"), g) if k == step_to_break else (l, g))


def test_nonfinite_loss_saves_crash_snapshot(tmp_path, monkeypatch):
    _nan_at(monkeypatch, 4)
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "nonfinite"
    root, rec, _ = _read(tmp_path)
    crash = root / "ckpt" / f"crash-{stop['step']:07d}.pt"
    st = torch.load(crash, weights_only=False)
    assert st["step"] == stop["step"] and all(torch.isfinite(v).all() for v in st["model"].values()
                                              if v.dtype.is_floating_point)
    assert rec["crash"]["snapshot"] == crash.name
    with pytest.raises(RuntimeError, match="训崩"):
        train.train(_cfg(tmp_path))  # 现场不进轮换，续训照样拒绝


def test_nonfinite_dev_reading_stops(tmp_path, monkeypatch):
    real = train.evaluate
    n = {"k": 0}

    def ev(*a, **k):
        out = real(*a, **k)
        n["k"] += 1
        if n["k"] == 2:
            out = {**out, "hybrid": float("nan")}
        return out
    monkeypatch.setattr(train, "evaluate", ev)
    stop = train.train(_cfg(tmp_path, **{"schedule.eval_every": 2}))
    assert stop["reason"] == "nonfinite" and stop["step"] == 4


def test_best_ckpt_kept_outside_rotation(tmp_path, monkeypatch):
    vals = iter([3.0, 2.0, 2.5])
    real = train.evaluate
    monkeypatch.setattr(train, "evaluate", lambda *a, **k: {**real(*a, **k), "hybrid": next(vals)})
    train.train(_cfg(tmp_path, **{"schedule.eval_every": 2, "schedule.keep_last": 1}))
    root, rec, rows = _read(tmp_path)
    best = torch.load(root / "ckpt" / "best.pt", weights_only=False)
    assert best["step"] == 4 and rec["best"] == {"step": 4, "hybrid": 2.0}
    assert not (root / "ckpt" / "step-0000004.pt").exists()  # 轮换删了第 4 步，best.pt 还在


def test_loss_spike_stops_with_checkpoint(tmp_path, monkeypatch):
    monkeypatch.setattr(G, "SPIKE_BASE_STEPS", 2)
    monkeypatch.setattr(G, "SPIKE_WINDOW", 2)
    _patch_train_loss(monkeypatch, lambda k, l, g: (l * 0 + 1e3, {**g, "total": 1e3}) if k > 2 else (l, g))
    stop = train.train(_cfg(tmp_path, **{"schedule.eval_every": 100}))
    assert stop["reason"] == "loss_spike" and stop["step"] < 6
    root, rec, _ = _read(tmp_path)
    assert (root / "ckpt" / f"step-{stop['step']:07d}.pt").exists() and rec["guards"][-1]["kind"] == "loss_spike"


def test_too_slow_stops_each_run_on_its_own(tmp_path, monkeypatch):
    """两个运行同时跑：期望值定得很严的那个停（too_slow），另一个照常跑满。"""
    monkeypatch.setattr(G, "SPEED_FROM", 1)
    monkeypatch.setattr(G, "SPEED_TO", 4)
    res = {}

    def run(name, expect):
        res[name] = train.train(_cfg(tmp_path / name, **{"limits.expect_s_step": expect}))
    ts = [threading.Thread(target=run, args=("slow", 1e-9)), threading.Thread(target=run, args=("fast", 1e3))]
    for t in ts:
        t.start()
    for t in ts:
        t.join(120)
    assert res["slow"]["reason"] == "too_slow" and res["slow"]["step"] == 4
    assert res["fast"]["reason"] == "max_steps" and res["fast"]["step"] == 6
    root, rec, _ = _read(tmp_path / "slow")
    assert (root / "ckpt" / "step-0000004.pt").exists() and rec["guards"][-1]["kind"] == "too_slow"


def test_guard_state_survives_resume(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 3, "schedule.eval_every": 1}))
    st = torch.load(tmp_path / "t" / "ckpt" / "step-0000003.pt", weights_only=False)
    assert "guards" in st and st["guards"]["best"] is not None
    train.train(_cfg(tmp_path, **{"schedule.eval_every": 1}))
    _, rec, _ = _read(tmp_path)
    assert rec["best"]["step"] >= 1


def test_expect_s_step_config():
    assert cfgmod.from_dict({}).limits.expect_s_step is None
    assert cfgmod.from_dict({"limits": {"expect_s_step": 0.39}}).limits.expect_s_step == 0.39
    for bad in (0.0, -1.0, float("nan"), float("inf"), True):
        with pytest.raises(ValueError, match="expect_s_step"):
            cfgmod.from_dict({"limits": {"expect_s_step": bad}})


def test_expect_s_step_may_change_on_resume(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 3}))
    assert train.train(_cfg(tmp_path, **{"limits.expect_s_step": 0.39}))["step"] == 6


def test_exit_code_nonzero_for_guard_stops():
    for r in ("loss_spike", "too_slow", "nonfinite"):
        assert train.exit_code({"reason": r}) == 1
    for r in ("max_steps", "max_hours", "max_usd"):
        assert train.exit_code({"reason": r}) == 0


# ---------- codex guards 第 1 轮 ----------

@pytest.mark.parametrize("which", ["other_item", "ema"])
def test_any_nonfinite_dev_value_stops(tmp_path, monkeypatch, which):  # #1
    real = train.evaluate
    n = {"k": 0}

    def ev(model, *a, **k):
        out = real(model, *a, **k)
        n["k"] += 1
        bad = (which == "other_item" and n["k"] == 1) or (which == "ema" and n["k"] == 2)
        return {**out, "total": float("inf")} if bad else out
    monkeypatch.setattr(train, "evaluate", ev)
    over = {"schedule.eval_every": 2}
    if which == "ema":
        over["ema.enabled"] = True
    stop = train.train(_cfg(tmp_path, **over))
    assert stop["reason"] == "nonfinite" and stop["step"] == 2
    assert not (tmp_path / "t" / "ckpt" / "best.pt").exists()


def test_best_from_final_eval_survives_resume(tmp_path, monkeypatch):  # #2
    vals = iter([3.0, 2.0, 2.5, 2.6])  # 第 2 步 3.0；跑满第 3 步补评估 2.0；续训后 2.5、2.6 都不该覆盖 best
    real = train.evaluate
    monkeypatch.setattr(train, "evaluate", lambda *a, **k: {**real(*a, **k), "hybrid": next(vals)})
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 3, "schedule.eval_every": 2}))
    train.train(_cfg(tmp_path, **{"schedule.eval_every": 2}))
    root, rec, _ = _read(tmp_path)
    assert rec["best"] == {"step": 3, "hybrid": 2.0}
    assert torch.load(root / "ckpt" / "best.pt", weights_only=False)["step"] == 3


def test_crash_snapshot_failure_still_stops_and_refuses_resume(tmp_path, monkeypatch):  # #3
    import runrec
    real = runrec.RunDir.save_as

    def boom(self, state, name):
        if name.startswith("crash-"):
            raise OSError("disk full")
        return real(self, state, name)
    monkeypatch.setattr(runrec.RunDir, "save_as", boom)
    _nan_at(monkeypatch, 3)
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "nonfinite"
    _, rec, _ = _read(tmp_path)
    assert rec["crash"]["snapshot"] is None and "disk full" in rec["crash"]["snapshot_error"]
    with pytest.raises(RuntimeError, match="训崩"):
        train.train(_cfg(tmp_path))
