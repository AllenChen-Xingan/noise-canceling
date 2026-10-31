"""exp006 第五步：离线开 / 关对比——路由（开 / 关）× 修音链（开 / 关）= 4 版，客观打分 + 盲听页。

| 版本 | 模型 | 修音链 |
|---|---|---|
| `base`   | 全程 GTCRN（D33 主模型） | 关 |
| `route`  | 按 vdn-scene 的已生效场景选模型（GTCRN / FastEnhancer-M），换模型处 50 ms 交叉淡化 | 关 |
| `polish` | 全程 GTCRN | 开，固定默认预设（tone 9，没有场景信息） |
| `both`   | 同 `route` | 开，预设跟着已生效场景走（tone 时间线，参数 1 s 过渡） |

- 场景决策来自 Rust `scene_offline` 在带噪输入上的输出（`feat_<split>_final.jsonl`），第 t 秒的决策只用 t 秒以前的声音，因果
- 模型推理直接调 exp003 `enhance.py` 的 `Gtcrn` / `FastEnhancer` / `stream`（只调用，不复制）；两个模型输出都已对齐到 0 延迟。
  路由的「两套模型同时跑、交叉淡化」是理想化：实时里新模型冷启动、要预热（规约 §3.5），这里两套从头一直在跑
- 修音链调 Rust `polish_offline`（离线多做一次 STFT 模拟「挂在模型 STFT 上」，见该文件说明）
- 响度：同 exp003——只在主讲人说话区间（干净参考能量 VAD）测 BS.1770，各版统一到 −26 LUFS；峰值超 −1 dBFS 时同段一起降。
  这只统一整段响度，段内忽大忽小（AGC 要治的）保留

客观指标（`scores.json`，旁证，人耳为准）：
- DNSMOS P.835（SIG / BAK / OVRL，非个性化）、ESTOI（对干净参考）
- 吃字：本机 Qwen3-ASR（D28）删除率，参照 = 干净参考的转写；只跑 ASR 子集（每类 12 段）
- 四种毛病的代理指标（自定义，口径写在下面）：
  · **音乐噪声** `lkr`：非语音帧、300–6000 Hz 的 STFT 功率的峰度，输出 ÷ 带噪输入取自然对数（对数峰度比，Uemura 等 2009 的做法；越大越「叮叮」）
  · **真空感** `resid_rel_db`：输出非语音段功率 ÷ 输出语音段功率（dB）；越负越「抽空」。另报 `gap_flutter_db`：非语音帧电平的标准差（背景忽有忽无）
  · **忽大忽小** `level_std_db`：语音占比 ≥ 50% 的 400 ms 窗的输出电平标准差（dB），减去干净参考的同一量（素材里故意加了说话人电平漂移）
  · **发闷 / 过亮** `hf_loss_db`：语音帧里 4–7.5 kHz 对 0.5–4 kHz 的能量比，输出减干净参考（dB；负 = 闷，正 = 比干净参考更亮）；
    理想值 0，胜负按 |偏差|（`hf_dev_abs`）判；窄带蓝牙段不算

用法（worktree 根下，**跑推理要先申请名额**）：
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/offline_ab.py <build> enhance   # 推理 + 路由 + 修音链 + 响度
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/offline_ab.py <build> score [--asr]
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/offline_ab.py <build> page    # 新的盲听页 build
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/offline_ab.py <build> null    # 同底噪零假设对照（只算毛病指标）
  <主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp006-scene-sim/offline_ab.py <build> agg     # 只重新汇总
"""
import base64
import hashlib
import json
import secrets
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

import numpy as np
import pyloudnorm
import soundfile as sf
from pystoi import stoi

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
EXP3 = ROOT / "docs/experiments/exp003-offline-ab"
sys.path.insert(0, str(EXP3))
from _common import tokens, vad_mask  # noqa: E402
from _paths import RES  # noqa: E402

