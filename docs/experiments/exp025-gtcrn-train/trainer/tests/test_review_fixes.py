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
    eo = torch.tensor(1e-12, dtype=torch.float64, requires_grad=True)
    lo = losses._onset_loss(eo, torch.tensor(1.0, dtype=torch.float64))
    assert lo.item() == pytest.approx(40.0)  # 字头损失碰到下限与 40 dB 封顶
    lo.backward()
    assert eo.grad.item() != 0


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
    g, gm = out["cpu"][1], out["mps"][1]
    assert g.norm() > 0
    rel = ((g - gm).norm() / g.norm()).item()  # 范数相对差（codex 第 10 轮 #2：不再以 1 为下限）
    assert rel <= 1e-3, rel
    assert ((g - gm * 1.01).norm() / g.norm()).item() > 1e-3  # 反向对照：差 1% 就红


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


def _signal_at(monkeypatch, sig, at_batch):
    """取第 at_batch 批时「收到」SIGTERM（模拟训练中途被抢占）。"""
    real = data.SyntheticSource.train_batches

    def gen(self, start_batch=0, device=None):
        for b in real(self, start_batch, device):
            if b["batch_index"] == at_batch:
                sig.got = "SIGTERM"
            yield b

    monkeypatch.setattr(data.SyntheticSource, "train_batches", gen)


def test_signal_mid_training_saves(tmp_path, monkeypatch):
    sig = train._Signals()
    _signal_at(monkeypatch, sig, 2)
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 100}), _signals=sig)
    assert stop["reason"] == "signal:SIGTERM" and stop["step"] == 3
    st, _ = train.RunDir(tmp_path / "r").load_latest("cpu")
    assert st["step"] == 3


def test_signal_during_setup_keeps_only_heartbeat(tmp_path):
    """准备期（标定前）就收到信号：不建 run.json、不存检查点，但心跳记下了准备期的账；下次开跑接上它。"""
    sig = train._Signals()
    sig.got = "SIGTERM"
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 100}), _signals=sig)
    assert stop["reason"] == "signal:SIGTERM" and stop["phase"] == "setup"
    assert not (tmp_path / "r" / "run.json").exists()
    hb = json.loads((tmp_path / "r" / "heartbeat.json").read_text())
    assert hb["billed_s"] > 0
    stop2 = train.train(_cfg(tmp_path, **{"schedule.max_steps": 2}))
    assert stop2["reason"] == "max_steps" and stop2["billed_s"] > hb["billed_s"]
    assert _rec(tmp_path)["billed_s_before"] == pytest.approx(hb["billed_s"], abs=0.01)


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
    cfg = _cfg(tmp_path, **{"data.kind": "loader", "data.shard_root": "s", "data.recipe": "r",
                            "data.num_workers": 1})
    with pytest.raises(ValueError, match="build_id"):
        data.LoaderSource(cfg, mod=mod)


# ---------- codex 第 1 轮 ----------

def test_nonfinite_gradient_does_not_update(tmp_path, monkeypatch):
    """损失有限、梯度为 NaN：不更新权重，按训崩停下，存下的权重全是有限数（codex 第 1 轮 #2）。"""
    real = torch.nn.utils.clip_grad_norm_
    calls = {"n": 0}

    def bad(params, *a, **k):
        calls["n"] += 1
        gn = real(params, *a, **k)
        return gn * float("nan") if calls["n"] >= 2 else gn

    monkeypatch.setattr(torch.nn.utils, "clip_grad_norm_", bad)
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 10, "schedule.ckpt_every": 1}))
    assert stop["reason"] == "nonfinite" and stop["step"] == 1
    assert _rec(tmp_path)["crash"]["grad_norm"] != _rec(tmp_path)["crash"]["grad_norm"]  # NaN
    st, _ = train.RunDir(tmp_path / "r").load_latest("cpu")
    assert all(torch.isfinite(v).all() for v in st["model"].values() if v.dtype.is_floating_point)


