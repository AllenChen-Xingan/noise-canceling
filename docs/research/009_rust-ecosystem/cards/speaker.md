# 009 Rust 生态调研 · 声纹与 VAD 组（speaker）

> 调研日 2026-10-30。路径 `resources/…` 都相对 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。
> 证据等级：【源码】亲眼读过的行 ·【官方】URL ·【API】GitHub / crates.io API 返回 ·【推断】我自己的判断。
> 没跑任何 cargo build / test / 推理，结论里凡是「能不能编过、跑多快」都是推断。

## 本组覆盖了哪些项目

**深读（读了热路径源码）**：kaldi-fbank（ryaningli，纯 Rust）· pyannote-rs（含 knf-rs）· earshot · silero（findit-studio）· sherpa-onnx（Rust 绑定 + C++ 声纹管理器）· speakrs
**中读（读了关键文件）**：WeSpeaker 与 3D-Speaker（fbank 参数，对拍基准）· kaldi-native-fbank（C++ 原版）· kaldi-native-fbank（RustedBytes 的 Rust 移植）· voice_activity_detector
**浅读（只看 API / 官方）**：sherpa-rs（已归档）· webrtc-vad · mel_spec · mfcc · silero-vad-rs · silero-rs（emotechlab）· TEN VAD · diaric / RustedBytes pyannote-rs · silero-vad（上游模型）

本次新克隆进 `resources/`：pyannote-rs · kaldi-native-fbank · kaldi-native-fbank-rs · kaldi-fbank · earshot · silero-findit · voice_activity_detector · speakrs · sherpa-onnx（全都 < 40 MB，没有超过 1 GB 的）。

## 推荐（首选 + 备选 + 理由）

| 子需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| 80 维 fbank（声纹前端） | **自写**，放 `dsp::fbank`，大约 200 行：预先规划好的 realfft + 调用方自带工作区 + 窗型可配。**对拍基准是 torchaudio `kaldi.fbank`，参数按模型逐项对齐**；`kaldi-fbank` 只放 dev-dependency 当第二个对照 | 直接依赖 `kaldi-fbank` 的 `FbankComputer::compute` 和 `extract_window`，**绕开它的 `OnlineFbank`**（那个每帧都要分配） | knf-rs 把窗型写死成 povey（WeSpeaker 训练时用的是 hamming），每次调用都分配，还要 cmake 编 C++，**不要用**。kaldi-fbank 是纯 Rust、Apache-2.0、参数齐全，但仓库刚建 2 个月、2 颗星、只有一个作者，不适合当唯一依赖。细节见下面「fbank 对拍参数表」 |
| 实时 VAD（降噪线程里，门控要「几十 ms 立刻放开」） | **earshot**，直接依赖，放 `engine` 或 `dsp` | 自写能量门限加滞回（兜底，零依赖） | 纯 Rust、零分配、不需要 ORT；16 ms 帧（256 点 @16 kHz），和 GTCRN 的 hop 正好相同；~8 KiB 状态 |
| 非实时 VAD（声纹线程挑「清楚的单人语音」） | **silero crate**（findit-studio），或者照它的思路用我们自己的 `ort` 跑 silero_vad.onnx | earshot 同一个实例（省掉 ORT 的一份开销） | Silero 的召回率有口碑；这个 crate 把会话（Session）和每路流的状态（StreamState）拆开、输入零拷贝，还有和 Python 的对拍工具；不在实时线程上跑，ORT 内部分配可以接受 |
| 声纹模型推理 | 用我们自己的 `ort`（engine 组定版本）加自写 fbank，放 `speaker` | sherpa-onnx 官方 crate 整包用（`SpeakerEmbeddingExtractor`） | sherpa-onnx 会自带并静态链接一份 onnxruntime，编译时还要从 GitHub 下载预编译库，和我们自己的 ORT 很可能重复【推断】；但它的模型元数据约定（`framework` / `normalize_samples`）值得照抄 |
| 在线注册 / 认人 | **抄思路自写**：sherpa-onnx `SpeakerEmbeddingManager` 的做法（质心先求和再 L2 归一化，预先归一化好的矩阵乘向量）+ pyannote-rs 的「过阈值就归到已有说话人，否则新建」增量法 | speakrs 的 AHC（kodama）用来锁定前一次性聚类 | 我们的需求是「锁定一簇 = 你」加上慢更新，现成 crate 没有一个直接对上；核心逻辑不到 100 行，自己写才能做到零分配、固定容量 |
| 离线说话人分离（eval 造标签、做对照） | **speakrs**，只在 `eval` 里当开发工具 | pyannote-rs | speakrs 在维护（10 月还有发布）、Apache-2.0、实现了完整的 pyannote community-1（PLDA + VBx）流程；pyannote-rs 已经一年多没提交 |

## 本组最值得学的 2 个设计

