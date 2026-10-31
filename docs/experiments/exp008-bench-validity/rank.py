"""exp008 第二步：系统阶梯 × 条件格 → 每段每系统的指标（给 rankstat.py 算排名效度）。

系统阶梯（第一轮，人 / 大总管定 ≥ 12 个；盘点见 docs/research/016_bench-synth/notes/G-system-ladder.md）。
**全部只作对照、不进产品。** 族：C 经典 DSP · A DNS3/SEtrain 系 · A′ VCTK · B DFN 系 · O 作弊天花板。

指标是 **Python 近似口径**，只用于「排名一致性」检查，不是 verifier 正本（正本在 Rust `src/eval/`，
那里还没有「对任意清单打分」的入口，见 mvp.md 给实现线的建议）：
  si_sdr（对主讲人轨）· estoi（对主讲人轨）· dnsmos_ovrl / sig / bak（无参考）·
  noise_att_db（无人声帧：输出功率 − 带噪功率，越低压得越多，对应 V-NR）·
  spl_db（主讲人帧：输出在主讲人轨上的投影增益，对应 V-SPLVL；越接近 0 越好）·
  nonowner_att_db（只有干扰者帧：输出 − 带噪，对应 V-INTF 的近似）

用法：$PY docs/experiments/exp008-bench-validity/rank.py <build> [--systems a,b] [--jobs 8] [--limit N]
输出：docs/experiments/exp008-bench-validity/results/<build>-scores.jsonl（进仓，只有数，没有音频）
"""
import argparse
import hashlib
import json
import os
import sys
import tempfile
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
EXP003 = HERE.parent / "exp003-offline-ab"
sys.path.insert(0, str(EXP003))
MAIN = Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling")
RES = MAIN / "resources"
SR = 16000
HOP = 256

SYSTEMS = {  # id: (族, 说明)
    "passthrough": ("C", "直通（地板）"),
    "specsub": ("C", "谱减 α=3、下限 0.02，最小统计噪声估计"),
    "wiener_dd": ("C", "判决引导维纳 + 最小统计噪声估计（numpy；思路同 vdn-eval ref，不是同一份代码）"),
    "mmse_lsa": ("C", "Ephraim–Malah MMSE 对数谱幅度"),
    "gtcrn_vctk": ("A'", "GTCRN VCTK-DEMAND 版（biglinux 导出）"),
    "gtcrn": ("A", "GTCRN DNS3（D33 主模型）"),
    "gtcrn_mix05": ("A", "GTCRN 湿干比 0.5（参数变体）"),
    "ulunas": ("A", "UL-UNAS DNS3（与 GTCRN 同作者同模板）"),
    "fe_s": ("A", "FastEnhancer-S（同训练集）"),
    "fe_m": ("A", "FastEnhancer-M（同训练集）"),
    "dpdf_base": ("B", "DPDFNet baseline"),
    "dpdf2": ("B", "DPDFNet-2（exp003 候选）"),
    "dpdf8": ("B", "DPDFNet-8"),
    "oracle_irm": ("O", "理想比值掩码（用干净轨，作弊天花板）"),
}
# 第二轮（人定 2026-10-31：Hush dylib 允许、torch 现在装）：这些系统由 enh/ 下各自的脚本在自己的运行环境里先跑完，
# 增强音频写到 <audio-root>/<build>-enh/<id>/<mix 相对路径>（16 kHz float32，已补偿延迟、与输入等长），本脚本只读文件算指标。
# 全部只作对照、不进产品；「非因果」的连产品候选都不是。
FILE_SYSTEMS = {
    "hush": ("B", "Hush（Weya DFN-SE 16 k，预编译 arm64 dylib，衰减不设上限）"),
    "hush_lim12": ("B", "Hush 衰减上限 12 dB（参数变体）"),
    "dfn3": ("B", "DeepFilterNet3 默认版 48 k（经 Hush dylib 内的 libDF；权重许可不明，只作对照）"),
    "rnnoise": ("E", "RNNoise（nnnoiseless Rust 版，48 k）"),
    "webrtc_ns18": ("C*", "WebRTC NS 18 dB 档（sonora-ns；DSP，但噪声估计与增益规则不同于 C 族）"),
    "webrtc_ns6": ("C*", "WebRTC NS 6 dB 档（参数变体）"),
    "frcrn": ("F", "FRCRN_SE_16K（ClearerVoice，非因果）"),
    "mossformer_gan": ("F", "MossFormerGAN_SE_16K（ClearerVoice，非因果；与 FRCRN 同家同数据）"),
    "mossformer2_48k": ("F", "MossFormer2_SE_48K（ClearerVoice，非因果，48 k 往返）"),
    "urgent_bsrnn": ("G", "URGENT 2026 BSRNN 基线（非因果）"),
    "gmap_se": ("I", "G-MaP-SE（VoiceBank+DEMAND，非因果）"),
}
SYSTEMS.update(FILE_SYSTEMS)


