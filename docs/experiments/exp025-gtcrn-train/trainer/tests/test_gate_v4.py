"""过拟合判据第 4 版（事后改判据，大总管定，2026-11-25）：查「流程能学」，不查「走多远」。
过线：4 段 150 步训练损失相对下降 ≥ 15% · SI-SNR 提高 > 0 · 主干各层第 1 步梯度非零有限 · B 的注意力输出层动过。
oracle 比例与 +6 dB 只记录。另：H100 乙段 300 步开发份读数要低于第 0 步（dev_gate.py）。"""
import json
from pathlib import Path

import pytest
import torch
import yaml

import config as cfgmod
import dev_gate
import gate_v4 as g4
import oracle
import smoke
import train

REPORTS = Path(__file__).resolve().parents[1] / "smoke_reports"
GOOD = {"base.encoder": 1.0, "base.dpgrnn1": 0.5, "base.dpgrnn2": 0.5, "base.decoder": 2.0,
        "attn.q": 0.0, "attn.kv": 0.0, "attn.out": 1.0, "film.fc": 0.0}


def _cfg_file(tmp_path, batch=6):
    d = {"run": {"out_dir": str(tmp_path), "name": "s"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": batch, "num_workers": 0, "dev_n": batch,
                  "dev_batches": 1, "calib_batches": 1},
         "device": "cpu"}
    p = tmp_path / "c.yaml"
    p.write_text(yaml.safe_dump(d))
    return p


def test_drop_frac():
    assert g4.drop_frac(4.0, 3.0) == pytest.approx(0.25)
    assert g4.drop_frac(4.0, 5.0) == pytest.approx(-0.25)
    for first, last in ((0.0, -1.0), (-1.0, -2.0), (float("nan"), 1.0), (4.0, float("inf"))):
        assert g4.drop_frac(first, last) is None


def test_backbone_grads_ok():
    assert g4.backbone_grads_ok(GOOD)                                    # 注意力里面的层、条件口子为 0 不算
    assert not g4.backbone_grads_ok({**GOOD, "base.dpgrnn1": 0.0})
    assert not g4.backbone_grads_ok({**GOOD, "base.decoder": float("nan")})
    assert not g4.backbone_grads_ok({"attn.out": 1.0})                   # 一个主干组都没有
    assert not g4.backbone_grads_ok(None)


def test_verdict_rule():
    v = g4.verdict
    assert v(first=4.0, last=3.4, gain_db=0.1, grads=GOOD, moved=0.1) == "pass"           # 15%
    assert v(first=4.0, last=3.41, gain_db=0.1, grads=GOOD, moved=0.1) == "fail"          # 14.75%
    assert v(first=4.0, last=3.0, gain_db=0.0, grads=GOOD, moved=0.1) == "fail"           # SI-SNR 要 > 0
    assert v(first=4.0, last=3.0, gain_db=float("nan"), grads=GOOD, moved=0.1) == "fail"
    assert v(first=4.0, last=3.0, gain_db=1.0, grads={**GOOD, "base.encoder": 0.0}, moved=0.1) == "fail"
    assert v(first=4.0, last=3.0, gain_db=1.0, grads=GOOD, moved=0.0) == "fail"           # 注意力没动
    assert v(first=4.0, last=3.0, gain_db=1.0, grads=GOOD, moved=None) == "pass"          # A 没有注意力
    assert v(first=4.0, last=float("nan"), gain_db=1.0, grads=GOOD, moved=0.1) == "fail"
    # 相对下降要初值是正的有限数，否则判据不可用
    assert v(first=0.0, last=-1.0, gain_db=1.0, grads=GOOD, moved=0.1) == "criterion_unusable"
    assert v(first=-1.0, last=-2.0, gain_db=1.0, grads=GOOD, moved=0.1) == "criterion_unusable"
    assert v(first=float("nan"), last=1.0, gain_db=1.0, grads=GOOD, moved=0.1) == "criterion_unusable"


def test_check_overfit_uses_v4_and_records_old_readings(tmp_path, monkeypatch):
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    b = train.make_source(cfg).dev_batches()[0]
    ov = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=2, lr=1e-3, reference=False)
    assert "第 4 版" in ov["rule"] and "事后改判据" in ov["rule"] and "2026-11-25" in ov["rule"]
    assert ov["verdict"] == g4.verdict(ov["first"], ov["last"], ov["si_snr_gain_db"],
                                       ov["grad_norms_first_step"], ov["attn_out_max_change"])
    assert ov["drop_frac"] == pytest.approx((ov["first"] - ov["last"]) / ov["first"])
    for k in ("drop_frac_of_oracle_span", "oracle_loss", "si_snr_gain_db"):  # 旧读数照记
        assert k in ov
    assert {"base.encoder", "base.decoder", "attn.out"} <= set(ov["grad_norms_first_step"])
    segs = ov["per_segment"]
    assert len(segs) == 4 and [s["index"] for s in segs] == [0, 1, 2, 3]
    for s in segs:
        assert {"loss_before", "loss_after", "si_snr_db_before", "si_snr_db_after", "si_snr_gain_db"} <= set(s)


