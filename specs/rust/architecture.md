# Rust 架构规约（vdn workspace）

> **规约**：Rust 代码该长什么样。依据与取舍见 `docs/research/008_rust-architecture/`（`decision.md` 取舍卡 · `adr/` 五条决策记录 · `notes/` 调研原件 · `appendix-probes/` 本机探针）。
> 起草：手下 rustarch，2026-10-30。**AI 方案，待人过目**；依赖人拍板的地方标「⟨待拍板 N⟩」，编号对应 `decision.md` 末尾。
> 证据等级：【官】官方明载 ·【源】源码可证 ·【测】本机实测 ·【传】二手 ·【推】推断。
> 本规约**从 p1 骨架（分支 `feat/p1-offline-gtcrn`：`dsp ← engine ← models ← cli`）长出来**，不推倒重来：§2 写清每个阶段加什么、什么时候拆。

---

## 0. 一页纸

1. **模块化靠 crate 图**：一个 crate 一个职责，核心 crate 不碰平台、不碰推理库、不碰序列化。「离线评测与实时用同一份处理代码」由依赖图保证（§2）。
2. **「AST 做模块化」的用法：用 AST 守规矩，不用 AST 造代码。** clippy 禁用清单、cargo-deny、xtask 依赖检查、tidy 测试在语法树和依赖图上把架构规则变成 CI 红灯；过程宏生成代码先不用，触发条件写死（§6）。
3. **实时分三档**（§4）：A 档 IO 回调零分配零锁、机器可查；B 档推理线程里我们自己的代码零分配，ONNX Runtime 内部的分配登记在案，靠截止时间和送原声兜底；C 档其他线程不限。依据是本机实测：ort 跑 GTCRN 每帧约 1554 次 malloc、376 对 mutex【测】`docs/research/008_rust-architecture/appendix-probes/README.md` §1。
4. **核心 trait 用 p1 的波形接口**：`Model`（一个 hop 进、一个 hop 出），加 `Send`；引擎静态分发，运行期选模型用手写 `enum AnyModel`（§3）。
5. **每条实时规则都配机器检查**（§5），人肉审查不算。

---

## 1. 目录与命名

| 东西 | 约定 | 例 | 依据 |
|---|---|---|---|
| workspace 根 | 虚拟 manifest，`resolver = "3"`，edition 2024；`[workspace.package]` 统一 version / edition / license=`GPL-3.0-or-later` / `publish = false`；`[workspace.dependencies]` 统一版本；`[workspace.lints]` 统一 lint | p1 根 `Cargo.toml` 已是这样 | Cargo Book workspace 继承【官】；matklad「Large Rust Workspaces」【官·作者一手】 |
| crate 目录 | `src/<短名>/`，**扁平一层**，不嵌套 | `src/dsp/`、`src/engine/` | `.42cog/cog.md` 实体表（crate 住 `src/<crate>/`）；matklad 主张扁平布局【官·作者一手】 |
| package 名 | `vdn-<短名>`；lib 路径 `vdn_<短名>` | `vdn-dsp` / `vdn_dsp` | Bevy、Ruff 都给内部 crate 加项目前缀【官】；目录名与包名是否必须一致 ⟨待拍板 2⟩，默认保持 p1 现状 |
| bin 名 | 开发期命令行 `vdn`；两层进程各一个：`vdn-capture`（送音进程）、`vdn-denoised`（推理进程） | — | 避开 `infer` 一词的两种意思（后端 vs 进程）`notes/r8-critic.md` C7 |
| 推理后端 crate（拆出来之后） | `vdn-backend-<名>` | `vdn-backend-ort` | 同上 |
| 工具 | `xtask/`（workspace 成员，`publish = false`），用 `cargo xtask <子命令>` 跑 | `cargo xtask tidy` | rust-analyzer、matklad「cargo xtask」【官】 |
| 实时代码 | 会在 A / B 档线程上跑的函数，放在**实时 crate**（§4.2 列表），crate 根写 §5.2 的 deny 头；构造函数等非实时函数逐个 `#[allow]` 并注 `// NRT:` | — | tympan-aspl 在 `realtime` 模块根写 `#![deny]`【源】`resources/tympan-aspl/src/realtime/mod.rs:45` |
| 模型权重 | 仓外；仓里只留来源 · 许可 · sha256（CLAUDE.md 铁律 8） | GTCRN simple sha256 b4718df6…ebcb87【测】 | `.42cog/cog.md`「模型权重」行 |
| 构建产物 | `/target/` **在第一次构建前**就写进 `.gitignore`；每个新 worktree 开工先确认 | — | 2026-10-30 p1 分支误提交 target/（1799 文件、146 MB），CLAUDE.md 第五节第 2 条 |

---

## 2. crate 列表与依赖方向

### 2.1 目标形态（P4 时）

