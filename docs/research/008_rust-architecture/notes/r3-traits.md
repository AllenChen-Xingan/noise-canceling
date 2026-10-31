# R3 · 核心 trait 与数据流（调研笔记）

> 主题 3：模型适配器 / 帧处理器 / 门控 / 音色链的抽象；静态 vs 动态分发；跨线程状态与参数；模型热切换·交叉淡化·延迟补齐。
> 证据等级：【官】官方明载 /【源】源码可证 /【测】本机实测 /【传】二手或论坛 /【推】本笔记推断。
> 本机克隆根：`R = /Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources`（下文 `R/...:行号`）。
> 本笔记只取「规则、做法、取舍」层，不为单个项目写卡（逐项目核查归 rusteco）。
> 本轮**没有跑任何基准**，所以全篇没有【测】。

---

## 0. 结论先行（10 条）

1. **所有成熟项目都把「非实时的构造与分配」和「实时的处理」切成两个对象或两个阶段**。nih-plug 有 `initialize()`/`process()`，firewheel 有 `AudioNode`/`AudioNodeProcessor`，fundsp 有 `allocate()` 加 `Net` 的前端/后端。本项目照做：`*Spec`/`*Builder` 在控制线程上建好、分配好、预热好，再把成品 `Processor` 交给实时线程。实时 trait 里**不出现**会分配的方法。（出处见 §1）
2. **抽象按「帧」来做，不按「样本」或「块」**。最小可替换单位是 hop（GTCRN 是 256 样本 @16 kHz = 16 ms，【源】`R/biglinux-gtcrn/stream/gtcrn_stream.py:371`）。帧长、窗长和 `latency_samples` 做成适配器在运行时报告的元数据，不做成类型参数，因为主备模型要在运行时切换。
3. **模型适配器拆成两层**：`InferBackend`（ort / tract：按固定 shape 跑一次图）和 `SpectralModel`（GTCRN / FastEnhancer：知道输入输出张量的语义，并持有**显式 RNN/conv cache**，用乒乓缓冲交换）。STFT 分析/合成单独做成 `StftFrame`，参照 libDF 的 `DFState`。
4. **静态还是动态分发，看调用频率，不看信仰**。每样本、每节点调用的 DSP 用泛型和固定 struct（fundsp 明说动态图的开销就是 `Box<dyn AudioUnit>` 调用开销）。每帧调一次的模型槽位每秒只调约 62 次，`Box<dyn SpectralModel>` 的 vtable 成本可以忽略【推】。动态分发在实时路径上真正的危险是**换模型时在实时线程上 drop 旧 Box**，不是 vtable。
5. **跨线程按数据形状选原语**（Timur Doumler 的分法【官】）：标量参数用 `AtomicF32` 加实时侧平滑（nih-plug 的 `FloatParam` 就是这么做的【源】）；离散命令/事件用 rtrb SPSC；声纹结果这种「只要最新、不要每一个」的数据用 `triple_buffer`；体积大或换得少的对象（模型、EQ 系数表）走 rtrb 传指针，旧对象经**回收队列**送回非实时线程释放（fundsp `Net` 的 `NetReturn` 模式【源】，或者用 basedrop）。
6. **arc-swap 不推荐放在实时读端**。它的读是 lock-free、多数时候 wait-free【官】，但旧值的最后一次引用计数递减发生在读者那边，实时线程可能因此执行 dealloc【推，依据 arc-swap 性能文档】。seqlock 只适合 `Copy` 的小 POD，读者要重试，不是 wait-free【官】。
7. **热切换做成实时线程里的一个状态机**：`Steady(A)` → 收到 B → `Warming{A,B}`（两个都跑，丢掉 B 的输出，让 B 的 RNN 状态收敛）→ `Fading{A,B}`（时域等功率交叉淡化）→ `Steady(B)`，A 走回收队列。前提有两个：两路**先按引擎固定延迟 L_engine 补齐**（每个槽位配一条预分配的 pad 延迟线），并且 CPU 预算要容得下双倍推理。
8. **延迟是一等公民**：每个节点报告 `latency_samples()`，引擎在构造时求和并固定 `L_engine`。原声旁路和干湿混合也要按 L_engine 延迟对齐。fundsp 的 `route()` 做延迟分析【官】，nih-plug 的 `set_latency_samples` 注明运行中修改「may cause audio playback to be restarted」【源】。所以新模型延迟超过 L_engine 时，不做热切换，走重建流。
9. **「超级语法树 / AST」最合适的落点是两处**。一是用 proc-macro（derive）从一份参数 struct 生成参数表、序列化、UI 绑定，这符合唯一编码原则，nih-plug 的 `#[derive(Params)]` 就是先例【官】。二是把「实时路径不分配、不加锁」写成机器可检的注解或 lint：`rtsan-standalone` 的 `#[nonblocking]`【官】、clippy `disallowed_methods`、`assert_no_alloc`。**不建议**用宏或 AST 生成整条信号链，固定 struct 加 trait 已经够用，也更好调试【推】。
10. **最大的风险不在 trait 设计，在推理后端本身**：libDF 的 `DfTract::process` 路径上有 `tvec!`、`clone()`、`push_back` 和 `log::warn!`【源】`R/DeepFilterNet/libDF/src/tract.rs:460-542,527`；tract `SimpleState::run` 每次都返回一个新的 `TVec<TValue>`【官】。所以「推理线程零分配」很可能做不到，要尽早用 rtsan 实测，再决定推理是放在 CoreAudio IO 回调之外的专用高优先级线程（当前架构本来就是这样），还是改用 ort IoBinding 预绑定输出（§3.4）。这一条**待人拍板**。

