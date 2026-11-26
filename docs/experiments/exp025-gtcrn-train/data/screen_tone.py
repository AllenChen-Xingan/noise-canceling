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
- 结论还要经得起重算：`tone.per_second` 的逐秒条数要等于片段秒数（`diagnose.py` 逐秒切块只丢不足 1 秒的尾巴；
  写下的 `seconds` 只精确到毫秒，所以条数是真长度取整，可以差半个最末位——159999 采样 16 kHz 就是 9 条对 10.0 秒）、
  每秒的 `prominence_db` 都是有限数，再用 `diagnose.tone_verdict(逐秒条数, present_db)` 重算一遍，
  结果必须与存的那个结论逐字相同。重算用的 `present_db` 与 `drop_where` 一样取自 `screen_rules.json`
  的 `tone_check`，代码里不写这个阈值。对不上就停：结论与逐秒数据各说各话，不如先重跑
- 两份证据说的是同一段：筛查行写了片段长度（`seconds`，或 `measures` 里的）就要与单音行的 `seconds`
  差不超过一个毫秒位，否则停下。筛查行没写长度就跳过（没有第二处表示可比）
- 只动三样：`verdict` · `reasons` · `measures`；其余字段原样带过去，输入列表本身不改写
- 补测老来源时只并一段：`--only-regex` + `--base` 只把匹配的行换成合并结果，其余行**原样带过**（第五批 Room112 补单音检查
  就是这种：`screen/but-reverbdb-silence.jsonl` 的 248 条 Room112 换成并了单音结论的，其余 1022 条一个字节不动）。
  「原样」是逐字节：不匹配的行按 base 文件里那一行的原文写回，不重新序列化（空格 · 键序 · `\\uXXXX` 转义都不动）。
  路径对不上就停：悄悄跳过一行，等于那份筛查结论没并进去
- `--check FILE` 用同一份写法重算一遍再与 FILE 逐字节比（不先写盘）；不一样就退非零
用法：
    python3 screen_tone.py --screen screen.jsonl --tone tone.jsonl --out screen-tone.jsonl
    python3 screen_tone.py --base old.jsonl --screen old.jsonl --tone tone.jsonl \
        --only-regex '/Hotel_SkalskyDvur_Room112/' --out new.jsonl
    python3 screen_tone.py ... --only-regex ... --check new.jsonl   # 只核对，不写盘
