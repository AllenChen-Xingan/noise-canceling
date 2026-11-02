"""exp011：离线模拟本机闭环——评分模型当奖励逐窗选参数，比查表。设计见同目录 README.md。

用法（按顺序；每步可重跑，结果按 build id 落盘，不覆盖旧 build）：
  python exp011.py select                 # 选段、按说话人切两半 → builds/<id>/clips.json（新 build id）
  python exp011.py enhance  <build>       # GTCRN（+ 路由到 FE-M 的段跑 FE-M）一次 + 修音链 → 缓存
  python exp011.py post     <build>       # 8 组候选 × 每段：后处理 + 红线（Rust 驱动）
  python exp011.py reward   <build>       # 奖励 A：DistillMOS，每候选每 2 s 窗（要 distillmos 包）
  python exp011.py assemble <build>       # 五种做法拼出最终输出（报分半）+ 红线
  python exp011.py judge    <build>       # 留出 B（SCOREQ）· C（DNSMOS）· 吃字（Qwen3-ASR）
  python exp011.py report   <build>       # 汇总 → builds/<id>/results.json + results.md

音频与缓存：主目录 _tmp/exp011/<build>/（git 忽略）；小文件：本目录 builds/<build>/（提交）。
解释器：主目录 _tmp/exp002-venv（numpy · soundfile · onnxruntime · pyloudnorm）；reward / judge 步用装了评分模型的 venv（EXP011_PY_SCORE）。
"""
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
WT = HERE.parents[2]
EXP3 = WT / "docs/experiments/exp003-offline-ab"
FED2 = "20261030235558-fed2"