---

## 1. 别人怎么切「实时 / 非实时」

| 项目 | 非实时侧 | 实时侧 | 关键原话 / 位置 | 等级 |
|---|---|---|---|---|
| nih-plug | `initialize(&mut self, &AudioIOLayout, &BufferConfig, &mut impl InitContext) -> bool`；`params() -> Arc<dyn Params>` 和 `editor()` 都只在创建后查一次 | `process(&mut self, &mut Buffer, &mut AuxiliaryBuffers, &mut impl ProcessContext) -> ProcessStatus`；`reset()` 可能在音频线程调用，文档说 "You should thus not do any allocations in this function" | https://github.com/robbert-vdh/nih-plug/blob/master/src/plugin.rs（trait 文档：once-only 查询是为了 "allows the audio thread to have exclusive access to the `Plugin` object"）；`assert_process_allocs` feature 在 debug 下一旦分配就 abort | 【源】 |
| nih-plug Params | `#[derive(Params)]`，`#[id = "stable"]`（≤6 字符），`#[persist]`，`#[nested]` | `FloatParam.value: AtomicF32`（atomic_float crate），Relaxed 读写；`smoothed: Smoother<f32>` | https://github.com/robbert-vdh/nih-plug/blob/master/src/params.rs ；https://github.com/robbert-vdh/nih-plug/blob/master/src/params/float.rs ；persist 字段文档要求用 RwLock/Mutex，说明它**只在非实时侧用** | 【源】 |
| nih-plug 延迟 | `InitContext::set_latency_samples(&self, samples: u32)`："If the plugin is currently processing audio, then this may cause audio playback to be restarted." | — | https://github.com/robbert-vdh/nih-plug/blob/master/src/context/init.rs | 【源】 |
| firewheel | `AudioNode`：`info()`（名字、通道、延迟）、`construct_processor(ctx)`、可选 `update()` 用来排队事件 | `AudioNodeProcessor`：`events()`（"always called in a realtime thread"）、`process(ProcInfo, ProcBuffers, ...) -> ProcessStatus`；`new_stream()` "gets called on the main thread … safe to allocate/deallocate here" | https://github.com/BillyDM/Firewheel/blob/main/crates/firewheel-core/src/node.rs ；README 称 "Properly respects realtime constraints (no mutexes!)" | 【源】【官】 |
| fundsp | `allocate()`："should be called last before sending something into a real-time context"；`Net` 前端负责改图，`commit()` 把改好的图送到后端 | `NetBackend`："real-time safe backend renders audio"；固定块长 64（`MAX_BUFFER_SIZE`） | https://github.com/SamiPerttu/fundsp/blob/master/README.md ；`src/net.rs`：`commit()` 先排空 `NetReturn` 队列，注释写 "Deallocate all previous versions"，**旧对象在前端释放** | 【官】【源】 |
| cpal | 用 builder 建流 | 回调是泛型 `D: FnMut(&mut [T], &CallbackInfo) + Send + 'static`，状态 move 进闭包，不共享引用 | `R/cpal/src/traits.rs:400,458,620` | 【源】 |
| rubato | 构造时给定最大比率 | `process_into_buffer` 用预分配的输出（文档说 `process` 每次分配，"For realtime applications, use process_into_buffer"）；`output_delay()` 报告延迟；`Adjustable::set_resample_ratio(ratio, ramp)` 可以斜坡变比 | `R/rubato/src/lib.rs:188-196,482,545-558` | 【源】 |
| libDF | `DfTract::new(DfParams, &RuntimeParams)` 加载并预分配；`init()` 预填 rolling buffer | `DFState::process_frame(&mut self, input:&[f32], output:&mut [f32])`，用 `debug_assert` 检查长度等于 `frame_size`（= hop）；`DfTract::process(ArrayView2, ArrayViewMut2) -> Result<f32>`（返回 lsnr） | `R/DeepFilterNet/libDF/src/lib.rs:50-52,161-163`；`R/DeepFilterNet/libDF/src/tract.rs:198-204,400,509` | 【源】 |
| nnnoiseless | `DenoiseState::new() -> Box<DenoiseState>`（状态大，放堆上） | `process_frame(&mut self, output:&mut [f32], input:&[f32]) -> f32`（返回 VAD 概率）；帧长是**编译期常量** `FRAME_SIZE = 480` | `R/nnnoiseless/src/denoise.rs:53,95`；`R/nnnoiseless/src/lib.rs:38` | 【源】 |
| dasp | `Signal { type Frame: Frame; fn next(&mut self) -> Self::Frame; }`，永远不返回 None，耗尽后给 `EQUILIBRIUM`；通道数是 Frame 的关联类型 | — | https://docs.rs/dasp_signal/latest/dasp_signal/trait.Signal.html | 【官】 |

**从表中提炼的规则**【推】：
- R1：实时对象只暴露 `process` / `reset` / `events` 这类方法，并且都不返回需要分配的东西。构造、加载、`allocate` 放在另一个类型或另一个阶段。
- R2：长度契约用 `debug_assert`（libDF 的做法）加类型（`&[f32; HOP]` 或 newtype）表达，**不在实时路径返回 `Result` 去做字符串格式化**。错误压缩成 `Copy` 的状态码，交给监控线程。
- R3：自带「元数据」（lookahead、latency、frame_size）的模型都把这些当公开字段或方法暴露（libDF `pub lookahead`，`tract.rs:202-204`）。本项目把它收进 `FrameSpec`。
- R4：延迟变化属于重配置事件，不是普通参数（nih-plug 的说明）。

---

## 2. 签名级草案（Rust）

