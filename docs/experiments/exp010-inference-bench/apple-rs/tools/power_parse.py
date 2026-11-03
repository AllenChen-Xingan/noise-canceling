"""把 rt-matrix.sh 的 powermetrics 输出折成「每帧耗电」（BE-POWER，standard.md §6）。

v2（2026-11-02 自审后）：
- 只平均**落在基准计时窗内**的样本（bench 输出 win_start_unix / win_end_unix；样本时间戳是秒级，两头各收 1 s）。
  旧版把启动、加载、读转储、200 帧预热也算进去了，每帧耗电偏高。
- 时长用实测墙钟 wall_s，不用「帧数 × pace」的名义时长（有超时帧时实际更长）。
- 每轮各有一段空闲基线（idle_<轮>.txt），每段减去同轮的基线。
- 汇总按设置给 3 轮的中位与范围；相差小于空闲基线的轮间离散（噪声底）的，不算有差别。
旧数据（没有计时窗字段）退回整段平均，并标 window=false。
每帧耗电 µJ = (CPU+GPU+神经引擎平均功率 − 同轮空闲, mW) × wall_s ÷ 帧数 × 1000。
用法：python3 power_parse.py <结果目录>
"""
import json, os, re, statistics, sys
from datetime import datetime

res = sys.argv[1]
head = re.compile(r"^\*\*\* Sampled system activity \((.+?)\) \(([\d.]+)ms elapsed\) \*\*\*", re.M)
pw = {k: re.compile(rf"^{k} Power: (\d+) mW", re.M) for k in ("CPU", "GPU", "ANE")}


def samples(path):
    """[(unix 秒, {CPU,GPU,ANE})]，按样本切块。"""
    t = open(path, errors="ignore").read()
    idx = [(m.start(), m.group(1)) for m in head.finditer(t)] + [(len(t), None)]
    out = []
    for (a, ts), (b, _) in zip(idx, idx[1:]):
        blk = t[a:b]
        try:
            u = datetime.strptime(" ".join(ts.split()), "%a %b %d %H:%M:%S %Y %z").timestamp()
        except ValueError:
            continue
        v = {}
        for k, p in pw.items():
            m = p.search(blk)
            v[k] = int(m.group(1)) if m else 0
        out.append((u, v))
    return out


def avg(ss, lo=None, hi=None):
    sel = [v for u, v in ss if (lo is None or u >= lo - 1) and (hi is None or u <= hi + 1)]
    if not sel:
        return None, 0
    return {k: sum(x[k] for x in sel) / len(sel) for k in ("CPU", "GPU", "ANE")}, len(sel)


rows = []
idle_by_round = {}
for line in open(os.path.join(res, "rt.jsonl")):
    r = json.loads(line)
    rnd = r.get("round", 1)
    if rnd not in idle_by_round:
        f = os.path.join(res, "power", f"idle_{rnd}.txt")
        if not os.path.exists(f):
            f = os.path.join(res, "power", "idle.txt")
        idle_by_round[rnd], _ = avg(samples(f))
    idle = idle_by_round[rnd]
    pf = os.path.join(res, "power", f"{r.get('file', r['set'])}.txt")
    ss = samples(pf)
    row = {"round": rnd, "set": r["set"], "config": r["config"]}
    x = r.get("result")
    if x:
        win = "win_start_unix" in x and x["win_start_unix"] > 0
        p, n = avg(ss, x["win_start_unix"], x["win_end_unix"]) if win else avg(ss)
        wall = x.get("wall_s") or x["frames"] * (x.get("pace_ms") or 16) / 1000
        delta = sum(p[k] - idle[k] for k in ("CPU", "GPU", "ANE"))
        row.update({"window": win, "samples": n, "power_mw": {k: round(p[k], 1) for k in p},
                    "idle_mw": {k: round(idle[k], 1) for k in idle}, "delta_mw": round(delta, 1),
                    "uj_per_frame": round(delta * wall / x["frames"] * 1000, 1),
                    "p50_ms": x["p50_ms"], "p999_ms": x["p999_ms"], "max_ms": x["max_ms"],
                    "frames_over_3ms": x.get("frames_over_3ms"), "cpu_over_wall": x["cpu_over_wall"]})
    else:
        row["error"] = r.get("error")
    rows.append(row)

idle_cpu = [v["CPU"] for v in idle_by_round.values() if v]
noise = (max(idle_cpu) - min(idle_cpu)) if len(idle_cpu) > 1 else None
by = {}
for r in rows:
    if "delta_mw" in r:
        by.setdefault(r["config"], []).append(r)
summary = []
for c, rs in by.items():
    d = [r["delta_mw"] for r in rs]
    a = [r["power_mw"]["CPU"] for r in rs]
    summary.append({"config": c, "rounds": len(rs), "delta_mw_median": round(statistics.median(d), 1),
                    "delta_mw_range": [min(d), max(d)], "cpu_mw_median": round(statistics.median(a), 1),
                    "uj_per_frame_median": round(statistics.median(r["uj_per_frame"] for r in rs), 1),
                    "p50_ms_median": statistics.median(r["p50_ms"] for r in rs)})
summary.sort(key=lambda s: s["delta_mw_median"])
print(json.dumps({"idle_cpu_mw_by_round": idle_cpu, "noise_floor_mw": noise, "summary": summary, "rows": rows},
                 ensure_ascii=False, indent=1))
