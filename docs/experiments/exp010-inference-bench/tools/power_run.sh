#!/usr/bin/env bash
# exp010 耗电：按实时节奏（GTCRN 每 16 ms 一帧、FE-M 每 10 ms 一帧）各跑 60 秒，同时用 powermetrics 采 CPU 功率。
# 要 sudo（powermetrics 只能 root 跑），所以由人来跑：
#   1. 插电，关掉其他重活（编译、视频、别的基准）
#   2. 在仓库根目录跑：sudo bash docs/experiments/exp010-inference-bench/tools/power_run.sh 20261102-p1
#   3. 跑完约 8 分钟；结果写 results/<build-id>/power/，不覆盖已有目录
# 额外跑一段「空闲」作对照：同样 60 秒什么都不跑，只采功率。
set -euo pipefail
ID=${1:?build-id}
HERE=$(cd "$(dirname "$0")/.." && pwd)
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
U=${SUDO_USER:-$USER}
UH=$(eval echo "~$U")
BIN=$HERE/bench/target/release/exp010-bench
W=$M/_tmp/20261031-exp010/gtcrn_dns3
G=$M/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
F=$UH/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx
FT=$M/_tmp/20261102-exp010/fastenhancer_m_noseq.onnx
WAV=$M/resources/gtcrn/test_wavs/mix.wav
OUT=$HERE/results/$ID/power
[ -e "$OUT" ] && { echo "已有 $OUT，不覆盖"; exit 1; }
mkdir -p "$OUT"
{ date '+%F %T %z'; sysctl -n hw.model; sw_vers -productVersion; pmset -g batt | head -1; uptime; } > "$OUT/env.txt"

run() { # 名字 · 命令（命令为空 = 空闲对照）
  local name=$1; shift
  powermetrics --samplers cpu_power -i 1000 -n 62 > "$OUT/$name.powermetrics.txt" &
  local pm=$!
  sleep 1
  if [ $# -gt 0 ]; then sudo -u "$U" "$@" > "$OUT/$name.bench.json"; else sleep 60; fi
  wait $pm
  # 每秒一行「CPU Power: N mW」，取平均
  awk '/^CPU Power/ {s+=$3; n++} END {printf "%s mean_cpu_mW=%.1f samples=%d\n", "'"$name"'", s/n, n}' "$OUT/$name.powermetrics.txt" | tee -a "$OUT/summary.txt"
}

run idle
run gtcrn_ort    "$BIN" gtcrn-bench ort    "$G" "$WAV" --frames 3750 --pace-ms 16
run gtcrn_native "$BIN" gtcrn-bench native "$W" "$WAV" --frames 3750 --pace-ms 16
run gtcrn_tract  "$BIN" gtcrn-bench tract  "$G" "$WAV" --frames 3750 --pace-ms 16
run fem_ort      "$BIN" fem-bench ort      "$F" "$WAV" --frames 6000 --pace-ms 10
run fem_tract    "$BIN" fem-bench tract    "$FT" "$WAV" --frames 6000 --pace-ms 10
chown -R "$U" "$HERE/results/$ID"
echo "完成：$OUT/summary.txt"
