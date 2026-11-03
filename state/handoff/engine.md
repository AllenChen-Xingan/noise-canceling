# 现状页 · 手下 applebe（原名 engine）· 分支 research/019-apple-inference · issue #45 · 实验 exp010

## 状态：已收尾（2026-11-02，待大总管推送 / 开 PR，正文写 Closes #45）

- 交付：`docs/research/019_apple-inference/{decision.md, standard.md, notes/}` · `docs/experiments/exp010-inference-bench/{apple/, apple-rs/}` · 评审记录 `docs/reviews/019_apple-inference-codex.md`
- 评审：同模型 3 视角只读自审 1 轮 + codex 8 轮（第 7、8 轮连续无 P0/P1，满足停止线）；属实 41、驳回 0；剩 P2×2 列进 decision §7
- 测试：`cargo test --release`（单元 4 + 字段交叉校验 2）· `python3 -m unittest tests/test_power_parse.py`（6）· `exp010-apple selftest`
- 结论一句：GTCRN 暂排第一 = 纯手写 Rust（连续 0.088 ms，ORT 0.384 ms 的约 4.4 倍快；整数 0 分配）；苹果侧最快 = BNNS + GRU 展开（0.155 ms，约 2.5 倍）；耗电只有单次、噪声里的数（手写 · rt 81 mW、BNNS · rt 97 mW，未同场 ORT），待 #58 合并与 3 轮重测
- 合并顺序（大总管定）：本 PR 先合；fastpath #58 合并时删 `apple-rs/tests/fields.rs` APPLE_ONLY 里的三条计时窗字段

## 批次 20261102-b8（过程记录）
- 2026-11-02 报到大总管 noise-canceling-e3，计划获同意
- 管的实体：推理后端（苹果）· 模型（降噪）· 评测器（帧耗时 · 逐值一致）
- 机器：Apple M5 · macOS 27.0.1 · 插电
- 起调研 workflow wf_60470c8a-e59（4 调研 + 4 怀疑派），notes 写 `docs/research/019_apple-inference/notes/{coreml,bnns-graph,mlx,realtime-and-bindings}.md`
- 测量程序已写（未跑）：`docs/experiments/exp010-inference-bench/apple/` —— `convert_gtcrn.py` · `convert_fem.py` · `bench_coreml.swift`（GTCRN / FE-M 通用，四种计算单元 + 有状态 + 实时节奏模式）· `power.sh`（要 sudo，人跑）
- 口径已与 rustbe 对齐：转储 `--dump-io`；1 冷启动 + 200 预热 + 6000 计时；对拍前 610 帧；分配字段叫 `process_allocs_per_frame`
- FE-M 本地 PyTorch 权重是半截下载（4.6 MB，应 59.8 MB）；已重下到 `~/vdn-data/models/fastenhancer-dns-v1.0.0/ckpt-full/`，sha256 6f25dd9e…ffe0b 与官方一致，旧文件未动
- 16:2x 大总管通知 rustbe 占名额，applebe 暂停编译 / 推理，已排队
- rustbe 同步：手写 GTCRN 已与 ORT 逐值对拍（37 段，最大差 1.6e-4），正在加 tract

- 调研 workflow 收回（8 代理，4 份 notes 各过怀疑派）。要点：只有 BNNS Graph 可在推理线程每帧直接调；Core ML 把 GRU 翻成逐步循环（约 268 次/帧），神经引擎只算 fp16；MLX 每帧加锁等后台线程
- 待拍板已交大总管登记为 2-8（BNNS / MLState 要 macOS 15，D07 定 14.2+；等实测再问）
- D68 加活：`standard.md` 初稿已交（13 个 verifier，技术线标「AI 定；人可改」）；§8 成熟产品参照子代理在查；§9 Rust↔Python 差待测
- 正式测速从 Rust 调：新 crate `docs/experiments/exp010-inference-bench/apple-rs/`（Core ML · BNNS · 分配计数自检；MLX 走 mlx-c 载 `.mlxfn`，因本机无 Metal 编译器，mlx-c 对着 pip 的 libmlx 编，未写完）
- 一键脚本 `apple-rs/tools/run-all.sh`：建 venv → 转模型 → 用 rustbe 二进制写转储 → 构建自检 → 3 轮交替基准（含 ORT 与手写版同场）→ MLX Python 对照 → MLComputePlan。产物按 build id 写新目录
- D69（人定整机 ≤ 1% 核）已写进 standard.md §6.1：降噪推理分 0.50%，手写 GTCRN 0.63% 也超
- 口径跟 rustbe r5：qos ui、p10/p90、min_frame_snr、NaN 算坏值

