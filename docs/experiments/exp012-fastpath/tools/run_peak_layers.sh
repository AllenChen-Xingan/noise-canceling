#!/usr/bin/env bash
# exp012 计时：本机峰值（hw-peak）与手写 GTCRN 逐层计时（gtcrn-layers），性能核（rt）与能效核（bg）交替跑。
# 用法：run_peak_layers.sh <结果目录名> [轮数=3] [变体=native]
# 前提：已拿到重活名额、主目录 _tmp/heavy.lock 由大总管建好；bench 已 cargo build --release。
set -euo pipefail
# 防系统睡眠 + 必须插电（20261102-v2 踩坑：拔电后空闲睡眠 42 分钟，电池模式下性能核降到约一半频率）
if [ -z "${VDN_CAFFEINATED:-}" ]; then exec env VDN_CAFFEINATED=1 caffeinate -dims bash "$0" "$@"; fi
ac_check() { pmset -g batt | head -1 | grep -q "AC Power" || { echo "没插电（$(pmset -g batt | head -1)），停下：插电后重跑" >&2; exit 2; }; }
ac_check
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../../.." && pwd)"
main_dir="${VDN_MAIN:-/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling}"
bin="$repo/docs/experiments/exp010-inference-bench/bench/target/release/exp010-bench"
w="$main_dir/_tmp/20261031-exp010/gtcrn_dns3"
wav="$main_dir/resources/gtcrn/test_wavs/mix.wav"
id="${1:?结果目录名}"
rounds="${2:-3}"
variant="${3:-native}"
out="$repo/docs/experiments/exp012-fastpath/results/$id"
mkdir -p "$out"
[ -e "$out/hwpeak.jsonl" ] && { echo "已有 $out/hwpeak.jsonl，换个目录名（不覆盖）"; exit 1; }

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

for r in $(seq 1 "$rounds"); do
  ac_check
  for q in rt bg; do
    echo "round $r hw-peak $q" >&2
    "$bin" hw-peak --qos "$q" --reps 9 | sed "s/^{/{\"round\":$r,/" >> "$out/hwpeak.jsonl"
  done
done
for r in $(seq 1 "$rounds"); do
  # 连续跑（帧间不睡）：纯计算口径，可与 exp010 的 0.088 ms 对照
  "$bin" gtcrn-layers "$w" "$wav" --frames 6000 --warmup 200 --qos rt --pace-ms 0 --variant "$variant" \
    | sed "s/^{/{\"round\":$r,/" >> "$out/layers.jsonl"
  for q in rt bg; do
    echo "round $r gtcrn-layers $q paced" >&2
    "$bin" gtcrn-layers "$w" "$wav" --frames 6000 --warmup 200 --qos "$q" --pace-ms 16 --variant "$variant" \
      | sed "s/^{/{\"round\":$r,/" >> "$out/layers.jsonl"
  done
done
echo "load_end: $(sysctl -n vm.loadavg)" >> "$out/env.txt"
echo "done → $out" >&2
