"""exp008 第二轮 · torch 离线增强系统的公共跑法（urgent_bsrnn / gmap_se 共用）。

约定见同目录 README.md：读 truth.jsonl 的 mix，写 <MAIN>/_tmp/exp008/<build>-enh/<id>/<同一相对路径>，
16 kHz 单声道 float32、与输入等长；已存在的跳过；跑完写 ../builds/<build>/enh/<id>.json。
模型是非因果、无跨段状态的整段离线模型：每个工作进程只载一次权重，每段独立前向（没有跨段状态可带）。
"""
import argparse
import hashlib
import json
import multiprocessing as mp
import os
import platform
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf

sys.dont_write_bytecode = True  # 别往 resources/ 里写 __pycache__

MAIN = Path("/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling")
EXP = Path(__file__).resolve().parent.parent
SR = 16000


def sha256(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def xcorr_lag(y, ref, maxlag=2000):
    n = len(y) + len(ref)
    nfft = 1 << (n - 1).bit_length()
    c = np.fft.irfft(np.fft.rfft(y, nfft) * np.conj(np.fft.rfft(ref, nfft)), nfft)
    c = np.concatenate([c[-maxlag:], c[: maxlag + 1]])
    return int(np.argmax(c) - maxlag)


_SYS = None  # 每个工作进程里的 (enhance_fn)


def _init(factory):
    global _SYS
    import torch
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    _SYS = factory()


def _work(job):
    src, dst = job
    x, sr = sf.read(src, dtype="float32")
    assert sr == SR and x.ndim == 1, (src, sr, x.shape)
    t0 = time.time()
    y = np.asarray(_SYS(x), dtype=np.float32).reshape(-1)
    dt = time.time() - t0
    assert len(y) == len(x), (src, len(y), len(x))
    assert np.all(np.isfinite(y)), src
    dst.parent.mkdir(parents=True, exist_ok=True)
    tmp = dst.with_suffix(".part.wav")
    sf.write(tmp, y, SR, subtype="FLOAT")
    os.replace(tmp, dst)
    return str(dst), dt


def run(sys_id, factory, record_base):
    ap = argparse.ArgumentParser()
    ap.add_argument("build")
    ap.add_argument("--jobs", type=int, default=1)
    ap.add_argument("--limit", type=int, default=None)
    ap.add_argument("--cells", default=None, help="逗号分隔，只跑这些轴（截断时用，记进 notes）")
    a = ap.parse_args()

    bdir = EXP / "builds" / a.build
    rows = [json.loads(l) for l in open(bdir / "truth.jsonl")]
    if a.cells:
        keep = set(a.cells.split(","))
        rows = [r for r in rows if r["cell"] in keep]
    if a.limit:
        rows = rows[: a.limit]
    inroot = MAIN / "_tmp" / "exp008" / a.build
    outroot = MAIN / "_tmp" / "exp008" / f"{a.build}-enh" / sys_id
    jobs = []
    for r in rows:
        rel = r["files"]["mix"]["path"]
        dst = outroot / rel
        if not dst.exists():
            jobs.append((str(inroot / rel), dst))
    print(f"[{sys_id}] {len(rows)} 段，待跑 {len(jobs)}", flush=True)

    t0 = time.time()
    per = []
    if jobs:
        ctx = mp.get_context("spawn")
        with ctx.Pool(a.jobs, initializer=_init, initargs=(factory,)) as pool:
            for i, (d, dt) in enumerate(pool.imap_unordered(_work, jobs), 1):
                per.append(dt)
                if i % 10 == 0 or i == len(jobs):
                    print(f"[{sys_id}] {i}/{len(jobs)}  本段 {dt:.1f}s  累计 {time.time()-t0:.0f}s", flush=True)
    wall = time.time() - t0

    # 延迟核对：前 6 段（全表前 6 段，不受 --cells 影响）
    allrows = [json.loads(l) for l in open(bdir / "truth.jsonl")][:6]
    lags = []
    for r in allrows:
        out = outroot / r["files"]["mix"]["path"]
        if not out.exists():
            continue
        y, _ = sf.read(out, dtype="float32")
        o, _ = sf.read(inroot / r["files"]["owner"]["path"], dtype="float32")
        lags.append({"clip": r["files"]["mix"]["path"], "lag": xcorr_lag(y, o)})
    ok = bool(lags) and all(abs(l["lag"]) <= 1 for l in lags)

    import torch
    n_written = sum(1 for r in [json.loads(l) for l in open(bdir / "truth.jsonl")]
                    if (outroot / r["files"]["mix"]["path"]).exists())
    recp = bdir / "enh" / f"{sys_id}.json"
    prev = json.loads(recp.read_text()) if recp.exists() else {}
    rec = dict(record_base)
    rec.update({
        "id": sys_id,
        "delay_comp_samples": 0,
        "delay_check": {"method": "xcorr vs _owner.wav, ±2000 samples", "lags": lags, "pass": ok},
        "runtime": f"python {platform.python_version()} / torch {torch.__version__} / CPU, 每进程 1 线程，{a.jobs} 进程",
        "n_written": n_written,
        "seconds": round(prev.get("seconds", 0) + wall, 1),
        "sec_per_clip_this_run": round(float(np.mean(per)), 2) if per else prev.get("sec_per_clip_this_run"),
    })
    recp.parent.mkdir(parents=True, exist_ok=True)
    recp.write_text(json.dumps(rec, ensure_ascii=False, indent=2) + "\n")
    print(f"[{sys_id}] 写出 {n_written} 段；延迟核对 {'通过' if ok else '未通过'} {[l['lag'] for l in lags]}；"
          f"本次 {wall:.0f}s，平均每段 {np.mean(per) if per else float('nan'):.2f}s；记录 {recp}", flush=True)
