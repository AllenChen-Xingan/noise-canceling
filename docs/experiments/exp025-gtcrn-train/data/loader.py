"""exp025 · 给 trainer 的数据加载接口（和 trainer 2026-11-16 商定，见 data/README.md 第四节）。

- TrainStream：无限流，现场混音；第 i 个样本只由 (seed, 份, i) 决定
- DevSet：定长、按下标确定（map 风格），每次评估读同一批
- torch_iterable：整批出 dict；第 k 批 = 样本 k·B … k·B+B−1，只由 (seed, k, B) 决定，和 num_workers 无关；
  支持 start_batch 续训。DataLoader 用 batch_size=None
- 训练 build：分片索引里的切分表哈希 + 分片哈希 + 配方 + 混音代码的哈希 → build_id。启动时要求仓里
  `builds/<build_id>/build.json` 存在且交集检查通过（register_build 生成），否则抛错（027 第六节第 4 条）
"""
import hashlib
import io
import json
import re
import sys
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import leakcheck  # noqa: E402
import mix  # noqa: E402
import split  # noqa: E402

BUILDS = HERE / "builds"
SPLIT_CODE = {"train": 1, "dev": 2}
CODE = ("mix.py", "loader.py", "shard.py")


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def load_recipe(path):
    r = json.loads(Path(path).read_text())
    for k, v in r["knobs"].items():
        if "on" not in v or "src" not in v:
            raise ValueError(f"配方旋钮 {k} 缺 on 或 src（每个旋钮都要能单独开关、写出处）")
    return r


def build_info(shard_root, recipe_path):
    idx_bytes = (Path(shard_root) / "index.json").read_bytes()
    idx = json.loads(idx_bytes)
    parts = {
        "split_tables": idx["split_tables"],
        "shards": {s: {sp: v["shards"] for sp, v in d.items() if sp != "kind"} for s, d in idx["sources"].items()},
        "recipe_sha256": sha_bytes(Path(recipe_path).read_bytes()),
        "code_sha256": {f: sha_bytes((HERE / f).read_bytes()) for f in CODE},
    }
    bid = sha_bytes(json.dumps(parts, sort_keys=True).encode())[:16]
    return bid, parts, idx


def register_build(shard_root, recipe_path, splits_dir=split.OUT, builds_dir=BUILDS):
    """跑交集检查，写 builds/<id>/build.json（进仓）。检查不过也写（记作废），并抛错。"""
    bid, parts, idx = build_info(shard_root, recipe_path)
    tables = {}
    for src, want in idx["split_tables"].items():
        summ, rows = split.load_split(src, splits_dir)
        if summ["jsonl_sha256"] != want:
            raise SystemExit(f"分片用的 {src} 切分表 {want[:12]}… 和仓里的 {summ['jsonl_sha256'][:12]}… 不同")
        tables[src] = rows
    lk = leakcheck.check(tables)
    out = Path(builds_dir) / bid
    out.mkdir(parents=True, exist_ok=True)
    rec = {"build_id": bid, **parts, "leak_check": lk, "valid": lk["ok"], "dropped_rir": len(idx.get("dropped", [])),
           "determinism": "第 i 个样本只由 (seed, 份, i) 决定；第 k 批 = 样本 k·B…k·B+B−1，和 num_workers 无关；换批量 B 内容会变",
           "counts": {s: {sp: len(v["members"]) for sp, v in d.items() if sp != "kind"} for s, d in idx["sources"].items()}}
    (out / "build.json").write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n")
    if not lk["ok"]:
        raise SystemExit(f"build {bid} 交集检查不过，作废：{lk['hits'][:5]}")
    return bid