"""
import argparse
import copy
import json
import math
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RULES = HERE / "screen_rules.json"
sys.path.insert(0, str(HERE))
import diagnose  # noqa: E402  逐秒单音的结论只认它：tone_verdict 是「哪几秒算有」的唯一正本


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


def load_present_db(path=RULES):
    """从判据 json 读「单音比参考带高出多少 dB 才算在」。没有这一项、或不是有限数，都停下。"""
    rules = json.loads(Path(path).read_text(encoding="utf-8"))
    try:
        value = rules["tone_check"]["present_db"]
    except (KeyError, TypeError):
        raise SystemExit(f"{path} 的 tone_check 里没有 present_db：不知道多强才算有单音，先补判据")
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
        raise SystemExit(f"{path} 的 tone_check.present_db 不是有限数（现在是 {value!r}）")
    return value


# ---------------------------------------------------------------- 读写

def read_jsonl_lines(path):
    """一行一条，连同每行原文（不含行尾换行）：`--only-regex` 的「其余行原样带过」要用原文。

    重新序列化一遍会改掉 JSON 空格、键序、`\\uXXXX` 转义（第五批 codex 第 1 轮 #1），所以原文单独留着。
    空行跳过：它本来就不是一条结论。"""
    out = []
    for lineno, line in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
        text = line.strip()
        if not text:
            continue
        try:
            row = json.loads(text)
        except json.JSONDecodeError as e:
            raise SystemExit(f"{path} 第 {lineno} 行不是 json（{e}）")
        if not isinstance(row, dict):
            raise SystemExit(f"{path} 第 {lineno} 行不是对象（是 {type(row).__name__}）")
        out.append((row, line))
    return out


def read_jsonl(path):
    """一行一条；空行跳过，读不了的行报出是第几行（原文不要时用这一条）。"""
    return [row for row, _raw in read_jsonl_lines(path)]


def jsonl_text(rows):
    """行列表 → jsonl 正文。写盘与 `--check` 共用这一份写法（否则核对的是另一种序列化）。

    项是 `(row, 原文)`（`splice` 里不匹配 `--only-regex` 的那些）时按原文写回，不再序列化：JSON 空格、键序、
    `\\uXXXX` 转义都不许被改掉；其余项照本文件唯一的写法序列化。"""
    out = []
    for item in rows:
        if isinstance(item, tuple):
            _row, raw = item
            out.append(raw + "\n")
        else:
            out.append(json.dumps(item, ensure_ascii=False) + "\n")
    return "".join(out)


def write_jsonl(rows, path):
    with open(path, "w", encoding="utf-8") as out:
        out.write(jsonl_text(rows))


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

# 片段秒数只写到毫秒：`diagnose.py:104` 与 `screen.py:340` 都是 `round(采样数 / sr, 3)`。
# 半个最末位（0.0005 s）就是「写下的 seconds」与「真长度」之间最多差多少。
SECONDS_ROUND_DIGITS = 3
SECONDS_HALF_UNIT = 0.5 * 10.0 ** -SECONDS_ROUND_DIGITS  # 0.0005 s
SECONDS_AGREE_UNIT = 10.0 ** -SECONDS_ROUND_DIGITS  # 0.001 s：两份 jsonl 的秒数只差一个最末位就算同一段


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


def check_tone_row(tone_row, present_db):
    """取这一条的单音结论并核它自洽；没有 `tone` 键说明 diagnose.py 没加 --tone 就跑过，不能当成「没有单音」。

    `present_db` 是 `screen_rules.json` 里那个阈值，只用来按逐秒证据重算一遍结论（不在代码里写死）。"""
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
    _check_per_second(tone_row, tone, present_db)
    return tone


def _check_per_second(tone_row, tone, present_db):
    """按逐秒证据重算结论，必须与存的那个逐字相同；条数也要对得上片段秒数。

    逐秒证据是 `tone.per_second`（正本：`diagnose.tone_per_second`），每秒一项、带 `prominence_db`；
    重算只走 `diagnose.tone_verdict`。任一项对不上就停：结论与数据各说各话时，判定（丢不丢）与理由都无从信。"""
    path = tone_row.get("path")
    per = tone.get("per_second")
    if not isinstance(per, list) or not per:
        raise SystemExit(f"{path} 的单音行里没有逐秒证据 tone.per_second（现在是 {per!r}）："
                         "核不了结论是算出来的还是拍进去的，重跑一次 diagnose.py --tone")
    for i, row in enumerate(per, 1):
        if not isinstance(row, dict):
            raise SystemExit(f"{path} 第 {i} 秒的逐秒证据不是对象（是 {type(row).__name__}）")
        db = row.get("prominence_db")
        if not isinstance(db, (int, float)) or isinstance(db, bool) or not math.isfinite(db):
            raise SystemExit(f"{path} 第 {i} 秒的 prominence_db = {db!r} 不是有限数："
                             "这份单音 jsonl 不是 diagnose.py --tone 写的，或写出来被改过")
    _check_per_second_count(path, tone_row.get("seconds"), len(per))
    got = diagnose.tone_verdict(per, present_db)
    if got != tone["verdict"]:
        raise SystemExit(
            f"{path} 的单音结论与逐秒证据重算的不一样：存的是 "
            f"{json.dumps(tone['verdict'], ensure_ascii=False, sort_keys=True)}，按 {len(per)} 秒的 per_second "
            f"与 present_db = {present_db:g} 重算得 {json.dumps(got, ensure_ascii=False, sort_keys=True)}："
            "先重跑一次 diagnose.py --tone，别拿对不上的结论去判丢不丢")


def _check_per_second_count(path, seconds, n):
    """逐秒证据的条数要等于片段整秒数：`diagnose.py` 逐秒切块只丢末尾不足 1 秒的尾巴（那一截测不出 1 Hz）。

    条数的正本是 `采样数 // sr`，而单音行写下的 `seconds` 是它四舍五入到毫秒的结果（`diagnose.py:104`）。
    所以真长度落在 `seconds ± 0.0005` 秒之间，条数只能是这两端取整后的两个值之一。
    例：159999 采样、16 kHz → 真长 9.9999375 秒、条数 9，写下的 `seconds` 却是 10.0。
    单音行没写 `seconds` 就停下：没有片段长度，核不了逐秒证据是齐的还是缺的。"""
    if seconds is None:
        raise SystemExit(f"{path} 的单音行里没有 seconds：核不了逐秒证据该有几条，"
                         "先跑一遍 diagnose.py --tone（它的每一行都写 seconds）")
    if not isinstance(seconds, (int, float)) or isinstance(seconds, bool) or not math.isfinite(seconds):
        raise SystemExit(f"{path} 的 seconds = {seconds!r} 不是有限数：算不出该有几秒的单音证据")
    low = math.floor(seconds - SECONDS_HALF_UNIT)
    high = math.floor(seconds + SECONDS_HALF_UNIT)
    if low == high:
        want = f"{low} 秒"
    else:
        want = f"{low} 或 {high} 秒"
    if not low <= n <= high:
        raise SystemExit(f"{path} 的单音逐秒证据有 {n} 秒，片段却长 {seconds} 秒："
                         "diagnose.py 只丢末尾不足 1 秒的尾巴（写下的秒数只精确到毫秒），"
                         f"所以本该有 {want} 的证据；缺了就比不出结论（先重跑一次 diagnose.py --tone）")


def _length_of(row):
    """一行里写的片段秒数：筛查行写在顶层，早先也有写进 `measures` 的；两处都没有就返回 None。"""
    measures = row.get("measures")
    for holder in (row, measures if isinstance(measures, dict) else {}):
        value = holder.get("seconds")
        if value is not None:
            return value
    return None


def _check_screen_length(path, screen_row, tone_row):
    """筛查行与单音行写的片段秒数要一致（差不超过一个毫秒位）：两份证据得是同一次读出来的那一段。

    筛查行没写长度就跳过（没有第二处表示可比）；写了却对不上就停下：拿别段的筛查结论配这段的单音结论，
    丢不丢都成了猜的。"""
    screen_seconds = _length_of(screen_row)
    if screen_seconds is None:
        return
    if not isinstance(screen_seconds, (int, float)) or isinstance(screen_seconds, bool) \
            or not math.isfinite(screen_seconds):
        raise SystemExit(f"{path} 的筛查行 seconds = {screen_seconds!r} 不是有限数：合不了这一条的片段长度")
    tone_seconds = tone_row.get("seconds")
    if not isinstance(tone_seconds, (int, float)) or isinstance(tone_seconds, bool) \
            or not math.isfinite(tone_seconds):
        raise SystemExit(f"{path} 的单音行 seconds = {tone_seconds!r} 不是有限数：合不了这一条的片段长度")
    if abs(screen_seconds - tone_seconds) > SECONDS_AGREE_UNIT:
        raise SystemExit(f"{path} 的筛查行写 {screen_seconds} 秒、单音行写 {tone_seconds} 秒："
                         f"差超过 {SECONDS_AGREE_UNIT:g} 秒，两份证据不是同一段，先重跑"
                         "（diagnose.py --tone 与 screen.py 读的该是同一个文件）")


def merge(screen_rows, tone_rows, drop_where, present_db):
    """合成一份筛查 jsonl。返回 {"rows": 新的行列表, "total"/"keep"/"drop"/"tone_dropped": 计数}。

    顺序跟 `screen_rows` 走；`screen_rows` 本身不改写（返回的是副本）。
    `drop_where` 与 `present_db` 都来自 `screen_rules.json` 的 `tone_check`（命令行读 json，代码里不写这两个数）。
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
        tone = check_tone_row(tones[row["path"]], present_db)
        _check_screen_length(row["path"], row, tones[row["path"]])
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


