"""云训练机的操作：选机房 · 建盘 · 开机（挂盘）· 跑命令 · 传文件 · 删机 · 记账。

所有花钱的动作（建盘、开机）由 CLI 要求显式 --yes；开机前先过 ledger.check_budget。
删机一律 DELETE（不是 stop：stop 后机器自带盘照收钱，034 compute.md 1.2）。
"""

from __future__ import annotations

import base64
import hashlib
import json
import os
import re
import secrets
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
    with L.budget_lock(ledger_path):
        return _create_volume_locked(cli, data_center, size_gb, name, ledger_path)


def _create_volume_locked(cli: Client, data_center: str, size_gb: int, name: str, ledger_path: str) -> dict:
    if find_volume(cli, name):
        raise SystemExit(f"已有同名网络盘 {name}，不重复建")
    L.check_budget(check_ledger_covers_cloud(cli, ledger_path), L.volume_hourly(size_gb), C.VOLUME_RESERVE_HOURS)
    v = cli.post("/v2/network-volumes", {"name": name, "size": size_gb, "dataCenter": data_center}, retries=0)
    try:  # 刚建的空盘：写账失败就删掉，免得有盘没账（评审第 2 轮 #3）
        now = L.iso(L.utcnow())
        L.append_row({"ts_utc": now, "action": "volume_create", "resource_id": v["id"],
                      "data_center": data_center, "usd_per_hr": f"{L.volume_hourly(size_gb):.6f}",
                      "started_utc": now, "note": f"{size_gb} GB ≈ {C.volume_usd_per_month(size_gb)} USD/月"},
                     ledger_path)
    except BaseException as e:
        try:
            cli.delete(f"/v2/network-volumes/{v['id']}")
        except BaseException as e2:
            raise SystemExit(f"{e}；而且删空盘 {v['id']} 失败（{e2}），请到控制台删") from e
        raise
    return v


def resize_volume(cli: Client, new_gb: int, name: str = C.VOLUME_NAME, ledger_path: str = L.DEFAULT_PATH) -> dict:
    """扩网络盘（只能变大，平台规定）。花钱：和建盘一样过账本锁、账本核对、花费上限（多出的 GB 预留一个月）。"""
    with L.budget_lock(ledger_path):
        rows = check_ledger_covers_cloud(cli, ledger_path)
        v = find_volume(cli, name)
        if not v:
            raise SystemExit(f"没有网络盘 {name}")
        cur = int(v.get("size") or 0)
        if not (cur < new_gb <= C.VOLUME_MAX_GB):
            raise SystemExit(f"[REFUSED] 新大小要大于现在的 {cur} GB、且不超过 {C.VOLUME_MAX_GB} GB：{new_gb}")
        L.check_budget(rows, L.volume_hourly(new_gb - cur), C.VOLUME_RESERVE_HOURS)
        out = cli.patch(f"/v2/network-volumes/{v['id']}", {"size": int(new_gb)})
        now = L.iso(L.utcnow())
        L.append_row({"ts_utc": now, "action": "volume_resize", "resource_id": v["id"],
                      "data_center": v.get("dataCenter", ""), "usd_per_hr": f"{L.volume_hourly(new_gb):.6f}",
                      "started_utc": now, "note": f"{cur} → {new_gb} GB ≈ {C.volume_usd_per_month(new_gb)} USD/月"},
                     ledger_path)
        return out or {}


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
        "image": C.CPU_IMAGE if cpu else C.IMAGE,
        "disk": C.CONTAINER_DISK_GB,
        "cmd": W.pod_cmd(max_hours),
        "env": W.pod_env(),
        "ports": [],
    }
    if volume_id:
        body["mounts"] = {"network": [{"volumeId": volume_id, "path": C.VOLUME_MOUNT}]}
    return body


def check_ledger_covers_cloud(cli: Client, ledger_path: str) -> list:
    """账本要认得云上我们的东西，否则花费上限会按 0 算（评审第 3 轮 #3）：
    云上有 vdn- 机器或本项目网络盘、账本里却没有它的记录，就拒绝花钱。返回账本行。"""
    if not os.path.exists(ledger_path):  # 评审第 4 轮 #4：新路径 = 没有历史花费，不能按 0 放行
        raise SystemExit(f"账本 {ledger_path} 不存在；正式账本是 cloud/costs.tsv。确要新开一本，先用 ledger.init 建表头")
    rows = L.read_rows(ledger_path)
    known = {r["resource_id"] for r in rows if r["action"] in ("pod_create", "volume_create")}
    v = find_volume(cli)
    missing = [x["id"] for x in list_our_pods(cli) if x["id"] not in known]
    if v and v["id"] not in known:
        missing.append(v["id"])
    if missing:
        raise SystemExit(f"账本 {ledger_path} 里没有云上这些资源的记录：{', '.join(missing)}；"
                         f"账本路径可能错了（默认 cloud/costs.tsv），不花钱")
    return rows


