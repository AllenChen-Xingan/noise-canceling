"""exp025 · 按 split_rules.json 把每个来源切成 train / dev / final / d63（027 第六节第 5 条，D63 第 1 条）。

- 切法确定：单位（说话人 / 房间 / 文件）按 sha256(种子:来源:单位) 排序后按比例分（最大余数法，合计不差一个）；
  不依赖 numpy 版本、文件系统列目录的顺序
- 按时段切的（QUT · DEMAND）：每个文件按样本号切几段，段界写进清单
- 输出（进仓）：`splits/<来源>.jsonl`（每文件一行：路径 · 字节 · sha256 · 单位 · 份 / 段界）+ `splits/<来源>.json`（摘要：
  规则哈希 · 种子 · 每份单位数与文件数 · jsonl 的 sha256）
用法（云上）：python3 split.py --root /workspace [--only qut-noise] [--inventory]
"""
import argparse
import glob as globmod
import hashlib
import json
import re
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import soundfile as sf

HERE = Path(__file__).resolve().parent
RULES = HERE / "split_rules.json"
OUT = HERE / "splits"
EXP013 = HERE.parents[1] / "exp013-scene-suite"


def load_rules(path=RULES):
    return json.loads(Path(path).read_text())


def rule_sha(rule):
    return hashlib.sha256(json.dumps(rule, sort_keys=True, ensure_ascii=False).encode()).hexdigest()


def list_files(root, rule):
    pats = rule["glob"] if isinstance(rule["glob"], list) else [rule["glob"]]
    out = set()
    for p in pats:
        out |= {Path(f) for f in globmod.glob(str(Path(root) / p), recursive=True)}
    rx = re.compile(rule["exclude_regex"]) if rule.get("exclude_regex") else None
    rel = sorted(str(f.relative_to(root)) for f in out if f.is_file())
    return [r for r in rel if not (rx and rx.search(r))]


def demand_envs():
    sys.path.insert(0, str(EXP013))
    import pools  # noqa: E402  正本：exp006 recipes.json 经 pools._exp006_rules

    return set(pools.EXP006_DEMAND)


def room_map(root, rule):
    """从 OpenSLR 28 包内的 Kaldi rir_list 读「文件名 → 房间号」；房间号前加列表所在目录，防不同房型重号。"""
    m = {}
    for pat in rule.get("room_lists", []):
        for lst in sorted(globmod.glob(str(Path(root) / pat))):
            tag = Path(lst).parent.name
            for line in Path(lst).read_text().splitlines():
                rid = re.search(r"--room-id\s+(\S+)", line)
                wav = re.search(r"(\S+\.wav)", line)
                if rid and wav:
                    m[Path(wav.group(1)).name] = f"{tag}/{rid.group(1)}"
    return m


def unit_of(rel, rule, rooms=None):
    u = rule["unit"]
    if u == "speaker":
        return Path(rel).parts[-3]
    if u == "room":
        name = Path(rel).name
        if name not in rooms:
            raise SystemExit(f"{rel} 不在任何 rir_list 里，查不到房间号：不许按文件切（同房间会跨份）")
        return rooms[name]
    if u == "file":
        return rel
    raise ValueError(u)


def rank_key(seed, source, unit):
    return hashlib.sha256(f"{seed}:{source}:{unit}".encode()).hexdigest()


def allocate(n, frac, order):
    """最大余数法：各份个数之和恰为 n。order 是份的先后（决定排序后谁拿前面的单位）。"""
    raw = {k: frac.get(k, 0.0) * n for k in order}
    cnt = {k: int(raw[k]) for k in order}
    left = n - sum(cnt.values())
    for k in sorted(order, key=lambda k: (-(raw[k] - cnt[k]), order.index(k)))[:left]:
        cnt[k] += 1
    return cnt


def assign_units(units, frac, seed, source, order, strata=None):
    """返回 {单位: 份}。strata 给了（单位 → 层）时每层分别按比例分。"""
    if abs(sum(frac.values()) - 1.0) > 1e-9:
        raise ValueError(f"{source} 比例之和 {sum(frac.values())} ≠ 1")
    groups = {}
    for u in sorted(set(units)):
        groups.setdefault((strata or {}).get(u, ""), []).append(u)
    out = {}
    for _, us in sorted(groups.items()):
        us = sorted(us, key=lambda u: rank_key(seed, source, u))
        cnt, i = allocate(len(us), frac, order), 0
        for k in order:
            for u in us[i : i + cnt[k]]:
                out[u] = k
            i += cnt[k]
    return out


def time_segments(frames, spec):
    return {k: [int(round(a * frames)), int(round(b * frames))] for k, a, b in spec}


