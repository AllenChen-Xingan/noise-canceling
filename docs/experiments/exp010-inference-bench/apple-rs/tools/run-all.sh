#!/bin/bash
# exp010 · applebe 一次跑完（要名额独占）。每次运行一个 build id，产物写新目录，不覆盖旧的（CLAUDE.md 第五节第 1 条）。
#   大文件（模型、转储）→ 主目录 _tmp/exp010-apple/<build-id>/（git 忽略）
#   小文件（结果 jsonl、机器状态、转换记录、哈希）→ apple/results/<build-id>/（进库）
# 单条失败只记一行 error，接着跑下一条（不停摆）。
# 用法：tools/run-all.sh [阶段...]      阶段：setup convert dump build bench mlx plan（默认全部）
set -u
HERE=$(cd "$(dirname "$0")/.." && pwd)                       # apple-rs
EXP=$(cd "$HERE/.." && pwd)                                  # exp010-inference-bench
MAIN=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
RUSTBE=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling-rustbe/docs/experiments/exp010-inference-bench/bench
VENV=$MAIN/_tmp/exp010-venv
PY=$VENV/bin/python
BID=${BID:-$(date +%Y%m%d-%H%M%S)-$(git -C "$HERE" rev-parse --short HEAD)}
OUT=$MAIN/_tmp/exp010-apple/$BID
RES=$EXP/apple/results/$BID
mkdir -p "$OUT" "$RES"
STAGES=${*:-setup convert dump build bench mlx plan}
log() { echo "[$(date +%H:%M:%S)] $*" | tee -a "$RES/run.log"; }
has() { case " $STAGES " in *" $1 "*) return 0;; esac; return 1; }

machine() {
  { echo "{\"when\":\"$(date -Iseconds)\",\"model\":\"$(sysctl -n hw.model)\",\"chip\":\"$(sysctl -n machdep.cpu.brand_string)\","
    echo "\"os\":\"$(sw_vers -productVersion) ($(sw_vers -buildVersion))\",\"power\":\"$(pmset -g batt | head -1 | sed "s/.*'\(.*\)'.*/\1/")\","
    echo "\"load\":\"$(sysctl -n vm.loadavg)\",\"phase\":\"$1\"}"; } | tr -d '\n' >> "$RES/machine.jsonl"
  echo >> "$RES/machine.jsonl"
}

log "build $BID → $OUT · 结果 $RES"
machine start

if has setup; then
  if [ ! -x "$PY" ]; then
    log "建 venv（torch 与 exp008 同版本，走 uv 缓存）"
    uv venv --python 3.12 "$VENV" >> "$RES/run.log" 2>&1
    uv pip install --python "$PY" "torch==2.15.0" coremltools mlx numpy pyyaml >> "$RES/run.log" 2>&1 || log "ERROR pip"
  fi
  uv pip list --python "$PY" 2>/dev/null | grep -Ei "^(torch|coremltools|mlx|numpy) " > "$RES/python-packages.txt"
fi

if has convert; then
  log "转 GTCRN"
  "$PY" "$EXP/apple/convert_gtcrn.py" "$MAIN/resources/gtcrn/stream" "$OUT" > "$RES/convert_gtcrn.stdout" 2> "$RES/convert_gtcrn.stderr" || log "ERROR convert_gtcrn"
  cp "$OUT/convert.json" "$RES/" 2>/dev/null
  log "转 FE-M"
  "$PY" "$EXP/apple/convert_fem.py" "$MAIN/resources/fastenhancer" "$HOME/vdn-data/models/fastenhancer-dns-v1.0.0/ckpt-full/fastenhancer_m" "$OUT" \
    > "$RES/convert_fem.stdout" 2> "$RES/convert_fem.stderr" || log "ERROR convert_fem"
  cp "$OUT/convert_fem.json" "$RES/" 2>/dev/null
fi