| crate | 层 | 档 | 职责 | 允许的外部依赖 | 不许依赖 |
|---|---|---|---|---|---|
| `vdn-types` | 核心·词汇 | A/B | `Model` trait、`ModelError`、`FrameCtx`、`Status`、`SpeakerVerdict`（打包进 `AtomicU64`）、`SharedParams` 布局 | 无 | 一切平台库、推理库、serde |
| `vdn-dsp` | 核心 | B | STFT/iSTFT、延迟线、包络（快开慢关）、交叉淡化、biquad、压缩限幅、FTZ guard、**重分块器**（任意长度进 → 整 hop 出）、固定比例重采样包装 | realfft、rubato | 同上 |
| `vdn-engine` | 核心 | B | 逐 hop 调度：重采样 → 模型 → 三件保命 → 门控 → 音色链；模型级送原声；`latency_samples` 求和；**eval 与实时进程唯一的处理入口** | `vdn-types`、`vdn-dsp` | 推理库、平台库、serde、hound |
| `vdn-models` | 核心 | B | 每个模型一个文件（GTCRN、FastEnhancer-M）：帧规格、状态张量形状、前后处理；`AnyModel` | `vdn-types`、`vdn-dsp`；P1 期间直接依赖 ort，拆后端后改依赖后端 trait | 平台库、engine |
| `vdn-speaker` | 核心 | C | fbank、声纹打分、自动注册、门控判决；每 0.5 s 一次；结果写进 `SpeakerVerdict` | `vdn-types`、`vdn-dsp` | 平台库、engine |
| `vdn-backend-ort` / `vdn-backend-tract` | 适配·推理 | B | 推理后端；兄弟 crate，互不依赖 | ort / tract-onnx | 平台库、engine |
| `vdn-shm` | 协议 | A/B | 跨进程共享内存：`#[repr(C)]` 头（magic、版本、采样率、`L_total`、心跳）、自写 SPSC；核心部分 `#![no_std]`；cbindgen 生成 C 头给 HAL 插件 | 核心无；`posix` feature 用 libc | engine、推理库 |
| `vdn-rt-thread` | 适配·平台 | — | 提实时优先级、加入 / 自建 Audio Workgroup（手写约 60 行 `extern "C"`）、启动自检；两个进程共用 | libc / mach 绑定 | engine、推理库 |
| `vdn-io-macos` | 适配·平台 | A | cpal 采集与输出、设备监听与热重建、rtrb 进程内环、`Slip` 漂移补偿、进程级原声旁路 | cpal（钉版本）、objc2-core-audio、rtrb、rubato | engine、推理库 |
| `vdn-control` | 边界 | C | 配置、预设、控制消息、序列化；**serde 只许在这里和 bin 里** | serde、toml | 平台库、推理库 |
| `vdn-eval` | 边界·离线 | C | 读 WAV 逐帧喂 `vdn-engine`；指标、场景矩阵、报告 JSON | hound、serde、`vdn-engine`、后端 | `vdn-io-macos`、`vdn-rt-thread` |
| `vdn`（bin，现 `src/cli`） | 应用 | — | 开发期命令行：denoise / bench / eval | — | — |
| `vdn-capture`（bin） | 应用 | — | 送音进程：io-macos + shm + control；推理进程死掉时送原声 | — | **推理库、`vdn-engine`、`vdn-models`**（D03 崩溃隔离） |
| `vdn-denoised`（bin） | 应用 | — | 推理进程：shm + engine + 后端 + speaker + rt-thread | — | `vdn-io-macos` |
| `xtask` | 工具 | — | tidy、check-deps、模型下载校验 | 尽量少 | — |

HAL 插件用 C 写（`docs/research/003_mac-audio-pipeline/decision.md`「我选择什么」），**不在 Rust workspace 里**，只吃 `vdn-shm` 生成的头文件。它的目录是新顶层目录 ⟨待拍板 5⟩。

### 2.2 依赖方向图

```
 核心（离线评测与实时共用；依赖树里没有 cpal / objc2 / ort / tract / serde / hound）
 ┌──────────────────────────────────────────────────────────────────────┐
 │                         vdn-types                                     │
 │                ┌───────────┼────────────┬──────────────┐              │
 │                ▼           ▼            ▼              ▼              │
 │            vdn-dsp ──► vdn-models   vdn-speaker   （后端 trait）       │
 │                │           │                                          │
 │                └─────► vdn-engine ◄─── 只认 trait，不认具体后端         │
 └──────────────────────────────┼────────────────────────────────────────┘
 ═══════════════════════════════╪════════════ 核心 / 适配分界 ══════════════
                                │
   vdn-backend-ort   vdn-backend-tract        vdn-shm (no_std)  ──cbindgen──► HAL 插件（C，仓内不在 workspace）
          │                 │                   │      │
          └──────┬──────────┘                   │      │
                 ▼                              ▼      ▼
   vdn-eval ◄── 后端 + engine        vdn-denoised     vdn-capture
   vdn (cli)                         (engine+后端      (io-macos + shm + control；
                                      +speaker+shm      不链 engine / 推理库)
                                      +rt-thread)              │
                                                               ▼
                                                         vdn-io-macos ── 唯一碰 CoreAudio
   vdn-control ◄── 只被 bin 依赖
```

箭头 = 「被谁依赖」方向自上而下；**任何核心 crate 都不许依赖分界线以下的东西**（§5 R12 机器检查）。

### 2.3 分阶段：从 p1 骨架长到目标形态

| 阶段 | 现在有 | 这一步加 / 拆 | 触发理由 |
|---|---|---|---|
| **P1**（p1 骨架） | `vdn-dsp` · `vdn-engine`（含 `model.rs`）· `vdn-models`（直接用 ort）· `vdn-cli` | 不拆。加：`Model: Send`、`AnyModel`、`process` 里的 `assert_eq!` 改 `debug_assert_eq!`（长度不符整 hop 送原声）、bench 计每 hop 分配次数、`.gitignore` `/target/`、`xtask`（tidy + check-deps 两个子命令） | 4 个 crate 够用；拆分没有新的依赖边界要守 |
| **P2**（离线认人） | 同上 | 加 `vdn-speaker`；**把 `engine::model` 拆成 `vdn-types`**（speaker 要发布结果给 engine，但不许依赖 engine）；指标与报告一出现就拆 `vdn-eval` | §2.4 判据 1、判据 5 |
| 接 tract 对照时 | `vdn-models` 依赖 ort | 拆 `vdn-backend-ort` / `vdn-backend-tract`，`vdn-models` 只依赖后端 trait | 判据 3（同一接口可替换实现）；不用互斥 feature【官】Cargo Book「Features」 |
| **P3**（实时直通） | — | 加 `vdn-io-macos`、`vdn-rt-thread`，bin `vdn-capture`（单进程直通） | 判据 1、判据 6（unsafe 集中） |
| **P4**（实时降噪 + 两层进程） | — | 加 `vdn-shm`、bin `vdn-denoised`、`vdn-control` | 判据 4（给 C 出头文件、`no_std`）；D03 两层进程 |

