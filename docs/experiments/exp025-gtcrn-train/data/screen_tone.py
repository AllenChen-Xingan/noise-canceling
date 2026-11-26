"""把 diagnose.py 的逐秒单音结论（`--tone`）并进 screen.py 的筛查结果：有单音的片段不进训练。

分工：`screen.py` 判人声 · 标签 · 平稳度，输出一份 jsonl（每条 keep / drop）；
`diagnose.py --tone` 判 250 Hz 单音在哪几秒（全程没有 · 只在开头 · 后面也有），另输出一份 jsonl。
本工具把两份按 `path` 合成一份：`split.py` 只读合成后的这份，就有单音的片段一条都进不了训练。

- 「单音在哪就该丢」只有一个正本：`screen_rules.json` 的 `tone_check.drop_where`。代码里不写那份名单，
  改判据只改 json；名单外的位置（例如只在开头）保持原判定，那一项要改 `split.py`，不在这里做
- 对不上就停：筛查里有一条、单音里查不到 → 停并报出是哪条。悄悄跳过一条，就等于把单音带进训练
- 单音结论本身要自洽：`verdict.where` 只许 `none` / `first_only` / `later_too`，`present_seconds` 必须是整数列表，
  且「没有」⇔ 空列表、「只在第 1 秒」⇔ `[1]`（正本见 `diagnose.tone_verdict`，秒号从 1 起）。
  对不上就停：那说明这份 jsonl 不是 `diagnose.py --tone` 写的，后面的判定无从谈起
- 只动三样：`verdict` · `reasons` · `measures`；其余字段原样带过去，输入列表本身不改写
用法：
    python3 screen_tone.py --screen screen.jsonl --tone tone.jsonl --out screen-tone.jsonl
"""
import argparse
import copy
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RULES = HERE / "screen_rules.json"


# ---------------------------------------------------------------- 判据

def load_drop_where(path=RULES):
    """从判据 json 读「单音在哪几秒就该丢」的名单。没有这一项、或不是非空字符串列表，都停下。"""
    rules = json.loads(Path(path).read_text(encoding="utf-8"))
    try:
        values = rules["tone_check"]["drop_where"]
    except (KeyError, TypeError):
        raise SystemExit(f"{path} 的 tone_check 里没有 drop_where：不知道哪些单音该丢，先补判据")
    if not isinstance(values, list) or not values:
        raise SystemExit(f"{path} 的 tone_check.drop_where 不是非空列表（现在是 {values!r}）")
    for v in values:
        if not isinstance(v, str):
            raise SystemExit(f"{path} 的 tone_check.drop_where 里有非字符串的项：{v!r}")
    return list(values)


# ---------------------------------------------------------------- 读写

def read_jsonl(path):
    """一行一条；空行跳过，读不了的行报出是第几行。"""
    rows = []
    for lineno, line in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as e:
            raise SystemExit(f"{path} 第 {lineno} 行不是 json（{e}）")
        if not isinstance(row, dict):
            raise SystemExit(f"{path} 第 {lineno} 行不是对象（是 {type(row).__name__}）")
        rows.append(row)
    return rows


def write_jsonl(rows, path):
    with open(path, "w", encoding="utf-8") as out:
        for row in rows:
            out.write(json.dumps(row, ensure_ascii=False) + "\n")


def index_by_path(rows, what):
    """path → 行。没有 path 或 path 重复都停下：一条片段只能有一行，重复就不知道听哪行。"""
    out = {}
    for row in rows:
        path = row.get("path")
        if not isinstance(path, str) or not path:
            raise SystemExit(f"{what} 里有行没有 path：{json.dumps(row, ensure_ascii=False)[:200]}")
        if path in out:
            raise SystemExit(f"{what} 里 path 重复：{path}（一条片段只能有一行）")
        out[path] = row
    return out


# ---------------------------------------------------------------- 合并

def _seconds_text(secs):
    """[1, 3] → 「1、3」；整数不带小数点、不带方括号，读给人听。"""
    out = []
    for s in secs:
        if isinstance(s, float) and s.is_integer():
            out.append(str(int(s)))
        else:
            out.append(str(s))
    return "、".join(out)


def tone_reason(tone):
    """单音这一条的理由（写给人看：哪个频率、哪几秒）。"""
    hz = tone.get("hz")
    if not isinstance(hz, (int, float)) or isinstance(hz, bool):
        raise SystemExit(f"单音行缺少 hz，写不出理由：tone = {json.dumps(tone, ensure_ascii=False)[:200]}")
    verdict = tone.get("verdict") or {}
    secs = verdict.get("present_seconds") or []
    return f"有单音：{hz:g} Hz 在第 {_seconds_text(secs)} 秒（diagnose.py --tone）"


WHERE = ("none", "first_only", "later_too")  # 正本：`diagnose.tone_verdict` 只发这三个
NONE, FIRST_ONLY, LATER_TOO = WHERE


