# 009 · 推断模块组卡片（推理：ONNX 运行时 · 流式状态 · 帧循环）

> 2026-10-30 · rusteco 调研分身写（AI 草稿，待人过目）。证据路径都相对 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。
> 本轮新克隆了两个仓：`resources/ort`（pykeio/ort，约 8 MB）和 `resources/tract`（sonos/tract，约 93 MB）。其余仓是原来就有的。没有跑任何构建或推理。

## 本组覆盖

**深读（读了热路径源码）**：ort · biglinux-gtcrn（ladspa/ 下的 Rust 代码）· DeepFilterNet libDF（含 ladspa）· tract · nnnoiseless
**浅读（API、官方文档，加少量文件）**：rten · burn / burn-onnx · candle / candle-onnx · wonnx · fastenhancer · Xiaobin-Rong/gtcrn（看流式导出接口）

## 推荐：首选 + 备选 + 理由

| 子需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| GTCRN 流式 ONNX 运行时（engine） | **ort 2.0.0-rc.15，CPU EP，单线程，关掉 spinning** | **tract 0.23**（直接用，或者经 `ort-tract` 后端换着跑） | 已经有人用 ort 在 Rust 里逐帧跑通 GTCRN（biglinux-gtcrn）。四个输入、四个输出、名字总长不到 768 B 时，ort 在 Rust 侧全走栈；有 `OutputSelector::preallocate`，输出能写进预分配的缓冲。tract 的 ONNX 算子表把 GTCRN 用到的 32 个算子全登记了，但它每个节点每次 eval 都新建输出张量，分配次数比 ort 多 |
| FastEnhancer-M（同接口备选） | ort | tract | 官方只给 ONNX 发布物和 Python 版 onnxruntime 的流式示例，**没有 Rust 实现**。wav2wav 版本的 STFT 在图里，缓存作为显式输入输出 |
| 声纹 ECAPA / CAM++（speaker，跑在能效核上，不在实时路径） | ort（和 engine 共用一个 Environment，各开一个 Session） | tract | 不在实时路径上，分配不分配无所谓；共用一个运行时可以少带一份依赖 |
| 逐帧不分配的做法 | **初始化时建好两套状态 Tensor，奇偶帧轮换：A 作输入、B 作预分配输出，下一帧对调；只用两个固定的 RunOptions** | 退一步：输入和状态每帧拷进固定的 `Tensor`，不再每帧 `TensorRef::from_array_view` | biglinux-gtcrn 每帧新建四个 TensorRef，ORT 还要新分配四个输出（见卡片）；有了预分配，就同时去掉了分配和状态拷贝 |
| CoreML / ANE | **不作默认**，只当一次有界小实验 | — | 官方 CoreML EP 文档的算子表里没有 GRU 和 ScatterND（GTCRN 两样都用），图会被切碎、回落到 CPU；单帧这么小的模型，调度开销很可能吃掉省下的电（这是推断，需要实测） |
| 纯 Rust 小网络、VAD 兜底参考 | nnnoiseless（只看看） | — | 栈数组做 GRU 的范本；本身是 RNNoise 系，降噪质量不进我们的主线 |

## 本组最值得学的设计

1. **GTCRN 的「状态显式进出」接口，加上 ort 的预分配输出，做成双缓冲**
   - `resources/biglinux-gtcrn/ladspa/src/model.rs:282-302`：三份缓存（conv 16896、tra 96、inter 1056 个 f32）作为输入传进模型，模型把新状态作为输出吐回来，再 `copy_from_slice` 写回。
   - `resources/ort/src/session/run_options.rs:95-126`：`preallocate` 让输出直接落进调用方事先给的 Value，而且「只要还用这个 OutputSelector / RunOptions，就复用同一个值」。
   - 两样合起来：建两个 RunOptions，一个让输出写进状态 B，一个写进状态 A，每帧换着用。这样不拷贝状态，ORT 也不新分配输出。我们的 engine crate 就照这个写。
2. **libDF 的 STFT 状态机：scratch 预分配，analysis_mem 滚动**（`resources/DeepFilterNet/libDF/src/lib.rs:119-122` 预分配，`:356-393` 逐帧分析）。离线 WAV 和实时回调都调同一个 `process(&mut self, noisy, enh)`，正好对上我们「同一份处理代码」的软约束。**它的反例也要一起学**：`lib.rs:360` 每帧调 `make_input_vec()` 新分配一块；`tract.rs:541-542` 每帧 clone 两次 Tensor。我们自己写的时候，把 input buffer 也提前预分配好。