- D70（人定：榨干硬件 · 非常低能耗 · 轻量）已写进 standard.md：新 BE-EXHAUST（§7.1 计算单元矩阵）· BE-MEM（常驻内存 ≤ 10 MB）· BE-SIZE ≤ 5 MB · 耗电按每帧 µJ · §6.2 选赢家规则。apple-rs 已加 footprint 字段
- **收尾必做**：decision.md 第一节写风险④——手写方案已知部分 CPU 合计 ≥ 1.95%，离整机上限 2% 只剩 0.05%（大总管要求）

- 17:14–17:40 名额：build 20261102-173000-b2 跑完（`apple/results/20261102-173000-b2/summary.md`）。苹果最快 = BNNS + GRU 展开 0.155 ms（ORT 0.384，手写 0.088），0 分配，134 dB；Core ML 全落 CPU、300 malloc/帧、fp16 不过；MLX Python 5.7 ms；FE-M 转不了（view_as_complex）；修了 coremltools 分组 ConvTranspose 错转
- ⚠️ 实时节奏（16 ms）10 分钟：BNNS p50 0.82 ms、max 11.8 ms、4.8% 核 → 连续跑低估约 5 倍；已写 `apple-rs/tools/rt-matrix.sh`（QoS / rt 策略 / 工作组 / 忙等对照），等名额（47 方 18:00–18:30 之后）
- DFN3 `_ll` 下界 6.1% 核 → 不做流式重导出，盲听用离线（大总管同意）；MLX 走 Rust 那格写「不测」+原因
- `decision.md` 草稿已写，第 4 节等实时策略结果

- D71（共用频谱前端）写进 decision §5；D73（最低 14.4）、D74（tc 默认开、工作组默认不加）、D75（挡线改成「对性能的影响」+ 耗电主排序，CPU% 降参考）写进 standard / decision
- 新加：ECAPA 进 BE-EXHAUST（`apple-rs/tools/ecapa.sh`，ONNX→onnx2torch→Core ML）· BE-IMPACT（`loadgen` + `tools/impact.sh`）· 自建工作组区间 `--interval-wg`（`tools/rt-matrix.sh`，要人 sudo，命令见 `tools/HUMAN-RUN-power.md`）

- 18:12 名额：ECAPA 转不出（onnx2torch 补了两处后，coremltools 不支持 `prod`）；BE-IMPACT 两次结果**作废**
- ⚠️ 事故（applebe 的错）：`impact.sh` 旧版 `eval "$CMD" &` 只杀子 shell，留下 21 个孤儿降噪进程（约半核），已修（exec + kill/wait 核实 + trap）。教训：后台起进程一律 `exec`，结束必须 `kill -0` 核实；大总管请人清理，applebe 未动这些进程
- 18:2x 人跑 powermetrics 推迟到清理之后

## 下一步
1. 名额（18:37 起约 22 分钟）：cargo build → ecapa.sh → impact.sh（bnns rt · native rt）→ 叫大总管转人跑 sudo rt-matrix
2. §9 Rust↔Python 差表（rustbe 有 ORT 5% 数；MLX 只有 Python）
3. 补 decision.md 第 4 节 → 对抗自审一轮 → PR（Closes #45）

## 转交
- （已报大总管）D69 整机 ≤ 0.01 核：认人判断回路 ≈ 1.2% 核，单项超总线，需降约 8 倍到 0.15%（不归 019）
- （已报大总管）FE-M 在 ORT 上 ≈ 3.7% 核，要快 7 倍以上才进线，D33 备选地位待重估
- （已报大总管）STFT / 重采样 / 采集输出 / 修音的 CPU 都没量过

## 批次 20261031-b6（旧）
- workflow wf_502b2e03-c65 中断，只写出 `notes/our-models.md`
