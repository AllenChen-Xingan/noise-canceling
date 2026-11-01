"""exp008 第二轮 · ClearerVoice 三个模型（frcrn / mossformer_gan / mossformer2_48k）的共用跑法。

只调用 <MAIN>/resources/ClearerVoice-Studio/clearvoice 里的模型类与 decode 函数（resources/ 只读，不复制代码）；
绕开 ClearVoice 包装层的自动下载：权重预先放在 ~/vdn-data/models/<HF 名>/，这里自己读 state_dict。
输入归一与 ClearVoice 文件模式一致：FRCRN 先 audio_norm（-25 dB RMS 两段），输出乘回 scalar；
MossFormerGAN 内部按整段 RMS 归一再除回；MossFormer2_48K 不归一（×32768 进 fbank，掩码乘在原谱上）。
三者都不截断幅度（输出写 float32，不 clip）。
"""
import argparse
import hashlib
import json
import os
import platform
import sys
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

sys.dont_write_bytecode = True  # 不在 resources/ 里留 __pycache__

import numpy as np
import soundfile as sf
from scipy.signal import resample_poly

MAIN = Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling")
CV_ROOT = MAIN / "resources/ClearerVoice-Studio/clearvoice"  # 其下的 clearvoice/ 是包
HERE = Path(__file__).resolve().parent
EXP = HERE.parent
MODELS = Path.home() / "vdn-data/models"

SPECS = {
    "frcrn": dict(hf="FRCRN_SE_16K", sr=16000, family="F", use_norm=True),
    "mossformer_gan": dict(hf="MossFormerGAN_SE_16K", sr=16000, family="F", use_norm=False),
    "mossformer2_48k": dict(hf="MossFormer2_SE_48K", sr=48000, family="F", use_norm=False),
}

_M = {}  # 每个工作进程一份：sid, args, model, decode


def _sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def _args(hf):
    import yaml
    cfg = yaml.safe_load(open(CV_ROOT / "clearvoice/config/inference" / f"{hf}.yaml"))
    cfg.update(mode="inference", use_cuda=0, num_gpu=0, checkpoint_dir=str(MODELS / hf))
    return argparse.Namespace(**cfg)


def _load_state(model, ckpt):
    import torch
    sd = torch.load(ckpt, map_location="cpu", weights_only=False)
    sd = sd.get("model", sd) if isinstance(sd, dict) else sd
    sd = {k.removeprefix("module."): v for k, v in sd.items()}
    own = model.state_dict()
    miss = [k for k in own if k not in sd or sd[k].shape != own[k].shape]
    if miss:
        raise RuntimeError(f"{len(miss)} 个参数没对上，例如 {miss[:5]}")
    model.load_state_dict({k: sd[k] for k in own})
    return len(own)


def _init(sid):
    import torch
    torch.set_num_threads(1)
    if str(CV_ROOT) not in sys.path:
        sys.path.insert(0, str(CV_ROOT))
    spec = SPECS[sid]
    args = _args(spec["hf"])
    if sid == "frcrn":
        from clearvoice.models.frcrn_se.frcrn import FRCRN_SE_16K as Net
    elif sid == "mossformer_gan":
        from clearvoice.models.mossformer_gan_se.generator import MossFormerGAN_SE_16K as Net
    else:
        from clearvoice.models.mossformer2_se.mossformer2_se_wrapper import MossFormer2_SE_48K as Net
    from clearvoice.utils.decode import decode_one_audio
    from clearvoice.dataloader.dataloader import audio_norm
    model = Net(args).model
    ckpt = MODELS / spec["hf"] / open(MODELS / spec["hf"] / "last_best_checkpoint").readline().strip()
    n = _load_state(model, ckpt)
    model.eval()
    _M.update(sid=sid, args=args, model=model, decode=decode_one_audio, norm=audio_norm, n_params=n)