---

### ort（https://github.com/pykeio/ort）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | ONNX Runtime 1.31 的安全 Rust 绑定。也能把后端换成 tract 或 candle，调用 API 不变 |
| 许可证 | `MIT OR Apache-2.0`，**兼容 GPL-3.0：是**（选 MIT 那一支最省事；Apache-2.0 跟 GPL-3.0 也兼容）。底下的 ONNX Runtime 是 MIT；它的传递依赖（protobuf、abseil 等）是 BSD 或 Apache 系，这一条是推断，没逐个核。直接依赖只有 `ort-sys`、`smallvec` 两个必选，`ndarray`、`tracing` 等是可选的（`resources/ort/Cargo.toml:120-127`） |
| 维护状态 | 最新 v2.0.0-rc.15，2026-10-19 发布（前一版 rc.14 是 2026-10-12）；最近提交 2026-10-25；open issues 4 个。最近 5 个 issue 里有 3 个维护者当天到两天内就回了，剩下 1 个是 PR、1 个没回。**仍是 rc，2.0 还没出正式版**；MSRV 1.92，edition 2024（`Cargo.toml:29-30`） |
| 实时安全做法 | **Rust 侧**：输入不超过 6 个、输出不超过 4 个时，指针数组放在 SmallVec 栈上（`src/util/stack.rs:13-15`）；输入输出名字总长不到 768 B 时，C 字符串也在栈上拼（`:21`、`:96-100`）。GTCRN 是 4 进 4 出，正好卡在栈路径里。**没预分配的输出**由 ORT 每次 run 新建（`src/session/run.rs:276-285`，`from_session_output_ptr`）；**预分配了的**只做一次 `Arc::clone`，是原子操作，不分配（`run_options.rs:138`，`src/value/mod.rs:393-397`）。`TensorRef::from_array_view` 每调一次都要走一遍 `CreateTensorWithDataAsOrtValue`，再新建一个 ValueInner（`src/value/impl_tensor/create.rs:152-189`；`ValueInner::new` 里是 `Arc::new`，即 Rust 侧也有一次堆分配，`src/value/mod.rs:103`），所以**不能每帧调**。同理，未预分配的输出经 `from_session_output_ptr` 也各有一次 `Arc::new`。IoBinding 路径每次 run 还要调 `GetBoundOutputValues`，这个调用要从分配器分配、用完释放（`run.rs:301-307`），**不如 preallocate 干净**。**ORT 内部**（C++ 的 arena、线程池）不透明：同一份 Session 的 Run 不能多线程并发（`run.rs:211-219` 作者的长注释），所以必须一个线程独占一个 Session |
| 值得学的一处 | `src/session/builder/impl_config_keys.rs:127-137`：intra/inter 线程池的 spinning **默认开着**。我们配 `with_intra_threads(1)` 时再加 `with_intra_op_spinning(false)` 作保险（biglinux 只设了单线程，没关 spinning：`biglinux-gtcrn/ladspa/src/model.rs:84-90`）。**复核降级**：intra=1 时 ORT 通常不建工作线程，spinning 多半无从发生，「不关就吃掉能耗预算」是【推断】，需 powermetrics 实测。`:47-56` 的 `with_flush_to_zero` 文档写的是作用于「session 内部线程池的所有线程」（`:47-48`），intra=1 时推理跑在调用线程上，**是否生效未证实**【推断】；且 DeepFilterNet #713 表明 FTZ 会让递推归一化状态变成精确 0 进而 0/0 出 NaN——**FTZ 不能当默认开，要和 NaN 守卫一起实测再定** |
| 我们怎么用 | **直接依赖**，用在 `engine`（GTCRN / FastEnhancer）和 `speaker`（ECAPA / CAM++）。features 写 `default-features=false` 再加 `std`、`download-binaries`，**还要选一个 TLS feature**（如 `tls-rustls`，biglinux 即 `ort-sys/tls-rustls`，`biglinux-gtcrn/ladspa/Cargo.toml:25`）和需要的 `api-NN`（默认是 `api-31`，`resources/ort/Cargo.toml` `[features] default`）——否则下载或较新 API 可能不可用【推断】；或者链自编的最小 ORT（见 biglinux 的 `required_ops.config`） |
| 风险 | ① 仍是 rc，API 每个 rc 都可能有破坏性改动；要锁 `=2.0.0-rc.15`。② 默认从 cdn.pyke.io 下载预编译库（`ort-sys/build/download/dist.tsv:2`，带 sha256），发布时要把来源和哈希写进构建说明；要可复现的话，就自己编最小的静态 ORT。③ ORT 内部的分配和锁我们看不见，只能实测，见「存疑」 |
| 证据 | 【源码】stack.rs、run.rs、run_options.rs、value/mod.rs、create.rs、impl_config_keys.rs、Cargo.toml、dist.tsv ·【API】GitHub releases/issues、crates.io（ort 2.0.0-rc.15，2026-10-19）·【官方】https://ort.pyke.io/ （仓内 `docs/content/perf/io-binding.mdx`、`setup/linking.mdx:13-14`）·【推断】ORT 传递依赖的许可证 |

