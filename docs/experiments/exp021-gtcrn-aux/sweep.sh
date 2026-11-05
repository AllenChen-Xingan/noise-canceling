#!/usr/bin/env bash
# exp021 混回参数扫描：对一个原始输出外部目录重混若干档，再用关保命的派生测试集一次打分（不带评分模型）。
# 用法（在 worktree 根目录跑，需重活名额）：
#   docs/experiments/exp021-gtcrn-aux/sweep.sh <前缀> <原始输出目录（相对主目录）> [预热段目录]
# 内置模型（GTCRN）必须给预热段目录（raw 子命令写的 <out>.head），否则与内置成绩单差约 0.1 dB 降噪量（runs/20261105-parity-gtcrn）
# 产物：主目录 _tmp/030/<前缀>/<档>/（音频）· _tmp/030/sweep-<前缀>-<时间>/（成绩单 + clips.jsonl）
set -euo pipefail
prefix="$1"
src="$2"
head=()
[ $# -ge 3 ] && head=(--head "$3")
B=target/release/exp021-remix
E=target/release/vdn-eval
suite=_tmp/030/scenes-safety-off.json

# 档位：说话时混回量（open_db）× 停顿下限（floor_db）；一次只动一个，另一个取默认（−6 / −12）
grid=(
  "-3 -12" "-6 -12" "-9 -12" "-12 -12"
  "-6 -6" "-6 -9" "-6 -18" "-6 -24"
)
systems=()
for g in "${grid[@]}"; do
  read -r o f <<<"$g"
  dir="_tmp/030/${prefix}/o${o#-}-f${f#-}"
  [ -e "../noise_canceling/${dir}" ] || caffeinate -i "$B" remix --src "$src" --out "$dir" --prefix "$prefix" --open-db "$o" --floor-db "$f" "${head[@]}"
  systems+=("external:${dir}")
done
nomix="_tmp/030/${prefix}/nomix"
[ -e "../noise_canceling/${nomix}" ] || caffeinate -i "$B" remix --src "$src" --out "$nomix" --prefix "$prefix" --off
systems+=("external:${nomix}")

list=$(IFS=,; echo "${systems[*]}")
out="../noise_canceling/_tmp/030/sweep-${prefix}-$(date +%Y%m%d-%H%M%S)"
caffeinate -i "$E" scorecard --suite "$suite" --system "$list" --out "$out"
echo "扫描完成：$out"
