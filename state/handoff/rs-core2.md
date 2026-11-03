# 现状页 · 手下 rs-core2（批次 20261102-b8-rust）

> 管的实体：`vdn-types` · `vdn-dsp` · `vdn-engine` · `vdn-models`
> 分支 `feat/rust-skeleton-core`，worktree `noise_canceling-rs-core`
> 交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-core2.md`

## 手上
- **已收尾**（2026-11-02），等大总管验收

## 做到哪（交代第 1–7 条全部完成）
1. **后处理级串进 Pipeline**（处理图第一步）：`Pipeline<M, S: StageChain = ()>`，模型之后、升采样之前按顺序跑；静态分发、不分配。有延迟的级构造时报 `StageLatency`。最简单的级 `testing::Gain`（固定增益）。签名见 `state/handoff/rs-core2-pipeline-signature.md`，大总管认可，已发 rs-entry
2. **送原声改用 48 kHz 原声**（审查偏离第 6 条）：`Engine::process_status` 只回报 `Status`；Pipeline 把每 hop 的状态随样本进输出队列，出队时用 48 kHz 原声切换（接缝衰减切过去、5 ms 线性淡化切回）
3. **延迟断言以 Pipeline 为正本**：`src/models/tests/latency.rs` 不抄数字，全从 `Pipeline::latency_samples()` / `latency_ledger()` 取，平台部分去掉预算里的重采样项（避免记两次）
4. **FTZ 守卫**（审查偏离第 5 条）：`vdn_dsp::FtzGuard`（`new` 开、`off` 关，出作用域恢复）。Pipeline 用它包降采样、后处理级、升采样、出队混合；Engine 用 `off` 包推理调用，调用方线程自己开着 FTZ 推理也不跟着开（R9）
5. **回归**：见下「测试与对拍」
6. 签名第一时间发出（编译通过即发）
7. 三视角自审完成，问题已修

## 测试与对拍（实测，本 worktree）
- `vdn denoise`（GTCRN，合成 4 s 16 kHz WAV `_tmp/20261102-b8-rust/core/synth16k.wav`）改动前后 **4 个场景字节一致**：默认 · `--no-safety` · 故障注入（`--inject 20:nan,40:error,60:explode,80:stall=50 --deadline-ms 20`）· `--no-fast-open`。前后文件在 `_tmp/20261102-b8-rust/core2/`
- `VDN_REQUIRE_MODELS=1 cargo test -p vdn-types -p vdn-dsp -p vdn-engine -p vdn-models`：115 条通过、0 失败（GTCRN 与 FE-M 真模型冒烟都跑了）
- clippy（四 crate `--all-targets -D warnings`）无警告；下游 `vdn-eval`、`vdn-speaker-lab`、`vdn-cli` clippy 也无警告
- `cargo run -p xtask -- tidy`、`check-deps` 通过；`cargo fmt --check` 通过
- **变异检查**（临时改坏代码、确认测试会红、再核对还原）：让 Pipeline 忽略 Status → 故障注入测试红；关后处理级检查 → 后处理级坏值测试红；关切回拖尾标记 → 故障注入测试红

### 新测试（`src/engine/tests/stages.rs`，14 条）
- 增益级按顺序作用、延迟不变；空串与 `new` 逐样本相同；有延迟的级被拒；账本列出各级且合计 = `latency_samples()`
- 后处理级拿到的 16 kHz 原声与声明延迟对齐；`FrameCtx.now` 由 Pipeline 填、调用方上下文传到
- **故障注入**（NaN · 后端报错 · 幅度异常 · 超时，三种构造 / 块长）：送原声区间内逐样本等于延迟 L 的 48 kHz 原声（误差 < 1e-4），15 kHz 分量能量比误差 < 0.1%（16 kHz 原声里没有它）；全程峰值 ≤ 原声 × 1.1；切换处相邻差 ≤ 原声 × 1.1（无爆音）
- 后处理级出 NaN / 1e3：整 hop 送原声、报状态、级复位；输入带 NaN / Inf：输出不锁死成 NaN
- FTZ：推理时关、后处理级里开；调用方开着 FTZ 时推理仍关着、返回后调用方状态恢复；带级带故障不分配
- vdn-dsp：守卫内非正规数冲成 0、守卫外不冲、嵌套与反向守卫恢复正确（本机 arm64 实测）

### 延迟（实测，测试打印，两模型相同）
| 情形 | Pipeline 部分（代码声明） | 平台部分（规划预算） | 合计 |
|---|---|---|---|
| IO 块 96 帧，`with_block` | 32.00 ms | 7.67 ms | **39.67 ms**（≤ 45） |
| IO 块 256 帧，`new`（009 原目标） | 33.98 ms | 11.00 ms | **44.98 ms**（只记录） |
- 算不进的：麦克风与输出设备的硬件 / 驱动延迟、蓝牙、系统混音器（P3 实测）；跨进程环与 HAL 余量仍是预算
- ⚠️ 规约 §3.7.6 表里「009 目标 47 ms，超 2 ms」已过时：当时重采样按预算 4 ms、攒 hop 按 16 ms 记；按代码是 2 ms 和 15.98 ms，合计 44.98 ms。表格不归我改，请大总管更新

## 改了哪些文件
- `src/types/src/stage.rs`（加 `StageChain`、`stages!` 宏、2 条单测）· `src/types/src/lib.rs`（导出 `StageChain`）
- `src/dsp/src/ftz.rs`（新）· `src/dsp/src/lib.rs`（`forbid` → `deny(unsafe_code)`，导出 `FtzGuard`）· `src/dsp/Cargo.toml`（`unsafe = true`）
- `src/engine/src/engine.rs`（`run` / `process_status` / `aligned_dry` / `config`；推理外包 `FtzGuard::off`；接缝非有限保护）
- `src/engine/src/pipeline.rs`（后处理级 · 48 kHz 送原声 · FTZ · 输入非有限按 0 · 新错误变体与两个计数）
- `src/engine/src/lib.rs`（导出 `StageChain` `Status` `stages!`；`testing::Gain`）
- `src/engine/tests/stages.rs`（新）· `src/models/tests/latency.rs`（重写）
- `state/handoff/rs-core2.md`、`state/handoff/rs-core2-pipeline-signature.md`

## 新增的第二处表示及其校验（唯一真相源规矩）
| 第二处 | 正本 | 机器校验 |
|---|---|---|
| `latency_ledger()` 里的延迟项 | `Pipeline::latency_samples()` | 单测：账合计 = `latency_samples()`（engine 与 models 各一条） |
| latency.rs 用的假模型 hop / 延迟 | `ModelKind::declared()` | 直接取值；`smoke.rs` 打开真模型核对声明 |
| 平台部分去掉重采样项（按名字前缀 `resample`） | `planned_platform()` | 单测：Pipeline 账 + 平台部分里重采样项恰好 1 个 |
| Pipeline 的后处理级检查（非有限 / `max_abs`） | Engine `classify` 的同一规则 | 只读同一个 `SafetyConfig::max_abs`；**规则写了两处、无比对**（建议以后抽成一个函数，待后续批次） |
- 删掉了原 latency.rs 里钉死的数字（131/3、47 等），不再有抄来的延迟数
- **无校验的第二处**：规约 §3.7.6 的延迟表（见上，已过时）

## 已知缺口（本批不做，记下）
- 衰减下限与快开慢关仍在 Engine 里混 **16 kHz 原声**：正常降噪时原声成分只到 8 kHz；而送原声时是全频带——一次超时故障约 16 ms 里高频忽然出现又消失（未量化）。见「待人拍板」
- x86_64 的 FTZ 分支本机编译不了（只装了 aarch64 目标），只做了人工核对
- Engine 自己的 DSP（延迟线、包络）和推理在同一函数里，未包 FTZ（不违反 R9；长时间静音包络是否进非正规数未量化）
- `last_status()` 是输入侧状态，比耳朵听到的早约一个 hop；一次调用跑多 hop 只留最后一个（已写进文档；要计数看 stats）
- `release` 下才跑的测试仍没跑（要名额，留给大总管的闸门）

## 外部依赖
- 没有引入新的外部依赖

## 实体行（给大总管写进 `.42cog/cog.md`）
| 实体 | 是什么 | 谁产生 | 谁是谁的输入 |
|---|---|---|---|
| `StageChain`（在 `vdn-types`） | 一串按固定顺序静态调用的后处理级，`stages![a, b]` | rs-core2 | 被 `Pipeline` 持有；元素是 `Stage`（门控、衰减下限、修音链节点） |
| `FtzGuard`（在 `vdn-dsp`） | 作用域内开 / 关「非正规数冲 0」，出作用域恢复 | rs-core2 | 被 `Pipeline`（包 DSP 段）和 `Engine`（推理时关）使用 |

## 已定（大总管 2026-11-02）
- dsp 进 unsafe 白名单，原因 FTZ（arm64 读写 FPCR 只能用内联汇编）。lib.rs 改 `#![deny(unsafe_code)]`，只在 ftz 模块放开；非 aarch64 / x86_64 与 `cfg(miri)` 下守卫是空操作。规约 R10 白名单由大总管收尾时更新

