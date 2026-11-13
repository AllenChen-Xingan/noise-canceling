"""exp022 · 把盲听页导出的答卷 + 揭盲表，转成 `vdn-eval calib` 读的三份输入（codex 第 3 轮 P2 #1）。

盲听页一题只有一个候选（一段降噪输出），答卷是 `answers[{q, usable, eaten}]`；calib 读的是
`trials[].by_model`（答卷）、`per_clip[屏][系统]`（指标）、`[屏][系统]{scoreq, distill_mos}`（评分模型）。这里：
- 每题当一「屏」，屏名 = `q<题号>`，候选 = 揭盲表里的系统名；带噪原声记为 `noisy`，指标都取 0（评估器的数本来就是相对原声的差）
  - 吃字：系统的 del = 揭盲表里「比原声多错几个」，sub = 0，ref_len 同原声 → judge.rs 算出的「多错」与成绩单一致
  - 听感：系统的 scoreq / distill_mos = 揭盲表里的 Δ，原声取 0
- 吃字答案：yes → 勾了这份；no → `["none"]`（明确没吃）；unsure 或没答 → 不写（judge.rs 记判不了）
- 重测题（`retest_of` 非空）不进：只用来量同一人前后一致不一致，另报
- 答卷的 build 与揭盲表不一致、题号在揭盲表里找不到 → 报错，不静默出零条

用法：python3 docs/experiments/exp022-judge-tpr-tnr/convert_answers.py <答卷.json> <揭盲表.json> <输出目录>
然后：vdn-eval calib --answers <输出目录>/answers.json --scores <输出目录>/scores.json \\
                     --predictors <输出目录>/predictors.json --round 2
注意：calib 的混淆矩阵是等权的「这批答卷上的率」；本页是分层抽样，估总体要按 inclusion_prob 加权（weights.json）。
"""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCORECARD = HERE.parents[2].parent / "noise_canceling" / "_tmp/scorecard"  # 主目录，只读


def ref_len_from_run(run, system, clip):
    for line in (SCORECARD / run / "clips.jsonl").read_text().splitlines():
        r = json.loads(line)
        if r["system"] == system and r["clip"] == clip:
            return next(s["extra"]["ref_len"] for s in r["scores"] if s["v"] == "V-EAT-ASR")
    raise KeyError(f"{run} 里找不到 {system} {clip}")


def convert(answers, key, ref_len=ref_len_from_run):
    if answers.get("build") != key.get("build"):
        raise ValueError(f"答卷 build {answers.get('build')} ≠ 揭盲表 build {key.get('build')}")
    byq = {q["q"]: q for q in key["questions"]}
    trials, per, preds, weights, retests = [], {}, {}, {}, []
    for a in answers["answers"]:
        if a["q"] not in byq:
            raise KeyError(f"第 {a['q']} 题不在揭盲表里")
        k = byq[a["q"]]
        if k["retest_of"] is not None:
            retests.append({"q": a["q"], "of": k["retest_of"], "usable": a.get("usable"), "eaten": a.get("eaten")})
            continue
        screen, sysname, j = f"q{a['q']:02d}", k["system"], k["judge"]
        n = j.get("ref_len") or ref_len(k["run"], sysname, k["clip"])
        bm = {"usable": {sysname: a["usable"]} if a.get("usable") else {}}
        if a.get("eaten") == "yes":
            bm["eaten"] = [sysname]
        elif a.get("eaten") == "no":
            bm["eaten"] = ["none"]
        trials.append({"kind": "real", "clip": screen, "slots": {"A": sysname}, "by_model": bm})
        per[screen] = {"noisy": {"del": 0, "sub": 0, "ref_len": n},
                       sysname: {"del": j["eat_extra"], "sub": 0, "ref_len": n}}
        preds[screen] = {"noisy": {"scoreq": 0.0, "distill_mos": 0.0},
                         sysname: {"scoreq": j["d_scoreq"], "distill_mos": j["d_distill"]}}
        weights[screen] = {"inclusion_prob": k["inclusion_prob"], "stratum": k["stratum"]}
    if not trials:
        raise ValueError("转换后一题都没有：检查答卷")
    return ({"build": key["build"], "trials": trials}, {"per_clip": per}, preds,
            {"weights": weights, "retests": retests, "prereg": key.get("prereg")})


def main():
    ans, key, out = json.loads(Path(sys.argv[1]).read_text()), json.loads(Path(sys.argv[2]).read_text()), Path(sys.argv[3])
    out.mkdir(parents=True, exist_ok=True)
    for name, obj in zip(("answers", "scores", "predictors", "weights"), convert(ans, key)):
        (out / f"{name}.json").write_text(json.dumps(obj, ensure_ascii=False, indent=1) + "\n")
    print(f"写出 {out}/answers.json · scores.json · predictors.json · weights.json")


if __name__ == "__main__":
    main()
