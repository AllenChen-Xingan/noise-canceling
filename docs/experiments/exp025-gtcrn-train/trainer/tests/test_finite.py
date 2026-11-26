"""不是有限数只有一个入口（codex guards 第 4 轮 · 改结构）。

起因：同一个坑连着两轮出现——「下一批想停 → 返回 None」的路把已经读到的 NaN 掩掉了（先开发份评估、再标定）。
所以不再逐处补判断，改成：`guards.is_finite` 是唯一的判断，`require_finite` 直接抛异常，
训练循环里只有一处接住它。本文件同时钉住停止原因的优先级表。
"""
import json

import numpy as np
import pytest
import torch

import guards as G
import train
from test_train import _cfg, _read


# ---------- is_finite：唯一的判断 ----------

def test_is_finite_ignores_non_readings():
    """bool / None / 字符串不是读数，当作没问题（原来的 _fin 也这样）。"""
    for x in (True, False, None, "nan", b"inf", {"a": None}, [True, None], (None, "inf")):
        assert G.is_finite(x), x


def test_is_finite_recurses_over_dict_list_tuple():
    assert G.is_finite({"a": 1.0, "b": [2, (3.0, {"c": 4})]}) and G.is_finite([]) and G.is_finite({})
    for bad in ({"a": float("nan")}, [1.0, float("inf")], ({"x": [float("-inf")]},),
                {"total": torch.tensor(float("nan"))}, np.array(float("inf")).item()):
        assert not G.is_finite(bad), bad


def test_is_finite_counts_zero_dim_tensor_and_numpy_scalar():
    assert G.is_finite(torch.tensor(1.5)) and G.is_finite(np.float32(2.0)) and G.is_finite(np.int64(3))
    assert not G.is_finite(torch.tensor(float("nan"))) and not G.is_finite(np.float64("inf"))


# ---------- NonFinite / require_finite ----------

def test_require_finite_passes_good_readings():
    assert G.require_finite("evaluate", {"hybrid": 1.0, "lvl_valid": 3, "note": None}) is None


def test_nonfinite_carries_where_and_json_detail():
    with pytest.raises(G.NonFinite) as ei:
        G.require_finite("train_loss", {"total": torch.tensor(float("nan")), "hybrid": 1.0})
    e = ei.value
    assert isinstance(e, Exception) and e.where == "train_loss"
    assert isinstance(e.detail, dict) and isinstance(e.detail["total"], float)
    assert json.loads(json.dumps(e.detail)).keys() == e.detail.keys()  # 可 JSON 化（张量已变 float）
    assert "train_loss" in str(e)


def test_nonfinite_detail_lists_every_bad_path():
    with pytest.raises(G.NonFinite) as ei:
        G.require_finite("evaluate", {"dev": {"hybrid": float("inf")}, "dev_ema": [{"total": float("nan")}]})
    assert set(ei.value.detail) == {"dev.hybrid", "dev_ema[0].total"}
    assert all(v != v or v in (float("inf"), float("-inf")) for v in ei.value.detail.values())


# ---------- guards 自己的行为不变 ----------

def test_guard_still_judges_readings_after_swap():
    g = G.Guard()
    assert g.on_train_step(1, float("nan"), 0.1) is None  # 不是有限数不参与损失猛涨的判断
    assert g.on_train_step(2, 1.0, float("inf")) is None  # 单步墙钟不是有限数：不记
    assert g.on_eval(1, float("nan"))["nonfinite"] is True
    assert g.on_eval(2, None)["nonfinite"] is True  # 缺读数按非有限处理，不崩
    assert g.on_eval(3, torch.tensor(3.9))["best"] is True  # 0 维张量按数处理


# ---------- train.py：坏读数一律抛，只有一处接住 ----------

