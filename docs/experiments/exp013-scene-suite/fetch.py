"""exp013 · 下载评测素材。
- 第一批（大总管 2026-11-03 批）：ESC-50 约 170 段 + DEMAND TCAR / TMETRO / PSTATION
- 第二批 `--only rir`（人选下载档 A，大总管 2026-11-03 转）：MIT IR Survey + ACE 单声道包 + McVAMPIRE，实测 RIR，见 reverb-plan.md

- 存到 ~/vdn-data/ 下（仓外）；每个文件的 URL · sha256 · 许可写进本目录 `materials.json`（进仓）
- 已存在且哈希对得上的跳过；单个包实测超过 150 MB 就停（大总管规定）
- ESC-50 每类按文件名排序取前 PER_CLASS 个（确定性，可复现）

用法：python3 docs/experiments/exp013-scene-suite/fetch.py [--only esc50,demand]
"""
import argparse
import csv
import hashlib
import io
import json
import time
import urllib.request
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
DATA = Path.home() / "vdn-data"
OUT = HERE / "materials.json"
MAX_BYTES = 150 * 1024 * 1024

ESC_BASE = "https://raw.githubusercontent.com/karolpiczak/ESC-50/master"
ESC_CLASSES = [
    "dog", "crying_baby", "keyboard_typing", "mouse_click", "door_wood_knock", "door_wood_creaks",
    "footsteps", "coughing", "laughing", "sneezing", "car_horn", "can_opening", "drinking_sipping",
    "pouring_water",
]
PER_CLASS = 12
ESC_LICENSE = "CC-BY-NC-3.0（ESC-50 数据集许可；只评测，不训练）"

DEMAND_URL = "https://zenodo.org/records/1227121/files/{env}_16k.zip?download=1"
DEMAND_ENVS = ["TCAR", "TMETRO", "PSTATION"]
DEMAND_LICENSE = "CC-BY-4.0（Zenodo 1227121；原发布 CC-BY-SA-3.0，D64 两者都可）"


def sha256(b):
    return hashlib.sha256(b).hexdigest()


def get(url):
    with urllib.request.urlopen(url, timeout=120) as r:
        n = int(r.headers.get("content-length") or 0)
        if n > MAX_BYTES:
            raise SystemExit(f"停：{url} 有 {n / 1e6:.0f} MB，超过 150 MB，先报大总管")
        return r.read()


def load_record():
    return json.loads(OUT.read_text()) if OUT.exists() else {"_说明": "exp013 下载素材登记：URL · sha256 · 许可。音频在 ~/vdn-data/ 下，不进仓", "files": {}}


def fetch_esc50(rec):
    meta = get(f"{ESC_BASE}/meta/esc50.csv").decode()
    rows = [r for r in csv.DictReader(io.StringIO(meta)) if r["category"] in ESC_CLASSES]
    d = DATA / "esc50"
    d.mkdir(parents=True, exist_ok=True)
    (d / "esc50.csv").write_text(meta)
    for cls in ESC_CLASSES:
        pick = sorted((r for r in rows if r["category"] == cls), key=lambda r: r["filename"])[:PER_CLASS]
        for r in pick:
            key = f"esc50/{r['filename']}"
            p = DATA / key
            old = rec["files"].get(key)
            if p.exists() and old and sha256(p.read_bytes()) == old["sha256"]:
                continue
            b = get(f"{ESC_BASE}/audio/{r['filename']}")
            p.write_bytes(b)
            rec["files"][key] = {"url": f"{ESC_BASE}/audio/{r['filename']}", "sha256": sha256(b), "bytes": len(b),
                                 "license": ESC_LICENSE, "cls": cls, "fold": int(r["fold"]), "src_file": r["src_file"]}
        print(f"esc50 {cls}: {len(pick)}", flush=True)


def fetch_demand(rec):
    for env in DEMAND_ENVS:
        key = f"demand/{env}_16k.zip"
        p = DATA / key
        if not p.exists():
            b = get(DEMAND_URL.format(env=env))
            p.write_bytes(b)
        b = p.read_bytes()
        rec["files"][key] = {"url": DEMAND_URL.format(env=env), "sha256": sha256(b), "bytes": len(b), "license": DEMAND_LICENSE}
        with zipfile.ZipFile(p) as z:
            z.extractall(DATA / "demand")
        print(f"demand {env}: {len(b) / 1e6:.0f} MB", flush=True)


