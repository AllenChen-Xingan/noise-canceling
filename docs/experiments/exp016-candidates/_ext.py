"""exp016 离线推理脚本共用：读 exp013 片段清单 → 造三种输入 → 交给候选处理 → 写外部输出目录。

外部输出目录的格式正本在 Rust 入口 `src/eval/src/system/external.rs`（说明 `external-entry.md`）：
- `run.json`（schema `vdn-external-output/1`）+ 每种输入一个子目录 `mix/` `bg/` `trunc/` `clean/`，里面 `<片段 id>.wav`
- 四种输入的算法照 `Kind::input`：mix = mix.wav；bg = noise.wav + interf.wav 逐样本相加（float32）；
  trunc = 每个场景清单顺序第一段（has_clean_ref）的 mix，从第 floor(n/2) 样本起补零；
  clean = 干净人声（清单 `owner` 轨，同 `load_scene_clip`），每段都跑，成绩单新规则 v2 的三项用
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
KINDS = ("mix", "bg", "trunc", "clean")


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
    """一段的各种输入：[(种类, 样本)]。顺序 mix · bg · trunc · clean；不适用的种类不给。"""
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
    out.append(("clean", get("owner")))
    return out


def jobs(clips, root, picks, shard=(0, 1)):
    """全部要跑的 (片段, 种类, 输入)；按片段分片（shard = (第几片, 共几片)）。"""
    k, n = shard
    for i, c in enumerate(c for c in clips if c["has_clean_ref"]):
        if i % n != k:
            continue
        for kind, x in inputs_of(c, root, picks):
            yield c, kind, x


def measure_lookahead(process, x, tol=1e-4):
    """实测前视：同一段输入截半（第 m = n/2 样本起补零）再跑，找两次输出第一个差超过 tol 的样本 i，前视 = m − i。
    与入口截半自检同一思路；用于定声明延迟（取标称值与实测值的较大者），入口再在别的片段上复核。"""
    m = len(x) // 2
    t = x.copy()
    t[m:] = 0.0
    a, b = process(x), process(t)
    d = np.flatnonzero(np.abs(a[:m] - b[:m]) > tol)
    return int(m - d[0]) if len(d) else 0


def xcorr_lag(a, b, max_lag=4000):
    """b 相对 a 的滞后样本数（正 = b 落后），FFT 互相关，只在 ±max_lag 内找峰。"""
    n = len(a) + len(b)
    nf = 1 << (n - 1).bit_length()
    c = np.fft.irfft(np.fft.rfft(b, nf) * np.conj(np.fft.rfft(a, nf)), nf)
    lags = np.concatenate([np.arange(0, max_lag + 1), np.arange(-max_lag, 0)])
    vals = np.concatenate([c[: max_lag + 1], c[-max_lag:]])
    return int(lags[np.argmax(vals)])


def probe(process, clips, root, nominal):
    """在清单第一段的 mix 上实测前视与对齐（不是截半自检挑的那批之外的保证，只作定声明延迟的依据）。
    返回 (声明延迟, 记录)。对齐后滞后 ≠ 0 就报错：输出没对齐，入口会把延迟算错。"""
    ok = [c for c in clips if c["has_clean_ref"]]
    c, c2 = ok[0], ok[1 % len(ok)]
    x = inputs_of(c, root, set())[0][1]
    la = measure_lookahead(process, x)
    a1 = process(x)
    lag = xcorr_lag(x, a1)
    if abs(lag) > 1:
        raise SystemExit(f"对齐不对：{c['id']} 输出相对输入滞后 {lag} 样本")
    # 每段之间状态清干净：A → B → A 两次 A 的输出应一致（codex 016 脚本第 2 轮 P2 #5）
    process(inputs_of(c2, root, set())[0][1])
    reset_diff = float(np.max(np.abs(process(x) - a1)))
    if reset_diff > 1e-6:
        raise SystemExit(f"每段之间状态没清干净：A→B→A 两次 A 输出最大差 {reset_diff:.2e}")
    return max(nominal, la), {"probe_clip": c["id"], "nominal_latency": nominal, "measured_lookahead": la,
                              "lag_after_align": lag, "reset_check_clip": c2["id"], "reset_max_abs_diff": reset_diff}


def check_existing(path, n):
    """续跑跳过前校验已有输出：能读、16 kHz、等长、全有限；不对就报错（不静默重写，留给人看）。"""
    y = read16(path)
    if len(y) != n or not np.all(np.isfinite(y)):
        raise SystemExit(f"已有输出 {path.parent.name}/{path.name} 不完整或有非有限值，请人工处理")


def versions():
    """推理时实际用到的依赖版本（只记已载入的）。"""
    v = {"python": sys.version.split()[0], "numpy": np.__version__, "soundfile": sf.__version__,
         "libsndfile": sf.__libsndfile_version__, "platform": platform.platform(), "machine": platform.machine()}
    for m in ("onnxruntime", "soxr"):
        if m in sys.modules:
            v[m] = getattr(sys.modules[m], "__version__", "未知")
    if "soxr" in sys.modules:
        sx = sys.modules["soxr"]
        v["libsoxr"] = getattr(sx, "__libsoxr_version__", "未知")
        ext = sorted(Path(sx.__file__).parent.glob("soxr_ext*.so"))
        v["soxr_ext_sha256"] = sha256_file(ext[0]) if ext else "未知"
    if "onnxruntime" in sys.modules:
        ort = sys.modules["onnxruntime"]
        libs = sorted(Path(ort.__file__).parent.glob("capi/libonnxruntime*.dylib"))
        v["libonnxruntime_sha256"] = sha256_file(libs[0]) if libs else "未知"
    return v


def fingerprint(model_info, setup):
    """一次运行的指纹：代码（本模块 + 候选脚本 + 它导入的项目内模块）哈希 · 模型信息 · 运行配置 · 依赖版本 · 片段清单哈希。
    第一个分片写下，之后每个分片、续跑、finalize 都必须一致（codex 016 脚本第 1 轮 P1 #3 · P2 #5）。"""
    code = {}
    for m in list(sys.modules.values()):
        f = getattr(m, "__file__", None)
        if f and Path(f).resolve().is_relative_to(ROOT):
            code[str(Path(f).resolve().relative_to(ROOT))] = sha256_file(f)
    main_mod = sys.modules["__main__"]
    if getattr(main_mod, "__file__", None):
        f = Path(main_mod.__file__).resolve()
        code[str(f.relative_to(ROOT)) if f.is_relative_to(ROOT) else f.name] = sha256_file(f)
    return {"code_sha256": dict(sorted(code.items())), "model": model_info, "setup": setup() if setup else None,
            "versions": versions(), "clips_sha256": sha256_file(ROOT / CLIPS)}