def test_all_checkpoints_corrupt_is_an_error_not_a_fresh_run(tmp_path):
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 2}))
    for f in (tmp_path / "r" / "ckpt").glob("step-*.pt"):
        f.write_bytes(b"x")
    before = (tmp_path / "r" / "run.json").read_text()
    with pytest.raises(RuntimeError, match="没有可读的检查点"):
        train.train(_cfg(tmp_path))
    assert (tmp_path / "r" / "run.json").read_text() == before  # 原记录没被覆盖


def test_usd_accumulates_per_segment_price(tmp_path):
    """续训时改单价只作用于新的一段，不追溯（codex 第 1 轮 #6）。"""
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 2, "limits.usd_per_hour": 1000.0}))
    usd1 = json.loads((tmp_path / "r" / "heartbeat.json").read_text())["usd"]  # 第一段最后落盘的账
    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 4, "limits.usd_per_hour": 0.0}))
    assert usd1 > 0 and stop["usd"] == pytest.approx(usd1, abs=1e-4)  # 第二段单价 0：不追溯、不再涨


def test_heartbeat_thread_writes_while_data_stalls(tmp_path, monkeypatch):
    """取数卡住时心跳线程照写（不靠训练步），被杀前的计费不丢（codex 第 1 轮 #4）。"""
    import time as _t
    real = data.SyntheticSource.train_batches
    seen = []

    def slow(self, start_batch=0, device=None):
        for b in real(self, start_batch, device):
            _t.sleep(0.3)
            hb = tmp_path / "r" / "heartbeat.json"
            if hb.exists():
                seen.append(json.loads(hb.read_text())["billed_s"])
            yield b

    monkeypatch.setattr(data.SyntheticSource, "train_batches", slow)
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 3, "schedule.log_every": 1000}), _beat_s=0.05)
    assert len(seen) >= 2 and seen[-1] > seen[0]


def test_signal_saves_without_extra_eval(tmp_path, monkeypatch):
    sig = train._Signals()
    _signal_at(monkeypatch, sig, 2)  # 第 3 步是评估步（eval_every = 3）：收到信号后不评估
    train.train(_cfg(tmp_path, **{"schedule.max_steps": 100}), _signals=sig)
    rows = [json.loads(x) for x in (tmp_path / "r" / "metrics.jsonl").read_text().splitlines()] \
        if (tmp_path / "r" / "metrics.jsonl").exists() else []
    assert not any("dev" in r for r in rows)  # 被抢占时先存盘，不再花时间评估（codex 第 1 轮 #3）


# ---------- codex 第 2 轮 ----------

def test_calibrate_weight_positive_even_if_hybrid_negative():
    """HybridLoss 均值为负或为 0 时，标定出的权重仍是有限正数（codex 第 2 轮 #4）。"""
    assert losses.calibrate_weight(-3.0, 1.5) == pytest.approx(0.2)
    assert losses.calibrate_weight(0.0, 0.0) == pytest.approx(0.1)
    with pytest.raises(ValueError):
        losses.calibrate_weight(float("nan"), 1.0)


def test_onset_default_matches_rust_rule(tmp_path):
    """默认训练口径与 V-ONSET 正本同（不跳过 a = 0）；跳过只是开关（codex 第 2 轮 #3）。"""
    assert losses.LossCfg().on_skip_leading is False
    assert cfgmod.Config().loss.on_skip_leading is False
    assert train.loss_cfg(cfgmod.from_dict({"loss": {"on_skip_leading": True}}), 1, 1).on_skip_leading


# ---------- codex 第 3 轮 ----------

def test_signal_on_last_step_skips_final_eval(tmp_path, monkeypatch):
    """最后一步收到信号：停下原因是信号（优先于「步数跑满」），不补评估（codex 第 3 轮 #1）。"""
    sig = train._Signals()
    _signal_at(monkeypatch, sig, 3)  # max_steps = 4，取第 4 批（下标 3）时收到
    stop = train.train(_cfg(tmp_path, **{"schedule.eval_every": 100}), _signals=sig)
    assert stop["reason"] == "signal:SIGTERM" and stop["step"] == 4
    rows = [json.loads(x) for x in (tmp_path / "r" / "metrics.jsonl").read_text().splitlines()]
    assert not any("dev" in r for r in rows)


