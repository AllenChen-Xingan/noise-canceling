"""单批过拟合判据第 3 版（事后改判据：AI 提、大总管定，2026-11-25）：
过线只看开发份第 0 批的前 4 段；下限 = 这 4 段的 oracle 损失；oracle 不可用时判「判据不可用」而不是「不过」；
原来 32 段的读数照算照记，只作参考。门槛（50% · +6 dB）、步数、lr 不动。"""
import json

import pytest
import torch
import yaml

import config as cfgmod
import losses
import oracle
import smoke
import train


def _cfg_file(tmp_path, batch=6):
    d = {"run": {"out_dir": str(tmp_path), "name": "s"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": batch, "num_workers": 0, "dev_n": batch,
                  "dev_batches": 1, "calib_batches": 1},
         "device": "cpu"}
    p = tmp_path / "c.yaml"
    p.write_text(yaml.safe_dump(d))
    return p


def _batch(tmp_path, batch=6):
    cfg = cfgmod.load(str(_cfg_file(tmp_path, batch)), {})
    return cfg, train.make_source(cfg).dev_batches()[0]


def test_head_slices_every_per_segment_field(tmp_path):
    _, b = _batch(tmp_path)
    h = smoke.head(b, 4)
    for k in ("noisy", "clean", "replay", "near_silent"):
        assert h[k].shape[0] == 4 and torch.equal(h[k], b[k][:4])
    for k in ("owner_ivs", "interf_ivs"):
        assert h[k] == b[k][:4]
    with pytest.raises(ValueError):
        smoke.head(b, 7)


def test_verdict_rule():
    v = smoke.overfit_verdict
    assert v(first=4.0, last=0.9, oracle_loss=-2.0, gain_db=6.5, moved=0.1) == "pass"      # 走完 51.7%
    assert v(first=4.0, last=1.1, oracle_loss=-2.0, gain_db=6.5, moved=0.1) == "fail"      # 48.3%
    assert v(first=4.0, last=0.0, oracle_loss=-2.0, gain_db=5.9, moved=0.1) == "fail"      # 增益不够
    assert v(first=4.0, last=0.0, oracle_loss=-2.0, gain_db=6.5, moved=0.0) == "fail"      # 注意力没动
    assert v(first=4.0, last=0.0, oracle_loss=-2.0, gain_db=6.5, moved=None) == "pass"     # A 没有注意力
    assert v(first=4.0, last=float("nan"), oracle_loss=-2.0, gain_db=6.5, moved=0.1) == "fail"
    # 判据本身不可用：oracle 不是有限数，或不比起点低（没有可走的距离）
    assert v(first=4.0, last=0.0, oracle_loss=float("nan"), gain_db=9.0, moved=0.1) == "criterion_unusable"
    assert v(first=4.0, last=0.0, oracle_loss=4.0, gain_db=9.0, moved=0.1) == "criterion_unusable"
    assert v(first=4.0, last=0.0, oracle_loss=5.0, gain_db=9.0, moved=0.1) == "criterion_unusable"


def test_gate_uses_first_4_segments_and_their_oracle(tmp_path):
    cfg, b = _batch(tmp_path)
    ov = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=2, lr=1e-3)
    assert ov["n_segments"] == smoke.OVERFIT_N == 4
    want = oracle.oracle_loss(losses.TrainLoss(losses.LossCfg()), smoke.head(b, 4), torch.device("cpu"))["loss"]
    assert ov["floor"] == pytest.approx(want) and ov["floor_kind"] == "oracle"
    assert ov["floor_reachable"] is True
    assert ov["verdict"] in ("pass", "fail", "criterion_unusable") and ov["pass"] == (ov["verdict"] == "pass")
    assert "floor_40db" in ov  # 旧下限照记，只作参考


def test_reference_run_recorded_but_not_gating(tmp_path, monkeypatch):
    cfg, b = _batch(tmp_path)
    ov = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=2, lr=1e-3)
    ref = ov["reference"]
    assert ref["n_segments"] == 6 and ref["steps"] == 2 and "drop_frac_of_oracle_span" in ref
    assert "pass" not in ref and "verdict" not in ref
    no_ref = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=2, lr=1e-3, reference=False)
    assert "reference" not in no_ref and no_ref["verdict"] == ov["verdict"]  # 参考跑不跑，不改过线结果


def test_criterion_unusable_propagates_to_report_and_exit_code(tmp_path, monkeypatch):
    monkeypatch.setattr(oracle, "oracle_loss", lambda *a, **k: {"loss": float("nan"), "si_snr_db": float("nan")})
    out = tmp_path / "r.json"
    rc = smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-steps", "1", "--timing-steps", "1",
                     "--out", str(out)])
    r = json.loads(out.read_text())
    assert r["overfit"]["verdict"] == "criterion_unusable" and r["verdict"] == "criterion_unusable"
    assert r["pass"] is False and rc == smoke.EXIT_CRITERION_UNUSABLE != 1


def test_cli_skip_overfit_ref(tmp_path):
    out = tmp_path / "r.json"
    smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-steps", "1", "--timing-steps", "1",
                "--skip-overfit-ref", "--out", str(out)])
    assert "reference" not in json.loads(out.read_text())["overfit"]


def test_rule_text_marks_after_the_fact_change(tmp_path):
    cfg, b = _batch(tmp_path)
    ov = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=1, lr=1e-3, reference=False)
    assert "事后改判据" in ov["rule"] and "2026-11-25" in ov["rule"]
