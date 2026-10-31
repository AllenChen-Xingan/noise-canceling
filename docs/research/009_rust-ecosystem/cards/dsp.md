# 009 · Rust 生态调研 · DSP 组项目卡

> 2026-10-30 · 子代理产出（AI 调研，待人过目）。证据路径相对 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。
> 只读代码、git log、GitHub API、crates.io API；**没有跑任何 cargo build / test / 推理**，所有性能与延迟数字都是读代码或官方文档得来，需实测。

## 本组覆盖

**深读（读了热路径源码）**：rubato · realfft · RustFFT · sonora（WebRTC 纯 Rust 移植，含 AGC2 限幅器与 RNN VAD）· DeepFilterNet libDF 的 STFT · nnnoiseless（及其依赖 easyfft）· webrtc-audio-processing（tonarino 绑定）
**中读**：Accelerate/vDSP 绑定（accelerate-rs / apple-sys / accelerate-src）· biquad · ebur128
**浅读（只看 API 与官方信息）**：dasp · fundsp · hound · symphonia · phastft / microfft · 现成 STFT crate（stft / ruststft / stft-rs）

## 推荐（每个子需求一行）

| 子需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| FFT（GTCRN 的 512 点实 FFT；fbank 也用） | **realfft 3.5.1**（底下 RustFFT，默认开 NEON） | 自己写几行 `extern "C"` 调系统 vDSP_DFT（不用第三方绑定） | realfft 有 `process_with_scratch`，热路径零分配；MIT，GPL-3.0 兼容；RustFFT 在 aarch64 运行时检测 NEON 并自动选 NEON 规划器。vDSP 只在实测 realfft 不够快时再上 |
| STFT / iSTFT 封装（窗 512、帧移 256、sqrt-Hann） | **自己写**（放 `dsp` crate），思路参考 libDF `frame_analysis/frame_synthesis` | —— | 现成 STFT crate 都不贴我们的「逐帧流式 + 预分配」需求；libDF 的写法清楚但每帧有两次分配（见卡），照思路改成预分配即可，约百行 |
| 重采样 48 k ↔ 16 k（整数比 3） | **rubato 5.0.1 `Fft`（同步）**，用 `new_custom` 把子块调小压延迟 | rubato `Async`（sinc，`FixedAsync`） | 都有 `process_into_buffer` 预分配接口；`Fft` 延迟 = `fft_size_out/2`，整数比下块可以很小；sinc 版延迟 = `sinc_len × 比率 / 2`，便于兼顾时钟漂移。两者延迟要在 45 ms 预算里实测选 |
| 时钟漂移（真麦时钟 vs 虚拟麦时钟） | **rubato `Slip`**（几 ppm 级「打滑离合器」，偶尔插/丢一帧 + 交叉淡化） | rubato `Async` + `set_resample_ratio_relative` | 送音进程两个设备时钟必漂；Slip 几乎不加延迟、无重采样滤波 |
| 限幅 / 轻压缩 | **照 sonora AGC2 limiter 的思路自己写** | 直接依赖 sonora-agc2（接受每帧一次小分配，或提 PR 去掉） | 子帧电平估计 + 插值增益曲线 + 攻击段幂次插值，设计成熟；但 `Limiter::process` 每次分配一个 `Vec`（见卡），不能原样进实时线程 |
| 双二阶滤波（高通去直流、音色链 EQ） | **biquad 0.6**（no_std、无分配） | 自写 DF2T（十来行） | `run()` 纯标量运算，无分配无锁 |
| 离线 WAV 读写（eval） | **hound 3.5** | symphonia 0.6（MPL-2.0，可兼容） | 只在离线评测与命令行用，不进实时线程 |
| 响度测量（eval 指标、音色链调参） | **ebur128 0.1.11** | Python pyloudnorm（resources/ 已有）对拍 | MIT；只用在 eval，不进实时线程 |
| 传统降噪 / VAD 的对照基线 | **sonora**（WebRTC NS + AGC2 RNN VAD，纯 Rust） | nnnoiseless（RNNoise，带 vad_prob） | 给 eval 当「传统方法」参照，也可当门控里的快速 VAD 候选（VAD 选型归别组） |

## 本组最值得学的两处设计

1. **rubato 的「分配版 / 预分配版」双接口**：`process()` 只是在内部 new 一个输出缓冲再调 `process_into_buffer()`（resources/rubato/src/lib.rs:213-230、263-268）。我们的 `dsp`/`engine` 每个处理器都照这个套路：一个给离线 eval 用的方便接口，一个给实时线程用的预分配接口，**两者走同一份核心代码**——正好满足「离线评测与实时回调用同一份处理代码」。
2. **sonora AGC2 限幅器的「子帧增益 + 攻击段幂次插值」**：每 10 ms 帧切成若干子帧，各算一个增益，样点间线性插值；若是攻击（增益下降）则第一子帧用 `(1-t)^8` 快速压下，防止过冲（resources/sonora/crates/sonora-agc2/src/limiter.rs:12-23、34-40）。没有前瞻、零附加延迟，正合 D32。我们音色链末端的限幅器直接按这个思路写。

---

