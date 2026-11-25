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
# - 替换前失败：只删本次临时文件，旧文件不动（BAD / LEFTOVER）
# - 替换那一步失败：可能已替换，按结果未知；也尝试删临时文件（MVFAIL / MVFAIL_LEFTOVER）
WRITE_SH = (
    'umask 077; d=$(dirname "$P"); '
    'if [ -d "$P" ]; then echo DIR; exit 0; fi; '
    'mkdir -p "$d" || { echo BAD; exit 0; }; '
    'case "$(readlink -f "$d")/" in /root/*) ;; *) echo OUTSIDE; exit 0;; esac; '
    't=$(mktemp "$d/.secret.XXXXXX") || { echo BAD; exit 0; }; '
    'if printf "%s=%s\\n" "$K" "$V" > "$t" '
    '&& [ "$(sha256sum "$t" | cut -d" " -f1)" = "$W" ] && [ "$(stat -c %a "$t")" = 600 ]; then '
    'if mv -fT "$t" "$P"; then echo OK; elif rm -f "$t"; then echo MVFAIL; else echo MVFAIL_LEFTOVER; fi; '
    'elif rm -f "$t"; then echo BAD; else echo LEFTOVER; fi'
)

RESULTS = {"DIR": "目标是个目录", "OUTSIDE": "目标目录实际不在 /root/ 下（可能链接到了网络盘）",
           "BAD": "写入或核对没通过，旧文件没动，本次临时文件已删",
           "LEFTOVER": "写入或核对没通过，而且本次临时文件没删掉（在目标同目录、.secret. 开头），请删机或手动删"}


def _read(name: str, env_file: Optional[str]) -> tuple:
    """返回 (值或 None, 路径)。解码等错误照常抛出，由调用方在安全边界里接住。"""
    path = env_file or os.path.join(main_repo_root(), ".env.local")
    try:
        with open(path, encoding="utf-8") as f:
            return parse_env_file(f.read()).get(name) or None, path
    except FileNotFoundError:
        return None, path


def _check_remote(remote: str) -> str:
    norm = posixpath.normpath(remote)
    if not remote.startswith("/") or norm != remote or not norm.startswith(CONTAINER_DIR):
        raise SystemExit(f"[REFUSED] 密钥只许放在容器盘 {CONTAINER_DIR} 下（网络盘会留下）：{remote}")
    return norm


def _int_or_tag(x) -> str:
    """状态码、退出码只认范围内的整数，其余一律换成固定标签（评审第 5 轮 #1：别把远端给的任意值拼进说明）。"""
    return str(x) if isinstance(x, int) and not isinstance(x, bool) and -1000 < x < 1000 else "非整数"


def _read_and_send(cli: Client, pod_id: str, name: str, env_file: Optional[str], remote: str) -> tuple:
    """读密钥、发出去、消化结果，全在这一个函数里；任何异常都在这里接住换成安全的
    (结果码, 说明, 位数)。密钥只存在于本函数的帧里，而本函数从不往外抛（评审第 1–4 轮异常带出密钥，改结构）。
    请求发出之后出的任何岔子（含响应格式不对）都按 UNKNOWN：远端可能已经替换（第 5 轮 #2）。"""
    sent = False
    try:
        value, path = _read(name, env_file)
        if value is None:
            return "MISSING", f"{path} 里没有 {name}，没发任何请求", 0
        n = len(value)
        want = hashlib.sha256(f"{name}={value}\n".encode()).hexdigest()
        try:
            sent = True
            r = O.exec_cmd(cli, pod_id, ["sh", "-c", WRITE_SH], env={"K": name, "V": value, "P": remote, "W": want})
        except BaseException as e:
            return "UNKNOWN", f"{type(e).__name__} 状态码 {_int_or_tag(getattr(e, 'status', None))}", n
        if r.get("timedOut"):
            return "UNKNOWN", "超时", n
        status = r.get("stdout")
        status = status.strip() if isinstance(status, str) else ""
        if r.get("exitCode") == 0 and status == "OK":
            return "OK", "", n
        if status in ("MVFAIL", "MVFAIL_LEFTOVER"):
            left = "，而且临时文件没删掉" if status == "MVFAIL_LEFTOVER" else "，临时文件已删"
            return "UNKNOWN", "替换那一步失败，可能已经替换了" + left, n
        if status in RESULTS:
            return status, RESULTS[status], n
        return "UNKNOWN", f"没有完成标记，退出码 {_int_or_tag(r.get('exitCode'))}", n
    except BaseException as e:  # 读文件出错（如坏 UTF-8）；或请求发出后响应格式不对
        return ("UNKNOWN", f"响应格式不对（{type(e).__name__}）", 0) if sent else ("ERROR", type(e).__name__, 0)


def put_secret(cli: Client, pod_id: str, name: str, env_file: Optional[str] = None,
               remote: str = DEFAULT_REMOTE) -> int:
    """把 env_file（默认主仓库 .env.local）里的 name 送进机器；返回位数。
    先校验键名和路径（这时还没读密钥），再读和发；报错只有 [结果码] 和中文说明。"""
    if not re.fullmatch(r"[A-Z][A-Z0-9_]{1,63}", name):
        raise SystemExit(f"[REFUSED] 键名只用大写字母、数字、下划线：{name!r}")
    remote = _check_remote(remote)
    code, why, n = _read_and_send(cli, pod_id, name, env_file, remote)
    if code == "OK":
        return n
    if code == "UNKNOWN":
        raise SystemExit(f"[{code}] 送 {name} 的结果未知（{why}）；细节不显示。先在机器里查 {remote} 的权限和位数再决定重送")
    raise SystemExit(f"[{code}] 送 {name} 没成：{why}；细节不显示")