### 2.4 什么时候拆 crate、什么时候只是 module

满足**任一条**就拆 crate，一条都不满足就留作 module【官·作者一手】matklad「Fast Rust Builds」：「The most important property of a crate is which crates it doesn't (transitively) depend on」：

1. **要机器守的依赖边界**：它不许看见某个依赖，或某个依赖不许看见它（本项目最主要的理由）
2. **实时档不同**：A/B 档与 C 档用不同的 lint 强度
3. **同一接口的可替换实现**：后端、平台
4. **产物或使用方不同**：bin、给 C 出头文件、`no_std`
5. **重依赖隔离**：ort、tract、serde、hound 放叶子
6. **unsafe 集中**：其余 crate 都 `#![forbid(unsafe_code)]`

不拆的情形：只有一个使用方；和兄弟代码共用私有不变量（要 `pub(crate)`）；泛型会落在 crate 边界上（matklad：「do not put generic code at the boundaries between the crates」）。**不建** `utils` / `common` 杂物 crate；代码量不是判据。

---

## 3. 核心 trait 与数据流

### 3.1 模型适配器（`vdn-types`，P2 前住 `vdn-engine::model`）

在 p1 现有 trait 上只改两处：加 `Send`；构造与运行分开写进约定。

```rust
/// 模型适配器。实现者：Gtcrn、FastEnhancer、测试用 DelayGain。
/// 约定：
/// - 构造（`open` 等）在非实时线程上做，可以分配、加载文件、预热；实时线程只调 `process` / `reset`
/// - 「一个 hop 进、一个 hop 出」的波形接口；频域模型在适配器里自己做 STFT（用 vdn-dsp）
/// - 输出比输入晚 `latency_samples()` 个样本
/// - `process` 不分配（Rust 侧）、不加锁、不做 IO、不 panic；出错返回 Err
pub trait Model: Send {
    fn name(&self) -> &'static str;
    fn sample_rate(&self) -> u32;
    fn hop(&self) -> usize;
    fn latency_samples(&self) -> usize;
    fn reset(&mut self);
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError>;
}

/// 出错路径也不分配：不带 String（p1 已是这样）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelError { Backend, Shape }
```

**为什么用波形接口而不是频域接口**：GTCRN 是 512/256 STFT 频域模型【源】`resources/biglinux-gtcrn/stream/gtcrn_stream.py:371`，FastEnhancer 有自己的前端（`resources/fastenhancer` 是官方源码，形状待查【源】`notes/r7-skeptic.md` C25）；外层统一成波形，换模型时引擎不用知道谁做 STFT。设计稿 §11「所有频谱处理共用一次 STFT」留到音色链真要频谱时，再加一个**扩展 trait**（例如 `SpectralTap`），不改 `Model`。

### 3.2 分发方式：静态为主，运行期选择用手写 enum

```rust
// vdn-models
pub enum AnyModel { Gtcrn(Gtcrn), FastEnhancer(FastEnhancer) }
impl Model for AnyModel { /* 每个方法一个 match，手写 */ }

// vdn-engine
pub struct Engine<M: Model> { model: M, /* … */ }
// 实时进程与 eval 都用 Engine<AnyModel>；单测用 Engine<DelayGain>
```

- 每秒约 62 次模型调用（hop 16 ms），虚调用开销相对约 0.75 ms 的推理可以忽略【测】`appendix-probes/README.md` §1；**动态分发的真风险是在实时线程上 new / drop 一个 Box**，不是 vtable【推】`notes/r3-traits.md` §3
- 选 enum 而不是 `Box<dyn Model>`：变体是已知的封闭集合（2 个），换模型时整个 `AnyModel` 值经队列交接，不涉及堆上 trait 对象的释放；**两个变体不需要 `enum_dispatch` 宏**（§6 不生成代码）
- p1 CLI 现在的 `Box<dyn Model + Send>` 与 `AnyModel` **二选一**，P1 收尾时统一成 `AnyModel`

### 3.3 引擎入口与后处理链

```rust
// vdn-types
#[derive(Clone, Copy)]
pub struct FrameCtx {
    pub sample_index: u64,          // 时间只用样本计数，eval 可复现时序
    pub speaker: SpeakerVerdict,    // 本 hop 开头读到的最新声纹判决
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status { Ok, Bypassed(FaultKind) }

// vdn-engine
impl<M: Model> Engine<M> {
    /// eval 与实时进程唯一的处理入口。input.len() == output.len() == hop()
    pub fn process(&mut self, input: &[f32], output: &mut [f32], ctx: &FrameCtx) -> Status;
    /// 整条链的延迟 = 重采样 + 模型 + 各 Stage 之和；启动时断言不超预算（R13）
    pub fn latency_samples(&self) -> usize;
}

/// 模型之后的一级处理（门控、衰减下限、音色链各节点）
pub trait Stage: Send {
    fn latency_samples(&self) -> usize { 0 }
    fn reset(&mut self);
    /// wet 原地改写；dry 是已对齐延迟的原声
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: &FrameCtx);
}
```

- 后处理链是**固定字段的 struct**（`PostChain { floor: …, gate: …, timbre: TimbreChain }`），按设计稿 §3 的固定顺序静态调用，不用 `Vec<Box<dyn Stage>>`：顺序是产品决定，不是运行期配置
- 任意长度的音频块（IO 回调、eval 随机切块）先经 `vdn-dsp` 的**重分块器**攒成整 hop 再进 `process`。这是「分块不变性」性质测试的对象（§7 G6）
- p1 的 `process(&mut self, input, output)` 返回 `()`；P2 引入 `FrameCtx` 时一并改成返回 `Status`

### 3.4 状态与参数跨线程