### rubato（https://github.com/HEnquist/rubato）
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 音频重采样：同步 FFT 型（`Fft`）、异步 sinc/多项式（`Async`）、时钟漂移用的「打滑」型（`Slip`） |
| 许可证 | MIT OR Apache-2.0；**与 GPL-3.0 兼容：是**（两者择一均可并入 GPL-3.0）。依赖 realfft(MIT)、num-*、audioadapter(MIT/Apache)、windowfunctions，未见不兼容项 |
| 维护状态 | 最新 5.0.1（2026-10-01）；最近提交 2026-10-01；open issues 0；最近 issue #142（2026-09-29 用户报 bug）当天由维护者修复并发版（#143/#144）——响应很快 |
| 实时安全做法 | `process_into_buffer` 写入调用方预分配的缓冲（resources/rubato/src/lib.rs:263-268）；`Fft` 的 FFT 用构造时备好的 scratch 调 `process_with_scratch`（resources/rubato/src/synchro.rs:123-124、156-157、180-184）；内部输入/输出暂存在 `new_custom` 里一次分配（synchro.rs:329、350-353）；热路径里只有 `copy_from_slice`/`copy_within`。声道掩码用 `copy_from_slice` 写进已有数组，不分配（lib.rs:169-183）。日志宏不开 `log` feature 时编译为空（lib.rs:14-18）——**开了 `log` feature 的 trace 会格式化 Debug，实时线程务必别开** |
| 值得学的一处 | `Slip`：两时钟差几 ppm 时偶尔插/丢一帧，并用最长 128 帧的交叉淡化把一帧位移摊开，说明里论证了为何几十帧就听不出（resources/rubato/src/slip.rs:9-30）。送音进程「真麦 → 虚拟麦」正好是这个场景，比完整重采样省延迟省电 |
| 我们怎么用 | **直接依赖**：`io-macos`（48k↔16k、漂移补偿）与 `dsp`（离线 eval 同一份重采样）。默认开 `fft_resampler`，不开 `log` |
| 风险 | ① 延迟：`Fft` 延迟 = `fft_size_out / 2` 个输出帧（synchro.rs:692-694），默认按约 256 帧切子块（synchro.rs:221-222）。按 synchro.rs:298-319 的算法推算：**上采样 16k→48k 约 384 个输出帧 ≈ 8 ms；下采样 48k→16k 约 43 个输出帧 ≈ 2.7 ms**（`FixedSync::Input`，chunk ≥ 256 时）【推断：复核者按源码手算，未实测】，往返合计约 11 ms，偏多；需 `new_custom` 调小子块（整数比 3 时最小块 3:1，可以很小，synchro.rs:298-319），代价是抗混叠截止变低。sinc 版延迟 = `sinc_len × 比率 / 2`（asynchro.rs:747-749）。② 5.x 刚换成 audioadapter 接口，API 近期变动多，锁版本 |
| 证据 | 【源码】lib.rs:14-18、169-183、213-230、263-268；synchro.rs:123-124、156-157、180-184、221-222、298-319、329-353、692-694；asynchro.rs:747-749、806-820；slip.rs:9-30 ·【API】crates.io 5.0.1/2026-10-01；GitHub pushed 2026-10-01、issues #142-#144 |

### realfft（https://github.com/HEnquist/realfft）
| 项 | 内容 |
|---|---|
| 做什么 | 在 RustFFT 上包一层实数 ↔ 复数 FFT（N 点实信号 → N/2+1 个复数），省一半算力 |
| 许可证 | MIT；**兼容：是**。唯一依赖 rustfft（MIT OR Apache-2.0）。注：3.5.1 才补上缺失的 LICENSE 文件（issue #48/#49，2026-10-26）；#45「改为 MIT+Apache 双许可」仍 open，对我们无影响 |
| 维护状态 | 最新 3.5.1（2026-10-26）；最近提交 2026-10-26；open issues 2；#48 用户提问当天维护者处理并发版——快 |
| 实时安全做法 | 两套接口：`process()` 内部每次 new scratch（resources/realfft/src/lib.rs:128-131、311-313）——**实时线程禁用**；`process_with_scratch()` 用调用方给的 scratch（lib.rs:138-143），scratch 由 `make_scratch_vec()`/`get_scratch_len()` 在初始化时备好。输入缓冲会被当草稿改写（lib.rs:128 文档）|
| 值得学的一处 | 逆变换遇到 DC / Nyquist 处虚部非零时，**先把虚部清零照常算完，再返回 `Err(InputValues)`**（lib.rs:671-682、752-754）。神经网络掩码乘出来的频谱几乎必然带非零虚部，所以我们的 iSTFT 必须把这个错误当「已处理的警告」而不是 `unwrap()` 崩掉——libDF 就是这么做的（见下） |
| 我们怎么用 | **直接依赖**：`dsp`（STFT/iSTFT、fbank 的 FFT），供 `engine`、`speaker`、`eval` 共用 |
| 风险 | 维护者单人（HEnquist，同 rubato）；`Err` 不能 unwrap；features 与 RustFFT 一一映射，默认已开 neon（resources/realfft/Cargo.toml `default = ["rustfft/default"]`） |
| 证据 | 【源码】realfft/src/lib.rs:128-143、311-313、671-682、752-754；realfft/Cargo.toml features 段 ·【API】crates.io 3.5.1/2026-10-26；issues #45 #48 #49 |

