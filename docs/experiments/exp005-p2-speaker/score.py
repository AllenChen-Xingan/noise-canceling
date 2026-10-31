"""exp005 第二、三步：跑 Rust 离线管线、打分。

  run    对每段 mix 跑 `vdn-speaker run`（GTCRN + 5 个版本 + 判断回路日志），再对主人轨 / 干扰轨各算一个整段声纹作「真值」
  asr    按主人每句话切段，Qwen3-ASR（本机 42model 服务，D28）转写 主人轨（参照）· vad（个性化关）· personal（个性化开）
  score  汇总指标 → results/scores.json

指标（005 decision「PSE 专属量法」的可落地子集）：
- IRL 干扰者残留电平：只有邻座在说的样本上 10·log10(Σ输出² / Σ输入干扰²)；不带门限的能量；同时报「锁定之后」那段
- 停顿里「开着」的比例：能量版 vs VAD 版快开慢关（exp004 的问题），按「无人声」（主人、邻座都不在说）的 hop 算
- 主人被压：门控增益逐 hop 记录，主人在说的 hop 里增益 < −3 dB 的比例；另报「每句话开头 2 s 内」
- 注册：锁定时刻、锁定前主人累计说了多久、锁的是不是主人（簇中心与主人 / 邻座真值声纹的余弦谁大）、改判次数
- 认错人：锁错 / 改判错的次数；锁定后「主人在说」的窗被判成 Other 的次数
- 吃字：参照 = 主人轨同一段的转写；按词计删除 / 替换 / 插入（插入多半是邻座漏进来的词，单列）
- 算力：判断回路每次计算耗时（单线程）、实时级每 hop 耗时
用法（项目根下）：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp005-p2-speaker/score.py {run|asr|score} <vdn-speaker 可执行文件>
音频与日志：<主目录>/_tmp/20261031-exp005/out/
"""
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

import numpy as np
import requests
import soundfile as sf

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "exp003-offline-ab"))
from _common import tokens, vad_mask  # noqa: E402
from _paths import MAIN  # noqa: E402

BASE = MAIN / "_tmp/20261031-exp005"
# 每轮输出一个目录，不覆盖上一轮：out = 第一轮（自审前，build 7448 用的），out-r2 = 自审修完后
SRC, OUT = BASE / "src", BASE / os.environ.get("EXP005_RUN", "out-r2")
SR, HOP = 16000, 256
VERS = ["raw", "floor", "energy", "vad", "personal", "aggressive"]
TAGGED = ("personal", "aggressive")  # 重放时带 tag 的版本
ASR_VERS = ["mix", "raw", "vad", "personal", "aggressive"]
ASR_URL = "http://127.0.0.1:11520/v1/audio/transcriptions"
ASR_KEY = "42model"  # 本机服务的固定占位 key，不是密钥（同 exp003 score.py）
ASR_MODEL = "qwen3-asr-plus:1.7b"
CACHE_FILE = BASE / "asr_cache.json"
CACHE = json.loads(CACHE_FILE.read_text()) if CACHE_FILE.exists() else {}
ONNX = Path.home() / "vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx"


def clips():
    return json.loads((HERE / "clips.json").read_text())["clips"]


def cmd_run(exe):
    OUT.mkdir(parents=True, exist_ok=True)
    gtcrn = MAIN / "resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx"
    truth = {}
    for c in clips():
        cid = c["id"]
        wet = OUT / f"{cid}.wet.wav"
        subprocess.run([exe, "denoise", str(SRC / f"{cid}_mix.wav"), str(wet), "--model-path", str(gtcrn)], check=True)
        subprocess.run([exe, "run", str(SRC / f"{cid}_mix.wav"), str(OUT), "--wet", str(wet),
                        "--speaker-onnx", str(ONNX)], check=True)
        truth[cid] = {}
        for tr in ("owner", "interf"):
            if tr == "interf" and not c["neighbors"]:
                continue
            r = subprocess.run([exe, "embed", str(SRC / f"{cid}_{tr}.wav"), "--speaker-onnx", str(ONNX)],
                               capture_output=True, text=True, check=True)
            truth[cid][tr] = json.loads(r.stdout)["emb"]
    (OUT / "truth_emb.json").write_text(json.dumps(truth))


