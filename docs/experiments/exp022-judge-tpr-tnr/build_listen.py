"""exp022 · 校准评估器的盲听页：约 20 题，每题只问「开会能用吗」「有没有吃字」（人定 2026-11-13）。

用途：给 `vdn-eval calib` 的过关线混淆矩阵攒人耳真值（调研 031 第 1、6 条）。
音频取自已有成绩单 run 的 `qa/asr-wav/`（成绩单打分时那份输出，不重跑模型）：
GTCRN · 参考降噪器 · DFN3 低延迟版 · DPDFNet-2，各 480 段，每段 12 s。

选题（事先登记，种子固定，写进揭盲表）：
- 每段先按评估器分层：「评估器判不过」= 两个评分模型任一 Δ < −SC-NRMOS-DELTA，或吃字多错 ≥ fail_chars；其余「判过」。
  物理硬事实项（延迟等）人听不出，不参与分层。过关线运行时从 `src/eval/src/thresholds/table.rs` 读
- 判不过只占约 3.5%，纯随机抽几乎全是判过，量不了真阳性率；所以两层各抽 N_PER 题。
  **分层抽样会让两层的比例失真**：揭盲表记下每层的总数与抽样比例，算真阳性率 / 真阴性率时要按比例加权回总体
- 同一个源片段（去掉设备后缀）只出一次，免得两题不独立
- 另选 N_RETEST 题（两层各一）在后半段原样再出一次，量同一个人前后一致不一致

给人的目录里只有页面和 q01.wav 这类无名文件；揭盲表写在听测目录之外，并复制进仓
`docs/experiments/exp022-judge-tpr-tnr/builds/<build>/`（CLAUDE.md 第五节第 1 条）。

用法（项目根下，要 soundfile / pyloudnorm）：
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp022-judge-tpr-tnr/build_listen.py [build-id]
"""
import hashlib
import json
import random
import shutil
import sys
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import tpr_tnr  # noqa: E402  过关线读取（正本 table.rs）

ROOT = HERE.parents[2]
MAIN = ROOT.parent / "noise_canceling"  # 主目录：音频与 _tmp 在这里（worktree 的 _tmp 不放大件）
SCORECARD = MAIN / "_tmp/scorecard"
RUNS = {  # run → 取哪些系统
    "20261105-005916-4ccf0638": ["gtcrn", "ref"],
    "dfn3-ll-seeded-20261105": ["dfn3-ll"],
    "dpdfnet2-seeded-20261105": ["dpdfnet2"],
}
ASR_MODEL = "qwen3-asr-plus:1.7b"
SEED = 20261113
N_PER = 9
N_RETEST = 2
LUFS, PEAK = -26.0, 10 ** (-1 / 20)
# 页面与文件名里不许出现的字样（模型名、场景名、揭盲相关）
FORBIDDEN = ["gtcrn", "dfn3", "dpdf", "deepfilter", "ref-", "unblind", "manifest", "-ext", "-mba",
             "cafe", "street", "transit", "rail", "office", "home-", "canteen", "cowork", "hotel",
             "library", "meeting-room", "phone-booth", "car-passenger", "bus-metro"]


# ── 纯函数（单测覆盖，不碰音频）────────────────────────────────────
def source_of(clip):
    """去掉设备后缀：同一句话的两种设备版本算同一个源片段。"""
    for suf in ("-ext", "-mba"):
        if clip.endswith(suf):
            return clip[: -len(suf)]
    return clip


def judge_of(row, lines):
    """一段的评估器判定（只看人听得出的：听感两项 + 吃字）。判不了返回 None。"""
    s = {x["v"]: x for x in row["scores"]}
    try:
        q = s["V-NRMOS-SCOREQ"]["value"]
        d = s["V-NRMOS-DISTILL"]["value"]
        e = s["V-EAT-ASR"]
        n = round(e["value"] / 100 * e["extra"]["ref_len"])  # 与 rules_v2::eat_chars 同式
    except (KeyError, TypeError):
        return None
    slack = -lines["nrmos_slack_v2"]
    listen_fail = q < slack - 1e-12 or d < slack - 1e-12
    eat_fail = n >= lines["eat_count_v2"]
    return {"verdict": "fail" if (listen_fail or eat_fail) else "pass",
            "d_scoreq": q, "d_distill": d, "eat_extra": n,
            "listen_fail": listen_fail, "eat_fail": eat_fail}


def select(pool, n_per, n_retest, seed):
    """pool: [{run, system, clip, judge}]。两层各抽 n_per，源片段不重复；再定重测题与题序。"""
    rng = random.Random(seed)
    strata = {"fail": [], "pass": []}
    for it in sorted(pool, key=lambda x: (x["run"], x["system"], x["clip"])):
        strata[it["judge"]["verdict"]].append(it)
    used, picks = set(), []
    for name in ("fail", "pass"):
        cand = strata[name][:]
        rng.shuffle(cand)
        got = 0
        for it in cand:
            if got == n_per:
                break
            if source_of(it["clip"]) in used:
                continue
            used.add(source_of(it["clip"]))
            picks.append(dict(it, stratum=name))
            got += 1
        if got < n_per:
            raise ValueError(f"{name} 层只有 {got} 个不重复源片段，不够 {n_per}")
    rng.shuffle(picks)
    order = [dict(p, retest_of=None) for p in picks]
    # 重测：两层各取第一个出现在前半段的题，原样放到最后几题（离原题至少隔半套题）
    half = len(order) // 2
    for name in ("fail", "pass")[:n_retest]:
        src = next(i for i, p in enumerate(order[:half]) if p["stratum"] == name and p["retest_of"] is None)
        order.append(dict(order[src], retest_of=src + 1))
    sizes = {k: len(v) for k, v in strata.items()}
    frac = {k: (n_per / sizes[k]) if sizes[k] else None for k in sizes}
    return order, sizes, frac


