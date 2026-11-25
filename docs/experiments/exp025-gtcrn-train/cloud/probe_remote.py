"""在云机器里跑：量 vCPU、内存、显卡、到各数据源的下载速度。只用标准库，输出一行 JSON。

用法（由 rp.py probe 上传后执行）：python3 probe_remote.py <url 文件> [每个地址最多秒数]
每个地址只读前若干秒，不落盘，测完即止。
"""

import json
import os
import subprocess
import sys
import time
import urllib.request


def measure(url: str, max_s: float, max_bytes: int = 2 << 30) -> dict:
    t0 = time.monotonic()
    got = 0
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "vdn-probe/1"})
        with urllib.request.urlopen(req, timeout=20) as r:
            first = time.monotonic() - t0
            while time.monotonic() - t0 < max_s and got < max_bytes:
                b = r.read(1 << 20)
                if not b:
                    break
                got += len(b)
        dt = time.monotonic() - t0
        return {"url": url, "ok": True, "bytes": got, "seconds": round(dt, 2),
                "first_byte_s": round(first, 2), "MB_per_s": round(got / dt / 1e6, 2) if dt else None}
    except Exception as e:  # 记下失败原因，不中断其他地址
        return {"url": url, "ok": False, "error": f"{type(e).__name__}: {e}"[:200],
                "bytes": got, "seconds": round(time.monotonic() - t0, 2)}


def sh(cmd: str) -> str:
    try:
        return subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30).stdout.strip()
    except Exception as e:
        return f"error: {e}"


def main(argv):
    urls = [u.strip() for u in open(argv[1]) if u.strip() and not u.startswith("#")]
    max_s = float(argv[2]) if len(argv) > 2 else 20.0
    mem_kb = 0
    for line in open("/proc/meminfo"):
        if line.startswith("MemTotal:"):
            mem_kb = int(line.split()[1])
    out = {
        "nproc": os.cpu_count(),
        "sched_affinity": len(os.sched_getaffinity(0)),
        "cgroup_cpu_max": sh("cat /sys/fs/cgroup/cpu.max 2>/dev/null"),
        "mem_gb": round(mem_kb / 1e6, 1),
        "gpu": sh("nvidia-smi --query-gpu=name,memory.total,driver_version --format=csv,noheader"),
        "workspace_df": sh("df -h /workspace 2>/dev/null | tail -1"),
        "downloads": [measure(u, max_s) for u in urls],
    }
    print(json.dumps(out, ensure_ascii=False))


if __name__ == "__main__":
    main(sys.argv)
