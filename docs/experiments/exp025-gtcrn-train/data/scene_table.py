"""exp025 · 目标场景噪声表：每个目标场景的训练 / 开发噪声来自哪几个来源、多少条、多少小时、几个单位、什么许可。

正本（这里只读、只拼表，一个数字也不手抄）：
- `builds/<id>/build.json`：配方哈希 · 每张切分表的哈希 · 这个 build 里实际有成员的噪声场景
- `recipe.json`：目标场景（`knobs.target_scene.scene`）与场景 → 噪声池（来源 + 路径正则）
- `split_rules.json`：来源 → 组（`sources.<来源>.group`）
- `sources.json`：组的许可 · 署名 · 能不能训练（`groups.<组>.license / attribution / train_ok`）
- `splits/<来源>.jsonl`：逐文件的份 / 段界 / 单位（摘要里的哈希由 split.load_split 核）
唯一手写输入是 `scene_notes.json`（每个场景的已知限制）：它不进任何 build id。

为什么先核哈希：切分表被改过、配方改过，这张表就不再描述那个 build，把数字报出去会误导人（CLAUDE.md 五.1 那一类坑）。

用法：
  python3 scene_table.py --build builds/10aa3a9fb8cef48f                # 打到标准输出
  python3 scene_table.py --build builds/<id> --out builds/<id>/scenes.md # 写文件（产物不进库）
  python3 scene_table.py --build builds/<id> --json                      # 出 dict（可配合 --out）
"""
import argparse
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import split  # noqa: E402  切分表读取与哈希核对只走它，不另写一份

SPLITS = ("train", "dev")  # 这张表只报这两份；final / d63 不进训练
MD_HEAD = "| 来源 | 份 | 条数 | 时长（小时，2 位小数） | 单位数 | 许可 |"


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def target_scenes(recipe):
    """recipe 里人定的目标场景。null / 空列表 = 没有目标场景可报（不静默报空表）。"""
    sc = recipe["knobs"]["target_scene"]["scene"]
    names = [sc] if isinstance(sc, str) else list(sc or [])
    if not names:
        raise SystemExit("recipe.json 的 knobs.target_scene.scene 是空的（null 或空列表）：没有目标场景可报")
    return names


def load_recipe(build, data_dir):
    """读配方并核 build 记的哈希：改编方而不重登记，build 就变了。"""
    p = Path(data_dir) / "recipe.json"
    body = p.read_bytes()
    got = split.sha256_bytes(body)
    want = build.get("recipe_sha256")
    if got != want:
        raise SystemExit(f"{p} 的 sha256 是 {got}，build {build.get('build_id')} 记的是 {want}："
                         "配方改过而 build 没重登记，这张表就不是这个 build 的")
    return json.loads(body)


def load_table(name, build, data_dir):
    """读一张切分表，核摘要哈希（split.load_split）再核 build 记的哈希；对不上即停。"""
    summary, rows = split.load_split(name, out_dir=Path(data_dir) / "splits")
    want = (build.get("split_tables") or {}).get(name)
    if want is None:
        raise SystemExit(f"build {build.get('build_id')} 的 split_tables 里没有来源 {name}：这个 build 没登记过它，不能用它报数")
    if summary["jsonl_sha256"] != want:
        raise SystemExit(f"切分表 splits/{name}.jsonl 的哈希 {summary['jsonl_sha256']} 和 build "
                         f"{build.get('build_id')} 记的 {want} 对不上：这张表不属于这个 build")
    return rows


def source_meta(name, rules, sources):
    """来源的许可 · 署名 · 能不能训练：来源 → split_rules 的组 → sources.json 的组。"""
    rule = (rules.get("sources") or {}).get(name)
    if rule is None:
        raise SystemExit(f"split_rules.json 里没有来源 {name}：认不出它的组，也就查不到许可")
    gname = rule.get("group")
    g = (sources.get("groups") or {}).get(gname)
    if g is None:
        raise SystemExit(f"sources.json 里没有组 {gname}（来源 {name} 要用）：查不到许可与 train_ok")
    return {"source": name, "group": gname, "license": g.get("license", ""),
            "attribution": g.get("attribution", ""), "train_ok": bool(g.get("train_ok"))}


def row_seconds(row, sp):
    """这一行在这一份的秒数；这一行不在这一份就返回 None。
    按时段切的来源（QUT · DEMAND）没有 split 字段，段界在 segments 里：时长按那一段算；
    其余来源按整文件算（frames / sr）。"""
    seg = row.get("segments")
    if seg and sp in seg:
        a, b = seg[sp]
        return (b - a) / row["sr"]
    if row.get("split") == sp:
        return row["frames"] / row["sr"]
    return None


def scene_pools(recipe):
    """场景 → [(来源, 编译好的正则)]。recipe 里 `_` 开头的键是说明，不是场景。"""
    pools = {}
    for scene, items in recipe["noise_pools"].items():
        if scene.startswith("_"):
            continue
        pools[scene] = sorted(((it["source"], re.compile(it["regex"])) for it in items), key=lambda x: (x[0], x[1].pattern))
    return pools