> 以下是**草案**，用来讨论形状，没有编译过【推】。crate 归属对应设计稿：`dsp`、`engine`、`speaker`、`eval`。

### 2.1 帧规格与公共元数据（crate `dsp`）

```rust
/// 一个帧处理单元对外承诺的时间契约。构造后不变；变了就是重配置。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameSpec {
    pub sample_rate: u32,     // 16_000（GTCRN）
    pub hop: usize,           // 每次 process 进出的样本数（GTCRN 256）
    pub window: usize,        // 分析窗长（GTCRN 512）
    pub latency_samples: usize, // 算法延迟（含窗、lookahead），由实现报告
}

/// 实时侧最小契约：一帧进、一帧出，原地处理。不分配、不加锁、不 panic。
pub trait FrameProcessor: Send {
    fn spec(&self) -> FrameSpec;
    /// 清零内部状态（RNN cache、滤波器记忆）。必须无分配（同 nih-plug `reset` 约束）。
    fn reset(&mut self);
    /// `io.len() == spec().hop`；debug 下 assert，release 下按状态码报告。
    fn process(&mut self, io: &mut [f32], ctx: &FrameCtx) -> FrameStatus;
}

#[derive(Clone, Copy)]
pub struct FrameCtx { pub frame_index: u64, pub dry: Option<&'static [f32]> /* 见注 */ }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameStatus { Ok, Bypassed, Underrun, BackendError(u16) }
```

注：`dry` 用 `'static` 只是占位，实际做法是由引擎持有预分配的干声缓冲，用参数传入 `&[f32]`【推】。

### 2.2 推理后端（crate `engine::infer`）

```rust
/// 一个模型 I/O 槽位：名字 + 固定 shape。构造时确定，实时期间不变。
pub struct IoSlot { pub name: &'static str, pub shape: &'static [usize] }

pub struct IoSchema { pub inputs: &'static [IoSlot], pub outputs: &'static [IoSlot] }

/// ort / tract 两种实现。load 在非实时线程调用；run 在推理线程调用。
pub trait InferBackend: Send + Sized {
    type Error: core::fmt::Debug + Send;
    fn load(onnx: &[u8], schema: &IoSchema, opts: &BackendOpts) -> Result<Self, Self::Error>;
    /// 输入/输出都是调用方预分配的扁平 f32 缓冲，按 schema 顺序。
    /// 目标：不分配（ort 走 IoBinding 预绑定输出；tract 现成 API 做不到，见 §3.4）。
    fn run(&mut self, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) -> Result<(), Self::Error>;
}
```

- 用 `Sized` 加关联 `Error`，**故意不做成对象安全**：后端只在模型适配器内部用泛型参数选择，不在运行时混用【推】。
- 依据：ort IoBinding 的说明是 "bind_output: Takes a preallocated tensor that the output is written into"【官】https://ort.pyke.io/perf/io-binding ；tract `SimpleState::run(TVec<TValue>) -> TractResult<TVec<TValue>>`【官】https://docs.rs/tract-core/latest/tract_core/plan/struct.SimpleState.html

### 2.3 显式递归状态（乒乓缓冲）

GTCRN 流式 ONNX 有 3 组 cache：`conv_cache [2,1,16,16,33]`、`tra_cache [2,3,1,1,16]`、`inter_cache [2,1,33,16]`。I/O 名是 `mix, conv_cache, tra_cache, inter_cache → enh, *_out`【源】`R/biglinux-gtcrn/stream/gtcrn_stream.py:378-380,414-415,432-434`。

```rust
/// N 组状态张量，in/out 两套缓冲每帧交换，零拷贝、零分配。
pub struct RecurrentState<const N: usize> {
    bufs: [[Box<[f32]>; N]; 2], // 构造时按 schema 分配
    cur: u8,
}
impl<const N: usize> RecurrentState<N> {
    pub fn inputs(&self) -> [&[f32]; N] { /* bufs[cur] */ todo!() }
    pub fn outputs(&mut self) -> [&mut [f32]; N] { /* bufs[cur ^ 1] */ todo!() }
    pub fn swap(&mut self) { self.cur ^= 1; }
    pub fn zero(&mut self) { /* fill(0.0)，无分配 */ }
    /// 给热切换和 A/B 评测用：把状态拷进预分配的快照，不分配。
    pub fn snapshot_into(&self, dst: &mut Self) { /* copy_from_slice */ }
}
```

### 2.4 频域模型适配器（crate `engine::model`）

```rust
pub type Cplx = num_complex::Complex32;

/// 一个「单帧频谱进 → 单帧增强频谱（或 mask）出」的流式模型。
/// GTCRN、FastEnhancer-M 各实现一份；STFT 不在这里（交给 StftFrame）。
pub trait SpectralModel: Send {
    fn spec(&self) -> FrameSpec;          // hop/window/latency_samples/sample_rate
    fn n_freq(&self) -> usize;            // 257
    fn reset_state(&mut self);            // RecurrentState::zero
    /// 每 hop 调一次。失败时 `out` 原样拷 `inp`（安全降级），返回状态码。
    fn infer(&mut self, inp: &[Cplx], out: &mut [Cplx]) -> FrameStatus;
    /// 可选：模型自带的 VAD / LSNR 之类副产物（libDF 返回 lsnr，nnnoiseless 返回 VAD 概率）。
    fn aux(&self) -> ModelAux { ModelAux::default() }
}

#[derive(Clone, Copy, Default)]
pub struct ModelAux { pub vad: Option<f32>, pub lsnr_db: Option<f32> }

/// 具体模型对后端做泛型：一份适配代码同时支持 ort 和 tract。
pub struct Gtcrn<B: InferBackend> { backend: B, state: RecurrentState<3>, mix: Box<[f32]>, enh: Box<[f32]> }
impl<B: InferBackend> SpectralModel for Gtcrn<B> { /* ... */ }

/// 非实时侧：描述 + 构造。热切换和 eval 都经过这里。
pub trait ModelBuilder: Send {
    fn spec(&self) -> FrameSpec;
    fn build(&self) -> Result<Box<dyn SpectralModel>, BuildError>; // 分配、加载、预热都在这里
}
```

