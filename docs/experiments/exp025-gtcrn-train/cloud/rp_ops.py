"""云训练机的操作：选机房 · 建盘 · 开机（挂盘）· 跑命令 · 传文件 · 删机 · 记账。

所有花钱的动作（建盘、开机）由 CLI 要求显式 --yes；开机前先过 ledger.check_budget。
删机一律 DELETE（不是 stop：stop 后机器自带盘照收钱，034 compute.md 1.2）。
"""

from __future__ import annotations

import base64
import hashlib
import shlex
import time
from typing import Callable, Optional

import config as C
import ledger as L
import watchdog as W
from rp_client import ApiError, Client

RUNNING = "RUNNING"
DEAD = ("EXITED", "ERROR", "TERMINATED")
QUEUE_STATES = ("QUEUED", "DEPLOYED", "FAILED", "EXPIRED", "CANCELLED")  # openapi 202 响应的状态


# ---------- 选机房 ----------

def gpu_offer(cli: Client, gpu_id: str = C.GPU_TYPE_ID) -> dict:
    """该卡在 Secure Cloud 的价格、最低 vCPU 与各机房库存。"""
    j = cli.get("/v2/catalog/gpus", include="AVAILABILITY", product="POD", cloud=C.CLOUD)
    for g in j.get("gpus", []):
        if g.get("id") == gpu_id:
            return g
    raise SystemExit(f"目录里没有 {gpu_id}")


def candidate_datacenters(cli: Client, gpu_id: str = C.GPU_TYPE_ID) -> list:
    """有该卡库存、且支持网络盘的机房，按库存高到低排。"""
    rank = {"HIGH": 3, "MEDIUM": 2, "LOW": 1, "NONE": 0}
    j = cli.get("/v2/catalog/datacenters", include="GPU_AVAILABILITY")
    out = []
    for dc in j.get("dataCenters", []):
        if not dc.get("networkVolumeTypes"):
            continue
        for g in dc.get("gpuAvailability") or []:
            if g.get("id") == gpu_id and rank.get(g.get("availability"), 0) > 0:
                out.append({"id": dc["id"], "name": dc.get("name", ""), "region": dc.get("region", ""),
                            "availability": g["availability"],
                            "volumeTypes": dc.get("networkVolumeTypes")})
    out.sort(key=lambda d: (-rank[d["availability"]], d["id"]))
    return out


def cpu_rate(cli: Client) -> float:
    """纯 CPU 机器每小时目录价 = 每核价 × 核数。"""
    for c in cli.get("/v2/catalog/cpus").get("cpus", []):
        if c.get("id") == C.CPU_FLAVOR_ID:
            return float((c.get("price") or {}).get("securePerVcpu") or 0) * C.CPU_VCPU
    raise SystemExit(f"目录里没有 {C.CPU_FLAVOR_ID}")


# ---------- 网络盘 ----------

def find_volume(cli: Client, name: str = C.VOLUME_NAME) -> Optional[dict]:
    for v in cli.get("/v2/network-volumes").get("networkVolumes", []):
        if v.get("name") == name:
            return v
    return None


def create_volume(cli: Client, data_center: str, size_gb: int = C.VOLUME_SIZE_GB,
                  name: str = C.VOLUME_NAME, ledger_path: str = L.DEFAULT_PATH) -> dict:
    if find_volume(cli, name):
        raise SystemExit(f"已有同名网络盘 {name}，不重复建")
    v = cli.post("/v2/network-volumes", {"name": name, "size": size_gb, "dataCenter": data_center})
    now = L.iso(L.utcnow())
    L.append_row({"ts_utc": now, "action": "volume_create", "resource_id": v["id"],
                  "data_center": data_center, "usd_per_hr": f"{L.volume_hourly(size_gb):.6f}",
                  "started_utc": now, "note": f"{size_gb} GB ≈ {C.volume_usd_per_month(size_gb)} USD/月"},
                 ledger_path)
    return v


def delete_volume(cli: Client, volume_id: str, ledger_path: str = L.DEFAULT_PATH, note: str = "") -> None:
    """删网络盘并记账（盘里的数据会一起没，调用前确认）。"""
    cli.delete(f"/v2/network-volumes/{volume_id}")
    now = L.iso(L.utcnow())
    L.append_row({"ts_utc": now, "action": "volume_delete", "resource_id": volume_id,
                  "ended_utc": now, "note": note}, ledger_path)


# ---------- 开机 / 删机 ----------

def compute_spec(cpu: bool = False) -> dict:
    if cpu:
        return {"cpu": {"id": C.CPU_FLAVOR_ID, "vcpuCount": C.CPU_VCPU}}
    return {"gpu": {"id": C.GPU_TYPE_ID, "count": 1, "minVcpuCountPerGpu": C.MIN_VCPU_PER_GPU}}


def compute_label(cpu: bool = False) -> str:
    return f"{C.CPU_FLAVOR_ID} x{C.CPU_VCPU}" if cpu else C.GPU_TYPE_ID


