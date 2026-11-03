#!/usr/bin/env bash
# exp012 第二段计时：修正后的 hw-peak 重跑 · native 对照（复测 peak1 的 rt 组）· native-v2 / native-v2-fast · 前后处理。
# 各变体按轮交替跑（同一轮内 native → v2 → v2-fast），抵消时间漂移。
# 用法：run_v2.sh <结果目录名> [轮数=3]
# 前提：已拿到重活名额、主目录 _tmp/heavy.lock 由大总管建好；bench 已 cargo build --release。
set -euo pipefail
# 防系统睡眠 + 电源模式（20261102-v2 踩坑：拔电后空闲睡眠 42 分钟，电池模式下性能核降到约一半频率）
# POWER_MODE=ac（默认）：必须插电；POWER_MODE=battery：必须用电池（测用户不插电的场景，人定 2026-11-02）
if [ -z "${VDN_CAFFEINATED:-}" ]; then exec env VDN_CAFFEINATED=1 caffeinate -dims bash "$0" "$@"; fi
POWER_MODE="${POWER_MODE:-ac}"
batt_line() { pmset -g batt | sed -n 2p | sed 's/^[[:space:]]*//'; }
ac_check() {
  local src; src=$(pmset -g batt | head -1)
  case "$POWER_MODE" in
    ac) echo "$src" | grep -q "AC Power" || { echo "POWER_MODE=ac 但没插电（${src}），停下" >&2; exit 2; } ;;
    battery) echo "$src" | grep -q "Battery Power" || { echo "POWER_MODE=battery 但插着电（${src}），停下" >&2; exit 2; } ;;
    *) echo "POWER_MODE 只收 ac / battery" >&2; exit 2 ;;
  esac
}
ac_check
# 开跑前等负载降下来（大总管要求：1 分钟负载 < 3，每 15 秒查一次，最多 5 分钟）
load1() { sysctl -n vm.loadavg | awk '{print $2}'; }
for _ in $(seq 1 20); do
  awk -v l="$(load1)" 'BEGIN{exit !(l < 3)}' && break
  echo "等负载降下来：1 分钟负载 $(load1)" >&2; sleep 15
done
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
  echo "load_at_start: $(sysctl -n vm.loadavg)"
  echo "git: $(git -C "$repo" rev-parse --short HEAD)"
  echo "bin_sha256: $(shasum -a 256 "$bin" | cut -d' ' -f1)"
  echo "heavy_lock: $(cat "$main_dir/_tmp/heavy.lock/owner" 2>/dev/null || echo 无)"
  echo "power_mode: $POWER_MODE"; echo "battery_start: $(batt_line)"
} > "$out/env.txt"
tag() { sed "s/^{/{\"round\":$1,/"; }

for r in $(seq 1 "$rounds"); do
  ac_check
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
  echo "load_r$r: $(sysctl -n vm.loadavg) · $(batt_line)" >> "$out/env.txt"
done
"$bin" dsp-bench "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 16 | tag 1 >> "$out/dsp.jsonl"
echo "load_end: $(sysctl -n vm.loadavg)" >> "$out/env.txt"
echo "battery_end: $(batt_line)" >> "$out/env.txt"
echo "done → $out" >&2
