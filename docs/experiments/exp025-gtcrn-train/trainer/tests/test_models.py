"""模型变体 A / B：参数量账 · 与官方逐值相同 · 零初始化无损 · 不看未来帧 · 与 exp019 流式版对拍。"""
import importlib.util
import sys

import pytest
import torch

import models
from official import EXP019, load_start_state, official_module

T = 40  # 帧数（≈ 0.64 s），小到 CPU 秒级


def _spec(seed=0, t=T, b=2):
    g = torch.Generator().manual_seed(seed)
    return torch.randn(b, 257, t, 2, generator=g) * 0.1


@pytest.fixture(scope="module")
def state():
    return load_start_state()


def _a(state):
    m = models.build("A").eval()
    m.load_base(state)
    return m


def _b(state, **kw):
    m = models.build("B", **kw).eval()
    m.load_base(state)
    return m


def test_param_counts_explain_48245_vs_23_67k():
    """实测 48,245 = 可训练 23,669（论文 23.67 K）+ ERB 两个固定滤波器组 2 × 64 × 192 = 24,576。"""
    pc = models.param_counts(models.build("A"))
    assert pc["total"] == 48245
    assert pc["trainable"] == 23669
    assert pc["fixed_names"] == {"base.erb.erb_fc.weight": 12288, "base.erb.ierb_fc.weight": 12288}


def test_b_extra_params():
    """B 比 A 多：注意力 q 16×16+16 · kv 16×32+32 · out 16×16+16 = 1,088（028 表）；条件口子 8×32 = 256。"""
    a = models.param_counts(models.build("A"))["trainable"]
    b = models.param_counts(models.build("B"))["trainable"]
    assert b - a == 1088 + 256


def test_a_equals_official(state):
    off = official_module().GTCRN().eval()
    off.load_state_dict(state)
    x = _spec()
    with torch.no_grad():
        assert torch.equal(_a(state)(x), off(x))


def test_b_rewritten_forward_equals_official(state):
    """B 重写的串接顺序（两层都关）与官方 forward 逐值相同——防止重写时漏了一步。"""
    m = _b(state, attn=False, film=False)
    x = _spec(1)
    with torch.no_grad():
        assert torch.equal(m(x), _a(state)(x))


def test_b_zero_init_equals_a(state):
    """零初始化下 B 与 A 输出差 ≤ 1e-6，cond 给全零、给随机向量、不给，都一样（交代第 3 条）。"""
    a, b = _a(state), _b(state)
    x = _spec(2)
    with torch.no_grad():
        ya = a(x)
        for cond in (None, torch.zeros(2, 8), torch.randn(2, 8), torch.randn(2, T, 8)):
            assert (b(x, cond) - ya).abs().max().item() <= 1e-6


def test_film_zero_cond_is_identity_after_training(state):
    """条件口子权重非零（模拟训后）时，cond 全零仍等于不带口子的模型（D98）。"""
    b = _b(state)
    with torch.no_grad():
        b.film.fc.weight.normal_()
        b.attn.out.weight.normal_(std=0.1)
    ref = _b(state, film=False)
    ref.attn.load_state_dict(b.attn.state_dict())
    x = _spec(3)
    with torch.no_grad():
        assert (b(x, torch.zeros(2, 8)) - ref(x)).abs().max().item() <= 1e-6
        assert (b(x, torch.randn(2, 8)) - ref(x)).abs().max().item() > 1e-3  # 非零向量确实起作用


def test_zero_init_layers_still_get_gradient(state):
    """零初始化的输出层不会卡死：一次反传后 attn.out 与 film.fc 的梯度非零。"""
    b = _b(state).train()
    y = b(_spec(4), torch.randn(2, 8))
    y.pow(2).mean().backward()
    assert b.attn.out.weight.grad.abs().sum() > 0
    assert b.film.fc.weight.grad.abs().sum() > 0


@pytest.mark.parametrize("variant", ["A", "B"])
def test_no_lookahead(state, variant):
    """改第 t0 帧及以后的输入，t0 之前的输出逐值不变（不看未来帧，028 硬条件）。B 用非零权重测。"""
    m = _a(state) if variant == "A" else _b(state)
    if variant == "B":
        with torch.no_grad():
            m.attn.out.weight.normal_(std=0.3)
            m.film.fc.weight.normal_()
    x1 = _spec(5)
    x2 = x1.clone()
    t0 = 25
    x2[:, :, t0:] = torch.randn_like(x2[:, :, t0:])
    cond = torch.randn(2, T, 8)
    with torch.no_grad():
        y1, y2 = m(x1, cond), m(x2, cond)
    assert (y1[:, :, :t0] - y2[:, :, :t0]).abs().max().item() <= 1e-6
    assert (y1[:, :, t0:] - y2[:, :, t0:]).abs().max().item() > 1e-3  # 反向对照：未来帧确实被改了


def _exp019_variants():
    name = "exp019_variants"
    if name in sys.modules:
        return sys.modules[name]
    spec = importlib.util.spec_from_file_location(name, EXP019 / "variants.py")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


@pytest.mark.parametrize("k", [4, 32])
def test_offline_matches_exp019_stream(k):
    """整段版注意力与 exp019 逐帧流式版（正本：以后 Rust 引擎照它实现）用同一组非零权重逐帧对拍。

    T 取得比 K 长，覆盖「开头补零」和「窗口已满」两段。
    """
    torch.manual_seed(k)
    off = models.TempAttnSeq(k_frames=k)
    st = _exp019_variants().TempAttn(k_frames=k)
    with torch.no_grad():
        for p in off.parameters():
            p.normal_(std=0.5)
    st.load_state_dict(off.state_dict())
    t = k + 9
    x = torch.randn(2, 16, t, 33)
    with torch.no_grad():
        y_off = off(x)
        s = st.init_state(2)
        frames = []
        for i in range(t):
            y, s = st(x[:, :, i:i + 1], s)
            frames.append(y)
        y_st = torch.cat(frames, dim=2)
    assert (y_off - y_st).abs().max().item() <= 1e-5


def test_unknown_variant():
    with pytest.raises(ValueError):
        models.build("C")
