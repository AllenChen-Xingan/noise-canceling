"""训练损失：梯度检查（027 3.2 第 2 条）· 样本不足规则（第 3 条）· 组合与开关 · 权重标定。"""
import numpy as np
import pytest
import torch

import losses
from official import official_loss_module
from test_losses_parity import N, _speechlike


def _pair(seed=0, n_bursts=6):
    rng = np.random.default_rng(seed)
    c, iv = _speechlike(rng, N, n_bursts)
    o = c * 0.6 + rng.standard_normal(N).astype(np.float32) * 0.01
    o[iv[0][0]:iv[0][0] + 800] *= 0.2
    return (torch.as_tensor(c, dtype=torch.float64)[None],
            torch.as_tensor(o, dtype=torch.float64)[None], iv)


def _directional(f, x, n_dir=6, eps=1e-6, seed=0, scale=1.0):
    """有限差分核梯度：随机方向 v 上，(f(x+εv) − f(x−εv)) / 2ε 与 ∇f·v 比。
    判据是真正的相对差：|fd − an| ≤ 1e-5·|an| + 1e-12（近零时只靠极小的绝对下限，codex 第 10 轮 #2）。
    scale ≠ 1 时把解析梯度故意放大，给反向对照用。返回 (梯度, 是否全部通过)。"""
    g = torch.Generator().manual_seed(seed)
    x = x.clone().requires_grad_(True)
    y = f(x)
    (grad,) = torch.autograd.grad(y, x)
    ok = True
    for _ in range(n_dir):
        v = torch.randn(x.shape, generator=g, dtype=x.dtype)
        with torch.no_grad():
            fd = (f(x + eps * v) - f(x - eps * v)) / (2 * eps)
        an = (grad * scale * v).sum()
        assert abs(an.item()) > 1e-9  # 方向导数不是近零，相对判据才有意义
        ok &= abs(fd.item() - an.item()) <= 1e-5 * abs(an.item()) + 1e-12
    return grad, ok


def test_splvl_gradient_finite_difference():
    c, o, iv = _pair(1)
    fr = torch.as_tensor(losses.owner_only_mask(iv, [], N))[None]
    f = lambda x: losses.splvl_db(x, c, fr)[0].sum()  # noqa: E731
    grad, ok = _directional(f, o)
    assert ok and grad.abs().sum() > 0
    assert not _directional(f, o, scale=1.01)[1]  # 反向对照：梯度错 1% 就红


def test_onset_gradient_finite_difference():
    c, o, _ = _pair(2)
    f = lambda x: losses.onset_excess_db(x, c)[0].sum()  # noqa: E731
    grad, ok = _directional(f, o)
    assert ok and grad.abs().sum() > 0
    assert not _directional(f, o, scale=1.01)[1]


def test_short_samples_skipped_not_in_denominator():
    """批里一段有效、一段不足：该项 = 有效那段的值（不除以 2）；整批无效 = 0 且仍可反传。"""
    c1, o1, iv1 = _pair(3)
    c2 = torch.zeros_like(c1)
    c2[0, 4000:7000] = 0.1  # 不足 50 帧、只有 1 个字头
    c, o = torch.cat([c1, c2]), torch.cat([o1, c2 * 0.5])
    fr = torch.as_tensor(np.stack([losses.owner_only_mask(iv1, [], N),
                                   losses.owner_only_mask([(4000, 7000)], [], N)]))
    d, v = losses.splvl_db(o, c, fr)
    assert v.tolist() == [True, False]
    assert losses.masked_mean(-d, v).item() == pytest.approx(-d[0].item())
    e, ve, cnt = losses.onset_excess_db(o, c)
    assert ve.tolist() == [True, False] and cnt[1] < losses.ONSET_MIN
    x = o[1:].clone().requires_grad_(True)
    dd, vv = losses.splvl_db(x, c[1:], fr[1:])
    m = losses.masked_mean(dd, vv)
    m.backward()
    assert m.item() == 0.0 and x.grad.abs().sum() == 0