### RustFFT（https://github.com/ejmahler/RustFFT）
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 任意长度复数 FFT，含 AVX/SSE/NEON/FCMA/WASM SIMD |
| 许可证 | MIT OR Apache-2.0；**兼容：是**。依赖 num-complex、num-traits、num-integer、strength_reduce、transpose、primal-check，均为 MIT/Apache 类 |
| 维护状态 | crates.io 最新 6.4.1 是 2025-09-18（**发布已超 12 个月**），但仓库活跃：最近提交 2026-10-28/29（NEON/FCMA butterfly 优化、2024 edition）；open issues 17；最近 5 条都是维护者自己或贡献者的 PR，有维护者评论——判「仍在维护」 |
| 实时安全做法 | 同样两套：`process()` 内 `vec!` 分配 scratch（resources/RustFFT/src/lib.rs:224-227），`process_with_scratch()` 不分配（lib.rs:240）。规划器 `FftPlanner::plan_fft` 返回 `Arc<dyn Fft>`（src/plan.rs:95），**规划与 twiddle 计算有分配，只能在初始化时做**；规划结果 `&self` 调用，可跨线程共享 |
| 值得学的一处 | NEON 支持：`FftPlanner::new()` 依次尝试 AVX→SSE→FCMA→NEON→WASM→标量（src/plan.rs:73-87）；NEON 规划器用 `is_aarch64_feature_detected!("neon")` 运行时检测（src/neon/neon_planner.rs:156-157）；默认 features 含 `neon`（Cargo.toml `default = ["avx","sse","neon"]`），`fcma` 需 nightly。Apple Silicon 上开箱即走 NEON |
| 我们怎么用 | **间接依赖**（经 realfft）；不直接用 |
| 风险 | 发布节奏慢（一年多没发版，NEON 优化在 main 上未发布）；不要为了新优化去依赖 git 版 |
| 证据 | 【源码】RustFFT/src/lib.rs:224-227、240；src/plan.rs:73-87、95；src/neon/neon_planner.rs:156-157；Cargo.toml features ·【API】crates.io 6.4.1/2025-09-18；GitHub pushed 2026-10-29、PR #202-#206 |

### Apple Accelerate / vDSP 的 Rust 绑定（accelerate-rs https://github.com/doom-fish/accelerate-rs · apple-sys https://github.com/youknowone/apple-sys · accelerate-src https://github.com/blas-lapack-rs/accelerate-src）
| 项 | 内容 |
|---|---|
| 做什么 | 调用 macOS 系统自带 Accelerate（含 vDSP FFT/DFT）的 Rust 绑定 |
| 许可证 | accelerate-rs：MIT OR Apache-2.0；apple-sys：BSD-2-Clause；accelerate-src：Apache-2.0/MIT。**兼容：是**。Accelerate 本身是系统库，GPL-3.0 的「系统库例外」覆盖【推断】 |
| 维护状态 | accelerate-rs 0.4.2（2026-10-26），仓库首提交也是 2026-10-26（git log 只见一天）、crates.io 下载 227——**极新、单组织、未经检验**。apple-sys 0.3.0（2026-03-07），bindgen 生成的原始绑定。accelerate-src 0.3.2 停在 2018-10，只做 BLAS/LAPACK 链接，**没有 vDSP** |
| 实时安全做法 | accelerate-rs 的安全层只包了复数 `vDSP_fft_zip`，并经一层 Swift bridge 调用（resources/accelerate-rs/src/vdsp.rs:56-68、76-77），构建要 `swift` 工具链（build.rs:15-40）；实数 `vDSP_fft_zrip`、`vDSP_DFT_*` 只在原始 FFI 里（src/ffi/generated/vdsp_missing.rs:418、4473-4483）。vDSP 本身 setup 一次后 execute 不分配【推断，官方文档惯例】 |
| 值得学的一处 | 无特别值得抄的设计；只说明一件事：**要用 vDSP 就自己写 4-5 个 `extern "C"`**（`vDSP_DFT_zrop_CreateSetup` / `vDSP_DFT_Execute` / `vDSP_DFT_DestroySetup`，`#[link(name="Accelerate", kind="framework")]`），比引入一个刚出生、还要 Swift 构建的 crate 稳 |
| 我们怎么用 | **只看看**；若实测 realfft 512 点超预算，在 `dsp` 里加一个 `cfg(target_os="macos")` 的 vDSP 后端，接口与 realfft 后端相同 |
| 风险 | vDSP 的 split-complex 布局与 realfft 的交织布局不同，要做换算；数值与 torch 对拍时注意 vDSP 的 2 倍缩放惯例【推断】 |
| 证据 | 【源码】accelerate-rs/src/vdsp.rs:56-68、76-77；build.rs:15-40；src/ffi/generated/vdsp_missing.rs:418、4473-4483 ·【API】crates.io apple-accelerate 0.4.2 / apple-sys 0.3.0 / accelerate-src 0.3.2(2018)；GitHub doom-fish/accelerate-rs pushed 2026-10-26 ·【推断】系统库例外、vDSP execute 不分配 |

