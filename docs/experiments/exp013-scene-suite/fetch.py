"""exp013 · 下载评测素材（大总管 2026-11-03 批：ESC-50 约 170 段 + DEMAND TCAR / TMETRO / PSTATION）。

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


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", default="esc50,demand")
    a = ap.parse_args()
    rec = load_record()
    try:
        if "esc50" in a.only:
            fetch_esc50(rec)
        if "demand" in a.only:
            fetch_demand(rec)
    finally:
        rec["updated"] = time.strftime("%Y-%m-%d %H:%M")
        rec["files"] = dict(sorted(rec["files"].items()))
        OUT.write_text(json.dumps(rec, ensure_ascii=False, indent=1) + "\n")


if __name__ == "__main__":
    main()
