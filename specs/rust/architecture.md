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
| 工具 | `src/xtask/`（package 名 `xtask`；放 `src/` 下以免新开顶层目录，铁律 10），**不设 `cargo xtask` 别名**：别名要新建顶层目录 `.cargo/`（铁律 10），为一个别名不值得，文档与 CI 一律写全 `cargo run -p xtask -- tidy` / `cargo run -p xtask -- check-deps`；xtask 不依赖任何 vdn crate |
| 模型权重 | 仓外；仓里只留来源 · 许可 · sha256（CLAUDE.md 铁律 8） |
| 构建产物 | `/target/` 在第一次构建前就在 `.gitignore` 里（2026-10-30 p1 分支误提交 target/ 后由大总管定） |

---

## 2. crate 层级与依赖方向

### 2.1 层级（目标形态）

| 层 | crate | 档 | 一句话 |
|---|---|---|---|
| 核心·词汇 | `vdn-types` | A/B | 共享的 trait 与小类型，零依赖：`Model`、`ModelError`、`Stage`、`FrameCtx`、`Status`、`BypassReason`、`SpeakerVerdict`；智能体部件的接口（§3.7）：感知输出、参数组、场景参数表、策略、我的配置、决策日志、延迟账、版本号 |
| 核心 | `vdn-dsp` | B | 纯信号处理积木：STFT、延迟线、包络、淡化、biquad、限幅、重分块器、固定比例重采样包装、FTZ 守卫 |
| 核心 | `vdn-models` | B | 每个模型一个文件 + `AnyModel` |
| 核心 | `vdn-engine` | B | `Engine`（模型采样率）与 `Pipeline`（IO 采样率，§3.3）；eval 与实时进程唯一的处理入口 |
| 核心 | `vdn-speaker` | C | 声纹、自动注册、门控判决 |
| 适配·推理 | `vdn-backend-ort` / `vdn-backend-tract` | B | 推理后端，兄弟 crate |
| 适配·协议 | `vdn-shm` | A/B | 跨进程共享内存布局与自写 SPSC，核心部分 `no_std` |
| 适配·平台 | `vdn-rt-thread` | — | 实时优先级、Audio Workgroup、启动自检 |
| 适配·平台 | `vdn-io-macos` | A | 采集与输出、设备监听、进程级原声旁路、`Slip` 漂移补偿 |
| 边界 | `vdn-control` | C | 配置、控制消息；serde 只在这里和 bin 里 |
| 边界·离线 | `vdn-eval` | C | 读 WAV / 合成场景逐 hop 喂 `Engine`（`Pipeline` 落地后改喂它，§7.1）；verifier、任务集、定标、报告 |
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
   vdn-capture  = io-macos + shm + rt-thread + control + dsp + hal（库）；hound · clap 只在 bin feature （不含 engine / models / 后端）
   vdn-io-macos = 平台库 + types（只取采样率常量）；vdn-control = types（同上）；vdn-shm = 无依赖（posix feature 用 libc）
