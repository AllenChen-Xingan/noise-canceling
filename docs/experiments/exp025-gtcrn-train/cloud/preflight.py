"""显卡机开机检查（大总管定 2026-11-25：以后的 H100 开机都先跑它）。

起因（实测）：pytorch 镜像里没有 git，trainer 的运行记录要读提交号，没有 git 就拒跑；
2026-11-25 H100 乙段因此第 2 步才失败，白花约 0.6 美元。
2026-11-26 加第 1 步：网络盘剩的空间太少就不开训练（盘满时训练状态写不进去）。

做三件事，任一不对就报错（CLI 带 --down-on-fail 时同时删机）：
1. 网络盘剩的空间 = 配额（平台接口给的，不是常量）− `du -s /workspace`；少于 MIN_FREE_GB 就停
2. 没有 git 就装（apt-get）。装可能超过 exec 单次 25 秒，所以放后台任务里跑、按运行编号等
3. `git -C <仓库> rev-parse HEAD` 必须等于给定的完整提交号（40 位）
"""

from __future__ import annotations

import re
import shlex

import config as C
import rp_ops as O
from rp_client import Client

INSTALL_GIT = "command -v git >/dev/null || (apt-get update -qq && apt-get install -y -qq git)"
# 网络盘按 GB 数已用量；100+ GB 时 du 会超 exec 单次 25 秒，所以放后台任务
DU_CMD = f"du -s --block-size=1G {C.VOLUME_MOUNT}"


def _last_lines(st: dict) -> list:
    return [x.strip() for x in st["log"].splitlines() if x.strip()]


def check_free_space(cli: Client, pod_id: str, min_free_gb: int = C.MIN_FREE_GB, timeout_s: float = 600,
                     sleep=None) -> tuple:
    """看一眼网络盘：返回（已用 GB、配额 GB、剩 GB）；不够或读不出就抛 SystemExit。
    配额从平台接口读当下的盘大小（盘已扩到 200 GB，不能写死）。
    不用 df：这个网络盘的 df 报的是整个集群，不是我们的配额。"""
    v = O.find_volume(cli)
    if not v:
        raise SystemExit(f"[NO_VOLUME] 找不到网络盘 {C.VOLUME_NAME}，无从判断剩多少空间")
    quota = int(float(v.get("size") or 0))
    run_id = O.start_job(cli, pod_id, DU_CMD, "diskcheck")
    kw = {"sleep": sleep} if sleep else {}
    st = O.wait_job(cli, pod_id, run_id, timeout_s, poll_s=5, **kw)
    lines = _last_lines(st)
    m = re.match(r"\d+", lines[-1]) if lines else None
    if st["rc"] != 0 or not m:
        raise SystemExit(f"[NO_DU] 读不到 {C.VOLUME_MOUNT} 已用空间（退出码 {st['rc']}）："
                         f"{' | '.join(lines[-3:])[:200]}")
    used = int(m.group(0))
    free = quota - used
    if free < min_free_gb:
        raise SystemExit(f"[LOW_DISK] 网络盘剩 {free} GB，少于 {min_free_gb} GB，不开训练"
                         f"（已用 {used} GB，配额 {quota} GB）；先清盘或扩盘再开")
    return used, quota, free


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