1. **「共享的不可变配方 + 每个句柄一份私有工作区」的 fbank**：`resources/speakrs/src/inference/cpu/fbank.rs:15-34`（`Recipe` 里放 FFT 计划、hamming 窗、mel 表，用 `Arc` 共享；`SignalWorkspace` 每个句柄一份），`:58-75` `compute_into` 直接写进调用方的存储，`:92-105` 去直流和预加重用 f64 算，注释原话「quiet mel bands are sensitive to cancellation」。对我们有用的地方：`dsp::fbank` 照这个结构写就能零分配；f64 这一招正好对准「对拍时安静频带差得最多」的坑。
2. **定长数组 + `ptr::copy` 移窗的流式 VAD**：`resources/earshot/src/lib.rs:107-113`（状态全部是定长数组）和 `:238-293`（每帧做预加重、Hann 窗、1024 点 rfft、40 维 log-mel、RMS 归一化、小型 GRU 网络，全程不分配）。它说明实时线程里可以放一个「不需要 ORT 的小模型」，也可以拿来当 `engine` 适配器状态布局的范本。

## fbank 对拍参数表（自写前必看）

| 参数 | WeSpeaker（ECAPA / ResNet） | 3D-Speaker CAM++ | sherpa-onnx 的做法 | knf-rs |
|---|---|---|---|---|
| 窗型 | **hamming**【源码】`resources/wespeaker/wespeaker/bin/infer_onnx.py:47`、`resources/wespeaker/wespeaker/dataset/processor.py:524` | 没传参，用 torchaudio 默认的 povey【源码】`resources/3D-Speaker/speakerlab/process/processor.py:153-154` | 两家都用 povey【源码】`resources/sherpa-onnx/sherpa-onnx/csrc/features.h:61` + `speaker-embedding-extractor-general-impl.h:34-37` | povey（`knfc.cc` 没设窗型，用的是 knf 默认）【源码】`resources/pyannote-rs/crates/knf-rs/sys/knfc/knfc.cc:18-33`、`resources/kaldi-native-fbank/kaldi-native-fbank/csrc/feature-window.h:41` |
| 输入幅度 | 乘 32768【源码】`infer_onnx.py:40` | 不放大，保持 [-1, 1]（直接用 `torchaudio.load` 的结果）【源码】`resources/3D-Speaker/speakerlab/bin/infer_sv.py:265-273` | 按模型元数据 `normalize_samples` 决定；WeSpeaker 是 0，也就是 ×32768【源码】`resources/sherpa-onnx/scripts/wespeaker/add_meta_data.py:127` | 先除以 32768【源码】`resources/pyannote-rs/crates/knf-rs/src/lib.rs:47-50` |
| snip_edges | torchaudio 默认 True【推断】 | 同左【推断】 | WeSpeaker 测试脚本用 False，3D-Speaker 测试脚本用 True【源码】`resources/sherpa-onnx/scripts/wespeaker/test.py:80`、`scripts/3dspeaker/test-onnx.py:80` | True【源码】`knfc.cc:25` |
| use_energy / dither | False / 0【源码】`infer_onnx.py:45-48` | dither=0【源码】`processor.py:143,154` | dither 默认 0【源码】`features.h:46` | dither 0【源码】`knfc.cc:23` |
| CMN（减去全句均值） | 有，不做 CVN【源码】`infer_onnx.py:49-50` | 有，`mean_nor=True`【源码】`infer_sv.py:276` | 有【源码】`speaker-embedding-extractor-general-impl.h:109` | 有【源码】`knf-rs/src/lib.rs:41-42` |
| log 下限 | torch float32 eps【源码】`resources/speakrs/src/inference/native_model/fbank.rs:25-26` | 同左【推断】 | — | — |

**要记住的结论**：
- 一，窗型和输入幅度**跟着模型走**，自写的 fbank 必须能配置，不能写死。
- 二，幅度只差一个常数倍（×32768），过了 log 就是逐 bin 一个常数偏移，CMN 会把它基本抵消；但 eps 下限附近的安静帧消不掉。所以对拍要分「CMN 前」和「CMN 后」两档阈值【推断】。
- 三，sherpa-onnx 和 pyannote-rs 拿 povey 窗喂 WeSpeaker 系的模型，说明这么做也能用；但「和训练时一致」的基准只有 Python 那一套。我们的对拍以 torchaudio 为准。

---

