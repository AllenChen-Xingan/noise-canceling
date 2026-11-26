"""H100 冒烟用的两件小工具（plan-h100-smoke.md 第三节）：
`smoke.py --write-known-dev`（开发份 CPU 参照读数）与 `cmp_ckpt.py`（续训核对）。
重点防「误判通过」：数据版本对不上、读数非有限、键缺失、NaN、形状或步数不同，一律判不过。"""
import json
import math

import pytest
import torch
import yaml

import cmp_ckpt
import config as cfgmod
import smoke
import train


def _cfg_dict(tmp_path, **data):
    return {"run": {"out_dir": str(tmp_path), "name": "s"},
            "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 2, "dev_batches": 1,
                     **data},
            "device": "cpu"}


def _cfg_file(tmp_path, name="c.yaml", **data):
    p = tmp_path / name
    p.write_text(yaml.safe_dump(_cfg_dict(tmp_path, **data)))
    return p


# ---------- --write-known-dev ----------

def test_write_known_dev_writes_reading_and_meta_without_overfit(tmp_path, monkeypatch):
    def boom(*a, **k):
        raise AssertionError("写参照时不该跑过拟合 / 测速")
    monkeypatch.setattr(smoke, "check_overfit", boom)
    monkeypatch.setattr(smoke, "timing", boom)
    out = tmp_path / "known.json"
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) == 0
    k = json.loads(out.read_text())
    assert set(k["readings"]) == {"hybrid"} and math.isfinite(k["readings"]["hybrid"])
    m = k["meta"]
    assert m["build_id"] == "synthetic-seed0" and m["device"] == "cpu"
    for key in smoke.KNOWN_META_KEYS:
        assert key in m


def test_written_known_dev_round_trips_and_meta_mismatch_fails(tmp_path):
    out = tmp_path / "known.json"
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) == 0
    known = json.loads(out.read_text())
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    src = train.make_source(cfg)
    ok = smoke.check_known_dev(cfg, src.dev_batches(), torch.device("cpu"), known, 1e-3, build_id=src.build_id)
    assert ok["pass"]
    # 开发份换了种子：读数可能碰巧接近，但数据不是同一份，必须判不过
    cfg2 = cfgmod.load(str(_cfg_file(tmp_path, "c2.yaml", dev_seed=999)), {})
    src2 = train.make_source(cfg2)
    bad = smoke.check_known_dev(cfg2, src2.dev_batches(), torch.device("cpu"), known, 1e9, build_id=src2.build_id)
    assert not bad["pass"] and bad["meta_mismatch"]
    # build 不同同样不过
    bad2 = smoke.check_known_dev(cfg, src.dev_batches(), torch.device("cpu"), known, 1e9, build_id="other")
    assert not bad2["pass"] and "build_id" in bad2["meta_mismatch"]


def test_known_dev_empty_readings_fail(tmp_path):
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    src = train.make_source(cfg)
    meta = smoke.known_meta(cfg, src.build_id, torch.device("cpu"))
    r = smoke.check_known_dev(cfg, src.dev_batches(), torch.device("cpu"), {"readings": {}, "meta": meta}, 1e-3,
                              build_id=src.build_id)
    assert not r["pass"]


def test_loader_mode_requires_meta(tmp_path, monkeypatch):
    """正式数据（loader）只认带 meta 的新格式：旧的扁平 {"hybrid": x} 无法核数据版本，判不过。"""
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    src = train.make_source(cfg)
    dev = src.dev_batches()
    got = smoke.check_known_dev(cfg, dev, torch.device("cpu"), {"hybrid": 0.0}, 1e9)["got"]["hybrid"]
    flat = {"hybrid": got}
    assert smoke.check_known_dev(cfg, dev, torch.device("cpu"), flat, 1e-3)["pass"]  # 合成数据：旧格式照旧可用
    cfg_l = cfgmod.from_dict({**cfgmod.to_dict(cfg), "data": {**cfgmod.to_dict(cfg)["data"], "kind": "loader"}})
    r = smoke.check_known_dev(cfg_l, dev, torch.device("cpu"), flat, 1e-3, build_id=src.build_id)
    assert not r["pass"] and r.get("meta_missing")


def test_write_known_dev_refuses_nonfinite(tmp_path, monkeypatch):
    monkeypatch.setattr(smoke, "dev_reading", lambda *a, **k: {"hybrid": float("nan")})
    out = tmp_path / "known.json"
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) != 0
    assert not out.exists()