def create_pod(cli: Client, data_center: str, volume_id: Optional[str], max_hours: float,
               usd_per_hr_hint: float, name: Optional[str] = None,
               ledger_path: str = L.DEFAULT_PATH, cpu: bool = False) -> dict:
    with L.budget_lock(ledger_path):
        return _create_pod_locked(cli, data_center, volume_id, max_hours, usd_per_hr_hint, name, ledger_path, cpu)


def _create_pod_locked(cli: Client, data_center: str, volume_id: Optional[str], max_hours: float,
                       usd_per_hr_hint: float, name: Optional[str], ledger_path: str, cpu: bool) -> dict:
    if not name:
        # 时间 + 8 位随机：同一秒开两台也不重名；结果未知时按名字清理只会碰到这一次建的（评审第 4 轮 #1）
        name = C.POD_NAME_PREFIX + time.strftime("%Y%m%d-%H%M%S-") + secrets.token_hex(4)
    if not name.startswith(C.POD_NAME_PREFIX):
        raise SystemExit(f"机器名必须以 {C.POD_NAME_PREFIX} 开头（reap 靠它认）")
    L.check_budget(check_ledger_covers_cloud(cli, ledger_path), usd_per_hr_hint, max_hours)
    # 建机不自动重试：回 503 时机器可能已经建了，重试会多开一台（评审第 1 轮 #4）
    try:
        p = cli.post("/v2/pods", pod_body(name, data_center, volume_id, max_hours, cpu), retries=0)
    except BaseException as e:
        if isinstance(e, ApiError) and 400 <= e.status < 500:
            raise  # 明确拒了：没建
        _cleanup_unknown_create(cli, name, ledger_path, e)  # 503 / 断网：结果未知（评审第 3 轮 #1）
        raise
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
    try:  # 机器已在计费：之后任何一步出错都先删机再报错（评审第 1 轮 #5）
        now = L.iso(L.utcnow())
        rate = float(p.get("cost") or usd_per_hr_hint)
        L.append_row({"ts_utc": now, "action": "pod_create", "resource_id": p["id"], "gpu": compute_label(cpu),
                      "data_center": p.get("dataCenterId") or data_center, "usd_per_hr": f"{rate:.4f}",
                      "started_utc": now, "note": f"max_hours={max_hours}"}, ledger_path)
        if rate > usd_per_hr_hint * C.PRICE_TOLERANCE:  # 和目录价检查同一个容差（评审第 1 轮 #3）
            raise SystemExit(f"实际每小时 {rate} 美元，高于预估 {usd_per_hr_hint} 的 {C.PRICE_TOLERANCE} 倍")
        others = [r for r in L.read_rows(ledger_path) if r.get("resource_id") != p["id"]]
        L.check_budget(others, rate, max_hours)  # 按实际价重算一遍
    except BaseException as e:
        try:
            delete_pod(cli, p["id"], ledger_path, note=f"开机后检查不过，立即删：{str(e)[:80]}")
        except BaseException as e2:
            raise SystemExit(f"{e}；而且删机 {p['id']} 失败（{e2}），请马上 rp.py down {p['id']} --yes 或 rp.py reap --yes") from e
        raise
    return p


def _cleanup_unknown_create(cli: Client, name: str, ledger_path: str, err: BaseException) -> None:
    """建机请求结果未知：按这次的唯一机器名查云上，查到就删；查不到也报「结果未知」。"""
    try:
        found = [x["id"] for x in list_our_pods(cli) if x.get("name") == name]
        for pid in found:
            delete_pod(cli, pid, ledger_path, note="建机结果未知，按名字查到后删")
    except BaseException as e2:
        raise SystemExit(f"建机结果未知（{err}），按名字 {name} 核查也失败（{e2}）；请稍后跑 rp.py reap --yes") from err
    raise SystemExit(f"建机结果未知（{err}）；按名字 {name} 查到 {len(found)} 台，已删") from err


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


def wait_exec_ready(cli: Client, pod_id: str, timeout_s: float = 900,
                    sleep: Callable[[float], None] = time.sleep,
                    clock: Callable[[], float] = time.monotonic) -> None:
    """状态是 RUNNING 不等于容器起好了（拉镜像要几分钟，期间 exec 回 409）：等到 exec 能跑通。"""
    t0 = clock()
    while True:
        try:
            if exec_cmd(cli, pod_id, ["true"], 10).get("exitCode") == 0:
                return
        except ApiError as e:
            if e.status not in (409, 503):
                raise
        if clock() - t0 > timeout_s:
            raise SystemExit(f"机器 {pod_id} {timeout_s:.0f} 秒内容器没起好")
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

