"""把一个密钥送进机器的容器盘（大总管批，2026-11-25；方案见 plan-eval-baseline.md 第一节）。

- 值在本机进程里从 .env.local 读出，不经命令行参数、不落临时文件。
- 值放进 exec 请求体的 env 字段；机器里的命令只引用变量 $V，所以 exec 的 command 和机器里 ps 看到的参数都没有值。
- 机器里 umask 077 写到容器盘（默认 /root/.env.local），权限 600；在机器里核 sha256 和权限，只回 OK / BAD。
- 只报「已写入，N 位」。出错时不回传任何远端正文和原始异常，只报类型、状态码、退出码。
残余风险（已由大总管接受）：值经 HTTPS 交给 RunPod 的 exec 接口，平台会不会记录请求里的 env 不知道。
"""

from __future__ import annotations

import hashlib
import os
import posixpath
import re
from typing import Optional

import rp_ops as O
from rp_client import Client, main_repo_root, parse_env_file

DEFAULT_REMOTE = "/root/.env.local"
CONTAINER_DIR = "/root/"  # 容器盘：删机时随容器一起没；网络盘 /workspace 不许放

# 机器里跑的命令：只引用变量，不含值（评审 secret-ops 第 1 轮 #2、#4 后重写）
# - 目标是目录、或父目录实际落在 /root/ 之外（比如链接到网络盘）：拒绝，什么都不写
# - mktemp 独占创建临时文件（600，不跟随已有链接）；先在临时文件上核哈希与权限，对了才 mv -T 原子替换
# - 任何一步失败只删本次的临时文件，旧文件不动
WRITE_SH = (
    'umask 077; d=$(dirname "$P"); '
    'if [ -d "$P" ]; then echo DIR; exit 0; fi; '
    'mkdir -p "$d" || { echo BAD; exit 0; }; '
    'case "$(readlink -f "$d")/" in /root/*) ;; *) echo OUTSIDE; exit 0;; esac; '
    't=$(mktemp "$d/.secret.XXXXXX") || { echo BAD; exit 0; }; '
    'if printf "%s=%s\\n" "$K" "$V" > "$t" '
    '&& [ "$(sha256sum "$t" | cut -d" " -f1)" = "$W" ] && [ "$(stat -c %a "$t")" = 600 ] '
    '&& mv -fT "$t" "$P"; then echo OK; else rm -f "$t"; echo BAD; fi'
)

RESULTS = {"DIR": "目标是个目录", "OUTSIDE": "目标目录实际不在 /root/ 下（可能链接到了网络盘）",
           "BAD": "写入或核对没通过，旧文件没动，本次临时文件已删"}


def read_local_secret(name: str, env_file: Optional[str] = None) -> str:
    path = env_file or os.path.join(main_repo_root(), ".env.local")
    try:
        with open(path, encoding="utf-8") as f:
            value = parse_env_file(f.read()).get(name, "")
    except FileNotFoundError:
        value = ""
    if not value:
        raise SystemExit(f"{path} 里没有 {name}，没发任何请求")
    return value


def _check_remote(remote: str) -> str:
    norm = posixpath.normpath(remote)
    if not remote.startswith("/") or norm != remote or not norm.startswith(CONTAINER_DIR):
        raise SystemExit(f"密钥只许放在容器盘 {CONTAINER_DIR} 下（网络盘会留下）：{remote}")
    return norm


def put_secret(cli: Client, pod_id: str, name: str, value: str, remote: str = DEFAULT_REMOTE) -> int:
    """写入并核对；返回值的位数。"""
    if not re.fullmatch(r"[A-Z][A-Z0-9_]{1,63}", name):
        raise SystemExit(f"键名只用大写字母、数字、下划线：{name!r}")
    remote = _check_remote(remote)
    want = hashlib.sha256(f"{name}={value}\n".encode()).hexdigest()
    # 报错里一律不带远端正文与原始异常（评审第 1 轮 #1、#3）：只给异常类型、状态码、退出码；
    # 异常在 except 块里换成一句话，出了块再抛，异常链里没有原始异常
    failed = None
    try:
        r = O.exec_cmd(cli, pod_id, ["sh", "-c", WRITE_SH], env={"K": name, "V": value, "P": remote, "W": want})
    except BaseException as e:
        failed = f"{type(e).__name__} {getattr(e, 'status', '')}".strip()
    if failed:
        raise SystemExit(f"送 {name} 的结果未知（{failed}）；细节不显示，免得带出密钥。先在机器里查 {remote} 的权限和位数再决定重送")
    if r.get("timedOut"):
        raise SystemExit(f"送 {name} 超时，结果未知；先在机器里查 {remote} 的权限和位数再决定重送")
    status = (r.get("stdout") or "").strip()
    if r.get("exitCode") != 0 or status != "OK":
        why = RESULTS.get(status, f"退出码 {r.get('exitCode')}")
        raise SystemExit(f"送 {name} 没成：{why}；细节不显示，免得带出密钥")
    return len(value)
