"""自审（2026-11-24，三个对抗视角）每条修复的回归测试。按 损失 · 模型 · 训练循环 分组。"""
import json

import numpy as np
import pytest
import torch

import config as cfgmod
import data
import losses
import models
import train
from official import load_start_state
from test_losses_parity import N, _speechlike


# ---------- 损失 ----------

def test_active_rms_ignores_pause_fraction():
    """电平归一按说话帧的电平：说话电平相同、停顿占比不同的两段，除数相同（整段 std 会差约 5 dB）。"""
    n = 32000
    a, b = torch.zeros(1, n), torch.zeros(1, n)
    tone = 0.1 * torch.sin(torch.arange(n) * 0.3)
    a[0, :28800] = tone[:28800]  # 90% 在说
    b[0, :9600] = tone[:9600]  # 30% 在说
    fa = torch.as_tensor(losses.owner_only_mask([(0, 28800)], [], n))[None]
    fb = torch.as_tensor(losses.owner_only_mask([(0, 9600)], [], n))[None]
    ra, rb = losses.active_rms(a, fa, 1e-3), losses.active_rms(b, fb, 1e-3)
    assert ra.item() == pytest.approx(rb.item(), rel=1e-3)
    assert a.std().item() / b.std().item() > 1.5  # 反向对照：整段 std 确实差很多


def test_active_rms_near_silent_uses_batch_median_and_floor():
    n = 16000
    c = torch.zeros(3, n)
    c[0, :8000] = 0.2
    c[1, :8000] = 0.05
    fr = torch.as_tensor(np.stack([losses.owner_only_mask([(0, 8000)], [], n)] * 2
                                  + [losses.owner_only_mask([], [], n)]))
    r = losses.active_rms(c, fr, 1e-3)
    assert r[2].item() == pytest.approx(r[:2].median().item())
    assert losses.active_rms(torch.zeros(1, n), fr[2:], 1e-3).item() == 1.0  # 整批都没说话：除数 1


def test_crushed_speech_still_has_gradient():
    """压过 40 dB 碰到下限 / 封顶：读数仍是 −40 dB（与 Rust 一致），但梯度不为 0（直通）。"""
    rng = np.random.default_rng(0)
    c, iv = _speechlike(rng, N, 6)
    ct = torch.as_tensor(c, dtype=torch.float64)[None]
    o = (ct * 1e-4).clone().requires_grad_(True)
    fr = torch.as_tensor(losses.owner_only_mask(iv, [], N))[None]
    d, _ = losses.splvl_db(o, ct, fr)
    assert d.item() == pytest.approx(-40.0)
    d.sum().backward()
    assert o.grad.abs().sum() > 0
    o2 = (ct * 1e-4).clone().requires_grad_(True)
    e, _, _ = losses.onset_excess_db(o2, ct)
    (-e).sum().backward() if e.item() != 0 else None
    lo = losses._onset_loss(torch.tensor(1e-12, dtype=torch.float64, requires_grad=True),
                            torch.tensor(1.0, dtype=torch.float64))
    assert lo.item() == pytest.approx(40.0)
    lo.backward()


def test_skip_leading_onset_only_in_training():
    """段从说话中间开始：对拍口径（与 Rust 同）把 a = 0 记为字头；训练口径跳过它。"""
    rng = np.random.default_rng(1)
    c, _ = _speechlike(rng, N, 6)
    c[: 4000] = (rng.standard_normal(4000) * 0.1).astype(np.float32)  # 开头就在说
    on_eval, _ = losses.onset_plan(c, N)
    on_train, _ = losses.onset_plan(c, N, skip_leading=True)
    assert on_eval[0] == 0 and 0 not in on_train and len(on_train) == len(on_eval) - 1


# ---------- 模型 ----------

def _b_nonzero():
    torch.manual_seed(0)
    m = models.build("B")
    m.load_base(load_start_state())
    with torch.no_grad():
        m.attn.out.weight.normal_(std=0.3)
        m.film.fc.weight.normal_(std=0.3)
    return m


def test_frozen_bn_train_mode_equals_eval_and_has_no_lookahead():
    """bn = frozen：训练模式输出 = 评估模式输出；训练模式下也不看未来帧（窗口已满段 t0 > K）。"""
    cfg = cfgmod.from_dict({"device": "cpu"})
    m = _b_nonzero()
    x1 = torch.randn(2, 257, 80, 2) * 0.1
    x2 = x1.clone()
    x2[:, :, 50:] = torch.randn_like(x2[:, :, 50:])
    with torch.no_grad():
        m.eval()
        ye = m(x1)
        train.set_train(m, cfg)
        y1, y2 = m(x1), m(x2)
    assert (y1 - ye).abs().max().item() <= 1e-6
    assert (y1[:, :, :50] - y2[:, :, :50]).abs().max().item() <= 1e-6
    bn = next(mm for mm in m.modules() if isinstance(mm, torch.nn.BatchNorm2d))
    before = bn.running_mean.clone()
    m(x1).pow(2).mean().backward()
    assert torch.equal(bn.running_mean, before)  # 统计量不更新
    assert bn.weight.grad is not None  # 缩放照训


def test_bn_train_mode_is_a_switch():
    cfg = cfgmod.from_dict({"model": {"bn": "train"}, "device": "cpu"})
    m = _b_nonzero()
    train.set_train(m, cfg)
    assert all(mm.training for mm in m.modules() if isinstance(mm, torch.nn.BatchNorm2d))


