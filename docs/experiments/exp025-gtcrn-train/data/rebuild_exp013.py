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
另一条路（2026-11-25 大总管定：云上用 main 出的新 build 只跑一次，直接核它）：
  python3 rebuild_exp013.py --check-dir <拷回的 build 目录> --inputs-log <云上 exp013_inputs.py --verify 的输出>
  同样四项：4540 个旧文件逐个相同 · exclude.json 全等 · 版本相同 · 1,994 个原料内容对上（build 自带的素材索引只记路径与字节数）
"""
import argparse
import hashlib
import json
import os
import re
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
        out[re.sub(r"^_tmp/exp013/[^/]+/", "", p)] = sha  # build id 只是名字，任何 id 都去掉
    return out


def compare(old_path, new_path):
    old, new = read_hashes(old_path), read_hashes(new_path)
    same = sum(1 for k, v in old.items() if new.get(k) == v)
    diff = sorted(k for k in old if k in new and new[k] != old[k])
    return {"old": len(old), "new": len(new), "same": same, "differ": len(diff), "missing": sorted(set(old) - set(new))[:50],
            "extra": sorted(set(new) - set(old))[:50], "n_extra": len(set(new) - set(old)), "differ_first": diff[:50],
            "ok": same == len(old)}  # 判据只看旧的 4540 个逐个相同；新代码多出的轨（如早期混响参考轨）单列 extra，不算不过


def code_drift():
    """当前 exp013 代码 / 参数文件和旧 build manifest 记的哈希比：列出变了的文件（变了不一定不过，结果以逐文件哈希为准）。"""
    import hashlib

    m = json.loads((OLD / "manifest.json").read_text())
    root = HERE.parents[3]
    out = []
    for f, h in {**m["code_sha256"], **m["params_sha256"]}.items():
        p = (root / f) if "/" in f else (EXP013 / f)
        if not p.exists() or hashlib.sha256(p.read_bytes()).hexdigest() != h:
            out.append(f)
    return out


def file_hashes(names):
    root = HERE.parents[3]
    return {f: hashlib.sha256(((root / f) if "/" in f else (EXP013 / f)).read_bytes()).hexdigest() for f in names}


def check_dir(new, inputs_ok):
    """核一份已出好的 build 目录（只要 hashes.txt · exclude.json · manifest.json）。"""
    new = Path(new)
    rep = compare(OLD / "hashes.txt", new / "hashes.txt")
    m = json.loads((new / "manifest.json").read_text())
    rep["exclude_equal"] = json.loads((new / "exclude.json").read_text()) == json.loads((OLD / "exclude.json").read_text())
    rep["version_drift"] = version_diff(m["versions"]) if False else {k: [v, m["versions"].get(k)] for k, v in
                                                                       json.loads((OLD / "manifest.json").read_text())["versions"].items() if m["versions"].get(k) != v}
    rep["inputs_ok"] = bool(inputs_ok)
    rep["checked_build"] = m.get("build_id", new.name)
    rep["new_code_sha256"] = {**m["code_sha256"], **m["params_sha256"]}
    rep["ok"] = rep["ok"] and rep["exclude_equal"] and not rep["version_drift"] and rep["inputs_ok"]
    return rep


def drift_covered():
    """rebuild/ 里有没有一份 ok 的报告，核的那份新 build 正是用现行 exp013 代码出的。"""
    for p in sorted(REPORTS.glob("exp013-*.json")) if REPORTS.exists() else []:
        r = json.loads(p.read_text())
        code = r.get("new_code_sha256")
        if r.get("ok") and code and all((HERE.parents[3] / f if "/" in f else EXP013 / f).exists() for f in code) and file_hashes(code) == code:
            return True
    return False


def run(cmd, env):
    print("$", " ".join(cmd), flush=True)
    subprocess.run(cmd, env=env, check=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", required=True, help="云上数据根；原料在 <root>/vdn-data")
    ap.add_argument("--fetch", action="store_true", help="走 A：从官方源重下原料")
    ap.add_argument("--workers", type=int, default=16)
    ap.add_argument("--allow-version-drift", action="store_true")
    ap.add_argument("--check-dir", default="", help="不重跑，直接核这份已出好的 build 目录")
    ap.add_argument("--inputs-log", default="", help="同一台机器上 exp013_inputs.py --verify 的输出（要有「原料全对」）")
    a = ap.parse_args()
    if a.check_dir:
        log = Path(a.inputs_log).read_text() if a.inputs_log else ""
        rep = check_dir(a.check_dir, "原料全对" in log)
        rep.update({"build": BUILD, "inputs_log_sha256": hashlib.sha256(log.encode()).hexdigest(), "when": time.strftime("%Y-%m-%d %H:%M:%S")})
        return report(rep)
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
    rep.update({"build": BUILD, "versions": versions(), "version_drift": drift, "code_drift": code_drift(), "fetched": a.fetch, "seconds": round(time.time() - t0, 1),
                "when": time.strftime("%Y-%m-%d %H:%M:%S")})
    report(rep)


def report(rep):
    REPORTS.mkdir(exist_ok=True)
    out = REPORTS / f"exp013-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out.write_text(json.dumps(rep, ensure_ascii=False, indent=1) + "\n")
    print(f"重建 {rep['same']}/{rep['old']} 一致，exclude.json {'相同' if rep['exclude_equal'] else '不同'}；报告 {out.name}（请提交进仓）")
    sys.exit(0 if rep["ok"] else 1)


if __name__ == "__main__":
    main()
