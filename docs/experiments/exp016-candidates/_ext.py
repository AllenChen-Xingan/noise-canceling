"""exp016 离线推理脚本共用：读 exp013 片段清单 → 造三种输入 → 交给候选处理 → 写外部输出目录。

外部输出目录的格式正本在 Rust 入口 `src/eval/src/system/external.rs`（说明 `external-entry.md`）：
- `run.json`（schema `vdn-external-output/1`）+ 每种输入一个子目录 `mix/` `bg/` `trunc/`，里面 `<片段 id>.wav`
- 三种输入的算法照 `Kind::input`：mix = mix.wav；bg = noise.wav + interf.wav 逐样本相加（float32）；
  trunc = 每个场景清单顺序第一段（has_clean_ref）的 mix，从第 floor(n/2) 样本起补零
- 输出：已对齐（去掉声明延迟）、16 kHz 单声道 32 位浮点、与输入等长
入口载入时会再全部自检一遍（片段数 · 采样率 · 时长 · NaN · 截半查偷看未来），这里只做最基本的检查。

候选脚本只需实现 `process(x16: np.ndarray) -> np.ndarray`（等长、已对齐）并给出声明延迟，然后调 `main()`。
音频只写主目录 `_tmp/exp016/<候选>/<run-id>/`；参数、哈希、清单写 `builds/<候选>-<run-id>/`（进仓）。
"""
import argparse
import datetime
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]  # 本工作副本根
SR = 16000
SCHEMA = "vdn-external-output/1"
# 场景测试集正式 build（共同规矩第 3 条：只认它与 src/eval/suite/scenes.json）
CLIPS = "docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/clips.jsonl"
KINDS = ("mix", "bg", "trunc")


def main_root():
    """放 _tmp/、resources/ 实物的主目录（worktree 里按 git 公共目录找回）。"""
    out = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--path-format=absolute", "--git-common-dir"],
                         capture_output=True, text=True, check=True).stdout.strip()
    return Path(out).parent


def scenes_clips_path():
    """片段清单路径取自 scenes.json（正本）；与本文件的 CLIPS 不一致就报错。"""
    suite = json.loads((ROOT / "src/eval/suite/scenes.json").read_text())
    paths = {t["real"]["clips"] for t in suite["tasks"]}
    if paths != {CLIPS}:
        raise SystemExit(f"scenes.json 引用的片段清单 {sorted(paths)} ≠ 本脚本的 {CLIPS}")
    return CLIPS


