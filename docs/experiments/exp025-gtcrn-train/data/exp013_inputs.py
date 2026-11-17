"""exp013 原料逐文件哈希：本机记一次（--record），云上重建前核一遍（--verify）。

- 文件清单跟 exp013 `pools.index_hash()` 同一份：先用 pools 的函数列出文件，再把「相对路径 + 字节数」的哈希
  和 `pools.index_hash()` 比——对不上就报错（正本是 pools.py，这里不许悄悄多列或少列）
- 再和正式 build `20261103230756-17f5` 的 manifest 里记的 material_index 比，保证本机原料就是当时那一份
- 输出 `exp013_inputs.json`（进仓）：每个文件的相对路径 · 字节数 · sha256
用法：python3 exp013_inputs.py --record | --verify [--data <原料根，默认 ~/vdn-data>]
"""
import argparse
import hashlib
import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP013 = HERE.parents[1] / "exp013-scene-suite"
BUILD = "20261103230756-17f5"
OUT = HERE / "exp013_inputs.json"


def load_pools(data=None):
    """import exp013 的 pools；data 给了就先设 HOME，让 pools.DATA 指向 <data>（pools 写死 ~/vdn-data）。"""
    if data:
        root = Path(data).resolve()
        if root.name != "vdn-data":
            raise SystemExit(f"--data 必须是名为 vdn-data 的目录（pools 写死 ~/vdn-data），收到 {root}")
        os.environ["HOME"] = str(root.parent)
    sys.path.insert(0, str(EXP013))
    import pools  # noqa: E402

    return pools


def file_list(pools):
    """和 pools.index_hash() 同一组文件；最后由 check_against_pools 比对，不一致即报错。"""
    L = pools.libri()
    files = [f for pool in ("owner", "interf") for fs in L[pool].values() for f in fs]
    files += sorted(pools.DEMAND.glob("*/ch*.wav")) + [pools.DATA / k for v in pools.esc50().values() for k in v]
    files += list(pools.ZH) + [pools.MUSIC]
    rir = pools.DATA / "rir"
    files += sorted(rir.glob("ace-single/Single/*/*/*.wav")) + sorted(rir.glob("mit-ir-survey/Audio/*.wav"))
    files += sorted(rir.glob("mcvampire/ir/*.wav")) + sorted(rir.glob("mcvampire/driving_noise/*.wav")) + [rir / "mcvampire/geometry.mat"]
    return [Path(f) for f in files]


def index_of(rows):
    return hashlib.sha256("\n".join(sorted(f"{r['path']}\t{r['bytes']}" for r in rows)).encode()).hexdigest()


def check_against_pools(pools, rows):
    want, n = pools.index_hash()
    got = index_of(rows)
    if got != want or n != len(rows):
        raise SystemExit(f"文件清单和 pools.index_hash() 不一致（{len(rows)} 对 {n}）：先改本脚本 file_list 跟上 pools.py")
    man = json.loads((EXP013 / "builds" / BUILD / "manifest.json").read_text())["material_index"]
    if got != man["sha256"] or n != man["files"]:
        raise SystemExit(f"原料索引 {got[:12]}… ≠ build {BUILD} 记的 {man['sha256'][:12]}…：原料不是当时那一份")
    return got


def sha256(path, buf=1 << 20):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(buf), b""):
            h.update(b)
    return h.hexdigest()


def rows_for(pools):
    return [{"path": str(f.relative_to(pools.DATA)), "bytes": f.stat().st_size} for f in file_list(pools)]


def record(pools):
    rows = rows_for(pools)
    idx = check_against_pools(pools, rows)
    for r in rows:
        r["sha256"] = sha256(pools.DATA / r["path"])
    out = {"_说明": f"exp013 正式 build {BUILD} 的原料逐文件 sha256（本机记，云上重建前核）；音频不进仓",
           "build": BUILD, "material_index": idx, "files": sorted(rows, key=lambda r: r["path"])}
    OUT.write_text(json.dumps(out, ensure_ascii=False, indent=0) + "\n")
    print(f"已记 {len(rows)} 个文件 → {OUT.name}")


def verify(pools, rec=None):
    """返回不一致的条目列表（空 = 全对）。"""
    rec = rec or json.loads(OUT.read_text())
    bad = []
    for r in rec["files"]:
        p = pools.DATA / r["path"]
        if not p.exists():
            bad.append(f"缺 {r['path']}")
        elif p.stat().st_size != r["bytes"] or sha256(p) != r["sha256"]:
            bad.append(f"哈希不对 {r['path']}")
    if not bad:
        check_against_pools(pools, rows_for(pools))
    return bad


def main():
    ap = argparse.ArgumentParser()
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--record", action="store_true")
    g.add_argument("--verify", action="store_true")
    ap.add_argument("--data", default="")
    a = ap.parse_args()
    pools = load_pools(a.data)
    if a.record:
        record(pools)
        return
    bad = verify(pools)
    print("原料全对" if not bad else f"{len(bad)} 处不对：{bad[:10]}")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