为什么**不**把 HOP 做成 `const HOP: usize` 关联常量：nnnoiseless 这么做是因为它只有一个模型（`R/nnnoiseless/src/lib.rs:38`）。本项目要在运行时切换 GTCRN 和 FastEnhancer，二者帧长未必一样，关联常量会让 `dyn SpectralModel` 失去对象安全，或者逼着我们把每种组合都单态化。折中做法是：**模型内部**用 const 泛型（比如 `Gtcrn<B, const HOP: usize = 256>`）让编译器展开循环，**对外**只报告运行时的 `FrameSpec`【推】。

本机缺口：`R/fastenhancer` 目录实际是一份项目骨架（README 是「Mac 开源实时语音降噪」），**不是 FastEnhancer 源码**，所以 FastEnhancer-M 的 hop、cache 形状这次没核到，未找到出处。

### 2.5 STFT 帧（参照 libDF `DFState`）

```rust
pub struct StftFrame { /* window, analysis_mem, synthesis_mem, fft plan, scratch */ }
impl StftFrame {
    pub fn new(spec: FrameSpec) -> Self;                       // 非实时
    pub fn analysis(&mut self, x: &[f32], spec_out: &mut [Cplx]);   // 实时
    pub fn synthesis(&mut self, spec_in: &mut [Cplx], y: &mut [f32]); // 实时
    pub fn latency_samples(&self) -> usize;                   // window - hop 或 window（按 OLA 方式）
}
```

libDF 的同构结构：`DFState { frame_size /* = hop */, analysis_mem, synthesis_mem, ... }`，`analysis_mem`/`synthesis_mem` 在 `new` 里按 `fft_size - frame_size` 预分配，实时侧用 `rotate_left` 原地滚动【源】`R/DeepFilterNet/libDF/src/lib.rs:104-139,357-415`。

### 2.6 门控（crate `engine::gate`）

```rust
/// 门控只读「最新快照」，不关心快照怎么来的（声纹线程、VAD、或 eval 的离线标注）。
#[derive(Clone, Copy, Default)]
pub struct GateInputs { pub speaker_sim: f32, pub speaker_seq: u32, pub vad: f32 }

pub trait GainLaw: Send {
    /// 返回本帧目标增益（线性）；平滑在 Gate 内部做。
    fn target_gain(&mut self, inp: &GateInputs, p: &GateParams) -> f32;
}

pub struct Gate<L: GainLaw> { law: L, smoother: OnePole, /* attack/release 系数 */ }
impl<L: GainLaw> FrameProcessor for Gate<L> { /* io *= smoothed gain */ }
```

`GateInputs` 由引擎在每帧开头从共享原语读一次（§4），再按值传进来。这样门控本身是纯函数式状态机，eval 可以直接喂离线序列，满足「离线与实时同一份代码」【推】。

### 2.7 音色链（crate `dsp`）

```rust
pub struct Biquad { b: [f32; 3], a: [f32; 2], z: [f32; 2] }
pub struct EqBank<const N: usize> { bands: [Biquad; N] }
pub struct TimbreChain<const N_EQ: usize> {
    pub dereverb: DeverbTail,
    pub comp: Compressor,
    pub eq: EqBank<N_EQ>,
    pub limiter: Limiter,
}
impl<const N: usize> FrameProcessor for TimbreChain<N> { /* 顺序调用，全部静态分发、可内联 */ }
```

**链是固定的，所以用固定 struct，不建图**。firewheel 和 fundsp `Net` 是给任意 DAG、游戏场景准备的（firewheel README：「targets games and generic applications」【官】），我们的链是线性、编译期已知的，引入图会带来 `Box<dyn>`、调度和回收的复杂度，却没有收益【推】。如果以后真要用户自定义链，再考虑 `enum Node { Eq(..), Comp(..), .. }` 加 `[Node; MAX]`。

### 2.8 引擎（crate `engine`）：实时与 eval 共用

```rust
pub struct Engine {
    stft: StftFrame,
    slot: ModelSlot,            // §5 的热切换状态机
    gate: Gate<SpeakerLaw>,
    timbre: TimbreChain<5>,
    dry_delay: DelayLine,       // 原声旁路按 L_engine 对齐
    shared: SharedHandles,      // §4：参数原子、声纹 triple_buffer 读端、命令 rtrb 读端、回收 rtrb 写端
    scratch: Scratch,           // 预分配
}
impl Engine {
    pub fn latency_samples(&self) -> usize;              // = L_engine，固定
    pub fn process_hop(&mut self, input: &[f32], output: &mut [f32]); // 实时 & eval 唯一入口
}
```

eval crate 只做这件事：读 WAV → 按 hop 切块 → `process_hop` → 写 WAV。声纹在 eval 里可以同步跑，也可以按「每 0.5 s 发布一次」的离线时间表喂给 `GateInputs` 源，以复现实时时序【推】。

---

## 3. 静态分发 vs 动态分发

### 3.1 能找到的论述与数据

