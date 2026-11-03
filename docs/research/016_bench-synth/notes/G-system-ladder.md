# G · 系统阶梯盘点：拿来检验合成评测格排名效度（Kendall τ）的 ≥ 12 个降噪系统

> 2026-10-31 · 016 线子代理 · **只读盘点**：没装任何东西、没下载、没跑推理，也没做 git 写操作（只用 onnxruntime 读了 ONNX 的输入签名，没喂数据）。
> 本机 `resources/` 在主检出 `noise_canceling/resources/`（下文简写 `R/`）；权重目录 `~/vdn-data/models/`（简写 `M/`）；Python 环境 `noise_canceling/_tmp/exp002-venv`（简写 `venv`）。
> venv 实测有：numpy · scipy · soundfile · librosa · onnxruntime · pyloudnorm · pesq · pystoi；**没有 torch / torchaudio / deepfilternet / mlx**。系统 Rust 1.99.1，Swift 6.4。
> 用途：这些系统**只作对照、不进产品**，所以许可只要求「本机跑、不分发」站得住；非因果、大模型都可以。

## 0 · 结论（5 行）

1. **不装新环境就能凑出 16 个**：4 个经典 DSP（含直通）+ 9 个 ONNX 模型（7 个已有推理代码，2 个格式与已有代码一样）+ Hush 自带的 arm64 预编译库（能跑 Hush，【推断】也能跑 DFN3）+ 3 个参数变体。
2. **独立性是短板**：16 个里大致只有 **8–9 个互相独立的「族」**。DNS3 训练、SEtrain 系（GTCRN · UL-UNAS · FastEnhancer）和 DFN 系（DPDFNet · Hush · DFN3 · OpenNC）各占一大块。算 τ 时建议**分族报告**，或者每族只取一个代表再算一次「族间 τ」。
3. **排名上下都盖到了**：最差那头有直通、谱减（音乐噪声）、WebRTC NS 6 dB；最好那头有 DPDFNet-8、Hush、DFN3，另外可以加一个**理想掩码（oracle，要用干净参考）**当天花板。中间段最挤，GTCRN、UL-UNAS、FE-S/M 预计会互相交叉【推断】。
4. **要大总管批的**：装 torch（换来 FRCRN、MossFormerGAN、MossFormer2-48k、URGENT-BSRNN、Demucs 这几个**架构独立**的大模型）；Flow-BSRNN 1.24 GB 超过 500 MB；还有几次小下载（OpenNC、nnnoiseless 依赖的 crate、FastEnhancer 的 VCTK 版）；以及运行 Hush 的**预编译 dylib**（015 说过「产品别用预编译」，这里只作对照）。
5. 下面所有耗时都是【推断】：按 exp001 每帧实测（GTCRN 最坏一帧 0.62 ms、DPDFNet-2 最坏 0.89 ms，单线程 ort）和官方数字外推，没有实跑。

---

## 1 · 候选总表

列含义：**族/训练集**一列用来标独立性（同一个字母 = 同族或同训练集，相关性高）。**本机**：✅ 代码和权重都在 · 🟡 只有代码、或要写一段小适配 · ❌ 要下载。**耗时**：10 s、16 kHz 音频，单线程 CPU，含 Python 逐帧调用开销，全部【推断】。

### 1.1 经典 DSP 与锚点（族 C，无训练集）