def _batch(seed=4):
    c, o, iv = _pair(seed)
    c, o = c.float(), o.float()
    fr = torch.as_tensor(losses.owner_only_mask(iv, [], N))[None]
    return losses.stft(o), c, fr


def test_trainloss_defaults_equal_official_hybrid():
    """默认开关（保护项权重 0、其余全关）= 官方 HybridLoss 逐值相同。"""
    ps, c, fr = _batch()
    tot, logs = losses.TrainLoss(losses.LossCfg())(ps, c, fr)
    ref = official_loss_module().HybridLoss()(ps, losses.stft(c))
    assert torch.equal(tot, ref)
    assert set(logs) == {"hybrid", "total"}


def test_lvl_margin():
    """压低 < 余量时 L_lvl = 0；压低 > 余量时 = 压低 − 余量。"""
    c, _, iv = _pair(5)
    c = c.float()
    fr = torch.as_tensor(losses.owner_only_mask(iv, [], N))[None]
    cfg = losses.LossCfg(w_lvl=1.0)
    for gain, want in ((10 ** (-1 / 20), 0.0), (10 ** (-5 / 20), 3.0)):
        _, logs = losses.TrainLoss(cfg)(losses.stft(c * gain), c, fr)
        assert logs["lvl"] == pytest.approx(want, abs=0.05)


def test_onset_term_one_sided():
    """字头比整段少压（超额为负）时 L_on = 0；吞字头时为正。"""
    ps, c, fr = _batch(6)
    cfg = losses.LossCfg(w_on=1.0)
    _, good = losses.TrainLoss(cfg)(losses.stft(c * 0.9), c, fr)
    _, bad = losses.TrainLoss(cfg)(ps, c, fr)
    assert good["on"] == pytest.approx(0.0, abs=1e-3) and bad["on"] > 1.0


def test_level_norm_makes_hybrid_gain_invariant():
    """电平归一开时，输出与目标同乘 k，损失不变（Braun 归一的作用）；关时会变。"""
    ps, c, fr = _batch(7)
    on, off = losses.TrainLoss(losses.LossCfg(level_norm=True)), losses.TrainLoss(losses.LossCfg())
    k = 0.05
    assert on(ps * k, c * k, fr)[0].item() == pytest.approx(on(ps, c, fr)[0].item(), rel=1e-3)
    assert off(ps * k, c * k, fr)[0].item() != pytest.approx(off(ps, c, fr)[0].item(), rel=1e-2)


def test_kd_only_on_replay_and_sp_pulls_back():
    ps, c, fr = _batch(8)
    ps2 = torch.cat([ps, ps])
    c2, fr2 = torch.cat([c, c]), torch.cat([fr, fr])
    teacher = ps2.clone()
    teacher[1] = teacher[1] * 2.0  # 只有第 2 段老师与学生不同
    cfg = losses.LossCfg(kd=True)
    _, no_replay = losses.TrainLoss(cfg)(ps2, c2, fr2, teacher_spec=teacher, replay=torch.tensor([True, False]))
    _, replay = losses.TrainLoss(cfg)(ps2, c2, fr2, teacher_spec=teacher, replay=torch.tensor([False, True]))
    assert no_replay["kd"] == 0.0 and replay["kd"] > 0
    lin = torch.nn.Linear(3, 3)
    ref = {k: v.detach().clone() for k, v in lin.named_parameters()}
    assert losses.l2sp(lin, ref).item() == 0.0
    with torch.no_grad():
        lin.weight += 0.5
    assert losses.l2sp(lin, ref).item() == pytest.approx(9 * 0.25)


def test_calibrate_weight():
    assert losses.calibrate_weight(2.0, 4.0) == pytest.approx(0.05)
    assert losses.calibrate_weight(2.0, 0.0) == pytest.approx(0.2)  # 初始为 0：按「超出 1 dB = 10%」


def test_stft_roundtrip():
    x = torch.randn(2, N) * 0.1
    assert (losses.istft(losses.stft(x), N) - x).abs().max() < 1e-5