def test_crash_between_record_and_step0_can_rebuild(tmp_path, monkeypatch):
    """写了 run.json（phase = init）、第 0 步没存下就被杀：重启时按同配置重建，不被永久拒绝（codex 第 3 轮 #2）。"""
    real = train.RunDir.save

    def die(self, state, step, keep):
        raise KeyboardInterrupt("模拟被杀")

    monkeypatch.setattr(train.RunDir, "save", die)
    with pytest.raises(KeyboardInterrupt):
        train.train(_cfg(tmp_path))
    assert _rec(tmp_path)["phase"] == "init"
    monkeypatch.setattr(train.RunDir, "save", real)
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "max_steps" and _rec(tmp_path)["phase"] == "training"
    with pytest.raises(RuntimeError, match="data.batch"):  # 配置变了仍拒绝
        rec = _rec(tmp_path)
        rec["phase"] = "init"
        (tmp_path / "r" / "run.json").write_text(json.dumps(rec))
        for f in (tmp_path / "r" / "ckpt").glob("step-*.pt"):
            f.unlink()
        train.train(_cfg(tmp_path, **{"data.batch": 3}))


def test_evaluate_weights_protection_terms_by_valid_segments(monkeypatch):
    """一批 lvl = 4（2 段有效）、一批整批无效：开发读数是 4，不是 2（codex 第 3 轮 #3）。"""
    seq = iter([({"hybrid": 1.0, "lvl": 4.0, "lvl_valid": 2, "total": 5.0}),
                ({"hybrid": 3.0, "lvl": 0.0, "lvl_valid": 0, "total": 3.0})])
    monkeypatch.setattr(train, "forward_loss", lambda *a, **k: (None, next(seq)))
    m = torch.nn.Linear(1, 1)
    out = train.evaluate(m, None, [0, 1], "cpu", cfgmod.from_dict({"device": "cpu"}))
    assert out["lvl"] == 4.0 and out["hybrid"] == 2.0 and out["lvl_valid"] == 2


def test_export_compare_rejects_nonfinite(tmp_path, monkeypatch):
    """输出里有 inf 时不能因为 inf ≤ inf 判过（codex 第 3 轮 #4）。"""
    import export
    m, _ = export.model_from_ckpt(None)
    out = tmp_path / "a.onnx"
    export.export(m, out)
    real_fwd = type(m).forward

    def inf_fwd(self, spec, cond=None):
        y = real_fwd(self, spec, cond)
        return y + float("inf")

    monkeypatch.setattr(type(m), "forward", inf_fwd)
    r = export.compare(m, out)
    assert not r["pass"] and not r["cases"][0]["finite"]


# ---------- codex 第 4 轮 ----------

def test_same_run_cannot_start_twice(tmp_path):
    """同名运行已有进程占着目录：第二个直接拒绝，不读不写任何状态（codex 第 4 轮 #1）。"""
    rd = train.RunDir(tmp_path / "r")
    rd.lock()
    try:
        with pytest.raises(RuntimeError, match="另一个训练进程"):
            train.train(_cfg(tmp_path))
        assert not (tmp_path / "r" / "run.json").exists()
    finally:
        rd.unlock()
    assert train.train(_cfg(tmp_path, **{"schedule.max_steps": 1}))["reason"] == "max_steps"  # 释放后可跑


