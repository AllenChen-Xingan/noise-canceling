"""exp025 · 在云机器上按仓内脚本 + 种子重建 exp013 正式 build，逐文件核旧哈希（014 第 0 步 ③，判据 4540/4540）。

步骤：
1. 核版本：python · numpy · scipy · soundfile 要和旧 build manifest 记的一样（浮点结果逐位一致的前提）；不一样就停，
   除非 --allow-version-drift（结果照样核，但报告里标出版本不同）
2. 原料：两条路，优先 A
   A. 从官方源重下：本目录 fetch.py --only E1,E2,E3 + exp013 自己的 fetch.py --only esc50,rir
   B. 从本机上传 ~/vdn-data 的同名子目录到 <root>/vdn-data（cloudrun 做），不跑下载
   两条路之后都跑 exp013_inputs.py --verify：1,994 个原料逐个 sha256 对上才往下走
3. 跑 exp013 build.py（同一 build id，清单写临时目录，不碰仓里 builds/）
4. 比 hashes.txt：逐行比，外加 exclude.json 全等；报告写 rebuild/exp013-<时间>.json（进仓）
用法（云上，仓库根下）：
  python3 docs/experiments/exp025-gtcrn-train/data/rebuild_exp013.py --root /workspace [--fetch] [--workers 16]
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
EXP013 = HERE.parents[1] / "exp013-scene-suite"
BUILD = "20261103230756-17f5"
OLD = EXP013 / "builds" / BUILD
REPORTS = HERE / "rebuild"


def versions():
    import numpy
    import scipy
    import soundfile

    return {"python": sys.version.split()[0], "numpy": numpy.__version__, "scipy": scipy.__version__, "soundfile": soundfile.__version__}


def version_diff(old=None):
    old = old or json.loads((OLD / "manifest.json").read_text())["versions"]
    now = versions()
    return {k: [old[k], now.get(k)] for k in old if old[k] != now.get(k)}


def read_hashes(path):
    """hashes.txt → {相对路径（去掉 _tmp/exp013/<build>/ 前缀）: sha256}。"""
    out = {}
    for line in Path(path).read_text().splitlines():
        if not line.strip():
            continue
        sha, p = line.split("  ", 1)
        out[p.split(f"{BUILD}/", 1)[-1]] = sha
    return out


def compare(old_path, new_path):
    old, new = read_hashes(old_path), read_hashes(new_path)
    same = sum(1 for k, v in old.items() if new.get(k) == v)
    diff = sorted(k for k in old if k in new and new[k] != old[k])
    return {"old": len(old), "new": len(new), "same": same, "differ": len(diff), "missing": sorted(set(old) - set(new))[:50],
            "extra": sorted(set(new) - set(old))[:50], "differ_first": diff[:50], "ok": same == len(old) == len(new)}


def run(cmd, env):
    print("$", " ".join(cmd), flush=True)
    subprocess.run(cmd, env=env, check=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True, help="云上数据根；原料在 <root>/vdn-data")
    ap.add_argument("--fetch", action="store_true", help="走 A：从官方源重下原料")
    ap.add_argument("--workers", type=int, default=16)
    ap.add_argument("--allow-version-drift", action="store_true")
    a = ap.parse_args()
    if sys.platform == "darwin":
        raise SystemExit("重建在云上跑（D97）；本机只跑单元测试")
    drift = version_diff()
    if drift and not a.allow_version_drift:
        raise SystemExit(f"版本和旧 build 不同 {drift}：先装同版本（README 数据一节有命令），或加 --allow-version-drift")
    root = Path(a.root).resolve()
    env = dict(os.environ, HOME=str(root))  # exp013 写死 ~/vdn-data：把 HOME 指到数据根
    py = sys.executable
    t0 = time.time()
    if a.fetch:
        run([py, str(HERE / "fetch.py"), "--root", str(root), "--only", "E1,E2,E3"], env)
        run([py, str(EXP013 / "fetch.py"), "--only", "esc50,rir"], env)
    run([py, str(HERE / "exp013_inputs.py"), "--verify", "--data", str(root / "vdn-data")], env)
    with tempfile.TemporaryDirectory(dir=root) as tmp:
        t = Path(tmp)
        run([py, str(EXP013 / "build.py"), "--build", BUILD, "--workers", str(a.workers),
             "--builds-dir", str(t / "builds"), "--audio-root", str(t / "audio")], env)
        new = t / "builds" / BUILD
        rep = compare(OLD / "hashes.txt", new / "hashes.txt")
        rep["exclude_equal"] = json.loads((new / "exclude.json").read_text()) == json.loads((OLD / "exclude.json").read_text())
        rep["ok"] = rep["ok"] and rep["exclude_equal"]
    rep.update({"build": BUILD, "versions": versions(), "version_drift": drift, "fetched": a.fetch, "seconds": round(time.time() - t0, 1),
                "when": time.strftime("%Y-%m-%d %H:%M:%S")})
    REPORTS.mkdir(exist_ok=True)
    out = REPORTS / f"exp013-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out.write_text(json.dumps(rep, ensure_ascii=False, indent=1) + "\n")
    print(f"重建 {rep['same']}/{rep['old']} 一致，exclude.json {'相同' if rep['exclude_equal'] else '不同'}；报告 {out.name}（请提交进仓）")
    sys.exit(0 if rep["ok"] else 1)


if __name__ == "__main__":
    main()