def main_root():
    common = subprocess.run(["git", "-C", str(WT), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                            capture_output=True, text=True, check=True).stdout.strip()
    return Path(common).parent


MAIN = main_root()
FED2_AUDIO = MAIN / f"_tmp/20261031-exp006/{FED2}/test"
FED2_TRUTH = WT / f"docs/experiments/exp006-scene-sim/builds/{FED2}/truth.jsonl"
SR = 16000
WIN_S, CTX_S = 2.0, 4.0           # 每 2 s 决策一次；奖励看最近 4 s
XFADE_S = 0.05                    # 窗间等增益交叉淡化
PER_CLASS = 12
SEED = 20261102

# 候选参数（README 第 3 节）；c0 = 默认
CANDS = {
    "c0": {"floor_db": -12, "detect_db": 9, "strength": 1.0},
    "c1": {"floor_db": -6, "detect_db": 9, "strength": 1.0},
    "c2": {"floor_db": -20, "detect_db": 9, "strength": 1.0},
    "c3": {"floor_db": -12, "detect_db": 6, "strength": 1.0},
    "c4": {"floor_db": -12, "detect_db": 12, "strength": 1.0},
    "c5": {"floor_db": -6, "detect_db": 9, "strength": 0.8},
    "c6": {"floor_db": -12, "detect_db": 9, "strength": 0.8},
    "c7": {"floor_db": -20, "detect_db": 6, "strength": 1.0},
}
# 路由表（src/scene/src/route.rs params_for）：细场景 → (模型, 修音预设)；窄带蓝牙且 GTCRN 时预设 +10
ROUTE = {"quiet": ("gtcrn", 1), "hvac": ("gtcrn", 1), "babble": ("fem", 22), "clatter": ("gtcrn", 2),
         "neighbor": ("gtcrn", 3), "traffic": ("gtcrn", 4), "music": ("fem", 25)}
REDLINES = ("V-NAN", "V-ONSET", "V-SPLVL")      # 一票否决；Unknown 不算否决（记数另报）


def route(c):
    m, t = ROUTE[c["fine"]]
    if m == "gtcrn" and c["device"] == "bt_narrow":
        t += 10
    return m, t


def bdir(build):
    return HERE / "builds" / build


def adir(build):
    return MAIN / "_tmp/exp011" / build


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def clips(build):
    return json.loads((bdir(build) / "clips.json").read_text())["clips"]


def driver(jobs):
    """Rust 驱动：JSONL 进，JSONL 出。"""
    exe = WT / "target/release/exp011-driver"
    if not exe.exists():
        raise SystemExit(f"先编译驱动：CARGO_TARGET_DIR={WT}/target cargo build --release --manifest-path {HERE}/driver/Cargo.toml")
    p = subprocess.run([str(exe)], input="".join(json.dumps(j) + "\n" for j in jobs), capture_output=True, text=True)
    if p.returncode:
        raise RuntimeError(p.stderr[-2000:])
    return [json.loads(l) for l in p.stdout.splitlines() if l.strip()]


def redline_ok(scores):
    return all(scores[v]["verdict"] != "fail" for v in REDLINES)


# ---------------------------------------------------------------- select
def cmd_select():
    truth = [json.loads(l) for l in FED2_TRUTH.read_text().splitlines() if l.strip()]
    rng = np.random.default_rng(SEED)
    chosen = []
    for fine in sorted(ROUTE):
        pool = [t for t in truth if t.get("split") == "test" and t.get("fine") == fine and t.get("has_target") and t["device"] in ("none", "bt_narrow")
                and (FED2_AUDIO / f"{t['id']}_clean.wav").exists()]
        idx = rng.choice(len(pool), size=min(PER_CLASS, len(pool)), replace=False)
        chosen += [pool[i] for i in sorted(idx)]
    # 按说话人切两半：说话人随机排序后交替分，使两半段数接近
    spk = sorted({c["target_spk"] for c in chosen})
    rng.shuffle(spk)
    half, cnt = {}, {"tune": 0, "report": 0}
    for s in spk:
        n = sum(c["target_spk"] == s for c in chosen)
        h = "tune" if cnt["tune"] <= cnt["report"] else "report"
        half[s] = h
        cnt[h] += n
    out = []
    for c in chosen:
        m, t = route(c)
        out.append({"id": c["id"], "fine": c["fine"], "device": c["device"], "snr_db": c["snr_db"], "target_spk": c["target_spk"],
                    "half": half[c["target_spk"]], "route_model": m, "route_tone": t,
                    "noisy_sha256": sha(FED2_AUDIO / f"{c['id']}_noisy.wav"),
                    "clean_sha256": sha(FED2_AUDIO / f"{c['id']}_clean.wav")})
    build = time.strftime("%Y%m%d-%H%M%S") + "-" + hashlib.sha256(json.dumps(out).encode()).hexdigest()[:4]
    bdir(build).mkdir(parents=True)
    meta = {"build": build, "source": f"exp006 {FED2} test", "seed": SEED, "per_class": PER_CLASS, "halves": cnt,
            "n_speakers": len(spk), "cands": CANDS, "route": ROUTE, "win_s": WIN_S, "ctx_s": CTX_S, "xfade_s": XFADE_S,
            "redlines": REDLINES}
    (bdir(build) / "clips.json").write_text(json.dumps({"meta": meta, "clips": out}, ensure_ascii=False, indent=1))
    print(build, cnt, len(spk), "speakers")


# ---------------------------------------------------------------- enhance
def cmd_enhance(build):
    sys.path.insert(0, str(EXP3))
    import enhance as e3  # exp003：只调用
    import shutil
    d = adir(build) / "cache"
    d.mkdir(parents=True, exist_ok=True)
    polish = WT / "target/release/examples/polish_offline"
    if not polish.exists():
        raise SystemExit(f"先编译修音链：cargo build --release -p vdn-polish --example polish_offline（在 {WT}）")
    rows = []
    for c in clips(build):
        cid = c["id"]
        x, sr = sf.read(FED2_AUDIO / f"{cid}_noisy.wav", dtype="float32")
        assert sr == SR
        shutil.copy(FED2_AUDIO / f"{cid}_clean.wav", d / f"{cid}_clean.wav")
        sf.write(d / f"{cid}_noisy.wav", x, SR, subtype="FLOAT")
        t0 = time.perf_counter()
        sf.write(d / f"{cid}_gtcrn.wav", e3.stream(e3.Gtcrn(), x), SR, subtype="FLOAT")
        tg = time.perf_counter() - t0
        if c["route_model"] == "fem":
            sf.write(d / f"{cid}_fem.wav", e3.FastEnhancer().run(x), SR, subtype="FLOAT")
        # 两路修音：默认（GTCRN + 9）与查表（路由模型 + 路由预设）
        for tag, model, tone in (("def", "gtcrn", 9), ("tab", c["route_model"], c["route_tone"])):
            subprocess.run([str(polish), str(d / f"{cid}_noisy.wav"), str(d / f"{cid}_{model}.wav"),
                            str(d / f"{cid}_wet_{tag}.wav"), str(tone)], capture_output=True, text=True, check=True)
        rows.append({"id": cid, "gtcrn_s": round(tg, 3)})
        print(cid, c["fine"], c["route_model"], c["route_tone"], f"{tg:.2f}s", flush=True)
    (bdir(build) / "enhance.json").write_text(json.dumps(rows, indent=0))


# ---------------------------------------------------------------- post
def cmd_post(build):
    d = adir(build)
    (d / "cand").mkdir(exist_ok=True)
    jobs = []
    for c in clips(build):
        cid = c["id"]
        base = {"dry": str(d / f"cache/{cid}_noisy.wav"), "clean": str(d / f"cache/{cid}_clean.wav")}
        jobs.append({**base, "id": f"{cid}|default", "wet": str(d / f"cache/{cid}_wet_def.wav"),
                     "out": str(d / f"cand/{cid}_default.wav"), **CANDS["c0"]})
        for k, p in CANDS.items():
            jobs.append({**base, "id": f"{cid}|{k}", "wet": str(d / f"cache/{cid}_wet_tab.wav"),
                         "out": str(d / f"cand/{cid}_{k}.wav"), **p})
    t0 = time.perf_counter()
    res = driver(jobs)
    print(len(res), "jobs", f"{time.perf_counter() - t0:.1f}s")
    with open(bdir(build) / "post.jsonl", "w") as f:
        for r in res:
            f.write(json.dumps(r) + "\n")


def post_scores(build):
    return {r["id"]: r for r in map(json.loads, (bdir(build) / "post.jsonl").read_text().splitlines())}


# ---------------------------------------------------------------- reward（A：DistillMOS）
def windows(n):
    w = int(WIN_S * SR)
    return [(i * w, min(n, (i + 1) * w)) for i in range((n + w - 1) // w)]


def cmd_reward(build):
    import torch
    import distillmos
    torch.set_num_threads(2)
    m = distillmos.ConvTransformerSQAModel()
    m.eval()
    d = adir(build)
    out = {}
    t0 = time.perf_counter()
    for c in clips(build):
        cid = c["id"]
        for k in ["default", *CANDS]:
            y, _ = sf.read(d / f"cand/{cid}_{k}.wav", dtype="float32")
            seg = []
            for a, b in windows(len(y)):
                lo = max(0, b - int(CTX_S * SR))
                with torch.no_grad():
                    seg.append(float(m(torch.from_numpy(y[lo:b]).unsqueeze(0)).item()))
            with torch.no_grad():
                whole = float(m(torch.from_numpy(y).unsqueeze(0)).item())
            out[f"{cid}|{k}"] = {"win": seg, "whole": whole}
        print(cid, flush=True)
    meta = {"model": "distillmos ConvTransformerSQAModel", "version": getattr(distillmos, "__version__", "?"),
            "secs": round(time.perf_counter() - t0, 1)}
    (bdir(build) / "reward_A.json").write_text(json.dumps({"meta": meta, "scores": out}, indent=0))


# ---------------------------------------------------------------- assemble
def stitch(parts, n):
    """parts: [(起点样本, 波形全长)…]；窗边界处等增益交叉淡化 XFADE_S。"""
    y = np.zeros(n, np.float32)
    xf = int(XFADE_S * SR)
    bounds = [a for a, _ in parts] + [n]
    for i, (a, w) in enumerate(parts):
        b = bounds[i + 1]
        lo = max(0, a - xf // 2) if i else 0
        hi = min(n, b + xf // 2) if i < len(parts) - 1 else n
        g = np.ones(hi - lo, np.float32)
        if i:
            r = np.arange(min(xf, hi - lo), dtype=np.float32) / max(xf, 1)
            g[: len(r)] = r
        if i < len(parts) - 1:
            r = 1 - np.arange(min(xf, hi - lo), dtype=np.float32) / max(xf, 1)
            g[len(g) - len(r):] *= r
        y[lo:hi] += g * w[lo:hi]
    return y


def cmd_assemble(build):
    cl = clips(build)
    post = post_scores(build)
    A = json.loads((bdir(build) / "reward_A.json").read_text())["scores"]
    ok = {i: redline_ok(r["scores"]) for i, r in post.items()}
    # 调好的固定参数：调参半上，红线全过占比最高，其次 A 整段均分最高
    tune = [c["id"] for c in cl if c["half"] == "tune"]
    rank = sorted(CANDS, key=lambda k: (-np.mean([ok[f"{i}|{k}"] for i in tune]), -np.mean([A[f"{i}|{k}"]["whole"] for i in tune])))
    tuned = rank[0]
    d = adir(build)
    (d / "final").mkdir(exist_ok=True)
    choice, jobs = {}, []
    for c in cl:
        if c["half"] != "report":
            continue
        cid = c["id"]
        safe = [k for k in CANDS if ok[f"{cid}|{k}"]] or ["c0"]
        oracle = max(safe, key=lambda k: A[f"{cid}|{k}"]["whole"])
        ys = {k: sf.read(d / f"cand/{cid}_{k}.wav", dtype="float32")[0] for k in CANDS}
        n = len(ys["c0"])
        seq, cur = [], "c0"                       # 第一窗用查表（= c0 + 路由预设）
        for wi, (a, _) in enumerate(windows(n)):
            if wi:
                cur = max(safe, key=lambda k: A[f"{cid}|{k}"]["win"][wi - 1])
            seq.append(cur)
        online = stitch([(a, ys[k]) for (a, _), k in zip(windows(n), seq)], n)
        sf.write(d / f"final/{cid}_online.wav", online, SR, subtype="FLOAT")
        choice[cid] = {"tuned": tuned, "oracle": oracle, "online": seq, "n_safe": len(safe)}
        files = {"default": d / f"cand/{cid}_default.wav", "table": d / f"cand/{cid}_c0.wav",
                 "tuned": d / f"cand/{cid}_{tuned}.wav", "online": d / f"final/{cid}_online.wav",
                 "oracle": d / f"cand/{cid}_{oracle}.wav"}
        for sysname, p in files.items():
            jobs.append({"id": f"{cid}|{sysname}", "dry": str(d / f"cache/{cid}_noisy.wav"),
                         "clean": str(d / f"cache/{cid}_clean.wav"), "score": str(p)})
    res = driver(jobs)
    with open(bdir(build) / "final.jsonl", "w") as f:
        for j, r in zip(jobs, res):
            r["file"] = j["score"]
            r["sha256"] = sha(j["score"])
            f.write(json.dumps(r) + "\n")
    (bdir(build) / "choice.json").write_text(json.dumps({"tuned": tuned, "tuned_rank": rank, "clips": choice}, indent=1))
    print("tuned =", tuned, "rank", rank)


# ---------------------------------------------------------------- judge（B · C · 吃字）
ASR_URL = "http://127.0.0.1:11520/v1/audio/transcriptions"
ASR_MODEL = "qwen3-asr-plus:1.7b"


def asr(path, cache):
    key = sha(path)
    if key not in cache:
        o = subprocess.run(["curl", "-sS", "-f", "-m", "600", "-X", "POST", ASR_URL, "-H",
                            f"Authorization: Bearer {os.environ.get('VDN_ASR_KEY', '42model')}",
                            "-F", f"model={ASR_MODEL}", "-F", "language=en", "-F", f"file=@{path};type=audio/wav"],
                           capture_output=True, text=True, check=True)
        cache[key] = json.loads(o.stdout).get("text", "")
    return cache[key]


def words(t):
    return "".join(ch.lower() if ch.isalnum() or ch == "'" else " " for ch in t).split()


def del_sub(ref, hyp):
    """词级编辑距离里的删除 + 替换数（吃字只看这两项）。"""
    r, h = words(ref), words(hyp)
    D = np.zeros((len(r) + 1, len(h) + 1), int)
    D[:, 0] = np.arange(len(r) + 1)
    D[0, :] = np.arange(len(h) + 1)
    for i in range(1, len(r) + 1):
        for j in range(1, len(h) + 1):
            D[i, j] = min(D[i - 1, j] + 1, D[i, j - 1] + 1, D[i - 1, j - 1] + (r[i - 1] != h[j - 1]))
    i, j, ds = len(r), len(h), 0
    while i and j:
        if D[i, j] == D[i - 1, j - 1] + (r[i - 1] != h[j - 1]):
            ds += r[i - 1] != h[j - 1]
            i, j = i - 1, j - 1
        elif D[i, j] == D[i - 1, j] + 1:
            ds, i = ds + 1, i - 1
        else:
            j -= 1
    return ds + i, len(r)


def cmd_judge(build):
    fin = [json.loads(l) for l in (bdir(build) / "final.jsonl").read_text().splitlines()]
    d = adir(build)
    out = {}
    # C：DNSMOS（本仓 Rust）
    vdn_eval = WT / "target/release/vdn-eval"
    dns = MAIN / "resources/DNS-Challenge/DNSMOS/DNSMOS/sig_bak_ovr.onnx"
    for r in fin:
        o = subprocess.run([str(vdn_eval), "dnsmos", str(dns), r["file"]], capture_output=True, text=True, check=True)
        out.setdefault(r["id"], {})["C"] = json.loads(o.stdout.strip().splitlines()[-1])
    # B：SCOREQ（自然域、无参考）
    import scoreq
    sq = scoreq.Scoreq(data_domain="natural", mode="nr")
    for r in fin:
        out[r["id"]]["B"] = float(sq.predict(test_path=r["file"], ref_path=None))
    # A：整段（与奖励同一模型；只作对照）
    import torch
    import distillmos
    m = distillmos.ConvTransformerSQAModel()
    m.eval()
    for r in fin:
        y, _ = sf.read(r["file"], dtype="float32")
        with torch.no_grad():
            out[r["id"]]["A"] = float(m(torch.from_numpy(y).unsqueeze(0)).item())
    # 吃字：参考 = 干净参考的转写
    cp = d / "asr_cache.json"
    cache = json.loads(cp.read_text()) if cp.exists() else {}
    for r in fin:
        cid = r["id"].split("|")[0]
        ref = asr(d / f"cache/{cid}_clean.wav", cache)
        hyp = asr(r["file"], cache)
        out[r["id"]]["asr"] = {"del_sub": del_sub(ref, hyp)[0], "ref_words": len(words(ref))}
        cp.write_text(json.dumps(cache))
    (bdir(build) / "judge.json").write_text(json.dumps(out, indent=0))


# ---------------------------------------------------------------- report
SYSTEMS = ["default", "table", "tuned", "online", "oracle"]


def boot_ci(diffs_by_spk, n=2000, seed=SEED):
    rng = np.random.default_rng(seed)
    spk = list(diffs_by_spk)
    means = []
    for _ in range(n):
        pick = rng.choice(len(spk), size=len(spk), replace=True)
        v = np.concatenate([diffs_by_spk[spk[i]] for i in pick])
        means.append(v.mean())
    return float(np.percentile(means, 2.5)), float(np.percentile(means, 97.5))


def cmd_report(build):
    cl = {c["id"]: c for c in clips(build)}
    fin = {r["id"]: r for r in map(json.loads, (bdir(build) / "final.jsonl").read_text().splitlines())}
    J = json.loads((bdir(build) / "judge.json").read_text())
    ids = sorted({k.split("|")[0] for k in fin})
    tab = {}
    for s in SYSTEMS:
        ks = [f"{i}|{s}" for i in ids]
        tab[s] = {
            "n": len(ks),
            "B_scoreq": float(np.mean([J[k]["B"] for k in ks])),
            "C_dnsmos_ovrl": float(np.mean([J[k]["C"]["ovrl"] for k in ks])),
            "C_dnsmos_sig": float(np.mean([J[k]["C"]["sig"] for k in ks])),
            "C_dnsmos_bak": float(np.mean([J[k]["C"]["bak"] for k in ks])),
            "A_distillmos": float(np.mean([J[k]["A"] for k in ks])),
            "redline_pass": float(np.mean([redline_ok(fin[k]["scores"]) for k in ks])),
            "redline_unknown": int(sum(any(fin[k]["scores"][v]["verdict"] == "unknown" for v in REDLINES) for k in ks)),
            "eat_rate": float(sum(J[k]["asr"]["del_sub"] for k in ks) / max(1, sum(J[k]["asr"]["ref_words"] for k in ks))),
        }
    pairs = {}
    for a, b in (("online", "table"), ("online", "tuned"), ("oracle", "tuned"), ("tuned", "table"), ("table", "default")):
        for metric, get in (("B", lambda k: J[k]["B"]), ("C", lambda k: J[k]["C"]["ovrl"]), ("A", lambda k: J[k]["A"])):
            by = {}
            for i in ids:
                by.setdefault(cl[i]["target_spk"], []).append(get(f"{i}|{a}") - get(f"{i}|{b}"))
            by = {s: np.array(v) for s, v in by.items()}
            allv = np.concatenate(list(by.values()))
            pairs[f"{a}-{b}|{metric}"] = {"mean": float(allv.mean()), "ci95": boot_ci(by)}
    per_fine = {}
    for fine in sorted({cl[i]["fine"] for i in ids}):
        fi = [i for i in ids if cl[i]["fine"] == fine]
        per_fine[fine] = {s: float(np.mean([J[f"{i}|{s}"]["B"] for i in fi])) for s in SYSTEMS} | {"n": len(fi)}
    res = {"build": build, "systems": tab, "pairs": pairs, "per_fine_B": per_fine}
    (bdir(build) / "results.json").write_text(json.dumps(res, ensure_ascii=False, indent=1))
    print(json.dumps(res, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    cmd, *rest = sys.argv[1:]
    {"select": cmd_select, "enhance": cmd_enhance, "post": cmd_post, "reward": cmd_reward,
     "assemble": cmd_assemble, "judge": cmd_judge, "report": cmd_report}[cmd](*rest)