def test_known_dev_nan_and_missing_fail(tmp_path, monkeypatch):
    import smoke
    cfg = cfgmod.from_dict({"run": {"out_dir": str(tmp_path)},
                            "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "dev_n": 2, "dev_batches": 1},
                            "device": "cpu"})
    dev = data.make_source(cfg).dev_batches()
    monkeypatch.setattr(train, "evaluate", lambda *a, **k: {"hybrid": float("nan")})
    assert not smoke.check_known_dev(cfg, dev, torch.device("cpu"), {"hybrid": 1.0}, 1e-3)["pass"]
    assert not smoke.check_known_dev(cfg, dev, torch.device("cpu"), {"other": 1.0}, 1e-3)["pass"]
    assert not smoke.check_known_dev(cfg, dev, torch.device("cpu"), {}, 1e-3)["pass"]  # 空基准不算通过


# ---------- codex 第 6 轮 ----------

def test_watchdog_fires_only_when_stop_wanted_and_stalled(tmp_path):
    rd = train.RunDir(tmp_path / "w")
    fired = []
    t0 = __import__("time").monotonic()
    # 第二条原为 t0 − 999：guards ⑤ 加了「久无进展 900 s 就强退」后改成 200 s（> hang_s、< STALL_S）
    for want, last, expect in (("signal:SIGTERM", t0 - 999, True), (None, t0 - 200, False),
                               ("max_usd", lambda: __import__("time").monotonic(), False)):
        fired.clear()
        lp = last if callable(last) else (lambda v=last: v)
        b = train._Beat(rd, lambda: (0, 1.0, 0.0), 0.01, want_stop=lambda w=want: w, last_progress=lp,
                        hang_s=5.0, on_hang=fired.append)
        with b:
            __import__("time").sleep(0.1)
        assert bool(fired) == expect, (want, expect)


def test_stalled_fetch_after_signal_triggers_watchdog(tmp_path, monkeypatch):
    """worker = 0、取数卡住时收到信号：看门狗在 hang_s 后出手（codex 第 6 轮 #1）。测试里出手 = 放开卡住的取数。"""
    import threading as th
    release = th.Event()
    sig = train._Signals()
    real = data.SyntheticSource.train_batches

    def stuck(self, start_batch=0, device=None):
        for b in real(self, start_batch, device):
            if b["batch_index"] == 1:
                sig.got = "SIGTERM"
                release.wait(10)  # 模拟一次同步 I/O 卡住
            yield b

    monkeypatch.setattr(data.SyntheticSource, "train_batches", stuck)
    hung = []

    def on_hang(why):
        hung.append(why)
        release.set()

    stop = train.train(_cfg(tmp_path, **{"schedule.max_steps": 50, "limits.hang_s": 0.2}), _signals=sig,
                       _beat_s=0.05, _on_hang=on_hang)
    assert hung and "SIGTERM" in hung[0] and stop["reason"] == "signal:SIGTERM"


def test_loader_rejects_zero_workers(tmp_path):
    cfg = _cfg(tmp_path, **{"data.kind": "loader", "data.shard_root": "s", "data.recipe": "r"})
    with pytest.raises(ValueError, match="num_workers"):
        data.LoaderSource(cfg, mod=type("M", (), {}))


def test_calibration_gets_all_requested_batches_beyond_dev_n(tmp_path):
    """开发份只有 1 批（dev_n = 2，批量 2），标定仍用满配置的 5 批（codex 第 6 轮 #3）。"""
    train.train(_cfg(tmp_path, **{"data.dev_n": 2, "data.calib_batches": 5, "schedule.max_steps": 1}))
    assert _rec(tmp_path)["calib"]["batches"] == 5
    src = data.make_source(_cfg(tmp_path))
    assert torch.equal(src.calib_batches(2)[0]["noisy"], src.dev_batches()[0]["noisy"])  # 前几批与开发份相同


# ---------- codex 第 7 轮 ----------