### kaldi-fbank（https://github.com/ryaningli/kaldi-fbank）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | 把 kaldi-native-fbank 的 FBank 流程从零移植成纯 Rust 的流式特征提取器，各项参数都保留 |
| 许可证 | Apache-2.0【源码】`resources/kaldi-fbank/Cargo.toml:7`；**与 GPL-3.0 兼容：是**（Apache-2.0 → GPLv3 单向兼容）。运行时依赖只有 realfft、rustfft（都是 MIT/Apache）【源码】`Cargo.toml:17-18` |
| 维护状态 | crates.io 0.1.0（2026-09-01）【API】；最后一次提交 2026-09-01，0 个 open issue，2 颗星，仓库建于 2026-09-01【API】。**只有一次集中提交、没有 issue 历史，响应速度没法判断** |
| 实时安全做法 | FFT 只在构造时规划一次、预分配 scratch【源码】`src/fft.rs:31`、`:41-44`；`FbankComputer::compute` 接收调用方给的 `signal_frame` 和 `feature` 两个切片，**本身不分配**【源码】`src/feature_fbank.rs:66-128`。但 `OnlineFbank` 每帧都 `self.feature.clone()` 推进 VecDeque【源码】`src/online.rs:191`，波形缓冲用 `extend_from_slice` / `drain` 增减【源码】`src/online.rs:146`、`:205`，**这一层会分配**。mel 表放在 HashMap 里按 VTLN 系数缓存，compute 里会查一次表【源码】`src/feature_fbank.rs:53-57`、`:107` |
| 值得学的一处 | 把「抽取一帧窗」（`extract_window`）和「算一帧特征」（`FbankComputer::compute`）拆成两个纯函数，各自接收预分配的缓冲【源码】`src/online.rs:175-190`。我们自写的时候照这条接缝切：环形缓冲归我们自己管，算特征的那一半做成纯函数，离线和实时可以共用 |
| 我们怎么用 | **dev-dependency 当第二个对照**（`dsp` 的测试）；备选方案里直接依赖它的 `FbankComputer`。注意默认 dither=0.00003、窗型 povey、snip_edges=true【源码】`src/options.rs:52`、`:55`、`:58`，必须显式改成和模型一致；use_energy 默认 false【源码】`src/options.rs:247` |
| 风险 | 单作者、刚发布，长期维护存疑；README 说和 C++ 参考版 604 组用例全部一致，但对照的是 C++ knf，**不是 torchaudio**【官方】https://github.com/ryaningli/kaldi-fbank#readme；ARM 基准测的是 RK3576（Linux），不是 Apple Silicon【源码】`docs/benchmark-arm.md` 开头 |
| 证据 | 【源码】【API】【官方】 |

### pyannote-rs（含 knf-rs / knf-rs-sys）（https://github.com/thewh1teagle/pyannote-rs）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | 用 ORT 跑 pyannote 分段模型和 WeSpeaker ResNet34 声纹模型做说话人分离；fbank 由同一个仓库里的 knf-rs（C++ kaldi-native-fbank 的 FFI 封装）提供 |
| 许可证 | MIT【源码】`resources/pyannote-rs/Cargo.toml:6`；knf-rs-sys 通过 git submodule 引入的 kaldi-native-fbank 是 Apache-2.0【源码】`.gitmodules`；**与 GPL-3.0 兼容：是**。依赖 eyre、hound、ndarray、ort，都是 MIT/Apache |
| 维护状态 | pyannote-rs 0.3.4、knf-rs 0.3.2，都发布于 2025-09-07【API】；最后一次提交 2025-09-07「update ort」【源码】git log；18 个 open issue；最近 5 个里 4 个是 PR，只有 #28 有 2 条评论，#27（issue）0 回复【API】。**超过 12 个月没提交，命门 3 不达标** |
| 实时安全做法 | **不安全**。`compute_fbank` 先在 C++ 里 `new float[]`【源码】`crates/knf-rs/sys/knfc/knfc.cc:41`，回到 Rust 再 `to_vec()` 拷一份【源码】`crates/knf-rs/src/lib.rs:17-23`，然后 ndarray 求均值又分配一次【源码】`lib.rs:41-42`；声纹提取每次都 `vec!` 一份 f32 缓冲【源码】`src/embedding.rs:20-21`；C++ knf 的 rfft 每帧都 `kiss_fftr_alloc`【源码】`resources/kaldi-native-fbank/kaldi-native-fbank/csrc/rfft.cc:62` |
| 值得学的一处 | `EmbeddingManager::search_speaker`：遍历已有说话人算余弦，超过阈值就取最像的那个，否则在没到 `max_speakers` 时新建一个【源码】`src/identify.rs:29-49`。这就是最小的「在线聚类」，我们锁定前收集候选簇可以从这里起步（要改成固定容量数组、预先归一化） |
| 我们怎么用 | **不要**直接依赖；在线聚类的思路**抄了自己写**（`speaker`） |
| 风险 | knfc 把参数写死成 povey / 80 维 / snip_edges，又不暴露窗型【源码】`knfc.cc:18-33`，而 WeSpeaker 训练用的是 hamming【源码】`resources/wespeaker/wespeaker/dataset/processor.py:524`，**用它对拍会误判**；构建要 cmake 编 C++，build.rs 里还用 `cp -rf` 复制源码【源码】`crates/knf-rs/sys/build.rs:13-22` |
| 证据 | 【源码】【API】 |