SR = 16000
VERS = ["base", "route", "polish", "both"]
# 4 阶 Williams 方阵：行 = 段（循环使用），列 = 字母位置，值 = 基序里的第几个版本
WILLIAMS4 = [[0, 1, 3, 2], [1, 2, 0, 3], [2, 3, 1, 0], [3, 0, 2, 1]]
INFO = {
    "base": {"name": "基线：GTCRN，修音链关", "latency": "模型 16 ms；本行同 exp003 的 GTCRN"},
    "route": {"name": "路由开：按场景选 GTCRN / FastEnhancer-M，修音链关", "latency": "按池内最大（FE-M 22 ms）补齐，GTCRN 场景多 6 ms"},
    "polish": {"name": "修音链开（默认预设）：GTCRN", "latency": "修音链 0 ms（零前瞻）"},
    "both": {"name": "路由 + 修音链都开（预设跟场景）", "latency": "同「路由开」；修音链 0 ms"},
}
XFADE = int(0.05 * SR)
TARGET_LUFS, PEAK_MAX_DB = -26.0, -1.0
LISTEN = {"babble": 2, "neighbor": 2, "clatter": 2, "traffic": 2, "music": 2, "hvac": 1, "quiet": 1}
ASR_PER_CLASS = 12


def paths(build):
    """(数据目录, 听测目录, 工作目录)。听测目录只放打乱命名的音频与页面——**不放**揭盲表、写着条件名的清单和
    以条件名命名的文件（CLAUDE.md 第五节第 1 条，2026-10-31 补）；这些都在工作目录或 docs/ 里。"""
    d = ROOT / "_tmp/20261031-exp006" / build
    # 听测目录单独放一棵树，不和工作目录同父（往上翻一级也看不到答案）
    return d, ROOT / "_tmp/20261031-exp006-listen" / build, d / "ab_work"


def truth_of(build):
    return {json.loads(l)["id"]: json.loads(l) for l in (HERE / "builds" / build / "truth.jsonl").read_text().splitlines()}


def decisions(build, split):
    d, _, _ = paths(build)
    out = {}
    for l in (d / f"feat_{split}_final.jsonl").read_text().splitlines():
        r = json.loads(l)
        out.setdefault(r["id"], []).append((r["t"], r["model"], r["tone"], r["committed_name"]))
    return out


def model_weight(dec, n):
    """FE-M 的逐样本权重（0 = GTCRN，1 = FE-M）；决策在 t 秒生效，换模型处 50 ms 线性交叉淡化。开头还没决策时用 GTCRN。"""
    w = np.zeros(n, np.float32)
    cur = 0.0
    for t, model, _, _ in dec:
        k = int(t * SR)
        tgt = 1.0 if model == "fastenhancer-m" else 0.0
        if tgt != cur and k < n:
            e = min(n, k + XFADE)
            w[k:e] = np.linspace(cur, tgt, e - k, endpoint=False)
            w[e:] = tgt
            cur = tgt
    return w


def selection(truth):
    test = sorted([t for t in truth.values() if t["split"] == "test" and t["has_target"]], key=lambda t: t["id"])
    rng = np.random.default_rng(20261106)
    listen, asr = [], []
    for c in LISTEN:
        lo, hi = (0, 12) if c not in ("quiet", "hvac") else (-99, 99)  # 安静 / 空调的信噪比本来就高
        pool = [t for t in test if t["fine"] == c and t["device"] == "none" and lo <= (t["snr_db"] or 0) <= hi]
        if c == "music":
            pool = [t for t in pool if t["noise"].startswith("MapleLeaf")] + [t for t in pool if not t["noise"].startswith("MapleLeaf")]
            listen += pool[:LISTEN[c]]
        else:
            listen += list(rng.choice(pool, size=min(LISTEN[c], len(pool)), replace=False))
    for c in LISTEN:
        pool = [t for t in test if t["fine"] == c]
        asr += list(rng.choice(pool, size=min(ASR_PER_CLASS, len(pool)), replace=False))
    return test, [t["id"] for t in listen], sorted({t["id"] for t in asr} | {t["id"] for t in listen})


