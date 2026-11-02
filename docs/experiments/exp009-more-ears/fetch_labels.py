"""exp009 · 下载公开众包评分的「纯文本标签」到仓外，记 sha256 进 data-manifest.json（进仓）。

不下任何音频：
- urgent-challenge/vmc2026-track1-meta（固定 revision）：各子集的 MOS 均值、逐票、预测器分数 .scp
- CHiME-7 UDASE 主观集的逐听众 CSV：HF 镜像整包 3.49 GB，用 HTTP Range 只抽 zip 里两个 CSV（约 1.7 MB）

用法：python3 docs/experiments/exp009-more-ears/fetch_labels.py   （只用标准库）
"""
import hashlib
import io
import json
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path.home() / "vdn-data/public-mos"
META_REPO = "urgent-challenge/vmc2026-track1-meta"
META_REV = "ebe09d8891b7dc1f77ddf1772e42bb22de3ef500"
META_SUBSETS = ["urgent2024-sqa/test", "urgent2025-sqa/test", "tmhint-qi/test", "chime-7-udase-eval/test"]
META_FILES = ["meta.jsonl", "mos_overall.scp", "utt2system", "utt2reference",
              "dnsmos_ovrl.scp", "utmos.scp", "scoreq.scp", "nisqa_mos.scp", "distill_mos.scp"]
CHIME_ZIP = "https://huggingface.co/datasets/urgent-challenge/urgent26_track2_sqa/resolve/main/CHiME-7-UDASE-evaluation-data.zip"
CHIME_MEMBERS = ["raw_results_listening_test.csv", "MOS_results_listening_test.csv"]


class RangeFile(io.RawIOBase):
    """只读、可 seek 的远端文件：每次 read 发一个 HTTP Range 请求。"""

    def __init__(self, url):
        self.url, self.pos = url, 0
        head = urllib.request.urlopen(urllib.request.Request(url, method="HEAD"), timeout=60)
        self.size = int(head.headers["Content-Length"])

    def readable(self):
        return True

    def seekable(self):
        return True

    def tell(self):
        return self.pos

    def seek(self, off, whence=0):
        self.pos = off if whence == 0 else self.pos + off if whence == 1 else self.size + off
        return self.pos

    def readinto(self, buf):
        if self.pos >= self.size:
            return 0
        end = min(self.pos + len(buf), self.size) - 1
        req = urllib.request.Request(self.url, headers={"Range": f"bytes={self.pos}-{end}"})
        data = urllib.request.urlopen(req, timeout=120).read()
        buf[:len(data)] = data
        self.pos += len(data)
        return len(data)


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main():
    man = {"meta": {"repo": META_REPO, "revision": META_REV, "files": {}}, "chime7": {"zip": CHIME_ZIP, "files": {}}}
    base = ROOT / "vmc2026-track1-meta" / META_REV
    for s in META_SUBSETS:
        for f in META_FILES:
            dst = base / s / f
            if not dst.exists():
                dst.parent.mkdir(parents=True, exist_ok=True)
                url = f"https://huggingface.co/datasets/{META_REPO}/resolve/{META_REV}/{s}/{f}"
                try:
                    dst.write_bytes(urllib.request.urlopen(url, timeout=120).read())
                except Exception as e:  # 有的子集没有 meta.jsonl（没有逐票）
                    print("缺", s, f, e)
                    continue
            man["meta"]["files"][f"{s}/{f}"] = {"bytes": dst.stat().st_size, "sha256": sha(dst)}
    cdir = ROOT / "chime7-udase-listening"
    cdir.mkdir(parents=True, exist_ok=True)
    need = [m for m in CHIME_MEMBERS if not (cdir / m).exists()]
    rf = RangeFile(CHIME_ZIP)
    man["chime7"]["zip_bytes"] = rf.size
    if need:
        z = zipfile.ZipFile(io.BufferedReader(rf, buffer_size=1 << 20))
        for info in z.infolist():
            name = info.filename.rsplit("/", 1)[-1]
            if name in need and "listening_test" in info.filename:
                (cdir / name).write_bytes(z.read(info))
                man["chime7"].setdefault("members", {})[name] = info.filename
    for m in CHIME_MEMBERS:
        man["chime7"]["files"][m] = {"bytes": (cdir / m).stat().st_size, "sha256": sha(cdir / m)}
    out = Path(__file__).resolve().parent / "data-manifest.json"
    out.write_text(json.dumps(man, ensure_ascii=False, indent=1))
    print("写入", out)


if __name__ == "__main__":
    main()
