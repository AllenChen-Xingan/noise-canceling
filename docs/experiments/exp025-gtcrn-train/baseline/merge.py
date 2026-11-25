"""exp025 · 把云上与本机两份成绩单的逐段分数合成一份（基线成绩单的分工，见 README 第二节）。

- 云上那份：`vdn-eval scorecard --qa`，云上没有语音识别服务，吃字（V-EAT-ASR）判不了，其余各项都有
- 本机那份：同样的命令，本机有识别服务，吃字有值
- 合并：每段的吃字取本机，其余全部取云上（行格式不改，项的顺序不变）；同时把其余每一项两边逐段比一遍，写出最大差，
  作为跨平台对拍的读数（云上 Linux 与本机 macOS 的浮点舍入不同，读数不必为 0，判据见 README 第四节）

行格式的正本是 Rust `scorecard/load.rs` 的 `row_json`；本文件只换一个分数对象，不解析分数的含义。
合并后用 `vdn-eval scorecard --from-clips <合并目录>/clips.jsonl` 出成绩单（Rust 读回时会再核一遍行格式）。

用法：python merge.py --cloud <云上 clips.jsonl> --local <本机 clips.jsonl> --out <新目录>
"""
import argparse
import json
from pathlib import Path

EAT = "V-EAT-ASR"


def _by_clip(rows, side):
    out = {}
    for r in rows:
        if r["clip"] in out:
            raise ValueError(f"{side} 里片段 {r['clip']} 出现不止一次")
        out[r["clip"]] = r
    return out


def _scores(r):
    return {s["v"]: s for s in r["scores"]}


def merge(cloud_rows, local_rows):
    """返回 (合并后的行, {项: {pairs, max_abs_diff, one_sided}})。两边片段集合、系统必须一致。"""
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
        if sc.get(EAT, {}).get("value") is not None:
            raise ValueError(f"{clip}：云上那份已经有吃字的值，不该合并（云上不该有识别服务）")
        if sl.get(EAT, {}).get("value") is None:
            raise ValueError(f"{clip}：本机那份没有吃字的值（识别服务没开？）")
        for v, s in sc.items():
            if v == EAT:
                continue
            a, b = s.get("value"), sl.get(v, {}).get("value")
            rep = report.setdefault(v, {"pairs": 0, "max_abs_diff": 0.0, "one_sided": 0})
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


def main(argv=None):
    ap = argparse.ArgumentParser()
    ap.add_argument("--cloud", required=True)
    ap.add_argument("--local", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args(argv)
    read = lambda p: [json.loads(x) for x in Path(p).read_text().splitlines() if x.strip()]  # noqa: E731
    rows, rep = merge(read(a.cloud), read(a.local))
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    with open(out / "clips.jsonl", "x") as f:
        f.write("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in rows))
    with open(out / "merge-report.json", "x") as f:
        f.write(json.dumps({"cloud": a.cloud, "local": a.local, "clips": len(rows), "items": rep},
                           ensure_ascii=False, indent=1) + "\n")
    worst = sorted(rep.items(), key=lambda kv: -kv[1]["max_abs_diff"])[:5]
    print(f"合并 {len(rows)} 段 → {out}；两边差最大的项：" + " · ".join(f"{v} {d['max_abs_diff']:.3g}" for v, d in worst))


if __name__ == "__main__":
    main()
