# Rust 架构规约（vdn workspace）

> **规约 = 产出该长什么样**：只写规则和指针。理由在 `docs/research/008_rust-architecture/adr/`，实测数字在 `…/appendix-probes/README.md`，待办与待拍板在 `…/decision.md`，**依赖选型（用哪个 crate、哪个版本）以 `docs/research/009_rust-ecosystem/decision.md` 为准**，版本号以根 `Cargo.toml` 为准。
> 起草：手下 rustarch，2026-10-30，经三视角自审修订。**AI 方案，待人过目**；「⟨拍板 N⟩」= `decision.md` 待人拍板第 N 题。
> 证据等级：【官】官方文档 ·【源】源码可证 ·【测】本机实测 ·【推】推断。
> 本规约从 p1 骨架（`src/{dsp,engine,models,cli}`）长出来，不推倒重来。

---

## 0. 白话一页

> 术语见文末「词表」。

1. **怎么拆模块**：代码分成一个个「零件包」（Rust 叫 crate），编译器不让一个包偷用它没声明的包。核心包只管声音处理，不碰麦克风、不碰 AI 推理库、不碰文件格式，所以离线评测和实时运行必然用的是同一份处理代码。
2. **「用语法树做模块化」怎么用**：**用语法树守规矩，不用语法树造代码。** 用工具在代码的语法树和包的依赖图上自动检查「谁不许用谁」「实时代码里不许出现什么」，违反了 CI 就亮红灯；用宏自动生成代码先不做，等手抄第三次出错再上。
3. **实时线程分三档**：
   - A 档：麦克风 / 扬声器回调，一点都不许分配内存、加锁。
   - B 档：降噪线程。我们自己写的代码不许分配，AI 推理库（ONNX Runtime）内部会分配，这一点改不掉。所以给它定截止时间，来不及就送原声。
   - C 档：其他线程，不限。
4. **模型怎么换**：每个模型实现同一个接口（一段声音进、一段声音出），GTCRN 和 FastEnhancer-M 可以互换，引擎不用改。
5. **每条实时规矩都有一种机器检查**，不靠人看。

---

## 1. 目录与命名

| 东西 | 规则 |
|---|---|
| workspace 根 | 虚拟 manifest；`[workspace.package]` 统一 edition / license / `publish = false`；`[workspace.dependencies]` 统一版本；`[workspace.lints]` 统一 lint（p1 已是这样） |
| crate 目录 | `src/<短名>/`，扁平一层，不嵌套（`.42cog/cog.md` 实体表） |
| package 名 | `vdn-<短名>`，lib 路径 `vdn_<短名>`；目录名要不要也带前缀 ⟨拍板 2⟩ |
| crate 登记 | 建一个 crate，就在 `.42cog/cog.md` 实体表加一行（「是什么、谁产生」以那里为准）；本规约 §2 只管层级与依赖规则 |
| crate 的档位与禁依赖 | 写进各 crate `Cargo.toml` 的 `[package.metadata.vdn]`（§5.4），这是机读正本 |
| bin 名 | 开发期 `vdn`；两层进程 `vdn-capture`（送音）、`vdn-denoised`（推理）⟨拍板 4⟩ |
| 推理后端 crate（拆出之后） | `vdn-backend-<名>` |
| 工具 | `xtask/`，`.cargo/config.toml` 设 `[alias] xtask = "run -p xtask --"`；xtask 不依赖任何 vdn crate |
| 模型权重 | 仓外；仓里只留来源 · 许可 · sha256（CLAUDE.md 铁律 8） |
| 构建产物 | `/target/` 在第一次构建前就在 `.gitignore` 里（2026-10-30 p1 分支误提交 target/ 后由大总管定） |

---

## 2. crate 层级与依赖方向

### 2.1 层级（目标形态）

