#!/bin/bash
# exp010 · 认人声纹 ECAPA-TDNN512 进 BE-EXHAUST（要名额，约 4 分钟）
# 转换（ONNX → torch → Core ML）· 编 mlmodelc · Core ML CPU 与 BNNS 各 3 轮（每轮 200 个窗）· BNNS 按真实节奏（每 0.5 s 一个窗）跑 60 s
set -u
HERE=$(cd "$(dirname "$0")/.." && pwd); EXP=$(cd "$HERE/.." && pwd)
MAIN=/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling
PYCT=$MAIN/_tmp/exp010-venv-ct/bin/python
BID=${BID:-$(date +%Y%m%d-%H%M%S)-ecapa}
OUT=$MAIN/_tmp/exp010-apple/$BID; RES=$EXP/apple/results/$BID; mkdir -p "$OUT" "$RES"
BIN=$HERE/target/release/exp010-apple
uv pip install --python "$PYCT" onnx2torch onnx >> "$RES/run.log" 2>&1
(cd "$HERE" && cargo build --release 2>&1 | tail -1) >> "$RES/run.log"
"$PYCT" -X faulthandler "$EXP/apple/ecapa_prep.py" "$HOME/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx" "$OUT" > "$RES/ecapa_prep.json" 2> "$RES/ecapa_prep.stderr" || echo "ERROR prep" | tee -a "$RES/run.log"
"$BIN" compile "$OUT/ecapa_fp32.mlpackage" "$OUT/ecapa_fp32.mlmodelc" >> "$RES/compile.jsonl" 2>> "$RES/run.log"
A="--model ecapa --frames 200 --warmup 20 --parity-frames 200"
for round in 1 2 3; do
  for c in "coreml-cpu|$BIN bench coreml $OUT/ecapa_fp32.mlmodelc $OUT/ecapa $A --units cpu" "bnns|$BIN bench bnns $OUT/ecapa_fp32.mlmodelc $OUT/ecapa $A"; do
    n=${c%%|*}; cmd=${c#*|}
    l=$(eval "$cmd" 2>> "$RES/stderr.log") && echo "{\"round\":$round,\"config\":\"ecapa $n\",\"result\":$l}" >> "$RES/bench.jsonl" || echo "{\"round\":$round,\"config\":\"ecapa $n\",\"error\":\"见 stderr.log\"}" >> "$RES/bench.jsonl"
  done
done
l=$("$BIN" bench bnns "$OUT/ecapa_fp32.mlmodelc" "$OUT/ecapa" --model ecapa --frames 120 --warmup 5 --parity-frames 200 --pace-ms 500 2>> "$RES/stderr.log") && echo "{\"config\":\"ecapa bnns paced-500ms\",\"result\":$l}" >> "$RES/bench.jsonl"
(cd "$OUT" && shasum -a 256 ecapa.in.f32 ecapa.ort.f32) > "$RES/artifacts.sha256"; echo "$OUT" > "$RES/artifacts.path"
echo "$RES"