### sonora（https://github.com/dignifiedquire/sonora）
| 项 | 内容 |
|---|---|
| 做什么 | WebRTC AudioProcessing（M145 + 后续修复）的**纯 Rust 移植**：AEC3、噪声抑制、AGC2（含限幅器、RNN VAD）、高通、重采样、Ooura/PFFFT |
| 许可证 | BSD-3-Clause；**兼容：是**。外部依赖 cpufeatures、derive_more、tracing、bytemuck 等（resources/sonora/Cargo.toml workspace.dependencies），均宽松许可 |
| 维护状态 | 最新 0.2.0（2026-07-29）；最近提交 2026-10-09（主干）、2026-10-17 依赖机器人；open issues 9（全是 dependabot PR，无人工 issue 可看响应）。README 自述移植「AI assisted」，并称 2400+ C++ 测试在 Rust 后端通过（README「History」第 4 条）；CI 含 macOS ARM64 构建与测试 |
| 实时安全做法 | 公开 API 是 `&mut self`（resources/sonora/crates/sonora/src/audio_processing.rs:615-637），**没有 C++ 版那样的内部 mutex**（在 sonora/src 下 grep `Mutex/RwLock/.lock()` 无结果）。但并非处处零分配：`Limiter::process` 每次 `collect()` 一个 `Vec<&[f32]>`（crates/sonora-agc2/src/limiter.rs:98）；**复核补反例**：`GainController2::process`（crates/sonora/src/gain_controller2.rs:238）每帧调三次 `extract_channels`（同文件 108-111，把每个声道 `to_vec()` 复制成 `Vec<Vec<f32>>`），再 `collect` 出 `Vec<&mut [f32]>`（313-331），噪声估计处还有一次 `collect`（264）——**AGC2 整条管线每帧多次堆分配，不只 Limiter 一处**。全管线是否零分配**需实测（如 resources/ 里的 rust-assert-no-alloc / rtsan-standalone-rs）** |
| 值得学的一处 | AGC2 限幅器（见文件开头「最值得学」第 2 条，limiter.rs:12-40、93-117）：状态都是定长数组（limiter.rs:68-74），按子帧插值、攻击段幂次插值，零前瞻。另：RNN VAD 内部固定 24 kHz、10 ms 一帧，`analyze()` 先重采样到预分配缓冲再算（crates/sonora-agc2/src/vad_wrapper.rs:131-147；rnn_vad/common.rs:5-6），可作门控「快开」的 VAD 候选 |
| 我们怎么用 | **抄思路自己写**：`dsp` 的限幅器/轻压缩；**直接依赖（仅 eval）**：WebRTC NS 作为传统降噪对照基线、RNN VAD 作对照 |
| 风险 | 0.x、单维护者、AI 辅助移植——数值正确性靠其自报的 C++ 测试；限幅器热路径有分配；MSRV 1.91、edition 2024 |
| 证据 | 【源码】sonora/Cargo.toml；README.md「Supported Platforms」「History」；crates/sonora/src/audio_processing.rs:615-637；crates/sonora-agc2/src/limiter.rs:12-40、68-74、93-117；vad_wrapper.rs:131-147；rnn_vad/common.rs:5-6 ·【API】crates.io 0.2.0/2026-07-29；GitHub pushed 2026-10-17、issues #42-#46 |

### webrtc-audio-processing（https://github.com/tonarino/webrtc-audio-processing）
| 项 | 内容 |
|---|---|
| 做什么 | 对 C++ WebRTC AudioProcessing（PulseAudio 抽出版）的 Rust 绑定，`bundled` feature 现场编 C++ |
| 许可证 | crates.io 显示 non-standard（用 `license-file = "COPYING"`，resources/webrtc-audio-processing/Cargo.toml:25），COPYING 实为 Google 的 BSD-3 文本（COPYING:1-5）；**兼容：是**【推断：BSD-3 + WebRTC 专利授权与 GPL-3.0 无冲突】 |
| 维护状态 | 最新 2.1.0（2026-05-13）；最近提交 2026-07-16；open issues 8；**#103「Panic on MacOS 27」（2026-09-20，bundled 构建在 macOS 27 + rustc 1.98 上 panic）只有报告者自己补充，未见维护者回复**；#102 修 Apple 交叉编译的 PR 仍 open |
| 实时安全做法 | 每次 `process_capture_frame` 都把声道指针 `collect()` 成一个新 `Vec<*mut f32>`（resources/webrtc-audio-processing/src/lib.rs:210-215、388-401）——**热路径有堆分配**；C++ 侧 APM 内部用锁【推断：本地子模块未检出，未亲见】；帧长必须正好 10 ms，否则 panic（lib.rs:208-209 文档） |
| 值得学的一处 | `stream_delay_ms` 用原子量存，`u64` 存不进 `i32` 就表示「未设置」（lib.rs:224-225），一个原子同时表达「值 + 有无」，免锁——我们「原子发布声纹结果」可借这种编码 |
| 我们怎么用 | **不要**（实时侧）；若要 WebRTC 基线，用纯 Rust 的 sonora |
| 风险 | 我们的机器正是 macOS 27（Darwin 27），#103 直接撞上；需要 meson/C++ 工具链；热路径分配 |
| 证据 | 【源码】Cargo.toml:25；COPYING:1-5；src/lib.rs:208-215、224-225、388-401 ·【API】crates.io 2.1.0/2026-05-13；GitHub issues #99-#103 ·【推断】C++ 内部锁、专利授权兼容 |

