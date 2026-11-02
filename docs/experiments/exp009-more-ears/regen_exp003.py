"""exp009 · 重新生成作者在 exp003（build 20261030110755-f8c3）听过的音频，供本机预测器打分。

原听测音频在当时 abtest worktree 的 _tmp/ 里，已不在本机。这里复用 exp003 的 enhance.py（同一份处理代码），
只把输出改到本 worktree 的 _tmp/20261031-exp009/exp003-regen/（git 忽略），不碰 exp003 目录（enhance.py
默认会写 exp003-offline-ab/align.json）。输入片段取自 exp004 留下的 src/（clips.json 与 exp003 逐字相同，已核）。

核对：生成后逐段比对 build f8c3 的 files.json 里每个版本的 gain_db / lufs_after / peak_dbfs，写进
results/regen_check.json。差值 ≤ 0.05 dB 视为「与作者听到的是同一份」。

用法（项目根下，要名额）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp009-more-ears/regen_exp003.py
"""
import json
import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP003 = HERE.parent / "exp003-offline-ab"
sys.path.insert(0, str(EXP003))
import enhance  # noqa: E402
from _paths import MAIN, ROOT  # noqa: E402

OUT = ROOT / "_tmp/20261031-exp009/exp003-regen"
SRC = MAIN / "_tmp/20261030-exp004/src"
BUILD = EXP003 / "builds/20261030110755-f8c3/files.json"
MODELS = ["noisy", "dpdfnet2", "gtcrn", "fastenhancer_m"]


def main():
    (OUT / "src").mkdir(parents=True, exist_ok=True)
    for f in SRC.glob("c*.wav"):
        if not (OUT / "src" / f.name).exists():
            shutil.copyfile(f, OUT / "src" / f.name)
    shutil.copyfile(EXP003 / "clips.json", OUT / "clips.json")
    enhance.OUT = OUT
    enhance.HERE = OUT  # clips.json 从这里读，align.json 写到这里
    sys.argv = [sys.argv[0], "--no-voxlock"]
    enhance.main()

    new = json.loads((OUT / "files.json").read_text())
    old = json.loads(BUILD.read_text())
    rows, worst = [], 0.0
    for cid, o in old.items():
        for m in MODELS:
            a, b = o["versions"][m], new[cid]["versions"][m]
            d = {k: round(b[k] - a[k], 3) for k in ("gain_db", "lufs_after", "peak_dbfs")}
            worst = max(worst, *(abs(v) for v in d.values()))
            rows.append({"clip": cid, "model": m, "build_file": a["file"], "regen_file": b["file"], "delta": d})
    check = {"build": "20261030110755-f8c3", "max_abs_delta_db": worst, "same_as_heard": worst <= 0.05, "rows": rows}
    (HERE / "results").mkdir(exist_ok=True)
    (HERE / "results/regen_check.json").write_text(json.dumps(check, ensure_ascii=False, indent=1))
    print("最大差值", worst, "dB →", "一致" if worst <= 0.05 else "不一致，打分结论要降级")


if __name__ == "__main__":
    main()
