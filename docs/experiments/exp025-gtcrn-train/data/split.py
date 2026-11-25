"""exp025 · 按 split_rules.json 把每个来源切成 train / dev / final / d63（027 第六节第 5 条，D63 第 1 条）。

- 切法确定：单位（说话人 / 房间 / 文件）按 sha256(种子:来源:单位) 排序后按比例分（最大余数法，合计不差一个）；
  不依赖 numpy 版本、文件系统列目录的顺序
- 按时段切的（QUT · DEMAND）：每个文件按样本号切几段，段界写进清单
- 输出（进仓）：`splits/<来源>.jsonl`（每文件一行：路径 · 字节 · sha256 · 单位 · 份 / 段界）+ `splits/<来源>.json`（摘要：
  规则哈希 · 种子 · 每份单位数与文件数 · jsonl 的 sha256）。正文超 GZIP_OVER 时存成 `<来源>.jsonl.gz`（auto-commit hook
  不收超 10 MB 的文件）；摘要里的哈希始终是解压后正文的
用法（云上）：python3 split.py --root /workspace [--only qut-noise] [--inventory]
"""
import argparse
import glob as globmod
import gzip
import hashlib
import json
import re
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path


HERE = Path(__file__).resolve().parent
GZIP_OVER = 8 << 20  # 留余量，低于 hook 的 10 MB 线
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


def parse_rir_line(line):
    """Kaldi rir_list 一行 → (路径, room-id)。按「--键 值」成对读，剩下的那个才是路径；
    不能找第一个 .wav：RWCP type4 的 room-id 本身就是文件名（`--room-id RWCP_000l.wav`）。"""
    toks, opts, free, i = line.split(), {}, [], 0
    while i < len(toks):
        if toks[i].startswith("--") and i + 1 < len(toks):
            opts[toks[i]], i = toks[i + 1], i + 2
        else:
            free.append(toks[i])
            i += 1
    path = [opts["--rir-rspecifier"]] if "--rir-rspecifier" in opts else free
    if len(path) != 1 or "--room-id" not in opts:
        raise SystemExit(f"rir_list 这一行认不出路径与房间号：{line!r}")
    return path[0], opts["--room-id"]


def room_map(root, rule):
    """从 OpenSLR 28 包内的 Kaldi rir_list 读「相对路径 → 房间号」；房间号前加列表所在目录，防不同房型重号。
    room_lists_sha256：规则是对着这几份上游 rir_list 定的，内容变了就停。
    room_merge：上游 room-id 不是房间的那几个库（见 split_rules.json 里的 why），整库并成一个单位。"""
    root, m = Path(root), {}
    pins = rule.get("room_lists_sha256")
    seen = set()
    merge = [(re.compile(x["match"]), x["unit"]) for x in rule.get("room_merge", [])]
    for pat in rule.get("room_lists", []):
        for lst in sorted(globmod.glob(str(root / pat))):
            rel = str(Path(lst).relative_to(root))
            body = Path(lst).read_bytes()
            if pins is not None:
                if pins.get(rel) != hashlib.sha256(body).hexdigest():
                    raise SystemExit(f"{rel} 和规则核过的上游版本不同（sha256 对不上）：房间规则要重新核")
                seen.add(rel)
            tag = Path(lst).parent.name
            lparts = Path(rel).parts
            for line in body.decode().splitlines():
                if not line.strip():
                    continue
                wav, rid = parse_rir_line(line)
                wparts = Path(wav).parts
                if wparts[0] not in lparts:
                    raise SystemExit(f"{rel}：路径 {wav} 和列表所在目录对不上")
                key = str(Path(*lparts[: lparts.index(wparts[0])], *wparts))  # 完整相对路径：三套模拟 RIR 文件名重复
                unit = f"{tag}/{rid}"
                for rx, to in merge:
                    if rx.search(unit):
                        unit = to
                        break
                if key in m:
                    raise SystemExit(f"{key} 在 rir_list 里出现两次")
                m[key] = unit
    if pins is not None and seen != set(pins):
        raise SystemExit(f"rir_list 少了：{sorted(set(pins) - seen)}")
    return m


def unit_of(rel, rule, rooms=None):
    u = rule["unit"]
    if u == "speaker":
        return Path(rel).parts[-3]
    if u == "room":
        name = str(Path(rel))
        if name not in rooms:
            raise SystemExit(f"{rel} 不在任何 rir_list 里，查不到房间号：不许按文件切（同房间会跨份）")
        return rooms[name]
    if u == "file":
        for x in rule.get("unit_regex", []):  # 同一次录音被切成多个文件时按录音切（027）
            m = re.search(x["match"], rel)
            if m:
                return m.expand(x["unit"])
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


