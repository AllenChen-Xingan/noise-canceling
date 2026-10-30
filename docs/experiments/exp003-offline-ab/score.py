"""exp003 第三步：客观旁证打分。只作旁证，结论以人耳 AB 为准（.42cog/intent.md 真相源权重）。

每段 × 每版本（noisy / dpdfnet2 / gtcrn）：
- DNSMOS P.835（SIG/BAK/OVRL）：用响度统一后的、人实际听到的那份文件（audio/）；直接调 resources/DNS-Challenge 的 ComputeScore（只调用，不复制）
- ESTOI、SI-SDR：用对齐后、未归一的 raw/ 文件，对干净参考
- 非语音段：按干净参考做能量 VAD（20 ms 帧，高于最响帧 −35 dB 算有声，前后各扩 100 ms），其余算非语音
  · noise_att_db = 输出非语音段功率 ÷ 带噪输入非语音段功率（dB，越负压得越多）
  · resid_rel_db = 输出非语音段功率 ÷ 输出有声段功率（dB，与音量无关，越负越干净）
- 吃字（可选，--whisper）：whisper.cpp large-v3-turbo 分别转写干净参考与各版本，以「干净参考的转写」为参照
  做编辑距离对齐，删除 / 替换 / 插入分开记；英文按词、中文按字；删除率 = 删除 ÷ 参照长度
输出：docs/experiments/exp003-offline-ab/scores.json
用法：_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/score.py [--whisper]
"""
import json
import re
import subprocess
import sys
from pathlib import Path

import numpy as np
import soundfile as sf
from pystoi import stoi

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
OUT = ROOT / "_tmp/20261030-exp003"
DNSMOS_DIR = ROOT / "resources/DNS-Challenge/DNSMOS"
sys.path.insert(0, str(DNSMOS_DIR))
from dnsmos_local import ComputeScore  # noqa: E402

SR = 16000
VERS = ["noisy", "dpdfnet2", "gtcrn"]
WHISPER_BIN = "/opt/homebrew/bin/whisper-cli"
WHISPER_MODEL = Path.home() / "vdn-data/models/ggml-large-v3-turbo.bin"


def si_sdr(est, ref):
    n = min(len(est), len(ref))
    est, ref = est[:n] - est[:n].mean(), ref[:n] - ref[:n].mean()
    target = (np.dot(est, ref) / np.dot(ref, ref)) * ref
    return float(10 * np.log10(np.sum(target ** 2) / np.sum((est - target) ** 2)))


def vad_mask(ref, frame=320, thr_db=-35.0, dilate=5):
    nf = len(ref) // frame
    e = 10 * np.log10(np.mean(ref[: nf * frame].reshape(nf, frame) ** 2, axis=1) + 1e-12)
    m = e > e.max() + thr_db
    d = m.copy()
    for k in range(1, dilate + 1):
        d[k:] |= m[:-k]
        d[:-k] |= m[k:]
    return np.repeat(d, frame), nf * frame


def frame_power(x, mask, n):
    x = x[:n]
    return float(np.mean(x[mask] ** 2) + 1e-12), float(np.mean(x[~mask] ** 2) + 1e-12)


def whisper_text(path, lang):
    cmd = [WHISPER_BIN, "-m", str(WHISPER_MODEL), "-f", str(path), "-l", lang, "-nt", "-np",
           "-tp", "0", "-nf", "-mc", "0"]
    r = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return " ".join(r.stdout.split())


def tokens(text, lang):
    if lang == "zh":
        return [c for c in text if re.match(r"[一-鿿0-9a-zA-Z]", c)]
    return re.findall(r"[a-z0-9']+", text.lower())


def edit_ops(ref, hyp):
    """Levenshtein 对齐，返回 (删除, 替换, 插入)。"""
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
            S += ref[i - 1] != hyp[j - 1]
            i, j = i - 1, j - 1
        elif i > 0 and d[i, j] == d[i - 1, j] + 1:
            D, i = D + 1, i - 1
        else:
            I, j = I + 1, j - 1
    return D, S, I