def test_write_known_dev_creates_missing_parent_dir(tmp_path):
    out = tmp_path / "new" / "deeper" / "known.json"
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) == 0
    assert json.loads(out.read_text())["readings"]["hybrid"] is not None


def test_write_known_dev_refuses_overwrite(tmp_path):
    out = tmp_path / "known.json"
    out.write_text("{}")
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) != 0
    assert out.read_text() == "{}"


# ---------- 开发份 vs 训练份的 build id（#212）----------

class _FakeSource:
    """训练数据和开发数据各是一份（data.dev_recipe）：两个 build id 不同。"""

    def __init__(self, train_id, dev_id):
        self.build_id = train_id
        self.dev_build_id = dev_id

    def dev_batches(self, *_a, **_k):
        return [object()]


def test_write_known_dev_meta_uses_dev_build_id(tmp_path, monkeypatch):
    """参照读数是开发份的：meta.build_id 要写开发份的 id，不是训练那份。"""
    monkeypatch.setattr(smoke, "make_source", lambda cfg: _FakeSource("train-X", "dev-Y"))
    monkeypatch.setattr(smoke, "dev_reading", lambda *a, **k: {"hybrid": 1.0})
    out = tmp_path / "known.json"
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) == 0
    assert json.loads(out.read_text())["meta"]["build_id"] == "dev-Y"


def test_smoke_run_compares_known_dev_against_dev_build_id(tmp_path, monkeypatch):
    """--known-dev 传的 id 也要是开发份的，否则可能拿训练份的参照去对。"""
    monkeypatch.setattr(smoke, "make_source", lambda cfg: _FakeSource("train-X", "dev-Y"))
    seen = {}

    def fake_check_known_dev(cfg, dev, device, known, tol, build_id=None):
        seen["build_id"] = build_id
        return {"pass": True}

    monkeypatch.setattr(smoke, "check_known_dev", fake_check_known_dev)
    monkeypatch.setattr(smoke, "check_init", lambda *a, **k: {"pass": True})
    monkeypatch.setattr(smoke, "check_overfit", lambda *a, **k: {"pass": True, "verdict": "pass"})
    monkeypatch.setattr(smoke, "check_reproduce", lambda *a, **k: {"pass": True})
    monkeypatch.setattr(smoke, "timing", lambda *a, **k: {"pass": True})
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    rep = smoke.run(cfg, overfit_steps=1, overfit_lr=1e-3, timing_steps=1,
                     known_dev={"readings": {"hybrid": 1.0}}, known_tol=1e-3)
    assert seen["build_id"] == "dev-Y" and rep["pass"]


# ---------- cmp_ckpt.py ----------

def _ckpt(path, model, step=6, **extra):
    torch.save({"model": model, "step": step, **extra}, path)
    return str(path)


def _sd(seed=0):
    g = torch.Generator().manual_seed(seed)
    return {"a.weight": torch.randn(3, 4, generator=g), "a.bias": torch.randn(4, generator=g),
            "bn.num_batches_tracked": torch.tensor(7)}


def test_cmp_identical_passes(tmp_path, capsys):
    a, b = _ckpt(tmp_path / "a.pt", _sd()), _ckpt(tmp_path / "b.pt", _sd())
    assert cmp_ckpt.main([a, b]) == 0
    r = json.loads(capsys.readouterr().out)
    assert r["pass"] and r["max_abs_diff"] == 0.0 and r["n_keys"] == 3


def test_cmp_small_diff_respects_tol(tmp_path):
    sd = _sd()
    sd2 = {k: v.clone() for k, v in sd.items()}
    sd2["a.weight"][0, 0] += 1e-3
    a, b = _ckpt(tmp_path / "a.pt", sd), _ckpt(tmp_path / "b.pt", sd2)
    assert cmp_ckpt.main([a, b]) != 0                   # 默认容差 0：要求逐值相同
    assert cmp_ckpt.main([a, b, "--tol", "1e-2"]) == 0
    assert cmp_ckpt.main([a, b, "--tol", "1e-4"]) != 0


