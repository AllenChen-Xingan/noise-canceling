"""exp003 附：逐段列出「原声转写里在、降噪后转写里被删掉」的词（吃字候选），写成 eaten_words.md。

只读 scores.json（score.py --asr 的产物），不跑模型。
- 参照 = 干净参考的 Qwen3-ASR 转写（和 score.py 一致）；英文按词、中文按字（夹在中文里的英文按词）
- 对每段、每个降噪候选：把参照分别与「原声转写」「该候选转写」做编辑距离对齐，取「候选里被删、原声里没被删」的参照位置
- 同时给出 LibriSpeech 人工转写（有就给），用来看参照本身有没有错
- 「两次转写是否一致」直接来自 scores.json 的 asr_stable：一致说明不是同一文件重复转写的随机波动；
  但仍可能是 ASR 对细小声学差异敏感（不是人耳能听出的吞字）——这一层只能靠人听，表里给出时间位置不了，只给上下文
用法：python3 docs/experiments/exp003-offline-ab/eaten_words.py
"""
import json
import re
from pathlib import Path

HERE = Path(__file__).resolve().parent


def tokens(text, lang):
    if lang == "zh":
        return re.findall(r"[一-鿿]|[a-z0-9']+", text.lower())
    return re.findall(r"[a-z0-9']+", text.lower())


def deleted_positions(ref, hyp):
    """Levenshtein 对齐，返回参照里被删除的位置集合。"""
    n, m = len(ref), len(hyp)
    d = [[0] * (m + 1) for _ in range(n + 1)]
    for i in range(n + 1):
        d[i][0] = i
    for j in range(m + 1):
        d[0][j] = j
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            d[i][j] = min(d[i - 1][j] + 1, d[i][j - 1] + 1, d[i - 1][j - 1] + (ref[i - 1] != hyp[j - 1]))
    i, j, dels = n, m, set()
    while i > 0 or j > 0:
        if i > 0 and j > 0 and d[i][j] == d[i - 1][j - 1] + (ref[i - 1] != hyp[j - 1]):
            i, j = i - 1, j - 1
        elif i > 0 and d[i][j] == d[i - 1][j] + 1:
            dels.add(i - 1)
            i -= 1
        else:
            j -= 1
    return dels


def main():
    sc = json.loads((HERE / "scores.json").read_text())
    clips = {c["id"]: c for c in json.loads((HERE / "clips.json").read_text())["clips"]}
    models = [v for v in next(iter(sc["per_clip"].values())) if v not in ("noisy",) and isinstance(next(iter(sc["per_clip"].values()))[v], dict)]
    lines = ["# exp003 · 吃字逐段明细（脚本生成，勿手改）", "",
             f"ASR：{sc['asr_model']['model']}，每个文件转写两次，逐字一致 {sc['asr_model']['stable_files']}。",
             "「新删」= 降噪后转写里被删、原声转写里没被删的参照词。参照 = 干净参考的转写。", ""]
    total = {m: 0 for m in models}
    for cid, r in sc["per_clip"].items():
        lang = clips[cid]["lang"]
        ref = tokens(r["asr_ref"], lang)
        base = deleted_positions(ref, tokens(r["noisy"]["asr"], lang))
        rows = []
        for m in models:
            new = sorted(deleted_positions(ref, tokens(r[m]["asr"], lang)) - base)
            total[m] += len(new)
            if new:
                ctx = " ".join(f"**{w}**" if k in new else w for k, w in enumerate(ref))
                rows.append((m, new, ctx, r[m]["asr"], r[m]["asr_stable"]))
        if not rows:
            continue
        lines += [f"## {cid} · {clips[cid]['desc']}", ""]
        if r.get("truth"):
            lines += [f"- 人工转写（LibriSpeech）：{r['truth'].lower()}"]
        lines += [f"- 参照（干净参考转写）：{r['asr_ref']}", f"- 原声转写：{r['noisy']['asr']}（两次一致：{r['noisy']['asr_stable']}）"]
        for m, new, ctx, hyp, st in rows:
            lines += [f"- **{m}** 新删 {len(new)} 个：{'、'.join(ref[k] for k in new)}",
                      f"  - 参照中位置（加粗）：{ctx}", f"  - {m} 转写：{hyp}（两次一致：{st}）"]
        lines.append("")
    lines += ["## 合计（新删词数）", "", "| 候选 | 新删 |", "|---|---|"] + [f"| {m} | {total[m]} |" for m in models]
    (HERE / "eaten_words.md").write_text("\n".join(lines) + "\n")
    print(total)


if __name__ == "__main__":
    main()
