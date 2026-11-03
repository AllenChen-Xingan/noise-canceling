#!/usr/bin/env bash
# Rust ORT（bench）对 Python onnxruntime（exp001 口径）：同一 GTCRN、逐位相同的输入，交替跑 N 轮。要先拿到重活名额。
# 用法：bash tools/run_py_vs_rust.sh <build-id> [轮数=3]
set -euo pipefail
ID=${1:?build-id}; ROUNDS=${2:-3}
HERE=$(cd "$(dirname "$0")/.." && pwd)
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
BIN=$HERE/bench/target/release/exp010-bench
PY=$M/_tmp/exp002-venv/bin/python   # onnxruntime 1.31.1，与 Rust 侧 ORT 1.31 同版本
G=$M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
WAV=$M/resources/gtcrn/test_wavs/mix.wav
DUMP=$M/_tmp/20261102-exp010/$ID          # 转储约 25 MB，放仓外
OUT=$HERE/results/$ID
[ -e "$OUT" ] && { echo "已有 $OUT，不覆盖"; exit 1; }
mkdir -p "$OUT" "$DUMP"
{
  echo "date=$(date '+%F %T %z')"; echo "machine=$(sysctl -n hw.model)"; echo "os=$(sw_vers -productVersion)"
  echo "power=$(pmset -g batt | head -1 | sed "s/.*'\(.*\)'.*/\1/")"; echo "uptime_before: $(uptime)"
  echo "python=$($PY -c 'import sys,onnxruntime,numpy;print(sys.version.split()[0],"ort",onnxruntime.__version__,"numpy",numpy.__version__)')"
  ps -Ao %cpu,comm -r | head -4
} > "$OUT/env.txt"
# 先转储一次输入与 Rust ORT 输出（这次的计时也记下，不进对比）
$BIN gtcrn-bench ort "$G" "$WAV" --dump-io "$DUMP/io" > "$OUT/dump-run.json"
cp "$DUMP/io.json" "$OUT/io.json"
for r in $(seq 1 "$ROUNDS"); do
  $BIN gtcrn-bench ort "$G" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"
  $PY -I "$HERE/tools/py_ort_bench.py" "$G" "$DUMP/io" | sed "s/^{/{\"round\": $r, /" >> "$OUT/bench.jsonl"
done
echo "uptime_after: $(uptime)" >> "$OUT/env.txt"
cat "$OUT/bench.jsonl"