# 方案写的大小（MB）；实测超过 1.2 倍就停（大总管规定）
RIR = {
    "mit-ir-survey": {"url": "https://mcdermottlab.mit.edu/Reverb/IRMAudio/Audio.zip", "plan_mb": 12, "md5": None,
                      "license": "页面未写许可；镜像标 CC-BY-4.0【未核原文】；只评测、本地卷积、不外发"},
    "ace-single": {"url": "https://zenodo.org/api/records/6257551/files/ACE_Corpus_RIRN_Single.tbz2/content", "plan_mb": 417,
                   "md5": "d365026b3974a198ef58e8d3e647a415",
                   "license": "Zenodo 6257551 元数据 CC-BY-4.0；IEEE DataPort 同页另写 CC-BY-ND-4.0。ND 只禁止分享改编品，本地卷积只评测不外发不冲突（README 第二节）"},
    "mcvampire": {"url": "https://zenodo.org/api/records/12806684/files/In-Car_McVAMPIRE_V1.0.zip/content", "plan_mb": 256,
                  "md5": "187b2d91cb8a3b0ea0ae397a3de8afe0", "license": "CC-BY-4.0（Zenodo 12806684）；只评测"},
}


def stream(url, path, plan_mb):
    """流式下载到文件（不进内存）；Content-Length 超过方案 1.2 倍就停。"""
    with urllib.request.urlopen(url, timeout=300) as r:
        n = int(r.headers.get("content-length") or 0)
        if n > plan_mb * 1.2e6:
            raise SystemExit(f"停：{url} 有 {n / 1e6:.0f} MB，比方案的 {plan_mb} MB 大 20% 以上，先报大总管")
        h5, h256 = hashlib.md5(), hashlib.sha256()
        with open(path, "wb") as f:
            while True:
                b = r.read(1 << 20)
                if not b:
                    break
                f.write(b)
                h5.update(b)
                h256.update(b)
    return h5.hexdigest(), h256.hexdigest(), path.stat().st_size


def fetch_rir(rec):
    import subprocess

    root = DATA / "rir"
    root.mkdir(parents=True, exist_ok=True)
    for name, v in RIR.items():
        ext = ".tbz2" if v["url"].split("/content")[0].endswith(".tbz2") else ".zip"
        arc = root / f"{name}{ext}"
        key = f"rir/{arc.name}"
        if not (arc.exists() and rec["files"].get(key, {}).get("bytes") == arc.stat().st_size):
            md5, sha, size = stream(v["url"], arc, v["plan_mb"])
            if v["md5"] and md5 != v["md5"]:
                arc.unlink()
                raise SystemExit(f"{name} md5 对不上（{md5} ≠ {v['md5']}），已删，先报大总管")
            rec["files"][key] = {"url": v["url"], "sha256": sha, "md5": md5, "bytes": size, "license": v["license"]}
        out = root / name
        if not out.exists():
            out.mkdir()
            if ext == ".zip":
                with zipfile.ZipFile(arc) as z:
                    z.extractall(out)
            else:
                subprocess.run(["tar", "-xjf", str(arc), "-C", str(out)], check=True)
        print(f"rir {name}: {arc.stat().st_size / 1e6:.0f} MB → {out.relative_to(DATA)}", flush=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default="esc50,demand", help="esc50 · demand · rir（逗号分隔）")
    a = ap.parse_args()
    rec = load_record()
    try:
        if "esc50" in a.only:
            fetch_esc50(rec)
        if "demand" in a.only:
            fetch_demand(rec)
        if "rir" in a.only:
            fetch_rir(rec)
    finally:
        rec["updated"] = time.strftime("%Y-%m-%d %H:%M")
        rec["files"] = dict(sorted(rec["files"].items()))
        OUT.write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n")


if __name__ == "__main__":
    main()
