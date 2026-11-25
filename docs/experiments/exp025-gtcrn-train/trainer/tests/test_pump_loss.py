"""人声电平稳定损失（大总管派活，人同意 D107）：可微复刻 V-PUMPV「人声忽大忽小」，开关 loss.w_pump，默认 0。
对拍 vdn-eval 正本见 tests/test_losses_parity.py 的 pumpv 两条。"""
import numpy as np
import pytest
import torch

import config as cfgmod
import losses
import train

N = 16_000 * 4


def _speech(rng, n=N):
    return (rng.standard_normal(n) * 0.1).astype(np.float32)


def _owner(n=N):
    return torch.ones(1, n // losses.MASK_FRAME, dtype=torch.bool)


def test_constant_gain_reads_zero():
    rng = np.random.default_rng(0)
    c = torch.as_tensor(_speech(rng))[None].double()
    v, ok, nw = losses.pumpv_var(0.5 * c, c, _owner())
    assert ok.item() and nw[0] == 40 and v.item() == pytest.approx(0.0, abs=1e-9)


def test_alternating_gain_reads_its_variance():
    """每 100 ms 窗交替 +3 / −3 dB（功率）：方差 = 9 dB²。"""
    rng = np.random.default_rng(1)
    c = torch.as_tensor(_speech(rng))[None].double()
    g = torch.ones(N, dtype=torch.float64)
    for k in range(40):
        g[k * 1600:(k + 1) * 1600] = 10 ** ((3 if k % 2 else -3) / 20)
    v, ok, _ = losses.pumpv_var(c * g, c, _owner())
    assert ok.item() and v.item() == pytest.approx(9.0, rel=1e-9)


def test_needs_10_voiced_windows_and_80_percent():
    rng = np.random.default_rng(2)
    c = torch.as_tensor(_speech(rng))[None].double()
    fr = torch.zeros(1, N // 160, dtype=torch.bool)
    fr[0, :90] = True                     # 9 个整窗
    assert not losses.pumpv_var(c, c, fr)[1].item()
    fr[0, 90:98] = True                   # 第 10 窗 8 / 10 帧有声：算
    assert losses.pumpv_var(c, c, fr)[1].item()
    fr[0, 90:98] = False
    fr[0, 90:97] = True                   # 7 / 10：不算
    assert not losses.pumpv_var(c, c, fr)[1].item()


def test_zero_clean_window_skipped_and_all_zero_invalid():
    c = torch.zeros(1, N, dtype=torch.float64)
    v, ok, nw = losses.pumpv_var(c + 0.01, c, _owner())
    assert not ok.item() and nw[0] == 0 and v.item() == 0.0


def test_owner_mask_matches_rust_semantics():
    """V-PUMPV 用主讲人掩码（paint(owner)，不扣旁人），不是 V-SPLVL 的 owner && !interf。"""
    own, itf = [(0, N)], [(16000, 32000)]
    m = losses.owner_mask(own, N)
    assert m.all() and m.shape == (N // 160,)
    assert not losses.owner_only_mask(own, itf, N).all()


def _crit(**kw):
    return losses.TrainLoss(losses.LossCfg(**kw))


def _pred_and_clean(seed=3):
    rng = np.random.default_rng(seed)
    clean = torch.as_tensor(np.stack([_speech(rng), _speech(rng)]))
    noisy = clean + 0.05 * torch.randn(clean.shape, generator=torch.Generator().manual_seed(seed))
    pred = losses.stft(noisy).requires_grad_(True)
    fr = torch.ones(2, N // 160, dtype=torch.bool)
    return pred, clean, fr


def test_w_pump_zero_is_bit_identical():
    pred, clean, fr = _pred_and_clean()
    t0, l0 = _crit()(pred, clean, fr)
    g0 = torch.autograd.grad(t0, pred)[0]
    t1, l1 = _crit(w_pump=0.0)(pred, clean, fr, owner_frames=fr)
    g1 = torch.autograd.grad(t1, pred)[0]
    assert torch.equal(t0, t1) and torch.equal(g0, g1) and l0 == l1 and "pump" not in l1


def test_w_pump_positive_adds_term_with_gradient():
    pred, clean, fr = _pred_and_clean()
    t0, _ = _crit()(pred, clean, fr)
    t1, logs = _crit(w_pump=0.5)(pred, clean, fr, owner_frames=fr)
    assert {"pump", "pump_valid"} <= set(logs) and logs["pump_valid"] == 2
    assert t1.item() == pytest.approx(t0.item() + 0.5 * logs["pump"], rel=1e-5)
    g = torch.autograd.grad(t1 - t0, pred)[0]
    assert torch.isfinite(g).all() and g.abs().sum() > 0
    with pytest.raises(ValueError, match="owner_frames"):
        _crit(w_pump=0.5)(pred, clean, fr)


def test_descending_the_term_reduces_pumping():
    """同向：对 V-PUMPV 复刻求导往下走，输出的忽大忽小读数变小。"""
    rng = np.random.default_rng(4)
    c = torch.as_tensor(_speech(rng))[None].double()
    g = torch.as_tensor(1 + 0.5 * np.sin(np.arange(N) * 2 * np.pi * 2 / 16000))[None]
    gain = torch.zeros(1, 40, dtype=torch.float64, requires_grad=True)  # 每窗一个可学的增益（dB）
    opt = torch.optim.Adam([gain], lr=0.1)
    first = None
    for _ in range(50):
        o = c * g * (10 ** (gain / 20)).repeat_interleave(1600, dim=1)
        v, _, _ = losses.pumpv_var(o, c, _owner())
        first = v.item() if first is None else first
        opt.zero_grad()
        v.sum().backward()
        opt.step()
    assert first > 1.0 and v.item() < 0.1 * first


def test_config_default_and_validation():
    cfg = cfgmod.from_dict({})
    assert cfg.loss.w_pump == 0.0
    for bad in (-1.0, "auto", float("nan")):
        with pytest.raises(ValueError, match="w_pump"):
            cfgmod.from_dict({"loss": {"w_pump": bad}})
    lc = train.loss_cfg(cfgmod.from_dict({"loss": {"w_pump": 0.3}}), 1.0, 1.0)
    assert lc.w_pump == 0.3


def test_forward_loss_passes_owner_frames_only_when_on(monkeypatch):
    seen = []
    real = losses.TrainLoss.forward

    def spy(self, *a, **k):
        seen.append("owner_frames" in k)
        return real(self, *a, **k)
    monkeypatch.setattr(losses.TrainLoss, "forward", spy)
    rng = np.random.default_rng(5)
    clean = torch.as_tensor(np.stack([_speech(rng)]))
    b = {"noisy": clean, "clean": clean, "replay": torch.zeros(1, dtype=torch.bool), "near_silent": torch.zeros(1),
         "owner_ivs": [[(0, N)]], "interf_ivs": [[]]}
    m = train.build_model(cfgmod.from_dict({"model": {"variant": "A"}, "device": "cpu"}), torch.device("cpu"))
    train.forward_loss(m, _crit(), b, torch.device("cpu"))
    _, logs = train.forward_loss(m, _crit(w_pump=1.0), b, torch.device("cpu"))
    assert seen == [False, True] and logs["pump_valid"] == 1


def test_training_with_w_pump_zero_never_touches_the_term(tmp_path, monkeypatch):
    """w_pump = 0：整条训练不算这一项，也不多算主讲人掩码；检查点与默认配置逐值相同。"""
    import cmp_ckpt
    import data
    from test_train import _cfg

    def boom(*a, **k):
        raise AssertionError("w_pump = 0 时不该被调用")
    monkeypatch.setattr(losses, "pumpv_var", boom)
    monkeypatch.setattr(data, "owner_frames", boom)
    monkeypatch.setattr(train, "owner_frames", boom)
    train.train(_cfg(tmp_path / "a"))
    train.train(_cfg(tmp_path / "b", **{"loss.w_pump": 0.0}))
    r = cmp_ckpt.compare(torch.load(tmp_path / "a/t/ckpt/step-0000006.pt", weights_only=False),
                         torch.load(tmp_path / "b/t/ckpt/step-0000006.pt", weights_only=False), tol=0.0)
    assert r["pass"], r


def test_training_with_w_pump_on_logs_the_term(tmp_path):
    from test_train import _cfg
    train.train(_cfg(tmp_path, **{"loss.w_pump": 0.1}))
    import json
    rows = [json.loads(x) for x in (tmp_path / "t" / "metrics.jsonl").read_text().splitlines()]
    assert any("pump_valid" in r for r in rows if "dev" not in r)


# ---- 大总管定（2026-11-25）：pump_on 可选 all | clean_only；老 run.json 缺字段按默认值处理 ----

def _strip_pump_from_record(root):
    import json
    p = root / "run.json"
    rec = json.loads(p.read_text())
    for k in ("w_pump", "pump_on"):
        rec["config"]["loss"].pop(k, None)
    p.write_text(json.dumps(rec))


def test_old_run_json_without_pump_fields_can_resume(tmp_path):
    from test_train import _cfg
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 3}))
    _strip_pump_from_record(tmp_path / "t")          # 模拟改动前写的运行记录
    assert train.train(_cfg(tmp_path))["step"] == 6


def test_old_run_json_refused_when_pump_actually_turned_on(tmp_path):
    from test_train import _cfg
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 3}))
    _strip_pump_from_record(tmp_path / "t")
    with pytest.raises(RuntimeError, match="loss.w_pump"):
        train.train(_cfg(tmp_path, **{"loss.w_pump": 0.5}))


