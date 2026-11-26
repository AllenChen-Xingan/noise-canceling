"""数据适配：合成源可复现 · 与 traindata loader 的接口契约（假模块）· 批次错位即停 · 真 loader 对接（合入后跑）。"""
import json
import types
from pathlib import Path

import numpy as np
import pytest
import torch

import config as cfgmod
import data
import train


def _cfg(tmp_path, **data_over):
    d = {"kind": "synthetic", "seg_s": 1.0, "batch": 2, "num_workers": 0, "dev_n": 2, "dev_batches": 1,
         "calib_batches": 1, **data_over}
    return cfgmod.from_dict({"run": {"out_dir": str(tmp_path), "name": "d", "seed": 1}, "data": d,
                             "schedule": {"max_steps": 3, "ckpt_every": 3, "eval_every": 99, "log_every": 1},
                             "device": "cpu"})


def test_synthetic_batch_k_depends_only_on_seed_and_k(tmp_path):
    src = data.make_source(_cfg(tmp_path))
    it = src.train_batches(0)
    b0, b1, b2 = next(it), next(it), next(it)
    again = next(src.train_batches(2))
    assert torch.equal(b2["noisy"], again["noisy"]) and again["batch_index"] == 2
    assert not torch.equal(b0["noisy"], b1["noisy"])
    assert all(len(iv) > 0 for iv in b0["owner_ivs"])


def _example(i, n):
    rng = np.random.default_rng(i)
    c = (rng.standard_normal(n) * 0.05).astype(np.float32)
    return {"noisy": c + 0.01, "clean": c,
            "meta": {"replay": i % 3 == 0, "near_silent": False, "owner_ivs": [[0, n]], "interf_ivs": []}}


class _FakeDev:
    build_id = "fake-build-1"

    def __init__(self, *a, n=4, **k):
        self.size, self.len = n, 8000

    def __len__(self):
        return self.size

    def __getitem__(self, i):
        return _example(10_000 + i, self.len)


class _FakeIt(torch.utils.data.IterableDataset):
    """模块级（DataLoader 子进程要能序列化它，和真 loader 的 TorchIterable 一样）。"""
    build_id = "fake-build-1"

    def __init__(self, batch_size, start_batch, render, n=8000):
        self.bs, self.k0, self.render, self.n = batch_size, start_batch, render, n

    def __iter__(self):
        w = torch.utils.data.get_worker_info()
        wid, nw = (w.id, w.num_workers) if w else (0, 1)
        k = self.k0 + wid
        while True:
            exs = [_example(k * self.bs + j, self.n) for j in range(self.bs)]
            yield {"noisy": np.stack([e["noisy"] for e in exs]), "clean": np.stack([e["clean"] for e in exs]),
                   "replay": np.array([e["meta"]["replay"] for e in exs]),
                   "near_silent": np.array([False] * self.bs),
                   "owner_ivs": [e["meta"]["owner_ivs"] for e in exs], "interf_ivs": [[] for _ in exs],
                   "meta": [e["meta"] for e in exs], "batch_index": k}
            k += nw


def _fake_loader(n, batch_size_seen):
    """按 traindata 2026-11-24 接口（README 第四节）写的假 loader：CPU 路出整批 dict + batch_index。"""
    def torch_iterable(shard_root, recipe_path, split="train", seed=0, seg_s=4.0, batch_size=32,
                       start_batch=0, render="gpu", **kw):
        batch_size_seen.append((batch_size, start_batch, render))
        return _FakeIt(batch_size, start_batch, render, n)

    return types.SimpleNamespace(DevSet=_FakeDev, torch_iterable=torch_iterable,
                                 render_on_gpu=lambda b, d: pytest.fail("CPU 上不该调 render_on_gpu"))


def test_loader_contract_cpu_path(tmp_path):
    cfg = _cfg(tmp_path, kind="loader", shard_root="s", recipe="r.json", num_workers=1)
    seen = []
    src = data.LoaderSource(cfg, mod=_fake_loader(8000, seen))
    assert src.build_id == "fake-build-1"
    b = next(src.train_batches(5, device=torch.device("cpu")))
    assert seen == [(2, 5, "cpu")]
    assert b["batch_index"] == 5 and b["noisy"].dtype == torch.float32 and b["replay"].dtype == torch.bool
    assert b["owner_ivs"] == [[(0, 8000)], [(0, 8000)]]
    dev = src.dev_batches()
    assert len(dev) == 1 and dev[0]["clean"].shape == (2, 8000)


def test_normalize_missing_key_fails():
    with pytest.raises(KeyError, match="owner_ivs"):
        data.normalize({"noisy": [[0.0]], "clean": [[0.0]], "replay": [False], "near_silent": [False],
                        "interf_ivs": [[]]})


def test_train_stops_on_batch_misalignment(tmp_path, monkeypatch):
    """loader 若在第 k 步送来别的批（续训没对齐），训练当场报错，不会悄悄训错数据。"""
    real = data.SyntheticSource.train_batches

    def shifted(self, start_batch=0, device=None):
        yield from real(self, start_batch + 1, device)

    monkeypatch.setattr(data.SyntheticSource, "train_batches", shifted)
    with pytest.raises(RuntimeError, match="错位"):
        train.train(_cfg(tmp_path))


