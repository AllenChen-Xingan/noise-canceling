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
- 基准：r1 有 miri 并行；r2、r3 和别处 16:31–16:32 的编译重叠，都只作参考；**r4 为正式版**（16:32:36 开跑，负载 2.25 → 2.04）。名额已归还
- r4 结论：GTCRN 手写版 p50 0.10 ms，ORT 0.38 ms，手写版约快 3.7 倍，每帧零分配；tract 赢不了 ORT（GTCRN 和 FE-M 都是），每帧分配上千次。正式数字已发给 applebe
- bench 加了 `--pace-ms`（按实时节奏喂帧）；`tools/power_run.sh` 测耗电，要 sudo，待交人跑
- 自审：派了 3 个对抗视角（测量口径与代码 · 手写版能否上线 · 报告与写作），等回报
- 三份自审都回了，都没有 P0。已修：tract FE-M 打开时间混进 ORT 建会话；ORT 环境初始化单列为 `ort_env_ms`；对拍时 NaN 被漏判；新增 `min_frame_snr_db`；计时线程设 QoS USER_INTERACTIVE；新增 p10 / p90；每轮轮换后端顺序；耗电脚本加 trap；新增「ORT 跑改写图」对拍、10 分钟长跑对拍、reset 检查、tract 复用状态的证据；权重加载越界改为返回错误，并加了 MAX_G 检查。新口径已和 applebe 对齐
- 新活（D69 / D70）：手写版目标是每帧平均 ≤ 0.08 ms。先做了逐位不变的优化：卷积改成按区间算，去掉分支。旧版可执行文件留在 `target/release/exp010-bench-v0` 作基线
- 新活（人定）：Rust 对 Python 实测，脚本 `tools/py_ort_bench.py` 和 `tools/run_py_vs_rust.sh`
- 等名额：sample 定位热点 → r5 全套 → Rust 对 Python → 新旧手写版对比
- r5b（正式版）：手写版去掉卷积分支后每帧平均 0.088 ms，逐位不变；Rust 对 Python：Python 慢 5–6%，输出逐位相同
- blas1：Accelerate 版只快 4.7%，而且每帧 malloc 11 次，不采用；全进程 malloc：ORT 每帧 1557 次（FE-M 1023 次），手写版 0 次
- **paced1：按实时节奏跑，手写版慢约 7 倍**（p50 0.61 ms，最大 6.95 ms，6 帧超过 3 ms，占 0.032 个核），applebe 在 BNNS 上看到同一现象。已报大总管和 applebe
- 已写好、待名额（排在 applebe 之后）：`tools/run_rt_probe.sh`，测四种线程设置，并测 native-fast（内联 exp / tanh）。改了代码还没编译：native-fast、--workgroup
- blas1 → malloc1：Accelerate 那 11 次 malloc 出在 `cblas_sgemm → APL_sgemm_QR → posix_memalign`（每调用一次申请一块工作区）；`vDSP_mmul` 同样会分配。不采用
- rt1：实时节奏下，rt 线程策略最好（p50 0.21 ms，0.020 核）；加入工作组反而变差；native-fast 过 BE-PARITY，连续跑快 6.7%。每种设置只跑了一次 60 秒，波动比设置之间的差还大，需要重复测
- codex 换谱系评审 5 轮（`docs/reviews/010_exp010-rust-codex.md`）：P1×7、P2×20 全部属实、全部已改；第 4、5 轮连续两轮无 P0/P1，已达停止线
- ⚠️ 第 1 轮评审之后的 bench 改动（对拍不过时返回非零退出码、新字段、入口校验）只跑过 `cargo check`，没有重编、没有重跑；盘上的二进制是 18:46 编的。新字段要等下次跑基准才会出现在数据里
- **已收尾**。交付：`docs/experiments/exp010-inference-bench/{README.md,results/,bench/,tools/}`、`docs/reviews/010_exp010-rust-codex.md`
