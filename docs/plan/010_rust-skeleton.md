# 010 · Rust 工程骨架：规格怎么定（AI 方案，待人过目）

> 大事：**把 Rust 工程从底层到命令行整个搭出来，连同检查它的 harness（闸门）**。
> 第二大总管，2026-11-02；分支 `feat/rust-skeleton`，worktree `../noise_canceling-skeleton`。
> 证据：【测】本机实测 ·【读】读仓内文件 ·【推】推断。

## 一、现状（不是从零开始）

- 已有 11 个 crate、约 1.75 万行 Rust；`cargo check --workspace --all-targets` 3.9 s 通过【测】
- 目标形态已写在 `specs/rust/architecture.md` §2.1（约 18 个 crate、3 个可执行文件）【读】
- 已有闸门：fmt · clippy · test · `xtask tidy` · `xtask check-deps` · `xtask eval ci`；CI 只有一个 macOS job【读】
- 缺口见第三节。缺的主要是**实时与平台那一半**：重分块与重采样层、推理后端接口、共享内存、实时线程、Mac 音频 IO、虚拟麦克风、控制、两个常驻进程

## 二、规格分三层，每层落在会红的东西上

规格不写成长文。能用规则判的，写成代码和测试（CLAUDE.md 第三节）。

| 层 | 管什么 | 落在哪 | 怎么判 |
|---|---|---|---|
| 1 总图 | 有哪些 crate、谁能依赖谁、哪档实时 | `specs/rust/architecture.md` §2（已有，只补 D53 / D55 带来的改动） | `xtask check-deps` · `cargo deny` |
| 2 接口卡 | 每个 crate 对外的类型和函数 | 各 crate 的 `Cargo.toml` `[package.metadata.vdn]` + `lib.rs` 公开签名 + 文档注释 | 能编译 · 验收测试 |
| 3 闸门 | 整体是否守规矩、是否能跑 | `xtask` 子命令 + CI job（规约 §7 的 G1–G14） | 绿 / 红 |

做法：**先写接口卡和验收测试（会红），再填实现（变绿）**。还没实现的功能返回明确的「未实现」错误，不 `panic`，测试标 `#[ignore = "待实现：<哪件>"]`，用 `xtask` 统计剩多少。

## 三、目标 crate 与缺口

清单用途：按实体（crate）列出每个要建或要改的东西。

| 层 | crate | 档 | 现状 | 要做 |
|---|---|---|---|---|
| 词汇 | `vdn-types` | A/B | 有 | 补推理后端接口（D53）、跨进程头里的计数器类型 |
| 积木 | `vdn-dsp` | B | 有 STFT · 延迟线 · 包络 · 窗 | 补重分块器 · 固定比例重采样 · 淡化 · 限幅 · FTZ 守卫 |
| 推理 | `vdn-backend-ort` | B | 无（ort 直接写在 models 里） | 拆出；models 只认后端接口。Core ML / MLX / 手写 Rust 留槽位，等 019 结论 |
| 推理 | `vdn-models` | B | 有 GTCRN · FE-M | 改用后端接口 |
| 引擎 | `vdn-engine` | B | 有 `Engine`（16 kHz） | 补 `Pipeline`（48 kHz：重分块 → 重采样 → Engine → Stage → 回采样 → 原声混合） |
| 引擎 | `vdn-speaker` · `vdn-scene` · `vdn-polish` | B/C | 有，未接入 | 本批只接口对齐，不接入（接入在 backlog「引擎」） |
| 协议 | `vdn-shm` | A/B | 无 | 跨进程环与心跳，`#[repr(C)]` 全原子；loom + Miri |
| 平台 | `vdn-rt-thread` | — | 无 | 实时优先级 · Audio Workgroup · 启动自检 |
| 平台 | `vdn-io-macos` | A | 无 | 采集 / 输出 · 设备监听 · 漂移补偿；先 cpal 直通，再自写 IOProc |
| 平台 | `vdn-hal` | A | 无 | Rust cdylib 虚拟麦克风（D55），只依赖 `vdn-shm` |
| 边界 | `vdn-control` | C | 无 | 配置 · 控制消息 · 本地功能开关；serde 只在这里 |
| 边界 | `vdn-eval` | C | 有 | 改喂 `Pipeline` |
| 入口 | `vdn`（`src/cli/`） | — | 有 `denoise` · `bench` | 扩成总入口（见第四节） |
| 入口 | `vdn-denoised` · `vdn-capture` | — | 无 | 推理进程 · 送音进程（两个进程是 008 #4 的建议，待你定） |
| 工具 | `xtask` | tool | 有 tidy · check-deps · eval | 补 `ci`（本机一键跑全部闸门）· `todo`（统计未实现）· `deny` |

