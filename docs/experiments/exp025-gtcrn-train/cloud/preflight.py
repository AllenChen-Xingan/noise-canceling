"""显卡机开机检查（大总管定 2026-11-25：以后的 H100 开机都先跑它）。

起因（实测）：pytorch 镜像里没有 git，trainer 的运行记录要读提交号，没有 git 就拒跑；
2026-11-25 H100 乙段因此第 2 步才失败，白花约 0.6 美元。

做两件事，任一不对就报错（CLI 带 --down-on-fail 时同时删机）：
1. 没有 git 就装（apt-get）。装可能超过 exec 单次 25 秒，所以放后台任务里跑、按运行编号等
2. `git -C <仓库> rev-parse HEAD` 必须等于给定的完整提交号（40 位）
"""

from __future__ import annotations

import re
import shlex

import rp_ops as O
from rp_client import Client

INSTALL_GIT = "command -v git >/dev/null || (apt-get update -qq && apt-get install -y -qq git)"


def check_commit(cli: Client, pod_id: str, repo: str, commit: str, timeout_s: float = 300,
                 sleep=None) -> str:
    """装好 git 并核提交号；返回读到的提交号。不对就抛 SystemExit（不删机，删不删由调用方定）。"""
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise SystemExit(f"[REFUSED] 提交号要写完整的 40 位小写十六进制：{commit!r}")
    script = f"{INSTALL_GIT} >/dev/null 2>&1; git -C {shlex.quote(repo)} rev-parse HEAD"
    run_id = O.start_job(cli, pod_id, script, "preflight")
    kw = {"sleep": sleep} if sleep else {}
    st = O.wait_job(cli, pod_id, run_id, timeout_s, poll_s=5, **kw)
    lines = [x.strip() for x in st["log"].splitlines() if x.strip()]
    got = lines[-1] if lines else ""
    if st["rc"] != 0 or not re.fullmatch(r"[0-9a-f]{40}", got):
        raise SystemExit(f"[NO_GIT] 装 git 或读提交号失败（退出码 {st['rc']}）：{' | '.join(lines[-3:])[:200]}")
    if got != commit:
        raise SystemExit(f"[WRONG_COMMIT] {repo} 的提交号是 {got}，不是要的 {commit}")
    return got