| 来源 | 说了什么 | 等级 |
|---|---|---|
| fundsp README | `AudioNode` 是 "static, inlined"，`AudioUnit` 是 "dynamic, object safe"；"The overhead of `Net` is the overhead of calling into `Box<dyn AudioUnit>` objects"；"Static graphs can get optimized more effectively"；块处理（64 样本）"reduces function calling, processing setup and dynamic network overhead" | 【官】https://github.com/SamiPerttu/fundsp |
| enum_dispatch 文档 | 自测 Ryzen 7 2700x：blackbox 下 boxdyn 2,131,736 ns/iter，enumdispatch 471,740 ns/iter（约 4.5×）；1024 个随机类型的 Vec 下 5,900,191 对 479,630（作者称约 10×，比的是 customderive）。作者自己承认第一组 "wasn't a fair test" | 【官】（作者自测，非独立）https://docs.rs/enum_dispatch |
| Ferrous Systems 培训材料 | 认为 vtable 跳转代价小，"cheaper than an enum match"（检索摘要，未逐字核原页） | 【传】https://rust-training.ferrous-systems.com/latest/book/dynamic-dispatch |
| stanza.dev 教程 | enum 约 2.1 ns 对 dyn 约 8.4 ns，无方法学 | 【传】，不采信 |
| ljud / lyd（作者自述） | 同一作者两个 crate，Box<dyn> 对 enum，称 enum 在嵌入式上更快，没有基准 | 【传】https://docs.rs/crate/ljud/latest |
| 音频领域独立基准 | **未找到出处**（RustAudio 论坛、ADC、RustConf 都没检到「实时音频 dyn vs enum」的测量） | — |

### 3.2 取舍规则（本项目）【推】

| 位置 | 调用频率 | 选择 | 理由 |
|---|---|---|---|
| biquad、压缩、限幅、平滑器的内部 | 每样本 | 泛型 / 具体类型 | 需要内联和向量化；fundsp 的结论适用 |
| 音色链各级 | 每帧 × 4–6 级 | 固定 struct，静态调用 | 链在编译期已知 |
| 门控的 GainLaw | 每帧 | 泛型 `Gate<L>` | 只有一两种实现 |
| 模型槽位（GTCRN / FastEnhancer） | 每帧一次，约 62.5 次/秒 | `Box<dyn SpectralModel>`，或闭集 `enum AnyModel` | 一次推理是毫秒级，vtable 是纳秒级，差 6 个数量级；动态分发让 builder 能在运行时选 ONNX 和后端，热切换只需交换指针 |
| 推理后端（ort / tract） | 包在模型里 | 泛型 `Gtcrn<B>` | 避免 `dyn` 和关联 Error 冲突；后端在构造时选定 |

- **enum 还是 Box<dyn>**：模型集合小且封闭（2–4 个），用 `enum AnyModel` 加 `enum_dispatch` 或手写 match 也行，好处是不用 Box，热切换时可以 `mem::replace` 一个按值存放的 enum。代价是最大变体决定体积，需要把大张量 Box 起来。**两种都能接受，差异不在性能，在扩展性**：外部贡献者加新模型时，dyn 不必改 enum。建议默认用 `Box<dyn SpectralModel>`，并加一条测量：criterion 基准对比 `process_hop` 的两种实现，差异 < 1% 就锁定 dyn。这条需要 eval 落地后补【测】。
- **dyn 的真实风险**：①在实时线程上 `drop(Box<dyn ..>)`，会 dealloc，可能还会跑模型的 Drop（比如释放 ORT session，可能加锁）；②`Box::new` 发生在实时线程上。两者都靠 §4 的「构造在外、回收在外」来规避，与 vtable 无关。
- **单态化膨胀与编译时间**：Rust 社区的普遍看法是泛型会膨胀代码【传】。本项目的泛型组合数很小（2 后端 × 2 模型），不构成问题【推】。

### 3.3 机器证明（和「AST」诉求的交点）

| 手段 | 检什么 | 出处 | 等级 |
|---|---|---|---|
| `rtsan-standalone` | `#[nonblocking]` 标注的函数里拦截 malloc/calloc 等（文档示例：`Intercepted call to real-time unsafe function calloc in real-time context!`）；支持 Linux、macOS、iOS；`RTSAN_ENABLE=1` 开启 | https://docs.rs/rtsan-standalone | 【官】 |
| `assert_no_alloc` | 自定义全局分配器，在作用域内禁止分配 | https://lib.rs/crates/assert_no_alloc | 【官】 |
| nih-plug `assert_process_allocs` | debug 下 process 一旦分配就 abort | nih-plug `src/plugin.rs` 文档 | 【源】 |
| clippy `disallowed_methods` / `disallowed_types` | 在 rt 模块里静态禁止 `Vec::push`、`Box::new`、`Mutex`、`println!` 等（AST/HIR 层 lint） | https://rust-lang.github.io/rust-clippy/master/index.html#disallowed_methods | 【官】（lint 存在；具体配置是【推】） |
| 「不 panic」 | 未找到成熟的 Rust 实时 no-panic 证明工具；可选做法是 `#![deny(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used, clippy::panic)]` 加 `no_panic` crate（链接期检测） | `no_panic` 未在本轮核查，**未找到出处（本轮）** | 【推】 |

建议的组合【推】：clippy 禁用表做静态底线 → `assert_no_alloc` 做单元测试 → rtsan 在 eval 和实时冒烟里跑一遍真流程。三层都要有，因为 clippy 看不到第三方库内部的分配（比如 tract）。

### 3.4 推理后端零分配的现状（风险）