## 四、命令行 `vdn` 草案

用途：一个入口覆盖开发和用户两类操作。子命令名是草案，AI 可定。

| 子命令 | 做什么 | 依赖哪些 crate |
|---|---|---|
| `vdn denoise <in.wav> <out.wav>` | 离线降噪（已有） | engine · models |
| `vdn bench` | 每帧耗时 p50 / p99.9 / max（已有） | engine · models |
| `vdn eval ci｜qa` | 转调 `vdn-eval` | eval |
| `vdn devices` | 列输入 / 输出设备与采样率 | io-macos |
| `vdn run --passthrough｜--denoise` | 实时：麦克风 → 处理 → 扬声器或虚拟麦克风 | io-macos · engine · shm |
| `vdn plugin install｜status｜uninstall` | 装 / 查 / 卸虚拟麦克风（要密码） | hal 的产物 |
| `vdn doctor` | 自检：权限 · 设备 · 模型哈希 · 实时优先级 · 延迟账 | 全部 |

## 五、harness：闸门怎么落地

| 闸门 | 现在 | 本批要做 |
|---|---|---|
| G1–G2 格式 · clippy | 有 | 不变 |
| G3 依赖方向与许可 | 只有 check-deps | 加 `deny.toml` 与 `cargo deny check` |
| G4 tidy | 有 | 加：每个 crate 有 `metadata.vdn` · `lib.rs` 有档位头 |
| G5 单元 + 集成 | `cargo test` | 换 `cargo nextest`；实时 crate 加计数分配器断言 |
| G6 分块不变性 | 无 | proptest：随机切块喂 `Pipeline` = 整段喂 |
| G8 RTSan | 无 | macOS 本机 + CI 一个 job，先 B 档豁免表 |
| G9 Miri · loom | 无 | 只跑 `vdn-shm` · `vdn-dsp` |
| G10 耗时 | `vdn bench` | 进 `xtask ci --local` |
| G11 端到端延迟 | 无 | `vdn run` 脉冲法，等真设备 |
| G12 评测回归 | 有 | 改喂 `Pipeline` 后重写基线 |
| CI 费用 | 全在 macOS | 平台无关的闸门挪到 Linux（规约 §7） |

**骨架完成的判据**（verifier）：
1. 第三节每个 crate 都存在、能编译、`metadata.vdn` 正确
2. `cargo run -p xtask -- ci` 全绿（含 deny · check-deps · tidy · nextest）
3. `vdn denoise` 结果和改造前逐样本一致（防重构改坏声音）
4. `xtask todo` 列出剩余「未实现」，每条挂在一个 crate 下

## 六、分工（每个手下 2–4 个 crate）

| 手下 | 管的 crate | 依赖谁先做完 |
|---|---|---|
| 核心 | `vdn-types` · `vdn-dsp` · `vdn-backend-ort` · `vdn-models` | 无，先开 |
| 引擎 | `vdn-engine`（`Pipeline`）· `vdn-eval` 改喂 Pipeline | 核心的 dsp 积木 |
| 平台 | `vdn-shm` · `vdn-rt-thread` · `vdn-io-macos` · `vdn-hal` | types |
| 入口 | `vdn-control` · `vdn`（cli）· `vdn-denoised` · `vdn-capture` | 前三个的接口卡 |
| （大总管自己） | `xtask` · `deny.toml` · CI | — |