def splice(base_rows, screen_rows, tone_rows, drop_where, present_db, only_regex):
    """只把匹配 `only_regex` 的行换成 merge 的结果，其余行按 `base_rows` 原样带过。

    `base_rows` 的每一项可以是 `read_jsonl_lines` 的 `(row, 原文)`，也可以是单个 row（手上只有解析结果时）；
    给了原文的，不匹配的行就照原文那一行写回（空格 · 键序 · 转义都不动），不再序列化一遍。

    `base_rows` 定顺序与「不并入」的那几条的内容；`screen_rows` 里匹配的那几条必须与 `base_rows` 里匹配的
    逐条同 path、同序（对不上就停：拿另一份清单来并，等于悄悄挪走几条结论）。`tone_rows` 只许是匹配的行
    （否则说明拿错了单音文件，并进去的结论与这段声音无关）。返回值同 `merge`，`rows` 是拼好的整份。"""
    try:
        rx = re.compile(only_regex)
    except re.error as e:
        raise SystemExit(f"--only-regex {only_regex!r} 不是合法正则（{e}）：改成一个能匹配 path 的正则")

    def hit(row):
        path = row.get("path")
        if not isinstance(path, str) or not path:
            raise SystemExit(f"--base / --screen 里有行没有 path：{json.dumps(row, ensure_ascii=False)[:200]}")
        return rx.search(path) is not None

    base = [item if isinstance(item, tuple) else (item, None) for item in base_rows]
    base_hits = [row["path"] for row, _raw in base if hit(row)]
    sub_screen = [row for row in screen_rows if hit(row)]
    got_hits = [row["path"] for row in sub_screen]
    if got_hits != base_hits:
        lack = [p for p in base_hits if p not in set(got_hits)][:5]
        extra = [p for p in got_hits if p not in set(base_hits)][:5]
        raise SystemExit(
            f"--screen 里匹配 {only_regex!r} 的行与 --base 对不上（{len(got_hits)} vs {len(base_hits)} 条）："
            f"base 有、screen 没有 {lack}；screen 有、base 没有 {extra}。两份清单必须是同一批片段")
    stray = [row.get("path") for row in tone_rows if not hit(row)]
    if stray:
        raise SystemExit(f"--tone 里有 {len(stray)} 条不匹配 {only_regex!r}：{stray[:5]}（拿错单音文件了？）")
    merged = merge(sub_screen, tone_rows, drop_where, present_db)
    by_path = {row["path"]: row for row in merged["rows"]}
    rows = [by_path[row["path"]] if hit(row) else (row if raw is None else (row, raw))
            for row, raw in base]
    return {**merged, "rows": rows}


