"""exp008 · 合成数据合格检查（synth-standard.md 的数据侧检查，不跑任何降噪模型）。

  S1 可复现：同一 build 参数重生成一个子集，逐文件 sha256 相同
  S2 可控：单类轴上同一段素材的各档，只有被扫的那个因素在变（其余轨逐样本相同或只差整体缩放；实测因素值单调、贴近名义值）
  S3 真值完整：三轨相加 = mix（残差 ≤ −60 dB）；逐帧标签存在且长度对得上；名义值与实测值都记在清单里
  S6 统计对齐（噪声素材）：合成噪声 vs 本机真实录音同类，三组统计量（1/3 倍频程长时谱形状 · 包络调制谱 · 20 ms 电平分布）；
     过线 = 合成样本到真实类中心的距离 ≤ 「真实留出样本到同一中心距离」的 95 分位（真实自己的离散度当尺子），过线比例 ≥ 80%〔建议值〕
结果写 docs/experiments/exp008-bench-validity/results/<build>-checks.json（进仓）。

用法：$PY docs/experiments/exp008-bench-validity/checks.py <build> [--audio-root <dir>] [--docs-root <dir>] [--n-align 24]
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

import numpy as np
import soundfile as sf
from scipy import signal

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cells  # noqa: E402
import sim  # noqa: E402  （cells 已把 exp006 加进路径）

SR = cells.SR


# ---------------- 统计量（S6） ----------------
THIRD_OCT = [100 * 2 ** (k / 3) for k in range(0, 19)]  # 100 Hz … 6.4 kHz 中心频率


def ltas_shape(x):
    """1/3 倍频程长时谱（Welch），减去均值 → 只留形状（dB）。"""
    f, p = signal.welch(x, SR, nperseg=1024)
    out = []
    for fc in THIRD_OCT:
        sel = (f >= fc / 2 ** (1 / 6)) & (f < fc * 2 ** (1 / 6))
        out.append(10 * np.log10(np.mean(p[sel]) + 1e-20))
    out = np.array(out)
    return out - out.mean()


MOD_F = [0.5, 1, 2, 4, 8, 16, 32]


def mod_spectrum(x):
    """四个倍频程带（250–500 · 500–1k · 1–2k · 2–4k Hz）的包络调制谱，按倍频程调制频带取能量，相对各带直流归一（dB）。"""
    out = []
    for lo in (250, 500, 1000, 2000):
        b = sim.butter(x, "bandpass", (lo, lo * 2), 4)
        env = np.abs(signal.hilbert(b))
        env = signal.resample_poly(env, 1, 80)  # 200 Hz 包络采样
        fs = SR / 80
        f, p = signal.welch(env - env.mean(), fs, nperseg=min(len(env), 512))
        dc = env.mean() ** 2 + 1e-20
        for fm in MOD_F:
            sel = (f >= fm / np.sqrt(2)) & (f < fm * np.sqrt(2))
            out.append(10 * np.log10(np.sum(p[sel]) / dc + 1e-20) if sel.any() else -200.0)
    return np.array(out)


def level_dist(x):
    """20 ms 帧电平分布：p10 · p50 · p90 · 标准差（相对 p50，只留形状）。"""
    fr = 320
    nf = len(x) // fr
    e = 10 * np.log10(np.mean(x[: nf * fr].reshape(nf, fr) ** 2, axis=1) + 1e-20)
    p10, p50, p90 = np.percentile(e, [10, 50, 90])
    return np.array([p10 - p50, p90 - p50, np.std(e)])


STATS = {"ltas": ltas_shape, "mod": mod_spectrum, "level": level_dist}


def feats(x):
    x = np.asarray(x, np.float64)
    x = x / (sim.rms(x) + 1e-12)
    return {k: f(x) for k, f in STATS.items()}


def dist(a, b):
    return float(np.sqrt(np.mean((a - b) ** 2)))


# 名义噪声类型 → 本机真实录音参照（只评测；DEMAND CC BY 4.0 / BY-SA 冲突待 014 复核，按更严处理，不进训练）
REAL_REF = {
    "cafe": ["PCAFETER", "SCAFE"],
    "babble": ["PCAFETER", "SCAFE", "PRESTO"],
    "clatter": ["DKITCHEN"],
    "traffic": ["STRAFFIC", "TBUS"],
    "music": ["__piano__"],
    "stationary": [],  # 本机没有真实空调 / 风扇录音 → 判「无真实参照，D0」
}


def real_samples(env, part, k, n, seed):
    rng = np.random.default_rng(seed)
    out = []
    for i in range(k):
        if env == "__piano__":
            x, _ = sim.file_music(rng, n)
        else:
            x, _ = sim.demand(rng, env, "train" if part == "a" else "test", n)
        out.append(x)
    return out


def align(n_per, n):
    res = {}
    for kind, envs in REAL_REF.items():
        if not envs:
            res[kind] = {"verdict": "无真实参照", "level": "D0"}
            continue
        ref = [feats(x) for e in envs for x in real_samples(e, "a", n_per, n, 1)]
        held = [feats(x) for e in envs for x in real_samples(e, "b", n_per, n, 2)]
        center = {s: np.mean([r[s] for r in ref], axis=0) for s in STATS}
        thr = {s: float(np.percentile([dist(h[s], center[s]) for h in held], 95)) for s in STATS}
        row = {"real_envs": envs, "n_ref": len(ref), "n_held": len(held), "thr_p95": thr, "gen": {}}
        for gen, tab in (("A", cells.NOISE_A), ("B", cells.NOISE_B)):
            rng = np.random.default_rng(3 if gen == "A" else 4)
            xs = [tab[kind](rng, n, "cafe")[0] for _ in range(n_per)]
            fs = [feats(sim.fit(np.asarray(x, np.float64), n)) for x in xs]
            g = {}
            for s in STATS:
                d = [dist(f[s], center[s]) for f in fs]
                g[s] = {"median_dist": round(float(np.median(d)), 2), "pass_rate": round(float(np.mean([v <= thr[s] for v in d])), 2)}
            worst = min(g[s]["pass_rate"] for s in STATS)
            # 三档〔建议值〕：真实留出样本自己按 p95 线约 95% 过线，且同类别真实环境互比也只有 75–100%（smoke 实测），
            # 所以 0.8 附近是噪声底：≥ 0.8 过 · 0.5–0.8 临界（不判，加样本）· < 0.5 不过
            g["verdict"] = "过" if worst >= 0.8 else ("临界" if worst >= 0.5 else "不过")
            g["pass"] = g["verdict"] == "过"
            row["gen"][gen] = g
        res[kind] = row
    return res


# ---------------- S1–S3 ----------------
def load_rows(docs):
    return [json.loads(l) for l in open(docs / "truth.jsonl")]


def read(root, rel):
    return sf.read(root / rel, dtype="float64")[0]


def check_truth(rows, root):
    worst = -999.0
    bad_len = 0
    for r in rows:
        f = r["files"]
        mix = read(root, f["mix"]["path"])
        s = sum(read(root, f[k]["path"]) for k in ("owner", "interf", "noise"))
        err = 20 * np.log10(sim.rms(mix - s) / sim.rms(mix))
        worst = max(worst, err)
        if len(r["labels"]) != len(mix) // cells.HOP:
            bad_len += 1
    return {"worst_residual_db": round(worst, 1), "label_len_mismatch": bad_len, "pass": worst <= -60 and bad_len == 0}


def measured(r, root):
    f = r["files"]
    o = read(root, f["owner"]["path"])
    i = read(root, f["interf"]["path"])
    nz = read(root, f["noise"]["path"])
    lo = cells.active_level(o)
    lab = np.array(r["labels"])
    own = np.isin(lab, (1, 3))
    return {"snr_db": 20 * np.log10(lo / sim.rms(nz)),
            "sir_db": None if not np.any(i) else 20 * np.log10(lo / cells.active_level(i)),
            "t60_s": r["rir"]["t60_s"], "drr_db": r["rir"]["drr_db"],
            "overlap": float(np.mean(lab[own] == 3)) if own.any() else 0.0}


def corr1(a, b):
    if not np.any(a) and not np.any(b):
        return 1.0
    return float(np.dot(a, b) / (np.linalg.norm(a) * np.linalg.norm(b) + 1e-20))


def check_control(rows, root):
    """单类轴：同一 clip 各档，未被扫的轨必须相同（相关 ≥ 0.9999，允许整体缩放）；被扫因素的实测值随名义值单调。"""
    varied_track = {"snr_db": "noise", "sir_db": "interf", "room": None, "noise": "noise"}
    out = {}
    for cell, (key, levels) in cells.AXES.items():
        rs = [r for r in rows if r["cell"] == cell]
        if not rs:
            continue
        frozen_ok, mono_ok, total, err = 0, 0, 0, []
        for gen in ("A", "B"):
            for c in sorted({r["clip"] for r in rs if r["gen"] == gen}):
                grp = [r for r in rs if r["gen"] == gen and r["clip"] == c]
                total += 1
                ref = grp[0]
                ok = True
                for k in ("owner", "interf", "noise"):
                    if k == varied_track[key] or (key == "room"):
                        continue
                    a = read(root, ref["files"][k]["path"])
                    for g in grp[1:]:
                        if corr1(a, read(root, g["files"][k]["path"])) < 0.9999:
                            ok = False
                frozen_ok += ok
                m = [measured(g, root) for g in grp]
                if key in ("snr_db", "sir_db"):
                    pairs = [(g["level"], mm[key]) for g, mm in zip(grp, m) if g["level"] is not None and mm[key] is not None]
                    pairs.sort()
                    mono_ok += all(b[1] > a[1] for a, b in zip(pairs, pairs[1:]))
                    err += [abs(v - nom) for nom, v in pairs]
                elif key == "room":
                    t = [mm["t60_s"] or 0 for mm in m]
                    mono_ok += all(b > a for a, b in zip(t, t[1:]))
                else:
                    mono_ok += 1
        out[cell] = {"clips": total, "others_frozen": frozen_ok, "monotone": mono_ok,
                     "max_abs_err_db": round(max(err), 2) if err else None,
                     "pass": frozen_ok == total and mono_ok == total and (not err or max(err) <= 1.0)}
    return out


def check_repro(build, docs, root, out_root, docs_root):
    """S1：同参数重生成 S-SNR 子集到旁边的目录，逐文件比 sha256。"""
    tmp = Path(out_root) / (build + "-repro")
    n = len({r["clip"] for r in load_rows(docs)})
    cmd = [sys.executable, str(HERE / "cells.py"), "--build", build, "--n", str(n), "--only", "S-SNR",
           "--out-root", str(tmp), "--docs-root", str(tmp / "docs")]
    subprocess.run(cmd, check=True, capture_output=True)
    a = {(r["gen"], r["cell"], r["clip"], str(r["level"])): r["files"] for r in load_rows(docs) if r["cell"] == "S-SNR"}
    b = {(r["gen"], r["cell"], r["clip"], str(r["level"])): r["files"] for r in load_rows(tmp / "docs" / build)}
    same = sum(all(a[k][t]["pcm_sha256"] == b[k][t]["pcm_sha256"] for t in a[k]) for k in a if k in b)
    return {"compared": len(a), "identical": same, "pass": len(a) > 0 and same == len(a)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--audio-root", default=str(cells.OUT_ROOT))
    ap.add_argument("--docs-root", default=str(HERE / "builds"))
    ap.add_argument("--results", default=str(HERE / "results"))
    ap.add_argument("--n-align", type=int, default=24)
    a = ap.parse_args()
    docs = Path(a.docs_root) / a.build
    root = Path(a.audio_root) / a.build
    rows = load_rows(docs)
    rep = {"build": a.build,
           "S1_repro": check_repro(a.build, docs, root, a.audio_root, a.docs_root),
           "S2_control": check_control(rows, root),
           "S3_truth": check_truth(rows, root),
           "S6_align_noise": align(a.n_align, int(4.0 * SR))}
    out = Path(a.results)
    out.mkdir(parents=True, exist_ok=True)
    (out / f"{a.build}-checks.json").write_text(json.dumps(rep, ensure_ascii=False, indent=1, default=lambda o: o.item() if hasattr(o, "item") else str(o)))
    print(json.dumps(rep, ensure_ascii=False, indent=1, default=lambda o: o.item() if hasattr(o, "item") else str(o))[:4000])


if __name__ == "__main__":
    main()