# ---------------- 经典 DSP（numpy，512/256 sqrt-Hann） ----------------
def _win():
    return np.sqrt(0.5 - 0.5 * np.cos(2 * np.pi * np.arange(512) / 512))


def stft(x):
    w = _win()
    xp = np.concatenate([np.zeros(512), x, np.zeros(512)])
    n = (len(xp) - 512) // HOP + 1
    return np.stack([np.fft.rfft(xp[i * HOP: i * HOP + 512] * w) for i in range(n)]), len(x)


def istft(S, n):
    w = _win()
    y = np.zeros((len(S) - 1) * HOP + 512)
    for i, s in enumerate(S):
        y[i * HOP: i * HOP + 512] += np.fft.irfft(s, 512) * w
    return y[512: 512 + n]


def noise_track(P, win_s=1.5):
    """最小统计（简化）：功率先做 0.7 指数平滑，在 win_s 滑窗内取最小，×1.5 偏差补偿。只用过去的帧，因果。"""
    k = max(1, int(win_s * SR / HOP))
    sm = np.zeros_like(P)
    acc = P[0]
    for t in range(len(P)):
        acc = 0.7 * acc + 0.3 * P[t]
        sm[t] = acc
    out = np.zeros_like(P)
    for t in range(len(P)):
        out[t] = sm[max(0, t - k + 1): t + 1].min(axis=0) * 1.5
    return out + 1e-12


def specsub(x):
    S, n = stft(x)
    P = np.abs(S) ** 2
    N = noise_track(P)
    g = np.sqrt(np.maximum(1 - 3.0 * N / (P + 1e-12), 0.02 ** 2))
    return istft(S * g, n)


def wiener_dd(x, alpha=0.98, gmin=10 ** (-15 / 20)):
    S, n = stft(x)
    P = np.abs(S) ** 2
    N = noise_track(P)
    G = np.ones(S.shape[1])
    out = np.zeros_like(S)
    prev = np.zeros(S.shape[1])
    for t in range(len(S)):
        post = P[t] / N[t]
        prio = alpha * prev / N[t] + (1 - alpha) * np.maximum(post - 1, 0)
        G = np.maximum(prio / (1 + prio), gmin)
        out[t] = S[t] * G
        prev = np.abs(out[t]) ** 2
    return istft(out, n)


def mmse_lsa(x, alpha=0.98, gmin=10 ** (-20 / 20)):
    from scipy.special import exp1
    S, n = stft(x)
    P = np.abs(S) ** 2
    N = noise_track(P)
    out = np.zeros_like(S)
    prev = np.zeros(S.shape[1])
    for t in range(len(S)):
        post = np.maximum(P[t] / N[t], 1e-6)
        prio = np.maximum(alpha * prev / N[t] + (1 - alpha) * np.maximum(post - 1, 0), 1e-4)
        v = prio / (1 + prio) * post
        G = prio / (1 + prio) * np.exp(0.5 * exp1(np.maximum(v, 1e-8)))
        G = np.clip(G, gmin, 1.0)
        out[t] = S[t] * G
        prev = np.abs(out[t]) ** 2
    return istft(out, n)


def oracle_irm(x, owner):
    S, n = stft(x)
    O, _ = stft(owner)
    R = S - O
    m = np.sqrt(np.abs(O) ** 2 / (np.abs(O) ** 2 + np.abs(R) ** 2 + 1e-12))
    return istft(S * m, n)


# ---------------- 模型（调 exp003 enhance.py 的类，只调用不复制） ----------------
_MODELS = {}


def _enh():
    import enhance  # noqa: E402  exp003
    return enhance


class UlUnas:
    """UL-UNAS 流式 ONNX：输入 mix [1,257,1,2] + 3 个缓存。分析窗 Hann（周期）；合成仿 torch.istft：OLA 后逐样本除以窗平方包络。"""
    name = "ulunas"

    def __init__(self):
        e = _enh()
        self.sess = e.session(RES / "ul-unas/ulunas_onnx/onnx_models/ulunas_stream_simple.onnx")
        self.n_fft, self.hop, self.delay_frames = 512, 256, 0
        self.win = (0.5 - 0.5 * np.cos(2 * np.pi * np.arange(512) / 512)).astype(np.float32)
        ins = self.sess.get_inputs()
        self.cache_names = [i.name for i in ins[1:]]
        self.caches = {i.name: np.zeros(i.shape, np.float32) for i in ins[1:]}
        self.out_names = [o.name for o in self.sess.get_outputs()]

    def step(self, spec):
        feed = {"mix": np.stack([spec.real, spec.imag], -1)[None, :, None].astype(np.float32), **self.caches}
        outs = self.sess.run(self.out_names, feed)
        for name, val in zip(self.cache_names, outs[1:]):
            self.caches[name] = val
        y = outs[0].reshape(-1, 2)
        return y[:, 0] + 1j * y[:, 1]