def check_expect(name, rule, files):
    """文件数（与说话人数）对不上上游公布的数目就停：通配符写错时不写切分表。"""
    e = rule.get("expect", {})
    n = len(files)
    bad = []
    if "files" in e and n != e["files"]:
        bad.append(f"文件 {n} 个，应为 {e['files']}")
    if "min_files" in e and n < e["min_files"]:
        bad.append(f"文件 {n} 个，应至少 {e['min_files']}")
    if "units" in e and rule["unit"] == "speaker":
        u = len({unit_of(f, rule) for f in files})
        if u != e["units"]:
            bad.append(f"说话人 {u} 名，应为 {e['units']}")
    if bad:
        raise SystemExit(f"{name}：{'；'.join(bad)}（{e.get('src', '')}）。不写切分表；请把 split.py --inventory 的输出发给 traindata")


def split_source(root, name, rule, seed, order, workers=32, cache=None):
    """先只看路径把单位全认出来（认不出立刻停，不白算哈希），再逐文件读数（cache 是 filecache.FileCache，可复用上次的读数）。"""
    from filecache import FileCache

    root = Path(root)
    files = list_files(root, rule)
    if name == "demand-train":
        envs = demand_envs()
        files = [f for f in files if Path(f).parts[-2] in envs]
    if not files:
        raise SystemExit(f"{name}：按 {rule['glob']} 一个文件也没找到（先跑 fetch.py）")
    check_expect(name, rule, files)
    if rule["unit"] == "time":
        units = {f: f for f in files}
    else:
        rooms = room_map(root, rule) if rule["unit"] == "room" else None
        units = {f: unit_of(f, rule, rooms) for f in files}
    cache = cache or FileCache(None)
    with ThreadPoolExecutor(workers) as ex:
        rows = list(ex.map(lambda rel: cache.info(root, rel), files))
    for r in rows:
        r["unit"] = units[r["path"]]
    if rule["unit"] == "time":
        for r in rows:
            r["segments"] = time_segments(r["frames"], rule["time"])
    else:
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


def sha256_bytes(b):
    return hashlib.sha256(b).hexdigest()


def write(name, rule, seed, rows, out_dir=OUT):
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    body = "".join(json.dumps(r, ensure_ascii=False, sort_keys=True) + "\n" for r in sorted(rows, key=lambda r: r["path"])).encode()
    jl, js = out_dir / f"{name}.jsonl", out_dir / f"{name}.json"
    if js.exists():
        old = json.loads(js.read_text())
        if old["jsonl_sha256"] != sha256_bytes(body):
            raise SystemExit(f"{js.name} 已存在且内容不同：切分表进仓后不原地覆盖（CLAUDE.md 五.1）；要改切法先改规则、换种子并说明")
    big = len(body) > GZIP_OVER
    keep, drop = (jl.with_name(jl.name + ".gz"), jl) if big else (jl, jl.with_name(jl.name + ".gz"))
    drop.unlink(missing_ok=True)  # 同内容换存法（上面已核过哈希），不会丢东西
    keep.write_bytes(gzip.compress(body, compresslevel=9, mtime=0) if big else body)
    s = summary(name, rule, seed, rows, body)
    js.write_text(json.dumps(s, ensure_ascii=False, indent=1) + "\n")
    return s


def load_split(name, out_dir=OUT):
    """读一张切分表并核摘要里记的哈希；对不上即报错（loader 与 leakcheck 都走这里）。"""
    out_dir = Path(out_dir)
    s = json.loads((out_dir / f"{name}.json").read_text())
    jl, gz = out_dir / f"{name}.jsonl", out_dir / f"{name}.jsonl.gz"
    if jl.exists() == gz.exists():
        raise SystemExit(f"{name}：.jsonl 与 .jsonl.gz 要有且只有一份（现在{'两份都在' if jl.exists() else '都没有'}）")
    body = jl.read_bytes() if jl.exists() else gzip.decompress(gz.read_bytes())
    if sha256_bytes(body) != s["jsonl_sha256"]:
        raise SystemExit(f"{name}.jsonl 和摘要里的哈希对不上：切分表被改过")
    return s, [json.loads(x) for x in body.decode().splitlines()]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True)
    ap.add_argument("--only", default="")
    ap.add_argument("--inventory", action="store_true", help="只列每个来源匹配到的文件数与前 20 个路径，不写切分表")
    ap.add_argument("--out", default=str(OUT))
    ap.add_argument("--cache", default="", help="逐文件读数缓存（默认 <root>/cache/split-files.jsonl；传 none 不用）")
    ap.add_argument("--workers", type=int, default=32)
    a = ap.parse_args()
    R = load_rules()
    names = [n for n in (a.only.split(",") if a.only else R["sources"]) if n]
    from filecache import FileCache

    cache = FileCache(None if a.cache == "none" else (a.cache or Path(a.root) / "cache" / "split-files.jsonl"))
    for n in names:
        rule = R["sources"][n]
        if a.inventory:
            fs = list_files(Path(a.root), rule)
            print(f"{n}：{len(fs)} 个文件", *fs[:20], sep="\n  ")
            continue
        s = write(n, rule, R["seed"], split_source(a.root, n, rule, R["seed"], R["splits"], a.workers, cache), a.out)
        print(json.dumps(s, ensure_ascii=False), flush=True)
    if not a.inventory:
        print(f"读数缓存：复用 {cache.hits} 个，新算 {cache.misses} 个", flush=True)


if __name__ == "__main__":
    main()