### earshot（https://github.com/pykeio/earshot）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 的流式 VAD：16 ms 帧进，0–1 分数出；内置约 40 KiB 的小型 GRU 类网络，不需要 ORT |
| 许可证 | crate 声明 MIT OR Apache-2.0【源码】`resources/earshot/Cargo.toml:12`（GitHub API 识别成 Apache-2.0【API】）；**与 GPL-3.0 兼容：是**。默认没有运行时依赖，`libm` 可选【源码】`Cargo.toml:32-33` |
| 维护状态 | 1.2.2（2026-08-19）、1.2.1（2026-07-22）【API】；最后一次提交 2026-08-30；2 个 open issue（#4、#5 都 0 回复），外部 PR #7、#8 有维护者回复并已合并【API】。**在维护，issue 响应偏慢** |
| 实时安全做法 | **安全**。状态全部是定长数组：环形采样缓冲、3 帧特征、FFT 缓冲【源码】`src/lib.rs:107-113`；移窗用 `ptr::copy`【源码】`:245-247`、`:276-278`；网络状态是 `[i16; 128]`【源码】`src/default_predictor.rs:35-37`；权重用 `include_bytes!` 编进二进制并按 4 字节对齐【源码】`src/default_predictor.rs:5-12`；`predict` 不分配、不加锁、不做系统调用【源码】`src/lib.rs:238-293`。帧长不对时直接返回 0 分，不 panic【源码】`:240-242` |
| 值得学的一处 | 帧长 `FRAME_SIZE=256`（16 ms @16 kHz），只往回看 3 帧上下文，没有前瞻【源码】`src/lib.rs:91-94`、`src/frame.rs:2`；GTCRN 的流式 hop 正好也是 256【源码】`resources/gtcrn/stream/gtcrn_stream.py:456`。**同一个 16 ms 时钟上，降噪和 VAD 可以对齐，不用另起缓冲** |
| 我们怎么用 | **直接依赖**，用在降噪线程的「快速放开」判据（`engine` 的门控）；离线 eval 也用同一个实例 |
| 风险 | 模型训练数据和权重来源，仓库里没找到说明（README 只说「any language」「SNR ≥ 3 dB」）【源码】`README.md`，**权重能不能按 GPL 分发要再核**（许可证文件覆盖整个 crate，包括 weights.bin）；精度只有 README 里作者自己的图，我们要在自己的素材上重测；只支持 16 kHz |
| 证据 | 【源码】【API】【推断】 |

### silero（findit-studio）（https://github.com/findit-studio/silero）【深读】
| 项 | 内容 |
|---|---|
| 做什么 | 面向生产的 Silero VAD v5/v6 ONNX 封装：会话和流状态分离，附带分段器和与 Python 的对拍工具 |
| 许可证 | MIT OR Apache-2.0【源码】`resources/silero-findit/Cargo.toml:12`；依赖 zuoer（MIT OR Apache-2.0【API】）、ort、thiserror；内置的 silero_vad.onnx 来自上游 snakers4，MIT【API】；**与 GPL-3.0 兼容：是** |
| 维护状态 | crates.io `silero` 0.7.0（2026-08-22），0.6.0（07-19）【API】；最后一次提交 2026-09-12；2 个 open issue；最近的 PR 有维护者回复【API】。注意 crates.io 上的仓库地址写的是 Findit-AI/silero，实际已经迁到 findit-studio |
| 实时安全做法 | **基本安全**：输入和状态的 scratch Vec 每次 `clear` 再 `extend`，容量复用，不再分配【源码】`src/session.rs:182-191`；用 `TensorRef::from_array_view` 零拷贝传入【源码】`:194-198`；RNN 状态写回定长数组【源码】`:210-216`；`StreamState` 全是定长数组，所以快照就是一次 memcpy【源码】`src/stream.rs:39-40`、`src/session.rs:366`。**但 ORT 的 `run` 内部会分配输出张量**【推断】（这一点归 engine 组统一评估） |
| 值得学的一处 | 一个 Session 可以服务多路 `StreamState`，处理失败就回滚到快照【源码】`src/session.rs:340-383`。GTCRN 的卷积 / GRU 缓存也可以这样建模：模型句柄无状态、缓存归每路流，切换 FastEnhancer 或推理进程重启时，状态可以清掉、也可以回滚 |
| 我们怎么用 | 声纹线程里挑「清楚的单人语音」片段：**直接依赖**，或者照它的结构用我们自己的 ort 写（`speaker`）。CoreML 拿不走 LSTM 控制流，在 macOS 上实际只跑 CPU【源码】`Cargo.toml:44-45` |
| 风险 | ort 只跟到 rc.12【源码】`Cargo.toml:65`，和我们选的 ort 版本要对齐；chunk 固定 512 点外加 64 点上下文（32 ms + 4 ms）【源码】`src/stream.rs:19-22`，不适合放实时线程 |
| 证据 | 【源码】【API】【推断】 |