顺序：先由大总管写全部接口卡（一天内，第二节第 2 层），四个手下再并行填实现。

## 七、不在本批

- 修引擎停顿门控 · 认人 / 场景 / 修音接入引擎（backlog「引擎」）
- Core ML / MLX 后端实现（等 019 结论，D53）
- 装插件实验（D05）要你输密码、断音几秒，单独约时间

## 八、和其他线的冲突

- 019 engine 手下在写「GTCRN 手写 Rust」：落进 `vdn-backend-<名>` 槽位即可，不改本批接口
- 本分支从 `chore/coord-20261031-b6`（比 main 多 44 个提交）开出；建议先把那条合进 main，本分支再合入 main

---

## 九、本批定案（人定 2026-11-02）

- 人原话：「先吧基本上的所有骨架都做完 / 就是整个链条的骨架做完，能保证送声音出声音」「记住，你的工作是这一批的大总管，而不是干活的人 / 你要给各种手下派活」
- 深度 = 整条链路的骨架都建好，**声音送得进、出得来，有测试证明**；功能细节（降噪质量、自写 IOProc、真 HAL 驱动全部属性）后续批次
- 进程形态按 008 #4 建议：两个进程（`vdn-capture` 收音送音 · `vdn-denoised` 降噪）。建议，待你确认
- 工程细节（008 / 009 第五组）按建议推进，待你确认
- Pipeline 省 2 ms（人定 2026-11-02，原话「可以省」）：麦克风块长固定且整除模型 hop 时，输出预填从 hop−1 减到 hop−块长（GTCRN 链路 33.98 → 约 32 ms）；代价是换块长时输出不同，「切块不变」检查只在同一块长内成立

## 十、链路与接口约定（四个手下共同遵守；改约定先报大总管）

链路（箭头 = 声音走向；括号 = 哪个 crate）：

```
麦克风 ─(io-macos 输入回调)→ [capture: Forwarder] ─push→ shm 环 Raw
                                                         │
                         [denoised: 循环] ←pop─────────────┘
                              │ Pipeline（engine：48k→16k→Engine→16k→48k）
                              └push→ shm 环 Processed ──pop→ [capture: Forwarder]
                                                              │ 心跳新鲜 → 用 Processed
                                                              │ 心跳过期 → 用本地延迟 L_total 的原声（进程级送原声）
                                                              └push→ shm 环 Out
                                                                       │
                              [hal: HalSource::render] ←pop────────────┘ → 会议软件
                              （开发期没有 HAL 时：io-macos 输出回调读 Out → 扬声器）
```

单声道 f32，IO 采样率固定 48 000 Hz；所有延迟按 48 kHz 样本记。

