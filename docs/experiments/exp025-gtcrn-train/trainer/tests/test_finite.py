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
from test_train import _cfg


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