def run_ulunas(x):
    m = UlUnas()
    w = m.win
    xp = np.concatenate([np.zeros(512, np.float32), x.astype(np.float32), np.zeros(1024, np.float32)])
    nfr = (len(xp) - 512) // 256 + 1
    y = np.zeros(len(xp) + 512)
    env = np.zeros(len(xp) + 512)
    for k in range(nfr):
        seg = xp[k * 256: k * 256 + 512]
        out = m.step(np.fft.rfft(seg * w))
        y[k * 256: k * 256 + 512] += np.fft.irfft(out, 512) * w
        env[k * 256: k * 256 + 512] += w ** 2
    y = y / np.maximum(env, 1e-8)
    return y[512: 512 + len(x)]


def make_model(sid):
    e = _enh()
    if sid in ("gtcrn", "gtcrn_mix05"):
        e.GTCRN = RES / "gtcrn/stream/onnx_models/gtcrn_simple.onnx"
        return e.Gtcrn()
    if sid == "gtcrn_vctk":
        e.GTCRN = RES / "biglinux-gtcrn/stream/onnx_models/gtcrn_vctk.onnx"
        return e.Gtcrn()
    if sid in ("dpdf_base", "dpdf2", "dpdf8"):
        e.DPDF = RES / "weights/dpdfnet" / {"dpdf_base": "baseline.onnx", "dpdf2": "dpdfnet2.onnx", "dpdf8": "dpdfnet8.onnx"}[sid]
        return e.Dpdfnet()
    raise KeyError(sid)


def process(sid, x, owner, enh_path=None):
    x = x.astype(np.float32)
    if sid in FILE_SYSTEMS:
        y = sf.read(enh_path, dtype="float64")[0]
        if len(y) != len(x):
            raise RuntimeError(f"长度 {len(y)} ≠ 输入 {len(x)}")
        return y
    if sid == "passthrough":
        return x.astype(np.float64)
    if sid == "specsub":
        return specsub(x)
    if sid == "wiener_dd":
        return wiener_dd(x)
    if sid == "mmse_lsa":
        return mmse_lsa(x)
    if sid == "oracle_irm":
        return oracle_irm(x, owner)
    if sid == "ulunas":
        return run_ulunas(x)
    e = _enh()
    if sid in ("fe_s", "fe_m"):
        return e.FastEnhancer(size=sid[-1]).run(x).astype(np.float64)
    y = e.stream(make_model(sid), x).astype(np.float64)  # 每段新建模型，状态从零开始
    if sid == "gtcrn_mix05":
        y = 0.5 * y + 0.5 * x
    return y


# ---------------- 指标 ----------------
def si_sdr(est, ref):
    est, ref = est - est.mean(), ref - ref.mean()
    t = np.dot(est, ref) / (np.dot(ref, ref) + 1e-20) * ref
    return float(10 * np.log10(np.sum(t ** 2) / (np.sum((est - t) ** 2) + 1e-20) + 1e-20))


def hop_power(x):
    nf = len(x) // HOP
    return np.mean(x[: nf * HOP].reshape(nf, HOP) ** 2, axis=1)


_DNS = None


def dnsmos(y):
    global _DNS
    if _DNS is None:
        sys.path.insert(0, str(RES / "DNS-Challenge/DNSMOS"))
        from dnsmos_local import ComputeScore
        d = RES / "DNS-Challenge/DNSMOS/DNSMOS"
        _DNS = ComputeScore(str(d / "sig_bak_ovr.onnx"), str(d / "model_v8.onnx"))
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=True) as f:
        sf.write(f.name, y.astype(np.float32), SR)
        s = _DNS(f.name, SR, False)
    return float(s["OVRL"]), float(s["SIG"]), float(s["BAK"])