| 数据 | 方向 | 用什么 | 内存序 | 依据 |
|---|---|---|---|---|
| 连续参数（衰减下限、门限、EQ 增益…） | 控制 → 实时 | `SharedParams`：一组 `AtomicU32` 存 `f32::to_bits`，启动前建好的 `Arc`（实时侧只读，不 clone、不 drop） | Relaxed；实时侧进平滑器 | nih-plug `FloatParam` 用 Relaxed【源】`notes/r3-traits.md` §4.1 |
| 一组必须同时生效的参数（如 EQ 五段） | 控制 → 实时 | rtrb 送一个 `Copy` 的系数包 | rtrb 内部保证 | 同上 §4.3 |
| 离散命令（Reset、Bypass、换模型） | 控制 → 实时 | rtrb `Producer<Cmd>`，`Cmd` 是小 enum | 同上 | rtrb wait-free、构造后不分配【源】`resources/rtrb/src/lib.rs:1-9` |
| 声纹判决 | 声纹线程 → 实时 | `SpeakerVerdict`（状态 + 置信度 + 序号）打包进**一个 `AtomicU64`**；将来超过 64 位再换 triple_buffer（MPL-2.0，与 GPL-3.0 兼容） | 写 Release / 读 Acquire | 设计稿 §6「用原子变量传给推理线程」；triple_buffer 许可【源】`notes/r7-skeptic.md` C23 |
| 换模型 | 控制 → 实时 → 控制 | 新 `AnyModel` 经 rtrb 送入；**旧的经另一条 rtrb 送回**由非实时线程 drop；回收队列满就推迟切换 | 同上 | fundsp `NetReturn` 模式【源】`notes/r3-traits.md` §4.1 |
| 电平、计数、xrun | 实时 → UI | 一组原子 | Relaxed | — |
| 进程间音频 | 送音进程 ↔ 推理进程 | `vdn-shm` 自写 `#[repr(C)]` SPSC；rtrb 放不进共享内存 | Release / Acquire；loom 验证 | `resources/rtrb/src/lib.rs:114,159`（003 decision 已引） |

**不用**：`Mutex` / `RwLock` / `try_lock`（Doumler：「you should not use std::mutex on the audio thread, not even with try_lock」【官】https://timur.audio/using-locks-in-real-time-audio-processing-safely）；实时读端不用 arc-swap（读者可能做最后一次引用计数递减并释放【官+推】`notes/r7-skeptic.md` C15）。

### 3.5 送原声：两层，分工写死

| 层 | 谁做 | 挡什么 | 干声延迟 |
|---|---|---|---|
| **模型级** | `vdn-engine`（p1 已实现：报错 / NaN / 幅度异常 / 超截止时间 → 送对齐后的原声，接缝衰减，切回交叉淡化） | 单帧故障、模型状态坏、换模型 | `Engine::latency_samples()` |
| **进程级** | 送音进程的输出回调（`vdn-io-macos`），读原声环 C | 推理进程卡死、崩溃、心跳停 | 同一个 `L_total`，由推理进程写进 `vdn-shm` 头 |

两层用**同一个** `L_total`，不各自对齐（`notes/r8-critic.md` C9）。003 已定「原声旁路由输出回调做」【源】`docs/research/003_mac-audio-pipeline/decision.md`「分阶段方案」M1–M2 行。

---

## 4. 实时分档

### 4.1 三档

| 档 | 是哪些线程 | 承诺 | 验收方式 |
|---|---|---|---|
| **A 硬实时** | CoreAudio IO 回调（送音进程的输入 / 输出回调）；以后 HAL 插件的 IO（C，另按 Apple 头文件 `CA_REALTIME_API`） | **零分配、零锁、零系统调用、不 panic**，包括第三方代码 | 机器零容忍：§5 全部检查，RTSan **无任何豁免** |
| **B 软实时** | 推理进程的降噪线程（`Engine::process` 及其下的一切） | **我们自己的 Rust 代码零分配、零锁**；ONNX Runtime 内部的分配与 arena 锁**允许**，但必须登记在 RTSan 豁免文件里；每帧有截止时间，超时 / 出错由模型级送原声兜底 | Rust 侧：`assert_no_alloc` / 计数分配器 = 0；C++ 侧：RTSan 只允许豁免文件里的栈，出现新栈即红；耗时看 p99.9 与 max，不看均值 |
| **C 非实时** | 声纹线程、控制、日志、eval、CLI、构造期 | 不限，但不许阻塞 A/B 档等的东西 | 常规测试 |

**为什么 B 档不承诺零分配**：本机实测 ort 2.0.0-rc.15 跑 GTCRN simple，IoBinding 绑一次、单线程、预热 200 帧后，稳态每帧约 **1554 次 malloc、1555 次 free、376 对 mutex lock/unlock**，1 帧 → 2 帧严格翻倍；来源是 ORT 的 `BFCArena`、`Tensor::InitOrtValue`、MLAS GEMM【测】`appendix-probes/README.md` §1。同时耗时余量很大：最坏一帧约 1 ms，hop 16 ms【测】。所以「推理线程零分配」靠 ort 做不到，靠截止时间 + 送原声兜底更现实。降低分配的后续手段（关 arena、自定义分配器、tract 对照）见 `decision.md`「仍未补齐」。

**与 p1 / exp004「每帧 0 次分配」的关系（务必写进 exp004 README 补注）**：p1 测的是 **Rust 全局分配器**（`assert_no_alloc` / 计数分配器）。ONNX Runtime 是 C++，直接调 libc `malloc`，Rust 全局分配器**看不到**。所以「Rust 侧 0 次」和「C++ 侧约 1550 次」同时成立，不矛盾；只有 RTSan 这类在 libc 层拦截的工具看得到后者【源+测】`notes/r7-skeptic.md` C1、`appendix-probes/README.md` §1。Rust 侧是不是 0 取决于调法：p1 用 `run_with_options` + `OutputSelector` 预分配全部输出，实测 GTCRN 每帧 0 次【测·p1】（rusteco 009 `decision.md` §一.4）；探针里不预分配输出的 `run` 调法是 54 次、`run_binding` 是 28 次（`GetBoundOutputValues` 后 collect 出新的 `SessionOutputs`）【测】【源】`ort-2.0.0-rc.15/src/session/run.rs:288-320`。所以**规定 B 档的 ort 调法只用「预分配全部输出」或 raw `RunWithBinding`**。另外 ort 把输入 ≤ 6、输出 ≤ 4 放在栈上，FastEnhancer-M 是 7 进 7 出，超出上限，**每帧 7 次小分配**【源+测·p1】`ort/src/util/stack.rs:13-15`（rusteco 009 同节）——要么登记为 B 档已知分配，要么把状态张量合并后重新导出。