def asr(path):
    data = Path(path).read_bytes()
    k = f"{hashlib.sha256(data).hexdigest()}|{ASR_MODEL}|en"
    if k in CACHE:
        return CACHE[k]
    r = requests.post(ASR_URL, headers={"Authorization": f"Bearer {ASR_KEY}"},
                      data={"model": ASR_MODEL, "language": "en", "temperature": "0", "response_format": "json"},
                      files={"file": (Path(path).name, data, "audio/wav")}, timeout=600)
    r.raise_for_status()
    CACHE[k] = " ".join(r.json()["text"].split())
    CACHE_FILE.write_text(json.dumps(CACHE, ensure_ascii=False, indent=0))
    return CACHE[k]


def edit_ops(ref, hyp):
    n, m = len(ref), len(hyp)
    d = np.zeros((n + 1, m + 1), dtype=np.int32)
    d[:, 0] = np.arange(n + 1)
    d[0, :] = np.arange(m + 1)
    for i in range(1, n + 1):
        for j in range(1, m + 1):
            d[i, j] = min(d[i - 1, j] + 1, d[i, j - 1] + 1, d[i - 1, j - 1] + (ref[i - 1] != hyp[j - 1]))
    i, j, D, S, I = n, m, 0, 0, 0
    while i > 0 or j > 0:
        if i > 0 and j > 0 and d[i, j] == d[i - 1, j - 1] + (ref[i - 1] != hyp[j - 1]):
            S += int(ref[i - 1] != hyp[j - 1])
            i, j = i - 1, j - 1
        elif i > 0 and d[i, j] == d[i - 1, j] + 1:
            D, i = D + 1, i - 1
        else:
            I, j = I + 1, j - 1
    return int(D), int(S), int(I)


def cmd_asr():
    seg_dir = BASE / "asr_segs"
    seg_dir.mkdir(exist_ok=True)
    res = {}
    for c in clips():
        cid = c["id"]
        tracks = {"ref": sf.read(str(SRC / f"{cid}_owner.wav"), dtype="float32")[0]}
        tracks["mix"] = sf.read(str(SRC / f"{cid}_mix.wav"), dtype="float32")[0]
        for v in ASR_VERS[1:]:
            tracks[v] = sf.read(str(OUT / f"{cid}.{v}.wav"), dtype="float32")[0]
        rows = []
        for k, (s, e) in enumerate(c["timeline"]["owner"]):
            a, b = int(max(0, s - 0.2) * SR), int((e + 0.3) * SR)
            row = {"seg": k, "t": [s, e]}
            for name, x in tracks.items():
                f = seg_dir / f"{cid}_{k:02d}_{name}.wav"
                sf.write(str(f), x[a:b], SR, subtype="FLOAT")
                row[name] = asr(f)
            rows.append(row)
            print(cid, k, row["ref"][:40])
        res[cid] = rows
    (HERE / "results").mkdir(exist_ok=True)
    (HERE / "results/asr.json").write_text(json.dumps({"model": ASR_MODEL, "service": ASR_URL, "per_clip": res},
                                                     ensure_ascii=False, indent=1) + "\n")


def db(x):
    return float(10 * np.log10(x + 1e-20))