| # | 名称 | 架构 / 因果 | 族·训练集 | 许可（出处） | 本机 | 怎么离线跑 16 k wav | 权重 | 新装 | 10 s 耗时 |
|---|---|---|---|---|---|---|---|---|---|
| C0 | **直通 passthrough** | 输出 = 输入 · 因果 | 锚点（最差那头） | — | ✅ `src/eval/src/system.rs:4` `SystemKind::Passthrough` | vdn-eval 自带；或 numpy 一行 | 0 | 无 | ≈0 |
| C1 | **谱减（Boll 1979 + 过减）** | STFT 幅度减噪声估计，下限钳位 · 因果 | C | 自写 | 🟡 要写约 60 行 numpy | venv：scipy.signal.stft、最小值跟踪噪声、α = 2–4 过减 → 音乐噪声明显，预计排在下段 | 0 | 无 | < 0.2 s |
| C2 | **MMSE-LSA（Ephraim–Malah 1985 + 判决引导）** | 对数谱幅度 MMSE 增益 · 因果 | C（和 C3 很近） | 自写 | 🟡 约 80 行 numpy（scipy.special.exp1） | venv | 0 | 无 | < 0.3 s |
| C3 | **ref = RefWiener（vdn-eval 参考降噪器）** | 判决引导维纳 + 最小值跟踪，512/256，增益下限 −25 dB · 因果 | C（判决引导这一点和 C2 相同 → 相关性高） | 本仓 | ✅ `src/eval/src/system.rs:21-116` | 现成：vdn-eval `--system ref` | 0 | 无 | < 0.05 s |
| C4 | **WebRTC NS（sonora-ns，纯 Rust 移植）** | 分位数噪声估计 + 维纳，10 ms 帧 · 因果；4 档：6 / 12 / 18 / 21 dB（`R/sonora/crates/sonora-ns/src/config.rs:41-50`） | C（但噪声估计和增益规则与 C2/C3 不同） | BSD-3-Clause（`R/sonora/LICENSE` 首行 "Copyright (c) 2011, The WebRTC Project Authors"） | 🟡 源码在；要写约 50 行 bin（path 依赖 `sonora-ns`，wav 读写用 registry 已缓存的 hound-3.5.1） | `cargo build --offline`：sonora-ns → sonora-fft → sonora-simd，arm64 下**没有外部依赖**（cpufeatures 只在 x86 下才用，而且也已缓存） | 0 | 无（不用下载 crate） | < 0.05 s |
| C5 | **理想掩码 oracle IRM / 维纳（可选）** | 用干净参考算理想比值掩码 · 非因果（作弊） | 锚点（天花板） | 自写 | 🟡 约 30 行 numpy | 合成格里有干净参考，可以直接算 | 0 | 无 | < 0.1 s |

### 1.2 本机 ONNX（不用 torch）