| 层 | crate | 档 | 一句话 |
|---|---|---|---|
| 核心·词汇 | `vdn-types` | A/B | 共享的 trait 与小类型：`Model`、`ModelError`、`Status`、`BypassReason`、`SpeakerVerdict` |
| 核心 | `vdn-dsp` | B | 纯信号处理积木：STFT、延迟线、包络、淡化、biquad、限幅、重分块器、固定比例重采样包装、FTZ 守卫 |
| 核心 | `vdn-models` | B | 每个模型一个文件 + `AnyModel` |
| 核心 | `vdn-engine` | B | `Engine`（模型采样率）与 `Pipeline`（IO 采样率，§3.3）；eval 与实时进程唯一的处理入口 |
| 核心 | `vdn-speaker` | C | 声纹、自动注册、门控判决 |
| 适配·推理 | `vdn-backend-ort` / `vdn-backend-tract` | B | 推理后端，兄弟 crate |
| 适配·协议 | `vdn-shm` | A/B | 跨进程共享内存布局与自写 SPSC，核心部分 `no_std` |
| 适配·平台 | `vdn-rt-thread` | — | 实时优先级、Audio Workgroup、启动自检 |
| 适配·平台 | `vdn-io-macos` | A | 采集与输出、设备监听、进程级原声旁路、`Slip` 漂移补偿 |
| 边界 | `vdn-control` | C | 配置、控制消息；serde 只在这里和 bin 里 |
| 边界·离线 | `vdn-eval` | C | 读 WAV 逐帧喂 `Pipeline`；指标、报告 |
| 应用 | `vdn` · `vdn-capture` · `vdn-denoised` | — | bin |

HAL 虚拟麦克风驱动：003 定为 C 插件，不进 workspace；009 建议改用 Rust cdylib。两者谁对由 003 / 009 的拍板定；若改用 Rust，它进 workspace 成为 A 档 crate `vdn-hal`，只依赖 `vdn-shm`。驱动目录 ⟨拍板 5⟩。

### 2.2 依赖方向

```
                vdn-types
        ┌───────┬───┴────┬──────────────┐
        ▼       │        ▼              ▼
     vdn-dsp    │    vdn-speaker   vdn-backend-ort / -tract
        │       │                        │
        ├───────┼──────────┐             │
        ▼       ▼          │             │
     vdn-models ◄──────────┼─────────────┘   （models 用后端）
        │                  │
        ▼                  ▼
            vdn-engine  ◄── 只依赖 types、dsp；不认识具体模型与后端
   ════════════════════ 核心 / 适配分界 ════════════════════
   vdn-eval     = engine + models + 后端 + hound + serde
   vdn          = engine + models + 后端 + hound + clap
   vdn-denoised = engine + models + 后端 + speaker + shm + rt-thread + control
   vdn-capture  = io-macos + shm + rt-thread + control      （不含 engine / models / 后端）
   vdn-io-macos = 平台库；vdn-shm = 无依赖（posix feature 用 libc）
```

规则（机器检查见 §5 R12）：
1. 分界线以上（核心）不依赖分界线以下的任何 crate，也不依赖平台库、推理库、serde、hound
2. `vdn-engine` 不依赖 `vdn-models` 与后端；把具体模型交给引擎的是 bin（`Engine<AnyModel>`）
3. `vdn-capture` 的依赖闭包里没有 `vdn-engine`、`vdn-models`、推理库（D03 崩溃隔离）
4. **过渡期例外**：P2 拆出 `vdn-types` 之前，trait 住在 `vdn-engine::model`，所以 `vdn-models → vdn-engine` 这条边允许（p1 现状）；P1 的 `vdn-models` 直接依赖 ort，拆后端后改依赖后端

### 2.3 什么时候拆（可观察的事件）

阶段对照：P1 / P2 = 路线图阶段 C（离线）· P3 = M0（实时直通）· P4 = M1–M2（实时降噪 + 两层进程）· 自写 HAL 驱动 = M3。