### biglinux-gtcrn（https://github.com/biglinux/gtcrn-ladspa，fork 自 Xiaobin-Rong/gtcrn）【深读 · 重点】
| 项 | 内容 |
|---|---|
| 做什么 | 用 Rust、ort 和 realfft 写的 LADSPA 插件，逐帧（16 ms hop）跑 GTCRN 流式 ONNX，带 VAD 门限和双模型混合 |
| 许可证 | MIT（`ladspa/Cargo.toml:6`），**兼容：是**。依赖有 ort rc.11、realfft、ladspa 0.3（打过补丁），都是 MIT 或 Apache 系 |
| 维护状态 | 没发过版本；最近提交 2026-04-27（「Audit fixes: security, RT-safety…」）；open issues 0 个。勉强算在 12 个月以内。**只支持 Linux 和 LADSPA**，macOS 用不了这个插件，但 `model.rs` 跟平台无关，可以拿来参考 |
| 实时安全做法 | **推理直接在 LADSPA 的 run 回调里同步做**（`src/plugin.rs:640` 的 `run` → `:490` 调 `run_primary` → `:378-385` 调 `process_frame`），没有另开线程。状态缓冲在初始化时一次性分配（`model.rs:111-121`），出错时回落成原声，不 panic（`plugin.rs:383` 用 `unwrap_or(self.spectrum_buffer)`；`model.rs:177-187` 建会话失败就返回 None）。**但每帧有分配**：4 个 `TensorRef::from_array_view`（`model.rs:278-286`），ORT 每次新分配 4 个输出（`:288-298`）；另有 `spectrum_pool` 池，池空了才分配（`plugin.rs:441-449`） |
| 值得学的一处 | ① **状态接口**：3 份缓存的形状写成常量（`model.rs:25-27`，`283-286` 处用 `[2,1,16,16,33]`、`[2,3,1,1,16]`、`[2,1,33,16]`），和上游导出的接口完全对上（`resources/gtcrn/stream/gtcrn_stream.py:321-326`）。② **预热和冷启动**：开机先喂 20 帧合成的粉红噪声，把 GRU 状态从全零推开，避免一开始过度抑制（`model.rs:30-33`、`226-239`）；还会「记下预热后的参考状态，检测到起音时拉回」（`:130-149`、`:264-271`）。我们换模型、或推理进程崩溃重启时，正好用得上。③ **onnx 转 .ort，再按 `required_ops.config` 编最小 ORT**（`build.rs:1-4`，`required_ops.config` 第 4 行列出 GTCRN 用到的全部 32 个算子） |
| 我们怎么用 | **抄思路自己写**，用在 `engine`：状态常量、预热、出错回落原声。不直接依赖它（LADSPA 加 Linux） |
| 风险 | ① 推理放在音频回调里，跟我们的两进程架构是反着的，只能借鉴模型那一层。② 起音重置用的阈值（`model.rs:39-61`）是针对 DNS3 权重调的，不能直接照搬。③ build.rs 要用 Python 把 ONNX 转成 .ort（`build.rs` 约 90-100 行），我们的 CI 里得另配 |
| 证据 | 【源码】model.rs、plugin.rs、build.rs、required_ops.config、Cargo.toml、gtcrn_stream.py ·【API】GitHub（MIT，pushed 2026-04-27，issues 0，fork 自 Xiaobin-Rong/gtcrn） |

