"""exp005 第四步：生成「个性化 开 / 关」盲听页。

- 每段取约 25 s 的节选：锁定之后、「只有邻座在说」与「主人在说」都有的那一段（没锁定的段取正中）
- 每段两份候选：vad（个性化关）· personal（个性化开），随机排进 A / B；另给原声（参考）
- 检验题一道：A、B 是同一个文件（应答「差不多」）
- **每次生成一个新 build**：页面与音频写进 <主目录>/_tmp/20261031-exp005/builds/<build>/，不覆盖任何旧文件；
  揭盲表 unblind-key.json、manifest.js、files.json 当场复制进 docs/experiments/exp005-p2-speaker/builds/<build>/
  （CLAUDE.md 第五节第 1 条「揭盲表丢失之坑」）。页面与文件名里不出现版本名
- 节选做 15 ms 淡入淡出；两份候选按同一节选位置截取，不做响度归一（门控本身就是电平差，归一会抹掉它）
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp005-p2-speaker/build_page.py
"""
import base64
import hashlib
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
CANDS = ["vad", "personal"]
DESC = {"one": "主人和邻座一人轮流说", "multi": "邻桌几个人一直在聊，主人间或开口", "other_first": "会议开头邻座先说了一阵",
        "owner_only": "只有主人 + 咖啡馆底噪（检查个性化有没有伤到主人）"}


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
        io = (im[s:s + w] & ~om[s:s + w]).mean()
        ow = om[s:s + w].mean()
        sc = min(io, ow)
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
    key, mclips, files = {}, [], {}

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
        n = len(ys["vad"])
        a, b, lock = pick_window(c, log, n)
        order = CANDS[:]
        rng.shuffle(order)
        key[cid] = {"order": order, "excerpt_s": [a / SR, b / SR], "lock_t": lock}
        mclips.append({"id": cid, "desc": DESC[c["kind"]], "ref": save(cut(mix, a, b), f"{cid} mix"),
                       "cands": [save(cut(ys[v], a, b), f"{cid} {v}") for v in order]})
    # 检验题：同一个文件
    tc = rng.choice([c for c in clips if c["neighbors"]])
    tv = rng.choice(CANDS)
    a, b = key[tc["id"]]["excerpt_s"]
    y = sf.read(str(OUT / f"{tc['id']}.{tv}.wav"), dtype="float32")[0]
    f = save(cut(y, int(a * SR), int(b * SR)), f"trap {tc['id']} {tv}")
    trap = {"id": "trap-identical", "desc": DESC[tc["kind"]], "ref": mclips[[c["id"] for c in clips].index(tc["id"])]["ref"],
            "cands": [f, f]}
    items = mclips + [trap]
    rng.shuffle(items)
    manifest = {"build": build, "items": items,
                "slotmap": base64.b64encode(json.dumps({"clips": {k: v["order"] for k, v in key.items()},
                                                        "trap": {"clip": tc["id"], "version": tv}}).encode()).decode()}
    (page / "manifest.js").write_text("window.EXP005 = " + json.dumps(manifest, ensure_ascii=False) + ";\n")
    shutil.copyfile(HERE / "ab_page.html", page / "index.html")
    (page / "unblind-key.json").write_text(json.dumps({"build": build, "cands": {"vad": "个性化关（VAD 快开慢关 + 下限）",
                                                                              "personal": "个性化开（+ 注册 + 不对称门控）"},
                                                       "key": key, "trap": {"clip": tc["id"], "version": tv}},
                                                      ensure_ascii=False, indent=1))
    (page / "files.json").write_text(json.dumps(files, ensure_ascii=False, indent=1))
    keep = HERE / "builds" / build
    keep.mkdir(parents=True, exist_ok=False)
    for x in ["unblind-key.json", "manifest.js", "files.json"]:
        shutil.copyfile(page / x, keep / x)
    print("build", build, "→", page / "index.html")


if __name__ == "__main__":
    main()
