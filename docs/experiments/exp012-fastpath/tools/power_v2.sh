#!/usr/bin/env bash
# exp012 耗电：原写法（native）· v2 · v2-fast，按实时节奏（每 16 ms 一帧）各跑 60 秒，同时用 powermetrics 采 CPU / GPU / 神经引擎功率。
# 口径同 exp010 tools/power_run.sh 与 019 standard.md §6：线程设置 rt（D74 默认，不加工作组）· 3 轮交替、每轮轮换顺序 · 每轮一段 60 秒空闲作噪声底
# · 只采稳态（bench 预热完写就绪文件后才开始采 50 秒）· 报三项合计。
# 要 sudo（powermetrics 只能 root 跑），由人来跑：
#   1. 插电，关掉其他重活（编译、视频、别的基准）；大总管先建 _tmp/heavy.lock
#   2. 在 fastpath 工作目录（/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling-fastpath）跑：
#        sudo bash docs/experiments/exp012-fastpath/tools/power_v2.sh 20261102-pw1
#   3. 约 13 分钟（3 轮 × 4 段 × 约 62 秒）；结果写 results/<目录名>/power/，不覆盖已有目录
# 每帧能耗 ≈（后端三项合计平均功率 − 同一轮空闲的平均功率）× 16 ms。
# bench 输出里有 win_start_unix / win_end_unix（计时窗），可再按窗口精确截取样本（applebe 的 power_parse.py v2 口径）。
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
ID=${1:?结果目录名}
HERE=$(cd "$(dirname "$0")/.." && pwd)
REPO=$(cd "$HERE/../../.." && pwd)
M=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
U=${SUDO_USER:-$USER}
BIN=$REPO/docs/experiments/exp010-inference-bench/bench/target/release/exp010-bench
W=$M/_tmp/20261031-exp010/gtcrn_dns3
WAV=$M/resources/gtcrn/test_wavs/mix.wav
OUT=$HERE/results/$ID/power
[ -x "$BIN" ] || { echo "没有 $BIN，先 cargo build --release"; exit 1; }
[ -e "$OUT" ] && { echo "已有 $OUT，不覆盖"; exit 1; }
mkdir -p "$OUT"
trap 'chown -R "$U" "$HERE/results/$ID"' EXIT
chown -R "$U" "$HERE/results/$ID"
{
  date '+%F %T %z'; sysctl -n machdep.cpu.brand_string; sw_vers -productVersion; pmset -g batt | head -1
  echo "load: $(sysctl -n vm.loadavg)"; echo "git: $(git -C "$REPO" rev-parse --short HEAD 2>/dev/null || echo ?)"
  echo "bin_sha256: $(shasum -a 256 "$BIN" | cut -d' ' -f1)"
  echo "power_mode: $POWER_MODE"; echo "battery_start: $(batt_line)"
} > "$OUT/env.txt"

run() { # 名字 · 命令（命令为空 = 空闲对照）
  local name=$1; shift
  local ready="$OUT/$name.ready"
  if [ $# -gt 0 ]; then
    sudo -u "$U" "$@" --ready-file "$ready" > "$OUT/$name.bench.json" &
    local bp=$!
    for _ in $(seq 1 300); do [ -e "$ready" ] && break; sleep 0.1; done
    [ -e "$ready" ] || { echo "$name 30 秒内没就绪" >&2; kill $bp; exit 1; }
    powermetrics --samplers cpu_power,gpu_power,ane_power -i 1000 -n 50 > "$OUT/$name.powermetrics.txt"
    wait $bp
  else
    powermetrics --samplers cpu_power,gpu_power,ane_power -i 1000 -n 50 > "$OUT/$name.powermetrics.txt"
  fi
  awk '/^Combined Power \(CPU \+ GPU \+ ANE\): [0-9.]+ mW/ {t+=$(NF-1); n++} /^CPU Power: [0-9.]+ mW/ {c+=$3; m++} END {if (n==0) {print "'"$name"' 没解析到 Combined Power 行，见原文" > "/dev/stderr"; exit 1} printf "%s mean_combined_mW=%.1f mean_cpu_mW=%.1f samples=%d\n", "'"$name"'", t/n, (m?c/m:-1), n}' "$OUT/$name.powermetrics.txt" | tee -a "$OUT/summary.txt"
}

for r in 1 2 3; do
  ac_check
  run idle_r$r
  case $r in
    1) order="native native-v2 native-v2-fast" ;;
    2) order="native-v2 native-v2-fast native" ;;
    3) order="native-v2-fast native native-v2" ;;
  esac
  for k in $order; do
    run "${k}_r$r" "$BIN" gtcrn-bench "$k" "$W" "$WAV" --frames 3750 --pace-ms 16 --qos rt
  done
done
echo "load_end: $(sysctl -n vm.loadavg)" >> "$OUT/env.txt"
echo "battery_end: $(batt_line)" >> "$OUT/env.txt"
echo "完成：$OUT/summary.txt"