def main():
    use_whisper = "--whisper" in sys.argv
    clips = json.loads((HERE / "clips.json").read_text())["clips"]
    files = json.loads((OUT / "files.json").read_text())
    scorer = ComputeScore(str(DNSMOS_DIR / "DNSMOS/sig_bak_ovr.onnx"), str(DNSMOS_DIR / "DNSMOS/model_v8.onnx"))
    res = {}
    for c in clips:
        cid = c["id"]
        ref, _ = sf.read(OUT / "src" / f"{cid}_clean.wav", dtype="float32")
        mask, n = vad_mask(ref)
        raw = {v: sf.read(OUT / "raw" / f"{cid}_{v}.wav", dtype="float32")[0] for v in VERS}
        _, noisy_ns = frame_power(raw["noisy"], mask, n)
        r = {"nonspeech_s": round(float((~mask).sum()) / SR, 2)}
        if use_whisper:
            ref_txt = whisper_text(OUT / "src" / f"{cid}_clean.wav", c["lang"])
            r["whisper_ref"] = ref_txt
            ref_tok = tokens(ref_txt, c["lang"])
        for v in VERS:
            s = scorer(str(OUT / files[cid]["versions"][v]["file"]), SR, False)
            sp, ns = frame_power(raw[v], mask, n)
            row = {
                "SIG": round(float(s["SIG"]), 3), "BAK": round(float(s["BAK"]), 3), "OVRL": round(float(s["OVRL"]), 3),
                "ESTOI": round(float(stoi(ref, raw[v][: len(ref)], SR, extended=True)), 4),
                "SI_SDR_dB": round(si_sdr(raw[v], ref), 2),
                "noise_att_db": round(10 * np.log10(ns / noisy_ns), 2),
                "resid_rel_db": round(10 * np.log10(ns / sp), 2),
            }
            if use_whisper:
                hyp = whisper_text(OUT / files[cid]["versions"][v]["file"], c["lang"])
                D, S, I = edit_ops(ref_tok, tokens(hyp, c["lang"]))
                row.update({"whisper": hyp, "ref_len": len(ref_tok), "del": D, "sub": S, "ins": I,
                            "del_rate": round(D / max(1, len(ref_tok)), 4)})
            r[v] = row
        res[cid] = r
        print(cid, {v: (r[v]["OVRL"], r[v]["ESTOI"], r[v]["SI_SDR_dB"], r[v].get("del_rate")) for v in VERS}, flush=True)
    keys = ["SIG", "BAK", "OVRL", "ESTOI", "SI_SDR_dB", "noise_att_db", "resid_rel_db"] + (["del_rate"] if use_whisper else [])
    summary = {v: {k: round(float(np.mean([res[c][v][k] for c in res])), 3) for k in keys} for v in VERS}
    higher_better = {"SIG", "BAK", "OVRL", "ESTOI", "SI_SDR_dB"}
    wins = {}
    for k in keys:
        dp = sum((res[c]["dpdfnet2"][k] > res[c]["gtcrn"][k]) == (k in higher_better)
                 for c in res if res[c]["dpdfnet2"][k] != res[c]["gtcrn"][k])
        ties = sum(res[c]["dpdfnet2"][k] == res[c]["gtcrn"][k] for c in res)
        wins[k] = {"dpdfnet2": int(dp), "gtcrn": len(res) - int(dp) - ties, "tie": ties}
    out = {"per_clip": res, "mean": summary, "dpdfnet2_vs_gtcrn_wins": wins, "whisper": use_whisper}
    if use_whisper:
        out["whisper_model"] = {"file": "~/vdn-data/models/ggml-large-v3-turbo.bin",
                                "args": "-nt -np -tp 0 -nf -mc 0 -l <en|zh>"}
    (HERE / "scores.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps({"mean": summary, "wins": wins}, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