| crate | 必须对外提供（签名可微调，语义不变） |
|---|---|
| `vdn-dsp` | `resample::{Decimator3, Interpolator3, delay_high()}`：3 倍降 / 升，固定延迟，实时块不分配。草稿已在 `src/dsp/src/resample.rs`（大总管起的头，**未编译未测**，归核心手下核实或重写） |
| `vdn-engine` | `Pipeline<M: Model>`：`new(engine, io_rate) -> Result<Self, PipelineError>` · `process(&mut self, input: &[f32], output: &mut [f32])`（任意块长、无上限，输出与输入等长；最终签名见 rs-core 现状页，2026-11-02 定）· `set_mode(Mode::{Denoise, Passthrough})`（两模式延迟相同，切换按 hop 淡化）· `latency_samples()`（48 kHz）。`testing::Identity`（输出 = 输入、16 kHz、hop 256、延迟 0），供没有模型权重时跑通链路 |
| `vdn-backend-ort` | 从 `vdn-models/src/session.rs` 拆出：`open(path) -> Result<Session, BoxError>` · `zeros` · `pub use ort`；之后 `vdn-models` 不再直接依赖 `ort` |
| `vdn-shm` | `Bus`：`create(name, BusConfig) / open(name)`（POSIX 共享内存，跨进程）· `heap(BusConfig)`（进程内，测试用）。`BusConfig { rate, capacity_frames, l_total }`。`ring(RingId::{Raw, Processed, Out})` 给出单生产者 `push(&[f32]) -> usize` / 单消费者 `pop(&mut [f32]) -> usize` / `available()`；头部：`magic` · `version` · `beat()` / `heartbeat()`（降噪进程每块 +1）· `l_total()` · 计数器（溢出 · 欠载）。样本槽用 `AtomicU32` 存 `f32::to_bits`，读写索引 Release / Acquire。`unsafe` 只在 mmap 与布局转换处 |
| `vdn-rt-thread` | `promote_current(RtParams { period_ns, computation_ns, constraint_ns }) -> Result<(), RtError>`（mach time-constraint 策略）· `check_current() -> Result<RtStatus, RtError>`（读回策略，自检用；2026-11-02 改为 Result）。Audio Workgroup 加入：本批返回 `RtError::Unimplemented` |
| `vdn-io-macos` | `devices() -> Vec<DeviceInfo>` · `InputStream::open(device: Option<&str>, rate, block, cb: impl FnMut(&[f32]) + Send + 'static)` · `OutputStream::open(…, cb: impl FnMut(&mut [f32]) + Send + 'static)`；多声道设备取第 1 声道 / 复制到各声道。本批用 cpal（规约 §4：P3 允许，A 档按已知违规清单） |
| `vdn-hal` | `HalSource::new(consumer)` · `render(&mut [f32]) -> RenderStatus`（不够就补零并计欠载）；cdylib 工厂函数 `VdnHalFactory` 先返回空指针（= 不加载，安全）；`bundle/Info.plist` 模板。真驱动接口归后续批次（装插件要人输密码，D05） |
| `vdn-control` | `Config`（serde_json 文件：设备 · 块长 · 模式 · 模型 · 总线名）· `Cmd`（小 `Copy` 枚举：SetMode · Shutdown）。serde 只许在这里和 bin 里 |
| `vdn-capture`（lib + bin） | `Forwarder`：输入一块原声 → 推 Raw → 从 Processed 取同样多 → 按心跳决定用处理后的还是本地延迟原声 → 推 Out。**心跳超过 100 ms 不动就切原声**，恢复后淡回 |
| `vdn-denoised`（bin） | 建总线 → 循环：Raw 够一块就跑 `Pipeline` → 推 Processed → `beat()`；`--passthrough` 用 `Identity`，不需要模型权重 |
| `vdn`（cli） | （2026-11-02 定：forbid 去掉 cpal，开发期单进程入口可用 io-macos；cpal 仍禁在核心 crate 与 `vdn-denoised`）新增 `devices` · `run --passthrough｜--denoise`（单进程开发模式：麦克风 → Pipeline → 扬声器）· `chain-test`（无设备：合成信号走完整链路，打印结果并按判据退出）|

## 十一、「送声音出声音」的判据（verifier，机器判）

| # | 判据 | 怎么跑 | 过线 |
|---|---|---|---|
| V1 | 直通链路逐样本对齐 | `vdn chain-test --mode passthrough`（进程内线程 + 真 POSIX 共享内存，440 Hz + 2 kHz 正弦 5 s） | 输出与「输入延迟 L_total」的相关 ≥ 0.999，欠载 0 |
| V2 | 降噪进程死掉仍出声 | `chain-test --kill-denoiser-at 2s` | 2 s 后 ≤ 150 ms 内切到原声；全程无 > 150 ms 的静音段 |
| V3 | 真模型链路出声 | `chain-test --mode denoise`（本机有 GTCRN 权重时） | 输出非静音、有限值、长度对齐 |
| V4 | 两个真进程 | 先起 `vdn-denoised --passthrough`，再起 `vdn-capture --source sine --sink file` | 文件里是延迟后的正弦（同 V1 判据） |
| V5 | 真麦克风（人判，可选） | `vdn run --passthrough --seconds 5` | 你从扬声器 / 耳机听到自己的声音 |
| V6 | 闸门 | `cargo run -p xtask -- ci`（fmt · clippy · test · tidy · check-deps） | 全绿 |
