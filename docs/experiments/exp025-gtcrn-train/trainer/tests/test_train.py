"""配置正本与实例一致 · 训练循环：运行记录 · 检查点 · 续训逐值等同 · 时长 / 花费上限 · 训崩 · 拒绝续训。

全部用合成小样、CPU、每次几步（秒级）。
"""
import dataclasses
import json
from pathlib import Path

import pytest
import torch
import yaml

import config as cfgmod
import losses
import train

HERE = Path(__file__).resolve().parents[1]


def _flat(d, p=""):
    out = set()
    for k, v in d.items():
        if isinstance(v, dict):
            out |= _flat(v, p + k + ".")
        else:
            out.add(p + k)
    return out


@pytest.mark.parametrize("name", ["default.yaml", "smoke-cpu.yaml"])
def test_yaml_loads(name):
    cfgmod.load(HERE / "configs" / name)


def test_default_yaml_covers_every_field_and_matches_defaults():
    """default.yaml 恰好覆盖 dataclass 全部字段，值与 dataclass 默认值相同（正本与实例交叉校验）。"""
    y = yaml.safe_load((HERE / "configs" / "default.yaml").read_text())
    d = cfgmod.to_dict(cfgmod.Config())
    assert _flat(y) == _flat(d)
    assert cfgmod.to_dict(cfgmod.from_dict(y)) == d


def test_unknown_key_and_bad_values_rejected():
    with pytest.raises(ValueError, match="未知键"):
        cfgmod.from_dict({"optim": {"learning_rate": 1}})
    with pytest.raises(ValueError):
        cfgmod.from_dict({"model": {"variant": "C"}})
    with pytest.raises(ValueError):
        cfgmod.from_dict({"loss": {"w_lvl": "big"}})
    with pytest.raises(ValueError):
        cfgmod.from_dict({"limits": {"max_usd": 0}})
    for bad in ({"max_usd": float("inf")}, {"max_hours": float("nan")}, {"usd_per_hour": -1.0},
                {"usd_per_hour": float("nan")}):
        with pytest.raises(ValueError, match="limits"):
            cfgmod.from_dict({"limits": bad})


def _cfg(tmp_path, **over):
    d = {"run": {"out_dir": str(tmp_path), "name": "t", "seed": 3},
         "model": {"variant": "B"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 4,
                  "dev_batches": 2, "calib_batches": 2},
         "optim": {"lr": 1e-3, "warmup_steps": 2},
         "schedule": {"max_steps": 6, "ckpt_every": 2, "eval_every": 3, "log_every": 1, "keep_last": 2},
         "limits": {"max_hours": 1.0, "max_usd": 5.0},
         "device": "cpu"}
    for k, v in over.items():
        a, b = k.split(".")
        d.setdefault(a, {})[b] = v
    return cfgmod.from_dict(d)


def _read(tmp_path, name="t"):
    root = tmp_path / name
    rec = json.loads((root / "run.json").read_text())
    rows = [json.loads(x) for x in (root / "metrics.jsonl").read_text().splitlines()]
    return root, rec, rows


def test_run_record_and_checkpoints(tmp_path):
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "max_steps" and stop["step"] == 6
    root, rec, rows = _read(tmp_path)
    assert rec["git"]["commit"] and rec["seed"] == 3 and rec["data_build_id"] == "synthetic-seed3"
    assert rec["config"]["model"]["variant"] == "B"
    assert rec["calib"]["lvl"]["auto"] and rec["calib"]["lvl"]["weight"] > 0
    assert sum("dev" in r for r in rows) == 2  # 第 3、6 步
    assert all(torch.isfinite(torch.tensor(r["total"])) for r in rows if "total" in r)
    names = sorted(p.name for p in (root / "ckpt").glob("step-*.pt"))
    assert names == ["step-0000000.pt", "step-0000004.pt", "step-0000006.pt"]  # 起点 + 最近 2 份
    st = torch.load(root / "ckpt" / "step-0000006.pt", weights_only=False)
    assert st["step"] == 6 and {"model", "optim", "sched", "billed_s", "calib"} <= set(st)


def test_resume_equals_uninterrupted(tmp_path):
    """先训 3 步停下、再续到 6 步，与一口气训 6 步的权重逐值相同。"""
    train.train(_cfg(tmp_path / "a"))
    train.train(_cfg(tmp_path / "b", **{"schedule.max_steps": 3}))
    stop = train.train(_cfg(tmp_path / "b"))
    assert stop["step"] == 6
    wa = torch.load(tmp_path / "a/t/ckpt/step-0000006.pt", weights_only=False)["model"]
    wb = torch.load(tmp_path / "b/t/ckpt/step-0000006.pt", weights_only=False)["model"]
    for k in wa:
        assert torch.equal(wa[k], wb[k]), k
    _, rec, _ = _read(tmp_path / "b")
    assert len(rec["resumes"]) == 1 and rec["resumes"][0]["from_step"] == 3