### nnnoiseless（https://github.com/jneem/nnnoiseless）
| 项 | 内容 |
|---|---|
| 做什么 | RNNoise 的纯 Rust 移植：48 kHz、480 样点（10 ms）一帧，自带 STFT 前端、基音滤波、输出 VAD 概率 |
| 许可证 | BSD-3-Clause；**兼容：是**。核心依赖只有 easyfft（MIT OR Apache-2.0），其余（hound、dasp、hdf5…）都是 optional（resources/nnnoiseless/Cargo.toml:40-52） |
| 维护状态 | 最新 0.5.2（2025-12-18）；最近提交 2026-09-28；open issues 4；2026-09 有外部贡献者一串 PR（「把大部分 Vec 换成数组」#41、LazyLock #40）被合并，有讨论——活跃度中等 |
| 实时安全做法 | `process_frame` 的临时量都是栈上定长数组（resources/nnnoiseless/src/denoise.rs:95-98）；窗、DCT 表用 `LazyLock` 静态初始化（src/lib.rs:99-115）——**首次访问会初始化，要在非实时线程预热**。FFT 走 easyfft：easyfft 用**线程局部**的 `RefCell<FftPlanner>` 和按长度缓存 scratch 的 `HashMap`，首次在某线程调用时分配（【官方】https://github.com/WalterSmuts/easyfft/blob/main/src/lib.rs L57-58、src/dyn_size/realfft.rs L96-113）——**稳态不分配，但每个新线程第一次调用会分配，必须在实时线程上先空跑一帧预热** |
| 值得学的一处 | 帧内「静音就跳过网络」：`compute_frame_features()` 返回 silence 时不跑 RNN、不乘增益，直接合成输出（denoise.rs:101-113；判据在 features.rs:160 的能量阈值）——能耗 D31 的现成省电招：GTCRN 前加一个能量门，静音帧不推理（注意 GRU 状态要按模型要求处理） |
| 我们怎么用 | **只看看 + eval 基线**：作「经典轻量降噪」对照；不进主管线（GTCRN 是 16 kHz、帧移 256，帧结构不同） |
| 风险 | 48 kHz 固定；easyfft 的线程局部预热坑；easyfft 本身最后发版 2025-05-28 |
| 证据 | 【源码】Cargo.toml:40-52；src/denoise.rs:95-113；src/lib.rs:99-115；src/features.rs:160、264-293 ·【官方】easyfft 源码（经 gh api 读取）lib.rs L57-58、dyn_size/realfft.rs L96-113 ·【API】crates.io nnnoiseless 0.5.2、easyfft 0.4.2/2025-05-28；GitHub PR #38-#42 |

### DeepFilterNet · libDF 的 STFT（https://github.com/Rikorose/DeepFilterNet）
| 项 | 内容 |
|---|---|
| 做什么 | DeepFilterNet 的 Rust 核心库 libDF：STFT/iSTFT、ERB 特征、tract 推理（我们只看 STFT 部分） |
| 许可证 | MIT/Apache-2.0（resources/DeepFilterNet/libDF/Cargo.toml:8）；**兼容：是**（只借思路，不依赖） |
| 维护状态 | 最新 release v0.5.6（2023-08-31）；GitHub pushed 2024-10-17——**超过 12 个月无提交**；open issues 67，最近 5 条（2026-09/10）无维护者回复 |
| 实时安全做法 | 好的一面：FFT scratch 在构造时备好（libDF/src/lib.rs:119-122），调用 `process_with_scratch`（lib.rs:385-388、398-400）；逆变换把 `InputValues` 错误吞掉（lib.rs:401-402），正是 realfft 卡里说的处理法。坏的一面：`frame_analysis` 每帧 `make_input_vec()` 新分配一个缓冲（lib.rs:360），`frame_synthesis` 每帧 `make_output_vec()`（lib.rs:397）；`rotate_left` 移位 analysis/synthesis 记忆（lib.rs:377-379、411-413） |
| 值得学的一处 | 帧分析/合成的结构本身：「上一帧剩余 × 前半窗 + 新输入 × 后半窗 → FFT → 归一化只在分析端做一次」（lib.rs:356-393），合成端加窗 + 重叠相加记忆（lib.rs:396-427）。我们照这个骨架写，把两处每帧分配改成结构体里的预分配缓冲，`rotate_left` 换成环形索引或 `copy_within` |
| 我们怎么用 | **抄思路自己写**：`dsp::stft`（窗改 sqrt-Hann 512/256，对齐 GTCRN 的 `torch.stft(x, 512, 256, 512, hann_window(512).pow(0.5))`，resources/gtcrn/stream/gtcrn_stream.py:371、375） |
| 风险 | 已停更；issue #713（2026-10-28，标题「Output turns to NaN after 78 s of digital silence when flush-to-zero is on」）报告：**开着 flush-to-zero** 时连续约 78 s 纯数字静音后输出变 NaN。报告者给的原因**不是 GRU 状态**，而是特征归一化 `band_unit_norm` 的递推状态每帧 ×0.99 衰减，掉到最小正规数以下被 FTZ 清成 0，随后 `0/sqrt(0)` = NaN；不开 FTZ 时停在次正规数、不出 NaN【API：issue 正文】。教训：①凡是「除以递推估计量」的地方都要防 0；②**开 FTZ 不是免费的**，会把「慢慢趋零」变成「突然等于零」；③engine 仍必须有 NaN/Inf 检查 + 送原声兜底 |
| 证据 | 【源码】libDF/Cargo.toml:8；libDF/src/lib.rs:119-122、356-427；gtcrn/stream/gtcrn_stream.py:371、375 ·【API】GitHub pushed 2024-10-17、release v0.5.6、issue #713 正文 |