def sha256_file(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def read16(p):
    x, sr = sf.read(str(p), dtype="float32", always_2d=False)
    if sr != SR:
        raise ValueError(f"{p} 采样率 {sr} ≠ {SR}")
    return x[:, 0] if x.ndim == 2 else x


def load_clips(clips_rel):
    return [json.loads(l) for l in (ROOT / clips_rel).read_text().splitlines() if l.strip()]


def trunc_picks(clips):
    """每个场景清单顺序第一段有干净参考的（同 external.rs `trunc_picks`）。"""
    seen, out = set(), set()
    for c in clips:
        if c["has_clean_ref"] and c["scene"] not in seen:
            seen.add(c["scene"])
            out.add(c["id"])
    return out


def inputs_of(c, root, picks, verify_hash=True):
    """一段的各种输入：[(种类, 样本)]。顺序 mix · bg · trunc；不适用的种类不给。"""
    a, h = c["audio"], c["hash"]["wav_sha256"]
    def get(k):
        p = root / a[k]
        if verify_hash and sha256_file(p) != h[k]:
            raise ValueError(f"{c['id']} 的 {k} sha256 对不上清单")
        x = read16(p)
        if len(x) != c["dur_samples"]:
            raise ValueError(f"{c['id']} 的 {k} 长 {len(x)} ≠ dur_samples {c['dur_samples']}")
        return x
    mix = get("mix")
    out = [("mix", mix)]
    if c["additive"]:
        out.append(("bg", (get("noise") + get("interf")).astype(np.float32)))
    if c["id"] in picks:
        t = mix.copy()
        t[len(t) // 2:] = 0.0
        out.append(("trunc", t))
    return out


def jobs(clips, root, picks, shard=(0, 1)):
    """全部要跑的 (片段, 种类, 输入)；按片段分片（shard = (第几片, 共几片)）。"""
    k, n = shard
    for i, c in enumerate(c for c in clips if c["has_clean_ref"]):
        if i % n != k:
            continue
        for kind, x in inputs_of(c, root, picks):
            yield c, kind, x


def write_out(path, y, n):
    y = np.asarray(y, dtype=np.float32)
    if y.shape != (n,):
        raise ValueError(f"{path.name} 输出形状 {y.shape} ≠ ({n},)")
    if not np.all(np.isfinite(y)):
        raise ValueError(f"{path.name} 输出有非有限值")
    path.parent.mkdir(parents=True, exist_ok=True)
    sf.write(str(path), y, SR, subtype="FLOAT")


def run_id():
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    commit = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short=8", "HEAD"],
                            capture_output=True, text=True).stdout.strip() or "nogit"
    return f"{stamp}-{commit}"


def parse_args(name):
    ap = argparse.ArgumentParser(description=f"exp016：{name} 离线跑 exp013 场景测试集")
    ap.add_argument("--run-id", help="续跑 / 多进程分片时共用一个 run-id；默认新建")
    ap.add_argument("--shard", default="0/1", help="第几片/共几片（按片段轮流分），如 1/4")
    ap.add_argument("--limit", type=int, default=0, help="只跑前 N 段（冒烟用；结果不能给成绩单）")
    ap.add_argument("--finalize", action="store_true", help="所有分片跑完后：写 run.json 与 builds/ 清单")
    return ap.parse_args()


def main(name, process, latency_samples, model_info, setup=None):
    """候选脚本入口。`process(x16) -> y16`（等长、已对齐）；`latency_samples` 为 16 kHz 声明延迟。
    `setup()` 可选，返回额外参数写进 builds/。不覆盖已有输出文件（同 run-id 续跑时跳过已写的）。"""
    args = parse_args(name)
    root = main_root()
    clips_rel = scenes_clips_path()
    clips = load_clips(clips_rel)
    if args.limit:
        clips = clips[: args.limit]
    picks = trunc_picks(clips)
    rid = args.run_id or run_id()
    out = root / "_tmp/exp016" / name / rid
    builds = HERE / "builds" / f"{name}-{rid}"
    if args.finalize:
        return finalize(name, out, builds, clips_rel, clips, picks, latency_samples, model_info, setup, args)
    k, n = (int(v) for v in args.shard.split("/"))
    t0, done, skipped, audio_s = time.time(), 0, 0, 0.0
    for c, kind, x in jobs(clips, root, picks, (k, n)):
        p = out / kind / f"{c['id']}.wav"
        if p.exists():
            skipped += 1
            continue
        write_out(p, process(x), len(x))
        done += 1
        audio_s += len(x) / SR
        if done % 20 == 0:
            el = time.time() - t0
            print(f"[{name} {k}/{n}] {done} 个 · {audio_s:.0f} s 音频 · {el:.0f} s · {el / audio_s:.3f} 倍实时", flush=True)
    el = time.time() - t0
    stats = {"shard": f"{k}/{n}", "files": done, "skipped_existing": skipped, "audio_s": round(audio_s, 1),
             "wall_s": round(el, 1), "rtf": round(el / audio_s, 4) if audio_s else None}
    (out / "shards").mkdir(parents=True, exist_ok=True)
    (out / "shards" / f"{k}-of-{n}.json").write_text(json.dumps(stats, ensure_ascii=False) + "\n")
    print(f"[{name}] 分片 {k}/{n} 完成：{stats}")


def finalize(name, out, builds, clips_rel, clips, picks, latency_samples, model_info, setup, args):
    """写 run.json（入口读）与 builds/（进仓：参数 · 每个输出文件的 sha256 · 分片耗时）。"""
    if builds.exists():
        raise SystemExit(f"{builds} 已存在，不覆盖")
    want = {(kind, c["id"]) for c in clips if c["has_clean_ref"]
            for kind in KINDS if kind == "mix" or (kind == "bg" and c["additive"]) or (kind == "trunc" and c["id"] in picks)}
    have = {(p.parent.name, p.stem) for p in out.glob("*/*.wav")}
    if want != have:
        raise SystemExit(f"输出不全：缺 {len(want - have)} 个、多 {len(have - want)} 个")
    rj = {"schema": SCHEMA, "name": name, "clips": clips_rel, "sr": SR, "latency_samples": int(latency_samples),
          "inputs": list(KINDS), "model": model_info}
    rj_path = out / "run.json"
    if rj_path.exists():
        raise SystemExit(f"{rj_path} 已存在，不覆盖")
    rj_path.write_text(json.dumps(rj, ensure_ascii=False, indent=2) + "\n")
    builds.mkdir(parents=True)
    (builds / "run.json").write_text(rj_path.read_text())
    hashes = sorted(f"{sha256_file(p)}  {p.parent.name}/{p.name}" for p in out.glob("*/*.wav"))
    (builds / "hashes.txt").write_text("\n".join(hashes) + "\n")
    shards = [json.loads(p.read_text()) for p in sorted((out / "shards").glob("*.json"))]
    params = {"candidate": name, "limit": args.limit, "files": len(hashes), "shards": shards,
              "python": sys.version.split()[0], "platform": platform.platform(), "machine": platform.machine(),
              "cpu_count": os.cpu_count(), "extra": setup() if setup else None,
              "out_dir": f"_tmp/exp016/{name}/{out.name}"}
    (builds / "params.json").write_text(json.dumps(params, ensure_ascii=False, indent=2) + "\n")
    print(f"[{name}] 已写 {rj_path.relative_to(main_root())} 与 {builds.relative_to(ROOT)}（{len(hashes)} 个文件）")
