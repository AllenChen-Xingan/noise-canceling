"""exp005 第四步：生成盲听页——「压得干净 vs 不吃字」的取舍（大总管 2026-10-30 定四个候选）。

- 每题：**原声**（标明，参照）+ 三份盲测候选，随机排进 A / B / C：
  · raw：只有 GTCRN（模型原样输出）
  · personal：保守版（衰减下限 −12 dB + VAD 快开慢关 open −6 dB + 不对称门控 下限 −12 dB、确认 2 次、慢收 0.5 s）
  · aggressive：激进版（不混回原声 + 门控 下限 −20 dB、确认 1 次、慢收 0.3 s）
- 每段取约 25 s 节选：锁定之后、「只有邻座在说」与「主人在说」都有的那一段（没锁定的段取正中）
- 检验题一道：某段的三个槽位里有两个内容相同（文件名不同），期望「分不出」或不把那两份分出高下
- **每次生成一个新 build**：页面与音频写进 <主目录>/_tmp/20261031-exp005/builds/<build>/，不覆盖任何旧文件
- **答案不放在听测目录里**（CLAUDE.md 第五节第 1 条）：揭盲表 unblind-key.json 与写着版本名的 files.json 只写进
  docs/experiments/exp005-p2-speaker/builds/<build>/ 并提交；听测目录只有 index.html、manifest.js（题号随机、无槽位映射、
  无版本名、检验题与普通题同格式）和随机名音频。manifest.js 同时复制一份进仓
- 节选做 15 ms 淡入淡出；不做响度归一（门控本身就是电平差，归一会抹掉它）
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp005-p2-speaker/build_page.py
"""
import hashlib
import re
import json
import secrets
import shutil
import sys
from datetime import datetime
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
from _common import vad_mask  # noqa: E402
from _paths import MAIN  # noqa: E402

BASE = MAIN / "_tmp/20261031-exp005"
SRC, OUT = BASE / "src", BASE / "out"
SR = 16000
EXC = 25.0
CANDS = ["raw", "personal", "aggressive"]
NAMES = {"raw": "只有 GTCRN（模型原样输出）",
         "personal": "保守版：下限 −12 dB + VAD 快开慢关（open −6 dB）+ 门控（下限 −12 dB、确认 2 次、慢收 0.5 s）",
         "aggressive": "激进版：不混回原声 + 门控（下限 −20 dB、确认 1 次、慢收 0.3 s）"}
DESC = {"one": "主人和邻座一人轮流说", "multi": "邻桌几个人一直在聊，主人间或开口", "other_first": "会议开头邻座先说了一阵",
        "owner_only": "只有主人 + 咖啡馆底噪"}