| 事件 | 拆 / 加 | 拆完怎么验 |
|---|---|---|
| `vdn-speaker` 第一次要把判决交给引擎 | 把 `engine::model` 搬成 `vdn-types`；加 `vdn-speaker` | check-deps：speaker → engine 这条边不存在 |
| 第一次要算指标、写报告 JSON | 加 `vdn-eval`，CLI 的评测子命令改调它 | check-deps：engine 闭包里没有 hound / serde |
| 第一次要接第二个推理后端（tract） | 拆 `vdn-backend-ort` / `vdn-backend-tract`，models 只认后端 trait；**不用互斥 feature** | cargo-deny：ort 的直接依赖者只有 `vdn-backend-ort` |
| P3 开工（第一次碰真设备） | 加 `vdn-io-macos`、`vdn-rt-thread`、bin `vdn-capture` | check-deps：核心闭包里没有 cpal / coreaudio |
| P4 开工（两层进程） | 加 `vdn-shm`、`vdn-control`、bin `vdn-denoised` | check-deps：`vdn-capture` 闭包里没有 engine / 推理库 |

判据（满足任一条才拆，否则留作 module）：要机器守的依赖边界 · 实时档不同 · 同一接口的可替换实现 · 产物或使用方不同（bin、`no_std`、给 C 出头文件）· 重依赖隔离 · unsafe 集中。代码量不是判据；不建 `utils` / `common`；泛型不放在 crate 边界上。理由见 ADR 001。

---

## 3. 核心 trait 与数据流

### 3.1 模型接口

p1 的 `Model` 保持不变（`src/engine/src/model.rs`，P2 搬进 `vdn-types`）：`Send`；`name / sample_rate / hop / latency_samples / reset / process`；一个 hop 进、一个 hop 出的波形接口；`ModelError` 是不带 `String` 的 `Copy` 枚举。规则：
- **构造在非实时线程**（打开会话、预分配、预热），实时线程只调 `process` 与 `reset`
- 频域模型在适配器里自己做 STFT；「共用一次 STFT」（设计稿 §11）以后用**扩展 trait** 补，不改 `Model`
- 加载时用 `session.inputs()/outputs()` 校验名字与形状，不从 ONNX 元数据生成代码（p1 `gtcrn.rs` 已这样做）

### 3.2 分发

- `Engine<M: Model>` 静态分发；运行期选模型用 `vdn-models::AnyModel`（p1 已实现，含小宏 `each!`）
- 不用 `Box<dyn Model>`：变体是封闭集合，值不在堆上，没有在实时线程释放 trait 对象的问题（ADR 003）

### 3.3 Engine 与 Pipeline：两个采样率，两层

```rust
// vdn-types
#[derive(Clone, Copy)]
pub struct FrameCtx { pub speaker: SpeakerVerdict }   // 时间以 Pipeline 的样本计数为准，Engine 不另计

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BypassReason { Backend, NonFinite, Abnormal, Deadline, BadLen }

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ok,
    Bypassed(BypassReason), // 本 hop 整段原声
    Holding,                // 故障后多送原声的保持期
    Crossfading,            // 原声 → 模型的淡化期
}

// vdn-engine：模型采样率（GTCRN 16 kHz），一次一个模型 hop
impl<M: Model> Engine<M> {
    pub fn process(&mut self, input: &[f32], output: &mut [f32], ctx: FrameCtx) -> Status;
}

// vdn-engine：IO 采样率（48 kHz），任意长度的块
impl<M: Model> Pipeline<M> {
    /// 重分块 → 下采样 → Engine → 后处理 Stage → 上采样 → 按 Status 与 48 kHz 原声混合
    pub fn process(&mut self, input: &[f32], output: &mut [f32], ctx: FrameCtx);
    /// 整条链的延迟，单位：IO 采样率的样本
    pub fn latency_samples(&self) -> usize;
}

pub trait Stage: Send {          // 门控、衰减下限、音色链节点，跑在模型采样率
    fn reset(&mut self);
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
}
```

