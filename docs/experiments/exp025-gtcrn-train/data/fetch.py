"""exp025 · 下载训练原料与 exp013 原料（只在云机器上跑；D88 本机不放训练数据）。

- 只认 `sources.json`（正本）：地址 · 字节数 · 上游 md5 · sha256
- 断点续传：没下完的留成 `<文件>.part`，下次用 HTTP Range 接着下；服务器不认 Range（回 200）就从头下
- 下完核：字节数 = sources.json；有 md5 核 md5；有 sha256 核 sha256；都没有的，第一次下完把 sha256 记进
  `fetched.json`（进仓），以后以它为准（再下一次对不上即报错）
- 解压：拒绝绝对路径和「..」成员（压缩包是外来数据）；解压完写 `<dest>/.done-<文件名>` 标记，下次跳过
用法（云上，仓库根下）：
  python3 docs/experiments/exp025-gtcrn-train/data/fetch.py --root /workspace --only A1,A2,A3,A4,A5
  python3 .../fetch.py --root /workspace --only E1,E2,E3   # exp013 原料（ESC-50 与 RIR 走 exp013 fetch.py）
  python3 .../fetch.py --plan                               # 只列要下什么、多大，不联网
  python3 .../fetch.py --head --only A1,A2,A3,A4,A5         # 只核服务器报的大小，不下载
  开下载前会查盘：还要的空间（含解压峰值 ≈ 包 × 2）大于剩余空间就停
"""
import argparse
import hashlib
import json
import shutil
import sys
import tarfile
import time
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath

HERE = Path(__file__).resolve().parent
SOURCES = HERE / "sources.json"
FETCHED = HERE / "fetched.json"
CHUNK = 1 << 20
UA = "vdn-train-data-fetch/0.1"


def load_sources(path=SOURCES):
    return json.loads(Path(path).read_text())["groups"]


def load_fetched(path=FETCHED):
    p = Path(path)
    return json.loads(p.read_text()) if p.exists() else {"_说明": "第一次在云上下完记下的 sha256（sources.json 里为 null 的那些）", "files": {}}


def save_fetched(rec, path=FETCHED):
    rec["files"] = dict(sorted(rec["files"].items()))
    Path(path).write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n")


def hashes(path):
    h5, h256 = hashlib.md5(), hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(CHUNK), b""):
            h5.update(b)
            h256.update(b)
    return h5.hexdigest(), h256.hexdigest()


def download(url, out, expect_bytes, retries=5, timeout=120):
    """断点续传下载到 out；返回实际字节数。中途断了重试，每次从 .part 的末尾接着下。"""
    out = Path(out)
    part = out.with_name(out.name + ".part")
    out.parent.mkdir(parents=True, exist_ok=True)
    for attempt in range(retries):
        have = part.stat().st_size if part.exists() else 0
        if expect_bytes and have > expect_bytes:
            part.unlink()
            have = 0
        if expect_bytes and have == expect_bytes:
            break
        req = urllib.request.Request(url, headers={"User-Agent": UA, **({"Range": f"bytes={have}-"} if have else {})})
        try:
            with urllib.request.urlopen(req, timeout=timeout) as r:
                mode = "ab" if (have and r.status == 206) else "wb"  # 服务器不认 Range 时回 200，从头写
                with open(part, mode) as f:
                    for b in iter(lambda: r.read(CHUNK), b""):
                        f.write(b)
            if not expect_bytes:
                break
        except (OSError, urllib.error.URLError) as e:
            print(f"  第 {attempt + 1} 次中断：{e}；{2 ** attempt} s 后接着下", flush=True)
            time.sleep(2 ** attempt)
    n = part.stat().st_size
    if expect_bytes and n != expect_bytes:
        raise SystemExit(f"{out.name} 下了 {n} 字节，应为 {expect_bytes}：留着 .part，重跑会接着下")
    part.rename(out)
    return n


def verify(path, f, fetched):
    """核 md5 / sha256；返回 sha256。sources.json 没 sha256 的，用 fetched.json 里的（没有就记下）。"""
    md5, sha = hashes(path)
    if f.get("md5") and md5 != f["md5"]:
        raise SystemExit(f"{f['name']} md5 {md5} ≠ 上游 {f['md5']}")
    want = f.get("sha256") or fetched["files"].get(f["url"], {}).get("sha256")
    if want and sha != want:
        raise SystemExit(f"{f['name']} sha256 {sha} ≠ 记录 {want}")
    if not f.get("sha256"):
        fetched["files"][f["url"]] = {"name": f["name"], "bytes": Path(path).stat().st_size, "sha256": sha, "md5": md5}
    return sha


def _safe(name):
    p = PurePosixPath(name)
    if p.is_absolute() or ".." in p.parts:
        raise SystemExit(f"压缩包成员路径不安全：{name!r}，拒绝解压")
    return p