def enhance(build):
    import enhance as e3  # exp003：只调用
    d, ab, wk = paths(build)
    (wk / "raw").mkdir(parents=True, exist_ok=True)
    (ab / "audio").mkdir(parents=True, exist_ok=True)
    truth = truth_of(build)
    test, listen, _ = selection(truth)
    dec = decisions(build, "test")
    polish_bin = ROOT / "target/release/examples/polish_offline"
    meter = pyloudnorm.Meter(SR)
    files = {}
    perf = []
    import os
    lim = int(os.environ.get("EXP006_LIMIT", "0"))  # 冒烟用：只跑前几段 + 听测段
    if lim:
        test = [t for t in test if t["id"] in listen][:lim]
    for t in test:
        cid = t["id"]
        x, _ = sf.read(d / "test" / f"{cid}_noisy.wav", dtype="float32")
        ref, _ = sf.read(d / "test" / f"{cid}_clean.wav", dtype="float32")
        g = e3.stream(e3.Gtcrn(), x)
        f = e3.FastEnhancer().run(x)
        w = model_weight(dec[cid], len(x))
        routed = (1 - w) * g + w * f
        raw = {"noisy": x, "base": g, "route": routed}
        for v in ("noisy", "base", "route"):
            sf.write(wk / "raw" / f"{cid}_{v}.wav", raw[v], SR, subtype="FLOAT")
        tl = wk / "raw" / f"{cid}_tones.txt"
        tl.write_text("0 9\n" + "".join(f"{tt:.3f} {tone}\n" for tt, _, tone, _ in dec[cid]))
        for v, wet, tone in (("polish", "base", "9"), ("both", "route", str(tl))):
            p = subprocess.run([str(polish_bin), str(wk / "raw" / f"{cid}_noisy.wav"), str(wk / "raw" / f"{cid}_{wet}.wav"),
                                str(wk / "raw" / f"{cid}_{v}.wav"), tone], capture_output=True, text=True, check=True)
            perf.append(json.loads(p.stderr.strip().splitlines()[-1]))
            raw[v], _ = sf.read(wk / "raw" / f"{cid}_{v}.wav", dtype="float32")
        # 响度：主讲人说话区间统一到 −26 LUFS，峰值超限同段一起降
        mask, n = vad_mask(ref)
        gains = {}
        for v in ["noisy", *VERS]:
            y = raw[v][:n][mask]
            gains[v] = TARGET_LUFS - meter.integrated_loudness(y) if len(y) > SR // 2 else 0.0
        peak = max(db(np.max(np.abs(raw[v])) * 10 ** (gains[v] / 20)) for v in gains)
        extra = min(0.0, PEAK_MAX_DB - peak)
        files[cid] = {"fine": t["fine"], "user": t["user"], "device": t["device"], "snr_db": t["snr_db"], "noise": t["noise"],
                      "route": [(tt, m, tone, nm) for tt, m, tone, nm in dec[cid]], "extra_gain_db": round(extra, 2), "versions": {}}
        for v in ["noisy", *VERS]:
            y = raw[v] * 10 ** ((gains[v] + extra) / 20)
            fn = f"audio/{cid}_{secrets.token_hex(4)}.wav"
            sf.write(ab / fn, y, SR, subtype="PCM_16")
            files[cid]["versions"][v] = {"file": fn, "sha256": hashlib.sha256((ab / fn).read_bytes()).hexdigest(),
                                         "gain_db": round(gains[v] + extra, 2)}
        print(cid, t["fine"], files[cid]["route"][-1][1:], flush=True)
    sp = float(np.mean([p["spectral_us_per_hop"] for p in perf]))
    ag = float(np.mean([p["agc_us_per_hop"] for p in perf]))
    meta = {"build": build, "listen": listen, "n_clips": len(files), "polish_us_per_hop": {"spectral": round(sp, 2), "agc": round(ag, 2),
            "cpu_share_pct_of_one_core": round(100 * (sp + ag) / 16000, 3)}}
    (wk / "files.json").write_text(json.dumps({"meta": meta, "clips": files}, ensure_ascii=False, indent=0))
    print(json.dumps(meta, ensure_ascii=False))


def db(x):
    return 20 * np.log10(max(float(x), 1e-12))


def stft_pow(x, n_fft=512, hop=256):
    win = np.sqrt(0.5 - 0.5 * np.cos(2 * np.pi * np.arange(n_fft) / n_fft))
    nf = (len(x) - n_fft) // hop + 1
    fr = np.stack([x[i * hop: i * hop + n_fft] * win for i in range(nf)])
    return np.abs(np.fft.rfft(fr, axis=1)) ** 2


