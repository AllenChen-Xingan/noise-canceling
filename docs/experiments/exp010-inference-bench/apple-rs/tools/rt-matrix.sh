#!/bin/bash
# exp010 · 实时线程策略对照（要名额）。问题：BNNS GTCRN 连续跑 p50 0.155 ms，按 16 ms 节奏跑却是 0.82 ms、4.8% 核（build 20261102-173000-b2）。
# 每个设置按实时节奏跑 60 s（3750 帧），比 p50 / p99.9 / max / 超 3 ms 帧数 / cpu_over_wall；最好的那个再跑 10 分钟（BE-DEADLINE）。
# 用法：tools/rt-matrix.sh <模型 build 目录（含 gtcrn_unrolled_fp32.mlmodelc 与 gtcrn.in/ort.f32）> [最好设置的名字]
set -u
HERE=$(cd "$(dirname "$0")/.." && pwd); EXP=$(cd "$HERE/.." && pwd)
O=$1; BEST=${2:-}
BID=${BID:-$(date +%Y%m%d-%H%M%S)-rt}
RES=$EXP/apple/results/$BID; mkdir -p "$RES"
BIN=$HERE/target/release/exp010-apple
(cd "$HERE" && cargo build --release 2>&1 | tail -1) >> "$RES/run.log"
BN="$BIN bench bnns $O/gtcrn_unrolled_fp32.mlmodelc $O/gtcrn --model gtcrn --pace-ms 16"
CM="$BIN bench coreml $O/gtcrn_unrolled_fp32.mlmodelc $O/gtcrn --model gtcrn --units cpu --pace-ms 16"
SETS=(
  "bnns ui|$BN"
  "bnns rt|$BN --qos rt"
  "bnns ui+workgroup|$BN --workgroup"
  "bnns rt+workgroup|$BN --qos rt --workgroup"
  "bnns default-qos|$BN --qos default"
  "bnns ui+spin（只排查）|$BN --spin"
  "coreml-cpu ui|$CM"
  "coreml-cpu rt|$CM --qos rt"
)
echo "{\"when\":\"$(date -Iseconds)\",\"load\":\"$(sysctl -n vm.loadavg)\",\"power\":\"$(pmset -g batt | head -1)\"}" >> "$RES/machine.jsonl"
if [ -z "$BEST" ]; then
  for c in "${SETS[@]}"; do
    n=${c%%|*}; cmd=${c#*|}
    echo "[$(date +%H:%M:%S)] $n" | tee -a "$RES/run.log"
    l=$(eval "$cmd --frames 3750" 2>> "$RES/stderr.log") && echo "{\"config\":\"$n\",\"result\":$l}" >> "$RES/rt.jsonl" || echo "{\"config\":\"$n\",\"error\":\"见 stderr.log\"}" >> "$RES/rt.jsonl"
  done
else
  for c in "${SETS[@]}"; do
    n=${c%%|*}; cmd=${c#*|}
    [ "$n" = "$BEST" ] || continue
    echo "[$(date +%H:%M:%S)] 10 分钟：$n" | tee -a "$RES/run.log"
    l=$(eval "$cmd --frames 37500" 2>> "$RES/stderr.log") && echo "{\"config\":\"$n\",\"long\":true,\"result\":$l}" >> "$RES/rt.jsonl"
  done
fi
echo "{\"when\":\"$(date -Iseconds)\",\"load\":\"$(sysctl -n vm.loadavg)\",\"phase\":\"end\"}" >> "$RES/machine.jsonl"
echo "$RES"