@pytest.mark.skipif(not torch.backends.mps.is_available(), reason="本机没有 MPS；云上 CUDA 不用 MPS 绕法")
def test_b_on_mps_matches_cpu():
    """B（非零权重、打 exp018 绕法）在 MPS 上的前向与梯度和 CPU 一致（自审 P2：注意力从没在 MPS 上核过）。"""
    x = torch.randn(2, 257, 63, 2) * 0.1
    out = {}
    for dev in ("cpu", "mps"):
        m = _b_nonzero().to(dev)
        if dev == "mps":
            train.mps_patch(m)
        y = m(x.to(dev))
        y.pow(2).mean().backward()
        out[dev] = (y.detach().cpu(), m.attn.q.weight.grad.cpu())
    assert (out["cpu"][0] - out["mps"][0]).abs().max().item() <= 1e-4
    g = out["cpu"][1]
    assert (g - out["mps"][1]).abs().max().item() <= 1e-3 * max(1.0, g.abs().max().item())


# ---------- 训练循环 ----------

def _cfg(tmp_path, **over):
    d = {"run": {"out_dir": str(tmp_path), "name": "r", "seed": 2},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 6,
                  "dev_batches": 1, "calib_batches": 3},
         "schedule": {"max_steps": 4, "ckpt_every": 2, "eval_every": 3, "log_every": 1, "keep_last": 3},
         "device": "cpu"}
    for k, v in over.items():
        a, b = k.split(".")
        d.setdefault(a, {})[b] = v
    return cfgmod.from_dict(d)


def _rec(tmp_path):
    return json.loads((tmp_path / "r" / "run.json").read_text())


def test_signal_stops_and_saves(tmp_path):
    sig = train._Signals()
    sig.got = "SIGTERM"
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 100}), _signals=sig)
    assert stop["reason"] == "signal:SIGTERM"
    assert (tmp_path / "r" / "ckpt" / "last.txt").exists()


@pytest.mark.parametrize("exc,reason", [(StopIteration, "data_exhausted"),
                                        (RuntimeError("DataLoader timed out after 600 seconds"), "data_timeout")])
def test_data_end_or_timeout_stops_cleanly(tmp_path, monkeypatch, exc, reason):
    real = data.SyntheticSource.train_batches

    def gen(self, start_batch=0, device=None):
        it = real(self, start_batch, device)
        yield next(it)
        raise exc

    monkeypatch.setattr(data.SyntheticSource, "train_batches",
                        gen if exc is not StopIteration else lambda self, start_batch=0, device=None: iter(
                            [next(real(self, start_batch, device))]))
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 10}))
    assert stop["reason"] == reason and stop["step"] == 1


def test_billing_includes_setup_and_heartbeat_on_resume(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 2}))
    hb = tmp_path / "r" / "heartbeat.json"
    hb.write_text(json.dumps({"step": 3, "billed_s": 5000.0}))  # 模拟：检查点之后又跑了一段才被抢占
    stop = train.train(_cfg(tmp_path))
    rec = _rec(tmp_path)
    assert rec["resumes"][0]["billed_s_before"] == 5000.0 and stop["billed_s"] > 5000.0
    assert rec["resumes"][0]["prev_stop"]["reason"] == "max_steps"


def test_corrupt_latest_checkpoint_falls_back(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 4}))
    ck = tmp_path / "r" / "ckpt"
    (ck / "step-0000004.pt").write_bytes(b"truncated")
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 6}))
    assert _rec(tmp_path)["resumes"][0]["from_step"] == 2 and stop["step"] == 6
    assert "step-0000004.pt" in (ck / "bad_ckpt.txt").read_text()


def test_resume_refused_after_nonfinite(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 2}))
    rec = _rec(tmp_path)
    rec["stop"] = {"reason": "nonfinite", "step": 2}
    (tmp_path / "r" / "run.json").write_text(json.dumps(rec))
    with pytest.raises(RuntimeError, match="训崩"):
        train.train(_cfg(tmp_path))


def test_calibration_uses_calib_batches_and_final_eval_between_evals(tmp_path):
    stop = train.train(_cfg(tmp_path))  # 4 步，每 3 步评估：第 3 步 + 停下时补第 4 步
    rec = _rec(tmp_path)
    assert rec["calib"]["batches"] == 3 and stop["step"] == 4
    rows = [json.loads(x) for x in (tmp_path / "r" / "metrics.jsonl").read_text().splitlines()]
    assert [r["step"] for r in rows if "dev" in r] == [3, 4]
    assert rec["env"]["torch"] and "gpu" in rec["env"]


def test_loader_requires_batch_index_and_build_id(tmp_path):
    with pytest.raises(KeyError, match="batch_index"):
        data.normalize({"noisy": [[0.0]], "clean": [[0.0]], "replay": [False], "near_silent": [False],
                        "owner_ivs": [[]], "interf_ivs": [[]]}, require_index=True)

    class NoId:
        build_id = None

        def __init__(self, *a, **k):
            pass

    mod = type("M", (), {"DevSet": NoId})
    cfg = _cfg(tmp_path, **{"data.kind": "loader", "data.shard_root": "s", "data.recipe": "r"})
    with pytest.raises(ValueError, match="build_id"):
        data.LoaderSource(cfg, mod=mod)