def forbidden_hits(texts):
    """给人的目录里的文件名与页面文字，命中禁用字样的列出来。"""
    hits = []
    for name, t in texts:
        low = t.lower()
        hits += [(name, w) for w in FORBIDDEN if w in low]
    return hits


# ── 读数据 · 写音频 ───────────────────────────────────────────────
def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def wav_of(run, clip, kind):
    d = SCORECARD / run / "qa/asr-wav"
    got = sorted(d.glob(f"{clip}#*-{kind}-*.wav"))
    if len(got) != 1:
        raise FileNotFoundError(f"{run} {clip} {kind}：找到 {len(got)} 个")
    return got[0]


def ref_text(run, clip):
    """干净录音的识别结果（评估器算吃字时的参照文本），供人核对吃字。"""
    cache = json.loads((SCORECARD / run / "qa/asr_cache.json").read_text())
    h = sha(wav_of(run, clip, "clean"))
    for lang in ("zh", "en"):
        t = cache.get(f"{h}|{ASR_MODEL}|{lang}|1")
        if t is not None:
            return t, lang
    return "", "?"


def load_pool(lines):
    pool = []
    for run, systems in RUNS.items():
        for line in (SCORECARD / run / "clips.jsonl").read_text().splitlines():
            r = json.loads(line)
            if r["system"] not in systems:
                continue
            j = judge_of(r, lines)
            if j is not None:
                pool.append({"run": run, "system": r["system"], "clip": r["clip"],
                             "scene": r["scene"], "tier": r["tier"], "judge": j})
    return pool


def write_pair(out_main, out_mix, main_src, mix_src):
    import numpy as np
    import pyloudnorm as pyln
    import soundfile as sf
    a, sr = sf.read(main_src, dtype="float64")
    b, sr2 = sf.read(mix_src, dtype="float64")
    assert sr == sr2
    meter = pyln.Meter(sr)
    ga = 10 ** ((LUFS - meter.integrated_loudness(a)) / 20)
    gb = 10 ** ((LUFS - meter.integrated_loudness(b)) / 20)
    peak = max(np.abs(a * ga).max(), np.abs(b * gb).max())
    shrink = min(1.0, PEAK / peak) if peak > 0 else 1.0  # 任一边会削顶时两边一起降
    sf.write(out_main, a * ga * shrink, sr, subtype="PCM_16")
    sf.write(out_mix, b * gb * shrink, sr, subtype="PCM_16")
    return {"gain_db_main": 20 * np.log10(ga * shrink), "gain_db_mix": 20 * np.log10(gb * shrink)}


def main():
    build = sys.argv[1] if len(sys.argv) > 1 else datetime.now().strftime("%Y%m%d-%H%M%S") + "-" + \
        hashlib.sha256(str(random.random()).encode()).hexdigest()[:4]
    base = MAIN / "_tmp/exp022-listen" / build
    listen, key = base / "listen", base / "key"
    if base.exists():
        raise SystemExit(f"{base} 已存在：不覆盖旧 build（CLAUDE.md 第五节第 1 条）")
    listen.mkdir(parents=True)
    key.mkdir()
    lines = tpr_tnr.read_lines()
    order, sizes, frac = select(load_pool(lines), N_PER, N_RETEST, SEED)
    qs, rows = [], []
    for i, it in enumerate(order, 1):
        main_src = wav_of(it["run"], it["clip"], it["system"])
        mix_src = wav_of(it["run"], it["clip"], "mix")
        fm, fx = f"q{i:02d}.wav", f"q{i:02d}_orig.wav"
        gains = write_pair(listen / fm, listen / fx, main_src, mix_src)
        text, lang = ref_text(it["run"], it["clip"])
        qs.append({"q": i, "audio": fm, "orig": fx, "text": text})
        rows.append({"q": i, **{k: it[k] for k in ("run", "system", "clip", "scene", "tier", "stratum",
                                                     "retest_of", "judge")},
                     "lang": lang, "ref_text": text, **gains,
                     "sha256_src": sha(main_src), "sha256_mix_src": sha(mix_src),
                     "sha256_out": sha(listen / fm), "sha256_out_orig": sha(listen / fx)})
    tpl = (HERE / "listen_template.html").read_text()
    html = tpl.replace("__BUILD__", build).replace("__QS__", json.dumps(qs, ensure_ascii=False))
    (listen / "index.html").write_text(html)
    hits = forbidden_hits([(p.name, p.name) for p in listen.iterdir()] + [("index.html", html)])
    if hits:
        shutil.rmtree(listen)
        raise SystemExit(f"给人的目录里出现禁用字样，已删除听测目录：{hits}")
    unblind = {"build": build, "seed": SEED, "n_per_stratum": N_PER, "n_retest": N_RETEST,
               "stratum_sizes": sizes, "sampling_fraction": frac, "lines": lines,
               "runs": RUNS, "lufs": LUFS, "questions": rows,
               "note": "分层抽样：算 TPR/TNR 时按 1/sampling_fraction 加权回总体；retest_of = 第几题的重测"}
    (key / "unblind-key.json").write_text(json.dumps(unblind, ensure_ascii=False, indent=1) + "\n")
    repo = HERE / "builds" / build
    repo.mkdir(parents=True)
    shutil.copy(key / "unblind-key.json", repo / "unblind-key.json")
    (repo / "listen-files.sha256").write_text(
        "".join(f"{sha(p)}  {p.name}\n" for p in sorted(listen.iterdir())))
    print(f"给人听：{listen}\n揭盲表：{key}（已复制进 {repo}）\n共 {len(order)} 题")


if __name__ == "__main__":
    main()