### sherpa-onnx（官方 Rust API）（https://github.com/k2-fsa/sherpa-onnx）【深读：Rust 绑定 + 声纹管理器】
| 项 | 内容 |
|---|---|
| 做什么 | k2-fsa 的全家桶 C++ 推理库；`sherpa-onnx` / `sherpa-onnx-sys` 两个 crate 是官方的安全封装和 FFI，覆盖声纹提取、声纹管理器、离线分离、VAD（silero / ten-vad）、流式降噪（GTCRN、DPDFNet、voxlock）等 |
| 许可证 | Apache-2.0【源码】`resources/sherpa-onnx/sherpa-onnx/rust/sherpa-onnx/Cargo.toml:6`；**与 GPL-3.0 兼容：是**；预编译库里静态链接了 onnxruntime（MIT）【推断】。**如果启用 ten-vad 模型，那份模型带「不得和 Agora 竞争」附加条款，与 GPL 不兼容**（见 TEN VAD 卡） |
| 维护状态 | 2.0.1（2026-10-26/27）【API】；最后一次提交 2026-10-29；637 个 open issue，最近 5 条都是当天提的，PR 有回复【API】。**非常活跃，但 issue 积压多** |
| 实时安全做法 | **不安全**：每次 `compute` 都 `from_raw_parts(..).to_vec()` 拷出，再调 C 侧的 Destroy 释放【源码】`rust/sherpa-onnx/src/speaker_embedding.rs:108-109`；C++ silero 每次推理都新建 `std::vector<Ort::Value>`【源码】`sherpa-onnx/csrc/silero-vad-model.cc:430-438`；管理器每 Add 一次就 `conservativeResize` 一次【源码】`csrc/speaker-embedding-manager.cc:31` |
| 值得学的一处 | **声纹管理器**：注册时多条向量先求和（「反正要归一化，不用除以 N」），再 L2 归一化，存成矩阵的一行【源码】`csrc/speaker-embedding-manager.cc:63-81`；检索时只把查询向量归一化，然后「矩阵 × 向量」一次算出所有余弦，取最大值和阈值比【源码】`:117-135`。另外，模型元数据里写 `framework` 和 `normalize_samples`，前端据此决定是否 ×32768【源码】`csrc/speaker-embedding-extractor-model.cc:97-107`、`scripts/wespeaker/add_meta_data.py:127`。**我们的 `speaker` crate 可以照这个约定，给 ONNX 带一份前端参数清单** |
| 我们怎么用 | **抄思路**（管理器、元数据约定）；整包依赖只作备选。它的流式 GTCRN 实现值得 engine 组看（`rust-api-examples/run-streaming-speech-enhancement-gtcrn.sh`） |
| 风险 | `sherpa-onnx-sys` 的 build.rs 编译时用 ureq 从 GitHub Releases 下载预编译库【源码】`rust/sherpa-onnx-sys/build.rs:13-15`、`:205-213`，离线构建和供应链都受影响；它自带一份 ORT，和我们的 `ort` crate 同进程时可能符号冲突【推断】；它给 WeSpeaker 用 povey 窗、snip_edges=false【源码】`csrc/features.h:55`、`:61`，和 WeSpeaker 官方 Python 不一致 |
| 证据 | 【源码】【API】【推断】 |

### speakrs（https://github.com/avencera/speakrs）【深读：fbank + 聚类】
| 项 | 内容 |
|---|---|
| 做什么 | Rust 版说话人分离，复刻 pyannote community-1 流程（分段 + WeSpeaker ResNet34 + PLDA + VBx / AHC），后端有原生 CPU、CoreML、CUDA |
| 许可证 | Apache-2.0【源码】`resources/speakrs/Cargo.toml:27`；关键依赖 kodama 0.3（层次聚类，MIT【推断】）、ndarray、rustfft【源码】`Cargo.toml:90-96`；**代码与 GPL-3.0 兼容：是**；**它下载的 pyannote / PLDA 模型文件另有许可，没核**（见文末存疑） |
| 维护状态 | 0.7.0（2026-10-26）【API】；最后一次提交 2026-10-28；0 个 open issue；最近 5 个都是维护者自己提、自己合【API】。**非常活跃，基本是单人维护** |
| 实时安全做法 | fbank 这一层零分配：FFT 计划和 mel 表放进 `Arc<Recipe>` 共享，工作区每个句柄一份，输出写进调用方给的 `ArrayViewMut2`【源码】`src/inference/cpu/fbank.rs:15-75`；逐帧循环只用栈上的 `[0.0; SPECTRUM_BINS]` 和预分配好的工作区【源码】`:88-123`。但它的窗长固定 10 s（160 000 点、998 帧）【源码】`src/inference/native_model/fbank.rs:7-13`，按批处理设计，不是流式。AHC 每次都分配距离矩阵【源码】`src/clustering/ahc.rs:43`、`:73-77` |
| 值得学的一处 | 见文件开头「最值得学」第 1 条（`fbank.rs:15-34`、`:58-75`、`:92-105`）；还有一处：按 bin 求 CMN 均值时也用 f64 累加【源码】`:126-131`。它的测试拿一个独立的 f64 参考实现对拍，容差 1e-3【源码】`src/inference/cpu/fbank/tests.rs:87`、`:145-161`，**我们对拍时可以照这个容差起步** |
| 我们怎么用 | `eval` 里**当开发工具**（离线造分离标签、做对照）；fbank 的结构**抄思路**写进 `dsp`；AHC 的阈值与流程**只看看**，锁定前一次性聚类时可以参考 |
| 风险 | 默认 feature `online` 会在运行时从 HF Hub 下载模型，还要 native-tls【源码】`Cargo.toml:38`；功能面很大（CUDA / PTX），我们只需要其中一小块 |
| 证据 | 【源码】【API】【推断】 |

