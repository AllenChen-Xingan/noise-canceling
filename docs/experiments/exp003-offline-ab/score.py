"""exp003 第三步：客观旁证打分。只作旁证，结论以人耳 AB 为准（.42cog/intent.md 真相源权重）。

每段 × 每版本（noisy / dpdfnet2 / gtcrn）：
- DNSMOS P.835（SIG/BAK/OVRL）：用响度统一后的、人实际听到的那份文件（audio/）；直接调 resources/DNS-Challenge 的 ComputeScore（只调用，不复制）
- ESTOI、SI-SDR：用对齐后、未归一的 raw/ 文件，对干净参考
- 非语音段：按干净参考做能量 VAD（20 ms 帧，高于最响帧 −35 dB 算有声，前后各扩 100 ms），其余算非语音
  · noise_att_db = 输出非语音段功率 ÷ 带噪输入非语音段功率（dB，越负压得越多）
  · resid_rel_db = 输出非语音段功率 ÷ 输出有声段功率（dB，与音量无关，越负越干净）
- 吃字（可选，--asr）：本机 42model 服务上的 Qwen3-ASR-Plus（人定 D28，默认 1.7B，OpenAI 兼容接口）
  分别转写干净参考与各版本（各版本用人实际听到的 audio/ 文件），以「干净参考的转写」为参照做编辑距离对齐；
  英文按词、中文按字；删除率（吃字率）= 删除 ÷ 参照长度，替换率同理，插入 ÷ 参照长度另记为「脑补率」
  · 每个文件转写两次，记录两次是否逐字相同（asr_stable）；不同时两次都存，打分用第一次
输出：docs/experiments/exp003-offline-ab/scores.json
用法：<主目录>/_tmp/exp002-venv/bin/python docs/experiments/exp003-offline-ab/score.py [--asr] [--asr-model qwen3-asr-plus:0.6b]
      （--whisper 是旧名，等同 --asr）
"""
import json
import re
import sys
from pathlib import Path

import numpy as np
import soundfile as sf
from pystoi import stoi

import requests

from _paths import RES, ROOT  # resources/ 只在主目录

HERE = Path(__file__).resolve().parent
OUT = ROOT / "_tmp/20261030-exp003"
DNSMOS_DIR = RES / "DNS-Challenge/DNSMOS"
sys.path.insert(0, str(DNSMOS_DIR))
from dnsmos_local import ComputeScore  # noqa: E402

SR = 16000
VERS = ["noisy", "dpdfnet2", "gtcrn"]
ASR_URL = "http://127.0.0.1:11520/v1/audio/transcriptions"
ASR_KEY = "42model"  # 本机服务的固定占位 key，不是密钥
ASR_MODEL = "qwen3-asr-plus:1.7b"
ASR_LANG = {"en": "en", "zh": "zh"}


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


def asr_once(path, lang, model):
    with open(path, "rb") as f:
        r = requests.post(ASR_URL, headers={"Authorization": f"Bearer {ASR_KEY}"},
                          data={"model": model, "language": ASR_LANG[lang], "temperature": "0",
                                "response_format": "json"},
                          files={"file": (Path(path).name, f, "audio/wav")}, timeout=600)
    r.raise_for_status()
    return " ".join(r.json()["text"].split())


def asr_text(path, lang, model):
    """转写两次；返回 (第一次文本, 两次是否逐字相同, 第二次文本)。"""
    a, b = asr_once(path, lang, model), asr_once(path, lang, model)
    return a, a == b, b


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
    use_asr = "--asr" in sys.argv or "--whisper" in sys.argv
    model = sys.argv[sys.argv.index("--asr-model") + 1] if "--asr-model" in sys.argv else ASR_MODEL
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
        if use_asr:
            ref_txt, stable, second = asr_text(OUT / "src" / f"{cid}_clean.wav", c["lang"], model)
            r["asr_ref"], r["asr_ref_stable"] = ref_txt, stable
            if not stable:
                r["asr_ref_second"] = second
            if c.get("target", {}).get("text"):
                r["truth"] = c["target"]["text"]  # LibriSpeech 人工转写，只作参照质量的旁证
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
            if use_asr:
                hyp, stable, second = asr_text(OUT / files[cid]["versions"][v]["file"], c["lang"], model)
                D, S, I = edit_ops(ref_tok, tokens(hyp, c["lang"]))
                L = max(1, len(ref_tok))
                row.update({"asr": hyp, "asr_stable": stable, "ref_len": len(ref_tok), "del": D, "sub": S, "ins": I,
                            "del_rate": round(D / L, 4), "sub_rate": round(S / L, 4), "ins_rate": round(I / L, 4)})
                if not stable:
                    row["asr_second"] = second
            r[v] = row
        res[cid] = r
        print(cid, {v: (r[v]["OVRL"], r[v]["ESTOI"], r[v]["SI_SDR_dB"], r[v].get("del_rate"), r[v].get("ins_rate")) for v in VERS}, flush=True)
    keys = ["SIG", "BAK", "OVRL", "ESTOI", "SI_SDR_dB", "noise_att_db", "resid_rel_db"] + (["del_rate", "sub_rate", "ins_rate"] if use_asr else [])
    summary = {v: {k: round(float(np.mean([res[c][v][k] for c in res])), 3) for k in keys} for v in VERS}
    higher_better = {"SIG", "BAK", "OVRL", "ESTOI", "SI_SDR_dB"}
    wins = {}
    for k in keys:
        dp = sum((res[c]["dpdfnet2"][k] > res[c]["gtcrn"][k]) == (k in higher_better)
                 for c in res if res[c]["dpdfnet2"][k] != res[c]["gtcrn"][k])
        ties = sum(res[c]["dpdfnet2"][k] == res[c]["gtcrn"][k] for c in res)
        wins[k] = {"dpdfnet2": int(dp), "gtcrn": len(res) - int(dp) - ties, "tie": ties}
    out = {"per_clip": res, "mean": summary, "dpdfnet2_vs_gtcrn_wins": wins, "asr": use_asr}
    if use_asr:
        n_files = [res[c].get("asr_ref_stable") for c in res] + [res[c][v]["asr_stable"] for c in res for v in VERS]
        out["asr_model"] = {"service": ASR_URL, "model": model, "params": "temperature=0, language=<en|zh>",
                            "repeat": 2, "stable_files": f"{sum(n_files)}/{len(n_files)}"}
    (HERE / "scores.json").write_text(json.dumps(out, ensure_ascii=False, indent=1))
    print(json.dumps({"mean": summary, "wins": wins}, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