def pod_body(name: str, data_center: str, volume_id: Optional[str], max_hours: float,
             cpu: bool = False) -> dict:
    # 容器睡到最长时限后删自己（第 2 层兜底，见 watchdog.py）；只是退出的话平台会重启它、照常计费
    body = {
        "name": name,
        "cloud": C.CLOUD,
        **compute_spec(cpu),
        "dataCenterIds": [data_center],
        "image": C.IMAGE,
        "disk": C.CONTAINER_DISK_GB,
        "cmd": W.pod_cmd(max_hours),
        "env": W.pod_env(),
        "ports": [],
    }
    if volume_id:
        body["mounts"] = {"network": [{"volumeId": volume_id, "path": C.VOLUME_MOUNT}]}
    return body


def create_pod(cli: Client, data_center: str, volume_id: Optional[str], max_hours: float,
               usd_per_hr_hint: float, name: Optional[str] = None,
               ledger_path: str = L.DEFAULT_PATH, cpu: bool = False) -> dict:
    if not name:
        name = C.POD_NAME_PREFIX + time.strftime("%Y%m%d-%H%M%S")
    if not name.startswith(C.POD_NAME_PREFIX):
        raise SystemExit(f"机器名必须以 {C.POD_NAME_PREFIX} 开头（reap 靠它认）")
    L.check_budget(L.read_rows(ledger_path), usd_per_hr_hint, max_hours)
    p = cli.post("/v2/pods", pod_body(name, data_center, volume_id, max_hours, cpu))
    if p.get("status") in QUEUE_STATES:  # 202：没货被排进队列
        if p.get("status") == "DEPLOYED" and p.get("podId"):
            p = cli.get(f"/v2/pods/{p['podId']}")
        else:
            try:
                cli.delete(f"/v2/queued-pods/{p['id']}")
                gone = "已取消队列项"
            except ApiError:
                gone = f"取消队列项 {p['id']} 失败，请到控制台取消"
            raise SystemExit(f"开机没直接开成（{p.get('status')}，{p.get('failureReason')}），{gone}")
    now = L.iso(L.utcnow())
    rate = float(p.get("cost") or usd_per_hr_hint)
    L.append_row({"ts_utc": now, "action": "pod_create", "resource_id": p["id"], "gpu": compute_label(cpu),
                  "data_center": p.get("dataCenterId") or data_center, "usd_per_hr": f"{rate:.4f}",
                  "started_utc": now, "note": f"max_hours={max_hours}"}, ledger_path)
    if rate * max_hours > usd_per_hr_hint * max_hours * 1.5 + 0.01:
        delete_pod(cli, p["id"], ledger_path, note="实际价远高于预估，立即删")
        raise SystemExit(f"实际每小时 {rate} 美元，远高于预估 {usd_per_hr_hint}，已删机")
    return p


def wait_running(cli: Client, pod_id: str, timeout_s: float = 900,
                 sleep: Callable[[float], None] = time.sleep, clock: Callable[[], float] = time.monotonic) -> dict:
    t0 = clock()
    while True:
        p = cli.get(f"/v2/pods/{pod_id}")
        if p.get("status") == RUNNING:
            return p
        if p.get("status") in DEAD:
            raise SystemExit(f"机器 {pod_id} 状态 {p.get('status')}")
        if clock() - t0 > timeout_s:
            raise SystemExit(f"机器 {pod_id} {timeout_s:.0f} 秒没跑起来（{p.get('status')}）")
        sleep(10)


def delete_pod(cli: Client, pod_id: str, ledger_path: str = L.DEFAULT_PATH, note: str = "") -> float:
    """删机并记账；返回估算花费。机器已经没了（404）也照记。"""
    try:
        cli.delete(f"/v2/pods/{pod_id}")
    except ApiError as e:
        if e.status != 404:
            raise
        note = (note + " 删时已不存在").strip()
    rows = L.read_rows(ledger_path)
    created = L.open_pods(rows).get(pod_id)
    now = L.utcnow()
    rate, hours, start = 0.0, 0.0, ""
    if created:
        start = created["started_utc"]
        rate = float(created["usd_per_hr"] or 0)
        hours = (now - L.parse_iso(start)).total_seconds() / 3600
    est = rate * hours
    L.append_row({"ts_utc": L.iso(now), "action": "pod_delete", "resource_id": pod_id,
                  "gpu": created.get("gpu", "") if created else "",
                  "data_center": created.get("data_center", "") if created else "",
                  "usd_per_hr": f"{rate:.4f}", "started_utc": start, "ended_utc": L.iso(now),
                  "hours": f"{hours:.4f}", "est_usd": f"{est:.4f}", "note": note}, ledger_path)
    W.unregister(pod_id)  # 删了就从本机看门狗登记表里去掉
    return est


def billed_usd(cli: Client, pod_id: str) -> Optional[float]:
    try:
        j = cli.get("/v2/billing/pods", podId=pod_id, bucketSize="day")
    except ApiError:
        return None
    t = (j.get("metadata") or {}).get("totals") or {}
    return t.get("totalAmount")


def list_our_pods(cli: Client) -> list:
    pods = cli.get("/v2/pods").get("pods", [])  # 一页默认 1000 台，够用
    return [p for p in pods if str(p.get("name", "")).startswith(C.POD_NAME_PREFIX)]


