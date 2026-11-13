"""exp022 · 校准评估器的盲听页：约 20 题，每题只问「开会能用吗」「有没有吃字」（人定 2026-11-13）。

用途：给 `vdn-eval calib` 的过关线混淆矩阵攒人耳真值（调研 031 第 1、6 条）。
音频取自已有成绩单 run 的 `qa/asr-wav/`（成绩单打分时那份输出，不重跑模型）：
GTCRN · 参考降噪器 · DFN3 低延迟版 · DPDFNet-2，各 480 段，每段 12 s。

选题（事先登记，种子固定，写进揭盲表）：
- 每段先按评估器分层：「评估器判不过」= 两个评分模型任一 Δ < −SC-NRMOS-DELTA，或吃字多错 ≥ fail_chars；其余「判过」。
  物理硬事实项（延迟等）人听不出，不参与分层。过关线运行时从 `src/eval/src/thresholds/table.rs` 读
- 判不过只占约 3.5%，纯随机抽几乎全是判过，量不了真阳性率；所以两层各抽 N_PER 题（不放回简单随机）。
- 同一个源片段（去掉设备后缀）只出一次：整套里有重复就整套重抽。这会让入样概率偏离 n/N，
  所以按同一设计模拟 SIMS 次估计每条的入样概率，连同抽样框写进揭盲表；估总体率时按它做逆概率加权
- 人耳真值怎么切、这一轮拿来干什么，听测前写进揭盲表（PREREG）
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
MAIN = ROOT.parent / "noise_canceling"  # 主目录：成绩单音频在这里，只读
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
SIMS = 200_000  # 入样概率模拟次数：判过层 π ≈ 0.005，蒙特卡罗相对误差约 3%
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
            "d_scoreq": q, "d_distill": d, "eat_extra": n, "ref_len": e["extra"]["ref_len"],
            "listen_fail": listen_fail, "eat_fail": eat_fail}


def frame_of(pool):
    """抽样框：按（run, 系统, 片段）排序后分两层。"""
    strata = {"fail": [], "pass": []}
    for it in sorted(pool, key=lambda x: (x["run"], x["system"], x["clip"])):
        strata[it["judge"]["verdict"]].append(it)
    return strata


def draw(strata, n_per, rng, max_tries=100000):
    """抽样设计（codex 第 2 轮 P1 #1）：两层各做一次不放回简单随机抽样；整套里同一源片段出现两次就整套重抽。
    入样概率由 [`inclusion_probs`] 按同一设计模拟估计，不用 n/N 近似。"""
    for t in range(max_tries):
        picks = [dict(it, stratum=k) for k in ("fail", "pass") for it in rng.sample(strata[k], n_per)]
        if len({source_of(p["clip"]) for p in picks}) == len(picks):
            return picks, t + 1
    raise ValueError(f"{max_tries} 次都抽不出源片段不重复的一套")


def item_key(it):
    return f'{it["run"]}|{it["system"]}|{it["clip"]}'


def inclusion_probs(strata, n_per, seed, sims):
    """照 [`draw`] 的设计模拟 sims 次，估计每条的入样概率（命中次数 / sims）。"""
    rng = random.Random(seed)
    hits = {}
    for _ in range(sims):
        picks, _ = draw(strata, n_per, rng)
        for p in picks:
            k = item_key(p)
            hits[k] = hits.get(k, 0) + 1
    return {k: v / sims for k, v in hits.items()}


def place_retests(order, n_retest, rng):
    """重测（codex 第 2 轮 P2 #2）：两层各挑一题当原题，换到前半段（不论洗牌后前半段是不是只有一层），
    再把原样的重测题接在最后，离原题至少隔半套题。"""
    order = [dict(p, retest_of=None) for p in order]
    half = len(order) // 2
    used = set()
    for name in ("fail", "pass")[:n_retest]:
        src = next(i for i, p in enumerate(order) if p["stratum"] == name)
        if src >= half:
            dst = next(i for i in range(half) if i not in used)
            order[src], order[dst] = order[dst], order[src]
            src = dst
        used.add(src)
    for src in sorted(used):
        order.append(dict(order[src], retest_of=src + 1))
    return order


def select(pool, n_per, n_retest, seed):
    """抽题 + 排题。返回（题序, 各层总数, 抽到这套用了几次）。"""
    strata = frame_of(pool)
    for k in strata:
        if len({source_of(it["clip"]) for it in strata[k]}) < n_per:
            raise ValueError(f"{k} 层不重复的源片段不够 {n_per}")
    rng = random.Random(seed)
    picks, tries = draw(strata, n_per, rng)
    rng.shuffle(picks)
    order = place_retests(picks, n_retest, rng)
    return order, {k: len(v) for k, v in strata.items()}, tries


# 人耳真值怎么切、这一轮拿来干什么：听测前写死（codex 第 2 轮 P2 #3 · issue 草稿 p2-8）
PREREG = {
    "usable_cut": "两种切法都报：宽切 no = 不过（主结论）；严切 no 或 marginal = 不过（敏感性）",
    "eaten_cut": "yes = 不过；no = 过；unsure = 判不了（不进吃字的混淆矩阵）",
    "use": "验证集：vdn-eval calib 第 2 轮（偶数轮，只用来考，不用来调线）",
    "weighting": "按 questions[].inclusion_prob 做逆概率加权估总体 TPR/TNR；重测题只用来量前后一致，不进率",
    "retest": "重测原题与重测题的两个答案不一致的比例 = 同一人前后不一致率，只报",
}
REQUIRED_KEY_FIELDS = ("prereg", "frame", "inclusion_sims", "questions", "seed")
REQUIRED_PREREG = ("usable_cut", "eaten_cut", "use", "weighting", "retest")


def check_key(key):
    """揭盲表必填项：顶层字段与事先登记的每个子字段都要在、且非空（codex 第 3 轮 P2 #2）。"""
    missing = [f for f in REQUIRED_KEY_FIELDS if not key.get(f) and key.get(f) != 0]
    pre = key.get("prereg") or {}
    missing += [f"prereg.{f}" for f in REQUIRED_PREREG
                if not (isinstance(pre.get(f), str) and pre[f].strip())]  # null、非字符串也算缺（codex 第 4 轮）
    if missing:
        raise ValueError(f"揭盲表缺字段：{missing}")


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


