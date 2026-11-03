"""把 rt-matrix.sh 的 powermetrics 输出折成「每帧耗电」（BE-POWER，standard.md §6）。
每帧耗电 µJ = (该段 CPU+GPU+神经引擎平均功率 − 空闲平均功率, mW) × 墙钟时长 (s) ÷ 帧数 × 1000。
墙钟时长 = 帧数 × pace_ms（按实时节奏跑，帧数 3750 × 16 ms = 60 s）。
用法：python3 power_parse.py <结果目录>
"""
import json, os, re, sys

res = sys.argv[1]
pat = {k: re.compile(rf"^{k} Power: (\d+) mW", re.M) for k in ("CPU", "GPU", "ANE")}


def avg(path):
    t = open(path, errors="ignore").read()
    out = {}
    for k, p in pat.items():
        v = [int(x) for x in p.findall(t)]
        out[k] = sum(v) / len(v) if v else 0.0
    out["samples"] = len(pat["CPU"].findall(t))
    return out


idle = avg(os.path.join(res, "power", "idle.txt"))
rows = []
for line in open(os.path.join(res, "rt.jsonl")):
    r = json.loads(line)
    p = avg(os.path.join(res, "power", f"{r['set']}.txt"))
    row = {"set": r["set"], "config": r["config"], "power_mw": {k: round(p[k], 1) for k in ("CPU", "GPU", "ANE")}, "samples": p["samples"]}
    if "result" in r:
        x = r["result"]
        frames, pace = x["frames"], x.get("pace_ms", 16) or 16
        delta = sum(p[k] - idle[k] for k in ("CPU", "GPU", "ANE"))
        row.update({"delta_mw": round(delta, 1), "uj_per_frame": round(delta * frames * pace / 1000 / frames * 1000, 1),
                    "p50_ms": x["p50_ms"], "p999_ms": x["p999_ms"], "max_ms": x["max_ms"],
                    "frames_over_3ms": x.get("frames_over_3ms"), "cpu_over_wall": x["cpu_over_wall"]})
    rows.append(row)
print(json.dumps({"idle_mw": {k: round(idle[k], 1) for k in ("CPU", "GPU", "ANE")}, "rows": rows}, ensure_ascii=False, indent=1))