### WeSpeaker 与 3D-Speaker（https://github.com/wenet-e2e/wespeaker · https://github.com/modelscope/3D-Speaker）【中读：fbank 基准与 ONNX 导出】
| 项 | 内容 |
|---|---|
| 做什么 | 两个候选声纹模型的官方训练 / 推理 / 导出代码，是我们 fbank 对拍的**唯一基准** |
| 许可证 | 两者代码都是 Apache-2.0【推断，按仓库 LICENSE 惯例；没逐个打开】；代码兼容 GPL-3.0；**预训练权重的许可不在本卡范围**（005 #2 待拍板） |
| 维护状态 | 已在 resources 里，没再查 API【—】 |
| 实时安全做法 | 不适用（Python）；WeSpeaker 的 C++ runtime fbank 每帧都 new 一个 `std::vector`【源码】`resources/wespeaker/runtime/core/frontend/fbank.h:147-148` |
| 值得学的一处 | WeSpeaker 的 C++ fbank 整个 `Compute` 不到 70 行，流程是：去直流 → 预加重（首样本也减 `coeff*x[0]`）→ hamming 窗 → 补零到 2 的幂做 FFT → 功率谱 → 三角 mel 加权 → 按 eps 下限取 log【源码】`resources/wespeaker/runtime/core/frontend/fbank.h:138-198`（预加重在 `:122-127`，mel 表构建在 `:51-88`）。**说明自写 fbank 的工作量很小**，难点只在参数对齐 |
| 我们怎么用 | **只看看 + 当对拍基准**。导出的输入输出名：WeSpeaker 是 `feats` → `embs`，T 维动态，opset 14【源码】`resources/wespeaker/wespeaker/bin/export_onnx.py:88-99`；3D-Speaker 是 `feature` → `embedding`，opset 11【源码】`resources/3D-Speaker/speakerlab/bin/export_speaker_embedding_onnx.py:181-190`。`speaker` 的适配器要能吃这两种命名 |
| 风险 | 两家的窗型、幅度约定不同（见对拍参数表），写错一项也不会报错，只会让声纹悄悄变差 |
| 证据 | 【源码】【推断】 |

### kaldi-native-fbank（C++ 原版）（https://github.com/csukuangfj/kaldi-native-fbank）【中读】
| 项 | 内容 |
|---|---|
| 做什么 | 不依赖外部库的 Kaldi 兼容 fbank（C++），sherpa-onnx 和 knf-rs 底下都是它 |
| 许可证 | Apache-2.0【API】；兼容 GPL-3.0：是 |
| 维护状态 | v1.22.5（2026-10-30）、v1.22.4（10-21）【API】；0 个 open issue，最近的外部 PR #90 有回复【API】。**活跃** |
| 实时安全做法 | rfft 每次调用都 `kiss_fftr_alloc`，还要拷进 `std::vector`【源码】`resources/kaldi-native-fbank/kaldi-native-fbank/csrc/rfft.cc:53`、`:62-64`，**不适合实时** |
| 值得学的一处 | 默认值就是「Kaldi 默认值」：dither 0.00003、povey 窗、snip_edges=true、use_energy=false【源码】`csrc/feature-window.h:36`、`:41`、`:49`，`csrc/feature-fbank.h:41`。它的 Python 测试对照的是 kaldifeat，容差 atol 1e-3【源码】`python/tests/test_online_fbank.py:19-42` |
| 我们怎么用 | **只看看**（参数默认值的权威出处） |
| 风险 | 用 FFI 引入要 cmake，不值得 |
| 证据 | 【源码】【API】 |

