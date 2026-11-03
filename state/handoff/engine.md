# 现状页 · 手下 applebe（原名 engine）· 分支 research/019-apple-inference · issue #45 · 实验 exp010

## 批次 20261102-b8（当前）
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

## 下一步
1. 等「名额给你」→ `apple-rs/tools/run-all.sh`；修编译错（第一次编 apple-rs）
2. MLX 的 Rust 路（mlx-c）+ §9 Rust↔Python 差
3. 写 `decision.md`，对抗自审一轮

## 转交
- （已报大总管）D69 整机 ≤ 0.01 核：认人判断回路 ≈ 1.2% 核，单项超总线，需降约 8 倍到 0.15%（不归 019）
- （已报大总管）FE-M 在 ORT 上 ≈ 3.7% 核，要快 7 倍以上才进线，D33 备选地位待重估
- （已报大总管）STFT / 重采样 / 采集输出 / 修音的 CPU 都没量过

## 批次 20261031-b6（旧）
- workflow wf_502b2e03-c65 中断，只写出 `notes/our-models.md`