### DeepFilterNet · libDF（https://github.com/Rikorose/DeepFilterNet）【深读 · 重点】
| 项 | 内容 |
|---|---|
| 做什么 | DeepFilterNet 的 Rust 库：STFT、ERB 特征、用 tract-pulse 做流式推理，另有 LADSPA、C API、WASM 几种封装 |
| 许可证 | `MIT/Apache-2.0`（`libDF/Cargo.toml:8`），**兼容：是**。依赖有 tract 0.21、rustfft、realfft、ndarray，都是 MIT 或 Apache 系；可选的 hdf5 只在训练用的 feature 里 |
| 维护状态 | 最新 release v0.5.6，2023-08-31；crates.io 上的 deep_filter 是 0.2.5（2022-07-28）；最近提交的作者日期 2024-09-25、提交日期 2024-10-17（本地 git log），push 是 2024-10-17；open issues 67 个。最近 5 个 issue（2026-09 到 10 月）**都没有维护者回复**。**不满足命门 3（近 12 个月没动静）** |
| 实时安全做法 | **好的一面**：FFT scratch 和 analysis/synthesis_mem 在初始化时分配（`libDF/src/lib.rs:119-122`），逐帧用 `process_with_scratch`（`:385-388`）。**不好的一面**：`frame_analysis` 每帧调 `make_input_vec()`，会新分配（`lib.rs:360`），`frame_synthesis` 每帧 `make_output_vec()`（`lib.rs:397`）；`process` 每帧对 Tensor 做两次 `clone()`（`tract.rs:541-542`），还有一次 `permute_axes`（`:462`）和 `Tensor::zero`（`:492`）；有 `log::warn!`（`:527`）。**LADSPA 封装**：音频回调里 `Mutex::lock` 共享的 VecDeque（`ladspa/src/lib.rs:410-416`），然后在音频回调里**循环 `lock` + `sleep` 轮询**等输出（`:419-432`，不是纯忙等，而是在实时回调里睡眠，更糟）；回调里还有 `control_tx.send`（`:405` 前后）；worker 线程也靠 sleep 轮询（`:141-144`） |
| 值得学的一处 | `tract.rs:769-799`：在时间轴上声明一个符号 `S`，`PulsedModel::new(&m, s, &1.to_dim())` 把整段序列的模型**自动改写成每次 1 帧的有状态模型**。GRU 和卷积的缓存藏在 tract 的 OpState 里，不用像 GTCRN 那样手动导出缓存。对我们来说：如果哪天想用整段导出的 ONNX，又不想手写缓存接口，这就是另一条路（前提是 tract 能把那张图转成 pulse 版本） |
| 我们怎么用 | **抄思路自己写**：`dsp` crate 的 STFT 状态机（照它的布局，但把 input buffer 也预分配）。**反面教材**：LADSPA 里用 Mutex 加 VecDeque 连音频线程和 worker，我们换成 SPSC 环形缓冲加原子发布。**不要**直接依赖（不维护了，tract 版本也旧） |
| 风险 | 不维护；tract 卡在 0.21，和新版 tract 的 API 差异不小；直接依赖的话等于要我们自己维护 |
| 证据 | 【源码】libDF/Cargo.toml、lib.rs、tract.rs、ladspa/src/lib.rs ·【API】GitHub（push 2024-10-17，issues 67，最近 5 个 issue 无维护者回复）、crates.io（deep_filter 0.2.5，2022-07-28） |

