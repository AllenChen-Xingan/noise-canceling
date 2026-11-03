#!/bin/bash
# exp010 · BE-IMPACT（D75：「实际耗电和对性能的影响」）——开着降噪，用户的前台活慢了多少。要名额独占，约 4 分钟。
# 前台负载：`exp010-apple loadgen`（全部逻辑核，每核 4 MB 数组乘加，工作量固定，只看墙钟）。选它的理由见 standard.md BE-IMPACT。
# 做法：「关 / 开」交替 7 对；开 = 降噪组合按实时节奏（16 ms）在后台跑，设置同 D74 默认（--qos rt）。
# 慢了多少 = 开的中位 ÷ 关的中位 − 1。另报两组「关」之间的离散度，当量测噪声底。
# 用法：tools/impact.sh <降噪命令（不含 --frames）> [名字]
#   例：tools/impact.sh "target/release/exp010-apple bench bnns <dir>/gtcrn_unrolled_fp32.mlmodelc <dir>/gtcrn --model gtcrn --pace-ms 16 --qos rt" bnns-rt
set -u
# ⚠️ 2026-11-02 事故：旧版用 `eval "$CMD" &`，$! 是后台子 shell 的进程号，kill 只杀了子 shell，基准程序成了孤儿（父进程 1）
# 一直按节奏跑，21 个，合计约半个核；那一批 BE-IMPACT 数据作废。现在：后台用 `exec` 让 $! 就是基准程序本身，
# 每对结束 kill + wait 并核实已退出；脚本无论怎么退出都由 trap 清理自己起的进程。
KIDS=()
cleanup() {
  for p in "${KIDS[@]:-}"; do [ -n "$p" ] && kill "$p" 2>/dev/null; done
  for p in "${KIDS[@]:-}"; do [ -n "$p" ] && wait "$p" 2>/dev/null; done
}
trap cleanup EXIT INT TERM HUP
HERE=$(cd "$(dirname "$0")/.." && pwd); EXP=$(cd "$HERE/.." && pwd)
CMD=$1; NAME=${2:-denoise}
BID=${BID:-$(date +%Y%m%d-%H%M%S)-impact}
RES=$EXP/apple/results/$BID; mkdir -p "$RES"
BIN=$HERE/target/release/exp010-apple
PASSES=${PASSES:-400}
echo "{\"when\":\"$(date -Iseconds)\",\"load\":\"$(sysctl -n vm.loadavg)\",\"power\":\"$(pmset -g batt | head -1)\",\"cmd\":\"$CMD\"}" >> "$RES/machine.jsonl"
"$BIN" loadgen --passes $PASSES > /dev/null   # 预热一次，不计
for k in 1 2 3 4 5 6 7; do
  l=$("$BIN" loadgen --passes $PASSES); echo "{\"pair\":$k,\"denoise\":false,\"r\":$l}" >> "$RES/impact.jsonl"
  eval "exec $CMD --frames 100000" > "$RES/denoise_$k.json" 2>> "$RES/stderr.log" &   # exec：$! 就是基准程序
  dn=$!; KIDS+=("$dn"); sleep 3   # 等降噪进入稳态
  kill -0 "$dn" 2>/dev/null || { echo "降噪进程没起来（见 stderr.log）"; exit 1; }
  l=$("$BIN" loadgen --passes $PASSES); echo "{\"pair\":$k,\"denoise\":true,\"name\":\"$NAME\",\"r\":$l}" >> "$RES/impact.jsonl"
  kill "$dn"; wait "$dn" 2>/dev/null
  if kill -0 "$dn" 2>/dev/null; then echo "降噪进程 $dn 没退出，停下"; exit 1; fi
  KIDS=()
  sleep 2
done
python3 - "$RES" <<'PY'
import json, statistics, sys
rows = [json.loads(l) for l in open(sys.argv[1] + "/impact.jsonl")]
off = [r["r"]["loadgen_ms"] for r in rows if not r["denoise"]]
on = [r["r"]["loadgen_ms"] for r in rows if r["denoise"]]
mo, mn = statistics.median(off), statistics.median(on)
spread = (max(off) - min(off)) / mo
out = {"off_median_ms": mo, "on_median_ms": mn, "slowdown_pct": round((mn / mo - 1) * 100, 2),
       "off_spread_pct": round(spread * 100, 2), "pairs": len(on),
       "verdict": "过目标" if mn / mo - 1 <= 0.01 else ("过挡线" if mn / mo - 1 <= 0.02 else "不过")}
json.dump(out, open(sys.argv[1] + "/impact_summary.json", "w"), ensure_ascii=False)
print(json.dumps(out, ensure_ascii=False))
PY
echo "$RES"
