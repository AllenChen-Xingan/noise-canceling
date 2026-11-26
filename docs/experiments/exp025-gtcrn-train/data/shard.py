"""exp025 · 预处理与分片（034 compute.md 第 133 行起：先转 16 kHz，再打成大分片放网络盘）。

- 读切分表（split.load_split，核哈希），只取 train / dev 两份；终测原料池与 d63 不进分片
- 每条原料转 16 kHz 单声道（取第 0 声道；重采样用 exp013 dsp.resample，正本）：人声与噪声存 16 位 PCM wav，RIR 存 32 位浮点 wav
- 按时段切的来源（QUT · DEMAND）只取该份的样本区间，再切成 ≤ CHUNK_S 秒一段
- RIR 先过 exp013 rirs.Measured 的可用性检查（valid），不可用的记数、不进分片
- 打成 tar 分片（webdataset 式：<key>.wav + <key>.json），每片约 SHARD_MB；另写 index.json：每个成员的分片 · 字节偏移 · 长度，
  loader 按偏移随机读，不用整片解包
用法（云上）：python3 shard.py --root /workspace --out /workspace/shards/<名字>
"""
import argparse
import hashlib
import io
import json
import sys
import tarfile
import time
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import split  # noqa: E402

EXP013 = HERE.parents[1] / "exp013-scene-suite"
sys.path.insert(0, str(EXP013))
from dsp import SR, resample  # noqa: E402

SHARD_MB = 256
CHUNK_S = 30.0
KEEP = ("train", "dev")
RECIPE = HERE / "recipe.json"


def default_sources(recipe_path=RECIPE):
    """默认要分片的来源 = recipe.json 里出现过的全部来源（人声 · RIR · 噪声池），按首次出现去重。
    正本是 recipe.json：这里以前手掘 5 个来源，第二批加了 CochlScene 就漏了（第二处表示，2026-11-25）。"""
    r = json.loads(Path(recipe_path).read_text())
    pools = [it["source"] for k, items in r["noise_pools"].items() if not k.startswith("_") for it in items]
    return list(dict.fromkeys(list(r["speech"]["train"]) + list(r["rir"]) + pools))


DEFAULT_SOURCES = ",".join(default_sources())  # 命令行的 --sources 默认值（测试核对它就是上面这份）


def kind_of(source):
    if source.startswith("librispeech"):
        return "speech"
    if source.endswith("-rir"):
        return "rir"
    return "noise"


def read_16k(path, start=0, stop=None):
    x, sr = sf.read(str(path), start=start, stop=stop, dtype="float32", always_2d=True)
    return resample(x[:, 0].astype(np.float64), sr).astype(np.float32)


def pieces(root, source, rows):
    """逐条产出 (份, key, 16 kHz 波形, 元数据)。"""
    root = Path(root)
    k = kind_of(source)
    for r in rows:
        if "segments" in r:
            for sp, (a, b) in sorted(r["segments"].items()):
                if sp not in KEEP:
                    continue
                step = int(CHUNK_S * r["sr"])
                for j, s0 in enumerate(range(a, b, step)):
                    s1 = min(b, s0 + step)
                    if s1 - s0 < int(0.5 * r["sr"]):
                        continue
                    key = f"{r['path']}@{s0}"
                    yield sp, key, read_16k(root / r["path"], s0, s1), {"src_path": r["path"], "src_sha256": r["sha256"], "start": s0, "stop": s1, "src_sr": r["sr"]}
        elif r["split"] in KEEP:
            x = read_16k(root / r["path"])
            if k == "rir" and not rir_ok(x, r["path"]):
                yield r["split"], None, None, {"src_path": r["path"], "dropped": "rirs.Measured.valid 为假"}
                continue
            yield r["split"], r["path"], x, {"src_path": r["path"], "src_sha256": r["sha256"], "unit": r["unit"]}


def rir_ok(x, name):
    from rirs import Measured  # noqa: E402

    try:
        return bool(Measured("openslr28", "", name, 0, np.asarray(x, float), 1.0).valid)
    except (IndexError, ValueError):
        return False


def wav_bytes(x, kind):
    b = io.BytesIO()
    sf.write(b, x, SR, format="WAV", subtype="FLOAT" if kind == "rir" else "PCM_16")
    return b.getvalue()


def safe_key(key):
    return hashlib.sha1(key.encode()).hexdigest()[:20]


class ShardWriter:
    def __init__(self, out, prefix):
        self.out, self.prefix, self.n, self.tar, self.size = Path(out), prefix, 0, None, 0
        self.members, self.shards = [], []

    def _open(self):
        name = f"{self.prefix}-{len(self.shards):05d}.tar"
        self.tar = tarfile.open(self.out / name, "w", format=tarfile.USTAR_FORMAT)
        self.shards.append(name)
        self.size = 0

    def add(self, key, data, meta):
        if self.tar is None or self.size > SHARD_MB * 1e6:
            self.close()
            self._open()
        sk = safe_key(key)
        for ext, payload in (("wav", data), ("json", json.dumps(meta, ensure_ascii=False).encode())):
            ti = tarfile.TarInfo(f"{sk}.{ext}")
            ti.size, ti.mtime = len(payload), 0
            self.tar.addfile(ti, io.BytesIO(payload))
            if ext == "wav":
                off = self.tar.offset - tarfile.BLOCKSIZE * ((len(payload) + tarfile.BLOCKSIZE - 1) // tarfile.BLOCKSIZE)
                self.members.append({"key": key, "shard": self.shards[-1], "offset": off, "bytes": len(payload), **meta})
        self.size += len(data)

    def close(self):
        if self.tar is not None:
            self.tar.close()
            self.tar = None


def sha_file(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def build(root, out, sources, splits_dir=split.OUT):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=False)  # 不覆盖旧分片（CLAUDE.md 五.1）
    index = {"sr": SR, "sources": {}, "dropped": [], "split_tables": {}}
    for src in sources:
        summ, rows = split.load_split(src, splits_dir)
        index["split_tables"][src] = summ["jsonl_sha256"]
        kind = kind_of(src)
        writers = {}
        for sp, key, x, meta in pieces(root, src, rows):
            if key is None:
                index["dropped"].append(meta)
                continue
            w = writers.setdefault(sp, ShardWriter(out, f"{kind}-{src}-{sp}"))
            w.add(key, wav_bytes(x, kind), {**meta, "samples": len(x)})
        for sp, w in writers.items():
            w.close()
            index["sources"].setdefault(src, {"kind": kind})[sp] = {"members": w.members, "shards": {s: sha_file(out / s) for s in w.shards}}
    (out / "index.json").write_text(json.dumps(index, ensure_ascii=False) + "\n")
    return index


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--sources", default=DEFAULT_SOURCES, help="要分片的来源，默认 = recipe.json 里出现的全部来源")
    a = ap.parse_args()
    t0 = time.time()
    idx = build(a.root, a.out, [s for s in a.sources.split(",") if s])
    n = {s: {sp: len(v["members"]) for sp, v in d.items() if sp != "kind"} for s, d in idx["sources"].items()}
    print(f"分片完成 {time.time() - t0:.0f} s：{json.dumps(n, ensure_ascii=False)}；丢掉的 RIR {len(idx['dropped'])} 条")


if __name__ == "__main__":
    main()