def test_resume_refused_when_batch_changes(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 2}))
    with pytest.raises(RuntimeError, match="data.batch"):
        train.train(_cfg(tmp_path, **{"data.batch": 3}))


@pytest.mark.parametrize("over,reason", [({"limits.max_hours": 1e-9}, "max_hours"),
                                         ({"limits.max_usd": 1e-9, "limits.usd_per_hour": 1.0}, "max_usd")])
def test_limits_hit_during_setup_stop_before_training(tmp_path, over, reason):
    """上限小到准备期就用完：不开训、不建检查点，账留在心跳里。"""
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 1000, **over}))
    assert stop["reason"] == reason and stop["phase"] == "setup" and stop["step"] == 0
    assert (tmp_path / "t" / "heartbeat.json").exists()


def test_limit_hit_after_resume_saves_checkpoint(tmp_path):
    """续训时上限已低于已花的钱：准备期就停（不再花钱），报告已落盘的步数。"""
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 2, "limits.usd_per_hour": 100.0}))
    spent = json.loads((tmp_path / "t" / "heartbeat.json").read_text())["usd"]
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 1000, "limits.usd_per_hour": 100.0,
                                         "limits.max_usd": spent * 0.99}))
    assert stop["reason"] == "max_usd" and stop["step"] == 2 and stop["phase"] == "setup"
    assert (tmp_path / "t" / "ckpt" / "last.txt").exists()  # 第一段的检查点还在，下次提高上限即可续


def test_nonfinite_stops_without_saving_bad_weights(tmp_path, monkeypatch):
    real = losses.TrainLoss.forward
    calls = {"n": 0}

    def boom(self, *a, **k):
        tot, logs = real(self, *a, **k)
        calls["n"] += 1
        return (tot * float("nan"), logs) if self.training and calls["n"] > 4 else (tot, logs)

    monkeypatch.setattr(losses.TrainLoss, "forward", boom)
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 50, "schedule.eval_every": 1000}))
    assert stop["reason"] == "nonfinite"
    _, rec, _ = _read(tmp_path)
    assert "crash" in rec
    st = torch.load(tmp_path / "t" / "ckpt" / (tmp_path / "t/ckpt/last.txt").read_text(), weights_only=False)
    assert all(torch.isfinite(v).all() for v in st["model"].values() if v.dtype.is_floating_point)


def test_switches_kd_sp_ema_run(tmp_path):
    stop = train.train(_cfg(tmp_path, **{"loss.kd": True, "loss.sp": True, "ema.enabled": True,
                                         "schedule.max_steps": 3}))
    assert stop["reason"] == "max_steps"
    _, _, rows = _read(tmp_path)
    assert any("sp" in r for r in rows)
    assert any("dev_ema" in r for r in rows)


def test_variant_a_and_fixed_weights(tmp_path):
    stop = train.train(_cfg(tmp_path, **{"model.variant": "A", "loss.w_lvl": 0.0, "loss.w_on": 0.5,
                                         "schedule.max_steps": 2}))
    _, rec, _ = _read(tmp_path)
    assert stop["step"] == 2 and rec["calib"]["lvl"]["weight"] == 0.0 and rec["calib"]["on"]["weight"] == 0.5
    assert rec["params"] == 23669 and rec["params_fixed"] == 24576


def test_dataclass_fields_documented_in_default_yaml():
    """每个子配置都在 default.yaml 里出现（防新加字段忘了写实例）。"""
    y = yaml.safe_load((HERE / "configs" / "default.yaml").read_text())
    for f in dataclasses.fields(cfgmod.Config):
        assert f.name in y


_CLOUD_CFG = HERE.parent / "cloud" / "config.py"


@pytest.mark.skipif(not _CLOUD_CFG.exists(), reason="cloudrun 的 cloud/config.py 还没合入本分支；合入后自动跑")
def test_usd_per_hour_matches_cloud_config():
    """单价的正本在 cloud/config.py（CLAUDE.md 五.5）：trainer 的默认值必须与它一致。"""
    import importlib.util
    spec = importlib.util.spec_from_file_location("exp025_cloud_config", _CLOUD_CFG)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    assert cfgmod.Config().limits.usd_per_hour == mod.USD_PER_HR_EXPECTED