规则：
- **送原声用 48 kHz 原声**（Pipeline 的延迟线），不用 16 kHz 的，否则原声缺 8 kHz 以上
- Engine 仍做模型级故障判定与 16 kHz 混合（p1 现状，离线 P1 直接用）；接入 Pipeline 后，Engine 只回报 `Status`，Pipeline 用 48 kHz 原声完成切换与淡化
- 所有延迟统一按 **IO 采样率样本**记账：重分块器（最多一个 IO hop）+ 两次重采样 + 模型；**P2 前所有 `Stage` 延迟必须为 0**，有延迟的 Stage 要等引入「每级 dry 对齐」再加
- 后处理链是固定字段的 struct，按设计稿 §3 的固定顺序静态调用
- `FrameCtx` 按值传
- eval 喂 Pipeline（48 kHz WAV）；P1 离线 CLI 直接喂 Engine（16 kHz），二者共用 Engine

### 3.4 跨线程

| 数据 | 方向 | 用什么 | 内存序 |
|---|---|---|---|
| 连续参数 | 控制 → 实时 | `AtomicU32` 存 `f32::to_bits`，放在启动前建好的共享结构里；实时侧不 clone、不 drop 它的 `Arc` | Relaxed，实时侧进平滑器 |
| 必须同时生效的一组参数 | 控制 → 实时 | rtrb 送一个 `Copy` 系数包 | rtrb 内部 |
| 离散命令 | 控制 → 实时 | rtrb，`Cmd` 是小 `Copy` 枚举 | rtrb 内部 |
| 声纹判决、状态快照 | 声纹线程 → 实时 | triple_buffer（009 首选） | 库内部（AcqRel） |
| 进程间音频与心跳 | 送音 ↔ 推理 | `vdn-shm` 自写 `#[repr(C)]` 全原子布局 | Release / Acquire，loom 验证 |

禁止：`Mutex` / `RwLock` / `try_lock`、MPMC 队列；arc-swap 只在非实时线程之间用。

### 3.5 换模型（GTCRN ↔ FastEnhancer-M）

两个模型的 hop 和延迟不同（GTCRN hop 256、FE-M hop 160，各自的 `latency_samples()` 以代码为准），所以：
- **不在 Engine 内部换**。外层同时持有两套 Pipeline，都补齐到**较大的那个延迟**，交叉淡化后丢掉旧的（p1 `model.rs` 注释已定此方向）
- `L_total`（送原声用的延迟）在**启动时**按「所有启用模型里的最大延迟」定死，换模型不改它；代价是延迟较小的模型也多付一点延迟
- 新 Pipeline 在非实时线程建好、预热、reset 后，经 rtrb 送进实时线程；旧的经回收 rtrb 送回非实时线程 drop；回收队列满就把它留在本地槽位，下一块再送
- 双跑淡化期间 CPU 翻倍；不够时停流重建

### 3.6 送原声：两层

| 层 | 谁 | 挡什么 |
|---|---|---|
| 模型级 | `Pipeline`（P1 期间是 `Engine`） | 单帧报错、NaN、幅度异常、超截止时间、换模型 |
| 进程级 | 送音进程输出回调读原声环 | 推理进程卡死、崩溃、心跳停 |

两层用同一个 `L_total`（推理进程写进 `vdn-shm` 头），不各自对齐（ADR 005）。

---

## 4. 实时三档

| 档 | 线程 | 承诺 | 验收 |
|---|---|---|---|
| **A 硬实时** | 送音进程的 IO 回调；HAL 驱动 IO | 零分配、零锁、零阻塞系统调用、不 panic，第三方代码也算。**唯一例外**：回调里一次非阻塞的跨进程唤醒（`os_sync_wake`，009 §一.1） | RTSan 遇到第一处违规就停（halt）；例外只按函数名放行 |
| **B 软实时** | 推理进程降噪线程 | **我们的 Rust 代码零分配、零锁**；ONNX Runtime 内部的分配与 arena 锁允许，按函数名或命名空间登记；适配器为 ort 编排输入输出的 Rust 侧分配，按模型登记上限（GTCRN 0）；每帧截止时间，超时 / 出错走模型级送原声 | RTSan 不停（`halt_on_error=false`）、符号化，去重后的唯一栈和豁免表比对，出现新栈即红；耗时看 p99.9 与 max |
| **C 非实时** | 声纹、控制、日志、eval、构造期 | 不限，但不许阻塞 A/B 等的东西 | 常规测试 |

