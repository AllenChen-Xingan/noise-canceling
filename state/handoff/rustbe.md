# 现状页 · 手下 rustbe（批次 20261102-b8 · 分支 exp/010-rust-backend · issue #51）

管的实体：推理后端（纯 Rust）· 模型（降噪）· 评测器（帧耗时 · 逐值一致）

- 2026-11-02 报到；大总管给名额（构建 + gtcrn-parity）
- 手写 GTCRN 能编译，对拍通过：37 段对 ORT，最差 SNR 119.2 dB，最大差 1.6e-4（`results/parity_native_vs_ort.jsonl`）；名额已归还
- 选 tract 0.23.9 做 Rust 推理库：它直接吃 ONNX，GRU / ConvTranspose / LayerNorm 都支持，它的每晚基准里就有 GTCRN 流式模型（子代理调研，读源码 + 官方 CI，笔记在 scratchpad，待并进 README）。candle-onnx 缺 GRU，排除
- bench 加了 tract 后端（`src/tract_be.rs`）、FE-M（`fem-bench` / `fem-parity`），以及给 applebe 用的 `--dump-io`；已和 applebe 对齐口径（它全进程分配数叫 `process_allocs_per_frame`）
- 坑：tract 复用 SimpleState 时第 1 帧起就算错（SNR 19 dB），每帧新开状态则对（130 dB），代价 p50 +0.02 ms。原因未查清
- 坑：tract 不支持 FE-M 图里的 SplitToSequence；`tools/fem_strip_seq.py` 把 8 处等价换成 Gather，对 ORT 原图 SNR 110.8 dB。改写后的图放主目录 `_tmp/20261102-exp010/`
- 建了 `_tmp/exp010-venv`（只装 onnx、numpy）
- 等待：rs-shm 那边在跑 cargo miri，占一个核，已报大总管；等它结束再跑正式基准（`tools/run_bench.sh <build-id>`）