## 待人拍板（建议，待你定）
1. **正常降噪时，混回去的那点原声要不要带 8 kHz 以上？** 现在「背景最多降 12 dB」这条混回的是 16 kHz 原声，只到 8 kHz；出故障送原声时是全频带。
   - 选 A（现状）：混回只到 8 kHz。后果：正常时声音略闷；故障那十几毫秒高频忽然出现
   - 选 B：混回用 48 kHz 原声。后果：声音更亮，但 8 kHz 以上没降过的噪声（如嘶声）也按 -12 dB 放回来
   - 建议：先保持 A，等「衰减下限」移成后处理级时做 AB 听测再定。如果你不回，默认 A，后果是正常降噪时略闷

## 流水
- 报到；读材料
- 请示 vdn-dsp unsafe；大总管同意自写（另要求 Miri 下空操作、测试证明推理在守卫外）
- 出改动前基准：`vdn denoise` 4 份
- vdn-types 加 `StageChain` + `stages!`（只加不改）；单测 2 条过
- Engine 拆 `run`，加 `process_status`；老 `process` 输出不变
- 写 `vdn-dsp/src/ftz.rs`，dsp 改 unsafe = true
- 重写 Pipeline；原有 24 条测试过；发签名给大总管与 rs-entry；按大总管要求补发行为变化原因给 rs-entry
- 新测试 11 条过；变异检查有效
- 改 latency.rs：按大总管新规矩（唯一真相源），不抄数字
- 回归对拍 4 份字节一致；fmt、测试、clippy、tidy、check-deps 全过
- 三个子代理自审（实时安全 · 48 kHz 送原声 · FTZ）：无 P0 / P1。修了：调用方开着 FTZ 时推理跟着开（加反向守卫）；后处理级坏值不判故障；输入 NaN 让接缝锁死；淡化时长无上限；溢出兜底读指针不前移；切回时升采样拖尾未标送原声；reset 不再把流时间归零；补测试 3 条并加强 2 条。衰减下限用 16 kHz 原声记为待人拍板
- 修后复跑：对拍 4 份仍字节一致；115 条测试过；clippy、tidy、check-deps 过；下游 eval / speaker-lab / cli clippy 过
