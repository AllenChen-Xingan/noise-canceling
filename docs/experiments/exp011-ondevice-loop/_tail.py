# ---------------------------------------------------------------- post
def make_ref(c, d):
    """红线参考：窄带蓝牙段的带噪输入只有 4 kHz 以下，干净参考也砍到 4 kHz，免得红线量的是模拟器的设备损失（自审 2026-11-02）。"""
    cid = c["id"]
    src = d / f"cache/{cid}_clean.wav"
    if c["device"] != "bt_narrow":
        return src
    dst = d / f"cache/{cid}_ref4k.wav"
    if not dst.exists():
        x, _ = sf.read(src, dtype="float32")
        X = np.fft.rfft(x)
        X[np.fft.rfftfreq(len(x), 1 / SR) > 4000] = 0
        sf.write(dst, np.fft.irfft(X, n=len(x)).astype(np.float32), SR, subtype="FLOAT")
    return dst


def cmd_post(build):
    d = adir(build)
    (d / "cand").mkdir(exist_ok=True)
    jobs = []
    for c in clips(build):
        cid = c["id"]
        base = {"dry": str(d / f"cache/{cid}_noisy.wav"), "clean": str(make_ref(c, d))}
        jobs.append({**base, "id": f"{cid}|default", "wet": str(d / f"cache/{cid}_wet_def.wav"),
                     "detect": str(d / f"cache/{cid}_gtcrn.wav"), "out": str(d / f"cand/{cid}_default.wav"), **CANDS["c0"]})
        for k, p in CANDS.items():
            jobs.append({**base, "id": f"{cid}|{k}", "wet": str(d / f"cache/{cid}_wet_tab.wav"),
                         "detect": str(d / f"cache/{cid}_{c['route_model']}.wav"), "out": str(d / f"cand/{cid}_{k}.wav"), **p})
    t0 = time.perf_counter()
    res = driver(jobs)
    print(len(res), "jobs", f"{time.perf_counter() - t0:.1f}s")
    with open(bdir(build) / "post.jsonl", "w") as f:
        for r in res:
            f.write(json.dumps(r) + "\n")
    cfg = {"cands": CANDS, "route": ROUTE, "win_s": WIN_S, "ctx_s": CTX_S, "xfade_s": XFADE_S, "redlines": REDLINES,
           "ucb_c": UCB_C, "note": "以本文件为准；clips.json 的 meta.cands 是选段时的旧版（c1 / c5 没有 open_db）"}
    (bdir(build) / "config.json").write_text(json.dumps(cfg, ensure_ascii=False, indent=1))


def post_scores(build):
    return {r["id"]: r for r in map(json.loads, (bdir(build) / "post.jsonl").read_text().splitlines())}