依据与实测数字：ADR 002、`appendix-probes/README.md` §1。**口径**：p1 测到的「每帧 0 次分配」统计的是 Rust 全局分配器；ONNX Runtime 是 C++，直接调 libc，Rust 全局分配器看不到。所以两者同时成立，不矛盾。

**cpal 的位置**：cpal 与 coreaudio-rs 在回调里会调 `mach_timebase_info`、在帧数变化时 `Vec::resize`（009 §一.1 引的源码行），过不了 A 档 RTSan。所以 cpal 只用于 P3 / M0 直通；A 档零容忍从 M1 自写 IOProc 起生效（采集方案以 009 为准）。P3 期间 A 档按已知违规清单跑，不算通过。

**实时 crate 与实时入口**：`vdn-dsp`、`vdn-engine`、`vdn-models` / `vdn-backend-*`（B）；`vdn-shm`、`vdn-io-macos` 的回调模块、`vdn-hal`（A）。实时入口 = 会在 A/B 档线程上被调用的 `process` / `reset` / 回调函数，打 `#[nonblocking]`（rtsan-standalone，可选 feature）。

---

## 5. 实时规则与机器检查

### 5.1 检查栈（每一层各管什么）

| 层 | 工具 | 管什么 | 管不到什么 |
|---|---|---|---|
| 静态·写法 | clippy `disallowed-methods/types/macros`（根 `clippy.toml` **一份**）+ restriction lint | 本 crate 里实时代码的调用点 | 依赖 crate 内部、宏展开内部；漏列的 API（禁用清单是第一层，不是兜底） |
| 静态·依赖直接边 | cargo-deny `[bans]`，内部 crate 也列进 deny（`allow-workspace` 保持默认 false） | 谁能**直接**依赖 X | 传递依赖（wrappers 只查直接依赖者，`resources/cargo-deny/src/bans.rs:616-650`） |
| 静态·依赖闭包 | `cargo xtask check-deps` | 某 crate / bin 的**整个闭包**里不许出现 X | — |
| 静态·项目约定 | xtask tidy（普通 `#[test]`，文本 / 轻语法检查，照 rust-analyzer） | forbid 头、`// NRT:` 注释、`.gitignore`、哨兵、`std::fs` / `log` 文本检查 | 语义 |
| 运行期·Rust 分配 | 计数分配器（release 有效）；debug 测试里的「不许分配」断言（009：nice-assert-no-alloc 或自写约 60 行） | Rust 全局分配器上的分配 | C / C++ 库的分配 |
| 运行期·libc | rtsan-standalone `#[nonblocking]`，`RTSAN_ENABLE=1` 构建，经 `cargo test` / `cargo run` 跑回放 | malloc / free / 锁 / sleep 等，包括 C++ 内部 | 回放没跑到的路径 |

### 5.2 规则表