def split_table(rows_by_source, items, sp):
    """一个场景一份的每个来源：条数 · 秒数 · 单位数（正则用 re.search 匹配切分表里的 path，和 loader 一致）。"""
    per = {}
    for name, rx in items:
        d = per.get(name)
        if d is None:
            d = per[name] = {"files": 0, "seconds": 0.0, "units": set()}
        for row in rows_by_source(name):
            if not rx.search(row["path"]):
                continue
            sec = row_seconds(row, sp)
            if sec is None:
                continue
            d["files"] += 1
            d["seconds"] += sec
            d["units"].add(row["unit"])
    return per


def as_split(per, metas):
    """整理成可写 JSON / markdown 的形状（排序、四舍五入、单位和元数据）。"""
    out = []
    for name in sorted(per):
        d = per[name]
        out.append({**metas[name], "files": d["files"], "seconds": round(d["seconds"], 2), "units": len(d["units"])})
    totals = {"files": sum(x["files"] for x in out), "seconds": round(sum(x["seconds"] for x in out), 2),
              "units": sum(x["units"] for x in out)}
    return {"sources": out, "totals": totals}


def build_tables(build_dir, data_dir=HERE, scenes=None):
    """按一个训练 build 报每个目标场景的噪声来源表。scenes 不给就用 recipe 里人定的那几个。"""
    build_dir, data_dir = Path(build_dir), Path(data_dir)
    build = load_json(build_dir / "build.json")
    recipe = load_recipe(build, data_dir)
    rules = split.load_rules(Path(data_dir) / "split_rules.json")
    sources = load_json(data_dir / "sources.json")
    notes = {k: v for k, v in load_json(data_dir / "scene_notes.json").items() if not k.startswith("_")}
    pools = scene_pools(recipe)
    want = list(scenes) if scenes else target_scenes(recipe)
    cache, metas = {}, {}

    def rows_by_source(name):
        if name not in cache:
            cache[name] = load_table(name, build, data_dir)
            metas[name] = source_meta(name, rules, sources)
        return cache[name]

    train_scenes = set((build.get("noise_scenes") or {}).get("train", []))
    out = {"build_id": build.get("build_id"), "recipe_sha256": build["recipe_sha256"],
           "build_dir": str(build_dir), "scenes": {}}
    for scene in want:
        if scene not in pools:
            raise SystemExit(f"场景 {scene} 在 recipe.json 的 noise_pools 里没有噪声池：先给它配来源与路径正则")
        items = pools[scene]
        for name, _ in items:  # 先把元数据取齐：train_ok 为假时一行也不报
            rows_by_source(name)
            if not metas[name]["train_ok"]:
                raise SystemExit(f"来源 {name}（组 {metas[name]['group']}）在 sources.json 里 train_ok 为假"
                                 f"（只评测、不进训练）：场景 {scene} 的噪声池不能训练")
        if scene not in notes or not str(notes[scene]).strip():
            raise SystemExit(f"target 场景 {scene} 在 scene_notes.json 里没有已知限制：表里不许空着这一行，请写上")
        splits = {}
        for sp in SPLITS:
            splits[sp] = as_split(split_table(rows_by_source, items, sp), metas)
        n_train = splits["train"]["totals"]["files"]
        if n_train == 0:
            raise SystemExit(f"场景 {scene} 在这个 build 里没有训练噪声（切分表的训练份一条也匹配不上）："
                             "先补数据或改噪声池正则，再登记 build")
        if scene not in train_scenes:
            raise SystemExit(f"场景 {scene} 有 {n_train} 条训练噪声，但 build {build.get('build_id')} 的 "
                             f"noise_scenes['train'] 里没有它：分片漏了这个场景，先重跑 shard.py")
        out["scenes"][scene] = {"notes": str(notes[scene]).strip(), "splits": splits}
    return out


def render_md(tables):
    """中文 markdown：每个场景一张表，表后一行已知限制。"""
    lines = [f"# 目标场景噪声表 · build {tables['build_id']}", "",
             f"配方 sha256（前 12 位）：{tables['recipe_sha256'][:12]}。"
             "数字由 `scene_table.py` 从 build 记录与切分表算出，不手抄；时长按这一份实际用的那一段算。", ""]
    for scene, s in tables["scenes"].items():
        lines += [f"## {scene}", "", MD_HEAD, "|---|---|---|---|---|---|"]
        for sp in SPLITS:
            for r in s["splits"][sp]["sources"]:
                lines.append(f"| {r['source']} | {sp} | {r['files']} | {r['seconds'] / 3600:.2f} | {r['units']} | {r['license']} |")
            tt = s["splits"][sp]["totals"]
            lines.append(f"| 合计 | {sp} | {tt['files']} | {tt['seconds'] / 3600:.2f} | {tt['units']} | |")
        lines += ["", f"已知限制：{s['notes']}", ""]
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description="按训练 build 报每个目标场景的噪声来源表")
    ap.add_argument("--build", required=True, help="builds/<id> 目录")
    ap.add_argument("--out", default="", help="写到文件（产物不进版本库）；不给就打标准输出")
    ap.add_argument("--json", action="store_true", help="出 dict 而不是 markdown")
    a = ap.parse_args()
    tables = build_tables(a.build)
    text = json.dumps(tables, ensure_ascii=False, indent=1) + "\n" if a.json else render_md(tables)
    if a.out:
        Path(a.out).parent.mkdir(parents=True, exist_ok=True)
        Path(a.out).write_text(text, encoding="utf-8")
        print(f"写到 {a.out}", file=sys.stderr)
    else:
        print(text, end="")


if __name__ == "__main__":
    main()
