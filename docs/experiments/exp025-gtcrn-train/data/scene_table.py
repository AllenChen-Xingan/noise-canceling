"""exp025 · 目标场景噪声表：每个目标场景的训练 / 开发噪声来自哪几个来源、多少条、多少小时、几个单位、什么许可。

正本（这里只读、只拼表，一个数字也不手抄）：
- `builds/<id>/build.json`：配方哈希 · 每张切分表的哈希 · 这个 build 里实际有成员的噪声场景
- `recipe.json`：目标场景（`knobs.target_scene.scene`）与场景 → 噪声池（来源 + 路径正则）
- `split_rules.json`：来源 → 组（`sources.<来源>.group`）
- `sources.json`：组的许可 · 署名 · 能不能训练（`groups.<组>.license / attribution / train_ok`）
- `splits/<来源>.jsonl`：逐文件的份 / 段界 / 单位（摘要里的哈希由 split.load_split 核）

为什么还要报「不同内容」时长：同一段声音被多个麦克风 / 声道同时录下时，按条数相加会夸大（DEMAND 是 16 声道阵列，BUT 一次布置最多 31 个麦克风）。哪些文件是同一段声音记在 `scene_notes.json` 的 `same_content`（也不进任何 build id）；没登记的来源，每个文件都算不同内容。

唯一手写输入是 `scene_notes.json`（每个场景的已知限制 + 哪些文件是同一段声音）：它不进任何 build id。

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
MD_HEAD = "| 来源 | 份 | 条数 | 时长（小时，按条数） | 不同内容（小时） | 单位数 | 许可 |"
MD_NOTE = ("两段时长：「时长（小时，按条数）」把每一份文件都加起来；「不同内容（小时）」把同一次录音的多个麦克风 / 声道"
           "算成一段（取组里最长的那一个）。哪些文件算同一段声音记在 `scene_notes.json` 的 `same_content`，"
           "没登记的来源两者相等。")


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
    """读一张切分表，核摘要哈希（split.load_split）再核 build 记的哈希；对不上即停。
    先查 build 有没有登记这个来源，再读文件：表根本没造出来（音频还在云上）时要报「这个 build 没登记过它」，
    不是一句 FileNotFoundError。"""
    want = (build.get("split_tables") or {}).get(name)
    if want is None:
        raise SystemExit(f"build {build.get('build_id')} 的 split_tables 里没登记来源 {name}：这个 build 没切过它，不能用它报数")
    summary, rows = split.load_split(name, out_dir=Path(data_dir) / "splits")
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


def scene_notes(data_dir):
    """读 scene_notes.json，分两堆：场景 → 已知限制（表里贴的那行）· 来源 → 同一段声音的识别规则。
    `same_content` 那一节不是场景，不能被当成场景的已知限制（少一行限制就查不到）。"""
    raw = load_json(Path(data_dir) / "scene_notes.json")
    notes = {k: v for k, v in raw.items() if not k.startswith("_") and k != "same_content"}
    return notes, same_content_rules(raw.get("same_content") or {})


def same_content_rules(raw):
    """来源 → {rx, group}：同一段声音的组键按 regex 匹配路径、再用 group 展开（写法同 split_rules 的 unit_regex）。
    没列出的来源不在返回值里 = 每个文件都是不同内容；`_` 开头的键是说明。"""
    out = {}
    for name, r in raw.items():
        if name.startswith("_"):
            continue
        if not r.get("regex") or not r.get("group"):
            raise SystemExit(f"scene_notes.json 的 same_content.{name} 缺 regex 或 group（两样都要）："
                             "认不出哪些文件是同一次录音，就报不了「不同内容」时长")
        out[name] = {"rx": re.compile(r["regex"]), "group": r["group"]}
    return out


def content_key(name, path, same_rules):
    """这个文件的「同一段声音」组键：组键相同 = 同一次录音。没登记的来源按路径各算一段。"""
    rule = same_rules.get(name)
    if rule is None:
        return path
    m = rule["rx"].search(path)
    if m is None:
        raise SystemExit(f"来源 {name} 在 scene_notes.json 的 same_content 里登记了正则 {rule['rx'].pattern}，"
                         f"但它匹配不上这个路径：{path}。认不出哪些文件是同一次录音，就不报「不同内容」"
                         "（不静默退回按文件算：那样会把夸大后的时长当成真的）")
    try:
        return m.expand(rule["group"])
    except (re.error, IndexError) as e:
        raise SystemExit(f"scene_notes.json 的 same_content.{name} 的 group {rule['group']!r} 展开失败（{e}）："
                         f"它要引用正则 {rule['rx'].pattern} 里的分组，检查一下分组个数")


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


def split_table(rows_by_source, items, sp, same_rules=None):
    """一个场景一份的每个来源：条数 · 秒数 · 不同内容的秒数 · 单位数。
    正则用 re.search 匹配切分表里的 path，和 loader 一致；不同内容按「同一次录音」分组后取每组最长的那一个
    （同一次录音同一段应该一样长，取最大是安全的选择）。"""
    same_rules = same_rules or {}
    per = {}
    for name, rx in items:
        d = per.get(name)
        if d is None:
            d = per[name] = {"files": 0, "seconds": 0.0, "units": set(), "groups": {}}
        for row in rows_by_source(name):
            if not rx.search(row["path"]):
                continue
            sec = row_seconds(row, sp)
            if sec is None:
                continue
            d["files"] += 1
            d["seconds"] += sec
            d["units"].add(row["unit"])
            key = content_key(name, row["path"], same_rules)
            if key not in d["groups"] or sec > d["groups"][key]:
                d["groups"][key] = sec
    return per


def as_split(per, metas):
    """整理成可写 JSON / markdown 的形状（排序、四舍五入、单位和元数据）。"""
    out = []
    for name in sorted(per):
        d = per[name]
        out.append({**metas[name], "files": d["files"], "seconds": round(d["seconds"], 2),
                    "unique_seconds": round(sum(d["groups"].values()), 2), "units": len(d["units"])})
    totals = {"files": sum(x["files"] for x in out), "seconds": round(sum(x["seconds"] for x in out), 2),
              "unique_seconds": round(sum(x["unique_seconds"] for x in out), 2),
              "units": sum(x["units"] for x in out)}
    return {"sources": out, "totals": totals}


def build_tables(build_dir, data_dir=HERE, scenes=None):
    """按一个训练 build 报每个目标场景的噪声来源表。scenes 不给就用 recipe 里人定的那几个。"""
    build_dir, data_dir = Path(build_dir), Path(data_dir)
    build = load_json(build_dir / "build.json")
    recipe = load_recipe(build, data_dir)
    rules = split.load_rules(Path(data_dir) / "split_rules.json")
    sources = load_json(data_dir / "sources.json")
    notes, same_rules = scene_notes(data_dir)
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
            try:
                rows_by_source(name)
            except SystemExit as e:  # 报错要说清是哪个场景用的来源：不然不知道哪张表缺
                raise SystemExit(f"场景 {scene}：{e}") from None
            if not metas[name]["train_ok"]:
                raise SystemExit(f"来源 {name}（组 {metas[name]['group']}）在 sources.json 里 train_ok 为假"
                                 f"（只评测、不进训练）：场景 {scene} 的噪声池不能训练")
        if scene not in notes or not str(notes[scene]).strip():
            raise SystemExit(f"target 场景 {scene} 在 scene_notes.json 里没有已知限制：表里不许空着这一行，请写上")
        splits = {}
        for sp in SPLITS:
            splits[sp] = as_split(split_table(rows_by_source, items, sp, same_rules), metas)
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
             "数字由 `scene_table.py` 从 build 记录与切分表算出，不手抄；时长按这一份实际用的那一段算。", MD_NOTE, ""]
    for scene, s in tables["scenes"].items():
        lines += [f"## {scene}", "", MD_HEAD, "|---|---|---|---|---|---|---|"]
        for sp in SPLITS:
            for r in s["splits"][sp]["sources"]:
                lines.append(f"| {r['source']} | {sp} | {r['files']} | {r['seconds'] / 3600:.2f} | "
                             f"{r['unique_seconds'] / 3600:.2f} | {r['units']} | {r['license']} |")
            tt = s["splits"][sp]["totals"]
            lines.append(f"| 合计 | {sp} | {tt['files']} | {tt['seconds'] / 3600:.2f} | "
                         f"{tt['unique_seconds'] / 3600:.2f} | {tt['units']} | |")
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
