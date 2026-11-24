"""防忘删机的两层兜底（只有删机才停止计费：容器到时限退出后平台会重启它，照常计费，2026-11-24 实测）。

第 1 层 · 本机看门狗：每次开机登记「删机时刻」= 开机 + 最长小时数 + 宽限；本机后台进程到点调 DELETE。
  本机睡眠或断网时不可靠（已用 caffeinate -i 防空闲睡眠，合盖仍会睡），所以还有第 2 层。
第 2 层 · 机器内自删：开机命令先睡到时限，再用平台注入机器的 RUNPOD_POD_ID 与 RUNPOD_API_KEY
  调 DELETE 删自己；失败每 60 秒重试。密钥由平台注入，不是我们的密钥，不进镜像、不进请求体。
"""

from __future__ import annotations

import datetime as dt
import json
import os
import shutil
import subprocess
import sys
import time
from typing import Callable, Optional

import config as C
import ledger as L

HERE = os.path.dirname(os.path.abspath(__file__))
WATCH_DIR = os.path.join(HERE, ".watch")  # cloud/.gitignore 忽略
DEADLINES = os.path.join(WATCH_DIR, "deadlines.json")
PIDFILE = os.path.join(WATCH_DIR, "watchdog.pid")
GRACE_S = 300  # 本机看门狗比机器内自删晚 5 分钟，自删失败时兜底

# 机器内自删脚本：只删 RUNPOD_POD_ID 这一台（URL 里只有自己的编号）。先试 v2 接口，再试 v1。
SELF_DELETE_PY = (
    "import os,sys,urllib.request as u\n"
    "pid=os.environ.get('RUNPOD_POD_ID');key=os.environ.get('RUNPOD_API_KEY')\n"
    "if not pid or not key: print('vdn-self-delete: no RUNPOD_POD_ID/RUNPOD_API_KEY',flush=True);sys.exit(2)\n"
    "for url in ('https://api.runpod.io/v2/pods/'+pid,'https://rest.runpod.io/v1/pods/'+pid):\n"
    "  try:\n"
    "    u.urlopen(u.Request(url,method='DELETE',headers={'Authorization':'Bearer '+key}),timeout=30)\n"
    "    print('vdn-self-delete: ok',url,flush=True);sys.exit(0)\n"
    "  except Exception as e: print('vdn-self-delete:',url,type(e).__name__,getattr(e,'code',''),flush=True)\n"
    "sys.exit(1)\n"
)


def pod_cmd(max_hours: float) -> list:
    """容器命令：睡到时限 → 自删；自删失败每 60 秒重试。"""
    secs = int(max_hours * 3600)
    return ["sh", "-c", f'sleep {secs}; while true; do python3 -c "$VDN_SELF_DELETE_PY" && sleep 600; sleep 60; done']


def pod_env() -> dict:
    return {"VDN_SELF_DELETE_PY": SELF_DELETE_PY}


# ---------- 第 1 层：本机看门狗 ----------

def _load(path: str = DEADLINES) -> dict:
    try:
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    except FileNotFoundError:
        return {}


def _save(d: dict, path: str = DEADLINES) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(d, f, indent=1)
    os.replace(tmp, path)


def register(pod_id: str, max_hours: float, now: Optional[dt.datetime] = None, path: str = DEADLINES) -> str:
    now = now or L.utcnow()
    when = L.iso(now + dt.timedelta(seconds=int(max_hours * 3600) + GRACE_S))
    d = _load(path)
    d[pod_id] = when
    _save(d, path)
    return when


def unregister(pod_id: str, path: str = DEADLINES) -> None:
    d = _load(path)
    if d.pop(pod_id, None) is not None:
        _save(d, path)


def tick(delete: Callable[[str], object], now: Optional[dt.datetime] = None, path: str = DEADLINES) -> list:
    """删掉所有到点的机器；返回删了哪些。删失败的留着下次再试。"""
    now = now or L.utcnow()
    done = []
    for pod_id, when in sorted(_load(path).items()):
        if L.parse_iso(when) <= now:
            try:
                delete(pod_id)
            except Exception as e:  # 断网等：下次再试
                print(f"watchdog: 删 {pod_id} 失败：{e}", file=sys.stderr, flush=True)
                continue
            unregister(pod_id, path)
            done.append(pod_id)
    return done


def run_loop(delete: Callable[[str], object], poll_s: float = 30, path: str = DEADLINES,
             sleep: Callable[[float], None] = time.sleep, now_fn: Callable[[], dt.datetime] = L.utcnow) -> None:
    """登记表空了就退出。"""
    while _load(path):
        for pid in tick(delete, now_fn(), path):
            print(f"watchdog: 到点已删 {pid}", flush=True)
        if _load(path):
            sleep(poll_s)


def alive(pidfile: str = PIDFILE) -> bool:
    try:
        with open(pidfile) as f:
            pid = int(f.read().strip())
        os.kill(pid, 0)
        return True
    except (FileNotFoundError, ValueError, ProcessLookupError, PermissionError):
        return False


def ensure_running(rp_path: str, spawn: Callable[..., object] = subprocess.Popen,
                   pidfile: str = PIDFILE) -> bool:
    """没有看门狗在跑就起一个（脱离本终端；有 caffeinate 就防空闲睡眠）。返回是否新起了一个。"""
    if alive(pidfile):
        return False
    os.makedirs(os.path.dirname(pidfile), exist_ok=True)
    argv = [sys.executable, rp_path, "watchdog"]
    if shutil.which("caffeinate"):
        argv = ["caffeinate", "-i"] + argv
    log = open(os.path.join(os.path.dirname(pidfile), "watchdog.log"), "a")
    p = spawn(argv, stdout=log, stderr=log, stdin=subprocess.DEVNULL, start_new_session=True)
    with open(pidfile, "w") as f:
        f.write(str(p.pid))
    return True


def assert_spec_safe() -> None:
    """自删脚本与命令要在 exec / 环境变量长度上限之内（openapi：env 值 ≤ 16384）。"""
    assert len(SELF_DELETE_PY) < 16384
    assert all(len(a) <= C.EXEC_ARG_MAX_CHARS for a in pod_cmd(1.0))