@pytest.mark.parametrize("kind", ["raises", "blocks"])
def test_watchdog_fires_even_if_heartbeat_write_fails(tmp_path, kind):
    """心跳写盘先报错 / 先卡住，**之后**才满足停止条件：看门狗照样按时出手（codex 第 7 轮 #1、第 8 轮 #3）。"""
    import threading as th
    import time as _t
    gate, entered = th.Event(), th.Event()
    rd = train.RunDir(tmp_path / "w")

    def bad(*a, **k):
        entered.set()
        if kind == "raises":
            raise OSError("disk gone")
        gate.wait(10)  # 卡住，直到测试结束才放开

    rd.beat = bad
    want = {"v": None}
    fired = []
    b = train._Beat(rd, lambda: (0, 1.0, 0.0), 0.02, want_stop=lambda: want["v"],
                    last_progress=lambda: _t.monotonic() - 200, hang_s=1.0, on_hang=fired.append)
    # 原为 − 999：超过 guards ⑤ 的 STALL_S 900 s 会在写盘前就出手；改成 200 s（> hang_s、< STALL_S）
    with b:
        assert entered.wait(2)  # 先确认写盘已进入失败 / 卡住
        want["v"] = "max_usd"  # 再满足停止条件
        t0 = _t.monotonic()
        while not fired and _t.monotonic() - t0 < 3:
            _t.sleep(0.01)
        assert fired and b.write_failures >= 1  # 写盘仍卡着 / 仍失败时就出手了
        gate.set()


def test_heartbeat_single_writer_and_no_late_commit(tmp_path):
    """写卡住时不叠第二个写入；运行结束后迟到的那次写入不提交，心跳不倒退、文件完整（codex 第 8 轮 #2）。"""
    import threading as th
    import time as _t
    rd = train.RunDir(tmp_path / "w")
    rd.beat(5, 500.0, 5.0)  # 已有的新状态
    gate = th.Event()
    real = rd.beat
    calls = []

    def slow(*a, **k):
        calls.append(a)
        gate.wait(5)
        real(*a, **k)

    rd.beat = slow
    b = train._Beat(rd, lambda: (1, 1.0, 0.1), 0.02)  # 旧快照（数值更小）
    with b:
        _t.sleep(0.2)
    gate.set()
    _t.sleep(0.2)
    assert len(calls) == 1  # 只有一个在途写入
    hb = json.loads((tmp_path / "w" / "heartbeat.json").read_text())
    assert hb == {"step": 5, "billed_s": 500.0, "usd": 5.0}  # 迟到的旧快照没覆盖
    assert not list((tmp_path / "w").glob("*.tmp"))


def test_deadline_watchdog_needs_no_disk(tmp_path):
    fired = []
    with train._Deadline(0.05, on_hang=fired.append):
        __import__("time").sleep(0.2)
    assert fired and "跑满" in fired[0]
    fired.clear()
    with train._Deadline(5.0, on_hang=fired.append):
        pass
    assert not fired


def test_hard_exit_exits_even_if_record_write_blocks(tmp_path, monkeypatch):
    """hung.json 写不进去（卡住）：最多等 2 s 仍强退。"""
    import threading as th
    import time as _t
    rd = train.RunDir(tmp_path / "w")
    gate = th.Event()
    monkeypatch.setattr(type(rd.root), "write_text", lambda self, *a, **k: gate.wait(10))
    exited = []
    t0 = _t.monotonic()
    train._hard_exit(rd, "测试", _exit=exited.append)
    gate.set()
    assert exited == [75] and _t.monotonic() - t0 < 3



# ---------- codex 第 9 轮 ----------

def test_deadline_uses_budget_when_it_is_tighter():
    lim = cfgmod.from_dict({"limits": {"max_hours": 6.0, "max_usd": 0.01, "usd_per_hour": 0.5,
                                       "hang_s": 10.0}}).limits
    assert train.deadline_s(lim) == pytest.approx(0.01 / 0.5 * 3600 + 10.0)  # 72 s + 10 s，不是 6 h
    lim = cfgmod.from_dict({"limits": {"max_hours": 1.0, "max_usd": 100.0, "usd_per_hour": 0.0}}).limits
    assert train.deadline_s(lim) == pytest.approx(3600 + lim.hang_s)
