"""费用账本 costs.tsv：每次开机 / 删机 / 建盘记一行；开机前检查不超上限。

估算口径：
- 机器：按 pod 的 `cost`（每小时美元）× 实际开机小时（删机时记）。有账单金额就以账单为准（billed_usd）。
- 网络盘：从建盘起按小时累计（每月价 ÷ 720 × 小时数），删盘后停止。
- 上限检查（check_budget）：已关机器 + 网络盘到今天 + 没删的盘往后一个月 + 开着的机器跑满时限
  + 本次最坏（每小时价 × (最长小时数 + 看门狗宽限)）≤ 上限。
- 局限：只在花钱前检查。机器都删了以后，网络盘仍按月收费；闲置多久删盘由人定（README「网络盘」）。
"""

from __future__ import annotations

import csv
import datetime as dt
import re
import os
from typing import Iterable, Optional

import config as C

GRACE_H = C.WATCHDOG_GRACE_S / 3600

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_PATH = os.path.join(HERE, "costs.tsv")


def utcnow() -> dt.datetime:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0)


def iso(t: Optional[dt.datetime]) -> str:
    return t.strftime("%Y-%m-%dT%H:%M:%SZ") if t else ""


def parse_iso(s: str) -> Optional[dt.datetime]:
    if not s:
        return None
    return dt.datetime.strptime(s, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=dt.timezone.utc)


def read_rows(path: str = DEFAULT_PATH) -> list:
    if not os.path.exists(path):
        return []
    with open(path, newline="", encoding="utf-8") as f:
        r = csv.DictReader(f, delimiter="\t")
        if tuple(r.fieldnames or ()) != C.COSTS_TSV_COLUMNS:
            raise ValueError(f"{path} 表头和 config.COSTS_TSV_COLUMNS 不一致")
        return list(r)


def append_row(row: dict, path: str = DEFAULT_PATH) -> None:
    extra = set(row) - set(C.COSTS_TSV_COLUMNS)
    if extra:
        raise ValueError(f"未知列：{sorted(extra)}")
    new = not os.path.exists(path) or os.path.getsize(path) == 0
    with open(path, "a", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=C.COSTS_TSV_COLUMNS, delimiter="\t", lineterminator="\n")
        if new:
            w.writeheader()
        w.writerow({k: row.get(k, "") for k in C.COSTS_TSV_COLUMNS})


def _f(s: str) -> float:
    return float(s) if s not in ("", None) else 0.0


def pod_spent(rows: Iterable[dict]) -> float:
    """已删机的花费：有账单用账单，否则用估算。"""
    total = 0.0
    for r in rows:
        if r["action"] == "pod_delete":
            total += _f(r["billed_usd"]) if r["billed_usd"] else _f(r["est_usd"])
    return total


def open_pods(rows: Iterable[dict]) -> dict:
    """开了还没记删除的机器：{pod_id: create 行}。"""
    live = {}
    for r in rows:
        if r["action"] == "pod_create":
            live[r["resource_id"]] = r
        elif r["action"] == "pod_delete":
            live.pop(r["resource_id"], None)
    return live


def volume_spent(rows: Iterable[dict], now: Optional[dt.datetime] = None) -> float:
    now = now or utcnow()
    created, total = {}, 0.0
    for r in rows:
        if r["action"] == "volume_create":
            created[r["resource_id"]] = (parse_iso(r["started_utc"]), _f(r["usd_per_hr"]))
        elif r["action"] == "volume_delete" and r["resource_id"] in created:
            t0, rate = created.pop(r["resource_id"])
            total += rate * (parse_iso(r["ended_utc"]) - t0).total_seconds() / 3600
    for t0, rate in created.values():
        total += rate * max((now - t0).total_seconds(), 0) / 3600
    return total


def live_pod_cost(rows: Iterable[dict], now: Optional[dt.datetime] = None) -> float:
    now = now or utcnow()
    return sum(_f(r["usd_per_hr"]) * max((now - parse_iso(r["started_utc"])).total_seconds(), 0) / 3600
               for r in open_pods(rows).values())


def spent_so_far(rows: list, now: Optional[dt.datetime] = None) -> float:
    return pod_spent(rows) + volume_spent(rows, now) + live_pod_cost(rows, now)


def _max_hours(row: dict) -> Optional[float]:
    m = re.search(r"max_hours=([0-9.]+)", row.get("note") or "")
    return float(m.group(1)) if m else None


def open_pods_worst(rows: Iterable[dict], now: Optional[dt.datetime] = None) -> float:
    """开着的机器按最坏算：跑满登记的最长时限（加看门狗宽限）；已超时的按实际已跑。没记时限的按已跑 + 1 小时。"""
    now = now or utcnow()
    total = 0.0
    for r in open_pods(rows).values():
        ran = max((now - parse_iso(r["started_utc"])).total_seconds(), 0) / 3600
        mh = _max_hours(r)
        cap_h = mh + GRACE_H if mh is not None else ran + 1.0
        total += _f(r["usd_per_hr"]) * max(ran, cap_h)
    return total


def volume_reserve(rows: Iterable[dict]) -> float:
    """还没删的网络盘：按往后 VOLUME_RESERVE_HOURS 小时的盘费预留（评审第 2 轮 #1）。盘不删就一直收钱。"""
    live = {}
    for r in rows:
        if r["action"] == "volume_create":
            live[r["resource_id"]] = _f(r["usd_per_hr"])
        elif r["action"] == "volume_delete":
            live.pop(r["resource_id"], None)
    return sum(live.values()) * C.VOLUME_RESERVE_HOURS


def check_budget(rows: list, usd_per_hr: float, max_hours: float,
                 cap: float = C.SPEND_CAP_USD, now: Optional[dt.datetime] = None) -> float:
    """返回本次开机后最坏总花费；超上限抛 SystemExit。
    最坏 = 已关机器 + 网络盘到今天 + 网络盘往后一个月 + 开着的机器跑满时限 + 本台跑满时限（含宽限）。"""
    done = pod_spent(rows) + volume_spent(rows, now)
    worst = done + volume_reserve(rows) + open_pods_worst(rows, now) + usd_per_hr * (max_hours + GRACE_H)
    if worst > cap:
        raise SystemExit(f"花费上限 {cap:.2f} 美元：已花约 {spent_so_far(rows, now):.2f}，"
                         f"盘费预留 {volume_reserve(rows):.2f}，开着的机器最坏 {open_pods_worst(rows, now):.2f}，本次最坏 {usd_per_hr * (max_hours + GRACE_H):.2f}，"
                         f"合计 {worst:.2f}，不开机")
    return worst


def volume_hourly(size_gb: int) -> float:
    return C.volume_usd_per_month(size_gb) / (30 * 24)