def cmd_score(tag=""):
    truth = json.loads((OUT / "truth_emb.json").read_text())
    asr_res = json.loads((HERE / "results/asr.json").read_text())["per_clip"] if (HERE / "results/asr.json").exists() else {}
    per = {}
    for c in clips():
        cid = c["id"]
        own = sf.read(str(SRC / f"{cid}_owner.wav"), dtype="float32")[0]
        itf = sf.read(str(SRC / f"{cid}_interf.wav"), dtype="float32")[0]
        log = json.loads((OUT / f"{cid}.log{tag}.json").read_text())
        WIN = log["judge"]["window_s"]
        ys = {v: sf.read(str(OUT / f"{cid}.{v}{tag if v in TAGGED else ''}.wav"), dtype="float32")[0] for v in VERS}
        ys["mix"] = sf.read(str(SRC / f"{cid}_mix.wav"), dtype="float32")[0][: len(ys["raw"])]
        n = len(ys["raw"])
        own, itf = own[:n], itf[:n]
        om = vad_mask(own)[0][:n] if own.any() else np.zeros(n, bool)
        im = vad_mask(itf)[0][:n] if itf.any() else np.zeros(n, bool)
        om = np.pad(om, (0, n - len(om)))
        im = np.pad(im, (0, n - len(im)))
        io, sil = im & ~om, ~im & ~om
        # 注册
        lock_t, events = None, []
        for l in log["judge_logs"]:
            if l["event"]:
                events.append({"t": l["t"], **l["event"]})
                if lock_t is None and "locked" in l["event"]:
                    lock_t = l["t"]
        cl = log["clusters"]
        def who(idx):
            if idx is None or "interf" not in truth[cid]:
                return None if idx is None else "owner?"
            cen = np.array(cl[idx]["centroid"])
            so, si = float(cen @ np.array(truth[cid]["owner"])), float(cen @ np.array(truth[cid]["interf"]))
            return {"is_owner": so > si, "cos_owner": so, "cos_interf": si}
        own_before = sum(max(0.0, min(e, lock_t) - s) for s, e in c["timeline"]["owner"]) if lock_t else None
        # 事件里的簇号会因挤掉而变，最终簇用 owner_cluster；锁定对错按最终与各事件时的簇中心无法复原，报最终
        final = who(log["owner_cluster"])  # 只作参考：整段干净主人轨的声纹与降噪后窗口的声纹余弦普遍偏低（见 README）
        # 判决对错（按时间表核对；只看锁定之后）：主人独白窗 / 邻座独白窗各被判成什么
        vc = {"owner_win": {"owner": 0, "others": 0, "unknown": 0}, "interf_win": {"owner": 0, "others": 0, "unknown": 0}}
        false_other = 0
        for l in log["judge_logs"]:
            a, b = int((l["t"] - WIN) * SR), int(l["t"] * SR)
            fo, fi = om[max(0, a):b].mean(), im[max(0, a):b].mean()
            if l["class"] == "others" and fo > 0.3:
                false_other += 1
            if lock_t is None or l["t"] < lock_t or l["class"] == "nospeech":
                continue
            if fo >= 0.6 and fi < 0.1:
                vc["owner_win"][l["class"]] += 1
            elif fi >= 0.6 and fo < 0.1:
                vc["interf_win"][l["class"]] += 1
        ow = vc["owner_win"]
        lock_correct = None if lock_t is None or (ow["owner"] + ow["others"]) == 0 else (ow["owner"] > ow["others"])
        # IRL
        lk = int(lock_t * SR) if lock_t else n
        irl, irl_post = {}, {}
        for v, y in ys.items():
            if io.any() and len(y) == n:
                irl[v] = db(np.sum(y[io] ** 2)) - db(np.sum(itf[io] ** 2))
                post = io.copy()
                post[:lk] = False
                irl_post[v] = db(np.sum(y[post] ** 2)) - db(np.sum(itf[post] ** 2)) if post.any() else None
        # 停顿里开着的比例（hop 级）
        nh = n // HOP
        hop_sil = sil[: nh * HOP].reshape(nh, HOP).all(axis=1)
        hop_own = om[: nh * HOP].reshape(nh, HOP).any(axis=1)
        hop_io = io[: nh * HOP].reshape(nh, HOP).all(axis=1)
        eo = np.array(log["per_hop"]["energy_open"][:nh], bool)
        vo = np.array(log["per_hop"]["vad_open"][:nh], bool)
        open_ratio = {
            "energy": {"silence": float(eo[hop_sil].mean()) if hop_sil.any() else None, "owner_speech": float(eo[hop_own].mean()),
                       "interf_only": float(eo[hop_io].mean()) if hop_io.any() else None},
            "vad": {"silence": float(vo[hop_sil].mean()) if hop_sil.any() else None, "owner_speech": float(vo[hop_own].mean()),
                    "interf_only": float(vo[hop_io].mean()) if hop_io.any() else None},
        }
        # 主人被压（门控增益）：保守 / 激进两套；全段与「锁定之后」分开报（锁定前门控不动，全段会稀释）
        onset_hops = np.zeros(nh, bool)
        for s_, e_ in c["timeline"]["owner"]:
            onset_hops[int(s_ * SR / HOP): min(nh, int(min(e_, s_ + 2.0) * SR / HOP))] = True
        post = np.arange(nh) * HOP >= lk

        def supp(key):
            g = np.array(log["per_hop"][key][:nh])
            gdb = 20 * np.log10(np.maximum(g, 1e-6))
            def frac(m):
                return float((gdb[m] < -3).mean()) if m.any() else None
            def mean(m):
                return float(gdb[m].mean()) if m.any() else None
            return {
                "hops_lt_m3db": frac(hop_own), "hops_lt_m3db_first2s": frac(onset_hops & hop_own),
                "hops_lt_m3db_postlock": frac(hop_own & post), "hops_lt_m3db_first2s_postlock": frac(onset_hops & hop_own & post),
                "mean_gain_db_owner": mean(hop_own), "mean_gain_db_interf_only": mean(hop_io),
                "mean_gain_db_interf_only_postlock": mean(hop_io & post),
            }
        owner_supp = supp("gain")
        owner_supp_aggr = supp("gain_aggressive")
        # 背景：无人声 hop 的输出电平（vad vs personal），看门控有没有让底噪忽大忽小
        sil_lvl = {v: db(np.mean(ys[v][sil] ** 2)) if sil.any() else None for v in ("mix", "raw", "vad", "personal", "aggressive")}
        # 吃字
        wer = {}
        if cid in asr_res:
            for v in ASR_VERS:
                if v not in asr_res[cid][0]:
                    continue
                D = S = I = N = 0
                for r in asr_res[cid]:
                    ref = tokens(r["ref"], "en")
                    d, s_, i = edit_ops(ref, tokens(r[v], "en"))
                    D, S, I, N = D + d, S + s_, I + i, N + len(ref)
                wer[v] = {"del": D / max(N, 1), "sub": S / max(N, 1), "ins": I / max(N, 1), "ref_words": N,
                          "n_del": D, "n_sub": S, "n_ins": I}
        comp = np.array([l["compute_ms"] for l in log["judge_logs"]])
        emb_comp = np.array([l["compute_ms"] for l in log["judge_logs"] if l["sim"] is not None or l["enrolled"] or l["class"] in ("unknown", "owner", "others")])
        st = {k: np.array(log["per_hop"][k]) / 1e3 for k in ("stage_ns_vad_env", "stage_ns_gate")}
        per[cid] = {
            "kind": c["kind"], "params": c["params"],
            "enroll": {"lock_t": lock_t, "owner_speech_before_lock_s": own_before, "events": events, "final_owner": final,
                       "lock_correct": lock_correct, "n_relock": sum(1 for e in events if "relocked" in e)},
            "verdicts": {"false_other_on_owner_any": false_other, "postlock": vc, "n": len(log["judge_logs"])},
            "irl_db": irl, "irl_db_postlock": irl_post, "open_ratio": open_ratio, "owner_suppression": owner_supp, "owner_suppression_aggressive": owner_supp_aggr,
            "silence_level_db": sil_lvl, "asr": wer,
            "compute": {"judge_ms_p50": float(np.median(emb_comp)) if len(emb_comp) else None,
                        "judge_ms_max": float(emb_comp.max()) if len(emb_comp) else None,
                        "judge_ticks": int(len(comp)), "embed_ticks": int(len(emb_comp)),
                        "vad_env_us_p50": float(np.median(st["stage_ns_vad_env"])), "vad_env_us_max": float(st["stage_ns_vad_env"].max()),
                        "gate_us_p50": float(np.median(st["stage_ns_gate"])), "gate_us_max": float(st["stage_ns_gate"].max())},
        }
        print(cid, json.dumps({"lock": lock_t, "ok": lock_correct, "vc": vc, "irl_post_vad": irl_post.get("vad"), "irl_post_pers": irl_post.get("personal"),
                               "irl_post_aggr": irl_post.get("aggressive"), "irl_post_raw": irl_post.get("raw"),
                               "supp": owner_supp["hops_lt_m3db_postlock"], "supp2s": owner_supp["hops_lt_m3db_first2s_postlock"],
                               "supp_aggr": owner_supp_aggr["hops_lt_m3db_postlock"], "supp2s_aggr": owner_supp_aggr["hops_lt_m3db_first2s_postlock"]}, default=str))
    (HERE / "results").mkdir(exist_ok=True)
    (HERE / f"results/scores{tag}.json").write_text(json.dumps({"per_clip": per}, ensure_ascii=False, indent=1) + "\n")


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "run":
        cmd_run(sys.argv[2])
    elif cmd == "asr":
        cmd_asr()
    elif cmd == "score":
        cmd_score(sys.argv[2] if len(sys.argv) > 2 else "")