### tract（https://github.com/sonos/tract）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | Sonos 的纯 Rust 推理引擎，支持 ONNX、NNEF、TF；能把模型「脉冲化」（pulse）成流式；有 Metal、wgpu、CUDA 后端 |
| 许可证 | MIT 或 Apache-2.0 双许可（`resources/tract/LICENSE:1-6`；GitHub 显示 NOASSERTION，是因为仓里有多份 LICENSE 文件），**兼容：是**。纯 Rust，没有 C++ 依赖 |
| 维护状态 | 最新 v0.23.9，2026-10-15（同一天还发了 0.22.5 和 0.21.19 两个回补版）；最近提交 2026-10-30；open issues 157 个（大多是维护者自己开的 PR）。非常活跃 |
| 实时安全做法 | **热路径会分配**：`SimpleState::run` 收 `TVec<TValue>`、返回 `TVec<TValue>`（`core/src/plan.rs:607-608`、`615-635`）；执行时每个节点都 `inputs.push(prec.clone())`，TValue 是 Arc，clone 不分配，但 TVec 超过内联容量时会分配（`:682-692`）；每个算子 eval 都新建输出张量，比如卷积是 `Tensor::uninitialized`（`core/src/ops/cnn/conv/blocked.rs:118`）。没设 executor 时单线程执行（`plan.rs:647-653`），不会凭空起线程池。CPU 路径**没有张量内存池**，只在 `gpu/src/memory/pool.rs` 里有 |
| 值得学的一处 | GTCRN 的 32 个算子在 `onnx/src/ops/` 里**全部登记了**（包括 GRU：`rec.rs:10`；ScatterND：`array/mod.rs:42`；ConvTranspose、PRelu）。只是登记，数值对不对、能不能 pulse 化，都没验证。`core/src/ops/gru_seq.rs:16-58` 把整段 GRU 融合成一个算子 |
| 我们怎么用 | **备选，直接依赖**（engine）。最便宜的 A/B 是走 `ort-tract`：调用 API 不变，只换后端（`resources/ort/docs/content/backends/tract.mdx:8-31`，支持 `Session::run_with_options`）。**复核补充**：同一份支持清单里 `SessionBuilder` 只列了 `with_optimization_level` 等少数方法，**没有列 `RunOptions`/`OutputSelector`、`with_intra_threads`**——即下面「奇偶帧预分配双缓冲」在 ort-tract 下多半用不上，A/B 时要退回每帧建输入的写法【推断：清单未列 ≠ 一定不支持，需实测】 |
| 风险 | ① 每帧分配是结构性的，改不了。好在推理在单独的进程或线程里，不是音频回调，可以接受，但抖动要实测。② GTCRN 导出图里有 ScatterND 写切片，tract 的 ScatterND 实现我们没读，正确性要和 Python 对拍。③ 0.21、0.22、0.23 API 变化大，要锁版本 |
| 证据 | 【源码】LICENSE、plan.rs、blocked.rs、onnx/src/ops/*、gru_seq.rs、README.md:110-123 ·【API】GitHub、crates.io（tract-onnx 0.23.9，2026-10-15） |

### nnnoiseless（https://github.com/jneem/nnnoiseless）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | RNNoise 的纯 Rust 移植（GRU 加频带增益），模型内嵌 |
| 许可证 | BSD-3-Clause（`Cargo.toml:6`），**兼容：是**。FFT 用 easyfft 0.4（MIT 或 Apache-2.0，crates.io 查的） |
| 维护状态 | 0.5.2，2025-12-18；最近提交 2026-09-28；open issues 4 个；最近几个 PR 维护者都回了 |
| 实时安全做法 | **栈数组**：GRU 的中间量 `[0.0; MAX_NEURONS]`（`src/rnn.rs:5`、`293-295`），dense 层的 buffer 也是栈数组（`:346-347`）；`process_frame` 里的增益也全是栈数组（`src/denoise.rs:95-98`）。2026-09 的提交 #41 是「Replace most Vecs with arrays」，作者在有意清掉分配 |
| 值得学的一处 | `rnn.rs:292-330`：GRU 状态由调用方拥有（`&mut [f32]`），计算全在栈上，是**手写小网络零分配**的范本。如果哪天要把 VAD 或某个小网络从 ONNX 换成手写，就照这个写 |
| 我们怎么用 | **只看看**（降噪质量不如 GTCRN，也不在主线）；做 VAD 兜底时可以参考 |
| 风险 | 提交 #40（标题「Replace `CommonState` and `common()` with `LazyLock`」）改用了 `LazyLock`：复核已确认用在窗、窗归一化系数和 DCT 表三处（`src/lib.rs:99-115`）。第一次访问会做初始化（std `LazyLock` 首次初始化期间其他线程会阻塞【推断】），如果第一次访问发生在音频线程上就有风险，必须在初始化时先碰一下。另：FFT 走 easyfft 的线程局部缓存，每个新线程首次调用会分配（见 dsp 卡） |
| 证据 | 【源码】Cargo.toml、rnn.rs、denoise.rs ·【API】GitHub commits（2026-09-27/28）、issues、crates.io（0.5.2，2025-12-18） |

### rten（https://github.com/robertknight/rten）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 的 ONNX 推理引擎（要先转成 .rten 格式） |
| 许可证 | MIT OR Apache-2.0，**兼容：是** |
| 维护状态 | 0.27.0，2026-10-02；push 2026-10-28；open issues 49 个；最近的 issue 维护者都回了 |
| 实时安全做法 | 有 `BufferPool`，在推理时复用大缓冲，但池子用 `Mutex`，小于阈值的分配仍走全局分配器（`src/buffer_pool.rs` 第 2 行、113-128 行，通过 GitHub API 读到，**不在 resources/ 里**） |
| 值得学的一处 | 有 `ops/rnn.rs`（GRU，约 159 行起），也有 `ops/scatter.rs`。作为第三个纯 Rust 候选，可以留着 |
| 我们怎么用 | **只看看** |
| 风险 | 要转成私有格式，又多一道对拍；没有 pulse 机制 |
| 证据 | 【API】GitHub contents/repo、crates.io ·【推断】GTCRN 能不能跑通 |

### burn / burn-onnx（https://github.com/tracel-ai/burn）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | Rust 深度学习框架。`burn-onnx` 在构建时把 ONNX **生成成 Rust 代码** |
| 许可证 | MIT OR Apache-2.0，**兼容：是** |
| 维护状态 | 稳定版 0.22.1（2026-10-20），预发布 0.23.0-pre.1（2026-10-29）；burn-onnx 2026-10-30 还有 push；open issues 184 个（大量是 PR） |
| 实时安全做法 | 没读。按张量框架的通常做法，每个算子会分配输出（推断） |
| 值得学的一处 | burn-onnx 的 `SUPPORTED-ONNX-OPS.md` 标明 GRU、ScatterND、ConvTranspose、PRelu 都支持（通过 GitHub API 读到） |
| 我们怎么用 | **不要**（框架太重；导出路径多一层代码生成） |
| 风险 | 版本迭代快，API 不稳 |
| 证据 | 【API】GitHub、crates.io、burn-onnx/SUPPORTED-ONNX-OPS.md |

### candle / candle-onnx（https://github.com/huggingface/candle）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | Hugging Face 的极简 Rust ML 框架，主攻 LLM；candle-onnx 是一个 ONNX 解释器 |
| 许可证 | MIT OR Apache-2.0，**兼容：是** |
| 维护状态 | 0.11.0，2026-06-26；push 2026-10-30；open issues 959 个 |
| 实时安全做法 | 没读 |
| 值得学的一处 | 没有 |
| 我们怎么用 | **不要**：candle-onnx 的 `eval.rs` 里**没有 "GRU"，也没有 "ConvTranspose"**，只有 LSTM 和 RNN（通过 API 拉下 eval.rs 用 grep 查的），GTCRN 跑不起来 |
| 风险 | — |
| 证据 | 【API】GitHub contents（candle-onnx/src/eval.rs）、crates.io |

### wonnx（https://github.com/webonnx/wonnx）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 基于 wgpu 的 ONNX 推理 |
| 许可证 | MIT OR Apache-2.0 |
| 维护状态 | **仓库已归档**（archived=true）；最后 push 2024-07-21；0.5.1，2023-09-30 |
| 实时安全做法 | — |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（不满足命门 3） |
| 风险 | — |
| 证据 | 【API】GitHub、crates.io |

### fastenhancer（https://github.com/aask1357/fastenhancer）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | FastEnhancer 官方仓（ICASSP 2026），PyTorch 训练，导出 ONNX |
| 许可证 | MIT，**兼容：是**（权重的许可证另算，归 D33 那条线核） |
| 维护状态 | 最近提交 2026-10-12；最近的 release 有 onnx-48khz-convstft-v0.1.0（2026-04-06，b/l/m/s/t 五个 .onnx）、ckpt-dereverb-v0.1.0（2026-07-03）；**复核补充：更早的 onnx-dns-v1.0.0（2025-10-10）与 onnx-vd-v1.0.0（2025-09-19）里都有 `fastenhancer_m.spec.onnx`（spec2spec）和 wav2wav 版，vd 那组还附带 `gtcrn.spec.onnx`**【API：releases 资产名】；open issues 4 个 |
| 实时安全做法 | **仓里没有 Rust**（find 没找到 .rs 文件）。流式做法：wav2wav 版本把 STFT 也导进图里，输入输出缓存的长度是 n−h（`docs/docs/onnx.md:7-8`、`63-79`） |
| 值得学的一处 | 有两种导出：spec2spec（STFT 留在外面）和 wav2wav（`onnx.md:7-8`）。**我们要选 spec2spec**，这样和 GTCRN 共用 `dsp` 里的 STFT，「同接口」才成立 |
| 我们怎么用 | 只取它的 ONNX 发布物；推理走 ort（engine） |
| 风险 | ~~只有 48 kHz 发布物~~ **复核更正**：DNS / VoiceBank-DEMAND 两组发布物大概率是 16 kHz（这两个数据集在该论文里按 16 kHz 用【推断：未读论文或模型输入形状】），且有现成的 `fastenhancer_m.spec.onnx`，「与 GTCRN 同接口（16 kHz、spec2spec）」很可能不必自己导出——但 STFT 参数（窗长/帧移）未必与 GTCRN 的 512/256 相同，要读 onnx.md 或模型输入形状确认 |
| 证据 | 【源码】docs/docs/onnx.md、README.md:20-22 ·【API】GitHub releases |

### Xiaobin-Rong/gtcrn（https://github.com/Xiaobin-Rong/gtcrn）【浅读 · 接口来源】
| 项 | 内容 |
|---|---|
| 做什么 | GTCRN 官方仓；`stream/` 下是流式改写和 ONNX 导出 |
| 许可证 | MIT，**兼容：是** |
| 维护状态 | 最近提交 2026-08-03；open issues 9 个 |
| 实时安全做法 | 不涉及（Python） |
| 值得学的一处 | `stream/gtcrn_stream.py:321-326` 写明了三份缓存的形状；`:261-263` 用切片原地写 conv_cache，导出 ONNX 后就成了 ScatterND。这是 CoreML 不支持、tract 要专门验的那个算子的来历 |
| 我们怎么用 | 接口的真相源（engine 的状态常量照这里定） |
| 风险 | — |
| 证据 | 【源码】stream/gtcrn_stream.py ·【API】GitHub |

---

## 三个问题的回答

**1. GTCRN 流式 ONNX 在 Rust 里，首选 ort 还是 tract？首选 ort。**
- 有现成的实证：biglinux-gtcrn 用 ort 在 Rust 里逐帧跑通，状态接口也对上了（【源码】model.rs:278-302）。
- 分配可控：Rust 侧走栈，输出可以预分配（【源码】stack.rs:13-21、run_options.rs:95-126）。tract 每个节点 eval 都分配，这是结构性的（【源码】plan.rs:682-692、blocked.rs:118）。
- ScatterND、GRU 这类算子，ORT 是 ONNX 的参考实现，对拍风险最低（推断）。
- tract 的位置是：纯 Rust、构建简单、没有 C++ 二进制下载。经 `ort-tract` 换后端，一行配置就能 A/B，所以留作备选。

**2. 逐帧推理怎么避免每帧分配？**（ort，engine crate）
1. 初始化时用 `Tensor::<f32>::new(&Allocator::default(), shape)` 建好：输入谱 1 份，状态 A、B 各 3 份，增强输出 2 份。
2. 建两个 `RunOptions`：奇数帧 `preallocate` 把输出写进状态 B 和 out_B，偶数帧写进状态 A 和 out_A（【源码】run_options.rs:125-126、138）。
3. 每帧用 `extract_tensor_mut` 把谱写进输入 Tensor（【源码】src/value/impl_tensor/extract.rs:427；`:258` 是带 `Result` 的 `try_extract_tensor_mut`），再用 `run_with_options(ort::inputs![&spec, &state_x...], &opts_parity)` 跑（【源码】session/input.rs:39 支持传 `&Value`）。**不调用** `TensorRef::from_array_view`（会新建 OrtValue，【源码】create.rs:152-189）。
4. 会话配置：`with_intra_threads(1)`、`with_inter_threads(1)`、`with_intra_op_spinning(false)`、Level3 优化、固定输入形状（【源码】impl_config_keys.rs:127-137；impl_options.rs:52、66）。`with_flush_to_zero()`（impl_config_keys.rs:47-56）**复核改为待实测**：intra=1 时是否作用于推理线程未证实，且有 DeepFilterNet #713 式 0/0 → NaN 的风险。
5. 预热 20 帧，同时让 ORT 的 arena 走完首次分配（【源码】biglinux model.rs:30-33；ORT 稳态下不再分配是推断，要实测）。
6. 这条推理线程不是音频回调：它和送音进程之间只走 SPSC 和共享内存。所以即使 ORT 内部偶尔分配，最坏也只是这一帧晚到、走原声旁路，不会爆音（D03）。

**3. CoreML / ANE 对耗电的影响（官方资料）**
- 【官方】Apple 称 ANE 是「Apple silicon 上用于 ML 推理的高能效、高吞吐引擎」，在 iPhone 13 上量过某个 Transformer 的平均延迟和功耗：3.47 ms 时 0.454 W，9.44 ms 时 0.072 W（https://machinelearning.apple.com/research/neural-engine-transformers ）。这是另一类模型、另一台设备，**不能直接套到 GTCRN 上**。
- 【官方】ONNX Runtime CoreML EP 文档：`MLComputeUnits` 有 CPUOnly、CPUAndNeuralEngine、CPUAndGPU、ALL 四档；CPUAndNeuralEngine「不保证整个模型都在 ANE 上跑」；动态形状会降性能，所以要设 `RequireStaticInputShapes=1`；不设 `ModelCacheDirectory` 的话，编译可能要几分钟；`ProfileComputePlan` 能看每个算子落在哪个硬件上。**算子表里没有 GRU，也没有 ScatterND**，ConvTranspose 只在 MLProgram 格式下支持（https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html ）。
- 【源码】ort 已经封装了这些选项（`resources/ort/src/ep/coreml.rs` 里的 ComputeUnits、ModelFormat、SpecializationStrategy）；pyke 的 aarch64-apple-darwin 预编译包自带 coreml（`ort-sys/build/download/dist.tsv:2`）。
- 【推断】GTCRN 每帧只有几十 MFLOPs，而 GRU 和 ScatterND 只能回落到 CPU，图会被切成很多 CPU↔ANE 来回跳的分段，每帧都要付调度和拷贝的开销，很可能比纯 CPU 单线程**更费电、延迟抖动也更大**。**结论**：默认 CPU EP；CoreML 只做一次有界实验，用 ProfileComputePlan 加 powermetrics 量。要不要做这个实验，可以列进待人拍板。

---

## 没查到 / 存疑

- **ORT 稳态下 Run 内部会不会分配、会不会加锁**：源码在 C++ 里，没读，Rust 侧看不出来。要用实测定：在推理线程上挂 malloc 计数（DYLD interpose 或 Instruments 的 Allocations），预热以后看每帧分配次数。
- **tract 能不能把 GTCRN 流式图加载起来、数值对不对**：只查了算子登记表，没验 ScatterND 和 GRU 的实现；也没确认 0.23 能不能直接吃 biglinux 或官方导出的 `gtcrn_simple.onnx`。需要一次有界的对拍小实验（拿到重活名额以后再做）。
- **ort 的 preallocate 用在「状态既当输入又当输出」时的别名问题**：双缓冲方案特意避开了同一块内存同时当输入和输出，但 ORT 是否禁止或者容忍这种别名，没查到文档。
- **ORT 预编译包的许可证清单**（ThirdPartyNotices）：没逐项核，结论「整棵树兼容」目前是推断。
- **nnnoiseless 提交 #40 的 LazyLock 具体用在哪**：没读 diff。
- **rten 的 GRU 和 ScatterND 实现**：只看到文件名。
- **FastEnhancer-M 的 16 kHz ONNX**：~~发布物里只看到 48 kHz 的那一组~~ 复核发现 onnx-dns-v1.0.0 / onnx-vd-v1.0.0 里有 `fastenhancer_m.spec.onnx`；其采样率与 STFT 参数是否与 GTCRN 对齐，交给 D33 那条线确认。