ORTBIN=$RUSTBE/target/release/exp010-bench
GT_ONNX=$MAIN/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx
FEM_ONNX=$HOME/vdn-data/models/fastenhancer-dns-v1.0.0/fastenhancer_m.onnx
WAV=$MAIN/resources/gtcrn/test_wavs/mix.wav
if has dump; then
  log "用 rustbe 的程序写转储（不重编它，只用现成二进制）"
  if [ -x "$ORTBIN" ]; then
    "$ORTBIN" gtcrn-bench ort "$GT_ONNX" "$WAV" --dump-io "$OUT/gtcrn" > "$RES/dump_gtcrn.json" 2>> "$RES/run.log" || log "ERROR dump gtcrn"
    "$ORTBIN" fem-bench ort "$FEM_ONNX" "$WAV" --dump-io "$OUT/fem" > "$RES/dump_fem.json" 2>> "$RES/run.log" || log "ERROR dump fem"
    cp "$OUT"/gtcrn.json "$RES/dump_gtcrn.meta.json" 2>/dev/null; cp "$OUT"/fem.json "$RES/dump_fem.meta.json" 2>/dev/null
  else
    log "ERROR 找不到 $ORTBIN（请 rustbe 先 cargo build --release）"
  fi
fi

BIN=$HERE/target/release/exp010-apple
if has build; then
  log "cargo build apple-rs"
  (cd "$HERE" && cargo build --release >> "$RES/run.log" 2>&1) || log "ERROR cargo build"
  "$BIN" selftest > "$RES/selftest.json" 2>> "$RES/run.log" || log "ERROR selftest（分配计数器不可信）"
  (cd "$EXP/apple" && ./build.sh >> "$RES/run.log" 2>&1) || log "ERROR swift/c build（对照程序）"
  for m in gtcrn_fp32 gtcrn_unrolled_fp32 fem_fp32; do
    [ -d "$OUT/$m.mlpackage" ] && "$BIN" compile "$OUT/$m.mlpackage" "$OUT/$m.mlmodelc" >> "$RES/compile.jsonl" 2>> "$RES/run.log"
  done
fi

# 被测组合：名字|命令。ORT 与手写 Rust（rustbe 二进制）同场轮流跑，供 BE-FASTER 同场比较。
GT="--model gtcrn"; FE="--model fem --fem-json $OUT/convert_fem.json"
CONFIGS=(
  "gtcrn ort|$ORTBIN gtcrn-bench ort $GT_ONNX $WAV"
  "gtcrn native|$ORTBIN gtcrn-bench native $MAIN/_tmp/20261031-exp010/gtcrn_dns3 $WAV"
  "gtcrn coreml-cpu fp32|$BIN bench coreml $OUT/gtcrn_fp32.mlpackage $OUT/gtcrn $GT --units cpu"
  "gtcrn coreml-gpu fp32|$BIN bench coreml $OUT/gtcrn_fp32.mlpackage $OUT/gtcrn $GT --units gpu"
  "gtcrn coreml-ane fp16|$BIN bench coreml $OUT/gtcrn_fp16.mlpackage $OUT/gtcrn $GT --units ane"
  "gtcrn coreml-all fp16|$BIN bench coreml $OUT/gtcrn_fp16.mlpackage $OUT/gtcrn $GT --units all"
  "gtcrn coreml-cpu fp16|$BIN bench coreml $OUT/gtcrn_fp16.mlpackage $OUT/gtcrn $GT --units cpu"
  "gtcrn coreml-cpu state-fp16|$BIN bench coreml $OUT/gtcrn_state_fp16.mlpackage $OUT/gtcrn $GT --units cpu --state"
  "gtcrn coreml-cpu unrolled-fp32|$BIN bench coreml $OUT/gtcrn_unrolled_fp32.mlpackage $OUT/gtcrn $GT --units cpu"
  "gtcrn bnns fp32|$BIN bench bnns $OUT/gtcrn_fp32.mlmodelc $OUT/gtcrn $GT"
  "gtcrn bnns unrolled-fp32|$BIN bench bnns $OUT/gtcrn_unrolled_fp32.mlmodelc $OUT/gtcrn $GT"
  "gtcrn bnns-multi fp32|$BIN bench bnns $OUT/gtcrn_fp32.mlmodelc $OUT/gtcrn $GT --multi"
  "fem ort|$ORTBIN fem-bench ort $FEM_ONNX $WAV"
  "fem coreml-cpu fp32|$BIN bench coreml $OUT/fem_fp32.mlpackage $OUT/fem $FE --units cpu"
  "fem coreml-gpu fp32|$BIN bench coreml $OUT/fem_fp32.mlpackage $OUT/fem $FE --units gpu"
  "fem coreml-ane fp16|$BIN bench coreml $OUT/fem_fp16.mlpackage $OUT/fem $FE --units ane"
  "fem coreml-all fp16|$BIN bench coreml $OUT/fem_fp16.mlpackage $OUT/fem $FE --units all"
  "fem bnns fp32|$BIN bench bnns $OUT/fem_fp32.mlmodelc $OUT/fem $FE"
)

