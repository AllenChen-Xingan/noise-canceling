#!/usr/bin/env bash
# exp012 第三段计时（D79：「快 / 省电」以能效核、16 ms 实时节奏为准）：全部按实时节奏、rt 线程设置跑。
# 每轮：节奏下的峰值探针（hw-peak --pace-ms 16）· 4 个变体逐层计时（native · v2 · v2-fast · v2-fast-pf）· v2-fast-pf 整帧。
# 每轮轮换变体顺序，抵消时间漂移。用法：run_paced.sh <结果目录名> [轮数=3]
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
    ac) echo "$src" | grep -q "AC Power" || { echo "POWER_MODE=ac 但没插电（$src），停下" >&2; exit 2; } ;;
    battery) echo "$src" | grep -q "Battery Power" || { echo "POWER_MODE=battery 但插着电（$src），停下" >&2; exit 2; } ;;
    *) echo "POWER_MODE 只收 ac / battery" >&2; exit 2 ;;
  esac
}
ac_check
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
[ -e "$out/layers.jsonl" ] && { echo "已有 $out/layers.jsonl，换个目录名（不覆盖）"; exit 1; }

{
  echo "date: $(date '+%F %T %z')"
  echo "machine: $(sysctl -n machdep.cpu.brand_string) · P 核 $(sysctl -n hw.perflevel0.physicalcpu) · E 核 $(sysctl -n hw.perflevel1.physicalcpu)"
  echo "os: macOS $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
  echo "power: $(pmset -g batt | head -1)"
  echo "load: $(sysctl -n vm.loadavg)"
  echo "git: $(git -C "$repo" rev-parse --short HEAD)"
  echo "bin_sha256: $(shasum -a 256 "$bin" | cut -d' ' -f1)"
  echo "heavy_lock: $(cat "$main_dir/_tmp/heavy.lock/owner" 2>/dev/null || echo 无)"
  echo "power_mode: $POWER_MODE"; echo "battery_start: $(batt_line)"
} > "$out/env.txt"
tag() { sed "s/^{/{\"round\":$1,/"; }

for r in $(seq 1 "$rounds"); do
  ac_check
  echo "round $r: 节奏下峰值探针" >&2
  "$bin" hw-peak --qos rt --pace-ms 16 --bursts 600 | tag "$r" >> "$out/hwpeak_paced.jsonl"
  case $r in
    1) order="native native-v2 native-v2-fast native-v2-fast-pf" ;;
    2) order="native-v2-fast-pf native native-v2 native-v2-fast" ;;
    *) order="native-v2-fast native-v2-fast-pf native native-v2" ;;
  esac
  for v in $order; do
    echo "round $r: 逐层 $v（16 ms 节奏）" >&2
    "$bin" gtcrn-layers "$w" "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 16 --variant "$v" | tag "$r" >> "$out/layers.jsonl"
  done
  "$bin" gtcrn-bench native-v2-fast-pf "$w" "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 16 | tag "$r" >> "$out/bench.jsonl"
  echo "load_r$r: $(sysctl -n vm.loadavg) · $(batt_line)" >> "$out/env.txt"
done
echo "load_end: $(sysctl -n vm.loadavg)" >> "$out/env.txt"
echo "battery_end: $(batt_line)" >> "$out/env.txt"
echo "done → $out" >&2