```

规则（机器检查见 §5 R12）：
1. 分界线以上（核心）不依赖分界线以下的任何 crate，也不依赖平台库、推理库、serde、hound
2. `vdn-engine` 不依赖 `vdn-models` 与后端；把具体模型交给引擎的是 bin（`Engine<AnyModel>`）
3. `vdn-capture` 的依赖闭包里没有 `vdn-engine`、`vdn-models`、推理库（D03 崩溃隔离）
4. `vdn-types` 已拆出（2026-10-31，align；比 §2.3 的触发事件提前，因为智能体部件的接口要在 speaker / scene / engine 之间共享）：`vdn-models` 只依赖 `vdn-types` + `vdn-dsp` + ort，**不再依赖 `vdn-engine`**（engine 只在 models 的 dev-dependencies 里供测试用）；`vdn_engine::model` 留一个转出模块兼容旧路径。`vdn-backend-ort` 已拆出（2026-11-02，rs-core）：`vdn-models` 只依赖它，不再直接依赖 ort；`vdn-eval`、`vdn-speaker`（onnx feature）、`vdn-speaker-lab` 仍直接依赖 ort，归后续批次
5. 采样率只在 `vdn-types::rate` 写一处（2026-11-02，issue #62）：`vdn-io-macos`、`vdn-control` 为此依赖 `vdn-types` 并转出 `IO_RATE`；不依赖 `vdn-types` 的底层 crate（shm · rt-thread · hal · dsp · backend-ort）非测试代码不写采样率字面量，`xtask tidy` R-RATE 机器检查

### 2.3 什么时候拆（可观察的事件）

阶段对照：P1 / P2 = 路线图阶段 C（离线）· P3 = M0（实时直通）· P4 = M1–M2（实时降噪 + 两层进程）· 自写 HAL 驱动 = M3。

| 事件 | 拆 / 加 | 拆完怎么验 |
|---|---|---|
| `vdn-speaker` 第一次要把判决交给引擎 | 加 `vdn-speaker`（`vdn-types` 已于 2026-10-31 先拆出） | check-deps：speaker → engine 这条边不存在 |
| 第一次要算指标、写报告 JSON | 加 `vdn-eval`，CLI 的评测子命令改调它 | check-deps：engine 闭包里没有 hound / serde |
| 第一次要接第二个推理后端（tract） | 拆 `vdn-backend-ort` / `vdn-backend-tract`，models 只认后端 trait；**不用互斥 feature** | cargo-deny：ort 的直接依赖者只有 `vdn-backend-ort` |
| P3 开工（第一次碰真设备） | 加 `vdn-io-macos`、`vdn-rt-thread`、bin `vdn-capture` | check-deps：核心闭包里没有 cpal / coreaudio |
| P4 开工（两层进程） | 加 `vdn-shm`、`vdn-control`、bin `vdn-denoised` | check-deps：`vdn-capture` 闭包里没有 engine / 推理库 |

判据（满足任一条才拆，否则留作 module）：要机器守的依赖边界 · 实时档不同 · 同一接口的可替换实现 · 产物或使用方不同（bin、`no_std`、给 C 出头文件）· 重依赖隔离 · unsafe 集中。代码量不是判据；不建 `utils` / `common`；泛型不放在 crate 边界上。理由见 ADR 001。

---

## 3. 核心 trait 与数据流

### 3.1 模型接口

p1 的 `Model` 保持不变，已搬进 `vdn-types`（`src/types/src/model.rs`）：`Send`；`name / sample_rate / hop / latency_samples / reset / process`；一个 hop 进、一个 hop 出的波形接口；`ModelError` 是不带 `String` 的 `Copy` 枚举。规则：
- **构造在非实时线程**（打开会话、预分配、预热），实时线程只调 `process` 与 `reset`
- 频域模型在适配器里自己做 STFT；「共用一次 STFT」（设计稿 §11）以后用**扩展 trait** 补，不改 `Model`
- 加载时用 `session.inputs()/outputs()` 校验名字与形状，不从 ONNX 元数据生成代码（p1 `gtcrn.rs` 已这样做）

### 3.2 分发

- `Engine<M: Model>` 静态分发；运行期选模型用 `vdn-models::AnyModel`（p1 已实现，含小宏 `each!`）
- 不用 `Box<dyn Model>`：变体是封闭集合，值不在堆上，没有在实时线程释放 trait 对象的问题（ADR 003）

### 3.3 Engine 与 Pipeline：两个采样率，两层

```rust
// vdn-types（src/types/src/frame.rs；p2 提议后的形态）
pub struct SampleClock(pub u64);                       // 流时间 = IO 采样率样本数；所有时间戳都用它
pub enum SpeakerClass { Unknown /*放开*/, NoSpeech /*保持现状*/, Target, Other }
pub struct SpeakerVerdict {                            // Copy；经 triple_buffer 进实时线程
    pub class: SpeakerClass, pub confidence: Confidence,
    pub window_start: SampleClock, pub window_end: SampleClock,   // 判决所用声纹窗：起始之后算出的判决才作数
    pub owner_level_db: Option<f32>,                   // 主人说话电平；None = 未注册
}
pub struct FrameCtx { pub now: SampleClock /*本 hop 首样本*/, pub speaker: SpeakerVerdict, pub voice_prob: Option<f32> }

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

