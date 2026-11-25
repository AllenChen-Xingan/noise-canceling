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
现行判据（2026-11-25 大总管定）：跨平台做不到逐位相同（云上 x86 对本机 arm64，样本差约 4e-13，见
exp013 crosscheck-20261125.md），所以不再要求云上重建逐位同 17f5。云上评测用本机 6323 的音频，改核：
  1. 本机 6323（main 现行代码）旧 4540 个文件逐个同 17f5
  2. 云上 6323 音频按 hashes.txt 全部对上（`sha256sum -c` 的输出）
  3. 云上那份 build（aa9f）的 exclude.json 与依赖版本同 17f5，代码与参数哈希同 6323
  4. 云上原料 1,994 个内容对上（exp013_inputs.py --verify；build 自带的素材索引只记路径与字节数）
  「官方源可复现」这次没验证：云上原料是本机上传的
  python3 rebuild_exp013.py --check-cloud --local <6323 目录> --cloud <aa9f 目录> --upload-log <sha256sum -c 输出> --inputs-log <verify 输出>
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


def _manifest(d):
    return json.loads((Path(d) / "manifest.json").read_text())


def check_cloud(local, cloud, upload_log, inputs_log):
    """按现行判据核（见文件头）；返回报告，ok 为四项全过。"""
    local, cloud = Path(local), Path(cloud)
    ml, mc, mo = _manifest(local), _manifest(cloud), _manifest(OLD)
    vs_old = compare(OLD / "hashes.txt", local / "hashes.txt")
    want = set(read_hashes(local / "hashes.txt"))
    okp, failed = set(), []
    for line in upload_log.splitlines():
        if ": " not in line:
            continue
        path, st = line.rsplit(": ", 1)
        key = re.sub(r"^_tmp/exp013/[^/]+/", "", path.strip())
        (okp.add(key) if st.strip() == "OK" else failed.append(key))
    up = {"expected": len(want), "ok": len(okp & want), "failed": failed[:50], "missing": sorted(want - okp)[:50]}
    up["all_ok"] = not failed and okp >= want
    code = lambda m: {**m["code_sha256"], **m["params_sha256"]}  # noqa: E731
    rep = {"local_build": ml.get("build_id", local.name), "cloud_build": mc.get("build_id", cloud.name),
           "local_vs_old": {k: vs_old[k] for k in ("old", "same", "differ", "missing", "n_extra", "ok")},
           "upload": up,
           "cloud_exclude_equal": json.loads((cloud / "exclude.json").read_text()) == json.loads((OLD / "exclude.json").read_text()),
           "cloud_version_drift": {k: [v, mc["versions"].get(k)] for k, v in mo["versions"].items() if mc["versions"].get(k) != v},
           "cloud_code_equal_local": code(mc) == code(ml),
           "inputs_ok": "原料全对" in inputs_log,
           "official_source": "未验证：云上原料是本机上传的，只证明与本机原料逐个相同（从官方源重下记进 backlog）",
           "bit_exact_cross_platform": "不要求：x86 对 arm64 浮点差约 4e-13（exp013 crosscheck-20261125.md）",
           "new_code_sha256": code(ml)}
    rep["ok"] = (vs_old["ok"] and up["all_ok"] and rep["cloud_exclude_equal"] and not rep["cloud_version_drift"]
                 and rep["cloud_code_equal_local"] and rep["inputs_ok"])
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
    ap.add_argument("--root", default="", help="云上数据根；原料在 <root>/vdn-data")
    ap.add_argument("--fetch", action="store_true", help="走 A：从官方源重下原料")
    ap.add_argument("--workers", type=int, default=16)
    ap.add_argument("--allow-version-drift", action="store_true")
    ap.add_argument("--check-cloud", action="store_true", help="不重跑：按现行判据核本机 6323 与云上 build（见文件头）")
    ap.add_argument("--local", default="", help="本机 build 目录（6323）")
    ap.add_argument("--cloud", default="", help="云上 build 的小文件目录（aa9f）")
    ap.add_argument("--upload-log", default="", help="云上对 6323 音频跑 sha256sum -c hashes.txt 的输出")
    ap.add_argument("--inputs-log", default="", help="同一台机器上 exp013_inputs.py --verify 的输出（要有「原料全对」）")
    a = ap.parse_args()
    if a.check_cloud:
        up, inp = (Path(x).read_text() for x in (a.upload_log, a.inputs_log))
        rep = check_cloud(a.local, a.cloud, up, inp)
        rep.update({"build": BUILD, "upload_log_sha256": hashlib.sha256(up.encode()).hexdigest(),
                    "inputs_log_sha256": hashlib.sha256(inp.encode()).hexdigest(), "when": time.strftime("%Y-%m-%d %H:%M:%S")})
        return report(rep)
    if not a.root:
        raise SystemExit("整套重建要 --root；只核云上 build 用 --check-cloud")
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
        m = json.loads((new / "manifest.json").read_text())
        rep["new_code_sha256"] = {**m["code_sha256"], **m["params_sha256"]}
    rep.update({"build": BUILD, "versions": versions(), "version_drift": drift, "code_drift": code_drift(), "fetched": a.fetch, "seconds": round(time.time() - t0, 1),
                "when": time.strftime("%Y-%m-%d %H:%M:%S")})
    report(rep)


def report(rep):
    REPORTS.mkdir(exist_ok=True)
    out = REPORTS / f"exp013-{time.strftime('%Y%m%d-%H%M%S')}.json"
    out.write_text(json.dumps(rep, ensure_ascii=False, indent=1) + "\n")
    if "local_vs_old" in rep:
        v, u = rep["local_vs_old"], rep["upload"]
        print(f"本机 {v['same']}/{v['old']} 同旧 build；云上音频 {u['ok']}/{u['expected']} 对上；exclude {'相同' if rep['cloud_exclude_equal'] else '不同'}；"
              f"版本{'相同' if not rep['cloud_version_drift'] else '不同'}；原料{'全对' if rep['inputs_ok'] else '不对'}；官方源：未验证")
    else:
        print(f"重建 {rep['same']}/{rep['old']} 一致，exclude.json {'相同' if rep['exclude_equal'] else '不同'}")
    print(f"{'通过' if rep['ok'] else '不过'}；报告 {out.name}（请提交进仓）")
    sys.exit(0 if rep["ok"] else 1)


if __name__ == "__main__":
    main()
