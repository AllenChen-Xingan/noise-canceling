"""训练记录（#236）：梯度裁剪统计 · 开发评估存音频。

全部用合成小样、CPU、每次几步（秒级）。
"""
import json
from pathlib import Path

import pytest
import soundfile as sf
import torch

import config as cfgmod
import train
import trainlog


def _cfg(tmp_path, **over):
    """与 tests/test_train.py 的合成配置同一套（几步就能跑完）。"""
    d = {"run": {"out_dir": str(tmp_path), "name": "t", "seed": 3},
         "model": {"variant": "B"},
         "data": {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 4,
                  "dev_batches": 2, "calib_batches": 2},
         "optim": {"lr": 1e-3, "warmup_steps": 2},
         "schedule": {"max_steps": 6, "ckpt_every": 2, "eval_every": 3, "log_every": 1, "keep_last": 2},
         "limits": {"max_hours": 1.0, "max_usd": 5.0},
         "device": "cpu"}
    for k, v in over.items():
        a, b = k.split(".")
        d.setdefault(a, {})[b] = v
    return cfgmod.from_dict(d)


def _rows(root: Path) -> list[dict]:
    return [json.loads(x) for x in (root / "metrics.jsonl").read_text().splitlines()]


# ---------- 裁剪统计：拿构造的范数直接喂 ----------

def test_clip_stats_window_counts_median_max_and_reset():
    cs = trainlog.ClipStats(3.0)
    for g in (1.0, 3.0, 5.0, 100.0):
        cs.add(g)
    out = cs.flush()
    assert out["clip_n"] == 4  # 窗口里的步数
    assert out["clip_frac"] == 0.75  # 3.0 也算被裁：分母多 1e-6，torch 的缩放系数略小于 1（见下面的对照测试）
    assert out["gn_pre_median"] == 4.0
    assert out["gn_pre_max"] == 100.0
    assert cs.flush() == {}  # 清过之后没有读数：什么都不多写


def _clip_probe(target: float, max_norm: float = 3.0) -> tuple[float, bool]:
    """造一个总范数 ≈ target 的梯度，照 torch 的做法裁一次：回（裁前范数, 梯度变没变）。"""
    w = torch.nn.Parameter(torch.zeros(4))
    w.grad = torch.full((4,), target / 2.0)  # 4 个分量各 target/2 → 总范数 = target
    before = w.grad.clone()
    gn = float(torch.nn.utils.clip_grad_norm_([w], max_norm))
    return gn, not torch.equal(before, w.grad)


@pytest.mark.parametrize("target,clipped", [(3.0 - 1e-5, False), (3.0 - 1e-6, False), (3.0, True), (3.0 + 1e-6, True)])
def test_clip_stats_clipped_is_what_torch_actually_does(target, clipped):
    """「被裁」= torch 真的改了梯度：判据是 `max_norm / (gn + 1e-6) < 1`，float32 算（与 clip_grad_norm_ 同一个数）。

    3.0 这一步也算被裁（分母多 1e-6）。3.0 − 1e-6 那一步 torch 的系数在 float32 里恰好等于 1，用 float64 算会
    误判成被裁——这一条把两种写法区分开。
    """
    gn, changed = _clip_probe(target)
    cs = trainlog.ClipStats(3.0)
    cs.add(gn)
    out = cs.flush()
    assert changed is clipped  # 先钉住 torch 的实际行为，再比统计
    assert (out["clip_frac"] == 1.0) is clipped


# ---------- 接进训练循环 ----------

def test_train_rows_have_clip_stats_and_cover_every_step(tmp_path):
    stop = train.train(_cfg(tmp_path))
    rows = _rows(tmp_path / "t")
    train_rows = [r for r in rows if "lr" in r]
    assert train_rows
    keys = {"clip_n", "clip_frac", "gn_pre_median", "gn_pre_max"}
    assert all(keys <= set(r) for r in train_rows)
    assert all("grad_norm" in r for r in train_rows)  # 原有的范数照旧
    assert sum(r["clip_n"] for r in train_rows) == stop["step"]  # 每一步都算进过某个窗口
    assert all(0.0 <= r["clip_frac"] <= 1.0 for r in train_rows)
    assert all(r["gn_pre_max"] >= r["gn_pre_median"] for r in train_rows)


# ---------- 开发评估存音频 ----------