| # | 规则 | 机器检查 |
|---|---|---|
| R1 | 实时路径不分配、不释放 | 禁用清单（分配类）· 计数分配器每 hop 计数 = 0（按模型登记的上限除外，§4 B 档）· RTSan |
| R2 | 实时路径不加阻塞锁 | 禁用清单（`Mutex`、`RwLock`、`Condvar`、`mpsc`）· RTSan |
| R3 | 实时路径不做 IO、日志、sleep | 禁用清单（`println!` 等宏、`thread::sleep`）· tidy 文本检查实时模块里没有 `std::fs`、`log::` · RTSan |
| R4 | 实时路径不 panic | 实时 impl 块 deny `unwrap_used / expect_used / panic / indexing_slicing / unreachable / todo`，并禁 `assert!` / `assert_eq!` / `assert_ne!` 宏（只许 `debug_assert*`）· 故障注入测试 |
| R5 | 不用最坏情况无界的算法 | bench 看 p99.9 / max（§6 G10） |
| R6 | 跨线程只用 §3.4 的原语 | 禁用清单（类型类）· 自写无锁结构只许在 `vdn-shm`，跑 loom + Miri |
| R7 | 实时线程不 drop 堆对象 | RTSan 拦 `free` · 计数分配器计 dealloc · 回收队列满的单测 |
| R8 | 实时线程有 time-constraint 优先级并加入 Audio Workgroup | 启动自检读回线程策略、检查 `os_workgroup_join` 返回码；失败计数并上报，不静默 |
| R9 | FTZ 守卫只包住 DSP 段，**不包推理调用** | 单测：守卫内非正规数被冲成 0、守卫外不冲；`Model::process` 的推理调用在守卫外（009 §一.2：intra=1 时 FTZ 会作用到 ORT，DeepFilterNet #713） |
| R10 | unsafe 只在白名单 crate | workspace `unsafe_code = "deny"`；非白名单 crate 的 `lib.rs` 写 `#![forbid(unsafe_code)]`（workspace 设 forbid 再局部 allow 会报 E0453）· `undocumented_unsafe_blocks = "deny"` · tidy 查 forbid 头。白名单 = `metadata.vdn.unsafe = true` 的 crate |
| R11 | 实时路径出错 = 返回 `Status` + 送原声 | `#[must_use] Status` · 故障注入：超时 / NaN → 输出等于对齐原声（带淡化） |
| R12 | 依赖方向（§2.2） | cargo-deny 管直接边 · check-deps 管闭包；规则都从 `metadata.vdn.forbid` 读 |
| R13 | 延迟有账 | 单测断言 `Pipeline::latency_samples()` 不超预算；P3 脉冲法实测（延迟账见 ADR 005） |
| R14 | 构建产物不进库 | tidy 查 `.gitignore` 含 `/target/` |

### 5.3 clippy 配置的写法

- 根目录**只放一份** `clippy.toml`；crate 目录下不放（会整份替换上层、不合并【测】`appendix-probes/README.md` §5）。建这个文件之后，**清单以文件为准**，本规约只写原则
- `[workspace.lints.clippy]` 把 `disallowed_methods / disallowed_types / disallowed_macros` 设为 `allow`；在**实时 impl 块或实时模块**上升为 deny（tympan-aspl 的做法：`resources/tympan-aspl/src/realtime/mod.rs:45`）。构造函数放另一个 `impl` 块，不用逐个 `#[allow]`
- panic 类 lint 放 crate 根，但不作用于测试：`#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]`；根 `clippy.toml` 设 `allow-unwrap-in-tests` 等
- 非实时整模块（离线驱动、故障注入器）在模块头 `#![allow(...)] // NRT: <理由>`
- 例外写法：`#[allow(clippy::disallowed_…)] // NRT: <理由>`，注释在同一行行尾或紧邻的上一行；tidy 用正则检查理由非空，并统计 allow 总数，总数涨了在 PR 里显示
- **初版禁用清单的类别**：分配（`Vec::push / with_capacity / reserve / extend_from_slice / resize / insert`、`Box::new`、`Arc::new`、`<[T]>::to_vec`、`Iterator::collect`、`vec!`、`format!`）· 锁与阻塞（`Mutex`、`RwLock`、`Condvar`、`mpsc`、`thread::sleep`、`thread::spawn`）· IO 与日志（`println! / eprintln! / print! / eprint! / dbg!`）· panic 宏（`assert! / assert_eq! / assert_ne!`）。**已实测命中的只有**：`Vec::push`、`Vec::reserve`、`Box::new`、`std::sync::Mutex`、`println!`、`format!`、`vec!`（`appendix-probes/README.md` §5）；其余每加一条，先在哨兵 crate 里写一个必须报红的负例
- 注意：`std::io::stdout` 拦不住 `println!`（展开调的是 `std::io::_print`）【测】；trait 方法（如 `Clone::clone`）会误伤 `Copy` 类型，不列，交给 RTSan 与计数分配器

### 5.4 机读正本：`[package.metadata.vdn]`

每个 crate 的 `Cargo.toml`：

```toml
[package.metadata.vdn]
tier = "B"                         # A | B | C | tool
unsafe = false                     # true = 进 unsafe 白名单
forbid = ["ort", "hound", "serde", "cpal"]   # 闭包里不许出现的 crate
```