### biquad（https://github.com/korken89/biquad-rs）
| 项 | 内容 |
|---|---|
| 做什么 | no_std 的二阶 IIR（DF1 / DF2T），带常见滤波器系数计算 |
| 许可证 | MIT OR Apache-2.0；**兼容：是**；运行时依赖 num-traits（开 `libm` feature，均 MIT/Apache）（resources/biquad-rs/Cargo.toml `[dependencies]`）（中读） |
| 维护状态 | 0.6.0（2026-03-22）；最近提交 2026-03-22；open issues 0；最近 PR 由维护者合并、有评论——低频但在维护 |
| 实时安全做法 | `#![no_std]`（resources/biquad-rs/src/lib.rs:51）；DF2T `run()` 只做 5 次乘加和状态更新（lib.rs:170-181），`update_coefficients` 直接替换系数（lib.rs:183-185），零分配零锁 |
| 值得学的一处 | 系数热替换不清状态（lib.rs:183-185）——音色链「慢慢均衡」改 EQ 时不爆音；但大幅跳变仍需平滑插值系数【推断】 |
| 我们怎么用 | **直接依赖**：`dsp`（高通去直流、音色链 EQ） |
| 风险 | f32 长时间静音下可能出现非正规数（denormal），见文末「存疑」 |
| 证据 | 【源码】biquad-rs/src/lib.rs:51、170-185 ·【API】crates.io 0.6.0/2026-03-22；GitHub issues #17-#21 |

### ebur128（https://github.com/sdroege/ebur128）
| 项 | 内容 |
|---|---|
| 做什么 | EBU R128 响度（短时/瞬时/积分响度、真峰值）测量，libebur128 的 Rust 移植 |
| 许可证 | MIT；**兼容：是**；依赖 bitflags、smallvec、dasp_sample/dasp_frame（均宽松） |
| 维护状态 | 0.1.11（2026-10-26）；最近提交 2026-10-27；open issues 4；#65（2026-10-23 用户报 bug）三天内维护者修复发版（#66-#68）——快 |
| 实时安全做法 | 默认 Queue 历史是 `VecDeque`，上限前会增长（resources/ebur128/src/history.rs:160-178）；Histogram 模式是定长 `Box<[u64;1000]>`（history.rs:52-61）——要在线用只能开 HISTOGRAM 模式。本项目只在 eval 里用，不进实时线程 |
| 值得学的一处 | 非正规数处理：x86 上用 RAII 守卫临时打开 FTZ、离开作用域恢复（resources/ebur128/src/filter.rs:395-431）；**非 x86（含 Apple Silicon）走手动兜底：状态绝对值小于 `f64::MIN_POSITIVE` 就置零**（filter.rs:250-253）。我们的滤波器/GRU 状态在 aarch64 上要自己做同样的事（或在实时线程设 FPCR.FZ）|
| 我们怎么用 | **直接依赖**：`eval`（响度指标、音色链调参） |
| 风险 | 无大风险；f64 计算，离线用无妨 |
| 证据 | 【源码】ebur128/src/history.rs:52-61、160-178；filter.rs:250-253、395-431 ·【API】crates.io 0.1.11/2026-10-26；issues #64-#68 |

### dasp（https://github.com/RustAudio/dasp）— 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 样本类型转换、帧、信号链、环形缓冲、插值（含 sinc） |
| 许可证 | MIT OR Apache-2.0（resources/dasp/LICENSE-*）；兼容：是 |
| 维护状态 | 最新 0.11.0 停在 **2020-05-29**；仓库最近提交 2025-09-09（本地 git log），GitHub pushed 2026-01-12；open issues 68，近期 PR 无人合并——**基本停滞** |
| 实时安全做法 | 未深读 |
| 值得学的一处 | `dasp_sample` 的样本格式互转可参考；ebur128 也依赖它 |
| 我们怎么用 | **不要**直接依赖（重采样用 rubato，格式转换自己写几行） |
| 风险 | 发版停滞 6 年 |
| 证据 | 【源码】dasp/LICENSE-*、git log ·【API】crates.io 0.11.0/2020-05-29；GitHub pushed 2026-01-12、open 68 |