def metrics(y, mix, owner, labels):
    from pystoi import stoi
    n = min(len(y), len(mix))
    y, mix, owner = y[:n], mix[:n], owner[:n]
    lab = np.array(labels)[: n // HOP]
    py, pm = hop_power(y), hop_power(mix)
    nv = lab == 0
    oi = lab == 2
    ow = np.isin(lab, (1, 3))
    ovrl, sig, bak = dnsmos(y)
    m = {"si_sdr": si_sdr(y, owner), "estoi": float(stoi(owner, y, SR, extended=True)),
         "dnsmos_ovrl": ovrl, "dnsmos_sig": sig, "dnsmos_bak": bak,
         "noise_att_db": float(10 * np.log10(py[nv].mean() / pm[nv].mean())) if nv.sum() >= 10 else None,
         "nonowner_att_db": float(10 * np.log10(py[oi].mean() / pm[oi].mean())) if oi.sum() >= 10 else None}
    if ow.any():
        idx = np.repeat(ow, HOP)[:n]
        idx = np.concatenate([idx, np.zeros(n - len(idx), bool)])
        g = np.dot(y[idx], owner[idx]) / (np.dot(owner[idx], owner[idx]) + 1e-20)
        m["spl_db"] = float(20 * np.log10(max(abs(g), 1e-6)))
    return {k: (None if v is None else round(v, 4)) for k, v in m.items()}


def job(args):
    row, root, systems = args
    f = row["files"]
    mix = sf.read(root / f["mix"]["path"], dtype="float64")[0]
    owner = sf.read(root / f["owner"]["path"], dtype="float64")[0]
    out = []
    for sid in systems:
        t0 = time.time()
        try:
            y = process(sid, mix, owner, root.parent / f"{root.name}-enh" / sid / f["mix"]["path"])
            if not np.all(np.isfinite(y)):
                raise RuntimeError("非有限值")
            m = metrics(y, mix, owner, row["labels"])
            err = None
        except Exception as ex:  # 单个系统失败不停摆，记下来
            m, err = {}, f"{type(ex).__name__}: {ex}"
        out.append({"cell": row["cell"], "gen": row["gen"], "clip": row["clip"], "level": row["level"],
                    "system": sid, "family": SYSTEMS[sid][0], "metrics": m, "error": err, "sec": round(time.time() - t0, 2)})
    return out


def sha_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def main():
    os.environ.setdefault("OMP_NUM_THREADS", "1")
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--systems", default=",".join(SYSTEMS))
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--audio-root", default=str(MAIN / "_tmp/exp008"))
    ap.add_argument("--docs-root", default=str(HERE / "builds"))
    ap.add_argument("--results", default=str(HERE / "results"))
    ap.add_argument("--tag", default="", help="输出名后缀，例如 r2；第二轮写新文件，不覆盖第一轮")
    a = ap.parse_args()
    systems = [s for s in a.systems.split(",") if s]
    rows = [json.loads(l) for l in open(Path(a.docs_root) / a.build / "truth.jsonl")]
    if a.limit:
        rows = rows[: a.limit]
    root = Path(a.audio_root) / a.build
    t0 = time.time()
    res = []
    with ProcessPoolExecutor(a.jobs) as ex:
        for i, r in enumerate(ex.map(job, [(r, root, systems) for r in rows], chunksize=2)):
            res += r
            if i % 25 == 0:
                print(f"{i + 1}/{len(rows)} 段 · {time.time() - t0:.0f} s", flush=True)
    out = Path(a.results)
    out.mkdir(parents=True, exist_ok=True)
    tag = f"{a.build}-scores" + (f"-{a.tag}" if a.tag else "") + (f"-limit{a.limit}" if a.limit else "")
    with open(out / f"{tag}.jsonl", "w") as f:
        for r in res:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    weights = {"gtcrn": RES / "gtcrn/stream/onnx_models/gtcrn_simple.onnx",
               "gtcrn_vctk": RES / "biglinux-gtcrn/stream/onnx_models/gtcrn_vctk.onnx",
               "ulunas": RES / "ul-unas/ulunas_onnx/onnx_models/ulunas_stream_simple.onnx",
               "dpdf_base": RES / "weights/dpdfnet/baseline.onnx", "dpdf2": RES / "weights/dpdfnet/dpdfnet2.onnx",
               "dpdf8": RES / "weights/dpdfnet/dpdfnet8.onnx",
               "fe_s": Path.home() / "vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_s.onnx",
               "fe_m": Path.home() / "vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx"}
    meta = {"build": a.build, "systems": {s: {"family": SYSTEMS[s][0], "desc": SYSTEMS[s][1],
                                              "weight_sha256": sha_file(weights[s]) if s in weights else None,
                                              "enh_record": f"builds/{a.build}/enh/{s}.json" if s in FILE_SYSTEMS else None} for s in systems},
            "rank_py_sha256": sha_file(__file__), "rows": len(rows), "seconds": round(time.time() - t0, 1),
            "errors": sum(1 for r in res if r["error"])}
    (out / f"{tag}-meta.json").write_text(json.dumps(meta, ensure_ascii=False, indent=1))
    print(json.dumps({k: meta[k] for k in ("rows", "seconds", "errors")}))


if __name__ == "__main__":
    main()
