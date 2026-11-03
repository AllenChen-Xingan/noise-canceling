#!/bin/sh
# exp010 耗电测量 · 要 sudo，由人跑（common.md 第 5 条）。插电、合盖以外无其他重活。
# 用法：sudo ./power.sh <名字> <要测的命令...>
# 做法：先空闲 30 s 取基线，再边跑命令边采样，每 500 ms 一次，CPU / GPU / 神经引擎功率都记。
set -e
name=$1; shift
out=${OUT:-../results/power}
mkdir -p "$out"
powermetrics -i 500 -n 60 --samplers cpu_power,gpu_power,ane_power > "$out/$name.idle.txt"
powermetrics -i 500 --samplers cpu_power,gpu_power,ane_power > "$out/$name.run.txt" &
pm=$!
"$@" > "$out/$name.bench.json"
kill $pm
echo "写好：$out/$name.{idle,run}.txt"