def extract(arc, dest, kind):
    dest = Path(dest)
    dest.mkdir(parents=True, exist_ok=True)
    if kind == "zip":
        with zipfile.ZipFile(arc) as z:
            for m in z.namelist():
                _safe(m)
            z.extractall(dest)
    elif kind == "tar":
        with tarfile.open(arc) as t:
            for m in t.getmembers():
                _safe(m.name)
                if m.issym() or m.islnk():
                    raise SystemExit(f"压缩包里有链接 {m.name!r}，拒绝解压")
            t.extractall(dest, filter="data")
    else:
        raise ValueError(kind)


def fetch_file(f, root, fetched, keep_archive=False):
    root = Path(root)
    if f["extract"] == "none":
        out = root / f["dest"]
        if out.exists() and hashes(out)[1] == (f.get("sha256") or fetched["files"].get(f["url"], {}).get("sha256")):
            return "已有"
        download(f["url"], out, f["bytes"])
        verify(out, f, fetched)
        return "下完"
    dest = root / f["dest"]
    done = dest / f".done-{f['name']}"
    if done.exists():
        return "已解压"
    arc = root / "archives" / f["name"]
    if not arc.exists():
        download(f["url"], arc, f["bytes"])
    sha = verify(arc, f, fetched)
    extract(arc, dest, f["extract"])
    done.write_text(sha + "\n")
    if not keep_archive:
        arc.unlink()  # 网络盘按量计费：解压完删包；sha256 已记，要复核就重下
    return "下完并解压"


PEAK_FACTOR = 2.05  # 压缩包与解压目录同时在盘上的峰值 ≈ 包大小 × 2（音频几乎压不动）+ 余量；解压完删包


def need_bytes(groups, only, root):
    """还要多少盘：没解压完的文件按 包 × PEAK_FACTOR 算（已有 .done 标记的不算）。"""
    n = 0
    for gid in only:
        for f in groups[gid]["files"]:
            done = Path(root) / f["dest"] / f".done-{f['name']}" if f["extract"] != "none" else Path(root) / f["dest"]
            if not done.exists():
                n += f["bytes"] * (PEAK_FACTOR if f["extract"] != "none" else 1.0)
    return int(n)


def check_space(groups, only, root):
    free = shutil.disk_usage(root).free
    need = need_bytes(groups, only, root)
    if need > free:
        raise SystemExit(f"盘不够：还要约 {need / 1e9:.1f} GB（含解压峰值），{root} 只剩 {free / 1e9:.1f} GB")
    return need, free


def head(groups, only):
    """只发 HEAD 请求，核 sources.json 里的字节数和服务器报的一致（不下载）。返回不一致的条目。"""
    bad = []
    for gid in only:
        for f in groups[gid]["files"]:
            req = urllib.request.Request(f["url"], method="HEAD", headers={"User-Agent": UA})
            with urllib.request.urlopen(req, timeout=60) as r:
                n = int(r.headers.get("content-length") or 0)
                rng = r.headers.get("accept-ranges", "")
            ok = n == f["bytes"]
            print(f"{gid} {f['name']}：服务器 {n} 字节 {'=' if ok else '≠'} 清单 {f['bytes']}；续传 {rng or '未声明'}", flush=True)
            if not ok:
                bad.append(f["name"])
    return bad


def plan(groups, only):
    total = 0
    for gid in only:
        g = groups[gid]
        n = sum(f["bytes"] for f in g["files"])
        total += n
        print(f"{gid} {g['name']}：{len(g['files'])} 个文件 {n / 1e9:.2f} GB · {g['license']} · 能训练={g['train_ok']}")
    print(f"合计 {total / 1e9:.2f} GB")
    return total


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default="", help="数据根（云上网络盘挂载点）")
    ap.add_argument("--only", default="A1,A2,A3,A4,A5")
    ap.add_argument("--plan", action="store_true", help="只列清单，不联网")
    ap.add_argument("--head", action="store_true", help="只发 HEAD 请求核大小，不下载（云上开工前先跑这个）")
    ap.add_argument("--keep-archive", action="store_true")
    a = ap.parse_args()
    groups = load_sources()
    only = [x for x in a.only.split(",") if x]
    unknown = [x for x in only if x not in groups]
    if unknown:
        raise SystemExit(f"sources.json 里没有 {unknown}")
    if a.plan:
        plan(groups, only)
        return
    if a.head:
        bad = head(groups, only)
        sys.exit(1 if bad else 0)
    if sys.platform == "darwin":
        raise SystemExit("本机不下训练数据（D88 · D97）：请在云机器上跑；只看清单用 --plan")
    if not a.root:
        raise SystemExit("要给 --root（云上网络盘挂载点）")
    need, free = check_space(groups, only, a.root)
    print(f"盘：还要约 {need / 1e9:.1f} GB，剩 {free / 1e9:.1f} GB", flush=True)
    fetched = load_fetched()
    try:
        for gid in only:
            for f in groups[gid]["files"]:
                t0 = time.time()
                st = fetch_file(f, a.root, fetched, a.keep_archive)
                print(f"{gid} {f['name']}：{st}，{time.time() - t0:.0f} s", flush=True)
    finally:
        save_fetched(fetched)
    print(f"完成；新记的 sha256 在 {FETCHED.name}，请提交进仓")


if __name__ == "__main__":
    main()