### fundsp（https://github.com/SamiPerttu/fundsp）— 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 函数式音频 DSP 组合库（振荡器、滤波器、限幅器、图结构） |
| 许可证 | MIT OR Apache-2.0；兼容：是 |
| 维护状态 | 0.23.0（2026-01-07）；pushed 2026-03-03；open issues 14；最近 5 个 PR（2026-04 至 09）均 0–1 评论、未合并——响应慢 |
| 实时安全做法 | 未读源码 |
| 值得学的一处 | 未读 |
| 我们怎么用 | **只看看**；抽象层太重，我们只要几个固定节点 |
| 风险 | 维护响应慢；API 变动大 |
| 证据 | 【API】crates.io 0.23.0；GitHub pushed 2026-03-03、PR #93-#97 |

### hound（https://github.com/ruuda/hound）— 浅读
| 项 | 内容 |
|---|---|
| 做什么 | WAV 读写 |
| 许可证 | Apache-2.0；兼容：是（Apache-2.0 与 GPL-3.0 兼容） |
| 维护状态 | 最新 3.5.1 是 **2023-09-25**（发版超 12 个月）；但仓库 2026-08 仍合并修复（#102 修 `get_i16_writer` 整数溢出导致的堆越界 soundness 问题）；open issues 40 |
| 实时安全做法 | 不进实时线程，不评 |
| 值得学的一处 | —— |
| 我们怎么用 | **直接依赖**：`eval` 与离线命令行（P1/P2） |
| 风险 | #101 的 soundness 修复在 main 上、**未发版**；但 #101 标题写明是 **32 位平台**上的整数溢出，aarch64（64 位）基本不触发【API：issue 标题】；仍建议避开 `get_i16_writer` |
| 证据 | 【API】crates.io 3.5.1/2023-09-25；GitHub issues #98-#102 |

### symphonia（https://github.com/pdeljanov/Symphonia）— 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 多格式音频解码（WAV/FLAC/MP3/AAC/Vorbis…） |
| 许可证 | MPL-2.0；**兼容：是**——MPL-2.0 §3.3 允许与 GPL 组合，除非源码带 Exhibit B「Incompatible With Secondary Licenses」声明；抽查 `symphonia/src/lib.rs` 未见该声明 |
| 维护状态 | 0.6.1（2026-08-13）；pushed 2026-08-13；open issues 121；最近 issue 当天有回复 |
| 实时安全做法 | 不进实时线程 |
| 值得学的一处 | —— |
| 我们怎么用 | **备选**：eval 需要读 FLAC/MP3 评测素材时再加 |
| 风险 | 依赖树较大 |
| 证据 | 【API】crates.io 0.6.1；GitHub LICENSE 与 lib.rs 内容（gh api 读取）；issues #601-#605 ·【推断】MPL 组合条件 |

### 其他 FFT 与现成 STFT crate — 浅读
| 项 | 内容 |
|---|---|
| 做什么 | phastft 0.4.1（2026-07-31，只做 2 的幂、新）；microfft 0.6.0（2024-04，no_std 定长，GitLab）；stft 0.2.0（2016）/ ruststft 0.4.0（2026-05）/ stft-rs 0.6.0（2026-04） |
| 许可证 | 均 MIT 或 MIT/Apache；兼容：是 |
| 维护状态 | 见上；下载量都远小于 realfft（phastft 1.3 万 vs realfft 1955 万） |
| 实时安全做法 | 未读 |
| 值得学的一处 | —— |
| 我们怎么用 | **不要**：STFT 自写约百行，FFT 用 realfft |
| 风险 | 小众、接口不稳 |
| 证据 | 【API】crates.io 搜索结果（q=stft / 各 crate） |

---

## 没查到 / 存疑

1. **延迟预算要实测**：GTCRN 的 STFT（窗 512、帧移 256 @16 kHz）算法延迟约等于窗长 32 ms【推断】，剩给「两次重采样 + 设备缓冲 + 共享内存交接」的只有约 13 ms。rubato `Fft` 默认参数上采样方向约 8 ms、下采样约 2.7 ms（复核者按源码推算），必须调小子块或改用 sinc 版——哪种在音质/延迟/耗电上最划算，要在实验线里量。
2. **非正规数（denormal）**：~~Apple Silicon 上 Rust 没有现成的 FTZ 守卫 crate~~ **复核更正：有**——resources/no_denormals（MIT，0.3.0）在 aarch64 上设 FPCR 的 FZ 位（resources/no_denormals/src/lib.rs:5、37-41，`1 << 24`，有 fp16 时再加 bit 19），但最后发版/推送 2025-09-11，**已超 12 个月**，体量很小，建议照它自写十来行而不依赖。ebur128 在非 x86 只能手动清零（filter.rs:250-253）。要么在实时线程入口设 FPCR.FZ（bit 24），要么在 biquad/GRU 状态上手动清零；ONNX Runtime 线程是否自设 FZ 未查【推断】。配合 DeepFilterNet #713 的「长静音变 NaN」，engine 必须有 NaN/Inf 守卫。
3. **sonora 全管线是否零分配**：只确认了 Limiter 有一处每帧分配，其余没逐个函数核；若要直接依赖需用 `assert_no_alloc` 一类工具实测。
4. **webrtc-audio-processing C++ 内部锁**：本地子模块未检出，只能推断 APM 内部有 mutex。
5. **torch.stft 对拍细节**：`torch.hann_window` 默认是 periodic 窗；`torch.stft` 默认 `center=True` 会反射填充——离线 eval 对拍要按此复现，实时流式则无 center 填充，两者首尾帧不同，对拍时只比中段【推断】。
6. **fbank 前端**（声纹组的事）：kaldi fbank 窗 25 ms/10 ms @16k = 400 点补到 512，也会用 realfft；Rust 版 kaldi fbank 现成 crate 本组未查。
7. **vDSP 与 realfft 在 512 点的实际速度对比**未测；只有需要时再做。
8. easyfft 源码是经 GitHub API 读取，不在 resources/ 里，行号以 main 分支当日为准。