### 4.2 实时 crate 名单

| crate | 档 | 实时入口（打 `#[nonblocking]` 的函数） |
|---|---|---|
| `vdn-dsp` | B（A 档也会调其中的淡化、Slip） | 各 `process` |
| `vdn-engine` | B | `Engine::process` |
| `vdn-models` / `vdn-backend-*` | B | `Model::process`、`reset` |
| `vdn-shm` | A/B | 读写函数 |
| `vdn-io-macos` | A | 回调模块 `callback` |

---

## 5. 实时安全规则：每条配机器检查

**机器检查栈**（§6 的 AST 用法落地在这里）：
- **静态**：一份根目录 `clippy.toml` 列禁用清单（§5.1）；`[workspace.lints.clippy]` 把 `disallowed_methods/types/macros` 设为 `allow`，**实时 crate 根**写 §5.2 的 deny 头把它们升为错误。只有根目录一份配置——crate 目录下的 clippy.toml 会整份替换上层、不合并【测】`appendix-probes/README.md` §5，分散写就违反 CLAUDE.md 铁律 7（唯一编码）
- **运行期**：`rtsan-standalone` 0.3（stable，macOS 实测可用）给实时入口打 `#[nonblocking]`，`RTSAN_ENABLE=1` 构建，经 `cargo test` / `cargo run` 跑 eval 回放【测】；`assert_no_alloc` 包住测试里的 `process`（**只在 debug 生效**，默认 feature `disable_release`【源+测】）；bench 用计数分配器（release 也有效）
- **依赖图**：cargo-deny `[bans]`（管**直接**依赖边）+ `cargo xtask check-deps`（读 `cargo metadata` 求**传递闭包**）。cargo-deny 的 `wrappers` 只检查被禁 crate 的直接依赖者【源】`resources/cargo-deny/src/bans.rs:616-650`，`engine → io-macos → cpal` 它不报，所以 check-deps 这一层不能省
- **xtask tidy**：照 rust-analyzer，写成普通 `#[test]` 随 `cargo test` 跑，做文本 / 轻语法检查【源】rust-analyzer `xtask/src/tidy.rs`

| # | 规则 | 适用 | 机器检查 |
|---|---|---|---|
| R1 | 实时路径不分配、不释放 | A 全部；B 我们的代码 | ① clippy `disallowed-methods/macros`（§5.1）② 测试里 `assert_no_alloc` 包 `process`（debug）③ bench 计数分配器每 hop = 0（release）④ RTSan `#[nonblocking]`：A 档零豁免，B 档只豁免 `libonnxruntime` 栈（豁免文件进仓，改动要说明） |
| R2 | 实时路径不加阻塞锁 | 同上 | ① clippy `disallowed-types`：`Mutex`、`RwLock`、`Condvar`、`std::sync::mpsc` ② RTSan 拦 `pthread_mutex_lock/unlock`【测】；注意 macOS 上 `std::sync::Mutex` 第一次 lock 还会 malloc【测】 |
| R3 | 实时路径不做 IO、日志、sleep、系统调用 | A、B | ① clippy `disallowed-macros`：`std::println`、`std::eprintln`、`std::print`、`std::eprint`、`std::dbg`、`std::format`、`log::*`；`disallowed-methods`：`std::thread::sleep`、`std::fs::*`（**`std::io::stdout` 拦不住 `println!`**【测】）② RTSan（拦 `nanosleep` 已实测）③ 日志只经 rtrb 送日志线程 |
| R4 | 实时路径不 panic | A、B | ① 实时 crate 根 deny `clippy::unwrap_used`、`expect_used`、`panic`、`indexing_slicing`、`unreachable`、`todo`；`assert!` 一律改 `debug_assert!` ② 故障注入测试（p1 的 `FaultInjector`）③ `#[no_panic]` 在 dsp 叶子函数上**有界试用**（只作参考：它证明的是这次构建的调用图，调用方传定长会让有风险的代码通过【测】） |
| R5 | 不用最坏情况无界的算法（摊还 O(1) ≠ 最坏 O(1)） | A、B | ① bench 看 p99.9 / max（§7 G19）② specs 负例（代码审查判据，辅助） |
| R6 | 跨线程只用 §3.4 列的原语 | A、B | ① clippy `disallowed-types`（R2 同一份清单）② 自写无锁结构只许在 `vdn-shm`，那里跑 loom + Miri |
| R7 | 实时线程不 drop 堆对象（换模型、换系数表） | A、B | ① RTSan 拦 `free` ② `assert_no_alloc` 同时计 dealloc【源】③ 回收队列满 → 推迟切换的单测 |
| R8 | 实时线程有 time-constraint 优先级并加入 Audio Workgroup | A、B | 启动自检：`thread_policy_get` 读回策略（`resources/audio_thread_priority/src/rt_mach.rs:91-106` 的做法）；`os_workgroup_join` 返回码检查；失败写日志并计数，不静默 |
| R9 | FTZ 由处理域 guard 统一打开 | B | 单测：guard 内 `4e-45` 变 0、FPCR bit 24 置位【测】`appendix-probes/README.md` §3；eval 与实时用同一个 guard（数值才一致） |
| R10 | unsafe 只在白名单 crate | 全仓 | ① `[workspace.lints.rust] unsafe_code = "deny"`；非白名单 crate 的 `lib.rs` 写 `#![forbid(unsafe_code)]`（crate 内可从 deny 升 forbid；workspace 设 forbid 再局部 allow 会报 E0453【测】）② `clippy::undocumented_unsafe_blocks = "deny"` ③ xtask tidy 检查每个非白名单 crate 有 forbid 头。白名单：`vdn-shm`、`vdn-rt-thread`、`vdn-io-macos`、`vdn-backend-ort`（raw `RunWithBinding` 要 unsafe） |
| R11 | 实时路径出错 = 返回 `Status` + 送原声，不传播、不阻塞上报 | B | ① `#[must_use] Status` ② 故障注入：模拟超时 / NaN → 断言输出 = 对齐原声（带淡化） |
| R12 | 依赖方向（§2.2）：核心不依赖适配层；`vdn-capture` 不链推理库与 engine | 全仓 | ① cargo-deny：把**内部 crate 本身**列进 deny 并给 wrappers，例如 `{ name = "vdn-io-macos", wrappers = ["vdn-capture"] }`、`{ name = "ort", wrappers = ["vdn-models"] }`（P1）→ 拆后 `["vdn-backend-ort"]`（cargo-deny 默认允许 ban 工作区成员【源】`bans/cfg.rs:567`，**未实测**）② `cargo xtask check-deps`：对每个核心 crate 和每个 bin 求依赖闭包，与禁表求交，非空即红（等价人工命令 `cargo tree -p vdn-capture -i ort` 为空） |
| R13 | 延迟有账：`Engine::latency_samples()` = 各级之和，不超预算 | B | 单测断言 `latency_samples / sr ≤ 预算`；P3 用脉冲法实测端到端（§8 E-lat）。按默认参数估算约 **56 ms，超 D32 的 45 ms**（STFT 32 + rubato `Fft` 默认两跳约 10.7 + IO 缓冲 512 帧 10.7 + HAL 余量 2.7，另加跨进程环余量）【推】rusteco 009 `decision.md` §一.3；调参后的目标：重采样两跳各 ≤ 2 ms、IO 缓冲 256 帧、环余量 ≤ 3 ms，合计 ≤ 45 ms |
| R14 | 构建产物不进库 | 全仓 | xtask tidy 检查 `.gitignore` 含 `/target/`；auto-commit hook 之前先有忽略规则 |