def flaw_metrics(y, noisy, ref, device):
    """四种毛病的代理指标（口径见文件头）。帧 = 512/256，帧的有声与否按干净参考帧能量（高于最响帧 −35 dB）。"""
    P, Pn, Pr = stft_pow(y), stft_pow(noisy), stft_pow(ref)
    n = min(len(P), len(Pn), len(Pr))
    P, Pn, Pr = P[:n], Pn[:n], Pr[:n]
    er = 10 * np.log10(Pr.sum(1) + 1e-12)
    sp = er > er.max() - 35
    ns = ~sp
    f = np.arange(P.shape[1]) * SR / 512
    band = (f >= 300) & (f <= 6000)
    out = {}
    if ns.sum() >= 10:
        def kurt(a):
            a = a.ravel()
            return np.mean(a ** 2) / (np.mean(a) ** 2 + 1e-30)
        out["lkr"] = round(float(np.log(kurt(P[ns][:, band]) / kurt(Pn[ns][:, band]))), 3)
        e_ns = 10 * np.log10(P[ns].sum(1) + 1e-12)
        out["gap_flutter_db"] = round(float(np.std(e_ns)), 2)
    out["resid_rel_db"] = round(float(10 * np.log10((P[ns].sum(1).mean() + 1e-12) / (P[sp].sum(1).mean() + 1e-12))), 2) if ns.any() else None
    # 400 ms 窗（≈25 帧）的语音电平
    def lvl_std(Q):
        vals = []
        for i in range(0, n - 25, 12):
            m = sp[i: i + 25]
            if m.mean() >= 0.5:
                vals.append(10 * np.log10(Q[i: i + 25][m].sum(1).mean() + 1e-12))
        return float(np.std(vals)) if len(vals) >= 3 else None
    a, b = lvl_std(P), lvl_std(Pr)
    out["level_std_db"] = round(a - b, 2) if a is not None and b is not None else None
    if device != "bt_narrow":
        hi, mid = (f >= 4000) & (f <= 7500), (f >= 500) & (f < 4000)
        r = lambda Q: 10 * np.log10(Q[sp][:, hi].sum() / (Q[sp][:, mid].sum() + 1e-12) + 1e-12)
        out["hf_loss_db"] = round(float(r(P) - r(Pr)), 2)
    return out


def score(build, use_asr):
    sys.path.insert(0, str(RES / "DNS-Challenge/DNSMOS"))
    from dnsmos_local import ComputeScore
    import score as s3  # exp003：ASR 与编辑距离，只调用
    d, ab, wk = paths(build)
    s3.CACHE_FILE = wk / "asr_cache.json"
    s3.CACHE = json.loads(s3.CACHE_FILE.read_text()) if s3.CACHE_FILE.exists() else {}
    files = json.loads((wk / "files.json").read_text())
    _, _, asr_ids = selection(truth_of(build))
    dn = RES / "DNS-Challenge/DNSMOS/DNSMOS"
    scorer = ComputeScore(str(dn / "sig_bak_ovr.onnx"), str(dn / "model_v8.onnx"))
    per = {}
    for cid, c in files["clips"].items():
        ref, _ = sf.read(d / "test" / f"{cid}_clean.wav", dtype="float32")
        raw = {v: sf.read(wk / "raw" / f"{cid}_{v}.wav", dtype="float32")[0] for v in ["noisy", *VERS]}
        r = {}
        do_asr = use_asr and cid in asr_ids
        if do_asr:
            ref_tok = tokens(s3.asr_once(d / "test" / f"{cid}_clean.wav", "en", s3.ASR_MODEL, 1), "en")
        for v in ["noisy", *VERS]:
            s = scorer(str(ab / c["versions"][v]["file"]), SR, False)
            row = {"SIG": round(float(s["SIG"]), 3), "BAK": round(float(s["BAK"]), 3), "OVRL": round(float(s["OVRL"]), 3),
                   "ESTOI": round(float(stoi(ref, raw[v][: len(ref)], SR, extended=True)), 4)}
            row.update(flaw_metrics(raw[v] * 10 ** (c["versions"][v]["gain_db"] / 20), raw["noisy"], ref, c["device"]))
            if do_asr:
                hyp = s3.asr_once(ab / c["versions"][v]["file"], "en", s3.ASR_MODEL, 1)
                D, S, I = s3.edit_ops(ref_tok, tokens(hyp, "en"))
                row.update({"del_rate": round(D / max(1, len(ref_tok)), 4), "ins_rate": round(I / max(1, len(ref_tok)), 4)})
            r[v] = row
        per[cid] = r
        print(cid, c["fine"], {v: r[v]["OVRL"] for v in r}, flush=True)
    aggregate(build, per, files)


# AGC 时间常数是在这 120 段上比出来的（按段号排序的前 120 段，见 README §4）；报告时另报其余段
AGC_TUNE_N = 120
# ASR 判定为不可信的段：任何一版删除率 ≥ 0.9（整句都没转出来——通常是 ASR 转成了旁人的话，原声也一样）
ASR_FAIL = 0.9