def pick_window(c, log, n):
    own = sf.read(str(SRC / f"{c['id']}_owner.wav"), dtype="float32")[0][:n]
    itf = sf.read(str(SRC / f"{c['id']}_interf.wav"), dtype="float32")[0][:n]
    om = np.pad(vad_mask(own)[0], (0, n))[:n] if own.any() else np.zeros(n, bool)
    im = np.pad(vad_mask(itf)[0], (0, n))[:n] if itf.any() else np.zeros(n, bool)
    lock = next((l["t"] for l in log["judge_logs"] if l["event"] and "locked" in l["event"]), None)
    w = int(EXC * SR)
    if lock is None or not im.any():
        s = max(0, (n - w) // 2)
        return s, s + w, lock
    start0 = int(lock * SR)
    best, bs = -1.0, max(0, min(start0, n - w))
    for s in range(start0, max(start0, n - w) + 1, SR // 2):
        sc = min((im[s:s + w] & ~om[s:s + w]).mean(), om[s:s + w].mean())
        if sc > best:
            best, bs = sc, s
    return bs, min(n, bs + w), lock


def cut(x, a, b):
    y = x[a:b].copy()
    f = int(0.015 * SR)
    r = np.linspace(0, 1, f, dtype=np.float32)
    y[:f] *= r
    y[-f:] *= r[::-1]
    return y


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def main():
    clips = json.loads((HERE / "clips.json").read_text())["clips"]
    build = datetime.now().strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    page = BASE / "builds" / build
    (page / "audio").mkdir(parents=True, exist_ok=False)
    rng = secrets.SystemRandom()
    key, items, files, cuts = {}, [], {}, {}

    def save(y, tag):
        f = f"audio/{secrets.token_hex(4)}.wav"
        sf.write(str(page / f), y, SR, subtype="PCM_16")
        files[f] = {"what": tag, "sha256": sha(page / f)}
        return f

    for c in clips:
        cid = c["id"]
        log = json.loads((OUT / f"{cid}.log.json").read_text())
        ys = {v: sf.read(str(OUT / f"{cid}.{v}.wav"), dtype="float32")[0] for v in CANDS}
        mix = sf.read(str(SRC / f"{cid}_mix.wav"), dtype="float32")[0]
        a, b, lock = pick_window(c, log, len(ys["raw"]))
        cuts[cid] = (a, b, ys, mix)
        order = CANDS[:]
        rng.shuffle(order)
        qid = "q" + secrets.token_hex(3)
        key[qid] = {"clip": cid, "order": order, "excerpt_s": [a / SR, b / SR], "lock_t": lock, "trap": False}
        items.append({"id": qid, "desc": DESC[c["kind"]], "ref": save(cut(mix, a, b), f"{cid} mix"),
                      "cands": [save(cut(ys[v], a, b), f"{cid} {v}") for v in order]})
    # 检验题：某段三个槽位里两个内容相同（各存一份，文件名不同）
    tc = rng.choice([c for c in clips if c["neighbors"]])
    a, b, ys, mix = cuts[tc["id"]]
    dup, other = rng.sample(CANDS, 2)
    slots = [dup, dup, other]
    rng.shuffle(slots)
    qid = "q" + secrets.token_hex(3)
    key[qid] = {"clip": tc["id"], "order": slots, "excerpt_s": [a / SR, b / SR], "trap": True,
                "expect": "两份相同的槽位不该被分出高下（例如不该一个被选为最轻、另一个被选为最响）"}
    items.append({"id": qid, "desc": DESC[tc["kind"]], "ref": save(cut(mix, a, b), f"trap {tc['id']} mix"),
                  "cands": [save(cut(ys[v], a, b), f"trap {tc['id']} {v}") for v in slots]})
    rng.shuffle(items)
    manifest = {"build": build, "items": items}
    (page / "manifest.js").write_text("window.EXP005 = " + json.dumps(manifest, ensure_ascii=False) + ";\n")
    shutil.copyfile(HERE / "ab_page.html", page / "index.html")
    keep = HERE / "builds" / build
    keep.mkdir(parents=True, exist_ok=False)
    (keep / "unblind-key.json").write_text(json.dumps({"build": build, "cands": NAMES, "key": key},
                                                      ensure_ascii=False, indent=1))
    (keep / "files.json").write_text(json.dumps(files, ensure_ascii=False, indent=1))
    shutil.copyfile(page / "manifest.js", keep / "manifest.js")
    # 揭盲自检
    leaked = [f.name for f in page.iterdir() if f.name in ("unblind-key.json", "files.json")]
    assert not leaked, f"听测目录里不该有揭盲文件：{leaked}"
    mtxt = (page / "manifest.js").read_text()
    for word in CANDS + ["trap", "slotmap"]:
        assert word not in mtxt, f"manifest.js 里出现了 {word!r}"
    assert not re.search(r"\bc0\d\b", mtxt), "manifest.js 里出现了段号"
    names = [f for it in items for f in it["cands"]]
    assert len(names) == len(set(names)), "manifest.js 里有重复的音频文件名"
    print("build", build, "→", page / "index.html")


if __name__ == "__main__":
    main()