@pytest.mark.parametrize("mutate", ["missing", "extra", "shape", "dtype", "nan_both", "inf", "step", "empty"])
def test_cmp_structural_mismatch_fails_even_with_huge_tol(tmp_path, mutate):
    sd, sd2 = _sd(), _sd()
    step2 = 6
    if mutate == "missing":
        del sd2["a.bias"]
    elif mutate == "extra":
        sd2["c"] = torch.zeros(1)
    elif mutate == "shape":
        sd2["a.bias"] = torch.zeros(5)
    elif mutate == "dtype":
        sd2["a.bias"] = sd2["a.bias"].double()
    elif mutate == "nan_both":  # 两边同处 NaN：abs(NaN − NaN) ≤ tol 为假，但也不能当「相同」放过
        sd["a.weight"][1, 1] = float("nan")
        sd2["a.weight"][1, 1] = float("nan")
    elif mutate == "inf":
        sd2["a.weight"][0, 0] = float("inf")
    elif mutate == "step":
        step2 = 5
    elif mutate == "empty":
        sd, sd2 = {}, {}
    a, b = _ckpt(tmp_path / "a.pt", sd), _ckpt(tmp_path / "b.pt", sd2, step=step2)
    assert cmp_ckpt.main([a, b, "--tol", "1e9"]) != 0


@pytest.mark.parametrize("steps", [(None, None), ("6", "6"), (True, True), (-1, -1), (6.0, 6.0)])
def test_cmp_missing_or_bad_step_fails(tmp_path, steps):
    pa, pb = tmp_path / "a.pt", tmp_path / "b.pt"
    for path, st in zip((pa, pb), steps):
        d = {"model": _sd()}
        if st is not None:
            d["step"] = st
        torch.save(d, path)
    assert cmp_ckpt.main([str(pa), str(pb)]) != 0
    assert cmp_ckpt.main([str(pa), str(pb), "--ignore-step"]) == 0  # 明确不比步数时，只看模型


def test_cmp_complex_imag_part_counts(tmp_path):
    a = _ckpt(tmp_path / "a.pt", {"z": torch.tensor([1 + 2j])})
    b = _ckpt(tmp_path / "b.pt", {"z": torch.tensor([1 + 3j])})
    assert cmp_ckpt.main([a, b]) != 0


def test_cmp_missing_or_corrupt_file_exits_2(tmp_path):
    a = _ckpt(tmp_path / "a.pt", _sd())
    (tmp_path / "bad.pt").write_bytes(b"not a checkpoint")
    assert cmp_ckpt.main([a, str(tmp_path / "nope.pt")]) == 2
    assert cmp_ckpt.main([a, str(tmp_path / "bad.pt")]) == 2


def test_known_dev_reference_must_be_cpu(tmp_path, monkeypatch):
    """GPU 上写的参照拿去和 GPU 比是自比：写时拒绝，读时 meta.device 不是 cpu 就判不过。"""
    monkeypatch.setattr(train, "resolve_device", lambda d: torch.device("meta"))
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(tmp_path / "k.json")]) == 2
    monkeypatch.undo()
    out = tmp_path / "known.json"
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--write-known-dev", str(out)]) == 0
    known = json.loads(out.read_text())
    known["meta"]["device"] = "cuda:0"
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    src = train.make_source(cfg)
    r = smoke.check_known_dev(cfg, src.dev_batches(), torch.device("cpu"), known, 1e9, build_id=src.build_id)
    assert not r["pass"] and "device" in r["meta_mismatch"]


def test_cmp_rejects_negative_or_nonfinite_tol(tmp_path):
    a, b = _ckpt(tmp_path / "a.pt", _sd()), _ckpt(tmp_path / "b.pt", _sd())
    for t in ("-1", "nan", "inf"):
        assert cmp_ckpt.main([a, b, "--tol", t]) != 0


def test_cmp_real_training_checkpoints(tmp_path):
    """真训练出的检查点：同配置两次 = 相同（CPU 确定性）；与第 0 步不同。"""
    def run(name, steps):
        cfg = cfgmod.from_dict({**_cfg_dict(tmp_path), "run": {"out_dir": str(tmp_path), "name": name},
                                "schedule": {"max_steps": steps, "ckpt_every": steps, "eval_every": 1000,
                                             "log_every": 1},
                                "optim": {"warmup_steps": 1}})
        train.train(cfg)
        return str(tmp_path / name / "ckpt" / f"step-{steps:07d}.pt")
    a, b = run("x", 2), run("y", 2)
    assert cmp_ckpt.main([a, b]) == 0
    z = str(tmp_path / "x" / "ckpt" / "step-0000000.pt")
    assert cmp_ckpt.main([a, z, "--ignore-step"]) != 0
