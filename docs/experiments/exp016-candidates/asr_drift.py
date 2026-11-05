"""exp016 跨 run 识别差：比两次成绩单 run 各自的识别缓存里「同一音频 · 同一模型 · 同一语言 · 同一次」的转写。

为什么这样测（大总管 2026-11-05 要实测跨 run 识别差）：吞字头（V-EAT-ASR）= 输出比原声多错的字数，原声直通
那一行恒为 0（原声和自己比），量不出跨 run 差。每次 run 都会重新识别同样的 480 段原声（缓存在各自 run 目录下），
两份缓存里键相同（文件 sha256 | 模型 | 语言 | 第几次，正本 `src/eval/src/qa/asr.rs`）的转写若逐字相同，
两次 run 给原声的基准就相同；不同时，一段转写的编辑距离是它让「多错字数」变动的上界。

判定（本脚本的选择，推断；否决线 = 逐段比原声多错 ≥ 5 字，人定 D85 题 3）：
- 全部相同 → 「一致」：吞字头照常与 GTCRN 比
- 每段最大编辑距离 ≤ MAX_OK（1 字，否决线的 1/5）→ 「差远小于否决线」：照常比，findings 写实测差
- 否则 → 「降为参考」

用法：python3 asr_drift.py <run A 的 asr_cache.json> <run B 的 asr_cache.json> [--json 输出]
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

MAX_OK = 1
VETO_CHARS = 5


def norm(t: str) -> str:
    """空白归一（同 asr.rs 口径：去掉所有空白）。"""
    return "".join(t.split())


def edit(a: str, b: str) -> int:
    """按字的编辑距离。"""
    prev = list(range(len(b) + 1))
    for i, x in enumerate(a, 1):
        cur = [i]
        for j, y in enumerate(b, 1):
            cur.append(min(prev[j] + 1, cur[j - 1] + 1, prev[j - 1] + (x != y)))
        prev = cur
    return prev[-1]


def drift(a: dict[str, str], b: dict[str, str]) -> dict:
    common = sorted(set(a) & set(b))
    per_audio: dict[str, int] = {}
    same = 0
    for k in common:
        d = edit(norm(a[k]), norm(b[k]))
        same += d == 0
        sha = k.split("|")[0]
        per_audio[sha] = max(per_audio.get(sha, 0), d)
    mx = max(per_audio.values(), default=0)
    if not common:
        verdict = "无共同键，无法判断"
    elif mx == 0:
        verdict = "一致"
    elif mx <= MAX_OK:
        verdict = "差远小于否决线"
    else:
        verdict = "降为参考"
    return {"common_keys": len(common), "identical_keys": same, "audios": len(per_audio),
            "audios_changed": sum(v > 0 for v in per_audio.values()), "max_edit_per_audio": mx,
            "max_ok": MAX_OK, "veto_chars": VETO_CHARS, "verdict": verdict}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("a", type=Path)
    ap.add_argument("b", type=Path)
    ap.add_argument("--json", type=Path)
    x = ap.parse_args()
    r = drift(json.loads(x.a.read_text()), json.loads(x.b.read_text()))
    print(json.dumps(r, ensure_ascii=False))
    if x.json:
        x.json.write_text(json.dumps(r, ensure_ascii=False, indent=1) + "\n")


if __name__ == "__main__":
    main()
