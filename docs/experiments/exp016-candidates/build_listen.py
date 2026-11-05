"""exp016 · 人耳盲听：GTCRN 对 DFN3 低延迟版，4 题 × 2 段 = 8 段，每段 12 s，共 96 s（≤ 8 段、≤ 2 分钟）。

听的是成绩单打分时那份输出（含引擎的原声混回，就是用户听到的），取自两次成绩单 run 的 `qa/asr-wav/`。
每题两边按同一积分响度对齐（pyloudnorm，-26 LUFS，峰值不超 -1 dBFS），免得「更响 = 更好」；增益写进揭盲表。
选题（2026-11-05，cands，依据 findings.md 与 `runs/weak-spots-20261105/`）：
  q 咔哒：cafe-s00-ext —— DFN3 咔哒最多的段之一（每分钟 125 个，GTCRN 60 个）
  q 户外：street-outdoor-s05-ext —— GTCRN 最弱的户外段（风声；Distill-MOS 差 +0.23）
  q 交通：transit-hub-c06-ext —— GTCRN 最弱的交通段（空调底噪；ESTOI 差 +0.046）
  q 字头：rail-air-cabin-s03-ext —— 守住项：DFN3 吞字头比 GTCRN 多 4.8 dB 的段（看 GTCRN 的优势人耳听不听得出）
题序和左右用固定种子打乱；页面里不出现片段名、场景名和模型名。

输出（build id = 时间戳 + 4 位随机）：
  <主目录>/_tmp/exp016-listen/<build>/listen/   给人听：index.html + q*_X.wav / q*_Y.wav（无模型名、无答案）
  <主目录>/_tmp/exp016-listen/<build>/key/      揭盲表 unblind-key.json + 清单 manifest.json（不给人）
  docs/experiments/exp016-candidates/builds/listen-<build>/   揭盲表与清单的副本（进仓，CLAUDE.md 第五节第 1 条）

用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp016-candidates/build_listen.py <build-id>
"""
import hashlib
import json
import shutil
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
import _paths  # noqa: E402  exp003：主目录

MAIN = _paths.MAIN
RUNS = {"gtcrn": "_tmp/scorecard/20261105-005916-4ccf0638", "dfn3-ll": "_tmp/scorecard/dfn3-ll-seeded-20261105"}
CLIPS = [  # (片段, 为什么选)
    ("cafe-s00-ext", "DFN3 咔哒最多（125 vs 60 个/分钟）"),
    ("street-outdoor-s05-ext", "GTCRN 最弱的户外段：风声，Distill-MOS 差 +0.23"),
    ("transit-hub-c06-ext", "GTCRN 最弱的交通段：空调底噪，ESTOI 差 +0.046"),
    ("rail-air-cabin-s03-ext", "守住项：DFN3 吞字头比 GTCRN 多 4.8 dB"),
]
SEED = 20261105
LUFS, PEAK = -26.0, 10 ** (-1 / 20)
MAX_FILES, MAX_SECONDS = 8, 120.0
FORBIDDEN = ["gtcrn", "dfn3", "deepfilter", "dpdf", "unblind", "cafe", "street", "transit", "rail", "-ext", "-mba"]


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def source(clip, system):
    hits = sorted((MAIN / RUNS[system] / "qa/asr-wav").glob(f"{clip}#*-{system}-*.wav"))
    if len(hits) != 1:
        raise SystemExit(f"{clip} 的 {system} 输出应恰好 1 个，找到 {len(hits)} 个")
    return hits[0]


def match_gain(x, sr, meter):
    """把 x 调到 LUFS 积分响度的增益（线性）；峰值超 PEAK 就再往下压。"""
    g = 10 ** ((LUFS - meter.integrated_loudness(x)) / 20)
    return min(g, PEAK / max(float(np.max(np.abs(x))), 1e-9))


def plan(rng, n):
    """题序 + 每题左右：返回 [(片段下标, X 是不是 gtcrn)]。"""
    order = rng.permutation(n)
    return [(int(i), bool(rng.integers(2))) for i in order]


def check_listen_dir(listen, files, page):
    """给人听的目录只许有 index.html 与 q*_X/Y.wav；页面不许出现模型名或片段名。"""
    assert sorted(p.name for p in listen.iterdir()) == sorted(files), "listen/ 里混进了别的文件"
    low = page.lower()
    for bad in FORBIDDEN:
        assert bad not in low, f"页面里出现了 {bad}"


PAGE = (HERE / "listen_page.html").read_text() if (HERE / "listen_page.html").exists() else None


