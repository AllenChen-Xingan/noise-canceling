#!/bin/bash
# exp010 · 实时线程设置 × 每帧耗电（D74 之后的版本）。**要人用 sudo 跑**（powermetrics），且要名额独占。
#   sudo tools/rt-matrix.sh <模型 build 目录（含 gtcrn_unrolled_fp32.mlmodelc 与 gtcrn.in/ort.f32）> [native 二进制 权重前缀 wav]
# 基线 = D74 的默认：time-constraint 开（period 16 · computation 1 · constraint 3 ms，同 rustbe 的 --qos rt）、不加工作组。
# 重点（推翻 D74 依据的那一问）：自建工作组区间、截止期报紧到 3 / 2 / 1 ms，系统会不会提频、每帧是否更省电。
# 每个设置按 16 ms 节奏跑 60 s（3750 帧），同时 powermetrics 每 500 ms 采一次；开头先空闲 30 s 取基线。
# 权限：只有 powermetrics 以 root 跑；基准程序与 power_parse.py 用 `sudo -u "$SUDO_USER"` 以人的账号跑
#   （产品以普通用户跑，root 下的线程调度可能不同；也不必给自编译的程序 root）。结果目录与文件先以普通用户建好。
# 输出：apple/results/<build id>/rt.jsonl（每帧耗时 · CPU 占用 · 超 3 ms 帧数）+ power/<设置>.txt；最后 tools/power_parse.py 算每帧耗电。
set -u
[ "$(id -u)" = 0 ] || { echo "要 sudo：powermetrics 需要 root"; exit 1; }
[ -n "${SUDO_USER:-}" ] && [ "$SUDO_USER" != root ] || { echo "要从普通账号用 sudo 启动（SUDO_USER 为空）：基准程序必须以人的账号跑"; exit 1; }
U() { sudo -u "$SUDO_USER" "$@"; }   # 以人的账号执行
pm=""
trap '[ -n "$pm" ] && kill "$pm" 2>/dev/null; [ -n "$pm" ] && wait "$pm" 2>/dev/null' EXIT INT TERM HUP   # 中途停下也不留下 powermetrics
HERE=$(cd "$(dirname "$0")/.." && pwd); EXP=$(cd "$HERE/.." && pwd)
O=$1; NATIVE=${2:-}; NWEIGHTS=${3:-}; NWAV=${4:-}
BID=${BID:-$(date +%Y%m%d-%H%M%S)-rt}
RES=$EXP/apple/results/$BID; U mkdir -p "$RES/power"
U touch "$RES/machine.jsonl" "$RES/run.log" "$RES/stderr.log" "$RES/rt.jsonl" "$RES/power/idle.txt"   # 先由人建好，root 只往里追加，属主不变
BIN=$HERE/target/release/exp010-apple
[ -x "$BIN" ] || { echo "先以普通用户 cargo build --release（不在 root 下编译）"; exit 1; }
BN="$BIN bench bnns $O/gtcrn_unrolled_fp32.mlmodelc $O/gtcrn --model gtcrn --pace-ms 16 --frames 3750"
SETS=(
  "bnns rt（D74 默认，基线）|$BN --qos rt"
  "bnns rt + 自建区间 3ms|$BN --qos rt --interval-wg 3"
  "bnns rt + 自建区间 2ms|$BN --qos rt --interval-wg 2"
  "bnns rt + 自建区间 1ms|$BN --qos rt --interval-wg 1"
  "bnns rt 截止报紧（0.5/1 ms，不加区间）|$BN --qos rt --tc-computation 0.5 --tc-constraint 1"
  "bnns ui 帧间忙等（只排查降频）|$BN --qos ui --spin"
)
if [ -n "$NATIVE" ]; then   # 手写 Rust（rustbe 的二进制）同场：只有它支持的开关
  SETS+=("native rt|$NATIVE gtcrn-bench native $NWEIGHTS $NWAV --pace-ms 16 --frames 3750 --qos rt")
  SETS+=("native ui|$NATIVE gtcrn-bench native $NWEIGHTS $NWAV --pace-ms 16 --frames 3750 --qos ui")
  SETS+=("native-blas rt|$NATIVE gtcrn-bench native-blas $NWEIGHTS $NWAV --pace-ms 16 --frames 3750 --qos rt")
fi
echo "{\"when\":\"$(date -Iseconds)\",\"load\":\"$(sysctl -n vm.loadavg)\",\"power\":\"$(pmset -g batt | head -1)\",\"native_bin_sha256\":\"$( [ -n "$NATIVE" ] && shasum -a 256 "$NATIVE" | cut -c1-64)\"}" >> "$RES/machine.jsonl"
echo "[$(date +%H:%M:%S)] 空闲基线 30 s" | tee -a "$RES/run.log"
powermetrics -i 500 -n 60 --samplers cpu_power,gpu_power,ane_power > "$RES/power/idle.txt" 2>/dev/null
i=0
for c in "${SETS[@]}"; do
  i=$((i+1)); n=${c%%|*}; cmd=${c#*|}
  echo "[$(date +%H:%M:%S)] $i $n" | tee -a "$RES/run.log"
  U touch "$RES/power/$i.txt"
  powermetrics -i 500 --samplers cpu_power,gpu_power,ane_power > "$RES/power/$i.txt" 2>/dev/null &   # 只有这一步是 root
  pm=$!
  l=$(eval "U $cmd" 2>> "$RES/stderr.log"); rc=$?
  kill "$pm"; wait "$pm" 2>/dev/null; pm=""
  if [ $rc = 0 ]; then echo "{\"set\":$i,\"config\":\"$n\",\"result\":$l}" >> "$RES/rt.jsonl"; else echo "{\"set\":$i,\"config\":\"$n\",\"error\":\"见 stderr.log\"}" >> "$RES/rt.jsonl"; fi
  sleep 5   # 两段之间让功率回落
done
echo "{\"when\":\"$(date -Iseconds)\",\"load\":\"$(sysctl -n vm.loadavg)\",\"phase\":\"end\"}" >> "$RES/machine.jsonl"
U touch "$RES/power_summary.json"
U python3 "$HERE/tools/power_parse.py" "$RES" > "$RES/power_summary.json" 2>> "$RES/stderr.log"
chown -R "$SUDO_USER" "$RES"   # 兜底：万一有文件被 root 新建
cat "$RES/power_summary.json"; echo; echo "$RES"