- `cargo xtask check-deps`：用 `cargo metadata --format-version 1` 读 `resolve.nodes[].deps[]`，只取普通依赖（`dep_kinds[].kind == null`），按包 BFS 求闭包，与 `forbid` 求交，非空以 1 退出。resolve 是 feature 合并后的图，只会多报，不会漏报
- tidy 校验：`deny.toml` 的 bans 与各 crate 的 `forbid` 一致；`unsafe = false` 的 crate 有 forbid 头；`tier` 为 A/B 的 crate 有实时 lint
- 这样拆 crate 时只改各自的 `Cargo.toml`，不用同步三处

### 5.5 RTSan 的用法

- 只在 CI 的一个 macOS job 和本机跑；可选 feature，平时构建不联网
- A 档：默认 `halt_on_error`；例外（跨进程唤醒）用 `function-name-matches` 放行
- B 档：`halt_on_error=false`、**必须符号化**（豁免靠匹配函数名，未符号化的栈匹配不上【官】Clang RealtimeSanitizer 文档）。ONNX Runtime 是静态链接的，栈里没有库名，所以按命名空间写：`call-stack-contains:onnxruntime::`、`call-stack-contains:Mlas`。豁免文件进仓，改动要在 PR 里说明，**升级 ort 必须重跑**
- 用默认去重模式、短回放（逐次模式日志爆量【测】`appendix-probes/README.md` §1）

### 5.6 panic 策略

实时代码按 R4 写成不 panic；兜底是进程隔离（§3.6 进程级）。release `panic = "abort"` ⟨拍板 3⟩；`extern "C"` 里 panic 会直接 abort（Rust 1.81 起）。

---

## 6. 「AST 做模块化」：用什么、不用什么

理由与 27 项逐条评估见 ADR 004、`notes/r1-ast.md` §6。

| 用 | 不用（或暂缓，写明触发条件） |
|---|---|
| crate / module / trait / 可见性做结构拆分（§2） | 自写 derive 宏：参数超过约 10 个，或手抄参数表第三次出错时再上 |
| clippy 禁用清单 + restriction lint（§5.3） | build.rs 生成代码：要生成就用 xtask 生成、提交、加一致性测试 |
| cargo-deny bans + xtask check-deps（§5.4） | dylint 自定义 lint：同一类违规漏网第三次再上 |
| xtask tidy（文本 / 轻语法，随 `cargo test` 跑） | ast-grep 规则：tidy 文本检查漏网时上，规则先配正例负例 |
| rtsan-standalone 的 `#[nonblocking]` 标实时入口（不自造标记） | 从 ONNX 元数据生成模型接口：手写形状 + 加载期校验 |
| cbindgen：`vdn-shm` → C 头（若驱动是 C） | Lisp / Elixir 式宏 DSL |
| 小范围 `macro_rules!`（≤30 行、不导出，如 p1 的 `each!`） | cargo-semver-checks：不对外发布前不用 |
| 借思想：const generics 定长帧（Zig comptime）；模型 / 参数 / eval build 用 sha256 当 ID（Unison）；架构规则即测试（ArchUnit / import-linter） | cargo-modules / cargo-depgraph 只画图，不做门禁 |

---

## 7. 质量闸门

层：**L0** 本机提交前 · **L1** 每个 PR · **L2** 合到 main 或每晚 · **L3** 每周 · **L4** 本机 Mac 手动。