- libDF `DfTract::process` 一帧内的路径：`self.enc.run(tvec!(self.erb_buf.clone(), …))`，`m.pop().unwrap().into_tensor()`，`rolling_spec_buf_y.push_back(self.spec_buf.clone())`，`log::warn!("Possible clipping detected …")`【源】`R/DeepFilterNet/libDF/src/tract.rs:460-498,527,531-542`。`set_atten_lim` 里也有 `log::warn!`（`tract.rs:387-398`）。**结论**：DeepFilterNet 的官方 Rust 实时实现本身就没有做到「推理零分配」。它能实时跑，是因为 tract 的分配快且有界，而不是因为不分配【推】。
- tract `SimpleState::run` 签名决定了每次都返回新的 `TVec`【官】。
- ort 有 IoBinding 加预分配输出【官】，但 ORT 内部是否还会分配（arena）没有核到，**未找到出处**，需要 rtsan 实测。
- **对架构的含义**【推】：严格的「零分配」只对 **CoreAudio IO 回调和共享内存搬运线程**承诺；推理线程是我们自己起的高优先级线程（`audio_thread_priority::promote_current_thread_to_real_time`，`R/audio_thread_priority/src/lib.rs:24-27`），对它的承诺放宽为「有界且不阻塞」，加一层 IO 侧 jitter buffer。这条是否接受，**待人拍板**。

---

## 4. 状态与参数跨线程

### 4.1 原语对照

| 原语 | 语义 | 实时侧安全性 | 适合本项目的什么 | 出处 / 等级 |
|---|---|---|---|---|
| 原子标量（`AtomicU32` 存 `f32::to_bits`，或 `atomic_float::AtomicF32`） | 单值、最新值，可以撕裂成「旧值或新值」，不会是半个值 | wait-free | 连续参数：门限、EQ 增益、干湿比、旁路开关 | nih-plug `FloatParam`（Relaxed）【源】；Doumler：单值用 `std::atomic`【官】https://timur.audio/using-locks-in-real-time-audio-processing-safely |
| 实时侧平滑器 | 读原子后在实时侧插值 | — | 所有连续参数，防 zipper 噪声 | nih-plug `smoothed: Smoother<f32>`【源】 |
| rtrb SPSC | FIFO，每个元素都送达 | "lock-free and wait-free"；只在构造时分配，"unless the type T does that internally" | 离散命令（Reset / Bypass / LoadModel(ptr) / SetEqCoeffs(ptr)）、回收队列、IO 音频块 | https://docs.rs/rtrb 【官】；`R/rtrb/src/lib.rs:150,346,592`，`R/rtrb/src/chunks.rs:179,342`【源】 |
| triple_buffer | 单写单读，读端拿**最新**一份，可能跳过中间版本；"non-blocking" | 读写都不阻塞（文档没用 wait-free 一词） | **声纹结果**（相似度 + 嵌入 + 序号）、给 UI 的电平/状态快照 | https://docs.rs/triple_buffer 【官】 |
| seqlock | 写者从不被读者阻塞；读者发现序号变了就**重试**；T 必须 `Copy` | 读端不是 wait-free（理论上会饿死） | 小 POD 快照，写频率极低时可用；本项目首选 triple_buffer | https://docs.rs/seqlock 【官】 |
| arc-swap | 读 "always lock-free … Most of the time … wait-free"；内容换掉后 "readers still need to decrement the reference counts of the old instance" | 读端可能成为最后一个持有者，在实时线程上 drop/dealloc | 非实时线程之间共享配置（控制面、UI）；**不放实时读端** | https://docs.rs/arc-swap/latest/arc_swap/docs/performance/index.html 【官】，「实时线程 dealloc」是【推】 |
| basedrop（`Owned`/`Shared`/`SharedCell`/`Collector`） | drop 时不释放，送进队列，由 Collector 在别的线程统一回收 | 文档定位是 "real-time audio and other latency-critical scenarios"，具体哪些操作实时安全文档没写 | 模型对象、大系数表的延迟回收（替代手写回收队列） | https://docs.rs/basedrop 【官】 |
| 手写「回收队列」 | 新对象经 rtrb 送入，旧对象经另一条 rtrb 送出 | 只要队列不满就安全；满了的兜底策略要事先定 | **模型热切换**的默认方案 | fundsp `Net::commit()` 排空 `NetReturn`："Deallocate all previous versions"【源】https://github.com/SamiPerttu/fundsp/blob/master/src/net.rs |
| Mutex / try_lock | — | Doumler：即使 `try_lock`，unlock 也可能因有等待者而走系统调用，"you should not use std::mutex on the audio thread, not even with try_lock" | 实时侧禁用 | 【官】Doumler |

### 4.2 本项目映射【推】

| 数据 | 方向 | 原语 | 细节 |
|---|---|---|---|
| 连续参数（gate 门限、attack/release、EQ 各段增益、压缩阈值、干湿） | 控制 → 实时 | `AtomicF32` × N，放在一个 `#[repr(C)]` 的 `SharedParams` 里 | 实时侧每帧开头读一次，进平滑器。EQ 增益变了以后，**系数在实时侧重算**（biquad 公式无分配、开销小），或者走下一行的离散命令 |
| 离散命令 | 控制 → 实时 | rtrb `Producer<Cmd>`，`Cmd: Copy` 小 enum | `Reset`、`Bypass(bool)`、`SwapModel(ModelHandle)`，其中 `ModelHandle` 是裸指针或 `basedrop::Owned` 的包装 |
| 声纹结果 | 声纹线程 → 实时 | `triple_buffer<SpeakerSnapshot>`，`SpeakerSnapshot { sim: f32, seq: u32, t_ms: u64 }` | 2 Hz 写、62.5 Hz 读，只要最新值，正好对上 triple_buffer 的语义。如果门控只需要一个标量，也可以退化成 `AtomicU64` 打包（sim 的 bits 加 seq） |
| 嵌入向量（192/512 维） | 声纹线程内部 / 到 UI | 不进实时线程 | 实时侧不需要向量本身 |
| 模型热切换 | 控制 → 实时 → 控制 | rtrb 送新 `Box<dyn SpectralModel>`，rtrb 回收旧的（或 basedrop） | 见 §5。回收队列容量 ≥ 2，满了就**推迟切换**，不在实时线程上 drop |
| 电平、状态、xrun 计数 | 实时 → UI | `triple_buffer<Meters>` 或一组原子 | — |
| IO 音频（跨进程） | 送音进程 → 推理进程 | 共享内存上的 SPSC | **rtrb 是进程内的结构（堆上 + Arc），不能直接映射到 shm**【推，依据 `R/rtrb/src/lib.rs:150` 的 `new` 在堆上分配】。需要在 mmap 区上自写 `#[repr(C)]` 头（读写索引用 `AtomicU32`，缓存行对齐）。这条归 io-macos，由 rusteco 核对 ringbuf 等是否支持外部存储 |

