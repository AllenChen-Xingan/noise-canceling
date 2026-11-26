"""过拟合慢的诊断（大总管 2026-11-25）：学习曲线 · 哪些参数在动 · 学习率。只诊断，不判过不过、不改判据。"""
import json

import pytest
import torch
import yaml

import config as cfgmod
import losses
import models
import overfit_probe as op
import smoke
import train


def _cfg_file(tmp_path, batch=4):
    d = {"run": {"out_dir": str(tmp_path), "name": "s"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": batch, "num_workers": 0, "dev_n": batch,
                  "dev_batches": 1, "calib_batches": 1},
         "device": "cpu"}
    p = tmp_path / "c.yaml"
    p.write_text(yaml.safe_dump(d))
    return p


def _setup(tmp_path):
    cfg = cfgmod.load(str(_cfg_file(tmp_path)), {})
    b = train.make_source(cfg).dev_batches()[0]
    m = smoke._model(cfg, torch.device("cpu"))
    train.set_train(m, cfg)
    return cfg, b, m


def test_marks():
    assert op.marks_for(2000) == [150, 300, 500, 1000, 2000]
    assert op.marks_for(400) == [150, 300, 400]
    assert op.marks_for(4) == [4]


def test_param_report_matches_counts_and_bn_frozen(tmp_path):
    cfg, _, m = _setup(tmp_path)
    r = op.param_report(m)
    assert r["trainable"] == models.param_counts(m)["trainable"]
    assert sum(g["trainable"] for g in r["groups"].values()) == r["trainable"]
    assert r["bn_layers"] > 0 and r["bn_in_train_mode"] == 0          # 冻结：批归一化全在评估模式
    assert r["bn_affine_trainable"] is True                            # 缩放与偏移照训，没被连带冻住
    assert set(r["frozen_param_names"]) <= {n for n in models.param_counts(m)["fixed_names"]}  # 只有 ERB 矩阵


def test_grad_norms_cover_groups_and_show_zero_init_head(tmp_path):
    cfg, b, m = _setup(tmp_path)
    crit = losses.TrainLoss(losses.LossCfg())
    loss, _ = train.forward_loss(m, crit, b, torch.device("cpu"))
    loss.backward()
    g = op.grad_norms(m)
    assert set(g) == {k for k, v in op.param_report(m)["groups"].items() if v["trainable"]}
    assert all(v == v and v >= 0 for v in g.values())                 # 有限、非负
    assert g["attn.out"] > 0                                           # 零初始化输出层自己有梯度
    assert g["attn.q"] == 0 and g["attn.kv"] == 0                      # 输出层为 0 时里面的层第一步没梯度（预期）
    assert g["film.fc"] == 0                                           # 条件口子喂全零（D98）


def test_cli_probe_report(tmp_path):
    out = tmp_path / "p.json"
    rc = smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-probe", str(out),
                     "--probe-steps", "4", "--probe-sweep-steps", "2"])
    assert rc == 0
    r = json.loads(out.read_text())
    assert r["complete"] is True and r["n_segments"] == 4 and "oracle_loss" in r
    c = r["curve"]
    assert c["lr"] == 1e-3 and [p["step"] for p in c["marks"]] == [4]
    for k in ("loss", "drop_frac_of_oracle_span", "si_snr_db", "si_snr_gain_db"):
        assert k in c["marks"][0]
    assert {s["lr"] for s in r["lr_sweep"]} == {3e-4, 3e-3} and all(s["steps"] == 2 for s in r["lr_sweep"])
    assert "grad_norms_first_step" in c and "grad_norms_last_step" in c and "params" in r
    assert "pass" not in r and "verdict" not in r


def test_probe_writes_partial_results_before_crash(tmp_path, monkeypatch):
    """中途出错（例如超时被杀前的异常）：已跑完的曲线留在文件里，complete 为 False。"""
    real = op.run_curve
    calls = {"n": 0}

    def flaky(*a, **k):
        calls["n"] += 1
        if calls["n"] == 2:
            raise RuntimeError("boom")
        return real(*a, **k)
    monkeypatch.setattr(op, "run_curve", flaky)
    out = tmp_path / "p.json"
    with pytest.raises(RuntimeError):
        smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-probe", str(out),
                    "--probe-steps", "3", "--probe-sweep-steps", "2"])
    r = json.loads(out.read_text())
    assert r["complete"] is False and r["curve"]["marks"][-1]["step"] == 3


def test_probe_refuses_overwrite(tmp_path):
    out = tmp_path / "p.json"
    out.write_text("{}")
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-probe", str(out),
                       "--probe-steps", "2", "--probe-sweep-steps", "1"]) == 2
    assert out.read_text() == "{}"