### kaldi-native-fbank（RustedBytes 的 Rust 移植）（https://github.com/RustedBytes/kaldi-native-fbank）【中读】
| 项 | 内容 |
|---|---|
| 做什么 | knf 的另一个 Rust 移植，带 fbank / mfcc / whisper 特征 / stft |
| 许可证 | MIT【源码】`resources/kaldi-native-fbank-rs/Cargo.toml:6`；依赖 realfft、rustfft、rand、thiserror、anyhow、log，都兼容 |
| 维护状态 | crates.io 0.1.0（2026-01-12）【API】；最后一次提交 2026-04-19（改 README）；0 个 issue，3 颗星【API】 |
| 实时安全做法 | **不安全**：每帧 FFT 都 `to_vec()` 一份输入、`vec!` 一份输出【源码】`src/rfft.rs:66`、`:69`；在线层每次调用新建窗缓冲，每帧新建特征 Vec【源码】`src/online.rs:105`、`:123` |
| 值得学的一处 | 反面教材：**默认 `use_energy: true`**【源码】`src/fbank.rs:23`，和上游的 false【源码】`resources/kaldi-native-fbank/kaldi-native-fbank/csrc/feature-fbank.h:41` 不一样，不留神输出就是 81 维；dither 用的是均匀分布，不是高斯【源码】`src/window.rs:189`；测试只断言 `is_finite`，没有数值对拍【源码】`tests/all.rs:230` |
| 我们怎么用 | **不要** |
| 风险 | 和上游语义不一致 |
| 证据 | 【源码】【API】 |

### voice_activity_detector（https://github.com/nkeenan38/voice_activity_detector）【中读】
| 项 | 内容 |
|---|---|
| 做什么 | Silero VAD 的 ort 封装，带流式 / 迭代器 API |
| 许可证 | crates.io 上显示「non-standard」（用的是 license-file）【API】，文件内容是 MIT 原文【源码】`resources/voice_activity_detector/LICENSE:1`；兼容：是 |
| 维护状态 | 0.2.1（2025-08-04）【API】；最后一次提交 2025-08-04；6 个 open issue；最近的 PR 有回复【API】。**刚好在 12 个月边缘之外，命门 3 不达标** |
| 实时安全做法 | **不安全**：每次 predict 都 `zeros` 新建输入，`to_owned` 拷贝状态【源码】`src/vad.rs:59`、`:69-70`；推理前要拿 `Mutex` 锁【源码】`:74`；出错直接 `unwrap` panic【源码】`:75` |
| 值得学的一处 | 反面教材：「状态 `mem::take` 出来再 `to_owned`」这种写法每帧都要分配【源码】`src/vad.rs:65-83` |
| 我们怎么用 | **不要** |
| 风险 | 把 ort 钉死在 `=2.0.0-rc.10`【源码】`Cargo.toml:22-30` |
| 证据 | 【源码】【API】 |

### sherpa-rs（https://github.com/thewh1teagle/sherpa-rs）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 早期社区版的 sherpa-onnx Rust 绑定 |
| 许可证 | MIT【API】 |
| 维护状态 | **已归档**；最后一次提交 2026-03-08「Add deprecation notice for sherpa-onnx crate」；最后发布 0.6.8（2025-10-05）【API】 |
| 实时安全做法 | 没读 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（官方 `sherpa-onnx` crate 已经替代它） |
| 风险 | 已停更 |
| 证据 | 【API】 |

### webrtc-vad（https://github.com/kaegi/webrtc-vad）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 绑定 WebRTC 的 GMM VAD（C 代码） |
| 许可证 | MIT（绑定本身），WebRTC 的 C 代码是 BSD-3【推断】；兼容 |
| 维护状态 | 0.4.0（2019-10-01）；最后一次提交 2019-10-07；5 个 open issue，近年的都没人回复【API】。**停更，命门 3 不达标** |
| 实时安全做法 | 没读（GMM 本身很轻【推断】） |
| 值得学的一处 | — |
| 我们怎么用 | **不要**；要 WebRTC 那一套的话，看 resources 里已有的 webrtc-audio-processing（归别组） |
| 风险 | 停更 |
| 证据 | 【API】【推断】 |

### mel_spec（https://github.com/wavey-ai/mel-spec）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 生成和 whisper.cpp 兼容的 Mel 频谱（Rust）【API description】 |
| 许可证 | MIT【API】 |
| 维护状态 | 0.5.0（2026-10-02）；0 个 open issue；历史 issue 有回复【API】 |
| 实时安全做法 | 没读 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**：它是 whisper 风格的 log-mel（不是 Kaldi fbank），和声纹模型的训练前端对不上【推断】 |
| 风险 | 语义不一致 |
| 证据 | 【API】【推断】 |

### mfcc（https://github.com/bytesnake/mfcc）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | Rust 算 MFCC |
| 许可证 | crates.io 显示 MIT，GitHub 上没识别出 LICENSE【API】 |
| 维护状态 | 0.1.0（2019-05-30）；最后一次提交 2023-09-25【API】。**停更** |
| 实时安全做法 | 没读 |
| 值得学的一处 | — |
| 我们怎么用 | **不要** |
| 风险 | 停更、许可证不清 |
| 证据 | 【API】 |