class Pools:
    """把分片索引按「人声 / 噪声池 / RIR」组织好；读音频按偏移随机读。"""

    def __init__(self, shard_root, recipe, idx, split_):
        self.root, self.idx, self.split = Path(shard_root), idx, split_
        src = idx["sources"]

        def members(name, required=True):
            if name not in src or split_ not in src[name]:
                if required:
                    raise SystemExit(f"分片里没有 {name} 的 {split_} 份")
                self.missing.append(name)  # 噪声池里某个来源这一份为空（如时段太短）：跳过，记下
                return []
            return src[name][split_]["members"]

        self.missing = []

        self.speech = [m for s in recipe["speech"]["train"] for m in members(s)]
        self.rir = [m for s in recipe["rir"] for m in members(s)]
        self.pools = {}
        for scene, items in recipe["noise_pools"].items():
            if scene.startswith("_"):
                continue
            ms = []
            for it in items:
                rx = re.compile(it["regex"])
                ms += [(m, bool(it.get("wind")), it["source"]) for m in members(it["source"], False) if rx.search(m["src_path"])]
            if ms:
                self.pools[scene] = ms
        if not self.speech or not self.pools:
            raise SystemExit("人声或噪声池为空：先检查分片与配方 noise_pools 的正则")
        self._fh = {}

    def close(self):
        for f in self._fh.values():
            f.close()
        self._fh = {}

    def __del__(self):
        self.close()

    def read(self, m):
        f = self._fh.get(m["shard"])
        if f is None:
            f = self._fh[m["shard"]] = open(self.root / m["shard"], "rb")
        f.seek(m["offset"])
        x, sr = sf.read(io.BytesIO(f.read(m["bytes"])), dtype="float32")
        assert sr == 16000
        return x


class _Base:
    def __init__(self, shard_root, recipe_path, split="train", seed=0, seg_s=4.0, builds_dir=BUILDS, check_build=True):
        self.recipe = load_recipe(recipe_path)
        self.build_id, _, idx = build_info(shard_root, recipe_path)
        if check_build:
            bj = Path(builds_dir) / self.build_id / "build.json"
            if not bj.exists() or not json.loads(bj.read_text())["valid"]:
                raise SystemExit(f"build {self.build_id} 没登记或交集检查不过：先跑 register_build（loader.py --register）")
        self.split, self.seed, self.n = split, int(seed), int(round(seg_s * 16000))
        self.p = Pools(shard_root, self.recipe, idx, split)

    def plan(self, i):
        """第 i 个样本的 Plan（CPU：读片段 + 抽随机数）；渲染另做（CPU 用 mix.render，GPU 用 gpu_mix.render_batch）。"""
        rng = np.random.default_rng([self.seed, SPLIT_CODE[self.split], int(i)])
        K = self.recipe["knobs"]
        p = self.p
        sp = p.speech[int(rng.integers(len(p.speech)))]
        scenes = sorted(p.pools)
        ts = K["target_scene"]
        replay = False
        if ts["on"] and ts["scene"]:
            if ts["scene"] not in p.pools:
                raise SystemExit(f"目标场景 {ts['scene']} 没有噪声池")
            others = [s for s in scenes if s != ts["scene"]]
            replay = bool(K["replay"]["on"] and others and rng.random() < K["replay"]["frac"])
            scene = others[int(rng.integers(len(others)))] if replay else ts["scene"]
        else:
            scene = scenes[int(rng.integers(len(scenes)))]
        pool = p.pools[scene]
        picks = [pool[int(j)] for j in rng.integers(len(pool), size=mix.MAX_NOISES)]
        noises = [(p.read(m), w, f"{src}:{m['key']}") for m, w, src in picks]
        rm = p.rir[int(rng.integers(len(p.rir)))] if p.rir else None
        ir = p.read(rm) if rm is not None else None
        pl = mix.plan(rng, p.read(sp), noises, ir, K, self.n)
        pl.meta.update({"i": int(i), "scene": scene, "replay": replay, "speech": sp["key"], "rir": rm["key"] if rm else None})
        return pl

    def example(self, i):
        """CPU 渲染一段（本机冒烟、对拍用；训练走 GPU）。"""
        noisy, clean, meta = mix.render(self.plan(i), self.n)
        return {"noisy": noisy, "clean": clean, "meta": meta}


class TrainStream(_Base):
    def __init__(self, *a, worker_id=0, num_workers=1, **k):
        super().__init__(*a, **k)
        self.worker_id, self.num_workers = worker_id, num_workers

    def __iter__(self):
        i = self.worker_id
        while True:
            yield self.example(i)
            i += self.num_workers