# ---------- 跑命令 ----------

def exec_cmd(cli: Client, pod_id: str, argv: list, timeout_s: int = 20, cwd: Optional[str] = None) -> dict:
    body = {"command": argv, "timeoutSeconds": max(1, min(int(timeout_s), C.EXEC_TIMEOUT_MAX_S))}
    if cwd:
        body["cwd"] = cwd
    return cli.post(f"/v2/pods/{pod_id}/exec", body)


def sh(cli: Client, pod_id: str, script: str, timeout_s: int = 20) -> dict:
    return exec_cmd(cli, pod_id, ["sh", "-c", script], timeout_s)


def check_ok(r: dict, what: str) -> str:
    if r.get("timedOut") or r.get("exitCode") != 0:
        raise SystemExit(f"{what} 失败：exit={r.get('exitCode')} timedOut={r.get('timedOut')} "
                         f"stderr={r.get('stderr', '')[-500:]}")
    return r.get("stdout", "")


def start_job(cli: Client, pod_id: str, script: str, job: str, jobs_dir: str = "/root/jobs") -> None:
    """长命令放后台跑（exec 单次最多等 25 秒）：输出写 <job>.log，退出码写 <job>.rc。"""
    d = shlex.quote(f"{jobs_dir}/{job}")
    inner = f"( {script} ) > {d}.log 2>&1; echo $? > {d}.rc"
    check_ok(sh(cli, pod_id, f"mkdir -p {shlex.quote(jobs_dir)} && rm -f {d}.rc && "
                              f"setsid nohup sh -c {shlex.quote(inner)} > /dev/null 2>&1 < /dev/null &"),
             f"启动 {job}")


def job_status(cli: Client, pod_id: str, job: str, jobs_dir: str = "/root/jobs", tail: int = 20) -> dict:
    d = shlex.quote(f"{jobs_dir}/{job}")
    out = check_ok(sh(cli, pod_id, f"cat {d}.rc 2>/dev/null || echo RUNNING; echo ---; tail -n {int(tail)} {d}.log 2>/dev/null"),
                   f"查 {job}")
    head, _, log = out.partition("---\n")
    head = head.strip()
    return {"done": head != "RUNNING", "rc": int(head) if head.lstrip("-").isdigit() else None, "log": log}


def wait_job(cli: Client, pod_id: str, job: str, timeout_s: float, poll_s: float = 15,
             sleep: Callable[[float], None] = time.sleep, clock: Callable[[], float] = time.monotonic) -> dict:
    t0 = clock()
    while True:
        st = job_status(cli, pod_id, job)
        if st["done"]:
            return st
        if clock() - t0 > timeout_s:
            raise SystemExit(f"{job} 超过 {timeout_s:.0f} 秒没完")
        sleep(poll_s)


# ---------- 传文件 ----------

def put_bytes(cli: Client, pod_id: str, data: bytes, remote: str) -> str:
    """把小文件传上去（每次 exec 最多约 240 × 4 KB base64）；返回远端 sha256 并核对。"""
    b64 = base64.b64encode(data).decode()
    step = C.EXEC_ARG_MAX_CHARS
    chunks = [b64[i:i + step] for i in range(0, len(b64), step)] or [""]
    q = shlex.quote(remote)
    check_ok(sh(cli, pod_id, f"mkdir -p \"$(dirname {q})\" && : > {q}.part"), "建远端文件")
    for i in range(0, len(chunks), C.PUT_ARGS_PER_CALL):
        part = chunks[i:i + C.PUT_ARGS_PER_CALL]
        r = exec_cmd(cli, pod_id, ["sh", "-c", f"printf %s \"$@\" | base64 -d >> {q}.part", "sh", *part])
        check_ok(r, f"上传第 {i // C.PUT_ARGS_PER_CALL + 1} 批")
    out = check_ok(sh(cli, pod_id, f"mv {q}.part {q} && sha256sum {q}"), "核哈希")
    got = out.split()[0] if out.split() else ""
    want = hashlib.sha256(data).hexdigest()
    if got != want:
        raise SystemExit(f"上传后哈希不一致：{remote}")
    return got


def fetch_bytes(cli: Client, pod_id: str, remote: str) -> bytes:
    """分块取回远端文件（每块 512 KB，base64 后 < 1 MiB 输出上限），核 sha256。"""
    q = shlex.quote(remote)
    out = check_ok(sh(cli, pod_id, f"stat -c %s {q} && sha256sum {q}"), f"查 {remote}").split()
    size, want = int(out[0]), out[1]
    buf = bytearray()
    n = C.FETCH_CHUNK_BYTES
    for i in range((size + n - 1) // n):
        r = sh(cli, pod_id, f"dd if={q} bs={n} skip={i} count=1 2>/dev/null | base64 -w0")
        if r.get("truncated"):
            raise SystemExit("取回时输出被截断")
        buf += base64.b64decode(check_ok(r, f"取第 {i + 1} 块"))
    if len(buf) != size or hashlib.sha256(buf).hexdigest() != want:
        raise SystemExit(f"取回后大小或哈希不一致：{remote}")
    return bytes(buf)
