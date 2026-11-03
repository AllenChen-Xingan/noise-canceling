#!/usr/bin/env bash
# exp012 第二段计时：修正后的 hw-peak 重跑 · native 对照（复测 peak1 的 rt 组）· native-v2 / native-v2-fast · 前后处理。
# 各变体按轮交替跑（同一轮内 native → v2 → v2-fast），抵消时间漂移。
# 用法：run_v2.sh <结果目录名> [轮数=3]
# 前提：已拿到重活名额、主目录 _tmp/heavy.lock 由大总管建好；bench 已 cargo build --release。
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
main_dir="${VDN_MAIN:-/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling}"
bin="$repo/docs/experiments/exp010-inference-bench/bench/target/release/exp010-bench"
w="$main_dir/_tmp/20261031-exp010/gtcrn_dns3"
wav="$main_dir/resources/gtcrn/test_wavs/mix.wav"
id="${1:?结果目录名}"
rounds="${2:-3}"
out="$repo/docs/experiments/exp012-fastpath/results/$id"
mkdir -p "$out"
[ -e "$out/bench.jsonl" ] && { echo "已有 $out/bench.jsonl，换个目录名（不覆盖）"; exit 1; }
V="native native-v2 native-v2-fast"

{
  echo "date: $(date '+%F %T %z')"
  echo "machine: $(sysctl -n machdep.cpu.brand_string) · P 核 $(sysctl -n hw.perflevel0.physicalcpu) · E 核 $(sysctl -n hw.perflevel1.physicalcpu)"
  echo "os: macOS $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
  echo "power: $(pmset -g batt | head -1)"
  echo "load: $(sysctl -n vm.loadavg)"
  echo "git: $(git -C "$repo" rev-parse --short HEAD)"
  echo "bin_sha256: $(shasum -a 256 "$bin" | cut -d' ' -f1)"
  echo "heavy_lock: $(cat "$main_dir/_tmp/heavy.lock/owner" 2>/dev/null || echo 无)"
} > "$out/env.txt"
tag() { sed "s/^{/{\"round\":$1,/"; }

for r in $(seq 1 "$rounds"); do
  echo "round $r: hw-peak" >&2
  for q in rt bg; do "$bin" hw-peak --qos "$q" --reps 7 | tag "$r" >> "$out/hwpeak.jsonl"; done
  echo "round $r: 连续跑（逐层 + 整帧）" >&2
  for v in $V; do
    "$bin" gtcrn-layers "$w" "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 0 --variant "$v" | tag "$r" >> "$out/layers.jsonl"
    "$bin" gtcrn-bench "$v" "$w" "$wav" --frames 6000 --warmup 200 --qos rt | tag "$r" >> "$out/bench.jsonl"
  done
  "$bin" dsp-bench "$wav" --frames 6000 --warmup 200 --qos rt | tag "$r" >> "$out/dsp.jsonl"
  echo "round $r: 实时节奏 16 ms（rt）" >&2
  for v in $V; do
    "$bin" gtcrn-bench "$v" "$w" "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 16 | tag "$r" >> "$out/bench.jsonl"
  done
  echo "load_r$r: $(sysctl -n vm.loadavg)" >> "$out/env.txt"
done
"$bin" dsp-bench "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 16 | tag 1 >> "$out/dsp.jsonl"
echo "load_end: $(sysctl -n vm.loadavg)" >> "$out/env.txt"
echo "done → $out" >&2
