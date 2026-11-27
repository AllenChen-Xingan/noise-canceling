"""训练记录（#236）：梯度裁剪统计 · 开发评估存音频。

全部用合成小样、CPU、每次几步（秒级）。
"""
import json
from pathlib import Path

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
    assert out["clip_frac"] == 0.5  # 3.0 不算被裁：判据是严格大于阈值
    assert out["gn_pre_median"] == 4.0
    assert out["gn_pre_max"] == 100.0
    assert cs.flush() == {}  # 清过之后没有读数：什么都不多写


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