_NO_LOADER = "traindata 的 loader.py 还没合入本分支；合入后自动跑。人工步骤：合入 exp/025-train-data，备好小分片后重跑本文件"


@pytest.mark.skipif(not data.LOADER_PATH.exists(), reason=_NO_LOADER)
def test_real_loader_importable_and_has_contract():
    mod = data.import_loader()
    for name in ("torch_iterable", "render_on_gpu", "DevSet"):
        assert hasattr(mod, name), name


@pytest.mark.skipif(not data.LOADER_PATH.exists() or not __import__("os").environ.get("VDN_LOADER_SHARDS"),
                    reason=_NO_LOADER + "；另设 VDN_LOADER_SHARDS=<小分片目录> 与 VDN_LOADER_RECIPE=<配方>")
@pytest.mark.parametrize("workers", [1, 2])
def test_real_loader_resume_from_batch_k_equals_continuous(tmp_path, workers):
    """真 loader：从第 k 批起取，与从 0 连续取到第 k 批逐值相同，与 worker 数无关（续训等价的数据那一半）。"""
    import os
    cfg = _cfg(tmp_path, kind="loader", shard_root=os.environ["VDN_LOADER_SHARDS"],
               recipe=os.environ["VDN_LOADER_RECIPE"], num_workers=max(workers, 1))
    src = data.LoaderSource(cfg)
    it = src.train_batches(0, device=torch.device("cpu"))
    seq = [next(it) for _ in range(3)]
    again = next(src.train_batches(2, device=torch.device("cpu")))
    assert [b["batch_index"] for b in seq] == [0, 1, 2] and again["batch_index"] == 2
    assert torch.equal(seq[2]["noisy"], again["noisy"]) and torch.equal(seq[2]["clean"], again["clean"])
    assert seq[2]["owner_ivs"] == again["owner_ivs"]


# ---------- 固定开发份：data.dev_recipe（#212 数据消融）----------

def _build_info(shard_root, recipe_path):
    return (f"build-{Path(recipe_path).stem}", {}, {})


class _RecipeDev:
    """假 DevSet：记下用的是哪份配方；build_id 由配方算（与真 loader 一样，配方进 build id）。"""

    def __init__(self, shard_root, recipe_path, n=4, seed=12345, seg_s=4.0, **k):
        self.build_id = _build_info(shard_root, recipe_path)[0]
        self.size, self.len = n, 8000

    def __len__(self):
        return self.size

    def __getitem__(self, i):
        return _example(10_000 + i, self.len)


def _recipe_loader(seen):
    """假 loader：每次建 DevSet / 取训练批都把用的配方记进 seen。"""
    def torch_iterable(shard_root, recipe_path, split="train", seed=0, seg_s=4.0, batch_size=32,
                       start_batch=0, render="gpu", **kw):
        seen.append(("train", recipe_path))
        return _FakeIt(batch_size, start_batch, render)

    def DevSet(shard_root, recipe_path, **k):
        seen.append(("dev", recipe_path))
        return _RecipeDev(shard_root, recipe_path, **k)

    return types.SimpleNamespace(DevSet=DevSet, torch_iterable=torch_iterable, build_info=_build_info,
                                 render_on_gpu=lambda b, d: pytest.fail("CPU 上不该调 render_on_gpu"))


def _loader_cfg(tmp_path, **over):
    return _cfg(tmp_path, kind="loader", shard_root="s", recipe="train.json", num_workers=1, **over)


def test_dev_recipe_unset_keeps_todays_behaviour(tmp_path):
    """不写 dev_recipe：开发份 · 标定 · 训练都用 data.recipe，两个 build id 相同（与加这个字段之前一致）。"""
    seen = []
    src = data.LoaderSource(_loader_cfg(tmp_path), mod=_recipe_loader(seen))
    assert src.build_id == "build-train" and src.dev_build_id == "build-train"
    src.dev_batches()
    src.calib_batches(1)
    next(src.train_batches(0, device=torch.device("cpu")))
    assert seen == [("dev", "train.json"), ("dev", "train.json"), ("train", "train.json")]


def test_dev_recipe_fixes_dev_set_and_keeps_training_recipe(tmp_path):
    """写了 dev_recipe：开发份与标定批用 dev_recipe，训练批仍用 recipe；build_id 是训练那份，dev_build_id 是开发份那份。"""
    seen = []
    cfg = _loader_cfg(tmp_path, dev_recipe="dev.json")
    src = data.LoaderSource(cfg, mod=_recipe_loader(seen))
    assert src.build_id == "build-train"  # 续训核对 / run.json 的 data_build_id 认训练数据
    assert src.dev_build_id == "build-dev"  # 消融要固定开发份：7 个运行必须同一个开发 build
    src.dev_batches()
    src.calib_batches(1)
    next(src.train_batches(0, device=torch.device("cpu")))
    assert seen == [("dev", "dev.json"), ("dev", "dev.json"), ("train", "train.json")]


def test_dev_build_id_recorded_in_run_json(tmp_path):
    """开始训练的 run.json 里记下 dev_build_id：判消融结果时核对 7 个运行用的是不是同一份开发集。"""
    out = train.train(_cfg(tmp_path))
    rec = json.loads((tmp_path / "d" / "run.json").read_text())
    assert rec["dev_build_id"] == rec["data_build_id"] == "synthetic-seed1"
    assert out["reason"] == "max_steps"