### 5.1 禁用清单（根目录 `clippy.toml` 的正本内容）

```toml
disallowed-methods = [
  "alloc::vec::Vec::push", "alloc::vec::Vec::with_capacity", "alloc::vec::Vec::reserve",
  "alloc::vec::Vec::extend_from_slice", "alloc::boxed::Box::new",
  "alloc::string::String::from", "alloc::borrow::ToOwned::to_owned", "alloc::string::ToString::to_string",
  "std::thread::sleep", "std::thread::spawn",
  { path = "std::fs::File::open", reason = "实时路径不做 IO" },
]
disallowed-types = [
  "std::sync::Mutex", "std::sync::RwLock", "std::sync::Condvar",
  "std::sync::mpsc::Sender", "std::sync::mpsc::Receiver",
]
disallowed-macros = [
  "std::println", "std::eprintln", "std::print", "std::eprint", "std::dbg",
  "std::format", "alloc::vec",
]
```

（路径写法已实测命中：`std::vec::Vec::push` 与 `alloc::vec::Vec::push` 都可以，`disallowed-macros` 能按路径禁 `std::println` / `std::format` / `alloc::vec`【测】`appendix-probes/README.md` §5。清单加项时只改这一处。）局限：clippy 只看本 crate 的调用点，看不到依赖 crate 内部与宏展开内部的调用——这正是运行期 RTSan 补的那一半。

### 5.2 实时 crate 的 deny 头

```rust
#![forbid(unsafe_code)] // 白名单 crate 除外
#![deny(
    clippy::disallowed_methods, clippy::disallowed_types, clippy::disallowed_macros,
    clippy::unwrap_used, clippy::expect_used, clippy::panic,
    clippy::indexing_slicing, clippy::unreachable, clippy::todo,
)]
```

构造函数等非实时函数用 `#[allow(clippy::disallowed_methods)] // NRT: 构造期预分配`。xtask tidy 检查：每个实时 crate 有这个头；每个 `#[allow(clippy::disallowed_*)]` 上方都有 `// NRT:` 注释。

### 5.3 panic 策略

- 实时代码按 R4 写成不 panic；兜底是**进程隔离**：推理进程 panic 就退出，送音进程看心跳停了走进程级原声（§3.5）
- release `panic = "abort"`（两个进程都是）⟨待拍板 3⟩；no_panic 证明构建另开 profile 用 unwind（no-panic 在 abort 下无效【官】no-panic README）
- 注意：`extern "C"` 里 panic 会直接 abort（Rust 1.81 起）【官+测】；coreaudio-rs 的回调是 `extern "C-unwind"`【源】`resources/coreaudio-rs/src/audio_unit/render_callback.rs:766`

---

## 6. 「AST 做模块化」：采用决定

人的原话是「用超级语法树 / ast 来把东西做成模块，或者大部分东西做成可拆分的核心包和模块」。在 Rust 里它能落到三件事，代价差别很大【推】`notes/r1-ast.md` §0：

| 含义 | 本项目怎么用 |
|---|---|
| **A 用语言自身的结构单元拆分**：crate / module / trait / 可见性 / feature 都是编译器在语法树上强制的边界 | **采用，主力**。§2 的 crate 图就是那棵结构树；核心包 = `vdn-types · vdn-dsp · vdn-models · vdn-engine · vdn-speaker` |
| **B 用 AST / 静态分析守住架构**：在语法树或依赖图上写规则，「谁不许依赖谁」「实时函数里不许出现什么」变成 CI 红灯 | **采用，主力**。§5 的检查栈全是这一类，与 CLAUDE.md「能用规则判的，绝不留给人判」同向 |
| **C 用 AST / 宏生成代码**：derive、macro_rules、build.rs、bindgen | **克制**。只有 cbindgen（共享内存头，Rust 为唯一真相源）现在用；其余写明触发条件 |