def test_pump_on_config():
    assert cfgmod.from_dict({}).loss.pump_on == "all"
    with pytest.raises(ValueError, match="pump_on"):
        cfgmod.from_dict({"loss": {"pump_on": "noisy"}})
    assert train.loss_cfg(cfgmod.from_dict({"loss": {"pump_on": "clean_only"}}), 1.0, 1.0).pump_on == "clean_only"


def test_clean_only_counts_only_clean_input_segments():
    """clean_only：只算输入几乎就是目标的段（输入 − 目标 的能量比目标低 ≥ 30 dB）。"""
    pred, clean, fr = _pred_and_clean()           # 两段都加了 −6 dB 左右的噪声
    noisy = clean.clone()
    noisy[0] = noisy[0] + 0.05 * torch.randn(N, generator=torch.Generator().manual_seed(9))
    assert losses.clean_input(noisy, clean).tolist() == [False, True]
    _, la = _crit(w_pump=0.5)(pred, clean, fr, owner_frames=fr, noisy=noisy)
    _, lc = _crit(w_pump=0.5, pump_on="clean_only")(pred, clean, fr, owner_frames=fr, noisy=noisy)
    assert la["pump_valid"] == 2 and lc["pump_valid"] == 1
    with pytest.raises(ValueError, match="noisy"):
        _crit(w_pump=0.5, pump_on="clean_only")(pred, clean, fr, owner_frames=fr)