| # | 名称 | 架构 / 因果 | 族·训练集 | 代码 / 权重许可（出处） | 本机 | 怎么跑 | 权重 | 新装 | 10 s 耗时 |
|---|---|---|---|---|---|---|---|---|---|
| A1 | **GTCRN（DNS3）** | 分组时序卷积 + 循环，48 K 参数，512/256 sqrt-Hann · 因果 | **A**：DNS3（`R/gtcrn/README.md:44-45`） | MIT（`R/gtcrn/LICENSE` "Copyright (c) 2024 Rong Xiaobin"）；权重在同一个仓库 | ✅ `R/gtcrn/stream/onnx_models/gtcrn_simple.onnx` | 现成：`docs/experiments/exp003-offline-ab/enhance.py` 的 `Gtcrn` 类；也可以 vdn-eval `--system gtcrn` | 0.5 MB | 无 | ≈0.3–0.6 s |
| A2 | **GTCRN（VCTK-DEMAND 版）** | 结构同 A1 | **A′**：VCTK-DEMAND（小数据集，泛化到合成场景预计更差 → 有助于拉开排名）【推断】 | MIT（`R/biglinux-gtcrn/LICENSE`，同一作者）；这个 ONNX 是 biglinux fork 自己导出的（`R/biglinux-gtcrn/ladspa/build.rs:81`），原仓只给了 `.tar` 检查点 | ✅ `R/biglinux-gtcrn/stream/onnx_models/gtcrn_vctk.onnx`（输入签名和 A1 **完全相同**，已读 ONNX 核过） | 把 `Gtcrn` 类的路径换一下就行 | 0.4 MB | 无 | 同 A1 |
| A3 | **UL-UNAS** | NAS 搜出来的超轻 U-Net，512/257 流式，状态显式 · 因果 | **A**：DNS3（检查点名 `model_trained_on_dns3.tar`），**和 GTCRN 同一作者、同一套 SEtrain 模板** → 独立性差 | MIT（`R/ul-unas/LICENSE` "Copyright (c) 2026 Xiaobin Rong"） | ✅ `R/ul-unas/ulunas_onnx/onnx_models/ulunas_stream_simple.onnx` | 照 `Gtcrn` 类改：输入 `mix [1,257,1,2]` + 3 个缓存（`R/ul-unas/ulunas_onnx/README.md:20-31`）；**窗是 Hann、不是 sqrt-Hann**（README 示例 `torch.hann_window(512)` + `torch.istft`），要在 numpy 里复现 istft 的窗归一 | 0.8 MB | 无 | ≈0.3–0.6 s |
| A4 | **FastEnhancer-S / -B** | 波形进出、内含 STFT 的 RNN-former · 因果 | **A**（同训练集）：DNS3 wideband + VCTK 干净语音（`R/fastenhancer/README.md:259-262`）；架构和作者都和 GTCRN 无关 | MIT（`R/fastenhancer/LICENSE` "Copyright (c) 2025 AHN Sung Hwan"）；ONNX 来自官方 release `onnx-dns-v1.0.0` | ✅ `M/fastenhancer-dns-v1.0.0/fastenhancer_{b,s}.onnx` | 现成：`enhance.py` 的 `FastEnhancer(size="s"/"b")` | 3.1 MB（三档合计） | 无 | ≈0.3–0.8 s |
| A5 | **FastEnhancer-M** | 同 A4，档位更大 | **A** | 同 A4 | ✅ `M/.../fastenhancer_m.onnx` | 现成：`enhance.py` 默认 M；vdn-eval `--system fastenhancer-m` | 同上 | 无 | ≈0.5–1 s |
| B1 | **DPDFNet-baseline** | DeepFilterNet 式两段（ERB 增益 + deep filter），去掉双路 RNN 的基线，16 k、320/160 · 因果（前瞻约 2 帧） | **B**：DFN 系；训练集模型卡没写（HF `Ceva-IP/DPDFNet`；要看论文 arXiv 2512.16420，【未核】） | 代码 Apache-2.0（`R/DPDFNet/LICENSE`）；权重 Apache-2.0（HF 模型卡 "License: apache-2.0"） | ✅ `R/weights/dpdfnet/baseline.onnx`（签名 `spec` + `state_in[38256]`，和 dpdfnet2 同格式） | 现成：`enhance.py` 的 `Dpdfnet` 类换路径（state 按 ONNX 元数据初始化） | 8.3 MB | 无 | ≈0.5–1 s |
| B2 | **DPDFNet-2 / -4 / -8** | 同上，加 2 / 4 / 8 块双路 RNN；0.36 / 1.35 / 2.36 / 4.37 GMACs（`R/DPDFNet/README.md:44-49`） | **B**（同族同训练集，档位变体） | 同 B1 | ✅ `R/weights/dpdfnet/dpdfnet{2,4,8}.onnx` | 同 B1；dpdfnet2 就是 exp003 用的那个 | 9.7 / 11 / 14 MB | 无 | 2：≈1 s；8：≈2–4 s |
| B3 | DPDFNet-2/8 48 kHz-HR（可选） | 同上，48 k | **B** | 同上 | ✅ `R/weights/dpdfnet/dpdfnet{2,8}_48khz_hr.onnx` | 要先 16→48 k 重采样（scipy `resample_poly`），再降回 16 k；元数据要另核 | 10 / 14 MB | 无 | ≈2–6 s |
| B4 | **Hush（Weya DeepFilterNet-SE 16 k）** | DFN3 结构的 16 k 精简版（`conv_ch=16`、DF 只到 64 bin），零前瞻 · 因果 | **B**：DFN 系；训练集混合（LibriSpeech、VCTK、Common Voice、ESC-50、DNS、AudioSet；`R/Hush/DATASETS.md`）。015 判断它**没有继承** DFN3 权重（015 `A-dfn-realtime.md` §2） | 代码和权重 Apache-2.0（`R/Hush/LICENSE`；HF `weya-ai/hush` 卡 `license: apache-2.0`，出处见 015 A §2） | ✅ 模型 `R/Hush/deployment/models/advanced_dfnet16k_model_best_onnx.tar.gz`；**预编译 arm64 库** `R/Hush/deployment/lib/libweya_nc.dylib`（`file` 核过：Mach-O arm64） | 现成：`R/Hush/deployment/examples/python/denoise_ctypes.py`（只要 numpy + ctypes，16-bit PCM wav）；`weya_nc_session_create(…, atten_lim_db)` 可以调衰减上限（`R/Hush/native/weya_nc_build/weya_nc/src/lib.rs:200-204`），阈值写死 −15/35/35 | 8.2 MB | 无（但要运行别人的预编译二进制，见 §3） | ≈0.6–1 s |
| B5 | **DeepFilterNet3（默认版 / `_ll`）** | ERB 增益 + deep filter，48 k，默认版前瞻 2 帧、`_ll` 零前瞻 · 因果 | **B**：DFN 系（DNS 等，论文 arXiv 2305.08227） | 代码 MIT/Apache-2.0（`R/DeepFilterNet/LICENSE`）；**权重许可没写明**（015 A §2：issue #697/#700/#709/#712 都没有维护者回复）→ 只作对照可以，不分发 | ✅ `R/DeepFilterNet/models/DeepFilterNet3_onnx.tar.gz`（7.6 MB）、`DeepFilterNet3_ll_onnx.tar.gz`（35 MB） | **【推断，要冒烟】**：Hush 的 dylib 内嵌的是 libDF v0.5.3，运行时读 `config.ini`，`model = deepfilternet3` 就放行（`R/Hush/native/weya_nc_build/vendor/deep_filter/src/tract.rs:306-312`），采样率取模型自己的。所以把 tar.gz 路径交给同一个 ctypes 脚本，**大概率**能跑 48 k 的 DFN3。注意 weya 内部用的是**线性重采样**（`lib.rs:105`），应先在外面用 scipy 升到 48 k、再按 input_sr = 48000 送进去。DFN2 会被 bail 掉（`tract.rs:308`）。备选路线：自己用 numpy 写 ERB 特征 + 三张图（约 200 行，工作量中等）；或者从源码编 libDF（缺 517 个 crate，要下载） | 7.6 / 35 MB | 无（走 dylib 的话） | 默认版 ≈1–2 s；`_ll` ≈2–3 s |
| V1 | VoxLock（int8，可选） | 目标说话人提取（WeSep + ECAPA 注册），LSTM，10 ms 块 · 因果 | **D**：LibriSpeech + AMI（`R/voxlock/README.md:191`） | Apache-2.0（`R/voxlock/LICENSE`；HF `arbazsiddiqui/voxlock`） | ✅ `M/voxlock/{enroll,chunk_int8}.onnx` | 现成：`enhance.py` 的 `Voxlock` 类；**要一段注册语音**（合成格可以用同一说话人的另一句干净语音）。它是 TSE、不是纯降噪：没有干扰人声的格里行为可能不典型，排名意义要打折 | 148 MB（目录合计；int8 一套约 74 MB） | 无 | ≈1–3 s |

