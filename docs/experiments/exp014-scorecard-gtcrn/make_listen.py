"""exp014 人耳片段：从一次成绩单 run 的转存音频里，按挑好的段复制「原声 / GTCRN 输出」两版，随机标 A / B。

- 给人听的目录（主目录 `_tmp/exp014/listen/<build-id>/`）只放 `NN-A.wav`、`NN-B.wav` 和 `listen.md`，不放揭盲表
- 揭盲表 `unblind-key.json`（A/B 各是哪个、源文件哈希）写进仓内 `listen/<build-id>/`，当场提交（CLAUDE.md 第五节第 1 条）
- 不覆盖：目标目录已存在就拒跑

用法：python3 make_listen.py <run 的 qa/asr-wav 目录> <picks.json> <给人听的目录> <仓内揭盲目录> [--seed N]
picks.json：[{"clip": "...", "why": "...", "listen_for": "..."}]
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import shutil
from pathlib import Path


def find(wavs: Path, clip: str, kind: str) -> Path:
    hits = sorted(wavs.glob(f"{clip}#*-{kind}-*.wav"))
    if len(hits) != 1:
        raise SystemExit(f"{clip} 的 {kind} 音频找到 {len(hits)} 个，应为 1 个")
    return hits[0]


def sha256(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest()


def make(wavs: Path, picks: list[dict], out: Path, key_dir: Path, seed: int) -> dict:
    if out.exists() or key_dir.exists():
        raise SystemExit(f"目标已存在，不覆盖：{out if out.exists() else key_dir}")
    out.mkdir(parents=True)
    key_dir.mkdir(parents=True)
    rng = random.Random(seed)
    key, lines = {"seed": seed, "items": []}, []
    for i, p in enumerate(picks, 1):
        src = {"原声": find(wavs, p["clip"], "mix"), "GTCRN": find(wavs, p["clip"], "gtcrn")}
        order = ["原声", "GTCRN"]
        rng.shuffle(order)
        item = {"n": i, "clip": p["clip"]}
        for ab, who in zip("AB", order):
            dst = out / f"{i:02d}-{ab}.wav"
            shutil.copyfile(src[who], dst)
            item[ab] = {"is": who, "src": src[who].name, "sha256": sha256(dst)}
        key["items"].append(item)
        lines.append(f"## {i:02d}\n\n- 为什么挑它：{p['why']}\n- 请听：{p['listen_for']}\n")
    head = ("# 人耳片段\n\n每题两个版本 `NN-A.wav` / `NN-B.wav`，同一段 12 秒录音。一个是原声，一个是降噪后的，"
            "顺序随机。请每题记下：A、B 哪个听着更好（或差不多），以及下面「请听」那一项你听到了什么。\n\n")
    (out / "listen.md").write_text(head + "\n".join(lines))
    (key_dir / "unblind-key.json").write_text(json.dumps(key, ensure_ascii=False, indent=1) + "\n")
    shutil.copyfile(out / "listen.md", key_dir / "listen.md")
    return key


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("wavs", type=Path)
    ap.add_argument("picks", type=Path)
    ap.add_argument("out", type=Path)
    ap.add_argument("key_dir", type=Path)
    ap.add_argument("--seed", type=int, default=20261104)
    a = ap.parse_args()
    make(a.wavs, json.loads(a.picks.read_text()), a.out, a.key_dir, a.seed)


if __name__ == "__main__":
    main()