def _nan_in_forward(monkeypatch):
    """标定 / 评估的前向一律读出 hybrid = NaN（按批号只坏第一批）。"""
    n = {"k": 0}

    def fl(model, crit, batch, device, **k):
        n["k"] += 1
        return torch.zeros(()), {"hybrid": float("nan") if n["k"] == 1 else 1.0}
    monkeypatch.setattr(train, "forward_loss", fl)
    return n


def test_calibrate_raises_and_stops_reading(tmp_path, monkeypatch):
    """标定读到坏读数立刻抛，不再返回 {"nonfinite": True}，也不再看下一批（第 4 轮 · 改结构）。"""
    n = _nan_in_forward(monkeypatch)
    model = torch.nn.Linear(1, 1)
    with pytest.raises(G.NonFinite) as ei:
        train.calibrate(model, _cfg(tmp_path), [0, 1], torch.device("cpu"))
    assert ei.value.where == "calibrate" and n["k"] == 1
    assert ei.value.detail["hybrid"] != ei.value.detail["hybrid"]  # NaN
    assert model.training  # finally 里的 set_train 照样跑过


def test_evaluate_raises_and_does_not_ask_again(tmp_path, monkeypatch):
    """一批读出坏读数立刻抛，不再问 should_stop（它会把 NaN 掩成「要停」）。"""
    n = _nan_in_forward(monkeypatch)
    asked = []

    def should_stop():
        asked.append(1)
        return "max_usd" if len(asked) == 2 else None
    model = torch.nn.Linear(1, 1)
    with pytest.raises(G.NonFinite) as ei:
        train.evaluate(model, None, [0, 1], torch.device("cpu"), _cfg(tmp_path), should_stop)
    assert ei.value.where == "evaluate" and n["k"] == 1 and len(asked) == 1
    assert model.training


def test_all_finite_helper_is_gone():
    """逐处判断的旧助手删掉了：判断只在 guards.is_finite（tests/test_finite.py 末尾的扫描钉住这一点）。"""
    assert not hasattr(train, "all_finite")


def test_nonfinite_loss_beats_a_stop_signal(tmp_path, monkeypatch):
    """损失坏了的同时收到信号：报 nonfinite 并留现场，不许被「想停」这条路掩掉。"""
    import data
    sig = train._Signals()
    real_batches = data.SyntheticSource.train_batches

    def gen(self, start_batch=0, device=None):
        for b in real_batches(self, start_batch, device):
            if b["batch_index"] == 0:
                sig.got = "SIGTERM"
            yield b
    monkeypatch.setattr(data.SyntheticSource, "train_batches", gen)

    real_fl = train.forward_loss

    def fl(model, crit, batch, device, **k):
        loss, logs = real_fl(model, crit, batch, device, **k)
        bad = model.training and batch.get("batch_index") == 0
        return (loss * float("nan"), logs) if bad else (loss, logs)
    monkeypatch.setattr(train, "forward_loss", fl)

    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 50, "schedule.eval_every": 100}), _signals=sig)
    rec = json.loads((tmp_path / "t" / "run.json").read_text())  # 一步都没算完，还没有 metrics.jsonl
    assert stop["reason"] == "nonfinite" and stop["step"] == 0  # 第 1 批就坏，一步都没算完
    assert rec["crash"]["where"] == "train_loss" and rec["stop"]["reason"] == "nonfinite"


def test_nonfinite_in_final_eval_after_max_steps(tmp_path, monkeypatch):
    """跑满后的补评估读到坏读数：照样按训崩停，坏读数不进权重。"""
    real = train.evaluate
    monkeypatch.setattr(train, "evaluate", lambda *a, **k: {**real(*a, **k), "hybrid": float("nan")})
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 3, "schedule.eval_every": 100}))
    root, rec, _ = _read(tmp_path)
    assert stop["reason"] == "nonfinite" and stop["step"] == 3
    assert rec["crash"]["where"] == "dev_eval" and set(rec["crash"]["detail"]) == {"dev.hybrid"}
    assert (root / "ckpt" / "step-0000003.pt").exists()  # 停下前先存过