def _check_verdict(path, verdict):
    """核 where 与 present_seconds 是不是 `diagnose.tone_verdict` 可能发出的那一对（秒号从 1 起）。

    where 只许那三个值；秒列表必须是整数列表；「没有单音」⇔ 空列表；「只在第 1 秒」⇔ [1]。
    任何一项对不上都停：不自洽的单音行会让判定与理由各说各话，不如先停下重跑。"""
    where, secs = verdict["where"], verdict.get("present_seconds")
    if where not in WHERE:
        raise SystemExit(f"{path} 的单音结论 where = {where!r}，不是 {' / '.join(WHERE)} 之一："
                         "这份单音 jsonl 不是 diagnose.py --tone 写的，先重跑")
    if not isinstance(secs, list) or not all(isinstance(s, int) and not isinstance(s, bool) for s in secs):
        raise SystemExit(f"{path} 的单音结论 where = {where!r}，present_seconds = {secs!r}"
                         " 不是整数列表（秒号从 1 起）：这份单音 jsonl 不是 diagnose.py --tone 写的")
    if (where == NONE) != (not secs):
        raise SystemExit(f"{path} 的单音结论 where = {where!r} 与 present_seconds = {secs!r} 对不上："
                         "where 是 none 时秒列表必须为空，否则必须非空")
    if (where == FIRST_ONLY) != (secs == [1]):
        raise SystemExit(f"{path} 的单音结论 where = {where!r} 与 present_seconds = {secs!r} 对不上："
                         "first_only 必须正好是 [1]")


def _verdict_of(tone_row):
    """取这一条的单音结论并核它自洽；没有 `tone` 键说明 diagnose.py 没加 --tone 就跑过，不能当成「没有单音」。"""
    tone = tone_row.get("tone")
    if not isinstance(tone, dict):
        raise SystemExit(
            f"{tone_row.get('path')} 的单音行里没有 tone 项：这份是 diagnose.py 没加 --tone 跑出来的，"
            "单音根本没查，不能当成没有。重跑一次 diagnose.py --tone")
    verdict = tone.get("verdict")
    if not isinstance(verdict, dict) or not isinstance(verdict.get("where"), str):
        raise SystemExit(
            f"{tone_row.get('path')} 的单音行里没有 verdict.where："
            f"tone = {json.dumps(tone, ensure_ascii=False)[:200]}")
    _check_verdict(tone_row.get("path"), verdict)
    return tone


def merge(screen_rows, tone_rows, drop_where):
    """合成一份筛查 jsonl。返回 {"rows": 新的行列表, "total"/"keep"/"drop"/"tone_dropped": 计数}。

    顺序跟 `screen_rows` 走；`screen_rows` 本身不改写（返回的是副本）。
    """
    index_by_path(screen_rows, "筛查 jsonl（screen.py）")  # 只为了查重复
    tones = index_by_path(tone_rows, "单音 jsonl（diagnose.py --tone）")
    missing = [row["path"] for row in screen_rows if row["path"] not in tones]
    if missing:
        shown = "、".join(missing[:5]) + ("…" if len(missing) > 5 else "")
        raise SystemExit(
            f"单音 jsonl 里查不到这 {len(missing)} 条片段：{shown}"
            "（先跑一遍 diagnose.py --tone；漏一条就会把带单音的片段放进训练）")
    drop_set = set(drop_where)
    rows, keep, drop, tone_dropped = [], 0, 0, 0
    for row in screen_rows:
        tone = _verdict_of(tones[row["path"]])
        where = tone["verdict"]["where"]
        secs = list(tone["verdict"]["present_seconds"])
        new = copy.deepcopy(row)
        measures = new.setdefault("measures", {})
        measures["tone_where"] = where
        measures["tone_present_seconds"] = secs
        if where in drop_set:
            new["verdict"] = "drop"
            new.setdefault("reasons", []).append(tone_reason(tone))
            if row.get("verdict") != "drop":
                tone_dropped += 1
        if new.get("verdict") == "keep":
            keep += 1
        else:
            drop += 1
        rows.append(new)
    return {"rows": rows, "total": len(rows), "keep": keep, "drop": drop,
            "tone_dropped": tone_dropped}


# ---------------------------------------------------------------- 命令行

def summary(res):
    return (f"共 {res['total']} 条：keep {res['keep']} · drop {res['drop']}"
            f"（其中因单音新丢 {res['tone_dropped']} 条）")


def run(args):
    out = Path(args.out)
    for name in ("screen", "tone"):
        if out.resolve() == Path(getattr(args, name)).resolve():
            raise SystemExit(f"--out 指到了 --{name} 上（{out}）：不原地覆盖，换一个文件名")
    drop_where = load_drop_where(args.rules)
    res = merge(read_jsonl(args.screen), read_jsonl(args.tone), drop_where)
    write_jsonl(res["rows"], out)
    print(summary(res))


def main(argv=None):
    ap = argparse.ArgumentParser(
        description="把单音结论并进筛查结果（丢哪些取 screen_rules.json 的 tone_check.drop_where）")
    ap.add_argument("--screen", required=True, help="screen.py 输出的 jsonl")
    ap.add_argument("--tone", required=True, help="diagnose.py --tone 输出的 jsonl")
    ap.add_argument("--out", required=True, help="合成后的 jsonl（不许等于 --screen / --tone）")
    ap.add_argument("--rules", default=str(RULES), help="判据 json（默认同目录的 screen_rules.json）")
    run(ap.parse_args(argv))


if __name__ == "__main__":
    main()