def exec_cmd(cli: Client, pod_id: str, argv: list, timeout_s: int = 20, cwd: Optional[str] = None,
             env: Optional[dict] = None) -> dict:
    """env 走请求体的 env 字段，不进命令参数（机器里 ps 看不到）；secret_ops 送密钥用它。"""
    body = {"command": argv, "timeoutSeconds": max(1, min(int(timeout_s), C.EXEC_TIMEOUT_MAX_S))}
    if cwd:
        body["cwd"] = cwd
    if env:
        body["env"] = env
    # 只在「肯定没执行」时重发：409 容器还在启动、429 限流。503 可能已经执行过，重发会把命令跑两遍（评审第 2 轮 #2）
    return cli.post(f"/v2/pods/{pod_id}/exec", body, retry_on=(409, 429))


def sh(cli: Client, pod_id: str, script: str, timeout_s: int = 20) -> dict:
    return exec_cmd(cli, pod_id, ["sh", "-c", script], timeout_s)


def check_ok(r: dict, what: str) -> str:
    if r.get("timedOut") or r.get("exitCode") != 0:
        raise SystemExit(f"{what} 失败：exit={r.get('exitCode')} timedOut={r.get('timedOut')} "
                         f"stderr={r.get('stderr', '')[-500:]}")
    return r.get("stdout", "")


def new_run_id(job: str) -> str:
    if not re.fullmatch(r"[A-Za-z0-9_.-]{1,40}", job):
        raise SystemExit(f"任务名只用字母、数字、_ . -（≤ 40 个）：{job!r}")
    return f"{job}-{time.strftime('%Y%m%d%H%M%S', time.gmtime())}-{secrets.token_hex(3)}"  # UTC


def start_job(cli: Client, pod_id: str, script: str, job: str, jobs_dir: str = "/root/jobs",
              run_id: Optional[str] = None) -> str:
    """长命令放后台跑（exec 单次最多等 25 秒）；返回这次运行的唯一编号 <任务名>-<时间>-<随机>。
    日志写 <编号>.log，退出码写 <编号>.rc；job_status / wait_job 只认编号。任务名只作显示，
    同名跑两次各有各的文件，等第一次的不会拿到第二次的结果（大总管裁决，评审第 6–7 轮 #3/#2）。
    脚本作为独立参数交给 sh -c；脚本和外层包装都先 sh -n；放后台后等日志出现才算启动。"""
    run_id = run_id or new_run_id(job)  # 调用方可先拿到编号再启动（评审第 8 轮 #1）
    d = shlex.quote(f"{jobs_dir}/{run_id}")
    inner = f"sh -c {shlex.quote(script)} > {d}.log 2>&1; echo $? > {d}.rc"
    for what, text in (("命令", script), ("包装", inner)):
        syn = sh(cli, pod_id, f"sh -n -c {shlex.quote(text)}")
        if syn.get("exitCode") != 0:  # 语法错的后台任务连日志都写不出，会一直像「在跑」（2026-11-25 实测）
            raise SystemExit(f"{job} 的{what}有 sh 语法错误（bash 专有写法请包进 bash -c）："
                             f"{(syn.get('stderr') or '')[-200:]}")
    try:  # 从这里起，命令可能已经在机器里跑了：出错只报「结果未知」并带上编号，不许当没启动去重试
        check_ok(sh(cli, pod_id, f"mkdir -p {shlex.quote(jobs_dir)} && "
                                 f"setsid nohup sh -c {shlex.quote(inner)} > /dev/null 2>&1 < /dev/null &"),
                 f"启动 {job}")
        up = sh(cli, pod_id, f"for i in 1 2 3 4 5; do [ -e {d}.log ] && exit 0; sleep 1; done; exit 1")
    except BaseException as e:
        raise SystemExit(f"{job} 启动结果未知（{e}）；运行编号 {run_id}。先用 rp.py job-wait {pod_id} {run_id} 核查，"
                         f"不要直接重跑，免得同一件事跑两遍") from e
    if up.get("exitCode") != 0:
        raise SystemExit(f"{job} 启动结果未知：放进后台 5 秒还没有日志；运行编号 {run_id}，"
                         f"先用 rp.py job-wait {pod_id} {run_id} 核查")
    return run_id