| # | 闸门 | 工具 | 层 | 失败意味着 |
|---|---|---|---|---|
| G1 | 格式 | `cargo fmt --all --check` | L0 L1 | 风格；自动修 |
| G2 | lint 与实时禁用 API | `cargo clippy --workspace --all-targets -- -D warnings` | L0 L1 | 实时路径出现分配 / 锁 / IO / panic 写法 |
| G3 | 依赖方向与许可 | `cargo deny check` + `cargo xtask check-deps` | L1 | 模块边界被打破，或不能按 GPL-3.0 分发 |
| G4 | tidy | `cargo test -p xtask` | L0 L1 | 规约被绕开 |
| G5 | 单元 + 集成 | `cargo nextest run`（含计数分配器断言、故障注入） | L1 | 功能回归 / 实时路径分配 |
| G6 | 分块不变性 | proptest：同一段音频随机切块喂 Pipeline，与整段一次喂的结果逐样本一致 | L1 少量 / L3 大量 | eval 与实时不是同一条路 |
| G7 | Python 对拍 | golden 向量，容差比较（数值可能随 CPU 特性变化【推】） | L1 短片段 + 升级 ort / 模型时 | 听到的不是那个模型 |
| G8 | 实时阻塞调用 | RTSan（§5.5） | L2，macOS arm64 | A 档有任何违规；B 档出现豁免以外的栈 |
| G9 | UB 与并发 | Miri（`vdn-dsp`、`vdn-shm`）；loom（`vdn-shm`） | L2 | 自写无锁协议有竞争 |
| G10 | 耗时与分配计数 | bench 每 hop p50 / p99 / p99.9 / max + 分配数 | L4（CI 虚拟机数值不可信） | 实时余量被吃掉 |
| G11 | 端到端延迟 | 脉冲法（P3 起） | L4 | 超 D32 |
| G12 | 评测集回归 | `vdn-eval` 与基线比 | L3 + 改模型 / DSP 时 | 客观指标退化（指标过 ≠ 人耳过，D33） |
| G13 | 长跑 | 真设备 ≥ 1 h：xrun、漂移、内存 | L4 | 现场才遇到的问题 |
| G14 | 人耳 | 盲听页（CLAUDE.md 第五节第 1 条） | 里程碑 | 验证红线 |

- **许可**：`deny.toml` 的 allow 列表以文件为准。原则：只放 GNU 列表上与 GPLv3 兼容的许可（Apache-2.0 只与 GPLv3 兼容、不与 GPLv2 兼容【官】https://www.gnu.org/licenses/license-list.en.html）；列表上查不到的（如 BSD-1-Clause、LLVM-exception）用 per-crate `exceptions` 逐个放行；MPL-2.0 的 Exhibit B cargo-deny 查不出，CI 另 grep；ORT 预编译库与 `.onnx` 权重不在 Cargo 图里，另列资产清单（来源 · 哈希 · 许可）
- **CI 费用**：核心 crate 不依赖平台，G1–G7、G9 放 Linux；macOS job 只跑 clippy（`aarch64-apple-darwin`）、io-macos 测试与 G8，并用 `paths:` 过滤（私有仓 macOS 分钟数约是 Linux 的 10 倍【官】GitHub Actions 计费文档）

---

## 词表

| 词 | 意思 |
|---|---|
| crate | Rust 的「零件包」，编译和依赖的最小单位 |
| module | crate 内部的分区 |
| trait | 接口：一组「必须提供的函数」，不同模型各自实现 |
| hop | 每次处理的一小段声音的样本数（GTCRN 是 256 个样本 = 16 ms） |
| `Send` | 「可以交给别的线程」的标记 |
| 静态分发 / 动态分发 | 编译时就定好调哪个实现 / 运行时查表决定 |
| enum | 「几选一」的类型，如 `AnyModel` = GTCRN 或 FastEnhancer |
| 分配（malloc）· 锁（mutex） | 向系统要内存 · 让线程排队等；实时线程上做这两件事可能卡住，造成爆音 |
| ONNX Runtime（ort） | 运行 AI 模型的推理库（C++ 写的，ort 是它的 Rust 包装） |
| AST（抽象语法树） | 编译器把代码解析成的树结构；检查工具在它上面找「不许出现的写法」 |
| clippy · cargo-deny · xtask · tidy | Rust 的代码检查器 · 依赖与许可检查器 · 项目自带的脚本工具 · 项目规矩的自动检查 |
| RTSan | 运行时检查：实时函数里一旦分配、加锁就报警 |
| CI | 每次提交自动跑的检查 |
| IO 回调 | 系统每隔几毫秒调一次、交给我们一块麦克风声音的函数 |
| 送原声（旁路） | 降噪出问题时，直接把没处理的声音送出去，宁可有噪音也不吃字 |
