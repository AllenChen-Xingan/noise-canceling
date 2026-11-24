"""数据来源适配：trainer 只认一种批次格式，两种来源。

批次（dict）：`noisy` (B,N) · `clean` (B,N) float32 · `replay` bool(B) · `near_silent` bool(B) ·
`owner_ivs` / `interf_ivs`（每段一个区间 list，样本下标左闭右开）。与 traindata 约定（2026-11-16 会话间消息）。

- `synthetic`：仓内现场合成的小样（带包络的噪声爆发当「人声」+ 有色噪声），只给测试与本机冒烟用，
  **不出任何效果结论**；第 k 批只由 (seed, k) 决定。
- `loader`：traindata 的 `../data/loader.py`（`torch_iterable` · `DevSet` · `build_id`）。区间 → 掩码在 trainer 里做
  （`losses.owner_only_mask`，正本归 trainer，和 analysis.rs 对拍）。
"""
from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import numpy as np
import torch

SR = 16_000
LOADER_PATH = Path(__file__).resolve().parents[1] / "data" / "loader.py"


# ---------- 合成小样 ----------

def synth_example(rng: np.random.Generator, n: int) -> dict:
    clean = np.zeros(n, np.float32)
    ivs, t = [], int(rng.uniform(0.05, 0.3) * SR)
    while True:
        dur = int(rng.uniform(0.2, 0.6) * SR)
        if t + dur > n:
            break
        env = np.sin(np.linspace(0, np.pi, dur)) ** 0.5
        f0 = rng.uniform(100, 250)
        tt = np.arange(dur) / SR
        voiced = sum(np.sin(2 * np.pi * f0 * h * tt) / h for h in range(1, 12))
        clean[t:t + dur] = (voiced * env * rng.uniform(0.03, 0.2)).astype(np.float32)
        ivs.append((t, t + dur))
        t += dur + int(rng.uniform(0.12, 0.5) * SR)
    white = rng.standard_normal(n)
    noise = np.cumsum(white) * 0.02 + white  # 偏低频的有色噪声
    noise = (noise - noise.mean()) / (noise.std() + 1e-9)
    ps = float((clean ** 2).mean()) + 1e-12
    snr = rng.uniform(-5, 15)
    noise *= np.sqrt(ps / 10 ** (snr / 10))
    return {"noisy": (clean + noise).astype(np.float32), "clean": clean, "owner_ivs": ivs, "interf_ivs": []}


def synth_batch(seed: int, k: int, batch: int, n: int) -> dict:
    rng = np.random.default_rng([seed, k])
    exs = [synth_example(rng, n) for _ in range(batch)]
    return collate(exs)


def collate(exs: list[dict]) -> dict:
    """单段 dict 拼成批。单段的标记可以在顶层，也可以在 `meta` 里（traindata DevSet 的写法）。"""
    def tag(e, k, default):
        return e.get("meta", {}).get(k, e.get(k, default))
    return normalize({
        "noisy": np.stack([e["noisy"] for e in exs]),
        "clean": np.stack([e["clean"] for e in exs]),
        "replay": [bool(tag(e, "replay", False)) for e in exs],
        "near_silent": [bool(tag(e, "near_silent", False)) for e in exs],
        "owner_ivs": [tag(e, "owner_ivs", []) for e in exs],
        "interf_ivs": [tag(e, "interf_ivs", []) for e in exs],
    })


BATCH_KEYS = ("noisy", "clean", "replay", "near_silent", "owner_ivs", "interf_ivs")


