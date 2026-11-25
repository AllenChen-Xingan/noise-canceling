#!/usr/bin/env python3
"""云训练机命令行（RunPod API v2）。用法见 README.md；`python3 rp.py -h` 看全部子命令。

花钱的子命令（volume-create · up · probe）必须带 --yes，且开机前过花费上限检查。
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import config as C  # noqa: E402
import ledger as L  # noqa: E402
import rp_ops as O  # noqa: E402
import subprocess  # noqa: E402
import watchdog as W  # noqa: E402
from rp_client import Client, load_key  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
SPAWN = subprocess.Popen  # 测试换成假的，不真起后台进程


def need_yes(a, what: str):
    if not a.yes:
        raise SystemExit(f"{what} 会花钱：确认后加 --yes 再跑")


def cmd_offers(cli, a):
    g = O.gpu_offer(cli)
    dcs = O.candidate_datacenters(cli)
    print(json.dumps({"gpu": g.get("id"), "secure_usd_per_hr": (g.get("price") or {}).get("secure"),
                      "minVcpuCount": g.get("minVcpuCount"), "availability": g.get("availability"),
                      "datacenters_with_volume": dcs,
                      "volume_usd_per_month": C.volume_usd_per_month()}, ensure_ascii=False, indent=1))


def cmd_volume_create(cli, a):
    need_yes(a, "建网络盘")
    v = O.create_volume(cli, a.dc, a.size, ledger_path=a.ledger)
    print(json.dumps(v, ensure_ascii=False))


def cmd_volume_delete(cli, a):
    need_yes(a, "删网络盘（数据一起删）")
    O.delete_volume(cli, a.id, a.ledger, note=a.note)
    print(f"已删网络盘 {a.id}")


def _volume_id(cli, a):
    if a.no_volume:
        return None, a.dc
    v = O.find_volume(cli)
    if not v:
        raise SystemExit(f"没有网络盘 {C.VOLUME_NAME}；先 volume-create，或加 --no-volume")
    if a.dc and a.dc != v["dataCenter"]:
        raise SystemExit(f"网络盘在 {v['dataCenter']}，不能在 {a.dc} 开机挂它")
    return v["id"], v["dataCenter"]


def _rate_hint(cli, cpu: bool = False) -> float:
    if cpu:
        price, want = O.cpu_rate(cli), C.CPU_USD_PER_VCPU_HR_EXPECTED * C.CPU_VCPU
    else:
        price, want = (O.gpu_offer(cli).get("price") or {}).get("secure"), C.USD_PER_HR_EXPECTED
    if not price:
        raise SystemExit("目录里拿不到 Secure 价格，不开机")
    if float(price) > want * C.PRICE_TOLERANCE:
        raise SystemExit(f"目录价 {price} 美元/时，高于预期 {want} 的 {C.PRICE_TOLERANCE} 倍，不开机")
    return float(price)


def bring_up(cli, a):
    vol, dc = _volume_id(cli, a)
    if not dc:
        raise SystemExit("要指定 --dc（不挂盘时）")
    p = O.create_pod(cli, dc, vol, a.max_hours, _rate_hint(cli, a.cpu), ledger_path=a.ledger, cpu=a.cpu)
    when = W.register(p["id"], a.max_hours)  # 第 1 层兜底：本机看门狗到点删机
    rp_args = ("--ledger", os.path.abspath(a.ledger)) + (("--env-file", os.path.abspath(a.env_file)) if a.env_file else ())
    W.ensure_running(os.path.abspath(__file__), spawn=SPAWN, rp_args=rp_args)
    print(f"看门狗登记：{p['id']} 最晚 {when} 删机", file=sys.stderr)
    print(f"已开机 {p['id']}（{dc}，每小时 {p.get('cost')} 美元，最长 {a.max_hours} 小时）", file=sys.stderr)
    try:
        p = O.wait_running(cli, p["id"])
        O.wait_exec_ready(cli, p["id"])
    except BaseException:
        O.delete_pod(cli, p["id"], a.ledger, note="没跑起来，自动删")
        raise
    gpu = p.get("gpu") or {}
    if not a.cpu and gpu.get("vcpuCount") and gpu["vcpuCount"] < C.MIN_VCPU_PER_GPU:
        O.delete_pod(cli, p["id"], a.ledger, note=f"vCPU {gpu['vcpuCount']} 不够，自动删")
        raise SystemExit(f"分到的机器只有 {gpu['vcpuCount']} vCPU，已删")
    return p


def cmd_up(cli, a):
    need_yes(a, "开机")
    p = bring_up(cli, a)
    print(p["id"])


def cmd_down(cli, a):
    est = O.delete_pod(cli, a.pod, a.ledger, note=a.note)
    print(f"已删机 {a.pod}，估算 {est:.4f} 美元")


def cmd_reap(cli, a):
    pods = O.list_our_pods(cli)
    for p in pods:
        O.delete_pod(cli, p["id"], a.ledger, note="reap")
        print(f"已删 {p['id']} {p.get('name')}")
    for pid in L.open_pods(L.read_rows(a.ledger)):
        if pid in {p["id"] for p in pods}:
            continue
        try:  # 只在云上确实没有、或名字仍是 vdn- 时才删 / 补账（评审第 1 轮 #1）
            live = cli.get(f"/v2/pods/{pid}")
        except O.ApiError as e:
            if e.status != 404:
                raise
            live = None
        if live and not str(live.get("name", "")).startswith(C.POD_NAME_PREFIX):
            print(f"跳过 {pid}：账上未关，但云上名字是 {live.get('name')!r}，不是 {C.POD_NAME_PREFIX} 开头，请人工核对")
            continue
        O.delete_pod(cli, pid, a.ledger, note="reap：账上未关、云上已无" if live is None else "reap")
    if not pods:
        print("云上没有 vdn- 开头的机器")


def cmd_watchdog(cli, a):
    """本机看门狗进程本身（由 up 自动起，一般不用手动跑）。登记表空了就退出。"""
    W.run_loop(lambda pid: O.delete_pod(cli, pid, a.ledger, note="watchdog 到点删机"))


def cmd_sh(cli, a):
    r = O.sh(cli, a.pod, a.script, a.timeout)
    sys.stdout.write(r.get("stdout", ""))
    sys.stderr.write(r.get("stderr", ""))
    if r.get("timedOut"):
        raise SystemExit("超时（命令可能仍在跑；长命令请用 job-start）")
    sys.exit(r.get("exitCode") or 0)


def cmd_job_start(cli, a):
    O.start_job(cli, a.pod, a.script, a.job)
    print(f"已启动 {a.job}")


def cmd_job_wait(cli, a):
    st = O.wait_job(cli, a.pod, a.job, a.timeout)
    sys.stdout.write(st["log"])
    sys.exit(st["rc"] or 0)


def cmd_put(cli, a):
    with open(a.local, "rb") as f:
        print(O.put_bytes(cli, a.pod, f.read(), a.remote))


def cmd_get(cli, a):
    data = O.fetch_bytes(cli, a.pod, a.remote)
    with open(a.local, "wb") as f:
        f.write(data)
    print(f"{a.local} {len(data)} 字节")


def cmd_spend(cli, a):
    rows = L.read_rows(a.ledger)
    print(json.dumps({"pods_closed_usd": round(L.pod_spent(rows), 4),
                      "pods_open_usd": round(L.live_pod_cost(rows), 4),
                      "volume_usd": round(L.volume_spent(rows), 4),
                      "total_usd": round(L.spent_so_far(rows), 4), "cap_usd": C.SPEND_CAP_USD,
                      "open_pods": list(L.open_pods(rows))}, ensure_ascii=False))


def cmd_probe(cli, a):
    """开一台、量 vCPU / 下载速度、马上删（finally 保证删）。"""
    need_yes(a, "试开机")
    p = bring_up(cli, a)
    out_path = os.path.join(HERE, "probes", time.strftime("%Y%m%d-%H%M%S") + ".json")
    try:
        for name in ("probe_remote.py", "probe_urls.txt"):
            with open(os.path.join(HERE, name), "rb") as f:
                O.put_bytes(cli, p["id"], f.read(), f"/root/probe/{name}")
        O.start_job(cli, p["id"], f"python3 /root/probe/probe_remote.py /root/probe/probe_urls.txt {a.seconds}",
                    "probe")
        st = O.wait_job(cli, p["id"], "probe", timeout_s=a.seconds * 12 + 120)
        line = st["log"].strip().splitlines()[-1] if st["log"].strip() else "{}"
        result = json.loads(line)
        result["pod"] = {k: p.get(k) for k in ("id", "dataCenterId", "cost", "gpu", "cudaVersion")}
        os.makedirs(os.path.dirname(out_path), exist_ok=True)
        with open(out_path, "w", encoding="utf-8") as f:
            json.dump(result, f, ensure_ascii=False, indent=1)
        print(out_path)
    finally:
        est = O.delete_pod(cli, p["id"], a.ledger, note="probe")
        print(f"已删机 {p['id']}，估算 {est:.4f} 美元", file=sys.stderr)


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--ledger", default=L.DEFAULT_PATH, help="费用账本（默认 cloud/costs.tsv）")
    ap.add_argument("--env-file", default=None, help="密钥文件（默认主仓库根 .env.local）")
    sub = ap.add_subparsers(dest="cmd", required=True)

    def add(name, fn, help_):
        p = sub.add_parser(name, help=help_)
        p.set_defaults(fn=fn)
        return p

    add("offers", cmd_offers, "本卡价格、vCPU、有货且能建网络盘的机房")
    p = add("volume-create", cmd_volume_create, "建网络盘（花钱）")
    p.add_argument("--dc", default=C.DATA_CENTER)
    p.add_argument("--size", type=int, default=C.VOLUME_SIZE_GB)
    p.add_argument("--yes", action="store_true")
    p = add("volume-delete", cmd_volume_delete, "删网络盘（数据一起删）")
    p.add_argument("--id", required=True)
    p.add_argument("--note", default="")
    p.add_argument("--yes", action="store_true")
    for name, fn, h in (("up", cmd_up, "开机并挂网络盘（花钱）"), ("probe", cmd_probe, "试开一台测速后删（花钱）")):
        p = add(name, fn, h)
        p.add_argument("--max-hours", type=float, default=C.DEFAULT_MAX_HOURS)
        p.add_argument("--no-volume", action="store_true")
        p.add_argument("--dc", default=None)
        p.add_argument("--cpu", action="store_true", help="开纯 CPU 机器（下载数据用），不开显卡")
        p.add_argument("--yes", action="store_true")
        if name == "probe":
            p.add_argument("--seconds", type=float, default=20.0, help="每个地址最多测几秒")
    p = add("down", cmd_down, "删机（不是 stop）并记账")
    p.add_argument("pod")
    p.add_argument("--note", default="")
    add("reap", cmd_reap, "删掉所有 vdn- 开头的机器（兜底）")
    add("watchdog", cmd_watchdog, "本机看门狗：到点删机（up 自动起）")
    add("spend", cmd_spend, "按账本汇总已花与上限")
    p = add("sh", cmd_sh, "在机器里跑一条短命令（≤25 秒）")
    p.add_argument("pod")
    p.add_argument("script")
    p.add_argument("--timeout", type=int, default=20)
    p = add("job-start", cmd_job_start, "后台跑长命令")
    p.add_argument("pod")
    p.add_argument("job")
    p.add_argument("script")
    p = add("job-wait", cmd_job_wait, "等后台命令结束并打印日志尾")
    p.add_argument("pod")
    p.add_argument("job")
    p.add_argument("--timeout", type=float, default=3600)
    for name, fn in (("put", cmd_put), ("get", cmd_get)):
        p = add(name, fn, "传上去" if name == "put" else "取回来（核 sha256）")
        p.add_argument("pod")
        if name == "put":
            p.add_argument("local")
            p.add_argument("remote")
        else:
            p.add_argument("remote")
            p.add_argument("local")
    return ap


def main(argv=None, client: Client = None):
    a = build_parser().parse_args(argv)
    cli = client or Client(load_key(env_file=a.env_file))
    a.fn(cli, a)


if __name__ == "__main__":
    main()