### 1.3 要下载、但不用 torch

| # | 名称 | 架构 / 因果 | 族·训练集 | 许可 | 本机 | 怎么跑 | 大小 | 新装 | 10 s 耗时 |
|---|---|---|---|---|---|---|---|---|---|
| R1 | **RNNoise（nnnoiseless，Rust 版）** | 22 个 Bark 频带增益 + GRU + 基音滤波，48 k、10 ms · 因果；权重编进源码（`R/nnnoiseless/src/weights.rnn`） | **E**：RNNoise 原始训练数据（2017，Xiph/Mozilla）；**独立族**，而且预计排在中下段 → 很有价值 | BSD-3-Clause（`R/nnnoiseless/COPYING`："Copyright (c) 2020, Joe Neeman / 2017, Mozilla / 2007-2017, Jean-Marc Valin"） | 🟡 源码在，没编译过 | `cargo build --release --bin nnnoiseless`（要 `bin` feature）；**库本身就缺 `easyfft`**，bin 还要 dasp 等 → 必须 `cargo fetch`（几 MB 的小 crate）。16→48 k 在外面用 scipy 做 | 权重已在源码里 | 拉几个 crate | < 0.1 s |
| R2 | OpenNC v1 | DFN 结构 8.2 M 参数，48 k，零前瞻，单图逐帧 ONNX，状态显式 · 因果 | **B**：DFN 系，DNS4 + MUSAN，从头训练（据 `github.com/MAkcanca/OpenNC` README） | 代码和权重 Apache-2.0（同上 README；`R/opennc-app/README.md:62` "Model weights: Apache 2.0"） | ❌ `R/opennc-app` 只有应用，没带模型 | 下载 `opennc-v1-stream.zip`（release 页没写大小，【推断】< 50 MB）；它自带 numpy + onnxruntime 的参考引擎 `inference/engine.py` | 未知（< 50 MB【推断】） | 下载 | ≈1 s（自报约 1 ms / 10 ms） |
| R3 | FastEnhancer VCTK-DEMAND 版 | 同 A4 | **A′**：VCTK-DEMAND | MIT（同 A4）；release `onnx-vd-v1.0.0`（30 个资产） | ❌ | 下载后直接套 `FastEnhancer` 类 | 每个几 MB | 下载 | 同 A4 |