def pin_fingerprint(shard_dir, fp, out=None):
    """第一次：原子建档（先写临时文件，再 `os.link` 发布；已有就不覆盖）。之后：比对。
    输出目录里已有 wav 却没有指纹（旧版脚本跑的）→ 拒绝续跑（codex 016 脚本第 2 轮 P1 #2）。"""
    p = shard_dir / "fingerprint.json"
    shard_dir.mkdir(parents=True, exist_ok=True)
    if not p.exists():
        if out is not None and any(out.glob("*/*.wav")):
            raise SystemExit(f"{out.name} 里已有输出但没有运行指纹（来源不明），换一个新 run-id 重跑")
        tmp = shard_dir / f".fingerprint.{os.getpid()}.tmp"
        tmp.write_text(json.dumps(fp, ensure_ascii=False, indent=2, sort_keys=True) + "\n")
        try:
            os.link(tmp, p)  # 目标已存在就失败：两个进程同时建档时只有一个成功
        except FileExistsError:
            pass
        finally:
            tmp.unlink()
    old = json.loads(p.read_text())
    if old != json.loads(json.dumps(fp, sort_keys=True)):
        diff = sorted(k for k in set(old) | set(fp) if old.get(k) != json.loads(json.dumps(fp.get(k))))
        raise SystemExit(f"本 run-id 已用别的代码 / 配置 / 环境跑过（不同之处：{diff}），换一个新 run-id 重跑")


def write_out(path, y, n):
    y = np.asarray(y, dtype=np.float32)
    if y.shape != (n,):
        raise ValueError(f"{path.name} 输出形状 {y.shape} ≠ ({n},)")
    if not np.all(np.isfinite(y)):
        raise ValueError(f"{path.name} 输出有非有限值")
    path.parent.mkdir(parents=True, exist_ok=True)
    # 先写临时文件再改名：中途被打断不会留下半截 wav，续跑时「已存在就跳过」不会把它当完成
    tmp = path.with_name(path.name + ".part")
    sf.write(str(tmp), y, SR, subtype="FLOAT", format="WAV")
    os.replace(tmp, path)


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
    ap.add_argument("--probe", action="store_true", help="冒烟：只在第一段上量前视与对齐、计一段耗时，什么都不写")
    return ap.parse_args()


