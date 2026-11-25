"""把一个密钥送进机器的容器盘（大总管批，2026-11-25；方案见 plan-eval-baseline.md 第一节）。

- 值在本机进程里从 .env.local 读出，不经命令行参数、不落临时文件。
- 值放进 exec 请求体的 env 字段；机器里的命令只引用变量 $V，所以 exec 的 command 和机器里 ps 看到的参数都没有值。
- 机器里 umask 077 写到容器盘（默认 /root/.env.local），权限 600；在机器里核 sha256 和权限，只回 OK / BAD。
- 只报「已写入，N 位」。出错时把值和它的 base64 从错误信息里换成 <secret>。
残余风险（已由大总管接受）：值经 HTTPS 交给 RunPod 的 exec 接口，平台会不会记录请求里的 env 不知道。
"""

from __future__ import annotations

import base64
import hashlib
import os
import posixpath
import re
from typing import Optional

import rp_ops as O
from rp_client import Client, main_repo_root, parse_env_file

DEFAULT_REMOTE = "/root/.env.local"
CONTAINER_DIR = "/root/"  # 容器盘：删机时随容器一起没；网络盘 /workspace 不许放

# 机器里跑的命令：只引用变量，不含值
WRITE_SH = (
    'umask 077; mkdir -p "$(dirname "$P")" && '
    'printf "%s=%s\\n" "$K" "$V" > "$P.part" && chmod 600 "$P.part" && mv "$P.part" "$P" && '
    '[ "$(sha256sum "$P" | cut -d" " -f1)" = "$W" ] && [ "$(stat -c %a "$P")" = 600 ] '
    '&& echo OK || { rm -f "$P.part" "$P"; echo BAD; }'
)


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


def _redact(text: str, value: str) -> str:
    for form in (value, base64.b64encode(value.encode()).decode().rstrip("="),
                 base64.urlsafe_b64encode(value.encode()).decode().rstrip("=")):
        text = text.replace(form, "<secret>")
    return text


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
    try:
        r = O.exec_cmd(cli, pod_id, ["sh", "-c", WRITE_SH], env={"K": name, "V": value, "P": remote, "W": want})
    except BaseException as e:
        raise SystemExit(_redact(f"送 {name} 失败：{e}", value)) from None
    if r.get("exitCode") != 0 or r.get("stdout", "").strip() != "OK":
        detail = _redact(f"exit={r.get('exitCode')} stdout={r.get('stdout', '')[-120:]} "
                         f"stderr={r.get('stderr', '')[-200:]}", value)
        raise SystemExit(f"送 {name} 没核对上，机器里没留下半截文件：{detail}")
    return len(value)