### 1.4 要装 torch（架构独立、多半是大模型）

| # | 名称 | 架构 / 因果 | 族·训练集 | 许可 | 本机 | 怎么跑 | 权重 | 新装 | 10 s 耗时 |
|---|---|---|---|---|---|---|---|---|---|
| T1 | FRCRN_SE_16K（ClearerVoice） | 频率循环 CRN，复数掩码 · 非因果 | **F**：阿里自有的「大规模数据」，README 没列清单 | 代码 Apache-2.0（`R/ClearerVoice-Studio/LICENSE`）；权重 Apache-2.0（HF `alibabasglab/FRCRN_SE_16K` 卡） | 🟡 代码在 `R/ClearerVoice-Studio/clearvoice/clearvoice/models/frcrn_se`，权重不在 | 装 torch + clearvoice 依赖；`ClearVoice(task='speech_enhancement', model_names=['FRCRN_SE_16K'])`（会**自动从 HF 下载**，要先关掉或改成本地路径） | 161 MB | torch + 下载 | ≈2–5 s |
| T2 | MossFormerGAN_SE_16K | MossFormer 时频块 + GAN 训练 · 非因果 | **F**（同一家、同一套数据，和 T1 相关） | Apache-2.0（HF 卡） | 🟡 同上 `mossformer_gan_se` | 同上 | 生成器 38.8 MB（判别器 97.6 MB 不用下） | torch + 下载 | ≈5–15 s |
| T3 | MossFormer2_SE_48K | MossFormer2 + 循环模块，48 k · 非因果 | **F** | Apache-2.0（HF 卡） | 🟡 同上 `mossformer2_se` | 同上；先升到 48 k 再降回 | 222 MB | torch + 下载 | ≈10–30 s |
| T4 | URGENT 2026 BSRNN 基线 | 频带分割 RNN · 非因果 | **G**：700 h「TBF」URGENT 数据（`R/urgent2026_challenge_track1/readme.md:96-102`） | 代码 Apache-2.0（`LICENSE.txt`）；权重 MIT（HF `lichenda/icassp_2026_urgent_baseline` 卡） | 🟡 `R/urgent2026_challenge_track1/baseline_code/inference.py` | 装 torch（还可能要 espnet 系依赖）+ `bsrnn.ckpt` | 151 MB | torch + 下载 | ≈5–20 s |
| T5 | URGENT 2026 Flow-BSRNN | 流匹配生成式 · 非因果 | **G** | MIT（同上） | 🟡 同上 | 同上，`flow_bsrnn.ckpt` | **1.24 GB（> 500 MB）** | torch + 大下载 | ≈30–120 s（多步采样） |
| T6 | Demucs denoiser（dns48 / dns64） | 波形 U-Net + LSTM · 因果 | **H**：DNS（master64 还加了 Valentini） | **CC-BY-NC 4.0**（`github.com/facebookresearch/denoiser` README），只作对照可以，不能分发 | ❌ 没克隆 | pip `denoiser`（要 torch）；权重首次运行时从 dl.fbaipublicfiles 拉 | 几十 MB【推断】 | torch + 下载 | ≈1–2 s |
| T7 | G-MaP-SE | MP-SENet 式 TS-Conformer + 说话人 GMM 先验，400/100 · 非因果 | **I**：VoiceBank+DEMAND（`R/G-MaP-SE/README.md`） | MIT（`R/G-MaP-SE/LICENSE` "Copyright (c) 2026 Hello3orld"） | ✅ 权重 `R/G-MaP-SE/ckpt/g_best`（9.4 MB）+ `clean_gmm_mu_k.pt`；还要 ECAPA（`M/wespeaker/voxceleb_ECAPA512.onnx` 已有） | 装 torch；`inference_g_map_se.py` | 9.4 MB | torch | ≈3–10 s |

