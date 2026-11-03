#!/usr/bin/env bash
# exp012 短计时（≤ 10 分钟，大总管要求）：几个变体按 16 ms 节奏交替比，外加 ui 连续跑逐层（看各段省了多少）。
# 用法：run_short.sh <结果目录名> "<变体1> <变体2> …" [轮数=3] [节奏帧数=2000]
# 节奏组：每条 2000 帧 × 16 ms = 32 秒；3 轮 × 3 个变体 ≈ 5 分钟。逐层：qos ui（rt 连续跑会被降级，见 README「口径坑」）
set -euo pipefail
# 防系统睡眠 + 电源模式（POWER_MODE=ac 默认 / battery）
if [ -z "${VDN_CAFFEINATED:-}" ]; then exec env VDN_CAFFEINATED=1 caffeinate -dims bash "$0" "$@"; fi
POWER_MODE="${POWER_MODE:-ac}"
batt_line() { pmset -g batt | sed -n 2p | sed 's/^[[:space:]]*//'; }
ac_check() {
  local src; src=$(pmset -g batt | head -1)
  case "${POWER_MODE}" in
    ac) echo "${src}" | grep -q "AC Power" || { echo "POWER_MODE=ac 但没插电（${src}），停下" >&2; exit 2; } ;;
    battery) echo "${src}" | grep -q "Battery Power" || { echo "POWER_MODE=battery 但插着电（${src}），停下" >&2; exit 2; } ;;
    *) echo "POWER_MODE 只收 ac / battery" >&2; exit 2 ;;
  esac
}
ac_check
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
V="${2:?变体列表}"
rounds="${3:-3}"
nf="${4:-2000}"
out="$repo/docs/experiments/exp012-fastpath/results/${id}"
[ -e "${out}" ] && { echo "已有 ${out}，换个目录名（不覆盖）"; exit 1; }
mkdir -p "${out}"
{
  echo "date: $(date '+%F %T %z')"
  echo "machine: $(sysctl -n machdep.cpu.brand_string) · P 核 $(sysctl -n hw.perflevel0.physicalcpu) · E 核 $(sysctl -n hw.perflevel1.physicalcpu)"
  echo "os: macOS $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
  echo "power: $(pmset -g batt | head -1)"
  echo "load_at_start: $(sysctl -n vm.loadavg)"
  echo "git: $(git -C "${repo}" rev-parse --short HEAD)"
  echo "bin_sha256: $(shasum -a 256 "${bin}" | cut -d' ' -f1)"
  echo "heavy_lock: $(cat "${main_dir}/_tmp/heavy.lock/owner" 2>/dev/null || echo 无)"
  echo "power_mode: ${POWER_MODE}"; echo "battery_start: $(batt_line)"
  echo "variants: ${V} · rounds: ${rounds} · paced_frames: ${nf}"
} > "${out}/env.txt"
tag() { sed "s/^{/{\"round\":$1,/"; }

for r in $(seq 1 "${rounds}"); do
  ac_check
  # 每轮把变体顺序轮转一位，抵消时间漂移
  set -- ${V}
  for _ in $(seq 2 "${r}"); do set -- "${@:2}" "$1"; done
  for v in "$@"; do
    echo "round ${r}: ${v} 16 ms 节奏" >&2
    "${bin}" gtcrn-bench "${v}" "${w}" "${wav}" --frames "${nf}" --warmup 200 --qos rt --pace-ms 16 | tag "${r}" >> "${out}/bench.jsonl"
  done
  for v in "$@"; do
    "${bin}" gtcrn-layers "${w}" "${wav}" --frames 6000 --warmup 200 --qos ui --pace-ms 0 --variant "${v}" | tag "${r}" >> "${out}/layers.jsonl"
  done
  echo "load_r${r}: $(sysctl -n vm.loadavg) · $(batt_line)" >> "${out}/env.txt"
done
echo "battery_end: $(batt_line)" >> "${out}/env.txt"
echo "done → ${out}" >&2
