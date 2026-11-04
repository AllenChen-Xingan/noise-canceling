"""exp013 · 场景测试集主入口：按 build id 批量出音频与清单。

- 音频 → 主目录 `_tmp/exp013/<build-id>/<片段 id>/*.wav`（float32，scipy 写，不带时间戳块，文件哈希可复现）
- 清单 → 本目录 `builds/<build-id>/`：clips.jsonl · manifest.json · hashes.txt · exclude.json · 参数快照（进仓）
- 已存在的 build 目录一律拒绝覆盖（CLAUDE.md 第五节 1 条）
用法（worktree 根下）：
  PY=<主目录>/_tmp/exp002-venv/bin/python
  $PY docs/experiments/exp013-scene-suite/build.py [--build <id>] [--only cafe,canteen] [--n 2] [--workers 4]
"""
import argparse
import hashlib
import json
import secrets
import subprocess
import sys
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
from scipy.io import wavfile

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import compose  # noqa: E402
import labels  # noqa: E402
import leak  # noqa: E402
import manifest  # noqa: E402
import pools  # noqa: E402
from dsp import SR, sample_hash  # noqa: E402

ROOT = HERE.parents[2]
CODE = ["build.py", "compose.py", "tracks.py", "labels.py", "manifest.py", "leak.py", "pools.py", "rooms.py",
        "synth_sounds.py", "device_chain.py", "dsp.py"]
PARAMS = ["scenes.json", "sounds.json", "device_chain.json", "materials.json"]


def main_root():
    out = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                         capture_output=True, text=True, check=True).stdout.strip()
    return Path(out).parent


def sha_file(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def write_wav(path, x):
    path.parent.mkdir(parents=True, exist_ok=True)
    wavfile.write(str(path), SR, np.asarray(x, "<f4"))
    return sha_file(path)


def run_job(job):
    scene, tier, idx, device, build, out_rel, main = job
    arrays, meta = compose.make_clip(scene, tier, idx, device)
    lab = labels.label(arrays, meta)
    cid = manifest.clip_id(scene, tier, idx, device)
    rel = Path(out_rel) / cid
    audio, wsha, ssha = {}, {}, {}
    for k in manifest.AUDIO_KEYS:
        if k == "stems":
            audio["stems"], wsha["stems"], ssha["stems"] = {}, {}, {}
            for inst, x in arrays["stems"].items():
                p = rel / f"stem-{inst}.wav"
                wsha["stems"][inst] = write_wav(Path(main) / p, x)
                ssha["stems"][inst] = sample_hash(np.asarray(x, "<f4"))
                audio["stems"][inst] = str(p)
            continue
        p = rel / f"{k}.wav"
        wsha[k] = write_wav(Path(main) / p, arrays[k])
        ssha[k] = sample_hash(np.asarray(arrays[k], "<f4"))
        audio[k] = str(p)
    other = "external-mic" if device == "mba-builtin" else "mba-builtin"
    rec = manifest.record(meta, lab, audio, {"samples": ssha, "wav_sha256": wsha}, len(arrays["mix"]), SR, other)
    errs = manifest.validate(rec)
    if errs:
        raise ValueError(f"{cid} 清单字段不合格：{errs[:5]}")
    return rec


def jobs_for(only, n_override, devices, build, out_rel, main):
    S = compose.SCENES
    out = []
    for scene in S["scenes"]:
        if only and scene not in only:
            continue
        for tier, cnt in S["counts"].items():
            for idx in range(n_override if n_override else cnt):
                for dev in devices:
                    out.append((scene, tier, idx, dev, build, out_rel, str(main)))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--build", default=time.strftime("%Y%m%d%H%M%S") + "-" + secrets.token_hex(2))
    ap.add_argument("--only", default="")
    ap.add_argument("--n", type=int, default=0, help="每档只出前 N 段（冒烟用）")
    ap.add_argument("--devices", default=",".join(compose.SCENES["devices"]))
    ap.add_argument("--workers", type=int, default=4)
    ap.add_argument("--builds-dir", default=str(HERE / "builds"))
    a = ap.parse_args()
    main_dir = main_root()
    out_rel = f"_tmp/exp013/{a.build}"
    bdir = Path(a.builds_dir) / a.build
    if bdir.exists() or (main_dir / out_rel).exists():
        sys.exit(f"build {a.build} 已存在，不覆盖；换一个 --build")
    only = set(filter(None, a.only.split(",")))
    jobs = jobs_for(only, a.n, a.devices.split(","), a.build, out_rel, main_dir)
    t0 = time.time()
    with ProcessPoolExecutor(a.workers) as ex:
        recs = list(ex.map(run_job, jobs, chunksize=4))
    recs.sort(key=lambda r: r["id"])
    bdir.mkdir(parents=True)
    (bdir / "clips.jsonl").write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in recs))
    hashes = []
    for r in recs:
        for k, v in r["hash"]["wav_sha256"].items():
            for kk, vv in (v.items() if isinstance(v, dict) else [(None, v)]):
                path = r["audio"][k][kk] if kk else r["audio"][k]
                hashes.append(f"{vv}  {path}")
    (bdir / "hashes.txt").write_text("\n".join(sorted(hashes)) + "\n")
    lk = leak.check(recs)
    (bdir / "exclude.json").write_text(json.dumps(leak.exclude_table(recs), ensure_ascii=False, indent=1) + "\n")
    snap = bdir / "params"
    snap.mkdir()
    for f in PARAMS:
        (snap / f).write_bytes((HERE / f).read_bytes())
    idx_hash, idx_n = pools.index_hash()
    counts = {}
    for r in recs:
        counts.setdefault(r["scene"], {}).setdefault(f"{r['tier']}/{r['device']}", 0)
        counts[r["scene"]][f"{r['tier']}/{r['device']}"] += 1
    lic = sorted({s["license"] for r in recs for s in r["sounds"]})
    man = {
        "build_id": a.build, "created": time.strftime("%Y-%m-%d %H:%M:%S"), "seconds": round(time.time() - t0, 1),
        "clips": len(recs), "audio_dir": out_rel, "counts": counts, "base_seed": compose.SCENES["base_seed"],
        "code_sha256": {f: sha_file(HERE / f) for f in CODE}, "params_sha256": {f: sha_file(HERE / f) for f in PARAMS},
        "material_index": {"sha256": idx_hash, "files": idx_n}, "leak_check": lk, "licenses": lic,
        "validity": "D0（016：只保证可复现、可控、真值完整；未做统计对齐与双生成器）",
        "use": "只评测，不训练（含 CC-BY-NC 素材）",
    }
    (bdir / "manifest.json").write_text(json.dumps(man, ensure_ascii=False, indent=1) + "\n")
    print(f"build {a.build}：{len(recs)} 段，{man['seconds']} s；泄漏检查 {'通过' if lk['ok'] else '不过：' + str(lk['hits'][:3])}")
    print(f"清单 {bdir.relative_to(ROOT)}；音频 主目录/{out_rel}")
    if not lk["ok"]:
        sys.exit(2)


if __name__ == "__main__":
    main()