class DevSet(_Base):
    def __init__(self, shard_root, recipe_path, n=2000, seed=12345, seg_s=4.0, **k):
        super().__init__(shard_root, recipe_path, split="dev", seed=seed, seg_s=seg_s, **k)
        self.size = int(n)

    def __len__(self):
        return self.size

    def __getitem__(self, i):
        if not 0 <= i < self.size:
            raise IndexError(i)
        return self.example(i)


def collate(exs):
    return {"noisy": np.stack([e["noisy"] for e in exs]), "clean": np.stack([e["clean"] for e in exs]),
            "replay": np.array([e["meta"]["replay"] for e in exs]), "near_silent": np.array([e["meta"]["near_silent"] for e in exs]),
            "owner_ivs": [e["meta"]["owner_ivs"] for e in exs], "interf_ivs": [e["meta"]["interf_ivs"] for e in exs],
            "meta": [e["meta"] for e in exs]}


def batch(ds, k, batch_size):
    """CPU 渲染的第 k 批（对拍用）。"""
    return collate([ds.example(k * batch_size + j) for j in range(batch_size)])


def plan_batch(ds, k, batch_size):
    """第 k 批的 Plan 打包（numpy），交给 render_on_gpu。只由 (seed, k, batch_size) 决定。"""
    import gpu_mix

    b = gpu_mix.collate_plans([ds.plan(k * batch_size + j) for j in range(batch_size)])
    b["batch_index"] = k
    return b


def render_on_gpu(b, device="cuda"):
    """plan_batch（或 DataLoader 送来的同一结构）→ 渲染好的一批 dict：noisy · clean [B, T] float32（在 device 上）·
    replay · near_silent bool [B] · owner_ivs · interf_ivs（list，样本下标，左闭右开）· clip_thr · meta · batch_index。"""
    import torch

    import gpu_mix

    out = gpu_mix.render_batch(b, device=device)
    out["replay"] = torch.tensor([m["replay"] for m in b["meta"]], device=device)
    out["near_silent"] = torch.tensor([m["near_silent"] for m in b["meta"]], device=device)
    out["batch_index"] = int(b["batch_index"])
    for m, iv, thr in zip(out["meta"], out["owner_ivs"], out["clip_thr"]):
        m.update(owner_ivs=iv, clip_thr=thr)
    return out


def torch_iterable(shard_root, recipe_path, split="train", seed=0, seg_s=4.0, batch_size=32, start_batch=0, render="gpu", **kw):
    """返回 torch IterableDataset；DataLoader(it, batch_size=None, num_workers=N)。
    render="gpu"（默认，D102 后）：worker 只出 plan_batch（读片段 + 随机数），主进程里 loader.render_on_gpu(b, "cuda") 混音；
    render="cpu"：worker 里直接 CPU 渲染（旧路，对拍与无卡冒烟用）。"""
    import torch
    from torch.utils.data import IterableDataset, get_worker_info

    class It(IterableDataset):
        def __init__(self):
            self.ds = _Base(shard_root, recipe_path, split=split, seed=seed, seg_s=seg_s, **kw)
            self.build_id = self.ds.build_id

        def __iter__(self):
            w = get_worker_info()
            wid, nw = (w.id, w.num_workers) if w else (0, 1)
            k = start_batch + wid
            while True:
                if render == "gpu":
                    yield plan_batch(self.ds, k, batch_size)
                else:
                    b = batch(self.ds, k, batch_size)
                    for key in ("noisy", "clean", "replay", "near_silent"):
                        b[key] = torch.from_numpy(b[key])
                    b["batch_index"] = k
                    yield b
                k += nw

    return It()


if __name__ == "__main__":
    import argparse

    ap = argparse.ArgumentParser()
    ap.add_argument("--register", action="store_true")
    ap.add_argument("--shards", required=True)
    ap.add_argument("--recipe", default=str(HERE / "recipe.json"))
    a = ap.parse_args()
    if a.register:
        print("build", register_build(a.shards, a.recipe), "已登记，交集检查通过")