def question(qid, n):
    def radios(name, opts):
        return "".join(f'<label><input type="radio" name="{qid}_{name}" value="{v}"> {t}</label>' for v, t in opts)
    overall = [("X++", "X 明显更好"), ("X+", "X 稍好"), ("=", "差不多 / 分不出"), ("Y+", "Y 稍好"), ("Y++", "Y 明显更好")]
    eaten = [("X", "X 丢了字或字头"), ("Y", "Y 丢了字或字头"), ("both", "两边都有"), ("none", "都没有 / 听不出")]
    click = [("X", "X 有咔哒 / 爆音"), ("Y", "Y 有"), ("both", "两边都有"), ("none", "都没有")]
    return (f'<section><h2>第 {n} 题</h2><div class="row">'
            f'<div><b>X</b><br><audio controls preload="auto" src="{qid}_X.wav"></audio></div>'
            f'<div><b>Y</b><br><audio controls preload="auto" src="{qid}_Y.wav"></audio></div></div>'
            f'<fieldset><legend>整体听感哪边更好？</legend>{radios("overall", overall)}</fieldset>'
            f'<fieldset><legend>哪边丢了字或字头？</legend>{radios("eaten", eaten)}</fieldset>'
            f'<fieldset><legend>哪边有咔哒 / 爆音？</legend>{radios("click", click)}</fieldset>'
            f'<p class="mut">想说为什么（可空）：</p><textarea id="{qid}_note"></textarea></section>')


def main():
    import pyloudnorm as pyln
    import soundfile as sf
    bid = sys.argv[1]
    W = MAIN / "_tmp/exp016-listen" / bid
    listen, key = W / "listen", W / "key"
    D = HERE / "builds" / f"listen-{bid}"
    if W.exists() or D.exists():
        sys.exit(f"build {bid} 已存在：不覆盖旧 build，换一个 build id")
    listen.mkdir(parents=True)
    key.mkdir()
    rng = np.random.default_rng(SEED)
    K = {"build": bid, "seed": SEED, "runs": RUNS, "loudness": f"每段对齐到 {LUFS} LUFS，峰值 ≤ -1 dBFS",
         "what": "成绩单打分时那份输出（含引擎原声混回）", "questions": {}}
    files, qs, total = {}, [], 0.0
    for n, (i, x_is_gtcrn) in enumerate(plan(rng, len(CLIPS)), 1):
        clip, why = CLIPS[i]
        qid = f"q{n}"
        side = {"X": "gtcrn" if x_is_gtcrn else "dfn3-ll", "Y": "dfn3-ll" if x_is_gtcrn else "gtcrn"}
        rec = {"clip": clip, "why": why, "slots": {}}
        for slot, system in side.items():
            src = source(clip, system)
            x, sr = sf.read(src, dtype="float32")
            g = match_gain(x, sr, pyln.Meter(sr))
            out = listen / f"{qid}_{slot}.wav"
            sf.write(out, np.clip(x * g, -1, 1), sr, subtype="PCM_16")
            files[out.name] = sha(out)
            total += len(x) / sr
            rec["slots"][slot] = {"system": system, "source": str(src.relative_to(MAIN)), "source_sha256": sha(src),
                                  "gain_db": round(20 * float(np.log10(g)), 2)}
        K["questions"][qid] = rec
        qs.append(question(qid, n))
    if len(files) > MAX_FILES or total > MAX_SECONDS:
        sys.exit(f"超限：{len(files)} 段、{total:.0f} s（上限 {MAX_FILES} 段、{MAX_SECONDS:.0f} s）")
    page = PAGE.replace("__QUESTIONS__", "\n".join(qs)).replace("__BUILD__", bid).replace(
        "__QIDS__", json.dumps(list(K["questions"])))
    (listen / "index.html").write_text(page)
    files["index.html"] = sha(listen / "index.html")
    check_listen_dir(listen, files, page)
    K["total_seconds"] = round(total, 1)
    (key / "unblind-key.json").write_text(json.dumps(K, ensure_ascii=False, indent=1))
    (key / "manifest.json").write_text(json.dumps({"build": bid, "listen_files_sha256": files}, ensure_ascii=False, indent=1))
    D.mkdir(parents=True)
    for f in ["unblind-key.json", "manifest.json"]:
        shutil.copyfile(key / f, D / f)
    print(f"给人听：_tmp/exp016-listen/{bid}/listen/（{len(files) - 1} 段，{total:.0f} s）")
    print(f"揭盲表（已复制进仓）：{D.relative_to(HERE.parents[2])}")


if __name__ == "__main__":
    main()