### 4.3 内存序

- 参数原子用 Relaxed 就够（nih-plug 的做法【源】）。如果要「参数组一致生效」，比如 EQ 五段同时变，单靠原子做不到，改走命令队列发一个 `Copy` 的系数包【推】。
- 指针交接（新模型）用 Release/Acquire，rtrb 内部已经保证。Eckes 的 ADC 2024 幻灯片专门有一页讲 Relaxed/Release/Acquire/AcqRel/SeqCst 和「When atomics can fail」【官】https://data.audio.dev/talks/2024/building-audio-apps-with-rust/slides.pdf （本机 pdftotext 抽取的文字大部分是乱码，只能确认这几页的标题与要点）。

---

## 5. 模型热切换 · 交叉淡化 · 延迟补齐

### 5.1 延迟模型

- GTCRN：STFT 512/256 @16 kHz（`gtcrn_stream.py:371`【源】）。OLA 合成的算法延迟约等于窗长 512 样本 = 32 ms（另一种口径是 window − hop = 256，取决于合成时怎么对齐）【推】，要在 `StftFrame::latency_samples()` 里**用脉冲测试实测确定**，这条留给 eval 做【测-待】。
- 预算：端到端附加 ≤ 45 ms。32 ms 算法延迟加 IO 缓冲和重采样（rubato `output_delay()`，`R/rubato/src/lib.rs:482`）之后，余量很小【推】。所以 `latency_samples` 必须是能逐级求和的数，并在启动时打印出来做断言。
- 引擎固定延迟 `L_engine = max(所有已注册模型的 latency) + 音色链 lookahead`（限幅器如果有 lookahead 也要计入）。
- 每个模型槽位前后有 `pad = L_engine − model.latency` 的预分配延迟线，干声旁路延迟 `L_engine`。这跟 fundsp 用 `route()` 分析延迟、自动去掉 pre-delay 是同一个思路【官】。
- 新模型的 latency 大于 `L_engine` 时，**不热切换**，走重配置（停流 → 重建引擎 → 起流）。这跟 nih-plug「改 latency 可能重启播放」是同一条规则【源】。

### 5.2 实时侧状态机（草案）

```rust
pub enum ModelSlot {
    Steady { cur: Box<dyn SpectralModel>, pad: DelayLine },
    Warming { cur: Box<dyn SpectralModel>, cur_pad: DelayLine,
              next: Box<dyn SpectralModel>, next_pad: DelayLine, left: u16 },
    Fading  { cur: Box<dyn SpectralModel>, cur_pad: DelayLine,
              next: Box<dyn SpectralModel>, next_pad: DelayLine, pos: u16, len: u16 },
}
// 每帧：
//  Steady   ：cur 推理
//  收到 SwapModel(next)：-> Warming{left = WARM_FRAMES}（next 已在控制线程预热过一轮静音/噪声）
//  Warming  ：cur、next 都推理；只输出 cur；left==0 -> Fading
//  Fading   ：两路都推理；时域等功率 g_a = cos(θ), g_b = sin(θ)（表在构造时预计算，无三角函数调用）
//  pos==len ：-> Steady{next}；把 cur 推进回收 rtrb；回收队列满就停在 Fading 末态继续双跑，下一帧再试
```

要点【推】：
- **交叉淡化在时域、在 pad 之后做**，保证两路相位对齐。频域 mask 插值对不同模型不成立（mask 语义不同）。
- 预热分两段：①控制线程上 `build()` 后先喂若干帧静音，触发 ORT/tract 的首跑开销（首跑分配多、慢）；②实时侧 Warming 让 RNN 状态跟上真实输入。帧数暂定 20–30 帧（320–480 ms），要用 eval 量「切换点前后的 SI-SDR 或谱距离」来定【测-待】。
- **CPU 预算**：Warming 加 Fading 期间推理开销翻倍。如果单模型实时因子 RTF 超过约 0.4，双跑就有 xrun 风险。备选方案是「淡出到干声或静音 → 硬切 → 淡入」，代价是有一段降噪空窗。阈值由实测 RTF 决定【推】。
- 门控和音色链状态**不随模型切换重置**，只换 `SpectralModel`。STFT 帧可以共享，前提是两个模型帧规格一致；不一致就各自带 STFT，这时 `ModelSlot` 存的单位改成「STFT + 模型」组合【推】。
- 同一机制也用来做 **eval 的 A/B**：离线跑同一段 WAV，在第 N 帧触发切换，检查输出无爆音（峰值、高频能量突变阈值），可以写成规则验证脚本【推】。