def test_forward_loss_passes_noisy_for_clean_only(monkeypatch):
    rng = np.random.default_rng(6)
    clean = torch.as_tensor(np.stack([_speech(rng)]))
    b = {"noisy": clean, "clean": clean, "replay": torch.zeros(1, dtype=torch.bool), "near_silent": torch.zeros(1),
         "owner_ivs": [[(0, N)]], "interf_ivs": [[]]}
    m = train.build_model(cfgmod.from_dict({"model": {"variant": "A"}, "device": "cpu"}), torch.device("cpu"))
    _, logs = train.forward_loss(m, _crit(w_pump=1.0, pump_on="clean_only"), b, torch.device("cpu"))
    assert logs["pump_valid"] == 1


# ---- codex pump-loss 第 1 轮 ----

def test_evaluate_weights_pump_by_valid_segments(monkeypatch):  # #1
    seq = iter([({"total": 1.0, "hybrid": 1.0, "pump": 9.0, "pump_valid": 1}),
                ({"total": 1.0, "hybrid": 1.0, "pump": 0.0, "pump_valid": 0})])
    monkeypatch.setattr(train, "forward_loss", lambda *a, **k: (None, next(seq)))
    m = train.build_model(cfgmod.from_dict({"model": {"variant": "A"}, "device": "cpu"}), torch.device("cpu"))
    out = train.evaluate(m, None, [1, 2], torch.device("cpu"), cfgmod.from_dict({}))
    assert out["pump"] == 9.0 and out["pump_valid"] == 1


@pytest.mark.parametrize("kind", ["zero_out", "tiny_out", "zero_clean", "w0", "w1", "w9", "w10", "clean_only_none"])
def test_backward_finite_on_edge_inputs(kind):  # #3：float32 反传不出 NaN / inf
    rng = np.random.default_rng(7)
    c = torch.as_tensor(_speech(rng))[None]
    o = (c * 0.7).clone()
    fr = _owner()
    if kind == "zero_out":
        o = torch.zeros_like(c)
    elif kind == "tiny_out":
        o = c * 1e-12
    elif kind == "zero_clean":
        c = torch.zeros_like(c)
    elif kind.startswith("w"):
        fr = torch.zeros_like(fr)
        fr[0, : 10 * int(kind[1:])] = True
    o = o.requires_grad_(True)
    if kind == "clean_only_none":
        pv, v, _ = losses.pumpv_var(o, c, fr)
        v = v & losses.clean_input(c + 0.1, c)
    else:
        pv, v, _ = losses.pumpv_var(o, c, fr)
    l = losses.masked_mean(pv, v)
    l.backward()
    assert torch.isfinite(l) and torch.isfinite(o.grad).all()


def test_short_owner_mask_reads_as_unvoiced_tail():  # codex pump-loss 第 2 轮 #1：掩码比音频短 = 超出部分按无声
    rng = np.random.default_rng(8)
    c = torch.as_tensor(_speech(rng))[None].double()
    o = c * (1 + 0.3 * torch.as_tensor(rng.standard_normal(N)))[None]
    full = torch.zeros(1, N // 160, dtype=torch.bool)
    full[0, :300] = True
    short = torch.ones(1, 300, dtype=torch.bool)
    a, b = losses.pumpv_var(o, c, full), losses.pumpv_var(o, c, short)
    assert torch.equal(a[0], b[0]) and a[2] == b[2] == [30]