逐项决定（完整 27 行表与出处见 `notes/r1-ast.md` §6，已按 r7 / r8 更正）：

| 工具 / 做法 | 决定 | 用在哪 | 理由与出处 |
|---|---|---|---|
| crate 拆分（扁平、虚拟 manifest、宽图） | 采用 | 全仓 | rust-analyzer 架构文档【官】；matklad【官·作者一手】 |
| `pub(crate)` 等可见性 | 采用 | crate 内 | Rust Reference【官】 |
| trait 静态分发 | 采用 | `Model`、`Stage` | §3.2 |
| Cargo feature | 只做加法的开关 | 检查开关（如 `rtsan`） | 互斥 feature 会被统一打开【官】Cargo Book |
| clippy `disallowed-*` + restriction lint | **采用，第一优先** | 根 `clippy.toml` + 实时 crate 头 | §5.1、§5.2；路径写法已实测 |
| `[workspace.lints]` | 采用 | 全仓 | 成员不能逐条覆盖【官+测】 |
| cargo-deny bans（含内部 crate） | **采用，第一优先**，只管直接边 | `deny.toml` | §5 R12；wrappers 语义已核源码 |
| `xtask check-deps`（cargo metadata 传递闭包） | **采用，第一优先** | `xtask/` | cargo-deny 管不到的那一半 |
| xtask tidy（文本 / 轻语法，进 `cargo test`） | 采用 | `xtask/` | rust-analyzer tidy.rs【源】 |
| rtsan-standalone `#[nonblocking]` | 采用（不自造 `#[realtime]` 标记） | 所有实时入口 | 本机实测可用【测】；rustc 内置版 `-Zsanitizer=realtime` 已进 nightly 1.93、仍 unstable【官】PR #147935，稳定后再迁 |
| assert_no_alloc | 采用（测试） | engine / dsp 单测、eval 回放 | 只在 debug 生效；看不到 C++ 分配【源+测】 |
| ast-grep YAML 规则 | **第二阶段**：tidy 文本检查漏网时上；规则先配正例负例 | `rules/` | 只懂语法、不懂类型；带属性的函数要写 `follows: attribute_item` 组合【官+推】`notes/r7-skeptic.md` C11 |
| syn 写检查脚本 | 按需，只在 xtask 里 | tidy 升级 | 编译慢、无类型信息 |
| cargo-modules / cargo-depgraph | 只画图，不做门禁 | 里程碑文档 | — |
| dylint 自定义 lint | 暂缓（同一类漏网第三次再上） | — | 要 nightly 与 rustc 内部 API【官+推】 |
| no_panic | 有界试用 | dsp 叶子函数 | 误判多【测】 |
| cargo-semver-checks / cargo-public-api | 暂不用 | 某 crate 单独发布时 | 不对外发布 |
| 自写 derive 宏（如参数表） | **暂不用**。触发条件：参数超过约 10 个，**或**手抄参数表第三次出错 | 将来的 `vdn-control` 参数表 | 编译时间、IDE、可读性代价【官】Rust 编译性能调查 2025；先例 nih-plug `#[derive(Params)]`【源】 |
| macro_rules! | 小范围（≤30 行、不导出） | 测试矩阵、批量 impl | — |
| build.rs 代码生成 | 不用；要生成就用 xtask 生成 + 提交 + 一致性测试 | 系数表等 | 构建期不可见、拖慢编译 |
| cbindgen | 采用 | `vdn-shm` → C 头（提交进仓） | 先例 `resources/DeepFilterNet/cbindgen.toml`【源】 |
| 从 ONNX 元数据生成模型接口 | 不生成；适配器手写形状常量，**加载时**对照 `session.inputs()/outputs()` 校验 | `vdn-models` | p1 `gtcrn.rs` 已这样做 |
| 借 Zig comptime | 借：const generics 定长帧 + 编译期断言 | dsp | — |
| 借 Unison 内容寻址 | 借：模型、参数、eval build 用 sha256 当 ID | 模型清单、eval 报告 | 与 CLAUDE.md 第五节 build id 规则同向 |
| 借 ArchUnit / import-linter / dependency-cruiser | 借思想，由 cargo-deny + check-deps + tidy 落地 | — | — |
| Lisp / Elixir 式宏 DSL | 不用 | — | 可读性 |

---

## 7. 质量闸门与 CI 分层

层：**L0** 提交前本机（秒级）· **L1** 每个 PR · **L2** 合到 main 或每晚（含 nightly / 慢工具）· **L3** 每周 · **L4** 本机 Mac 手动（设备、长跑、人耳）。完整 21 行表见 `notes/r6-quality.md` §3，下表已按 r7 更正。

