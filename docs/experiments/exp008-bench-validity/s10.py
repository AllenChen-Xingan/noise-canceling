"""exp008 · S10 隔离检查（synth-standard.md S10）：评测 build 的素材与某个训练 build 的素材逐段求交集。

查三类（都按清单字段算，不按代码推断）：
  1. 说话人：评测段的主讲人 / 邻座 ∈ 训练段的 target_spk 集合
  2. DEMAND 时段：同一环境、同一通道，时间区间 [起, 起 + 时长) 有重叠
  3. 合成噪声函数：评测段的 noise_src 是合成函数名（非 DEMAND / 非文件），且同名函数出现在训练段里
查不了的（如实列出）：babble 里 6 个说话人的 id、合成噪声的种子——两边清单都没记。

用法：python3 s10.py <评测 build> <训练 build 的 truth.jsonl 路径> [--split train]
输出：results/<评测 build>-s10-<训练 build 名>.json + 屏幕摘要。纯标准库，不读音频，秒级。
"""
import json
import re
import sys
from collections import Counter
from pathlib import Path

HERE = Path(__file__).resolve().parent
DEMAND_RE = re.compile(r"DEMAND/(\w+)/(ch\d+)@([\d.]+)s")
EVAL_DUR = 8.0


def demand_iv(src, dur):
    m = DEMAND_RE.search(src or "")
    if not m:
        return None
    t = float(m.group(3))
    return (m.group(1), m.group(2), t, t + dur)


def synth_name(src):
    if not src or src.startswith("DEMAND/") or "babble" in src:
        return None
    return re.split(r"[(@]", src)[0]


def main():
    build, train_truth = sys.argv[1], Path(sys.argv[2])
    split = sys.argv[sys.argv.index("--split") + 1] if "--split" in sys.argv else "train"
    ev = [json.loads(l) for l in open(HERE / "builds" / build / "truth.jsonl")]
    tr = [json.loads(l) for l in open(train_truth)]
    tr = [t for t in tr if t.get("split") == split]
    spk_tr = {t["target_spk"] for t in tr if t.get("target_spk")}
    iv_tr = [demand_iv(t.get("noise"), t.get("dur_s", 10.0)) for t in tr]
    iv_tr = [x for x in iv_tr if x]
    syn_tr = {synth_name(t.get("noise")) for t in tr} - {None}
    rows = []
    for e in ev:
        hit = []
        if e.get("speaker") in spk_tr:
            hit.append("speaker")
        if e.get("neighbor") and e.get("cond", {}).get("sir_db") is not None and e["neighbor"] in spk_tr:
            hit.append("neighbor")
        iv = demand_iv(e.get("noise_src"), EVAL_DUR)
        if iv and any(iv[0] == t[0] and iv[1] == t[1] and iv[2] < t[3] and t[2] < iv[3] for t in iv_tr):
            hit.append("demand_time")
        sn = synth_name(e.get("noise_src"))
        if sn and sn in syn_tr:
            hit.append("synth_fn:" + sn)
        unk = ["babble_speakers"] if "babble" in (e.get("noise_src") or "") else []
        rows.append({"cell": e["cell"], "gen": e["gen"], "clip": e["clip"], "level": e["level"], "hits": hit, "unknown": unk})
    by_gen = {g: {"n": sum(r["gen"] == g for r in rows), "n_hit": sum(r["gen"] == g and bool(r["hits"]) for r in rows),
                  "by_kind": dict(Counter(h.split(":")[0] for r in rows if r["gen"] == g for h in r["hits"]))} for g in ("A", "B")}
    out = {"eval_build": build, "train_truth": str(train_truth), "train_split": split, "n_train": len(tr),
           "n_train_speakers": len(spk_tr), "train_synth_fns": sorted(syn_tr), "summary": by_gen,
           "n_unknown_babble": sum(bool(r["unknown"]) for r in rows), "pass": not any(r["hits"] for r in rows), "rows": rows}
    name = train_truth.parent.name
    (HERE / "results" / f"{build}-s10-{name}.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps({k: out[k] for k in ("n_train", "n_train_speakers", "train_synth_fns", "summary", "n_unknown_babble", "pass")}, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