def sign_test_p(w, l):
    """双侧符号检验 p 值（平局已剔除）。"""
    from math import comb
    n, k = w + l, min(w, l)
    if n == 0:
        return 1.0
    return min(1.0, 2 * sum(comb(n, i) for i in range(k + 1)) / 2 ** n)


def aggregate(build, per, files):
    """从逐段分数汇总（不重算任何分数）。规则写在这里：
    - 「发闷」hf_loss_db 理想值是 0（与干净参考同样亮），按 |偏差| 判胜负、报 hf_dev_abs；正值 = 比干净参考更亮
      （第一版按「越大越好」判，方向错了，自审 P0）
    - 删除率：报全部与「去掉 ASR 不可信段」两套，并给符号检验 p 值
    - 有 null（同底噪对照）就一起报；AGC 调参段与其余段分开报音量起伏"""
    vers = ["noisy", *VERS] + (["null"] if any("null" in r for r in per.values()) else [])
    for r in per.values():
        for v in vers:
            if v in r and r[v].get("hf_loss_db") is not None:
                r[v]["hf_dev_abs"] = round(abs(r[v]["hf_loss_db"]), 3)
    keys = ["SIG", "BAK", "OVRL", "ESTOI", "lkr", "resid_rel_db", "gap_flutter_db", "level_std_db", "hf_loss_db", "hf_dev_abs",
            "del_rate", "ins_rate"]
    asr_bad = sorted(c for c, r in per.items() if any(r[v].get("del_rate", 0) >= ASR_FAIL for v in VERS + ["noisy"] if v in r))

    def agg(ids):
        out = {}
        for v in vers:
            out[v] = {}
            for k in keys:
                vals = [per[c][v][k] for c in ids if v in per[c] and per[c][v].get(k) is not None]
                if vals:
                    out[v][k] = {"mean": round(float(np.mean(vals)), 3), "median": round(float(np.median(vals)), 3), "n": len(vals)}
        return out

    by_class = {fine: agg([c for c in per if files["clips"][c]["fine"] == fine]) for fine in LISTEN}
    better_low = {"lkr", "gap_flutter_db", "level_std_db", "hf_dev_abs", "del_rate", "ins_rate"}
    no_wins = {"hf_loss_db"}  # 有正负两侧的量不直接比大小

    def wins(a, b, k, ids):
        w = l = 0
        for c in ids:
            if a not in per[c] or b not in per[c]:
                continue
            x, y = per[c][a].get(k), per[c][b].get(k)
            if x is None or y is None or x == y:
                continue
            if (x < y) == (k in better_low):
                w += 1
            else:
                l += 1
        return {"win": w, "lose": l, "p_sign": round(sign_test_p(w, l), 4)}

    ids = list(per)
    pair_list = [("polish", "base"), ("both", "route"), ("route", "base"), ("both", "base")]
    if "null" in vers:
        pair_list += [("polish", "null"), ("null", "base")]
    pairs = {f"{a}_vs_{b}": {k: wins(a, b, k, ids) for k in keys if k not in no_wins} for a, b in pair_list}
    ok_asr = [c for c in ids if c not in asr_bad]
    asr_clean = {f"{a}_vs_{b}": {k: wins(a, b, k, ok_asr) for k in ("del_rate", "ins_rate")} for a, b in pair_list}
    tune = sorted(ids)[:AGC_TUNE_N]
    rest = sorted(ids)[AGC_TUNE_N:]
    agc_split = {"tune_ids_n": len(tune), "rest_ids_n": len(rest),
                 "polish_vs_base_level_std_tune": wins("polish", "base", "level_std_db", tune),
                 "polish_vs_base_level_std_rest": wins("polish", "base", "level_std_db", rest),
                 "mean_level_std_rest": {v: agg(rest)[v].get("level_std_db", {}).get("mean") for v in ("base", "polish")}}
    out = {"build": build, "n": len(per), "asr_n": sum(1 for c in per if "del_rate" in per[c]["base"]),
           "rules": aggregate.__doc__, "asr_unreliable_clips": asr_bad,
           "all": agg(ids), "all_asr_reliable": agg(ok_asr), "by_class": by_class, "pairwise": pairs,
           "pairwise_asr_reliable": asr_clean, "agc_tau_split": agc_split, "per_clip": per,
           "polish_cost": files["meta"]["polish_us_per_hop"]}
    (HERE / "builds" / build / "ab_scores.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps({"all": out["all"], "pairwise": pairs, "asr_bad": asr_bad}, ensure_ascii=False)[:3000])


def null_control(build):
    """零假设对照：base（GTCRN）输出只加修音链默认预设的下限 + 舒适噪声（不平滑、不去发闷、不 AGC），
    看四种毛病的代理指标单靠「加底噪」会降多少（自审：lkr / 背景忽有忽无会被底噪按构造压低）。只算毛病指标，不跑模型、不跑 ASR。"""
    d, ab, wk = paths(build)
    files = json.loads((wk / "files.json").read_text())
    sc = json.loads((HERE / "builds" / build / "ab_scores.json").read_text())
    per = sc["per_clip"]
    dp = PolishParamsDefault
    pol = ROOT / "target/release/examples/polish_offline"
    for cid, c in files["clips"].items():
        ref, _ = sf.read(d / "test" / f"{cid}_clean.wav", dtype="float32")
        out = wk / "raw" / f"{cid}_null.wav"
        subprocess.run([str(pol), str(wk / "raw" / f"{cid}_noisy.wav"), str(wk / "raw" / f"{cid}_base.wav"), str(out), "9",
                        "--params", f"0,{dp['floor_db']},{dp['comfort_db']},0,0"], capture_output=True, check=True)
        y, _ = sf.read(out, dtype="float32")
        x, _ = sf.read(wk / "raw" / f"{cid}_noisy.wav", dtype="float32")
        per[cid]["null"] = flaw_metrics(y, x, ref, c["device"])
    aggregate(build, per, files)


# 与 vdn_polish::PolishParams::DEFAULT（tone 9）一致；null 对照只取其中的下限与舒适噪声
PolishParamsDefault = {"floor_db": -24.0, "comfort_db": -10.0}


def page(build):
    """盲听页：复用 exp003 的 ab_page.html（4 个候选它已支持），只替换几处文案与存储前缀；
    毛病标签默认展开（本实验就是要人标毛病）。每次一个新 page build，不覆盖旧的。"""
    _, ab, wk = paths(build)
    files = json.loads((wk / "files.json").read_text())
    listen = files["meta"]["listen"]
    clips = files["clips"]
    DESC = {"babble": "咖啡馆 / 餐厅多人嘈杂", "neighbor": "邻座有人说话", "clatter": "杯碟 / 键盘声", "traffic": "街道 / 公交",
            "music": "背景音乐", "hvac": "空调 / 风扇", "quiet": "安静房间"}
    pid = datetime.now().strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2)
    pdir = ab / "pages" / pid
    pdir.mkdir(parents=True, exist_ok=False)
    up = lambda f: "../../" + f
    rng = secrets.SystemRandom()
    key, mclips = {}, []
    # Williams 设计（4 阶）在生成时定死：每个版本在每个字母位置正好 3 次，且「谁紧挨在谁前面」的 12 种先后对各出现一次 / 每 4 段。
    # 循环拉丁方每段的环形先后都一样，平衡不了相邻顺序（自审 P3）。页面不在运行时轮换（它根本不知道哪个槽位是哪个版本）
    base = VERS[:]
    rng.shuffle(base)
    listen = listen[:]
    rng.shuffle(listen)
    for i, cid in enumerate(listen):
        order = [base[k] for k in WILLIAMS4[i % 4]]
        key[cid] = order
        v = clips[cid]["versions"]
        mclips.append({"id": cid, "desc": DESC[clips[cid]["fine"]], "text": None, "ref": up(v["noisy"]["file"]),
                       "cands": [up(v[m]["file"]) for m in order]})
    # 检验题：全同 + 劣化（劣化法同 exp003：低通 1 kHz + 5 dB 白噪，响度拉回）
    import build_page as b3  # exp003：只调用 degrade
    c1, c2 = rng.sample(listen, 2)
    m1 = rng.choice(VERS)
    tid = ["trap-1", "trap-2"]
    rng.shuffle(tid)  # 题号不透露哪道是全同、哪道是劣化
    traps = [{"id": tid[0], "kind": "identical", "clip": c1, "desc": DESC[clips[c1]["fine"]], "label": "所有候选其实是同一个文件（应答「分不出」）",
              "ref": up(clips[c1]["versions"]["noisy"]["file"]), "cands": [up(clips[c1]["versions"][m1]["file"])] * 4,
              "names": ["same"] * 4, "expect": {"best": ["same"]}, "source_model": m1}]  # 这题页面里就能判
    m2 = rng.choice(VERS)
    y, _ = sf.read(ab / clips[c2]["versions"][m2]["file"], dtype="float32")
    fd = f"audio/{c2}_{secrets.token_hex(4)}.wav"  # 与正常文件同长（第一版 6 位十六进制，看文件名就能认出劣化版）
    sf.write(ab / fd, b3.degrade(y, pyloudnorm.Meter(SR)), SR, subtype="PCM_16")
    others = [m for m in VERS if m != m2]
    rng.shuffle(others)
    slots = [(m2, clips[c2]["versions"][m2]["file"]), ("degraded", fd)] + [(m, clips[c2]["versions"][m]["file"]) for m in others[:2]]
    rng.shuffle(slots)
    traps.append({"id": tid[1], "kind": "degraded", "clip": c2, "desc": DESC[clips[c2]["fine"]], "label": "其中一份是明显劣化版（揭盲后离线判：最清楚、最自然都不该选它）",
                  "ref": up(clips[c2]["versions"]["noisy"]["file"]), "cands": [up(f) for _, f in slots],
                  "names": [n for n, _ in slots], "expect": {"not": []}, "source_model": m2})
    # manifest 里**没有答案**：没有槽位→版本映射、没有版本名（CLAUDE.md 第五节第 1 条，2026-10-31 补）。
    # 页面只记录「哪个字母是第几个槽位」，揭盲由仓里的 unblind.py 拿 docs/ 里的对照表做
    manifest = {"build": pid, "n_cands": 4, "clips": mclips,
                "models": {f"slot{i}": {"name": f"槽位 {i}", "latency": ""} for i in range(4)},
                # 检验题在页面里只叫「检验题」，真实说明（哪道是全同 / 劣化）只在 docs/ 的对照表里
                "traps": [{**{k: v for k, v in t.items() if k not in ("source_model", "names", "label", "kind", "expect")}, "label": "检验题"}
                          for t in traps]}
    (pdir / "manifest.js").write_text("window.EXP003 = " + json.dumps(manifest, ensure_ascii=False) + ";\n")
    html = (EXP3 / "ab_page.html").read_text()
    subs = [
        ('const KEY = JSON.parse(atob(M.slotmap || M.key));', 'const KEY = null;  // exp006：页面里不放答案，揭盲在仓里做'),
        ('LETTERS.forEach((L, j) => { slots[L] = KEY.clips[id].indexOf(base[(i + j) % N]); });',
         'LETTERS.forEach((L, j) => { slots[L] = j; });  // 生成时已按拉丁方排好每段的槽位顺序'),
        ('if (t.kind === "trap") return KEY.traps[t.trap][t.slots[L]];\n    return KEY.clips[t.clip][t.slots[L]];', 'return "slot" + t.slots[L];'),
        ('      ${s.head_to_head.dpdfnet2_vs_gtcrn ? `<p><b>DPDFNet-2 对 GTCRN 逐段</b>（先比「开会」评级，平级再看谁被选为最清楚）：DPDFNet-2 好 ${s.head_to_head.dpdfnet2_vs_gtcrn.dpdfnet2} 段 · GTCRN 好 ${s.head_to_head.dpdfnet2_vs_gtcrn.gtcrn} 段 · 分不出 ${s.head_to_head.dpdfnet2_vs_gtcrn.tie} 段。这就是「多 30 ms 延迟换来的效果」的主要证据。</p>` : ""}', ""),  # exp003 专用的「DPDFNet-2 对 GTCRN」一句
        ("// 两两对比（主要看 DPDFNet-2 对 GTCRN）：", "// 两两对比："),
        ("const ex = trapById[r.trap].expect;", "const ex = { not: [] };  // 检验题在页面里不判（判法会泄露是哪道），揭盲后离线判"),
        ('<h2>听完了，揭盲结果</h2>', '<h2>听完了</h2><p class="muted">页面里没有答案：下表只按槽位统计，看不出哪个是哪个。导出 JSON 交大总管揭盲。</p>'),
        ("<title>降噪盲听对比</title>", "<title>修音与场景盲听</title>"),
        ("<h1>降噪盲听对比</h1>", "<h1>修音与场景盲听（exp006）</h1>"),
        ("主讲人是声音最近、最清楚的那位（多数段开头约 1 秒后才开口，前 3 段一开头就说）；其他人声都算噪音。",
         "主讲人是声音最近、最清楚的那位（有的段开头 1 秒内就开口）；其他人声都算噪音。四个候选用的降噪模型相近，差别主要在<b>听感毛病</b>：金属声 / 水声、真空感、忽大忽小、发闷——听到就在每个候选下面点出来。"),
        ("大约 25–30 分钟。", "大约 25–35 分钟。"),
        ('const STORE = "exp003-ab-" + M.build;', 'const STORE = "exp006-ab-" + M.build;'),
        ('tg.className = "opts tagrow hidden";', 'tg.className = "opts tagrow";'),
        ('exp: "exp003-offline-ab"', 'exp: "exp006-scene-sim"'),
        ("a.download = `exp003-ab-result-", "a.download = `exp006-ab-result-"),
        ("交给大总管放进 <code>docs/experiments/exp003-offline-ab/</code>。", "交给大总管放进 <code>docs/experiments/exp006-scene-sim/results/</code>。"),
    ]
    for a, b in subs:
        assert html.count(a) == 1, f"exp003 页面变了，找不到：{a[:30]}"
        html = html.replace(a, b)
    assert "KEY." not in html.replace("KEY.clips[id].indexOf", ""), "页面里还有用到答案的地方"
    (pdir / "index.html").write_text(html)
    unblind = json.dumps({"build": pid, "data_build": build, "models": VERS, "key": key, "traps": traps}, ensure_ascii=False, indent=1)
    keep = HERE / "builds" / build / "pages" / pid
    keep.mkdir(parents=True, exist_ok=False)
    (keep / "unblind-key.json").write_text(unblind)  # 揭盲表只进 docs/（提交）与工作目录，不进听测目录
    (wk / f"unblind-key-{pid}.json").write_text(unblind)
    shutil.copyfile(pdir / "manifest.js", keep / "manifest.js")
    sub = {c: {k: v for k, v in clips[c].items()} for c in listen}
    (keep / "files.json").write_text(json.dumps({"meta": files["meta"], "clips": sub}, ensure_ascii=False, indent=0))
    self_check(ab, pdir, key)
    print("page", pid, "→", pdir / "index.html")


