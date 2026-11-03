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
  echo "uptime_before: $(uptime)"
  echo "git=$(git -C "$HERE" rev-parse --short HEAD)"
  shasum -a 256 "$W.bin" "$G" "$F" "$FT" "$WAV" "$BIN"
  echo "ort=静态链接进 bench 可执行文件（上面那行 sha256 即含 ORT）；crate 版本：$(grep -A1 '^name = "ort"$' "$HERE/bench/Cargo.lock" | tail -1)"
  ps -Ao %cpu,comm -r | head -4
} > "$OUT/env.txt"

$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native >> "$OUT/parity.jsonl"
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs tract  >> "$OUT/parity.jsonl"
$BIN fem-parity "$F" "$WAV" --vs tract --b-onnx "$FT" >> "$OUT/parity.jsonl"
$BIN fem-parity "$F" "$WAV" --vs ort --b-onnx "$FT" >> "$OUT/parity.jsonl"          # 单独证明改写等价
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs native --frames 37500 --reset-at 20000 >> "$OUT/parity.jsonl"  # 10 分钟长跑 + reset
$BIN gtcrn-parity "$W" "$G" "$WAV" --vs tract --frames 6201 >> "$OUT/parity.jsonl"   # 与基准等长

# tract 复用同一个运行状态（已知会算错）：留证据，并把「每帧新开状态」的代价和分配次数单独量出来
EXP010_TRACT_REUSE=1 $BIN gtcrn-parity "$W" "$G" "$WAV" --vs tract | sed 's/"vs":"tract"/"vs":"tract-reuse-known-wrong"/' >> "$OUT/parity.jsonl"
EXP010_TRACT_REUSE=1 $BIN gtcrn-bench tract "$G" "$WAV" | sed 's/"backend":"tract"/"backend":"tract-reuse-known-wrong"/' >> "$OUT/bench-extra.jsonl"

# 交替跑，且每轮把后端顺序轮换一位，避免发热 / 频率漂移总偏向同一个后端
G3=(ort native tract); F2=(ort tract)
for r in $(seq 1 "$ROUNDS"); do
  o=$(( (r - 1) % 3 )); GO=("${G3[@]:$o}" "${G3[@]:0:$o}")
  o=$(( (r - 1) % 2 )); FO=("${F2[@]:$o}" "${F2[@]:0:$o}")
  for k in "${GO[@]}"; do
    a=$G; [ $k = native ] && a=$W
    $BIN gtcrn-bench $k "$a" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"
  done
  for k in "${FO[@]}"; do
    a=$F; [ $k = tract ] && a=$FT
    $BIN fem-bench $k "$a" "$WAV" | sed "s/^{/{\"round\":$r,/" >> "$OUT/bench.jsonl"
  done
done
$BIN amx-probe > "$OUT/amx.jsonl"   # 小矩阵：Accelerate（AMX / SME）对手写循环
echo "load_after=$(sysctl -n vm.loadavg)" >> "$OUT/env.txt"
echo "uptime_after: $(uptime)" >> "$OUT/env.txt"
cat "$OUT/parity.jsonl" "$OUT/bench.jsonl" "$OUT/bench-extra.jsonl"