def test_dev_audio_files_written_per_eval(tmp_path):
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "max_steps"
    root = tmp_path / "t"
    step_dirs = sorted(p for p in (root / "dev_audio").glob("step-*") if p.is_dir())
    assert len(step_dirs) == sum("dev" in r for r in _rows(root))  # 每次开发评估一份
    n = 16000  # 合成源的段长 = seg_s 1.0 × 16 kHz
    for d in step_dirs:
        files = sorted(d.glob("enh-*.wav"))
        assert len(files) == trainlog.DEV_AUDIO_N
        for f in files:
            info = sf.info(f)
            assert info.samplerate == 16000 and info.subtype == "PCM_16"
            assert info.frames == n and info.channels == 1
    ref = sorted((root / "dev_audio" / "ref").glob("*.wav"))
    assert len(ref) == 2 * trainlog.DEV_AUDIO_N  # 带噪 + 干净，各 DEV_AUDIO_N 条，只写一次
    assert sum(p.name.startswith("noisy-") for p in ref) == trainlog.DEV_AUDIO_N
    assert sum(p.name.startswith("clean-") for p in ref) == trainlog.DEV_AUDIO_N


def test_save_dev_audio_keeps_module_modes_and_rng(tmp_path):
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    train.set_train(model, cfg)  # bn = frozen：批归一化层留在评估模式
    dev = train.make_source(cfg).dev_batches()
    before = {id(m): m.training for m in model.modules()}
    assert not all(before.values())  # 冻结的批归一化确实与训练层状态不同，这个断言才有区分力
    rng = torch.random.get_rng_state().clone()
    assert trainlog.save_dev_audio(model, dev, device, tmp_path, 7) is None
    assert {id(m): m.training for m in model.modules()} == before  # 每个模块原样还回去
    assert torch.equal(torch.random.get_rng_state(), rng)  # 不碰随机数
    assert (tmp_path / "dev_audio" / "step-0000007").is_dir()


def test_save_dev_audio_completes_partial_ref_and_keeps_existing(tmp_path):
    """参照写到一半失败（缺一份）：下一次评估把缺的补上，已经在的那几份不重写。

    判据按「缺文件」而不是「目录在不在」：一次 OSError 之后参照集不能永远残缺。
    """
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    dev = train.make_source(cfg).dev_batches()
    assert trainlog.save_dev_audio(model, dev, device, tmp_path, 1) is None
    ref = tmp_path / "dev_audio" / "ref"
    gone = ref / "clean-1.wav"
    assert gone.exists()
    gone.unlink()
    keep = sorted(p for p in ref.glob("*.wav"))
    assert len(keep) == 2 * trainlog.DEV_AUDIO_N - 1
    before = {p.name: p.stat().st_mtime_ns for p in keep}
    assert trainlog.save_dev_audio(model, dev, device, tmp_path, 2) is None
    assert gone.exists()  # 缺的补齐
    assert {p.name: p.stat().st_mtime_ns for p in keep} == before  # 已有的一律不重写
    assert len(list(ref.glob("*.wav"))) == 2 * trainlog.DEV_AUDIO_N


def test_save_dev_audio_reports_oserror_without_raising(tmp_path, monkeypatch):
    """磁盘满 / 目录不可写时只回一个错误字符串，训练不崩。"""
    def boom(*a, **k):
        raise OSError("磁盘满")
    monkeypatch.setattr(trainlog.sf, "write", boom)
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    dev = train.make_source(cfg).dev_batches()
    err = trainlog.save_dev_audio(model, dev, device, tmp_path, 1)
    assert isinstance(err, str) and "磁盘满" in err


# 写 wav 真的会抛的两种错：libsndfile 自己的错不是 OSError（`SoundFileError` → `RuntimeError`）
@pytest.mark.parametrize("make_exc", [lambda: sf.LibsndfileError(0, "磁盘满"),
                                      lambda: sf.SoundFileRuntimeError("磁盘满")],
                         ids=["LibsndfileError", "SoundFileRuntimeError"])
def test_save_dev_audio_reports_soundfile_error_without_raising(tmp_path, monkeypatch, make_exc):
    """soundfile 报错（`LibsndfileError` 是 RuntimeError 的子类、不是 OSError）一样只回错误字符串，不抛。"""
    def boom(*a, **k):
        raise make_exc()
    monkeypatch.setattr(sf, "write", boom)
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    dev = train.make_source(cfg).dev_batches()
    err = trainlog.save_dev_audio(model, dev, device, tmp_path, 1)
    assert isinstance(err, str) and "磁盘满" in err


