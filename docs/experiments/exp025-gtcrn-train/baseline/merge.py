"""exp025 · 把云上与本机两份成绩单的逐段分数合成一份（基线成绩单的分工，见 README 第二节）。

- 云上那份：`vdn-eval scorecard --qa`，云上没有语音识别服务，吃字（V-EAT-ASR）判不了，其余各项都有
- 本机那份：同样的命令，本机有识别服务，吃字有值
- 合并：每段的吃字取本机，其余全部取云上（行格式不改，项的顺序不变）；同时把其余每一项两边逐段比一遍，写出最大差，
  作为跨平台对拍的读数（云上 Linux 与本机 macOS 的浮点舍入不同，读数不必为 0，判据见 README 第四节）

行格式的正本是 Rust `scorecard/load.rs` 的 `row_json`；本文件只换一个分数对象，不解析分数的含义。
合并后用 `vdn-eval scorecard --from-clips <合并目录>/clips.jsonl --out <另一个新目录>` 出成绩单（Rust 读回时会再核一遍行格式）。
两边同目录 `scorecard.json` 的缺段记录必须都为空才合并；合并目录里写一份只含缺段记录的 `scorecard.json` 给 `--from-clips` 读。

用法：python merge.py --cloud <云上 clips.jsonl> --local <本机 clips.jsonl> --out <新目录> [--parity-only]
  `--parity-only`：识别上云以后两边都有吃字值，只对拍全部项（含吃字）、不合并，只写 merge-report.json
"""
import argparse
import json
from pathlib import Path

EAT = "V-EAT-ASR"
# 跨平台对拍的线（正本；README 第四节）：每项逐段值差的上限（该项自己的单位）与判定不同的段占比上限
PARITY_ABS = 1e-3
PARITY_FLIP = 0.01


def _by_clip(rows, side):
    out = {}
    for r in rows:
        if r["clip"] in out:
            raise ValueError(f"{side} 里片段 {r['clip']} 出现不止一次")
        out[r["clip"]] = r
    return out


def _scores(r):
    return {s["v"]: s for s in r["scores"]}


def parity(cloud_rows, local_rows):
    """只对拍、不合并（识别上云后两边都有吃字值）：全部项（含吃字）逐段比，返回与 merge 相同形状的报告。"""
    return _compare(cloud_rows, local_rows, eat_rules=False)[1]


def merge(cloud_rows, local_rows):
    """返回 (合并后的行, {项: {pairs, max_abs_diff, one_sided}})。两边片段集合、系统必须一致。"""
    return _compare(cloud_rows, local_rows, eat_rules=True)


def _compare(cloud_rows, local_rows, eat_rules):
    c, l = _by_clip(cloud_rows, "云上"), _by_clip(local_rows, "本机")
    if set(c) != set(l):
        raise ValueError(f"两边片段不一致：只在云上 {sorted(set(c) - set(l))[:5]}，只在本机 {sorted(set(l) - set(c))[:5]}")
    report = {}
    out = []
    for clip in [r["clip"] for r in cloud_rows]:
        rc, rl = c[clip], l[clip]
        for k in ("system", "scene", "tier", "device"):
            if rc[k] != rl[k]:
                raise ValueError(f"{clip} 的 {k} 两边不同：{rc[k]} ≠ {rl[k]}")
        sc, sl = _scores(rc), _scores(rl)
        if eat_rules and sc.get(EAT, {}).get("value") is not None:
            raise ValueError(f"{clip}：云上那份已经有吃字的值，不该合并（云上不该有识别服务）")
        if eat_rules and sl.get(EAT, {}).get("value") is None:
            raise ValueError(f"{clip}：本机那份没有吃字的值（识别服务没开？）")
        for v, s in sc.items():
            if eat_rules and v == EAT:
                continue
            a, b = s.get("value"), sl.get(v, {}).get("value")
            rep = report.setdefault(v, {"clips": 0, "pairs": 0, "max_abs_diff": 0.0, "one_sided": 0, "verdict_flips": 0})
            rep["clips"] += 1
            rep["verdict_flips"] += int(s.get("verdict") != sl.get(v, {}).get("verdict"))
            if a is None and b is None:
                continue
            if a is None or b is None:
                rep["one_sided"] += 1
                continue
            rep["pairs"] += 1
            rep["max_abs_diff"] = max(rep["max_abs_diff"], abs(a - b))
        row = dict(rc)
        row["scores"] = [sl[EAT] if s["v"] == EAT else s for s in rc["scores"]]
        out.append(row)
    return out, report


