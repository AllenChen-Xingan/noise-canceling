#!/usr/bin/env bash
# exp010 耗电：按实时节奏（GTCRN 每 16 ms 一帧、FE-M 每 10 ms 一帧）各跑 60 秒，同时用 powermetrics 采 CPU / GPU / 神经引擎功率。
# 口径对齐 standard.md §6：线程设置 rt（D74 默认）、3 轮交替、报三项合计；tract 已知不过挡线，不再测。
# 要 sudo（powermetrics 只能 root 跑），所以由人来跑：
#   1. 插电，关掉其他重活（编译、视频、别的基准）
#   2. 在仓库根目录跑：sudo bash docs/experiments/exp010-inference-bench/tools/power_run.sh 20261102-p1
#   3. 跑完约 13 分钟（3 轮 × 4 段 × 约 62 秒）；结果写 results/<build-id>/power/，不覆盖已有目录
# 额外跑一段「空闲」作对照：同样 60 秒什么都不跑，只采功率。
# 每帧能耗 ≈（后端三项合计平均功率 − 同一轮空闲的平均功率）× 帧间隔。采样窗只覆盖预热完之后的稳态 50 秒。
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
trap 'chown -R "$U" "$HERE/results/$ID"' EXIT   # 中途失败也把文件还给普通用户
{ date '+%F %T %z'; sysctl -n hw.model; sw_vers -productVersion; pmset -g batt | head -1; uptime; } > "$OUT/env.txt"

run() { # 名字 · 命令（命令为空 = 空闲对照）
  # 只采稳态：bench 预热完写就绪文件，之后才开始采 50 秒（bench 计时段约 60 秒，采样窗落在里面）。
  # 输入生成、ORT 准备、加载、预热这些启动成本不进平均（codex 第 4 轮 #4）
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
  # 每秒一行「Combined Power (CPU + GPU + ANE): N mW」，取平均；同时报 CPU 单项
  awk '/^Combined Power \(CPU \+ GPU \+ ANE\): [0-9.]+ mW/ {t+=$(NF-1); n++} /^CPU Power: [0-9.]+ mW/ {c+=$3; m++} END {if (n==0) {print "'"$name"' 没解析到 Combined Power 行，见原文" > "/dev/stderr"; exit 1} printf "%s mean_combined_mW=%.1f mean_cpu_mW=%.1f samples=%d\n", "'"$name"'", t/n, (m?c/m:-1), n}' "$OUT/$name.powermetrics.txt" | tee -a "$OUT/summary.txt"
}

# 线程设置用 D74 默认（rt = time-constraint，不加工作组）；3 轮交替，每轮轮换顺序，每轮都有一段空闲作噪声底
for r in 1 2 3; do
  run idle_r$r
  case $r in
    1) order="ort native fem_ort" ;;
    2) order="native fem_ort ort" ;;
    3) order="fem_ort ort native" ;;
  esac
  for k in $order; do
    case $k in
      ort)     run gtcrn_ort_r$r    "$BIN" gtcrn-bench ort    "$G" "$WAV" --frames 3750 --pace-ms 16 --qos rt ;;
      native)  run gtcrn_native_r$r "$BIN" gtcrn-bench native "$W" "$WAV" --frames 3750 --pace-ms 16 --qos rt ;;
      fem_ort) run fem_ort_r$r      "$BIN" fem-bench ort      "$F" "$WAV" --frames 6000 --pace-ms 10 --qos rt ;;
    esac
  done
done
echo "完成：$OUT/summary.txt"