### 5.3 对照：别人的热替换

| 项目 | 做法 | 等级 |
|---|---|---|
| fundsp | `Net::crossfade(node, Fade, fade_time, Box<dyn AudioUnit>)`："Replaces the given node in the network smoothly with a crossfade"；改动经 `commit()` 发给后端，旧版本经 `NetReturn` 回到前端释放 | 【源】fundsp `src/net.rs` |
| firewheel | 处理器在流停止时回到主线程；`new_stream()` 在主线程，可分配；旁路的去咔嗒由框架负责（"Declicking is handled by Firewheel"） | 【源】firewheel `node.rs` |
| rubato | 比率变化用 `ramp` 参数在下一块内斜坡过渡，属于同类「参数跳变平滑」 | 【源】`R/rubato/src/lib.rs:554-558` |
| libDF | 没有热切换；`init()` 预填 rolling buffer（相当于预热） | 【源】`tract.rs:400-422` |

---

## 6. 对人原话的回应：「超级语法树 / AST 做模块」「可拆分的核心包」「社区决策」

- **可拆分**：§2 的切法（`dsp` 无依赖纯函数 → `engine` 组装 → `infer` 后端 → `speaker` → `io-macos` → `eval`）本身就是「核心包加可替换模块」。trait 边界就是模块边界：`InferBackend`、`SpectralModel`、`FrameProcessor`、`GainLaw` 各管一层【推】。
- **AST 的正确用法**【推】：
  1. **参数单一真相源**：一个 `#[derive(VdnParams)]` 的 struct（proc-macro，用 syn/quote 操作 AST）自动生成 `SharedParams` 原子表、序列化（配置文件）、控制面 UI 绑定、eval 的参数扫描表。nih-plug 的 `#[derive(Params)]` 加 `#[id]` 是成熟先例【官】。
  2. **实时约束的机器证明**：`#[nonblocking]`（rtsan，属性宏）加 clippy disallowed 表。注解写在函数上，检查是机器的事，对应 CLAUDE.md 的「能用规则判的，绝不留给人判」。
  3. **不建议**：用宏 DSL 生成整个信号链或 trait 实现。宏展开后很难调试，错误信息差；fundsp 的运算符组合（`>>`、`|`）已经是类型级 DSL，但我们的链是固定的，用不上。这条可以写成取舍卡交人确认。
- **社区与会议里可引用的技术决策**（只列找到出处的）：
  - Stephan Eckes，"Building Audio Apps with Rust"，ADC 2024：实时安全 DSP、线程安全参数、`assert_no_alloc`、原子内存序。页面 https://conference.audio.dev/building-audio-apps-with-rust-an-overview-of-tools-and-techniques-stephan-eckes-adc-2024/ ，幻灯片 https://data.audio.dev/talks/2024/building-audio-apps-with-rust/slides.pdf ，视频 https://www.youtube.com/watch?v=G3WWr4uhP0I （视频链接来自 classcentral 转载，【传】）【官】
  - Ian Hobson，"Cross-Platform Music Software with Rust"，ADC 2025：https://conference.audio.dev/cross-platform-music-software-with-rust-ian-hobson-adc-2025 （本轮只读到摘要）【官】
  - Chase Kanipe，"Writing Elegant DSP Code with Rust"，ADC 2023：用类型系统组织处理链（二手榜单提及）https://thewolfsound.com/top-10-audio-developer-conference-2023-talks/ 【传】
  - Antoine van Gelder，"Bare Metal Audio Programming With Rust"，ADC20：https://www.youtube.com/watch?v=udlK1LQ3f3g （只核到标题）【传】
  - Timur Doumler，"Using locks in real-time audio processing, safely"（C++，但规则通用）：https://timur.audio/using-locks-in-real-time-audio-processing-safely 【官】
  - Ross Bencina，"Real-time audio programming 101: time waits for nothing"：http://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing ，本轮抓取时连接被拒，**内容没有核到**，只能作为业内常引文献【传】
  - P. Mainolfi，QUBX（DAFx24）：Rust 队列式多线程实时音频库 https://dafx.de/paper-archive/2024/papers/DAFx24_paper_43.pdf ，本轮正文抽取失败，只核到标题【官-标题】
  - RustConf 上的实时音频 trait 设计演讲：**未找到出处**

---

## 7. 待人拍板（交 decision-batching）

1. **推理线程的零分配承诺范围**：A 只承诺 IO 回调和搬运线程零分配，推理线程「有界不阻塞」（推荐，因为 tract 和 libDF 现状做不到）；B 全链零分配，强制用 ort IoBinding，并经 rtsan 验证，验证不过就换后端。
2. **模型槽位用 `Box<dyn SpectralModel>` 还是 `enum AnyModel`**：推荐 dyn，同时补一个 criterion 基准来锁定（§3.2）。
3. **热切换 CPU 不够时的降级**：双跑交叉淡化（推荐，需要 RTF ≤ 约 0.4）还是「淡出到干声 → 硬切 → 淡入」。
4. **参数 derive 宏**（§6 AST 用法 1）现在就做，还是等参数超过约 10 个再做。

## 8. 后续实测清单（让【推】变【测】）

- `StftFrame` 脉冲测试，确定 GTCRN 实际 `latency_samples`。
- rtsan 包住 `Engine::process_hop`，分别跑 ort 和 tract 后端，记录被拦截的调用。
- criterion：`process_hop` 用 dyn 和 enum 各跑一遍。
- 切换点爆音检测脚本（eval，规则验证层）。
- 补齐 FastEnhancer-M 的 hop 和 cache 形状（本机 `R/fastenhancer` 不是其源码）。
