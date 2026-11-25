"""split.py 用的逐文件读数缓存（sha256 · 帧数 · 采样率），放在网络盘上，重跑时不再逐个重算。

为什么不复用下载阶段的哈希：fetch.py 核的是整个压缩包的哈希，包里每个文件没有单独的哈希
（LibriSpeech 是 tar.gz，压根不带逐文件校验），所以逐文件 sha256 只能解压后算一次。
这里让它只算一次：键是 (相对路径, 字节数, mtime_ns)，三样都对上才用旧值；文件被改过就重算。
写法：每算完一个文件追加一行 JSON，中途被杀也保住已算的部分。
"""
import hashlib
import io
import json
import threading
from pathlib import Path

import soundfile as sf

SMALL = 64 << 20  # 小于 64 MB 的文件读一次进内存，哈希与读表头共用（网络盘上少开一次文件）


def measure(path):
    """读一遍文件：返回 sha256、帧数、采样率。"""
    p = Path(path)
    if p.stat().st_size < SMALL:
        data = p.read_bytes()
        i = sf.info(io.BytesIO(data))
        return hashlib.sha256(data).hexdigest(), int(i.frames), int(i.samplerate)
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    i = sf.info(str(p))
    return h.hexdigest(), int(i.frames), int(i.samplerate)


class FileCache:
    def __init__(self, path):
        self.path = Path(path) if path else None
        self.rows, self.lock, self.hits, self.misses = {}, threading.Lock(), 0, 0
        if self.path and self.path.exists():
            for line in self.path.read_text().splitlines():
                try:
                    r = json.loads(line)
                except json.JSONDecodeError:
                    continue  # 上次被杀时写了半行
                self.rows[r["path"]] = r

    def info(self, root, rel):
        p = Path(root) / rel
        st = p.stat()
        old = self.rows.get(rel)
        if old and old["bytes"] == st.st_size and old["mtime_ns"] == st.st_mtime_ns:
            with self.lock:
                self.hits += 1
            return {k: old[k] for k in ("path", "bytes", "sha256", "frames", "sr")}
        sha, frames, sr = measure(p)
        r = {"path": rel, "bytes": st.st_size, "mtime_ns": st.st_mtime_ns, "sha256": sha, "frames": frames, "sr": sr}
        with self.lock:
            self.misses += 1
            self.rows[rel] = r
            if self.path:
                self.path.parent.mkdir(parents=True, exist_ok=True)
                with open(self.path, "a") as f:
                    f.write(json.dumps(r, ensure_ascii=False) + "\n")
        return {k: r[k] for k in ("path", "bytes", "sha256", "frames", "sr")}