def test_oracle_unusable_no_longer_blocks(tmp_path, monkeypatch):
    """第 4 版不靠 oracle 过线：oracle 不可用只记下来，不判「判据不可用」。"""
    monkeypatch.setattr(oracle, "oracle_loss", lambda *a, **k: {"loss": float("nan"), "si_snr_db": float("nan")})
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    b = train.make_source(cfg).dev_batches()[0]
    ov = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=2, lr=1e-3, reference=False)
    assert ov["first"] > 0 and ov["verdict"] in ("pass", "fail")


def test_rejudge_this_diagnosis_passes_v4():
    """按第 4 版重判 2026-11-25 那次（第 3 版判不过的那份报告 + 诊断报告里第 1 步的梯度）。"""
    probe = json.loads((REPORTS / "20261125-overfit-probe-cloud-cpu.json").read_text())
    gate3 = json.loads((REPORTS / "20261125-cpu-gate3-FAILED.json").read_text())
    r = g4.rejudge(probe, gate3)
    assert r["overfit"]["drop_frac"] == pytest.approx(0.2475, abs=1e-3)
    assert r["overfit"]["verdict"] == "pass" and r["verdict"] == "pass"
    assert r["others"] == {"init_loss": True, "reproduce": True, "known_dev": True}
    assert r["gate3_verdict"] == "fail"
    m150 = next(m for m in probe["curve"]["marks"] if m["step"] == 150)
    o = r["overfit"]  # 派生读数与同一终值对应（codex gate-v4 第 2 轮 #3）
    assert o["last"] == m150["loss"] and o["drop_frac_of_oracle_span"] == m150["drop_frac_of_oracle_span"]
    assert o["drop_frac_of_oracle_span"] == pytest.approx(oracle.frac_of_span(o["first"], o["last"], probe["oracle_loss"]))


def _run_dir(tmp_path, rows, build="b1", dev_build=None, **data):
    d = tmp_path / "run"
    d.mkdir()
    dd = {"kind": "loader", "dev_n": 2000, "dev_seed": 7, "seg_s": 4.0, "batch": 32, "dev_batches": 20, **data}
    rec = {"config": {"data": dd}, "data_build_id": build}
    if dev_build is not None:
        rec["dev_build_id"] = dev_build
    (d / "run.json").write_text(json.dumps(rec))
    (d / "metrics.jsonl").write_text("".join(json.dumps(r) + "\n" for r in rows))
    return d


KNOWN = {"readings": {"hybrid": 3.9733}, "meta": {"kind": "loader", "build_id": "b1", "dev_n": 2000, "dev_seed": 7,
                                                   "seg_s": 4.0, "batch": 32, "dev_batches": 20, "device": "cpu"}}


def test_dev_gate_pass_fail_and_tol(tmp_path):
    rows = [{"step": 1, "total": 9.0}, {"step": 300, "dev": {"hybrid": 3.90}}]
    r = dev_gate.check(_run_dir(tmp_path, rows), KNOWN, step=300, tol=1e-3)
    assert r["verdict"] == "pass" and r["dev_hybrid"] == 3.90 and r["step0_hybrid"] == 3.9733
    (tmp_path / "run").rename(tmp_path / "r1")
    near = [{"step": 300, "dev": {"hybrid": 3.9727}}]                     # 低了，但没低过跨设备容差
    assert dev_gate.check(_run_dir(tmp_path, near), KNOWN, step=300, tol=1e-3)["verdict"] == "fail"


def test_dev_gate_incomplete_and_meta_mismatch(tmp_path):
    r = dev_gate.check(_run_dir(tmp_path, [{"step": 200, "dev": {"hybrid": 1.0}}]), KNOWN, step=300, tol=1e-3)
    assert r["verdict"] == "incomplete"
    (tmp_path / "run").rename(tmp_path / "r1")
    r = dev_gate.check(_run_dir(tmp_path, [{"step": 300, "dev": {"hybrid": 1.0}}], dev_batches=63),
                       KNOWN, step=300, tol=1e-3)
    assert r["verdict"] == "incomplete" and "dev_batches" in r["reason"]
    assert dev_gate.check(tmp_path / "r1", {"readings": {"hybrid": 3.9}}, step=300, tol=1e-3)["verdict"] == "incomplete"