def sha256(path, buf=1 << 20):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(buf), b""):
            h.update(b)
    return h.hexdigest()


def split_source(root, name, rule, seed, order, workers=16):
    root = Path(root)
    files = list_files(root, rule)
    if name == "demand-train":
        envs = demand_envs()
        files = [f for f in files if Path(f).parts[-2] in envs]
    if not files:
        raise SystemExit(f"{name}：按 {rule['glob']} 一个文件也没找到（先跑 fetch.py）")

    def info(rel):
        p = root / rel
        i = sf.info(str(p))
        return {"path": rel, "bytes": p.stat().st_size, "sha256": sha256(p), "frames": int(i.frames), "sr": int(i.samplerate)}

    with ThreadPoolExecutor(workers) as ex:
        rows = list(ex.map(info, files))
    if rule["unit"] == "time":
        for r in rows:
            r["unit"] = r["path"]
            r["segments"] = time_segments(r["frames"], rule["time"])
    else:
        rooms = room_map(root, rule) if rule["unit"] == "room" else None
        for r in rows:
            r["unit"] = unit_of(r["path"], rule, rooms)
        strata = None
        if rule.get("stratum_regex"):
            rx = re.compile(rule["stratum_regex"])
            strata = {r["unit"]: ("synth" if rx.search(r["path"]) else "real") for r in rows}
        amap = assign_units([r["unit"] for r in rows], rule["frac"], seed, name, order, strata)
        for r in rows:
            r["split"] = amap[r["unit"]]
    return rows


def summary(name, rule, seed, rows, jsonl_bytes):
    s = {"source": name, "group": rule["group"], "unit": rule["unit"], "seed": seed, "rule_sha256": rule_sha(rule),
         "files": len(rows), "units": len({r["unit"] for r in rows}), "jsonl_sha256": hashlib.sha256(jsonl_bytes).hexdigest()}
    if rule["unit"] == "time":
        s["time"] = rule["time"]
        s["seconds"] = round(sum(r["frames"] / r["sr"] for r in rows), 1)
    else:
        per = {}
        for r in rows:
            d = per.setdefault(r["split"], {"units": set(), "files": 0, "seconds": 0.0})
            d["units"].add(r["unit"])
            d["files"] += 1
            d["seconds"] += r["frames"] / r["sr"]
        s["per_split"] = {k: {"units": len(v["units"]), "files": v["files"], "seconds": round(v["seconds"], 1)} for k, v in sorted(per.items())}
    return s


def write(name, rule, seed, rows, out_dir=OUT):
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    body = "".join(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n" for r in sorted(rows, key=lambda r: r["path"])).encode()
    jl, js = out_dir / f"{name}.jsonl", out_dir / f"{name}.json"
    if js.exists():
        old = json.loads(js.read_text())
        if old["jsonl_sha256"] != hashlib.sha256(body).hexdigest():
            raise SystemExit(f"{js.name} 已存在且内容不同：切分表进仓后不原地覆盖（CLAUDE.md 五.1）；要改切法先改规则、换种子并说明")
    jl.write_bytes(body)
    s = summary(name, rule, seed, rows, body)
    js.write_text(json.dumps(s, ensure_ascii=False, indent=1) + "\n")
    return s


def load_split(name, out_dir=OUT):
    """读一张切分表并核摘要里记的哈希；对不上即报错（loader 与 leakcheck 都走这里）。"""
    out_dir = Path(out_dir)
    s = json.loads((out_dir / f"{name}.json").read_text())
    body = (out_dir / f"{name}.jsonl").read_bytes()
    if hashlib.sha256(body).hexdigest() != s["jsonl_sha256"]:
        raise SystemExit(f"{name}.jsonl 和摘要里的哈希对不上：切分表被改过")
    return s, [json.loads(x) for x in body.decode().splitlines()]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--only", default="")
    ap.add_argument("--inventory", action="store_true", help="只列每个来源匹配到的文件数与前 20 个路径，不写切分表")
    ap.add_argument("--out", default=str(OUT))
    a = ap.parse_args()
    R = load_rules()
    names = [n for n in (a.only.split(",") if a.only else R["sources"]) if n]
    for n in names:
        rule = R["sources"][n]
        if a.inventory:
            fs = list_files(Path(a.root), rule)
            print(f"{n}：{len(fs)} 个文件", *fs[:20], sep="\n  ")
            continue
        s = write(n, rule, R["seed"], split_source(a.root, n, rule, R["seed"], R["splits"]), a.out)
        print(json.dumps(s, ensure_ascii=False))


if __name__ == "__main__":
    main()