### silero-vad-rs（https://github.com/binarycrayon/silero-vad-rs）· silero-rs（https://github.com/emotechlab/silero-rs）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 另外两个 Silero VAD 的 Rust 封装 |
| 许可证 | 都是 MIT【API】 |
| 维护状态 | silero-vad-rs 0.1.2（2025-04-04），4 个 open issue 都没回复【API】，**停更**；silero-rs 最后一次实质提交 2026-07-23，之后只有 dependabot 的 PR 还开着【API】，crates.io 同名的 `silero-rs` 0.1.0（2023-11）不确定是不是同一个仓库【API】 |
| 实时安全做法 | 没读 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（findit-studio 的 silero 更完整） |
| 风险 | — |
| 证据 | 【API】 |

### TEN VAD（https://github.com/TEN-framework/ten-vad）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | Agora 出的轻量 VAD（C 库 + ONNX），sherpa-onnx 已经集成 |
| 许可证 | 「Apache-2.0 + 附加条件」：不得用来和 Agora 竞争、只能用于自己的应用、衍生作品继续受这些条款约束【API】（LICENSE 原文第 1–3 条）。**与 GPL-3.0 不兼容**（GPL 禁止附加进一步限制）【推断】 |
| 维护状态 | 最后一次提交 2026-10-09；27 个 open issue【API】 |
| 实时安全做法 | 没读 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（许可证红线） |
| 风险 | 许可证 |
| 证据 | 【API】【推断】 |

### diaric / RustedBytes pyannote-rs（https://github.com/findit-studio/diaric · https://github.com/RustedBytes/pyannote-rs）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | 又两个 Rust 说话人分离实现（diaric 是 findit-studio 出的，和 silero 同一家；RustedBytes 那个是 pyannote-rs 的 fork） |
| 许可证 | diaric 声明「(MIT OR Apache-2.0) AND Apache-2.0 AND MIT AND CC-BY-4.0 AND BSD-2/3」【API】，CC-BY-4.0 应该是捆绑的模型文件【推断】；RustedBytes 那个是 MIT【API】 |
| 维护状态 | diaric 0.2.0（2026-09-01），最后一次 push 2026-10-23【API】；RustedBytes 那个最后一次提交 2026-01-16【API】 |
| 实时安全做法 | 没读 |
| 值得学的一处 | — |
| 我们怎么用 | **只看看**；如果 speakrs 不合适，diaric 可以当备选 |
| 风险 | 没读源码 |
| 证据 | 【API】【推断】 |

### silero-vad（上游模型）（https://github.com/snakers4/silero-vad）【浅读】
| 项 | 内容 |
|---|---|
| 做什么 | Silero VAD 模型本体（PyTorch / ONNX） |
| 许可证 | MIT【API】；兼容 |
| 维护状态 | 最后一次 push 2026-10-22，15 个 open issue【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | 通过 silero crate 使用它的 onnx；仓库 123 MB，**没克隆**（不需要） |
| 风险 | — |
| 证据 | 【API】 |

---

## 没查到 / 存疑

1. **earshot 权重的来源和训练数据**：仓库里没写，许可证文件按字面覆盖 weights.bin，但训练数据的许可没法核。发布前要问作者，或者至少在 NOTICE 里写清楚。
2. **Rust 生态里没有「零分配、固定容量」的在线说话人注册 / 聚类 crate**。pyannote-rs 用 HashMap 加阈值，sherpa-onnx 每次 Add 都重新分配矩阵，speakrs 和 diaric 都是离线批处理。我们的「锁定一簇 = 你 + 慢更新」只能自写（逻辑简单，估计 < 150 行）。
3. **torchaudio `kaldi.fbank` 的 snip_edges 默认值**我是凭记忆写的 True，没读 torchaudio 源码（resources 里没有）。对拍脚本里要显式传参，别靠默认值。
4. **ORT `run` 在热路径上会分配输出张量**，这是推断；是否要用 IoBinding 预绑输出，归 engine 组定。声纹线程不是实时线程，影响不大。
5. **sherpa-onnx 自带的 ORT 和 `ort` crate 能不能共存于同一进程**没验证（推断会冲突）。如果整包用 sherpa-onnx，就只能全用它的 ORT。
6. **WeSpeaker、3D-Speaker、pyannote / PLDA 权重的许可**不在本组范围，见 005 #2；speakrs 下载的 PLDA 文件也一样。
7. WeSpeaker / 3D-Speaker 仓库 LICENSE 我没逐个打开，Apache-2.0 是推断。
8. 所有「能在 macOS/aarch64 编过」的判断都没实测：earshot、kaldi-fbank 是纯 Rust，风险低；sherpa-onnx 依赖它在 Releases 里有没有 macOS 预编译包（build.rs 里有 xcframework 的地址，推断有）。