# ---------------------------------------------------------------- reward（A：DistillMOS）
def windows(n):
    w = int(WIN_S * SR)
    return [(i * w, min(n, (i + 1) * w)) for i in range((n + w - 1) // w)]


def distill():
    import torch
    import distillmos
    torch.set_num_threads(2)
    m = distillmos.ConvTransformerSQAModel()
    m.eval()

    def f(y):
        with torch.no_grad():
            return float(m(torch.from_numpy(np.ascontiguousarray(y, np.float32)).unsqueeze(0)).item())
    return f


def cmd_reward(build):
    f = distill()
    d = adir(build)
    out = {}
    t0 = time.perf_counter()
    for c in clips(build):
        cid = c["id"]
        for k in ["default", *CANDS]:
            y, _ = sf.read(d / f"cand/{cid}_{k}.wav", dtype="float32")
            seg = [f(y[max(0, b - int(CTX_S * SR)):b]) for _, b in windows(len(y))]
            out[f"{cid}|{k}"] = {"win": seg, "whole": f(y)}
        print(cid, flush=True)
    meta = {"model": "distillmos ConvTransformerSQAModel (distill_mos_v7)", "secs": round(time.perf_counter() - t0, 1)}
    (bdir(build) / "reward_A.json").write_text(json.dumps({"meta": meta, "scores": out}, indent=0))


# ---------------------------------------------------------------- assemble
UCB_C = 0.1   # 只有部分反馈的在线闭环：置信上界的探索系数（MOS 分；事先定，不调）


def stitch(parts, n):
    """parts: [(起点样本, 整段波形)…]；窗边界处等增益线性交叉淡化 XFADE_S。"""
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


def offline_choices(cl, post, A):
    """只用调参半：安全集合 · 调好的固定参数 · 按场景调好的表 · 各候选的先验 A。报分半的红线与分数一概不看。"""
    tune = [c for c in cl if c["half"] == "tune"]
    ok = {i: redline_ok(r["scores"]) for i, r in post.items()}
    rate = {k: float(np.mean([ok[f"{c['id']}|{k}"] for c in tune])) for k in CANDS}
    safe = [k for k in CANDS if rate[k] >= rate["c0"]]          # 出厂安全集合：红线通过率不低于默认
    meanA = {k: float(np.mean([A[f"{c['id']}|{k}"]["whole"] for c in tune])) for k in CANDS}
    tuned = max(safe, key=meanA.get)
    table_tuned, prior = {}, {}
    for fine in sorted({c["fine"] for c in cl}):
        tf = [c for c in tune if c["fine"] == fine] or tune
        mA = {k: float(np.mean([np.mean(A[f"{c['id']}|{k}"]["win"]) for c in tf])) for k in CANDS}
        table_tuned[fine] = max(safe, key=mA.get)
        prior[fine] = mA
    return {"redline_rate_tune": rate, "safe": safe, "meanA_tune": meanA, "tuned": tuned,
            "table_tuned": table_tuned, "prior": prior}


def cmd_assemble(build):
    cl = clips(build)
    post = post_scores(build)
    A = json.loads((bdir(build) / "reward_A.json").read_text())["scores"]
    off = offline_choices(cl, post, A)
    safe = off["safe"]
    d = adir(build)
    (d / "final").mkdir(exist_ok=True)
    choice, jobs = {}, []
    for c in cl:
        if c["half"] != "report":
            continue
        cid, fine = c["id"], c["fine"]
        ys = {k: sf.read(d / f"cand/{cid}_{k}.wav", dtype="float32")[0] for k in CANDS}
        n = len(ys["c0"])
        W = windows(n)
        start = off["table_tuned"][fine]
        # 全信息：每窗看所有安全候选上一窗的 A（要并行跑 8 条链，用户电脑上做不到 → 只当上界）
        full = [start] + [max(safe, key=lambda k: A[f"{cid}|{k}"]["win"][w - 1]) for w in range(1, len(W))]
        # 部分反馈：只看到正在用的那组的 A；先验 = 调参半同场景均值（伪计数 1），置信上界选下一组
        s_sum = {k: off["prior"][fine][k] for k in safe}
        s_n = {k: 1 for k in safe}
        band, cur = [start], start
        for w in range(1, len(W)):
            s_sum[cur] += A[f"{cid}|{cur}"]["win"][w - 1]
            s_n[cur] += 1
            t = sum(s_n.values())
            cur = max(safe, key=lambda k: s_sum[k] / s_n[k] + UCB_C * np.sqrt(np.log(t) / s_n[k]))
            band.append(cur)
        oracle_A = max(safe, key=lambda k: A[f"{cid}|{k}"]["whole"])
        for name, seq in (("online_full", full), ("online_bandit", band)):
            sf.write(d / f"final/{cid}_{name}.wav", stitch([(a, ys[k]) for (a, _), k in zip(W, seq)], n), SR, subtype="FLOAT")
        choice[cid] = {"online_full": full, "online_bandit": band, "oracle_A": oracle_A, "table_tuned": start}
        files = {"default": d / f"cand/{cid}_default.wav", "table": d / f"cand/{cid}_c0.wav",
                 "table_tuned": d / f"cand/{cid}_{start}.wav", "tuned": d / f"cand/{cid}_{off['tuned']}.wav",
                 "online_full": d / f"final/{cid}_online_full.wav", "online_bandit": d / f"final/{cid}_online_bandit.wav",
                 "oracle_A": d / f"cand/{cid}_{oracle_A}.wav"}
        for sysname, p in files.items():
            jobs.append({"id": f"{cid}|{sysname}", "dry": str(d / f"cache/{cid}_noisy.wav"),
                         "clean": str(make_ref(c, d)), "score": str(p)})
    res = driver(jobs)
    with open(bdir(build) / "final.jsonl", "w") as f:
        for j, r in zip(jobs, res):
            r["file"] = j["score"]
            r["sha256"] = sha(j["score"])
            f.write(json.dumps(r) + "\n")
    (bdir(build) / "choice.json").write_text(json.dumps({"offline": off, "clips": choice}, ensure_ascii=False, indent=1))
    print("safe", safe, "tuned", off["tuned"], "table_tuned", off["table_tuned"])


# ---------------------------------------------------------------- judge（B · C · 吃字）
ASR_URL = "http://127.0.0.1:11520/v1/audio/transcriptions"
ASR_MODEL = "qwen3-asr-plus:1.7b"
ASR_REPEATS = 2   # 照 V-EAT-ASR：转两次，两次不一致的段不计（verifiers.rs:285）


def asr(path, cache, rep):
    key = f"{sha(path)}|{rep}"
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
    """词级编辑距离路径上的删除 + 替换数。同代价时先走对角线（各做法同一口径，绝对值可能偏高）。"""
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
    cl = {c["id"]: c for c in clips(build)}
    d = adir(build)
    (d / "tail").mkdir(exist_ok=True)
    out = {}
    import scoreq
    sq = scoreq.Scoreq(data_domain="natural", mode="nr")
    fA = distill()
    vdn_eval = WT / "target/release/vdn-eval"
    dns = MAIN / "resources/DNS-Challenge/DNSMOS/DNSMOS/sig_bak_ovr.onnx"
    t0 = time.perf_counter()
    for r in fin:
        y, _ = sf.read(r["file"], dtype="float32")
        tail = d / "tail" / (Path(r["file"]).stem + f"_{r['id'].split('|')[1]}_tail.wav")
        sf.write(tail, y[int(WIN_S * SR):], SR, subtype="FLOAT")      # 去掉第一窗（在线闭环第一窗 = 查表起点）
        o = subprocess.run([str(vdn_eval), "dnsmos", str(dns), r["file"]], capture_output=True, text=True, check=True)
        out[r["id"]] = {"C": json.loads(o.stdout.strip().splitlines()[-1]),
                        "B": float(sq.predict(test_path=r["file"], ref_path=None)),
                        "B_tail": float(sq.predict(test_path=str(tail), ref_path=None)),
                        "A": fA(y)}
    print("final scored", f"{time.perf_counter() - t0:.0f}s", flush=True)
    # 报分半每个候选的 B：算「用 B 挑、用 B 量」的真上界
    candB = {}
    for cid, c in cl.items():
        if c["half"] == "report":
            for k in CANDS:
                candB[f"{cid}|{k}"] = float(sq.predict(test_path=str(d / f"cand/{cid}_{k}.wav"), ref_path=None))
    print("cand B scored", f"{time.perf_counter() - t0:.0f}s", flush=True)
    cp = d / "asr_cache.json"
    cache = json.loads(cp.read_text()) if cp.exists() else {}
    for r in fin:
        cid = r["id"].split("|")[0]
        refs = [asr(d / f"cache/{cid}_clean.wav", cache, k) for k in range(ASR_REPEATS)]
        hyps = [asr(r["file"], cache, k) for k in range(ASR_REPEATS)]
        stable = len(set(map(lambda t: " ".join(words(t)), refs))) == 1 and len(set(map(lambda t: " ".join(words(t)), hyps))) == 1
        out[r["id"]]["asr"] = {"del_sub": del_sub(refs[0], hyps[0])[0], "ref_words": len(words(refs[0])), "stable": stable}
        cp.write_text(json.dumps(cache))
    print("asr done", f"{time.perf_counter() - t0:.0f}s", flush=True)
    (bdir(build) / "judge.json").write_text(json.dumps({"final": out, "candB": candB}, indent=0))


# ---------------------------------------------------------------- report
SYSTEMS = ["default", "table", "table_tuned", "tuned", "online_full", "online_bandit", "oracle_A"]
PRIMARY = [("online_bandit", "tuned"), ("online_full", "tuned")]
SECONDARY = [("online_bandit", "table_tuned"), ("online_full", "table_tuned"), ("table_tuned", "table"),
             ("tuned", "table"), ("table", "default")]
N_BOOT, N_PERM = 4000, 4000
NI_EAT_PP = 0.5      # 吃字率非劣效界：百分点
EQ_MOS = 0.1         # 判停线：B 分（SCOREQ，5 分制）


def by_spk(ids, cl, f):
    g = {}
    for i in ids:
        g.setdefault(cl[i]["target_spk"], []).append(f(i))
    return {s: np.array(v, float) for s, v in g.items()}


def boot_ci(g, seed=SEED):
    rng = np.random.default_rng(seed)
    spk = list(g)
    m = [np.concatenate([g[spk[i]] for i in rng.choice(len(spk), len(spk))]).mean() for _ in range(N_BOOT)]
    return [float(np.percentile(m, 2.5)), float(np.percentile(m, 97.5))]


def perm_p(g, seed=SEED):
    """按说话人整簇翻符号的置换检验（双侧）。"""
    rng = np.random.default_rng(seed)
    sums = np.array([v.sum() for v in g.values()])
    n = sum(len(v) for v in g.values())
    obs = abs(sums.sum() / n)
    null = np.abs((rng.choice([-1, 1], size=(N_PERM, len(sums))) * sums).sum(1) / n)
    return float((np.sum(null >= obs - 1e-12) + 1) / (N_PERM + 1))


def ratio_ci(ids, cl, num, den, seed=SEED):
    rng = np.random.default_rng(seed)
    g = {}
    for i in ids:
        g.setdefault(cl[i]["target_spk"], []).append(i)
    spk = list(g)
    m = []
    for _ in range(N_BOOT):
        pick = [i for s in rng.choice(len(spk), len(spk)) for i in g[spk[s]]]
        m.append(sum(num(i) for i in pick) / max(1, sum(den(i) for i in pick)))
    return [float(np.percentile(m, 2.5)), float(np.percentile(m, 97.5))]


def verdict_better(ci_B, rl_worse, eat_ci_hi):
    """事先写定的三档判读（README 第 6 节）。"""
    if ci_B[0] > 0 and rl_worse <= 1 and eat_ci_hi < NI_EAT_PP / 100:
        return "更好"
    if ci_B[1] < 0:
        return "更差"
    return "无法判定"


def cmd_report(build):
    cl = {c["id"]: c for c in clips(build)}
    fin = {r["id"]: r for r in map(json.loads, (bdir(build) / "final.jsonl").read_text().splitlines())}
    JJ = json.loads((bdir(build) / "judge.json").read_text())
    J, candB = JJ["final"], JJ["candB"]
    off = json.loads((bdir(build) / "choice.json").read_text())["offline"]
    ids = sorted({k.split("|")[0] for k in fin})
    ok = lambda i, s: redline_ok(fin[f"{i}|{s}"]["scores"])
    st = lambda i, s: J[f"{i}|{s}"]["asr"]["stable"] and J[f"{i}|default"]["asr"]["stable"]
    tab = {}
    for s in SYSTEMS:
        ks = [f"{i}|{s}" for i in ids]
        stable = [i for i in ids if J[f"{i}|{s}"]["asr"]["stable"]]
        tab[s] = {
            "n": len(ks),
            "B_scoreq": float(np.mean([J[k]["B"] for k in ks])),
            "B_tail": float(np.mean([J[k]["B_tail"] for k in ks])),
            "C_ovrl": float(np.mean([J[k]["C"]["ovrl"] for k in ks])),
            "C_sig": float(np.mean([J[k]["C"]["sig"] for k in ks])),
            "C_bak": float(np.mean([J[k]["C"]["bak"] for k in ks])),
            "A_distillmos": float(np.mean([J[k]["A"] for k in ks])),
            "redline_pass": float(np.mean([ok(i, s) for i in ids])),
            "redline_unknown": int(sum(any(fin[f"{i}|{s}"]["scores"][v]["verdict"] == "unknown" for v in REDLINES) for i in ids)),
            "eat_rate_stable": float(sum(J[f"{i}|{s}"]["asr"]["del_sub"] for i in stable) /
                                     max(1, sum(J[f"{i}|{s}"]["asr"]["ref_words"] for i in stable))),
            "asr_stable_n": len(stable),
        }
    pairs = {}
    for a, b in PRIMARY + SECONDARY:
        row = {}
        for metric, get in (("B", lambda i, s: J[f"{i}|{s}"]["B"]), ("B_tail", lambda i, s: J[f"{i}|{s}"]["B_tail"]),
                            ("C_ovrl", lambda i, s: J[f"{i}|{s}"]["C"]["ovrl"]), ("C_sig", lambda i, s: J[f"{i}|{s}"]["C"]["sig"]),
                            ("A", lambda i, s: J[f"{i}|{s}"]["A"])):
            g = by_spk(ids, cl, lambda i: get(i, a) - get(i, b))
            row[metric] = {"mean": float(np.concatenate(list(g.values())).mean()), "ci95": boot_ci(g), "perm_p": perm_p(g)}
        both = [i for i in ids if J[f"{i}|{a}"]["asr"]["stable"] and J[f"{i}|{b}"]["asr"]["stable"]]
        eat = lambda s: (lambda i: J[f"{i}|{s}"]["asr"]["del_sub"])
        den = lambda i: J[f"{i}|{a}"]["asr"]["ref_words"]
        row["eat_diff"] = {"ci95": ratio_ci(both, cl, lambda i: eat(a)(i) - eat(b)(i), den), "n": len(both)}
        row["redline_worse_n"] = int(sum(ok(i, b) and not ok(i, a) for i in ids))
        row["verdict"] = verdict_better(row["B"]["ci95"], row["redline_worse_n"], row["eat_diff"]["ci95"][1])
        # 讨好信号：A 显著涨，但留出的 B / C_sig 显著跌，或吃字显著多
        row["hacking_signal"] = bool(row["A"]["ci95"][0] > 0 and (row["B"]["ci95"][1] < 0 or row["C_sig"]["ci95"][1] < 0
                                                                   or row["eat_diff"]["ci95"][0] > 0))
        pairs[f"{a} − {b}"] = row
    # 三种事后最优（都只在安全集合里挑）
    safe = off["safe"]
    Aw = json.loads((bdir(build) / "reward_A.json").read_text())["scores"]
    tuned = off["tuned"]
    orc = {}
    for name, pick, meas in (("A挑A量", "A", "A"), ("A挑B量", "A", "B"), ("B挑B量（上界）", "B", "B")):
        sc = lambda i, k, m: Aw[f"{i}|{k}"]["whole"] if m == "A" else candB[f"{i}|{k}"]
        g = by_spk(ids, cl, lambda i: sc(i, max(safe, key=lambda k: sc(i, k, pick)), meas) - sc(i, tuned, meas))
        orc[name] = {"mean_vs_tuned": float(np.concatenate(list(g.values())).mean()), "ci95": boot_ci(g)}
    ub = orc["B挑B量（上界）"]["ci95"]
    stop = "支持「不做按声音自动调」" if ub[1] < EQ_MOS else ("支持「值得做」" if ub[0] > EQ_MOS else "无法判定")
    per_fine = {}
    for fine in sorted({cl[i]["fine"] for i in ids}):
        fi = [i for i in ids if cl[i]["fine"] == fine]
        per_fine[fine] = {"n": len(fi)} | {s: round(float(np.mean([J[f"{i}|{s}"]["B"] for i in fi])), 3) for s in SYSTEMS}
    per_dev = {}
    for dev in sorted({cl[i]["device"] for i in ids}):
        di = [i for i in ids if cl[i]["device"] == dev]
        per_dev[dev] = {"n": len(di)} | {s: round(float(np.mean([ok(i, s) for i in di])), 3) for s in SYSTEMS}
    res = {"build": build, "n_report": len(ids), "n_speakers": len({cl[i]["target_spk"] for i in ids}),
           "offline": {k: off[k] for k in ("safe", "tuned", "table_tuned", "redline_rate_tune")},
           "systems": tab, "pairs": pairs, "oracle": orc, "stop_rule": stop,
           "per_fine_B（描述性）": per_fine, "per_device_redline_pass（描述性）": per_dev}
    (bdir(build) / "results.json").write_text(json.dumps(res, ensure_ascii=False, indent=1))
    print(json.dumps(res, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    cmd, *rest = sys.argv[1:]
    {"select": cmd_select, "enhance": cmd_enhance, "post": cmd_post, "reward": cmd_reward,
     "assemble": cmd_assemble, "judge": cmd_judge, "report": cmd_report}[cmd](*rest)