### 1.5 看过但不进阶梯

| 名称 | 原因 |
|---|---|
| LiSenNet | `R/LiSenNet` 只有训练代码，**没有权重**（README 只说怎么训练和测试） |
| ID-PSE | 个性化 SE，要注册嵌入；`R/ID-PSE` **没有 LICENSE 文件**；要 torch |
| DENSE、SEF-PNet、D-LGTSE、USEF-TSE、speakerbeam、wesep、SpEx | 目标说话人提取，不是降噪；DENSE 没许可、没权重；speakerbeam 是「仅供评估」许可 |
| DFN2（含 `_ll`） | libDF v0.5.3 直接 bail（`tract.rs:308`）；要退回 v0.3.1 自己编 |
| MetalVoice（DFN3 Core ML） | 权重就是 DFN3，和 B5 重复；还要写 Swift |
| DNS-Challenge NSNet2 | 本地克隆是浅克隆，`NSNet2-baseline/` 已经不在树里；要另外下载（PyPI `nsnet2-denoiser` 写着权重 CC BY 4.0）。可以作为 R 类候补 |
| webrtc-audio-processing（tonarino） | 和 C4 是同一个算法，还要 C 库构建；用 sonora 就够了 |
| opus/dnn（LACE/NoLACE） | 是编解码器后处理，不是降噪 |

---

## 2 · 推荐清单（16 个，全部不用装新环境；★ = 每族代表）

