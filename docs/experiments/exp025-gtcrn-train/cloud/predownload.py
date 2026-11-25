"""在云机器里跑：用 aria2c 多连接把 sources.json 里的压缩包预先下到 <root>/archives/<文件名>。

为什么：OpenSLR 单连接 0.43–1.39 MB/s，aria2c 8 连接 + 三个镜像实测 5.6 MB/s（2026-11-24，28 号包）。
fetch.py（traindata 的正本）发现 archives/ 里已有包就跳过下载，照常核 md5 / sha256、解压、记 fetched.json。
本脚本只多做两件事：下完先过完整性检查（zip 用 unzip -t，tar 用 tar -t），每个文件记一行速度。
只用标准库；sources.json 由 traindata 维护，这里只读。

退出码：3 盘量会超上限 · 4 大小和清单不符 · 5 完整性检查不过 · 6 du 失败。
用法：python3 predownload.py <sources.json> <root> A1,A2,A3,A4,A5,E2 [--max-gb 120]
"""

import json
import os
import subprocess
import sys
import time

OPENSLR_MIRRORS = ("https://www.openslr.org", "https://openslr.trmal.net", "https://openslr.elda.org")


def mirrors(url: str) -> list:
    """OpenSLR 的地址展开成三个镜像（同一路径）；其他地址原样。"""
    for m in OPENSLR_MIRRORS:
        if url.startswith(m + "/resources/"):
            path = url[len(m):]
            return [x + path for x in OPENSLR_MIRRORS]
    return [url]


def du_gb(root: str) -> float:
    """du 失败就停，不当成 0（评审第 1 轮 #9）。"""
    r = subprocess.run(["du", "-sb", root], capture_output=True, text=True)
    out = r.stdout.split()
    if r.returncode != 0 or not out:
        print(json.dumps({"stop": f"du 失败：{r.stderr.strip()[-200:]}"}, ensure_ascii=False), flush=True)
        sys.exit(6)
    return int(out[0]) / 1e9


def integrity_cmd(path: str, kind: str) -> list:
    if kind == "zip":
        return ["unzip", "-tq", path]
    if kind == "tar":
        return ["tar", "-tzf", path] if path.endswith((".tar.gz", ".tgz")) else ["tar", "-tf", path]
    return ["true"]


def aria2_cmd(urls: list, out_dir: str, name: str) -> list:
    return ["aria2c", "-q", "-x8", "-s8", "-k8M", "--file-allocation=none", "--auto-file-renaming=false",
            "--allow-overwrite=false", "--max-tries=10", "--retry-wait=10", "-d", out_dir, "-o", name, *urls]


def done_marker(root: str, f: dict) -> str:
    return os.path.join(root, f["dest"], f".done-{f['name']}")


def main(argv):
    sources, root, only = argv[1], argv[2], argv[3].split(",")
    max_gb = float(argv[argv.index("--max-gb") + 1]) if "--max-gb" in argv else 120.0
    with open(sources, encoding="utf-8") as fh:
        groups = json.load(fh)["groups"]
    arc_dir = os.path.join(root, "archives")
    os.makedirs(arc_dir, exist_ok=True)
    total = sum(f["bytes"] for g in only for f in groups[g]["files"])
    got = 0
    for g in only:
        for f in groups[g]["files"]:
            path = os.path.join(arc_dir, f["name"])
            if os.path.exists(done_marker(root, f)):
                got += f["bytes"]
                print(json.dumps({"group": g, "name": f["name"], "status": "已解压，跳过"}), flush=True)
                continue
            used = du_gb(root)
            if used + f["bytes"] / 1e9 > max_gb:  # 把这个包也算进去，下完不会越线（评审第 1 轮 #9）
                print(json.dumps({"stop": f"盘量 {used:.1f} GB + 本包 {f['bytes'] / 1e9:.1f} GB 超过 {max_gb} GB"},
                                 ensure_ascii=False), flush=True)
                sys.exit(3)
            t0 = time.time()
            if not (os.path.exists(path) and os.path.getsize(path) == f["bytes"]
                    and not os.path.exists(path + ".aria2")):
                subprocess.run(aria2_cmd(mirrors(f["url"]), arc_dir, f["name"]), check=True)
            dt = time.time() - t0
            n = os.path.getsize(path)
            if n != f["bytes"]:
                print(json.dumps({"stop": f"{f['name']} 大小 {n} ≠ 清单 {f['bytes']}"}), flush=True)
                sys.exit(4)
            ok = subprocess.run(integrity_cmd(path, f["extract"]), stdout=subprocess.DEVNULL,
                                stderr=subprocess.PIPE, text=True)
            got += n
            print(json.dumps({"group": g, "name": f["name"], "bytes": n, "seconds": round(dt, 1),
                              "MB_per_s": round(n / dt / 1e6, 2) if dt > 1 else None,
                              "integrity": "ok" if ok.returncode == 0 else ok.stderr[-300:],
                              "progress_GB": round(got / 1e9, 2), "total_GB": round(total / 1e9, 2),
                              "disk_GB": round(du_gb(root), 1)}, ensure_ascii=False), flush=True)
            if ok.returncode != 0:
                sys.exit(5)


if __name__ == "__main__":
    main(sys.argv)