pub trait Stage: Send {          // vdn-types；门控、衰减下限、修音链节点，跑在模型采样率
    fn name(&self) -> &'static str;
    fn reset(&mut self);
    fn process(&mut self, wet: &mut [f32], dry: &[f32], ctx: FrameCtx);
    fn latency_samples(&self) -> usize { 0 }   // 进延迟账（§3.7.5）
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

### 3.7 智能体部件：感知 · 策略 · 记忆 · 学习 · 延迟账

> 依据：D35（降噪是没有语言模型的智能体系统）· D38（本机实时自适应，察觉不到、可关）· D39（无云端、不上传）· D40（场景路由）· D41（两层场景）；`docs/plan/005_evolution-and-simulation.md` 第〇节与「场景路由」。
> 接口与数据结构住在 `vdn-types`（零依赖）；**本节只定接口**，场景分类、声纹、学习的实现分别归 scene、speaker（p2）与后续批次。

**三条回路，三档线程**：

| 回路 | 周期 | 部件 | 档 | 碰声音吗 | 进延迟账吗 |
|---|---|---|---|---|---|
| 反射 | 每 hop（16 ms） | `Model` · `Stage` · `Engine` / `Pipeline` | B | 碰 | 进 |
| 判断 | 约 0.5 s | 感知（`SceneLabel`）· 策略（`Policy`） | C | **不碰**，只出 `ParamSet` | **不进** |
| 学习 | 数天，空闲时 | 学习（`Learner`）· 记忆（`UserProfile`）· 决策日志（`SessionRecord`） | C，省电核低优先级 | 不碰 | 不进 |

**数据流**：感知 → `SceneLabel` → `HysteresisGate`（换不换场景）→ `resolve_params`（场景参数表 → 我的配置 → 当前旋钮）→ `Decision { target: ParamSet, transition }` → rtrb 送一个 `Copy` 包进实时线程 → 实时侧 `ParamSet::glide_toward` 逐 hop 平滑；同时 `Decision::log_into` 写决策日志。

**进出实时线程的两个包**（ondevice 线需求，2026-10-31）：
- 判断 → 实时：`ParamBlock { seq, config: ContentId, model_hash: ContentId, params: ParamSet, transition }`，`Copy`，经 rtrb 送（§3.4「必须同时生效的一组参数」那一行就是它；§3.4 的 `AtomicU32` 只留给单个旋钮的即时值）。实时线程在帧边界取最新一份，没有新的沿用旧的，绝不等。完整格式校验在判断线程写入前做；实时线程只调 `ParamBlock::accept_rt`（`seq` 更新、`model_hash` 与正在跑的模型一致、全部有限 → 钳位后采用，否则沿用旧块）。**覆盖层张量绝不进参数块**。进 Stage 走 `Stage::set_params`
- 实时 → 判断：`FrameFeatures`（scene 定稿：电平 · 底噪 · SNR · VAD · 16 带能量 · 平坦度 · 高低频比 · 质心 · 谐波性与峰位 · 谱峰保持率 · 削波占比 · 是否送原声；谱取带噪输入、与 GTCRN STFT 同口径），每 hop 一个经 rtrb 送出，满了丢弃并计数；噪声谱复用模型的 STFT（扩展 trait `SpectrumTap::last_magnitude`，不改 `Model`），不另算。FE-M 这类拿不到谱的由特征通道自己做一次 FFT，只喂判断、不进音频路径
- 平滑：连续参数在 hop 内逐样本插值、dB 类字段限速 `Transition::max_db_per_s`（初值 1 dB/s）、手动改动最短斜坡 `manual_ramp_s`（初值 30 ms）；离散参数（模型、修音链预设）走交叉淡化，**等增益**（等功率对相关信号会鼓 +3 dB），淡化期算力 ×2 计入最坏帧；48 kHz 层的参数（`strength` 湿干比）在 Pipeline 层渐变。零前瞻、零附加延迟

#### 3.7.1 感知输出（`src/types/src/scene.rs`）
- `SceneLabel { user: UserScene, fine: FineSceneId, confidence: Confidence, at: SampleClock }`，`at` = 所用音频窗的末尾
- 两层（D41）：`UserScene`（用户看得到、可手动切、可锁定；现为 Quiet · Cafe · Commute · Music · Meeting，**类目待人定**）· `FineSceneId(u16)`（内部细场景，含义由场景参数表定义，加类目不改代码；0 = 未知）
- 判不准时不出新标签、保持当前场景；不设 `Other`

#### 3.7.2 策略（`src/types/src/policy.rs`）
- `trait Policy: Send { name; reset; decide(&PolicyInput) -> Decision }`，**可单独替换**（005 第〇节）；`TablePolicy` = 查表 + 滞回的参考实现
- 滞回 `Hysteresis { confirm_s, min_confidence, min_dwell_s }`：按 `(用户层, 细场景)` 这一对比较（分类器只出用户层时也能切）；连续确认才换，**一次低置信就打断、候选从头算**；换后至少停留
- 细场景在表里时，用户层以表为准（标签里的 `user` 只在细场景未知时用）
- 平滑 `Transition { glide_s, model_crossfade_s }`：连续参数一阶平滑，离散参数（模型、修音链预设）即时切、由 engine 交叉淡化（§3.5）
- 用户优先：`UserControl { scene: Auto | Manual(UserScene), lock, knobs }`，手动选场景时只用该场景下的细场景；锁定冻结的是「表 + 我的配置」那一层，**旋钮照常盖在上面**
- 反馈入口 `Policy::observe(&DecisionEvent)`（默认空）：以后的老虎机从这里拿奖励（005 第三节 B 第 3 步）
- 参数进 Stage：`Stage::set_params(&ParamSet)`（默认空，实时线程逐 hop 调，不分配）
- 滞回默认 1 s / 0.4 / 3 s 由 scene 线在模拟数据上标定（exp006 held-out：用户层准确率 93%、生效延迟中位 3.7 s、错切 0.88 次/分钟）；设备位不进滞回、另用施密特触发；平滑 1 s / 50 ms 仍是占位，待听测

#### 3.7.3 场景参数表（`src/types/src/table.rs`）与参数组（`params.rs`）
- `ParamSet`（`Copy`）：`model: ModelId` · `tone: ToneId`（修音链预设编号，0 = 不加工，预设内容归 `vdn-polish`）· `strength`（只管模型湿干比）· `floor_db` · `open_db` · `detect_db` · `release_s` · `hangover_s`；每个连续字段有闭区间范围，加载时校验，越界拒收
- `SceneTable`：每个用户层场景一组基础参数 + 最多 64 个细场景；**定长 `Copy`、< 8 KB**，控制线程解析校验好后按值换上，实时线程不分配
- 版本化：`TableVersion`（单调递增，回滚 = 换回旧版本）+ `ContentId`（规范文本的 FNV-1a 64，决策日志里记它）
- 序列化：自写行文本格式（`text.rs`：`键 = 值` + `[种类 名字]` 小节，第一行 `vdn-scene-table = <格式号>`，格式号比程序新一律拒读），往返无损；**核心不引 serde**（§2.2 规则 1）。细场景小节先继承所属用户层的参数

#### 3.7.4 记忆与学习（`src/types/src/memory.rs`）
- 配置绑定 `ConfigBinding { model_hash, device（输入设备 UID 的内容 ID，不存设备名）, os_major }`：`UserProfile::binding_matches` 不符就回出厂；配置文件当不可信输入校验（格式号、范围、重复、非有限值都拒收）
- 「我的配置」`UserProfile`：按 `SceneKey::{User, Fine}` 存 `Overrides`（每个字段 `Option`），叠加顺序 表 → 用户层覆盖 → 细场景覆盖；**只存参数，类型里没有放音频的位置**（D39）；有 `ProfileVersion` + `ContentId`，可回滚、可一键清空
- 声纹向量不放这里，归 `vdn-speaker` 自存
- `trait Learner { propose(&[SessionRecord], &UserProfile) -> Option<UserProfile> }`：只在空闲时调（插电 · 未开会 · 不发热，D38），返回**候选**，过回归门禁 / 影子比对更好才换；在线只调参数、不改权重（005 第五节待人定 #2）。**本批不实现**

#### 3.7.4b 留给后续 crate 的约束（ondevice 线提出，本批只记不做）
- **小适配器 / BN·LN 覆盖层**要在导出模型时就做成图输入（ORT 加载时会把 BN 融进卷积，事后改不了）；换覆盖层走 §3.5：预热、暗跑 N 帧、等增益淡化
- **计数器**（xrun、最坏帧 `fetch_max`、NaN 次数、按原因分的送原声次数）放进 `vdn-shm` 头，判断回路按 0.5 s 窗口 `swap(0)` 读清
- **配置仓的消费方是 `vdn-denoised`**，不是 HAL 插件（插件在沙箱里读不到用户目录）；场景参数表与我的配置绑定「模型哈希 · 设备 · macOS 大版本」，对不上就回出厂
- 进程级送原声由送音进程按每块心跳**立即**触发；看门狗只管重启推理进程（次数有上限）；所有失败走原声不走静音、延迟对齐（§3.6、ADR 005）
- **影子比对走离线同码路径**：`vdn-engine::offline` + `SafetyConfig::apply_params` 能跑任意参数组，用出厂测试集 / 模拟环境比，不在通话中跑
- **「正在开会」信号**：虚拟麦的 `StartIO` / `StopIO` 计数经 `vdn-shm` 头对外只读，空闲回路据此决定开不开跑
- **本地功能开关**：每个新行为一个开关，发版包里写死到期日，不联网下发（D39）；开关清单随 `vdn-control` 落地
- （若 ondevice 拍板 #1 选「通话中累计归一化统计」）推理时把 BN 输入的均值 / 方差作额外图输出，判断线程累加，不存音频；实时线程只拷固定大小数组，额外开销先实测

#### 3.7.5 决策日志（`src/types/src/record.rs`）
- `DecisionEvent`（`Copy`，≤ 64 B）：`SceneObserved`（感知原始输出，未经滞回；细场景分类的回放与回归靠它）· `SceneCommitted` · `ParamsApplied`（带完整 `ParamSet` 与来源 `ParamSource`）· `UserAction`（反馈事件总线：每条带当时的细场景与配置内容 ID；开关、旋钮、撤销、选场景、锁定、手动换模型、反馈「吃字 / 怪声 / 太吵 / 好 / AB 选 A / 选 B」、换设备——不记设备名、带新采样率）· `Bypass` · `Enroll`（声纹注册锁定 / 重锁 / 清空）· `TableChanged` · `ProfileChanged`
- `SessionRecord { header: { rate, table: (版本, 内容 ID), profile: (版本, 内容 ID) }, events }`：**不含音频**（D19）、只在本机（D39）；文本一行一事件
- 实时线程不写日志（R3）：送原声等事件以后经 rtrb 送出，由 C 档汇总成一条

#### 3.7.6 延迟账（`src/types/src/latency.rs`；ADR 005 是正本）
- 每个部件声明自己的延迟：`Model::latency_samples()`、`Stage::latency_samples()`、engine 的「攒满一个 hop」；`Engine::latency_ledger()` 汇总成 `LatencyLedger`，平台部分（重采样、IO、跨进程环、HAL）在写进代码前按 009 延迟账「目标上限」记成 `Basis::Budget`（`latency::planned_platform()`），P3 实测后改 `Basis::Measured`
- 判断回路与学习回路不进账
- 路由池里只放同延迟档的模型：`vdn-models::ModelKind::declared()` 声明 hop 与延迟，`tests/smoke.rs` 打开真模型核对
- **逐项硬加，不认「两段等待重叠」的推断**（align 自审：「IO 缓冲并入攒 hop」要求 IO 块整除模型 hop，FE-M 的 hop 是 480 帧 @48k，256 不整除，不成立）
- **当前账：正本在代码，本规约不抄数字**（CLAUDE.md 第五节第 5 条唯一真相源；2026-11-02 起）。正本 = `Pipeline::latency_ledger()`（代码声明的部分：重分块 · 重采样 · 模型）+ `vdn_types::latency::planned_platform()`（平台部分的规划预算，去掉其中的重采样项，避免重复记）；`cargo test -p vdn-models --test latency -- --nocapture` 打印当前账。旧表（2026-10-31，009 目标 47.00 ms · 提议 43.67 ms）作废：重采样代码声明 2.00 ms 而非规划 4.00 ms
  - 链路实测见 `docs/experiments/exp-v7-latency/`（判据 V7，release，空闲 + 压力）
  - 测试 `src/models/tests/latency.rs`：提议块下每个模型 ≤ 45 ms、池内同延迟；009 原目标 47 ms 钉成**已知缺口**（断言超预算），任何一项变了都会红
  - 对外口径：「代码里的部分 32 ms；平台部分未实测，按 009 目标合计 47 ms、按提议参数 43.67 ms」

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
| 静态·依赖闭包 | `cargo run -p xtask -- check-deps` | 某 crate / bin 的**整个闭包**里不许出现 X | — |
| 静态·项目约定 | xtask tidy（普通 `#[test]`，文本 / 轻语法检查，照 rust-analyzer） | forbid 头、`// NRT:` 注释、`.gitignore`、哨兵、`std::fs` / `log` 文本检查 | 语义 |
| 运行期·Rust 分配 | 计数分配器（release 有效）；debug 测试里的「不许分配」断言（009：nice-assert-no-alloc 或自写约 60 行） | Rust 全局分配器上的分配 | C / C++ 库的分配 |
| 运行期·libc | rtsan-standalone `#[nonblocking]`，`RTSAN_ENABLE=1` 构建，经 `cargo test` / `cargo run` 跑回放 | malloc / free / 锁 / sleep 等，包括 C++ 内部 | 回放没跑到的路径 |

### 5.2 规则表

| # | 规则 | 机器检查 |
|---|---|---|
| R1 | 实时路径不分配、不释放 | 禁用清单（分配类）· 计数分配器每 hop 计数 = 0（按模型登记的上限除外，§4 B 档）· RTSan |
| R2 | 实时路径不加阻塞锁 | 禁用清单（`Mutex`、`RwLock`、`Condvar`、`mpsc`）· RTSan |
| R3 | 实时路径不做 IO、日志、sleep | 禁用清单（`println!` 等宏、`thread::sleep`）· tidy 文本检查实时模块里没有 `std::fs`、`log::` · RTSan |
| R4 | 实时路径不 panic | crate 根 deny `unwrap_used / expect_used / panic / indexing_slicing`（不作用于测试）· `assert!` / `assert_eq!` / `assert_ne!` 由 **tidy** 查实时块（clippy 禁不了：`debug_assert*` 展开就是 `assert*`，禁了一起红【测】）· 故障注入测试 |
| R5 | 不用最坏情况无界的算法 | bench 看 p99.9 / max（§6 G10） |
| R6 | 跨线程只用 §3.4 的原语 | 禁用清单（类型类）· 自写无锁结构只许在 `vdn-shm`，跑 loom + Miri |
| R7 | 实时线程不 drop 堆对象 | RTSan 拦 `free` · 计数分配器计 dealloc · 回收队列满的单测 |
| R8 | 实时线程有 time-constraint 优先级并加入 Audio Workgroup | 启动自检读回线程策略、检查 `os_workgroup_join` 返回码；失败计数并上报，不静默 |
| R9 | FTZ 守卫只包住 DSP 段，**不包推理调用** | 单测：守卫内非正规数被冲成 0、守卫外不冲；`Model::process` 的推理调用在守卫外（009 §一.2：intra=1 时 FTZ 会作用到 ORT，DeepFilterNet #713） |
| R10 | unsafe 只在白名单 crate | workspace `unsafe_code = "deny"`；非白名单 crate 的 `lib.rs` 写 `#![forbid(unsafe_code)]`（workspace 设 forbid 再局部 allow 会报 E0453）· `undocumented_unsafe_blocks = "deny"` · tidy 查 forbid 头。白名单 = `metadata.vdn.unsafe = true` 的 crate（2026-11-02：`vdn-shm` · `vdn-rt-thread` · `vdn-hal` · `vdn-dsp`（只在 `ftz` 模块，FTZ 守卫要写 FPCR））|
| R11 | 实时路径出错 = 返回 `Status` + 送原声（NaN / 异常帧不能拿来做交叉淡化：硬切原声，用接缝偏移衰减保持波形连续——p1 `Engine` 已这样做） | `#[must_use] Status` · 故障注入：超时 / NaN → 输出等于对齐原声（带淡化） |
| R12 | 依赖方向（§2.2） | cargo-deny 管直接边 · check-deps 管闭包；规则都从 `metadata.vdn.forbid` 读 |
| R13 | 延迟有账 | `LatencyLedger`（§3.7.6）· `src/models/tests/latency.rs` 断言 ≤ 45 ms、池内同延迟；接入 Pipeline 后改断言 `Pipeline::latency_samples()`；P3 脉冲法实测（延迟账正本 ADR 005） |
| R14 | 构建产物不进库 | tidy 查 `.gitignore` 含 `/target/` |

### 5.3 clippy 配置的写法

- 根目录**只放一份** `clippy.toml`；crate 目录下不放（会整份替换上层、不合并【测】`appendix-probes/README.md` §5）。建这个文件之后，**清单以文件为准**，本规约只写原则
- `[workspace.lints.clippy]` 把 `disallowed_methods / disallowed_types / disallowed_macros` 设为 `allow`；在**实时 impl 块或实时模块**上升为 deny（tympan-aspl 的做法：`resources/tympan-aspl/src/realtime/mod.rs:45`）。构造函数放另一个 `impl` 块，不用逐个 `#[allow]`
- panic 类 lint 放 crate 根，但不作用于测试：`#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing))]`；根 `clippy.toml` 设 `allow-unwrap-in-tests` 等
- 非实时整模块（离线驱动、故障注入器）在模块头 `#![allow(...)] // NRT: <理由>`
- 例外写法：`#[allow(clippy::disallowed_…)] // NRT: <理由>`，注释在同一行行尾或紧邻的上一行；tidy 用正则检查理由非空，并统计 allow 总数，总数涨了在 PR 里显示
- **探针结论（align，2026-10-31，clippy 1.99）**：根 `clippy.toml` 里每一条都在哨兵 crate 里确认会报红；两条**放不进**：`assert*!`（会连 `debug_assert*` 一起禁，改由 tidy 查）、`<[T]>::to_vec`（各种路径写法都不命中，交给计数分配器与 RTSan）
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

- `cargo run -p xtask -- check-deps`：用 `cargo metadata --format-version 1` 读 `resolve.nodes[].deps[]`，只取普通依赖（`dep_kinds[].kind == null`），按包 BFS 求闭包，与 `forbid` 求交，非空以 1 退出。resolve 是 feature 合并后的图，只会多报，不会漏报
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
| clippy 禁用清单 + restriction lint（§5.3） | build.rs 生成代码。改用 schema：Rust 类型是正本，由工具 crate `vdn-schema` 导出到 `specs/schema/`、提交进仓，测试重新导出逐字比对（`specs/schema/README.md`） |
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
| G3 | 依赖方向与许可 | `cargo deny check` + `cargo run -p xtask -- check-deps` | L1 | 模块边界被打破，或不能按 GPL-3.0 分发 |
| G4 | tidy | `cargo test -p xtask` | L0 L1 | 规约被绕开 |
| G5 | 单元 + 集成 | `cargo nextest run`（含计数分配器断言、故障注入） | L1 | 功能回归 / 实时路径分配 |
| G6 | 分块不变性 | proptest：同一段音频随机切块喂 Pipeline，与整段一次喂的结果逐样本一致 | L1 少量 / L3 大量 | eval 与实时不是同一条路 |
| G7 | Python 对拍 | golden 向量，容差比较（数值可能随 CPU 特性变化【推】） | L1 短片段 + 升级 ort / 模型时 | 听到的不是那个模型 |
| G8 | 实时阻塞调用 | RTSan（§5.5） | L2，macOS arm64 | A 档有任何违规；B 档出现豁免以外的栈 |
| G9 | UB 与并发 | Miri（`vdn-dsp`、`vdn-shm`）；loom（`vdn-shm`） | L2 | 自写无锁协议有竞争 |
| G10 | 耗时与分配计数 | bench 每 hop p50 / p99 / p99.9 / max + 分配数 | L4（CI 虚拟机数值不可信） | 实时余量被吃掉 |
| G11 | 端到端延迟 | 脉冲法（P3 起） | L4 | 超 D32 |
| G12 | 评测集回归 | `cargo run -p xtask -- eval ci`（CI 层，每个 PR）· `-- eval qa`（QA 层，本机，改模型 / DSP 时）；见 §7.1 | L1（CI 层）· L4（QA 层） | 回归集退化或 verifier 被骗过（指标过 ≠ 人耳过，D33） |
| G13 | 长跑 | 真设备 ≥ 1 h：xrun、漂移、内存 | L4 | 现场才遇到的问题 |
| G14 | 人耳 | 盲听页（CLAUDE.md 第五节第 1 条） | 里程碑 | 验证红线 |

- **许可**：`deny.toml` 的 allow 列表以文件为准。原则：只放 GNU 列表上与 GPLv3 兼容的许可（Apache-2.0 只与 GPLv3 兼容、不与 GPLv2 兼容【官】https://www.gnu.org/licenses/license-list.en.html）；列表上查不到的（如 BSD-1-Clause、LLVM-exception）用 per-crate `exceptions` 逐个放行；MPL-2.0 的 Exhibit B cargo-deny 查不出，CI 另 grep；ORT 预编译库与 `.onnx` 权重不在 Cargo 图里，另列资产清单（来源 · 哈希 · 许可）
- **CI 费用**：CI 只跑 macOS（人定 D76，2026-11-02，原话『直接用mac 就好，先在mac上跑通』）；平台无关闸门放 Linux 的做法作废

### 7.1 eval（`vdn-eval`，`src/eval/`；设计与逐条对照见 `docs/experiments/exp007-eval-suite/README.md`）

**祈使句**：
1. **verifier 只住在 `vdn-eval`**，一条一个函数，登记在 `verifiers.rs` 的 `REGISTRY`（定义 · 算法 · 锚点 · 默认级别 · 已知失效）。结论只有三种：过 · 不过 · **不知道**（带原因）。前提不满足时返回不知道，不硬给分
2. **和实时同一份处理代码**：被测系统一律是真 `vdn_engine::Engine` 包模型、逐 hop 调 `process`；频谱分析用 `vdn_dsp::StreamingStft`。不许在 eval 里另写一条处理链
3. **两层**：CI 层（`cargo run -p xtask -- eval ci`）纯 DSP、确定性、无模型权重、几分钟内跑完，挂在 `ci.yml`；QA 层（`-- eval qa`）要本机模型（Qwen3-ASR、DNSMOS），不进 CI。CI 没有权重时被测系统是参考降噪器 `ref`（crate 内，替身，不是产品模型）；本机有 GTCRN 权重时自动加跑
4. **门禁规则**（`runner.rs` 头注释是正本）：回归集 L3 不过 → 阻断；L0/L1 感知量只报警，但**比冻结基线 `suite/baseline-ci.json` 差容差以上 → 阻断**；能力集从不阻断；防作弊探针没抓住劣化 → 阻断
5. **级别正本是 `suite/levels.json`**，只由 `vdn-eval calib --write-levels` 改。感知类定标前一律 L0；模拟真值指标只在合成片段上算 L3
6. **任务清单与参数进仓，音频不进仓**：CI 任务是合成场景（参数 + 种子即全部）；QA 任务引用仓外音频并核 sha256。报告 `--record` 进 `docs/experiments/exp007-eval-suite/results/`
7. 有意改变行为（改 engine 默认参数、换模型）时：`vdn-eval ci --write-baseline` 重写基线，PR 里写为什么变
8. ort 在 `vdn-eval` 里直接用（DNSMOS），是登记的例外；拆 `vdn-backend-ort` 时并入后端

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