def self_check(ab, pdir, key):
    """听测目录自检：不许有揭盲表 / 清单；页面与 manifest 里不许出现版本名或映射；音频文件名是「段号_随机串」。"""
    import re
    bad = [p for p in ab.rglob("*") if p.is_file() and (p.name in ("unblind-key.json", "files.json") or p.suffix == ".txt")]
    assert not bad, f"听测目录里有不该有的文件：{bad}"
    shown = {i["name"] for i in INFO.values()} | {"slotmap", "unblind", "GTCRN", "FastEnhancer"}
    checks = [(pdir / "manifest.js", set(VERS) | shown | {"identical", "degraded", "劣化", "同一个文件", "expect"}), (pdir / "index.html", shown)]  # 页面代码里有 JS 变量叫 base，只查显示名
    for f, names in checks:
        txt = f.read_text()
        hit = [n for n in names if re.search(r'(?<![A-Za-z_])' + re.escape(n) + r'(?![A-Za-z_])', txt)]
        assert not hit, f"{f.name} 里出现了 {hit}"
    odd = [p.name for p in (ab / "audio").iterdir() if not re.fullmatch(r"te\d{4}_[0-9a-f]{8}\.wav", p.name)]
    assert not odd, f"音频文件名不对：{odd[:3]}"
    print("自检通过：听测目录无揭盲表 / 清单，页面与 manifest 无版本名与映射")


if __name__ == "__main__":
    b, stage = sys.argv[1], sys.argv[2]
    if stage == "enhance":
        enhance(b)
    elif stage == "score":
        score(b, "--asr" in sys.argv)
    elif stage == "page":
        page(b)
    elif stage == "agg":  # 只重新汇总（不重算分数）
        _, _, wk = paths(b)
        sc = json.loads((HERE / "builds" / b / "ab_scores.json").read_text())
        aggregate(b, sc["per_clip"], json.loads((wk / "files.json").read_text()))
    elif stage == "null":
        null_control(b)