def normalize(b: dict) -> dict:
    """loader 出的整批 dict → trainer 批次格式；缺键即报错（接口对不上要早红）。"""
    miss = [k for k in BATCH_KEYS if k not in b]
    if miss:
        raise KeyError(f"loader 批次缺键：{miss}")
    out = {
        "noisy": torch.as_tensor(b["noisy"], dtype=torch.float32),
        "clean": torch.as_tensor(b["clean"], dtype=torch.float32),
        "replay": torch.as_tensor(b["replay"], dtype=torch.bool),
        "near_silent": torch.as_tensor(b["near_silent"], dtype=torch.bool),
        "owner_ivs": [[tuple(map(int, p)) for p in s] for s in b["owner_ivs"]],
        "interf_ivs": [[tuple(map(int, p)) for p in s] for s in b["interf_ivs"]],
    }
    if "batch_index" in b:
        out["batch_index"] = int(b["batch_index"])
    return out


class SyntheticSource:
    def __init__(self, cfg):
        self.cfg = cfg
        self.n = int(cfg.data.seg_s * SR)
        self.build_id = f"synthetic-seed{cfg.run.seed}"
        self.num_workers = 0

    def train_batches(self, start_batch: int = 0, device=None):
        k = start_batch
        while True:
            b = synth_batch(self.cfg.run.seed, k, self.cfg.data.batch, self.n)
            b["batch_index"] = k
            yield b
            k += 1

    def dev_batches(self) -> list[dict]:
        d = self.cfg.data
        nb = max(1, min(d.dev_batches, -(-d.dev_n // d.batch)))
        return [synth_batch(d.dev_seed, 10_000_000 + i, d.batch, self.n) for i in range(nb)]


# ---------- traindata 的 loader ----------

def import_loader(path: Path = LOADER_PATH):
    name = "exp025_loader"
    if name in sys.modules:
        return sys.modules[name]
    if not path.exists():
        raise FileNotFoundError(f"traindata 的 loader 还没落地：{path}")
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


class LoaderSource:
    def __init__(self, cfg, mod=None):
        self.cfg = cfg
        self.mod = mod or import_loader()
        d = cfg.data
        if not d.shard_root or not d.recipe:
            raise ValueError("data.kind = loader 时必须给 data.shard_root 与 data.recipe")
        self.num_workers = d.num_workers
        self.dev = self.mod.DevSet(d.shard_root, d.recipe, n=d.dev_n, seed=d.dev_seed, seg_s=d.seg_s)
        self.build_id = getattr(self.dev, "build_id", None) or "unknown"

    def train_batches(self, start_batch: int = 0, device=None):
        """traindata 约定：第 k 批只由 (seed, k, batch_size) 决定，与 num_workers 无关；torch_iterable 自己出整批，
        所以 DataLoader 用 batch_size=None。显卡上走 render="gpu"：worker 只出混音计划，主进程 render_on_gpu 混音
        （D102）；没有显卡走 render="cpu"。换批量会换内容（续训时 runrec 核）。"""
        d = self.cfg.data
        on_gpu = device is not None and torch.device(device).type == "cuda"
        it = self.mod.torch_iterable(d.shard_root, d.recipe, split="train", seed=self.cfg.run.seed,
                                     seg_s=d.seg_s, batch_size=d.batch, start_batch=start_batch,
                                     render="gpu" if on_gpu else "cpu")
        dl = torch.utils.data.DataLoader(it, batch_size=None, num_workers=d.num_workers,
                                         persistent_workers=d.num_workers > 0)
        for b in dl:
            yield normalize(self.mod.render_on_gpu(b, device) if on_gpu else b)

    def dev_batches(self) -> list[dict]:
        d = self.cfg.data
        n = min(len(self.dev), d.dev_batches * d.batch)
        return [collate([self.dev[j] for j in range(i, min(i + d.batch, n))]) for i in range(0, n, d.batch)]


def make_source(cfg):
    return SyntheticSource(cfg) if cfg.data.kind == "synthetic" else LoaderSource(cfg)


def frames_mask(batch: dict, n: int) -> torch.Tensor:
    """批次的「主说话人单独说话」10 ms 帧掩码 (B, n//160)。"""
    from losses import owner_only_mask
    return torch.as_tensor(np.stack([owner_only_mask(o, i, n)
                                     for o, i in zip(batch["owner_ivs"], batch["interf_ivs"])]))
