#!/usr/bin/env bash
# 手写 GTCRN：纯循环版（native）对 Accelerate 版（native-blas）· 对拍 + 计时 3 轮交替 + 全进程 malloc 计数。要先拿名额。
# 用法：bash tools/run_blas_probe.sh <build-id> [轮数=3]
set -euo pipefail
ID=${1:?build-id}; ROUNDS=${2:-3}
HERE=$(cd "$(dirname "$0")/.." && pwd)
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
BIN=$HERE/bench/target/release/exp010-bench
W=$M/_tmp/20261031-exp010/gtcrn_dns3
G=$M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
F=$HOME/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx
FT=$M/_tmp/20261102-exp010/fastenhancer_m_noseq.onnx
WAV=$M/resources/gtcrn/test_wavs/mix.wav
OUT=$HERE/results/$ID
[ -e "$OUT" ] && { echo "已有 $OUT，不覆盖"; exit 1; }
mkdir -p "$OUT"
{
  echo "date=$(date '+%F %T %z')"; echo "machine=$(sysctl -n hw.model)"; echo "os=$(sw_vers -productVersion)"
  echo "power=$(pmset -g batt | head -1 | sed "s/.*'\(.*\)'.*/\1/")"; echo "uptime_before: $(uptime)"
  echo "git=$(git -C "$HERE" rev-parse --short HEAD)"; shasum -a 256 "$BIN"
  ps -Ao %cpu,comm -r | head -4 || true
} > "$OUT/env.txt"
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native-blas >> "$OUT/parity.jsonl"
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native-blas --frames 37500 --reset-at 20000 >> "$OUT/parity.jsonl"
for r in $(seq 1 "$ROUNDS"); do
  if [ $((r % 2)) = 1 ]; then ks="native native-blas"; else ks="native-blas native"; fi
  for k in $ks; do
    $BIN gtcrn-bench $k "$W" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"
  done
done
# 全进程 malloc 计数（钩子会拖慢分配，所以单独跑，不进计时表）
for k in ort native native-blas tract; do
  a=$G; [[ $k == native* ]] && a=$W
  $BIN gtcrn-bench $k "$a" "$WAV" --frames 2000 --count-malloc >> "$OUT/malloc.jsonl"
done
for k in ort tract; do
  a=$F; [ $k = tract ] && a=$FT
  $BIN fem-bench $k "$a" "$WAV" --frames 2000 --count-malloc >> "$OUT/malloc.jsonl"
done
echo "uptime_after: $(uptime)" >> "$OUT/env.txt"
cat "$OUT/parity.jsonl"