def main(name, process, nominal_latency, model_info, setup=None, native_rate=None):
    """候选脚本入口。`process(x16) -> y16`（等长、已对齐）；`nominal_latency` 为模型标称的 16 kHz 延迟。
    声明延迟 = max(标称, 实测前视)。`setup()` 可选，返回额外参数写进 builds/。不覆盖已有输出文件（同 run-id 续跑时跳过）。
    `native_rate`：模型原生采样率（Hz），写进 run.json 给延迟账用；None = 不写（入口按 MODEL_RATE）。"""
    args = parse_args(name)
    root = main_root()
    clips_rel = scenes_clips_path()
    clips = load_clips(clips_rel)
    if args.limit:
        clips = clips[: args.limit]
    picks = trunc_picks(clips)
    rid = args.run_id or run_id()
    out = root / "_tmp/exp016" / name / rid
    # 分片耗时记在输出目录旁边：入口不许输出目录里有没列的子目录
    shard_dir = out.parent / f"{rid}.shards"
    builds = HERE / "builds" / f"{name}-{rid}"
    if args.probe:
        t0 = time.time()
        latency, rec = probe(process, clips, root, nominal_latency)
        rec.update(declared_latency=latency, probe_wall_s=round(time.time() - t0, 2))
        print(json.dumps(rec, ensure_ascii=False))
        return rec
    if args.finalize:
        pin_fingerprint(shard_dir, fingerprint(model_info, setup), out)  # 收尾时的环境也必须和跑的时候一样
        latency, rec = probe(process, clips, root, nominal_latency)
        info = dict(model_info, latency_probe=rec)
        return finalize(name, out, builds, clips_rel, clips, picks, latency, info, setup, args, shard_dir, native_rate)
    k, n = (int(v) for v in args.shard.split("/"))
    pin_fingerprint(shard_dir, fingerprint(model_info, setup), out)
    t0, done, skipped, audio_s = time.time(), 0, 0, 0.0
    for c, kind, x in jobs(clips, root, picks, (k, n)):
        p = out / kind / f"{c['id']}.wav"
        if p.exists():
            check_existing(p, len(x))
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
    shard_dir.mkdir(parents=True, exist_ok=True)
    (shard_dir / f"{k}-of-{n}.json").write_text(json.dumps(stats, ensure_ascii=False) + "\n")
    print(f"[{name}] 分片 {k}/{n} 完成：{stats}")


def finalize(name, out, builds, clips_rel, clips, picks, latency_samples, model_info, setup, args, shard_dir=None,
             native_rate=None):
    """写 run.json（入口读）与 builds/（进仓：参数 · 每个输出文件的 sha256 · 分片耗时）。"""
    if builds.exists():
        raise SystemExit(f"{builds} 已存在，不覆盖")
    want = {(kind, c["id"]) for c in clips if c["has_clean_ref"]
            for kind in KINDS if kind in ("mix", "clean") or (kind == "bg" and c["additive"])
            or (kind == "trunc" and c["id"] in picks)}
    have = {(p.parent.name, p.stem) for p in out.glob("*/*.wav")}
    if want != have:
        raise SystemExit(f"输出不全：缺 {len(want - have)} 个、多 {len(have - want)} 个")
    stray = [p.name for p in out.iterdir() if p.is_dir() and p.name not in KINDS] + [p.name for p in out.glob("*/*.part")]
    if stray:
        raise SystemExit(f"输出目录里有入口不认的东西：{stray}")
    n_of = {c["id"]: c["dur_samples"] for c in clips}
    for p in out.glob("*/*.wav"):  # 逐个校验：能读 · 16 kHz · 等长 · 全有限
        check_existing(p, n_of[p.stem])
    rj = {"schema": SCHEMA, "name": name, "clips": clips_rel, "sr": SR, "latency_samples": int(latency_samples),
          "inputs": list(KINDS), "model": model_info}
    if native_rate is not None:
        rj["native_rate"] = int(native_rate)
    rj_path = out / "run.json"
    if rj_path.exists():
        raise SystemExit(f"{rj_path} 已存在，不覆盖")
    rj_text = json.dumps(rj, ensure_ascii=False, indent=2) + "\n"
    builds.mkdir(parents=True)
    (builds / "run.json").write_text(rj_text)
    hashes = sorted(f"{sha256_file(p)}  {p.parent.name}/{p.name}" for p in out.glob("*/*.wav"))
    (builds / "hashes.txt").write_text("\n".join(hashes) + "\n")
    shards = [json.loads(p.read_text()) for p in sorted(shard_dir.glob("*.json"))] if shard_dir else []
    fp_path = shard_dir / "fingerprint.json" if shard_dir else None
    params = {"candidate": name, "limit": args.limit, "files": len(hashes), "shards": shards,
              "cpu_count": os.cpu_count(), "out_dir": f"_tmp/exp016/{name}/{out.name}",
              "fingerprint": json.loads(fp_path.read_text()) if fp_path and fp_path.exists() else None}
    (builds / "params.json").write_text(json.dumps(params, ensure_ascii=False, indent=2) + "\n")
    # run.json 最后发布：入口只认有 run.json 的目录；之前任何一步失败都可以删掉 builds/ 重试（codex 016 脚本第 2 轮 P2 #3）
    tmp = rj_path.with_name(".run.json.tmp")
    tmp.write_text(rj_text)
    os.replace(tmp, rj_path)
    print(f"[{name}] 已写 {out.name}/run.json 与 builds/{builds.name}（{len(hashes)} 个文件）")
