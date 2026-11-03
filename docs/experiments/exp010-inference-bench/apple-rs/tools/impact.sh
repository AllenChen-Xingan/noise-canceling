#!/bin/bash
# exp010 · BE-IMPACT（D75：「实际耗电和对性能的影响」）——开着降噪，用户的前台活慢了多少。要名额独占，约 4 分钟。
# 前台负载：`exp010-apple loadgen`（全部逻辑核，每核 4 MB 数组乘加，工作量固定，只看墙钟）。选它的理由见 standard.md BE-IMPACT。
# 做法：「关 / 开」交替 PAIRS 对（默认 15）；开 = 降噪组合按实时节奏（16 ms）在后台跑，设置同 D74 默认（--qos rt）。
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
trap cleanup EXIT
trap 'cleanup; exit 130' INT TERM HUP   # 中断就清理并退出，不接着起下一对
HERE=$(cd "$(dirname "$0")/.." && pwd); EXP=$(cd "$HERE/.." && pwd)
CMD=$1; NAME=${2:-denoise}
BID=${BID:-$(date +%Y%m%d-%H%M%S)-impact}
RES=$EXP/apple/results/$BID
[ -e "$RES" ] && { echo "结果目录已存在：$RES（换个 BID，不覆盖）"; exit 1; }
mkdir -p "$RES"
BIN=$HERE/target/release/exp010-apple
PASSES=${PASSES:-8000}
# v2（自审后）：前台负载只开「性能核个数」个线程（开满全部核时调度抖动 10–20%，盖住信号）；15 对；判定用成对差值、且要超过噪声底
PCORES=$(sysctl -n hw.perflevel0.logicalcpu 2>/dev/null || echo 4)
PAIRS=${PAIRS:-15}
LG="$BIN loadgen --threads $PCORES --passes $PASSES"
echo "{\"when\":\"$(date -Iseconds)\",\"load\":\"$(sysctl -n vm.loadavg)\",\"power\":\"$(pmset -g batt | head -1)\",\"cmd\":\"$CMD\"}" >> "$RES/machine.jsonl"
$LG > /dev/null   # 预热一次，不计
for k in $(seq 1 $PAIRS); do
  l=$($LG); echo "{\"pair\":$k,\"denoise\":false,\"r\":$l}" >> "$RES/impact.jsonl"
  eval "exec $CMD --frames 1500" > "$RES/denoise_$k.json" 2>> "$RES/stderr.log" &   # exec：$! 就是基准程序；1500 帧 = 24 s 上限，万一成孤儿也会自己结束
  dn=$!; KIDS+=("$dn"); sleep 3   # 等降噪进入稳态
  kill -0 "$dn" 2>/dev/null || { echo "降噪进程没起来（见 stderr.log）"; exit 1; }
  l=$($LG); echo "{\"pair\":$k,\"denoise\":true,\"name\":\"$NAME\",\"r\":$l}" >> "$RES/impact.jsonl"
  kill "$dn"; wait "$dn" 2>/dev/null
  if kill -0 "$dn" 2>/dev/null; then echo "降噪进程 $dn 没退出，停下"; exit 1; fi
  KIDS=()
  sleep 2
done
python3 - "$RES" <<'PY'
import json, statistics, sys
rows = [json.loads(l) for l in open(sys.argv[1] + "/impact.jsonl")]
off = {r["pair"]: r["r"]["loadgen_ms"] for r in rows if not r["denoise"]}
on = {r["pair"]: r["r"]["loadgen_ms"] for r in rows if r["denoise"]}
pairs = sorted(set(off) & set(on))
rel = [on[k] / off[k] - 1 for k in pairs]            # 成对差（同一对里先关后开）
offv = [off[k] for k in pairs]
noise = statistics.pstdev(offv) / statistics.mean(offv)  # 「关」之间的相对离散 = 噪声底
med = statistics.median(rel)
best = min(on.values()) / min(off.values()) - 1          # 最小值之比：最接近无干扰时的真实时长
if abs(med) <= noise:
    verdict = f"≤ 噪声底（{noise*100:.2f}%），判不了"
else:
    verdict = "过目标" if med <= 0.01 else ("过挡线" if med <= 0.02 else "不过")
out = {"pairs": len(pairs), "slowdown_paired_median_pct": round(med * 100, 2), "slowdown_min_ratio_pct": round(best * 100, 2),
       "noise_floor_pct": round(noise * 100, 2), "verdict": verdict}
json.dump(out, open(sys.argv[1] + "/impact_summary.json", "w"), ensure_ascii=False)
print(json.dumps(out, ensure_ascii=False))
PY
echo "$RES"