def parity_errors(report):
    """对拍不过的项（空 = 过）：逐段值差 > PARITY_ABS；有段一边有值一边没值；判定不同的段占全部段数 > PARITY_FLIP。"""
    errs = []
    for v, r in sorted(report.items()):
        if r["max_abs_diff"] > PARITY_ABS:
            errs.append(f"{v} 逐段值差最大 {r['max_abs_diff']:.3g} > {PARITY_ABS:g}")
        if r["one_sided"]:
            errs.append(f"{v} 有 {r['one_sided']} 段一边有值一边没值")
        n = r["clips"]
        if n and r["verdict_flips"] / n > PARITY_FLIP:
            errs.append(f"{v} 判定不同 {r['verdict_flips']}/{n} 段 > {PARITY_FLIP:.0%}")
    return errs


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--cloud", required=True)
    ap.add_argument("--local", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--parity-only", action="store_true", help="只对拍不合并：两边都有吃字值时用（识别上云以后）")
    a = ap.parse_args(argv)
    read = lambda p: [json.loads(x) for x in Path(p).read_text().splitlines() if x.strip()]  # noqa: E731
    if a.parity_only:
        rows, rep = None, parity(read(a.cloud), read(a.local))
    else:
        rows, rep = merge(read(a.cloud), read(a.local))
    # 两边同目录成绩单的缺段记录都必须为空（README 第四节验收：没跑成的段数为 0）
    for side, p in (("云上", a.cloud), ("本机", a.local)):
        miss = json.loads(Path(p).with_name("scorecard.json").read_text())["missing"]
        if miss:
            raise ValueError(f"{side}那份有 {len(miss)} 段没跑成（{miss[:2]}…），先补跑，不合并")
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    if rows is None:
        with open(out / "merge-report.json", "x") as f:
            f.write(json.dumps({"cloud": a.cloud, "local": a.local, "mode": "parity-only", "items": rep,
                                "parity": {"PARITY_ABS": PARITY_ABS, "PARITY_FLIP": PARITY_FLIP, "errors": parity_errors(rep)}},
                               ensure_ascii=False, indent=1) + "\n")
        errs = parity_errors(rep)
        print("跨平台对拍（只对拍）：" + ("过" if not errs else "不过 —— " + "；".join(errs)))
        return
    with open(out / "clips.jsonl", "x") as f:
        f.write("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in rows))
    # vdn-eval scorecard --from-clips 要读同目录 scorecard.json 的缺段记录：只写这一项（都为空）
    with open(out / "scorecard.json", "x") as f:
        f.write(json.dumps({"doc": "merge.py 产物：只含缺段记录，供 --from-clips 读；成绩单另出到别的目录", "missing": []},
                           ensure_ascii=False) + "\n")
    with open(out / "merge-report.json", "x") as f:
        f.write(json.dumps({"cloud": a.cloud, "local": a.local, "clips": len(rows), "items": rep,
                            "parity": {"PARITY_ABS": PARITY_ABS, "PARITY_FLIP": PARITY_FLIP, "errors": parity_errors(rep)}},
                           ensure_ascii=False, indent=1) + "\n")
    worst = sorted(rep.items(), key=lambda kv: -kv[1]["max_abs_diff"])[:5]
    print(f"合并 {len(rows)} 段 → {out}；两边差最大的项：" + " · ".join(f"{v} {d['max_abs_diff']:.3g}" for v, d in worst))
    errs = parity_errors(rep)
    print("跨平台对拍：" + ("过" if not errs else "不过 —— " + "；".join(errs)))


if __name__ == "__main__":
    main()