def test_save_dev_audio_survives_soundfile_error_with_broken_message(tmp_path, monkeypatch):
    """最坏情况：连 `str(错)` 自己都抛（`LibsndfileError` 的 __str__ 要回头问 libsndfile）。错误路径不许再抛一次。"""
    def boom(*a, **k):
        raise sf.LibsndfileError("不是整数", "")  # code 不是整数 → __str__ 抛 TypeError
    monkeypatch.setattr(sf, "write", boom)
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    dev = train.make_source(cfg).dev_batches()
    err = trainlog.save_dev_audio(model, dev, device, tmp_path, 1)
    assert isinstance(err, str) and "LibsndfileError" in err


def test_training_survives_audio_write_errors(tmp_path, monkeypatch):
    """写 wav 一律失败，训练照跑满 max_steps，只在记录里留 dev_audio_error 行（codex P1）。"""
    def boom(*a, **k):
        raise sf.LibsndfileError(0, "磁盘满")
    monkeypatch.setattr(sf, "write", boom)
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "max_steps" and stop["step"] == 6
    rows = _rows(tmp_path / "t")
    assert any("dev_audio_error" in r for r in rows)
    assert sum("dev" in r for r in rows) == 2  # 两次开发评估都照常记了读数


# ---------- 存档音频要认停下的信号 ----------

def test_save_dev_audio_stops_at_once_when_asked(tmp_path):
    """一开始就要求停：一份 wav 都不写，也不当错误（返回 None、不记 dev_audio_error）。"""
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    dev = train.make_source(cfg).dev_batches()
    assert trainlog.save_dev_audio(model, dev, device, tmp_path, 3, should_stop=lambda: True) is None
    assert not list(tmp_path.rglob("enh-*.wav"))


def test_save_dev_audio_stops_between_files(tmp_path, monkeypatch):
    """停下的信号写到一半才到：写下一份之前先问，已经写完的留着、其余的都不写（每份之前问一次）。"""
    written = []
    real = trainlog._write_wav

    def spy(path, wav):
        real(path, wav)
        written.append(path.name)

    monkeypatch.setattr(trainlog, "_write_wav", spy)
    cfg = _cfg(tmp_path)
    device = torch.device("cpu")
    model = train.build_model(cfg, device)
    dev = train.make_source(cfg).dev_batches()
    err = trainlog.save_dev_audio(model, dev, device, tmp_path, 1,
                                  should_stop=lambda: any(n.startswith("enh-") for n in written))
    assert err is None  # 不是错误，就是停下了
    assert sum(n.startswith("enh-") for n in written) == 1  # 只写完第一份就停（按「每批之前」判会写成两份）
    assert not (tmp_path / "dev_audio" / "step-0000001" / "enh-1.wav").exists()


def test_best_pt_is_saved_before_dev_audio(tmp_path, monkeypatch):
    """存档音频排在这一步的最后：写 wav 慢（网络盘 / 磁盘满）时不拖住最好的那份权重。"""
    seen = []
    real = trainlog.save_dev_audio

    def spy(model, dev, device, root, step, should_stop=None):
        seen.append({"step": step, "best_exists": (Path(root) / "ckpt" / "best.pt").exists(),
                     "has_stop": should_stop is not None})
        return real(model, dev, device, root, step, should_stop=should_stop)

    monkeypatch.setattr(trainlog, "save_dev_audio", spy)
    stop = train.train(_cfg(tmp_path))
    assert stop["reason"] == "max_steps"
    assert [s["step"] for s in seen] == [3, 6]  # eval_every = 3
    assert all(s["best_exists"] for s in seen)  # 第一次评估就已经把 best.pt 存下来了
    assert all(s["has_stop"] for s in seen)  # 存档也认停下的信号


# ---------- 停下时把剩下的裁剪窗口写掉 ----------

def test_clip_window_flushed_at_stop_with_tail_row(tmp_path):
    """最后几步不满一个记录窗口：照样记一行（clip_tail），每一步的范数都进过某个窗口。"""
    stop = train.train(_cfg(tmp_path, **{"schedule.log_every": 4, "schedule.max_steps": 6}))
    assert stop["step"] == 6
    rows = _rows(tmp_path / "t")
    with_clip = [r for r in rows if "clip_n" in r]
    assert sum(r["clip_n"] for r in with_clip) == stop["step"]  # 6 步一个不落
    assert with_clip[-1]["clip_tail"] is True
    assert not any(r.get("clip_tail") for r in with_clip[:-1])  # 只有收尾那一行带标记