def make_key(build, lines, sizes, tries, rows, html, pool):
    key = {"build": build, "seed": SEED, "n_per_stratum": N_PER, "n_retest": N_RETEST,
           "stratum_sizes": sizes, "draw_tries": tries, "lines": lines, "runs": RUNS, "lufs": LUFS,
           "prereg": PREREG,
           "design": "两层各不放回简单随机抽样；整套同一源片段重复则整套重抽；入样概率按同一设计模拟估计",
           "inclusion_sims": {"sims": SIMS, "seed": SEED + 1},
           # 抽样框：每条（run|系统|片段）与所在层，供以后复核入样概率
           "frame": {k: [item_key(it) for it in v] for k, v in frame_of(pool).items()},
           "questions": rows,
           "sha256_index_html": hashlib.sha256(html.encode()).hexdigest()}
    check_key(key)
    return key


def main():
    build = sys.argv[1] if len(sys.argv) > 1 else datetime.now().strftime("%Y%m%d-%H%M%S") + "-" + \
        hashlib.sha256(str(random.random()).encode()).hexdigest()[:4]
    base = ROOT / "_tmp/exp022-listen" / build  # 写在本 worktree 的 _tmp（git 忽略）；主目录只读
    listen, key = base / "listen", base / "key"
    if base.exists():
        raise SystemExit(f"{base} 已存在：不覆盖旧 build（CLAUDE.md 第五节第 1 条）")
    listen.mkdir(parents=True)
    key.mkdir()
    lines = tpr_tnr.read_lines()
    pool = load_pool(lines)
    order, sizes, tries = select(pool, N_PER, N_RETEST, SEED)
    probs = inclusion_probs(frame_of(pool), N_PER, SEED + 1, SIMS)
    qs, rows = [], []
    for i, it in enumerate(order, 1):
        main_src = wav_of(it["run"], it["clip"], it["system"])
        mix_src = wav_of(it["run"], it["clip"], "mix")
        fm, fx = f"q{i:02d}.wav", f"q{i:02d}_orig.wav"
        gains = write_pair(listen / fm, listen / fx, main_src, mix_src)
        text, lang = ref_text(it["run"], it["clip"])
        qs.append({"q": i, "audio": fm, "orig": fx, "text": text})
        pi = probs.get(item_key(it), 0.0)
        rows.append({"q": i, **{k: it[k] for k in ("run", "system", "clip", "scene", "tier", "stratum",
                                                     "retest_of", "judge")},
                     "inclusion_prob": pi, "inclusion_prob_mc_se": (pi * (1 - pi) / SIMS) ** 0.5,
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
    unblind = make_key(build, lines, sizes, tries, rows, html, pool)
    (key / "unblind-key.json").write_text(json.dumps(unblind, ensure_ascii=False, indent=1) + "\n")
    repo = HERE / "builds" / build
    repo.mkdir(parents=True)
    shutil.copy(key / "unblind-key.json", repo / "unblind-key.json")
    print(f"给人听：{listen}\n揭盲表：{key}（已复制进 {repo}）\n共 {len(order)} 题")


if __name__ == "__main__":
    main()
