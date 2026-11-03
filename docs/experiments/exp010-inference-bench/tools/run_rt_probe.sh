#!/usr/bin/env bash
# ① 手写版按实时节奏（每 16 ms 一帧）在四种线程设置下各跑 60 秒：ui / rt / ui+工作组 / rt+工作组
# ② native-fast（内联多项式 exp / tanh）的对拍 + 与 native 交替计时 3 轮
# 要先拿名额。用法：bash tools/run_rt_probe.sh <build-id>
set -euo pipefail
ID=${1:?build-id}
HERE=$(cd "$(dirname "$0")/.." && pwd)
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
BIN=$HERE/bench/target/release/exp010-bench
W=$M/_tmp/20261031-exp010/gtcrn_dns3
G=$M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
WAV=$M/resources/gtcrn/test_wavs/mix.wav
OUT=$HERE/results/$ID
[ -e "$OUT" ] && { echo "已有 $OUT，不覆盖"; exit 1; }
mkdir -p "$OUT"
{
  echo "date=$(date '+%F %T %z')"; echo "power=$(pmset -g batt | head -1 | sed "s/.*'\(.*\)'.*/\1/")"
  echo "uptime_before: $(uptime)"; echo "git=$(git -C "$HERE" rev-parse --short HEAD)"; shasum -a 256 "$BIN"
  echo "default_output=$(system_profiler SPAudioDataType 2>/dev/null | grep -B8 'Default Output Device: Yes' | head -1 | sed 's/^ *//')"
  ps -Ao %cpu,comm -r | head -4 || true
} > "$OUT/env.txt"
# ② 先跑短的
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native-fast >> "$OUT/parity.jsonl"
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native-fast --frames 37500 --reset-at 20000 >> "$OUT/parity.jsonl"
for r in 1 2 3; do
  if [ $((r % 2)) = 1 ]; then ks="native native-fast"; else ks="native-fast native"; fi
  for k in $ks; do $BIN gtcrn-bench $k "$W" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"; done
done
# ① 实时节奏 × 四种线程设置，各 3750 帧 ≈ 60 秒
for set in "ui" "rt" "ui --workgroup" "rt --workgroup"; do
  q=${set%% *}; extra=""; [[ $set == *workgroup* ]] && extra="--workgroup"
  $BIN gtcrn-bench native "$W" "$WAV" --frames 3750 --pace-ms 16 --qos "$q" $extra >> "$OUT/paced.jsonl"
done
echo "uptime_after: $(uptime)" >> "$OUT/env.txt"
cat "$OUT/parity.jsonl" "$OUT/bench.jsonl" "$OUT/paced.jsonl"