| # | 闸门 | 工具 | 层 | 失败意味着 |
|---|---|---|---|---|
| G1 | 格式 | `cargo fmt --all --check` | L0 L1 | 风格；自动修 |
| G2 | lint + 实时禁用 API | `cargo clippy --workspace --all-targets -- -D warnings`（§5.1 §5.2 随之生效） | L0 L1 | 实时路径出现分配 / 锁 / IO / panic 写法 → 拒绝合并 |
| G3 | 依赖方向与许可 | `cargo deny check`（licenses / advisories / bans / sources）+ `cargo xtask check-deps` | L1 | 模块边界被打破，或许可不能按 GPL-3.0 分发 |
| G4 | tidy | `cargo test -p xtask`（forbid 头、`// NRT:` 注释、`.gitignore` 含 `/target/`） | L0 L1 | 规约被绕开 |
| G5 | 单元 + 集成测试 | `cargo nextest run`（含 `assert_no_alloc` 包裹的 process 测试、故障注入） | L1 | 功能回归 / 实时路径分配 |
| G6 | 分块不变性 | proptest：随机切块经重分块器 vs 整段，输出逐样本一致 | L1（少量 case）L3（大量） | eval 与实时走的不是同一条路 |
| G7 | Python 对拍 | golden 向量，`Rust ort` vs exp003 Python，容差比较（不要求逐位：FTZ、KleidiAI 路径会让结果随 CPU 特性变化【测】） | L1（短片段）+ 升级 ort / 模型时强制 | 听到的不是那个模型 |
| G8 | 实时阻塞调用 | RTSan（`rtsan-standalone`，`RTSAN_ENABLE=1 cargo test`，默认去重模式 + `symbolize=false`，回放要短——逐次模式 100 帧日志超过 400 MB【测】） | L2，**macOS arm64** | A 档：出现任何拦截；B 档：出现豁免文件以外的栈 |
| G9 | UB / 并发 | Miri（`vdn-dsp`、`vdn-shm`）；loom（`vdn-shm` 的 SPSC 与心跳） | L2 | 自写无锁协议有竞争 |
| G10 | 耗时 | bench（divan 或自写直方图）每 hop p50 / p99 / p99.9 / max；计数分配器每 hop 分配数 | L4（CI 虚拟机数值不可信） | 实时余量被吃掉 |
| G11 | 端到端延迟 | 脉冲法（P3 起） | L4 | 超 D32 的 45 ms |
| G12 | 评测集回归 | `vdn-eval` 固定评测集，与基线比 | L3 + 改模型 / DSP 时 | 客观指标退化（指标过 ≠ 人耳过，D33） |
| G13 | 长跑 | BlackHole / 真设备 ≥ 1 h：xrun、漂移、内存曲线 | L4 | 现场才会遇到的问题 |
| G14 | 人耳 | 盲听页（CLAUDE.md 第五节第 1 条：build id、揭盲表入库） | 里程碑 | 验证红线，不交给机器 |

**许可**（G3 的 `deny.toml` `allow` 列表）：`GPL-3.0-or-later`（本项目）、`Apache-2.0`、`MIT`、`BSD-2-Clause`、`BSD-3-Clause`、`ISC`、`Zlib`、`BSL-1.0`、`Unicode-3.0`、`MPL-2.0`（audio_thread_priority、triple_buffer）。FSF 兼容列表上查不到的（如 `BSD-1-Clause`、`Apache-2.0 WITH LLVM-exception`）不进全局 allow，用 per-crate `exceptions` 逐个放行；MPL-2.0 的 Exhibit B（「不兼容次级许可」声明）cargo-deny 查不出，CI 另 grep 依赖源码。本节是 cargo-deny 白名单的**唯一正本**，rusteco 009 的草稿（`answers-p1.md` §5、`cards/tooling.md`）以此为准。本项目写 `GPL-3.0-or-later` 还是 `-only` 见 rusteco 009 待人拍板 2（p1 现为 or-later）。Apache-2.0 与 GPLv3 兼容、与 GPLv2 不兼容，所以本项目必须是 GPL-**3**【官】https://www.gnu.org/licenses/license-list.en.html 。cargo-deny 看不到 ort 下载的 ONNX Runtime 二进制和 `.onnx` 权重——这两类另列资产清单（来源 · 哈希 · 许可）。

**CI 费用**：私有仓库的 macOS 分钟数约是 Linux 的 10 倍【官】GitHub Actions 计费文档；核心 crate 不依赖平台，G1–G7、G9 都放 Linux，macOS job 只跑 clippy（`aarch64-apple-darwin`）、`vdn-io-macos` 测试与 G8，并用 `paths:` 过滤。

**依赖版本**：cpal 钉 `=0.18.2`（本机克隆是未发布的 0.19 master，回调类型名不同【官】crates.io / docs.rs）；ort 只有 rc，钉 `=2.0.0-rc.15`；`download-binaries` 下载的 ORT 缓存在 `~/Library/Caches/ort.pyke.io/`，CI 要缓存该目录【测】`notes/r8-critic.md` §3.3。

---

## 8. 还没核实、写代码前要先跑的

| # | 待核 | 为什么挡路 | 怎么核 |
|---|---|---|---|
| E-lat | 端到端延迟账 | 默认参数估约 56 ms，**必须调参**才进 45 ms【推】（R13） | P3 脉冲法逐项测：重采样型号与子块、IO 缓冲、环余量；调完仍超再拍板砍哪一级 ⟨待拍板 6⟩。另一条省法「推理进程直接写 HAL 读的环、缺数据改读原声环」可少一跳环，属 003 管线的「二期共享内存」决定，按 003 的触发条件（M0 实测）定，`vdn-shm` 的布局两种都能承载 |
| E-tract | tract 能不能跑 GTCRN、每帧分配多少 | B 档能不能更干净 | 用已发布的 `tract-onnx 0.23.9`；GTCRN simple 的 24 种算子 tract 都有注册【源】，但没跑过；要联网下载 crate + 名额 ⟨待拍板 7⟩ |
| E-ort | ort 关 arena / 自定义分配器后的 RTSan 计数 | 能否缩小 B 档豁免 | 改 `appendix-probes/ortprobe` |
| E-deny | cargo-deny 对工作区成员的 ban 实际效果 | R12 第 ① 层 | 装 cargo-deny 跑一次 |
| E-fe | FastEnhancer-M 的 hop、状态形状 | 第二个 `Model` 实现 | rusteco 009 已定用官方 16 kHz spec2spec 导出 `onnx-dns-v1.0.0/fastenhancer_m.spec.onnx`（可与 GTCRN 共用 dsp 的 STFT）；形状以加载期校验为准 |
| E-load | 负载下、能效核上的 p99.9 | 空载数据偏乐观 | P3 时重测 |
| E-rate | 音色链跑在 16 kHz 还是 48 kHz | 延迟账与 `vdn-dsp` 参数 | 默认：模型采样率（16 kHz）上跑，限幅器放 48 kHz 末端【推】；带宽扩展上了再定 |
