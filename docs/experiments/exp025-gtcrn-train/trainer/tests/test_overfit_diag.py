"""过拟合排查（大总管 2026-11-25）：`oracle.py` 的「掩码族能达到的损失」与 `smoke.py --overfit-diag`。
oracle 用合成小样核：约束（tanh 范围、ERB 展开）守住、族内能表示的目标能精确还原、比 40 dB 下限更低。"""
import json
import math

import pytest
import torch
import yaml

import losses
import oracle
import smoke
import train


def _cfg_file(tmp_path, **data):
    d = {"run": {"out_dir": str(tmp_path), "name": "s"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 2, "dev_batches": 1, **data},
         "device": "cpu"}
    p = tmp_path / "c.yaml"
    p.write_text(yaml.safe_dump(d))
    return p


def _batch(tmp_path, **data):
    import config as cfgmod
    cfg = cfgmod.load(str(_cfg_file(tmp_path, **data)), {})
    return train.make_source(cfg).dev_batches()[0]


def test_oracle_mask_respects_family_constraints(tmp_path):
    b = _batch(tmp_path)
    X, S = losses.stft(b["noisy"]), losses.stft(b["clean"])
    m_low, m_erb = oracle.family_mask(X, S)
    lim = oracle.MASK_LIM
    for z in (m_low, m_erb):
        assert z.real.abs().max() <= lim and z.imag.abs().max() <= lim
    assert m_low.shape[1] == 65 and m_erb.shape[1] == 64  # 官方 ERB(65, 64)


def test_oracle_recovers_target_inside_family():
    """目标 = 噪声输入 × 0.5（实数、在 tanh 范围内、全频同值，ERB 展开也能表示）：oracle 应几乎精确还原。"""
    g = torch.Generator().manual_seed(0)
    noisy = torch.randn(2, 16000, generator=g) * 0.1
    clean = 0.5 * noisy
    pred = oracle.oracle_spec(losses.stft(noisy), losses.stft(clean))
    y = losses.istft(pred, clean.shape[1])
    assert (losses.si_snr_db(y, clean) > 40).all()


def test_oracle_beats_initial_and_40db_floor_on_synthetic(tmp_path):
    """合成小样上：oracle 损失 < 起点模型损失，且低于 40 dB 下限（本机实测 −2.9 对 −0.4）。"""
    b = _batch(tmp_path)
    crit = losses.TrainLoss(losses.LossCfg())
    o = oracle.oracle_loss(crit, b, torch.device("cpu"))
    clean = b["clean"]
    fr = train.frames_mask(b, clean.shape[1])
    floor = smoke.near_perfect_loss(crit, clean, fr)
    assert math.isfinite(o["loss"]) and o["loss"] < floor and o["si_snr_db"] > 20


def test_floor_reachable_rule():
    """下限低于 oracle 损失 → 判下限不可达（oracle 是掩码族里的一个可行点）。"""
    assert oracle.floor_reachable(floor=-0.4, oracle_loss=-2.9)
    assert not oracle.floor_reachable(floor=-3.3, oracle_loss=-2.0)
    assert oracle.floor_reachable(floor=-2.0, oracle_loss=-2.0)
    assert not oracle.floor_reachable(floor=float("nan"), oracle_loss=-2.0)
    assert not oracle.floor_reachable(floor=-1.0, oracle_loss=float("nan"))


def test_frac_of_span():
    assert oracle.frac_of_span(4.0, 1.0, -2.0) == pytest.approx(0.5)
    assert oracle.frac_of_span(4.0, 1.0, 4.0) is None       # 距离 ≤ 0：没有意义
    assert oracle.frac_of_span(4.0, float("nan"), -2.0) is None


def test_overfit_diag_cli_report(tmp_path):
    out = tmp_path / "new" / "diag.json"
    rc = smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-steps", "3", "--overfit-diag", str(out)])
    assert rc == 0
    r = json.loads(out.read_text())
    for k in ("first", "last", "floor", "drop_frac_of_span", "oracle_loss", "oracle_si_snr_db",
              "drop_frac_of_oracle_span", "floor_reachable", "si_snr_gain_db", "build_id", "device", "batch_index"):
        assert k in r, k
    assert r["steps"] == 3 and r["batch_index"] == 0
    assert r["floor_reachable"] == oracle.floor_reachable(r["floor"], r["oracle_loss"])
    assert "pass" not in r  # 只是排查，不判过不过


def test_overfit_diag_refuses_overwrite_and_nonfinite(tmp_path, monkeypatch):
    out = tmp_path / "diag.json"
    out.write_text("{}")
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-steps", "1", "--overfit-diag", str(out)]) != 0
    assert out.read_text() == "{}"
    out2 = tmp_path / "diag2.json"
    monkeypatch.setattr(oracle, "oracle_loss", lambda *a, **k: {"loss": float("nan"), "si_snr_db": float("nan")})
    assert smoke.main(["--config", str(_cfg_file(tmp_path)), "--overfit-steps", "1", "--overfit-diag", str(out2)]) != 0
    assert not out2.exists()