| 顺位 | 系统 | 族 | 预期位置【推断】 | 状态 |
|---|---|---|---|---|
| 1 ★ | C0 直通 | 锚点 | 最底 | 现成（vdn-eval） |
| 2 | C1 谱减（α = 3，下限 0.02） | C | 下段（音乐噪声） | 写 numpy |
| 3 | C4a WebRTC NS 6 dB | C（参数变体） | 下段 | 写 Rust 小 bin（离线编译） |
| 4 ★ | C4b WebRTC NS 18 dB | C | 中下 | 同上 |
| 5 | C2 MMSE-LSA | C | 中下 | 写 numpy |
| 6 | C3 ref RefWiener | C | 中下 | 现成（vdn-eval） |
| 7 | A2 GTCRN-VCTK | A′ | 中下（跨域） | 换路径 |
| 8 ★ | A1 GTCRN-DNS3 | A | 中 | 现成 |
| 9 | A1-mix GTCRN 湿干比 0.5（输出 = 0.5·增强 + 0.5·对齐原声） | A（参数变体） | 中下 | 现成 + 一行 |
| 10 | A3 UL-UNAS | A | 中 | 改 `Gtcrn` 类（注意 Hann 窗） |
| 11 | A4 FastEnhancer-S | A（同训练集） | 中 | 现成 |
| 12 ★ | A5 FastEnhancer-M | A（同训练集） | 中上 | 现成 |
| 13 | B1 DPDFNet-baseline | B | 中上 | 换路径 |
| 14 ★ | B2 DPDFNet-8 | B | 上 | 换路径 |
| 15 ★ | B4 Hush（atten_lim 关） | B | 上 | ctypes 脚本现成 |
| 16 | B4-lim Hush atten_lim = 12 dB | B（参数变体） | 中上 | 同上，改一个参数 |
| （+17） | B5 DFN3 `_ll`（经 Hush dylib） | B | 上 | 【推断】要先冒烟一次 |
| （+18） | C5 理想掩码 oracle | 锚点 | 最顶 | 写 numpy；**它是作弊系统**，报 τ 时要分开算「含 oracle」和「不含 oracle」两版 |

**独立族统计**：锚点 · C 经典 · A 系（DNS3：GTCRN / UL-UNAS / FastEnhancer）· A′（VCTK）· B 系（DFN：DPDFNet / Hush / DFN3）。16 个里**真正独立的只有 5 族**。所以建议：
- 要提高独立性，**第一选择是 R1 RNNoise**（独立族 E，只要拉几个 crate）；其次是 torch 那一批（F、G、H、I 四个新族）。
- 算 τ 时同时报：全体 τ；每族取一个代表（★）的 τ；去掉参数变体后的 τ。参数变体只能检验「同一个模型调一下旋钮，指标的方向对不对」，不能当作独立样本。

## 3 · 要大总管批准的项

| 项 | 为什么要批 | 规模 |
|---|---|---|
| 运行 Hush 预编译 `libweya_nc.dylib`（B4、B5 都靠它） | 这是别人编好的二进制，要在本机执行；015 A 说过「别用预编译」（那是针对产品）。备选：从 `R/Hush/native/weya_nc_build` 源码自己编，但会缺大量 crate（tract 等），要下载 | 0 下载 |
| `cargo fetch` nnnoiseless 的依赖（R1） | 要从网上拉 crate | 几 MB |
| 下载 OpenNC v1 stream zip（R2）、FastEnhancer VCTK 版 ONNX（R3） | 下载 | 每个都 < 50 MB【推断】 |
| **装 torch**（T1–T7） | 新环境；铁律 8：venv 和模型缓存先进 `.gitignore` | torch 约 0.5–1 GB；权重 T1 161 MB、T2 39 MB、T3 222 MB、T4 151 MB、T7 已在本机 |
| **Flow-BSRNN（T5）** | 单个权重 1.24 GB，超过 500 MB | 1.24 GB |
| Demucs（T6） | CC-BY-NC：本机对照可以，产物不能分发 | 小 |

## 4 · 给实现的提醒

- 对齐和响度照 `enhance.py` 的口径：延迟用互相关核对，响度只在说话区间对齐。不然 C 类（零延迟）和 B 类（有 2–4 帧延迟）之间的差异会混进排名。
- 10 s × 16 个系统的总耗时【推断】约 15–25 s / 条；如果合成格有几百条，ONNX 那几类是大头，可以多进程并行。
- DFN 系在纯数字静音下有 NaN 风险（015 A，issue #713）：合成格里如果有全零片段，要垫一个 −120 dBFS 的底噪，或者先检查输出。