def check_file(path, text):
    """重算出来的正文与 `path` 逐字节比；不同就报第一处不同并停下（不写盘，交给人看）。"""
    want = Path(path).read_bytes()
    got = text.encode("utf-8")
    if want == got:
        print(f"{path} 与重新生成的结果逐字节一致")
        return
    want_lines, got_lines = want.decode("utf-8").splitlines(), text.splitlines()
    for i, (x, y) in enumerate(zip(want_lines, got_lines), 1):
        if x != y:
            raise SystemExit(f"{path} 第 {i} 行与重新生成的不同：\n  文件里：{x}\n  重算得：{y}")
    raise SystemExit(f"{path} 与重新生成的不同（行数 {len(want_lines)} vs {len(got_lines)}）："
                     "先重跑一次生成命令，再看是哪几行多了 / 少了")


# ---------------------------------------------------------------- 命令行

def summary(res):
    return (f"共 {res['total']} 条：keep {res['keep']} · drop {res['drop']}"
            f"（其中因单音新丢 {res['tone_dropped']} 条）")


def run(args):
    if args.only_regex and not args.base:
        raise SystemExit("--only-regex 要和 --base 一起用：--base 是「其余行原样带过」的那份")
    if args.base and not args.only_regex:
        raise SystemExit("--base 只在 --only-regex 模式下有意义：只并匹配的那一段，其余行原样带过")
    if not (args.out or args.check):
        raise SystemExit("要 --out（写盘）或 --check（只核对）中的至少一个")
    if args.out:
        out = Path(args.out)
        for name in ("screen", "tone", "base"):
            ref = getattr(args, name)
            if ref and out.resolve() == Path(ref).resolve():
                raise SystemExit(f"--out 指到了 --{name} 上（{out}）：不原地覆盖，换一个文件名")
    drop_where = load_drop_where(args.rules)
    present_db = load_present_db(args.rules)
    screen_rows = read_jsonl(args.screen)
    tone_rows = read_jsonl(args.tone)
    if args.only_regex:
        res = splice(read_jsonl_lines(args.base), screen_rows, tone_rows, drop_where, present_db,
                     args.only_regex)
    else:
        res = merge(screen_rows, tone_rows, drop_where, present_db)
    text = jsonl_text(res["rows"])
    if args.out:
        Path(args.out).write_text(text, encoding="utf-8")
    if args.check:
        check_file(args.check, text)
    print(summary(res))


def main(argv=None):
    ap = argparse.ArgumentParser(
        description="把单音结论并进筛查结果（丢哪些取 screen_rules.json 的 tone_check.drop_where）")
    ap.add_argument("--screen", required=True, help="screen.py 输出的 jsonl")
    ap.add_argument("--tone", required=True, help="diagnose.py --tone 输出的 jsonl")
    ap.add_argument("--out", default=None, help="合成后的 jsonl（不许等于 --screen / --tone / --base；--check 时可省）")
    ap.add_argument("--base", default=None, help="只并一段时的那份整表：不匹配 --only-regex 的行原样带过")
    ap.add_argument("--only-regex", default=None, help="只并 path 匹配这条正则的行（不匹配的行按 --base 原样带过）")
    ap.add_argument("--check", type=Path, default=None, help="与这份文件逐字节比；不同就退非零（不写盘）")
    ap.add_argument("--rules", default=str(RULES), help="判据 json（默认同目录的 screen_rules.json）")
    run(ap.parse_args(argv))


if __name__ == "__main__":
    main()