def test_dev_gate_uses_dev_build_id_for_reference(tmp_path):
    """参照读数是「开发份」的，要比开发份的 build id；拿训练那份比会让换了开发集的读数混过去（#212）。"""
    rows = [{"step": 300, "dev": {"hybrid": 3.90}}]
    train_ref = {**KNOWN, "meta": {**KNOWN["meta"], "build_id": "X"}}   # 参照用训练那份 id 算的：不能认
    dev_ref = {**KNOWN, "meta": {**KNOWN["meta"], "build_id": "Y"}}
    wrong = dev_gate.check(_run_dir(tmp_path, rows, build="X", dev_build="Y"), train_ref, step=300, tol=1e-3)
    assert wrong["verdict"] == "incomplete" and "build_id" in wrong["reason"]
    (tmp_path / "run").rename(tmp_path / "r1")
    right = dev_gate.check(_run_dir(tmp_path, rows, build="X", dev_build="Y"), dev_ref, step=300, tol=1e-3)
    assert right["verdict"] == "pass" and right["dev_hybrid"] == 3.90
    # 老 run.json 没有 dev_build_id：退回训练那份的 id（与今天行为一致）
    (tmp_path / "run").rename(tmp_path / "r2")
    old = dev_gate.check(_run_dir(tmp_path, rows, build="X"), train_ref, step=300, tol=1e-3)
    assert old["verdict"] == "pass"


def test_dev_gate_cli_exit_codes(tmp_path):
    d = _run_dir(tmp_path, [{"step": 300, "dev": {"hybrid": 3.0}}])
    k = tmp_path / "k.json"
    k.write_text(json.dumps(KNOWN))
    assert dev_gate.main([str(d), "--known-dev", str(k)]) == 0
    (d / "metrics.jsonl").write_text(json.dumps({"step": 300, "dev": {"hybrid": 5.0}}) + "\n")
    assert dev_gate.main([str(d), "--known-dev", str(k)]) == 1
    (d / "metrics.jsonl").write_text("")
    assert dev_gate.main([str(d), "--known-dev", str(k)]) == 2


# ---- codex 第 1 轮（gate-v4）----

def test_missing_backbone_group_fails():  # #1
    for k in g4.REQUIRED_BACKBONE:
        assert not g4.backbone_grads_ok({kk: v for kk, v in GOOD.items() if kk != k}), k
    assert not g4.backbone_grads_ok({"base.encoder": 1.0})


def test_bool_readings_rejected():  # #4
    assert g4.verdict(True, False, True, {k: True for k in g4.REQUIRED_BACKBONE}, True) != "pass"
    assert g4.drop_frac(True, False) is None
    assert not g4.backbone_grads_ok({**GOOD, "base.encoder": True})
    assert g4.verdict(4.0, 3.0, 1.0, GOOD, True) == "fail"


def test_dev_gate_rejects_weak_reference(tmp_path):  # #2
    rows = [{"step": 300, "dev": {"hybrid": 1.0}}]
    no_dev = {**KNOWN, "meta": {k: v for k, v in KNOWN["meta"].items() if k != "device"}}
    assert dev_gate.check(_run_dir(tmp_path, rows), no_dev, step=300, tol=1e-3)["verdict"] == "incomplete"
    (tmp_path / "run").rename(tmp_path / "r1")
    both_none = {**KNOWN, "meta": {**KNOWN["meta"], "seg_s": None}}
    assert dev_gate.check(_run_dir(tmp_path, rows, seg_s=None), both_none, step=300, tol=1e-3)["verdict"] == "incomplete"
    (tmp_path / "run").rename(tmp_path / "r2")
    assert dev_gate.check(_run_dir(tmp_path, [{"step": 300, "dev": {"hybrid": False}}]), KNOWN,
                          step=300, tol=1e-3)["verdict"] == "incomplete"
    (tmp_path / "run").rename(tmp_path / "r3")
    assert dev_gate.check(_run_dir(tmp_path, rows), {**KNOWN, "readings": {"hybrid": True}},
                          step=300, tol=1e-3)["verdict"] == "incomplete"
    (tmp_path / "run").rename(tmp_path / "r4")
    assert dev_gate.check(_run_dir(tmp_path, ["x"]), KNOWN, step=300, tol=1e-3)["verdict"] == "incomplete"


def test_last_is_measured_after_final_update(tmp_path):  # #5
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    b = train.make_source(cfg).dev_batches()[0]
    ov = smoke.check_overfit(cfg, b, torch.device("cpu"), steps=2, lr=1e-3, reference=False)
    assert "last_traj" in ov and ov["last"] != ov["last_traj"]  # 终值在最后一次更新之后重算，与 SI-SNR、逐段同一权重状态