if has bench; then
  machine bench-start
  for round in 1 2 3; do
    for c in "${CONFIGS[@]}"; do
      name=${c%%|*}; cmd=${c#*|}
      log "round $round · $name"
      if line=$(eval "$cmd" 2> "$RES/last.stderr"); then
        echo "{\"round\":$round,\"config\":\"$name\",\"result\":$line}" >> "$RES/bench.jsonl"
      else
        err=$(tail -c 300 "$RES/last.stderr" | tr '\n"' ' \047')
        echo "{\"round\":$round,\"config\":\"$name\",\"error\":\"$err\"}" >> "$RES/bench.jsonl"
      fi
      cat "$RES/last.stderr" >> "$RES/stderr.log"
    done
  done
  machine bench-end
fi

if has mlx; then
  log "MLX：导权重 · 对 torch 自检 · 导 .mlxfn · Python 计时（对照，D68）"
  "$PY" "$EXP/apple/mlx_gtcrn.py" export "$MAIN/resources/gtcrn/stream" "$OUT/gtcrn_mlx.npz" > "$RES/mlx_export.json" 2>> "$RES/run.log" || log "ERROR mlx export"
  MLX_ENABLE_TF32=0 "$PY" "$EXP/apple/mlx_gtcrn.py" check "$MAIN/resources/gtcrn/stream" "$OUT/gtcrn_mlx.npz" > "$RES/mlx_check.json" 2>> "$RES/run.log" || log "ERROR mlx check"
  MLX_ENABLE_TF32=0 "$PY" "$EXP/apple/mlx_gtcrn.py" export-mlxfn "$OUT/gtcrn_mlx.npz" "$OUT/gtcrn.mlxfn" >> "$RES/mlx_export.json" 2>> "$RES/run.log" || log "ERROR mlxfn"
  for round in 1 2 3; do
    for dev in cpu gpu; do
      log "round $round · mlx-python $dev"
      line=$(MLX_ENABLE_TF32=0 "$PY" "$EXP/apple/mlx_gtcrn.py" bench "$OUT/gtcrn_mlx.npz" "$OUT/gtcrn.in.f32" "$OUT/gtcrn.ort.f32" --device $dev 2>> "$RES/stderr.log") \
        && echo "{\"round\":$round,\"config\":\"gtcrn mlx-python-$dev\",\"result\":$line}" >> "$RES/bench.jsonl" \
        || echo "{\"round\":$round,\"config\":\"gtcrn mlx-python-$dev\",\"error\":\"见 stderr.log\"}" >> "$RES/bench.jsonl"
    done
  done
fi

if has plan; then
  log "MLComputePlan：每个算子落在哪个计算单元"
  for m in gtcrn_fp32 gtcrn_fp16 gtcrn_unrolled_fp32 fem_fp32 fem_fp16; do
    for u in cpu ane all; do
      "$EXP/apple/build/bench_coreml" plan "$OUT/$m.mlpackage" $u >> "$RES/plan.jsonl" 2> "$RES/plan_${m}_$u.tsv" || log "ERROR plan $m $u"
    done
  done
fi

# 哈希：大文件不进库，只留路径 + sha256（铁律 8）
(cd "$OUT" && find . -maxdepth 1 -type f \( -name '*.f32' -o -name '*.npz' -o -name '*.json' \) -exec shasum -a 256 {} \; ) > "$RES/artifacts.sha256"
echo "$OUT" > "$RES/artifacts.path"
machine end
log "完成 $BID"