## 复核记录（怀疑派 2）

> 2026-10-30 · 只读复核：亲手打开源码行、gh api、crates.io API；没跑构建或推理，也没克隆新仓。

**查了多少**：【源码】行号约 60 处（rubato、realfft、RustFFT、accelerate-rs、sonora、webrtc-audio-processing、nnnoiseless、libDF、biquad、ebur128、gtcrn），许可证与维护日期 18 个仓 / 22 个 crate 全部用 API 重查，issue 13 条（#103 #102 #713 realfft#45/#48/#49 rubato#142/#143 hound#101/#102 nnnoiseless#40/#41 ebur128#65）。

**改了哪些**
1. rubato `Fft` 延迟：原文「默认 → 16 kHz 侧约 8 ms」只说对了一个方向。按 synchro.rs:221-222、298-319、692-694 推算，上采样约 8 ms，下采样约 2.7 ms；风险栏和「存疑 1」都已改。
2. realfft 的证据行号前后不一致（669-680/748-752）：统一改成亲眼看过的 671-682、752-754。
3. sonora：补了**反例**。`GainController2::process`（gain_controller2.rs:238）每帧调 3 次 `extract_channels`（108-111，`to_vec` 复制）和多次 `collect`（264、313-331），所以「只有 Limiter 一处分配」不成立。
4. webrtc-audio-processing：`collect` 在 lib.rs:215，原来写的 210-214 没包含它，已改成 210-215。
5. DeepFilterNet #713：原文把 NaN 归因到「GRU 状态」，**不对**。issue 正文说的是开 FTZ 后，递推的 unit-norm 状态被清成 0，再算 0/0 得到 NaN。已改正，并加了「FTZ 不是免费的」这条教训。
6. biquad 写「无运行时依赖」**不对**：它依赖 num-traits（开 libm），许可证仍兼容。
7. 「Apple Silicon 没有现成 FTZ crate」**不对**：resources/no_denormals 在 aarch64 上设 FPCR bit 24（src/lib.rs:5、37-41）。不过它 2025-09-11 之后没更新，过不了命门 3，所以建议照着自己写。
8. hound #101 只影响 32 位平台，风险降级。ebur128 的行号 250-252 改成 250-253。

**核实无误（未改）**：rubato 的双接口（lib.rs:213-230、263-268）、Slip（slip.rs:9-30）；RustFFT 的 NEON 检测（neon_planner.rs:156-157）和规划顺序（plan.rs:73-87）；libDF 每帧两次分配（lib.rs:360、397）；nnnoiseless 用栈数组、静音时跳过网络（denoise.rs:95-113、features.rs:160）；easyfft 的线程局部缓存（经 API 读取，lib.rs:57-58、dyn_size/realfft.rs:96-113）。各 crate 的版本号和日期都与 API 一致。

**查不了的**
- symphonia、fundsp、dasp 只核了 API，没读源码。symphonia 的 lib.rs 文件头只有 MPL Exhibit A，没有 Exhibit B，已确认。
- 「vDSP 的 execute 不分配」「GPL 系统库例外」两条仍是推断。
- webrtc C++ 内部有没有锁：子模块没检出，看不到。
- rubato 的延迟数字是复核者手算的，没跑过。

**对推荐的异议**
- 首选的三条命门都没违反：realfft、rubato、biquad 都是 MIT/Apache，近 12 个月有发版，热路径有预分配接口。
- RustFFT 最后一次发版是 2025-09-18，已经超过 12 个月，严格按命门 3 是边缘情况。它的仓库仍活跃，可以接受，但建议在待人拍板里写一句。
- sonora：不只限幅器要「照思路自己写」。如果把 sonora 当实时门控的 VAD 候选，也要先用 assert_no_alloc 实测。直接依赖整条 AGC2 管线进实时线程是**不行**的。
- 漏掉的备选：去非正规数可以参考 no_denormals（照着自己写）；零分配实测工具 resources/ 里已经有 rust-assert-no-alloc、nice-assert-no-alloc、rtsan-standalone-rs，卡里应该指过去。fbank 用的 Rust crate 可以看 resources/kaldi-native-fbank-rs（归声纹组）。
