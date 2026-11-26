"""exp025 · 训练前清单第 3 条的证据：每份配方（基准 + 5 份消融）都对上一份 build 记录，且那份记录逐项自洽。

正本与来源（这里只读、只核，一个数字也不手抄）：
- `recipe.json` 与 `recipes/ablation/*.json`：配方；消融清单的唯一出处是 `ablation_recipes.VARIANTS`
- `builds/<id>/build.json`：build 记录（build_id · 配方哈希 · 分片 · 切分表哈希 · 代码哈希 · 交集检查）
- `splits/<来源>.jsonl`：切分表正文，哈希只认 `split.load_split`
- 交集检查：只认 `leakcheck.check` 重算的那一遍（按 build 记录里列出的来源重跑），不读记录里的结论
- `leakcheck.EXCLUDE`：排外表的正本；报告里报它的 sha256 前 12 位，说明重算用的是哪份表
- 本地代码文件：`loader.CODE` 与 `loader.EXP013_CODE`（形状同 `loader.build_info` 记的 `code_sha256`）
- `recipe.json` 的目标场景：只认 `scene_table.target_scenes`

每份配方找它自己的 build：配方字节的 sha256 相同，**并且**分片与基准 build 的分片相同
（基准 build = 配 `recipe.json` 的那一份；分片相同才说明这批消融是在同一批数据上跑的）。
任何一份配方 0 个或多于 1 个候选都停，并把是哪个配方、哪几个 build 写进报错。

然后对那份 build 逐项核，每项一个布尔列：
build_id 可重算 · valid · 交集检查通过 · 命中 0 · DEMAND 命中 0 · 交集重算一致 · missing 空 ·
切分表哈希与仓里一致 · 代码哈希与本地一致 · 目标场景都有训练噪声 · 变体只差一个旋钮（`knobs.<knob>.on` + `_ablation`）。
「交集重算一致」不是读记录：按记录里列的来源重跑 `leakcheck.check`，重算的 `ok` 为真、`n_hits` 为 0、
且重算的 `per_key` 与记录逐键相同，这一列才是「是」。重算按「来源列表 + 切分表哈希」缓存（6 个 build 共用一份，只跑一遍）。

为什么生成而不手写：这是训练前清单上要用的一张证据表；手抄一遍就多一份会过期、又没法机器核的副本。

用法（`loader` 要 numpy，用与 `cloud_steps.sh setup` 建的那个 python）：
  python3 build_evidence.py                          # 打到标准输出
  python3 build_evidence.py --out evidence-<id>.md   # 写文件
  python3 build_evidence.py --check evidence-<id>.md # 与提交进仓的那份比；不同或本身不过就退 1
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ablation_recipes  # noqa: E402  消融清单（名字 · 旋钮 · 目标值）的唯一出处
import leakcheck  # noqa: E402  交集检查的重算只走它（结论不读记录）
import loader  # noqa: E402  build_id 重算与 code_sha256 的形状只认它
import scene_table  # noqa: E402  目标场景只认它
import split  # noqa: E402  切分表读取与哈希核对只走它

BASE = "recipe.json"
ABLATION = Path("recipes") / "ablation"
COLUMNS = (  # 每项检查一列：键 → 表头（中文，短）
    ("id", "id 可重算"),
    ("valid", "valid"),
    ("leak_ok", "交集通过"),
    ("leak_hits", "命中 0"),
    ("leak_demand", "DEMAND 0"),
    ("leak_recheck", "交集重算一致"),
    ("missing", "missing 空"),
    ("split_tables", "切分表哈希"),
    ("code", "代码哈希"),
    ("scenes", "目标场景（build 记录）"),
    ("one_knob", "只差一个旋钮"),
)


def sha_file(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def recipe_files(data_dir=HERE):
    """[(名字, 路径)]：先是基准配方，再是 5 份消融配方（顺序照 `ablation_recipes.VARIANTS`）。

    目录里多出一份不在清单里的 json 就停：那种文件没有旋钮归属，核不了「只差一个旋钮」，也不该悄悄漏掉。
    """
    data_dir = Path(data_dir)
    out = [(BASE, data_dir / BASE)]
    for name, _knob, _on in ablation_recipes.VARIANTS:
        out.append((f"{ABLATION}/{name}.json", data_dir / ABLATION / f"{name}.json"))
    want = {p.name for _n, p in out[1:]}
    have = {p.name for p in (data_dir / ABLATION).glob("*.json")}
    if have - want:
        raise SystemExit(f"{data_dir / ABLATION} 里有清单（ablation_recipes.VARIANTS）没登记的配方：{sorted(have - want)}")
    for name, path in out:
        if not path.is_file():
            raise SystemExit(f"缺配方 {name}：{path}")
    return out


def all_builds(builds_dir):
    """{build_id: build 记录}——build 记录只在 `builds/<id>/build.json`。"""
    builds_dir = Path(builds_dir)
    return {p.parent.name: load_json(p) for p in sorted(builds_dir.glob("*/build.json"))}


def one_build(hits, label, sha, why):
    """候选数必须是 1（0 个 = 没跑、2 个以上 = 分不清是哪一份）：两种都停。"""
    if len(hits) == 1:
        return hits[0]
    ids = sorted(b.get("build_id", "?") for b in hits)
    raise SystemExit(f"{label}：按{why}（配方 {sha[:12]}…）应有恰好 1 个 build，现在 {len(hits)} 个：{ids}。"
                     "0 个 = 云上还没登记；多于 1 个 = 分不清是哪一份，先查 builds/")


def match_builds(recipes, builds):
    """{配方名: build 记录}。基准先按配方哈希认（分片以它为准），其余还要分片与它相同。"""
    base_name, base_path = recipes[0]
    base_sha = sha_file(base_path)
    base = one_build([b for b in builds.values() if b.get("recipe_sha256") == base_sha],
                     base_name, base_sha, "配方哈希")
    out = {base_name: base}
    for name, path in recipes[1:]:
        sha = sha_file(path)
        hits = [b for b in builds.values() if b.get("recipe_sha256") == sha and b.get("shards") == base.get("shards")]
        out[name] = one_build(hits, name, sha, "配方哈希 + 基准 build 的分片")
    return out


def code_hashes():
    """本地代码文件的哈希，形状与 `loader.build_info` 记的 `code_sha256` 相同（`loader.CODE` + `exp013/`）。
    代码就在本目录（`loader.build_info` 也是这么算的），不跟 `--data-dir` 走。"""
    return {**{f: sha_file(HERE / f) for f in loader.CODE},
            **{f"exp013/{f}": sha_file(loader.EXP013 / f) for f in loader.EXP013_CODE}}


def recipe_delta(base, other, path=()):
    """两份配方不同的叶子路径（`_ablation` 整块不算：它是变体自己的记账，由 `one_knob` 单独核）。"""
    out = set()
    if isinstance(base, dict) and isinstance(other, dict):
        for k in set(base) | set(other):
            if k == "_ablation" and not path:
                continue
            p = path + (k,)
            if k in base and k in other:
                out |= recipe_delta(base[k], other[k], p)
            else:
                out.add(".".join(p))
        return out
    if base != other:
        out.add(".".join(path) or "（整份）")
    return out


def one_knob(recipe, base, base_sha, knob=None):
    """这份配方是不是「基准 + 一个旋钮」：(bool, 原因)。

    knob=None 是基准：整份必须与 `recipe.json` 相同、且没有 `_ablation`；否则变体只许差 `knobs.<knob>.on`，
    且 `_ablation` 记的旋钮名 · 目标值 · 基准哈希都要对得上。"""
    delta = recipe_delta(base, recipe)
    ab = recipe.get("_ablation")
    if knob is None:
        if ab is not None:
            return False, "基准配方不该有 `_ablation`"
        return (not delta), ("与 recipe.json 相同" if not delta else f"与 recipe.json 不同：{sorted(delta)}")
    if not isinstance(ab, dict) or set(ab) != {"base_sha256", "knob", "on"}:
        return False, f"`_ablation` 缺字段或多了字段：{ab!r}"
    if ab["knob"] != knob:
        return False, f"`_ablation.knob` = {ab['knob']}，清单里这一份是 {knob}"
    want = {f"knobs.{knob}.on"}
    if delta != want:
        extra = sorted(delta - want)
        return False, (f"除了 {', '.join(want)} 又改了：{extra}" if extra
                       else f"和基准一模一样：{knob} 没翻，`_ablation` 却说翻了")
    cur_on = ((recipe.get("knobs") or {}).get(knob) or {}).get("on")
    base_on = ((base.get("knobs") or {}).get(knob) or {}).get("on")
    if cur_on != ab["on"] or base_on == ab["on"]:
        return False, f"旋钮 {knob} 的开关（{cur_on}）与 `_ablation.on` = {ab['on']} 对不上"
    if ab["base_sha256"] != base_sha:
        return False, "`_ablation.base_sha256` 不是仓里 recipe.json 的哈希"
    return True, f"只翻 knobs.{knob}.on → {ab['on']}"


def leak_recheck(build, splits_dir, cache=None):
    """按 build 记录里列的来源重跑一遍 `leakcheck.check`（切分表只认 `splits_dir`）。

    结果按「来源列表 + 切分表哈希」缓存：同一批消融的 6 个 build 共用一份切分表，只该解析一遍。"""
    sources = list(build.get("leak_checked_sources") or [])
    key = (tuple(sources), tuple(sorted((build.get("split_tables") or {}).items())))
    cache = {} if cache is None else cache
    if key not in cache:
        try:
            cache[key] = leakcheck.check({s: split.load_split(s, splits_dir)[1] for s in sources})
        except FileNotFoundError as e:
            raise SystemExit(f"读不了排外表 {leakcheck.EXCLUDE}（{e}）：交集重算不了，不能当成通过")
    return cache[key]


def build_checks(build, recipe, base_recipe, base_sha, code, splits_dir, knob=None, leak_cache=None):
    """{列键: (bool, 原因)}——每一列都是一句能读懂的判断，原因在通过时也写（报告里只列不通过的）。"""
    bid = build.get("build_id")
    lk = build.get("leak_check") or {}
    per = lk.get("per_key") or {}
    out = {}

    parts = {k: build[k] for k in loader.ID_KEYS if k in build}
    got = loader.build_id_of(parts)
    lack = sorted(set(loader.ID_KEYS) - set(parts))
    out["id"] = (got == bid, f"身份字段重算得 {got}" + ("" if got == bid else f" ≠ 记录里的 {bid}")
                 + (f"（记录缺 {lack}，缺的字段不进哈希）" if lack else ""))

    out["valid"] = (build.get("valid") is True, "build 记作废" if build.get("valid") is not True else "记录记 valid")
    out["leak_ok"] = (lk.get("ok") is True, "交集检查没通过" if lk.get("ok") is not True else f"对照 {lk.get('against', '（没记）')}")
    out["leak_hits"] = (lk.get("n_hits") == 0, f"交集检查命中 {lk.get('n_hits')} 条")
    out["leak_demand"] = (per.get("demand_segments") == 0, f"DEMAND 评测段命中 {per.get('demand_segments')} 条")

    rec = leak_recheck(build, splits_dir, leak_cache)
    rec_per = rec.get("per_key") or {}
    diff = sorted(k for k in set(rec_per) | set(per) if rec_per.get(k) != per.get(k))
    same = rec.get("ok") is True and rec.get("n_hits") == 0 and not diff
    out["leak_recheck"] = (same, "重算得 ok=True · 命中 0 · per_key 与记录逐键相同" if same else
                           f"重算得 ok={rec.get('ok')} · 命中 {rec.get('n_hits')} 条"
                           + (f"；per_key 与记录不同：{[(k, per.get(k), rec_per.get(k)) for k in diff[:5]]}"
                              if diff else ""))

    miss = build.get("missing")
    out["missing"] = (miss == [], f"missing = {miss!r}（这一份没有成员的来源，本不该有）")

    tables = build.get("split_tables") or {}
    bad = []
    for name, want in sorted(tables.items()):
        try:
            summary, _rows = split.load_split(name, splits_dir)
        except (SystemExit, OSError) as e:  # 表没了或被改过：这一列就是「否」，不中断整张表
            bad.append(f"{name}（{e}）")
            continue
        if summary["jsonl_sha256"] != want:
            bad.append(f"{name}（表 {summary['jsonl_sha256'][:12]}… ≠ 记的 {want[:12]}…）")
    if not tables:
        bad.append("记录里一张切分表也没有")
    out["split_tables"] = (not bad, "；".join(bad) if bad else f"{len(tables)} 张表逐张核过")

    out["code"] = (build.get("code_sha256") == code,
                   "代码哈希与本地文件不同：build 之后改过 loader.CODE 或 exp013 的模块"
                   if build.get("code_sha256") != code else f"与本地 {len(code)} 个文件一致")

    have = set((build.get("noise_scenes") or {}).get("train") or [])
    want_scenes = scene_table.target_scenes(recipe)
    lost = [s for s in want_scenes if s not in have]
    out["scenes"] = (not lost, f"目标场景 {lost} 在这个 build 的训练份里没有噪声" if lost
                     else f"{len(want_scenes)} 个目标场景都有训练噪声")

    ok, why = one_knob(recipe, base_recipe, base_sha, knob)
    out["one_knob"] = (ok, why)
    return out


def report(data_dir=HERE):
    """(markdown, 每一行的 build 与检查)。data_dir 下有 recipe.json · recipes/ · builds/ · splits/（测试用临时目录也走这里）。"""
    data_dir = Path(data_dir)
    recipes = recipe_files(data_dir)
    builds = all_builds(data_dir / "builds")
    if not builds:
        raise SystemExit(f"{data_dir / 'builds'} 里一个 build 记录也没有")
    matched = match_builds(recipes, builds)
    base_recipe = load_json(recipes[0][1])
    base_sha = sha_file(recipes[0][1])
    code = code_hashes()
    knobs = {f"{ABLATION}/{name}.json": knob for name, knob, _on in ablation_recipes.VARIANTS}

    rows = []
    leak_cache = {}  # 同一批消融共用一份切分表：交集检查只重算一遍
    for name, path in recipes:
        recipe = load_json(path)
        checks = build_checks(matched[name], recipe, base_recipe, base_sha, code, data_dir / "splits",
                              knobs.get(name), leak_cache)
        rows.append({"recipe": name, "build": matched[name], "checks": checks})
    text, _ok = render(rows)
    return text, rows


def render(rows):
    """(markdown, 是否全部通过)：一行一个 build，11 个布尔列全「是」才算过；不通过的项在表后逐条写原因。"""
    head = ["build", "配方", "配方哈希（前 12 位）", "交集检查的来源数", "排除表"] + [h for _k, h in COLUMNS]
    ex_sha = sha_file(leakcheck.EXCLUDE)[:12]  # 这次重算实际用的那份 exclude.json
    lines = [f"# 训练前清单第 3 条的证据 · 基准 build {rows[0]['build'].get('build_id')}", "",
             "本文件由 `build_evidence.py` 从 `builds/<id>/build.json` · 配方文件 · 切分表算出（重算即得，不手抄）："
             "`python3 build_evidence.py` 打标准输出，`--check <本文件>` 逐字节核对。"
             "「排除表」一列先是 build 记录里 `leak_check.against` 的名字，后面是本机 `leakcheck.EXCLUDE` 这份 "
             f"exclude.json 的 sha256 前 12 位（`{ex_sha}`）；build 记录不存哈希，所以它只说明这次重算用的是哪份表。"
             "「交集重算一致」一列按记录里列出的来源重跑一遍 `leakcheck.check`，再与记录的那份逐键比。"
             "「目标场景（build 记录）」一列读的是 build 记录里的 `noise_scenes`：分片索引在云上，本机只能核记录。",
             "", "| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    for r in rows:
        b = r["build"]
        against = (b.get("leak_check") or {}).get("against") or "（没记）"
        cells = [b.get("build_id", "?"), r["recipe"], (b.get("recipe_sha256") or "")[:12],
                 str(len(b.get("leak_checked_sources") or [])), f"{against}（本机这份 sha256 {ex_sha}）"]
        cells += ["是" if r["checks"][k][0] else "**否**" for k, _h in COLUMNS]
        lines.append("| " + " | ".join(cells) + " |")
    ok = all(all(v[0] for v in r["checks"].values()) for r in rows)
    lines += ["", f"全部通过：{'是' if ok else '否'}"]
    if not ok:
        lines += ["", "不通过的项："]
        for r in rows:
            for k, _h in COLUMNS:
                good, why = r["checks"][k]
                if not good:
                    lines.append(f"- {r['build'].get('build_id')} · {r['recipe']} · {k}：{why}")
    return "\n".join(lines) + "\n", ok


def main(argv=None):
    ap = argparse.ArgumentParser(description="训练前清单第 3 条的证据：配方 ↔ build 记录逐项核")
    ap.add_argument("--check", type=Path, default=None, help="与提交进仓的这一份比；不同或本身不过就退 1")
    ap.add_argument("--out", type=Path, default=None, help="写到文件；不给就打标准输出")
    ap.add_argument("--data-dir", type=Path, default=HERE, help="数据目录（默认本目录）")
    a = ap.parse_args(argv)
    text, rows = report(a.data_dir)
    ok = all(all(v[0] for v in r["checks"].values()) for r in rows)
    if a.check is not None:
        if not a.check.is_file():
            print(f"没有 {a.check}：先跑 python3 build_evidence.py --out {a.check}", file=sys.stderr)
            return 1
        want = a.check.read_text(encoding="utf-8")
        for i, (x, y) in enumerate(zip(want.splitlines(), text.splitlines()), 1):
            if x != y:
                print(f"{a.check} 与重算结果不同（第 {i} 行）：重跑 python3 build_evidence.py --out {a.check}", file=sys.stderr)
                print(f"  文件里：{x}\n  重算得：{y}", file=sys.stderr)
                return 1
        if want != text:
            print(f"{a.check} 与重算结果不同（行数 {len(want.splitlines())} vs {len(text.splitlines())}）："
                  f"重跑 python3 build_evidence.py --out {a.check}", file=sys.stderr)
            return 1
        print(f"{a.check} 与重算结果逐字节一致（{len(rows)} 行 build）")
    elif a.out is not None:
        a.out.parent.mkdir(parents=True, exist_ok=True)
        a.out.write_text(text, encoding="utf-8")
        print(f"写到 {a.out}", file=sys.stderr)
    else:
        print(text, end="")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