# 在机器里读任务状态（评审第 10–12 轮同一处三次「换写法就漏」，按 CLAUDE.md 第二节改结构）：
# 用 Python 读文件，只有 FileNotFoundError 才算「不存在」，其余错误原样报出；状态与日志分字段输出，日志内容不会和标记混淆。
JOB_STATUS_PY = r"""
import json, sys
d, n = sys.argv[1], int(sys.argv[2])
out = {}
def rd(p, limit=None):
    try:
        with open(p, "rb") as f:
            if not limit:
                return f.read(), False
            f.seek(0, 2); size = f.tell(); f.seek(max(0, size - limit))
            return f.read(), size > limit
    except FileNotFoundError:
        return None, False
try:
    rc, _ = rd(d + ".rc")
    out["rc_raw"] = None if rc is None else rc.decode("utf-8", "replace").strip()
    log, cut = rd(d + ".log", 262144)
    out["log_exists"] = log is not None
    if log is not None:
        lines = log.decode("utf-8", "replace").splitlines(True)
        out["log"] = "".join(lines[-n:])
        out["log_bytes_cut"] = cut  # 按字节裁过：最前面那行可能不完整
        out["log_lines_cut"] = len(lines) > n
except Exception as e:
    out["error"] = type(e).__name__ + ": " + str(e)
print(json.dumps(out))
"""


def job_status(cli: Client, pod_id: str, job: str, jobs_dir: str = "/root/jobs", tail: int = 20) -> dict:
    """返回 done / rc / log（最后 tail 行）/ log_seen（日志文件是否存在）。
    读不出（非「不存在」的错误）、exec 输出被截断，一律报错，不当成「没日志」或「完成」。"""
    r = exec_cmd(cli, pod_id, ["python3", "-c", JOB_STATUS_PY, f"{jobs_dir}/{job}", str(int(tail))])
    if r.get("truncated"):
        raise SystemExit(f"查 {job} 的输出被截断，结果不完整，不判完成")
    st = json.loads(check_ok(r, f"查 {job}").strip().splitlines()[-1])
    if st.get("error"):
        raise SystemExit(f"查 {job} 读文件出错：{st['error']}")
    raw = st.get("rc_raw")
    rc = int(raw) if raw and raw.lstrip("-").isdigit() else None
    # 只有读到合法退出码才算完；.rc 空着（正在写）或乱码都当没完（评审第 1 轮 #11）
    return {"done": rc is not None, "rc": rc, "log": st.get("log", ""), "log_seen": bool(st.get("log_exists")),
            "log_bytes_cut": bool(st.get("log_bytes_cut")), "log_lines_cut": bool(st.get("log_lines_cut"))}


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

def put_bytes(cli: Client, pod_id: str, data: bytes, remote: str, overwrite: bool = False) -> str:
    """把小文件传上去（每次 exec 最多约 240 × 4 KB base64）；返回远端 sha256 并核对。
    每次上传用自己的临时文件；核过哈希才发布（评审第 5 轮 #2、第 6 轮 #1）：
    - 默认：ln 临时文件 → 正式名，已存在就失败（原子的「不存在才发布」）
    - --overwrite：先把旧文件硬链接到唯一的备份名（已存在就失败），成功了才 mv 替换（第 6 轮 #2）"""
    b64 = base64.b64encode(data).decode()
    step = C.EXEC_ARG_MAX_CHARS
    chunks = [b64[i:i + step] for i in range(0, len(b64), step)] or [""]
    q = shlex.quote(remote)
    tq = shlex.quote(f"{remote}.part-{secrets.token_hex(4)}")
    if not overwrite:  # 先查一次，省得白传；真正的保证在发布那一步
        r = sh(cli, pod_id, f"test ! -e {q}")
        if r.get("exitCode") != 0:
            raise SystemExit(f"远端已有 {remote}；确要覆盖加 --overwrite（旧文件会留作 .bak-时间-随机）")
    check_ok(sh(cli, pod_id, f"mkdir -p \"$(dirname {q})\" && : > {tq}"), "建远端文件")
    for i in range(0, len(chunks), C.PUT_ARGS_PER_CALL):
        part = chunks[i:i + C.PUT_ARGS_PER_CALL]
        r = exec_cmd(cli, pod_id, ["sh", "-c", f"printf %s \"$@\" | base64 -d >> {tq}", "sh", *part])
        check_ok(r, f"上传第 {i // C.PUT_ARGS_PER_CALL + 1} 批")
    out = check_ok(sh(cli, pod_id, f"sha256sum {tq}"), "核哈希")
    got = out.split()[0] if out.split() else ""
    want = hashlib.sha256(data).hexdigest()
    if got != want:
        raise SystemExit(f"上传后哈希不一致：{remote}（临时文件留着，正式文件没动）")
    if overwrite:
        bak = shlex.quote(f"{remote}.bak-{time.strftime('%Y%m%d%H%M%S')}-{secrets.token_hex(3)}")
        pub = f"if [ -e {q} ]; then ln {q} {bak} || exit 1; fi && mv -f {tq} {q}"
    else:
        pub = f"ln {tq} {q} && rm -f {tq}"
    r = sh(cli, pod_id, pub)
    if r.get("exitCode") != 0:
        raise SystemExit(f"发布 {remote} 失败（{'备份旧文件失败' if overwrite else '远端已有同名文件'}）；"
                         f"正式文件没动，临时文件留着：{(r.get('stderr') or '')[-120:]}")
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
