#!/usr/bin/env bash
# exp010 Rust 一侧：对拍 + 基准，结果写 results/<build-id>/。要先拿到重活名额。
# 用法：bash tools/run_bench.sh <build-id> [轮数=3]
set -euo pipefail
ID=${1:?build-id}; ROUNDS=${2:-3}
HERE=$(cd "$(dirname "$0")/.." && pwd)
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
BIN=$HERE/bench/target/release/exp010-bench
W=$M/_tmp/20261031-exp010/gtcrn_dns3
G=$M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
F=$HOME/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx
# tract 不支持 SplitToSequence：用 tools/fem_strip_seq.py 改写成 Gather 的版本（fem-parity 证明输出不变）
FT=$M/_tmp/20261102-exp010/fastenhancer_m_noseq.onnx
WAV=$M/resources/gtcrn/test_wavs/mix.wav
OUT=$HERE/results/$ID
[ -e "$OUT" ] && { echo "已有 $OUT，不覆盖"; exit 1; }
mkdir -p "$OUT"

{
  echo "date=$(date '+%F %T %z')"
  echo "machine=$(sysctl -n hw.model) $(sysctl -n machdep.cpu.brand_string)"
  echo "os=$(sw_vers -productVersion) ($(sw_vers -buildVersion))"
  echo "power=$(pmset -g batt | head -1 | sed "s/.*'\(.*\)'.*/\1/")"
  echo "rustc=$(rustc --version)"
  echo "load=$(sysctl -n vm.loadavg)"
  echo "git=$(git -C "$HERE" rev-parse --short HEAD)"
  shasum -a 256 "$W.bin" "$G" "$F" "$FT" "$WAV" "$BIN"
} > "$OUT/env.txt"

$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native >> "$OUT/parity.jsonl"
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs tract  >> "$OUT/parity.jsonl"
$BIN fem-parity "$F" "$WAV" --tract-onnx "$FT" >> "$OUT/parity.jsonl"

# 交替跑，降低机器状态漂移对某一后端的偏向
for r in $(seq 1 "$ROUNDS"); do
  for k in ort native tract; do
    a=$G; [ $k = native ] && a=$W
    $BIN gtcrn-bench $k "$a" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"
  done
  for k in ort tract; do
    a=$F; [ $k = tract ] && a=$FT
    $BIN fem-bench $k "$a" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"
  done
done
echo "load_after=$(sysctl -n vm.loadavg)" >> "$OUT/env.txt"
cat "$OUT/parity.jsonl" "$OUT/bench.jsonl"
