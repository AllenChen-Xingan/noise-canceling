# ADR 005 · 送原声两层分工、panic 策略、延迟记账

- 状态：提议（2026-10-30，rustarch）· 涉及 ⟨待拍板 3、6⟩
- 规约落点：`specs/rust/architecture.md` §3.5、§5 R13、§5.3

## 背景
D03：推理进程崩了，送音进程照样送原声。p1 的 `Engine` 已实现模型级送原声（报错、NaN、幅度异常、超时）。003 已定「原声旁路由输出回调做」。D32：端到端附加延迟 ≤ 45 ms。

## 决定
1. **两层送原声，分工写死**：
   - 模型级由 `vdn-engine` 的 `Pipeline` 做（用 **48 kHz 原声**，否则原声缺 8 kHz 以上；P1 离线期间仍由 `Engine` 在 16 kHz 做）：单帧故障、状态坏、换模型
   - 进程级由送音进程输出回调做：推理进程卡死、崩溃、心跳停
   - 两层用同一个 `L_total`（启动时按所有启用模型的最大延迟定死，经 `vdn-shm` 头传给送音进程），不各自对齐，换模型也不改
2. **panic**：实时代码按 R4 写成不 panic；release `panic = "abort"`；推理进程 abort 由进程级送原声兜底；no_panic 证明构建另开 unwind profile
3. **延迟记账**（本 ADR 是延迟账的正本）：`Pipeline::latency_samples()` = 重分块（最多一个 IO hop）+ 两次重采样 + 模型（Stage 在 P2 前必须为 0 延迟），统一按 IO 采样率样本计，单测断言不超预算；固定比例重采样（48↔16 kHz）放 `vdn-engine::Pipeline`，eval 读 48 kHz WAV 也走同一条路；`Slip` 漂移补偿只在 `vdn-io-macos`（eval 不需要）
4. P3 第一件事是脉冲法实测端到端延迟

## 理由
- 不写死分工，两层都做延迟对齐，就会对齐两次，切换时跳字或重字（003 decision「拿三把尺子量一遍」清楚行）【推】`notes/r8-critic.md` C9
- 进程隔离是唯一能挡住「我们自己的代码崩溃」的兜底；有了它，release 用 abort 比 unwind 更简单可预测【推】`notes/r4-realtime.md` §2.5
- 延迟按默认参数估约 56 ms，**超 45 ms**：STFT 32 + rubato `Fft` 默认两跳约 10.7 + IO 缓冲 512 帧 10.7 + HAL 余量 2.7 + 环余量【推】rusteco 009 `decision.md` §一.3（r8 按管线 6–10 ms、重采样 2–6 ms 估的 41–49 ms 偏乐观，`notes/r8-critic.md` §3.8）；所以重采样和 IO 缓冲必须调参；重采样器放 engine 是延迟与「同一份代码」两条硬约束的交点
- rubato `Fft` 同步重采样延迟 `fft_size_out / 2`，`Slip` 延迟 0【源】`resources/rubato/src/synchro.rs:692-694`、`slip.rs:481-483`

## 代价
- `vdn-shm` 头多一个字段；`L_total` 取最大延迟，延迟较小的模型也多付一点
- abort 意味着推理进程里没有「接住 panic 再继续」的机会，每次 panic 都是一次进程重启 + 一段原声
- 延迟真超了 45 ms，要人定砍哪一级（⟨待拍板 6⟩）