def enhance(x16):
    """x16: 16 kHz float 一维。返回与之等长的 16 kHz float32。"""
    import torch
    sid, args = _M["sid"], _M["args"]
    sr = SPECS[sid]["sr"]
    x = x16.astype(np.float64)
    if sr != 16000:
        x = resample_poly(x, sr // 16000, 1)
    scalar = 1.0
    if SPECS[sid]["use_norm"]:
        x, scalar = _M["norm"](x)
    torch.manual_seed(0)  # 48K 的 kaldi fbank 带 dither=1.0，固定种子以便复现
    with torch.no_grad():
        y = _M["decode"](_M["model"], torch.device("cpu"), x[None, :].astype(np.float32), args)
    y = np.asarray(y, dtype=np.float64).reshape(-1)
    if len(y) < len(x):
        raise RuntimeError(f"模型输出 {len(y)} 短于输入 {len(x)}")
    y = y[: len(x)] * scalar
    if sr != 16000:
        y = resample_poly(y, 1, sr // 16000)[: len(x16)]
    if len(y) != len(x16) or not np.all(np.isfinite(y)):
        raise RuntimeError(f"输出长度 {len(y)} / 非有限值")
    return y.astype(np.float32)


def _job(t):
    sid, src, dst = t
    if not _M:
        _init(sid)
    t0 = time.time()
    x, fs = sf.read(src, dtype="float32")
    assert fs == 16000 and x.ndim == 1, (fs, x.shape)
    y = enhance(x)
    dst.parent.mkdir(parents=True, exist_ok=True)
    tmp = dst.with_suffix(".part.wav")
    sf.write(tmp, y, 16000, subtype="FLOAT")
    os.replace(tmp, dst)
    return str(dst), time.time() - t0, float(np.max(np.abs(y))), float(np.max(np.abs(x)))


def _lag(y, ref, maxlag=2000):
    n = len(y) + len(ref)
    nf = 1 << (n - 1).bit_length()
    c = np.fft.irfft(np.fft.rfft(y, nf) * np.conj(np.fft.rfft(ref, nf)), nf)
    c = np.concatenate([c[-maxlag:], c[: maxlag + 1]])
    return int(np.argmax(np.abs(c)) - maxlag), float(np.max(np.abs(c)) / (np.linalg.norm(y) * np.linalg.norm(ref) + 1e-12))


def main(sid):
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--cells", default="", help="逗号分隔，只跑这些轴（例如 S-SNR,S-NOISE,S-SIR）")
    a = ap.parse_args()
    spec = SPECS[sid]
    rows = [json.loads(l) for l in open(EXP / "builds" / a.build / "truth.jsonl")]
    if a.cells:
        keep = set(a.cells.split(","))
        rows = [r for r in rows if r["cell"] in keep]
    if a.limit:
        rows = rows[: a.limit]
    src_root = MAIN / "_tmp/exp008" / a.build
    dst_root = MAIN / "_tmp/exp008" / f"{a.build}-enh" / sid
    todo = [(sid, src_root / r["files"]["mix"]["path"], dst_root / r["files"]["mix"]["path"]) for r in rows]
    todo = [t for t in todo if not t[2].exists()]
    print(f"{sid}: {len(rows)} 段，待跑 {len(todo)}", flush=True)
    t0 = time.time()
    times, peaks, fails = [], [], []
    with ProcessPoolExecutor(a.jobs, initializer=_init, initargs=(sid,)) as ex:
        futs = [(t, ex.submit(_job, t)) for t in todo]
        for i, (t, f) in enumerate(futs):
            try:
                _, dt, py, px = f.result()
                times.append(dt)
                peaks.append((py, px))
            except Exception as e:  # 单段失败不停摆，记下来
                fails.append({"path": str(t[1].relative_to(src_root)), "error": repr(e)[:300]})
            if i % 10 == 0 or i == len(futs) - 1:
                print(f"  {i + 1}/{len(futs)} · {time.time() - t0:.0f} s · 均 {np.mean(times) if times else 0:.2f} s/段", flush=True)
    seconds = time.time() - t0

    # 延迟核对：前 6 段（全表顺序），输出 vs owner 互相关
    all_rows = [json.loads(l) for l in open(EXP / "builds" / a.build / "truth.jsonl")]
    checks = []
    for r in all_rows[:6]:
        p = dst_root / r["files"]["mix"]["path"]
        if not p.exists():
            continue
        y = sf.read(p, dtype="float64")[0]
        o = sf.read(src_root / r["files"]["owner"]["path"], dtype="float64")[0]
        x = sf.read(src_root / r["files"]["mix"]["path"], dtype="float64")[0]
        lag, rho = _lag(y, o)
        checks.append({"path": r["files"]["mix"]["path"], "lag": lag, "peak_ncc": round(rho, 4),
                       "len_out": len(y), "len_in": len(x)})
    ok = bool(checks) and all(abs(c["lag"]) <= 1 and c["len_out"] == c["len_in"] for c in checks)

    rec_path = EXP / "builds" / a.build / "enh" / f"{sid}.json"
    old = json.loads(rec_path.read_text()) if rec_path.exists() else {}
    n_written = sum((dst_root / r["files"]["mix"]["path"]).exists() for r in all_rows)
    import torch
    import torchaudio
    ck_name = open(MODELS / spec["hf"] / "last_best_checkpoint").readline().strip()
    ck = MODELS / spec["hf"] / ck_name
    runs = old.get("runs", []) + [{"cells": a.cells or "all", "limit": a.limit, "jobs": a.jobs, "n_run": len(todo),
                                   "n_fail": len(fails), "seconds": round(seconds, 1),
                                   "sec_per_seg_mean": round(float(np.mean(times)), 3) if times else None,
                                   "max_abs_out": round(max((p[0] for p in peaks), default=0), 4),
                                   "max_abs_in": round(max((p[1] for p in peaks), default=0), 4),
                                   "fails": fails[:20]}]
    rec = {
        "id": sid, "family": spec["family"], "causal": False, "sr_model": spec["sr"],
        "weights": [{"path": str(ck).replace(str(Path.home()), "~"), "sha256": _sha(ck), "bytes": ck.stat().st_size,
                     "license": "Apache-2.0",
                     "license_source": f"https://huggingface.co/alibabasglab/{spec['hf']}（模型卡 license: apache-2.0）"}],
        "code_source": f"resources/ClearerVoice-Studio/clearvoice/clearvoice（Apache-2.0，见 resources/ClearerVoice-Studio/LICENSE）；"
                       f"本仓 enh/clearvoice_common.py 只 import 其模型类与 utils.decode.decode_one_audio",
        "delay_comp_samples": 0,
        "delay_check": {"n": len(checks), "ok": ok, "rule": "输出 vs owner 互相关 ±2000 样本峰值滞后 |lag|≤1", "items": checks},
        "runtime": f"python {platform.python_version()} / torch {torch.__version__} / torchaudio {torchaudio.__version__} / "
                   f"CPU, torch.set_num_threads(1) per process",
        "n_written": n_written, "n_total": len(all_rows),
        "seconds": round(sum(r["seconds"] for r in runs), 1),
        "runs": runs,
        "notes": NOTES[sid],
    }
    rec_path.parent.mkdir(parents=True, exist_ok=True)
    rec_path.write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n")
    print(json.dumps({k: rec[k] for k in ("id", "n_written", "seconds")}, ensure_ascii=False), "delay ok" if ok else "DELAY CHECK FAILED", checks)


NOTES = {
    "frcrn": "非因果，整段离线一次过（8 s < one_time_decode_length 120 s，不分块）。按 ClearVoice 文件模式先 audio_norm 到 -25 dB RMS，"
             "输出乘回 1/scalar；decode 内部补零到窗长倍数，取前 len(x) 样本。不 clip。每个工作进程载一次权重，"
             "模型无跨调用状态，等价于每段从零开始。",
    "mossformer_gan": "非因果，整段一次过（8 s < 10 s，不分块）。内部按整段 RMS 归一、输出再除回（幅度与输入同量级）；"
                      "尾部用开头样本补齐到 hop 倍数再截回。只载生成器 last_best_checkpoint.pt，判别器没下载。不 clip。",
    "mossformer2_48k": "非因果，16 k→48 k resample_poly(3,1) 后整段一次过（8 s < 20 s，不分块），跑完 resample_poly(1,3) 回 16 k。"
                       "ClearVoice 对该模型不做输入归一：×32768 后算 kaldi fbank（dither=1.0，故每段 torch.manual_seed(0)），"
                       "掩码乘在原谱上，因此输出电平跟随输入。不 clip。",
}
