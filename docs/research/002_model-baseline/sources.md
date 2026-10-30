# 两类来源 · 起步用哪个降噪模型与代码作为基线

> 2026-10-29 · 手下 model。取材方法：每个候选派一个核查代理，**用同一张核查表**（A 延迟 · B 采样率 · C 算力 · D ONNX/状态/STFT · E Rust 可复用 · F 许可证 · G 低 SNR 证据 · H 维护 · I 坑），每份再由一个怀疑派代理回源码反驳（workflow `wf_a4112186-a18`，10 个代理，全程只读）。
> 下面只登记**经怀疑派复核后站得住**的结论；被推翻的已改正，证据不足的标「待核」。
> 证据标记：**【官方】** 官方明载 · **【本机】** 本机读源码/实测 · **【传言】** 第三方或未复核。

---

## 一、自有积累

| 是什么 | 在哪 | 还算数吗 | 判据 |
|---|---|---|---|
| 设计与参考汇总（人整理） | `vault/raw/20261029-voice-denoise-design.md` §5、§11、§16 | **部分算数** | 选型方向、算力预算（每 10 ms 帧 ≤ 2 ms）仍成立；「DPDFNet-4 是最平衡起点」被本次核查部分推翻——它没算 DPDFNet 的 4 帧内部延迟（见 gap.md） |
| exp001 每帧耗时实测 | `docs/experiments/exp001-model-frame-time/README.md` | **算数**（本机今天跑的） | Apple M5 · ORT 1.31.1 · 单线程 · 6000 帧；只量模型本体、随机输入 |
| 本机其他项目 | `scan-own.sh` 未跑 | — | 本项目首次涉及降噪模型，无旧代码可复用（项目只有 3 个初始化提交，`git log`） |

---

## 二、他人积累

### 登记总表（许可证与权重来源单列，大总管要求）

| 是什么 | 官方仓库 / 本地 | 代码许可证 | **权重来源** | **权重许可证** | 角色 | 读多深 | 还活着吗 |
|---|---|---|---|---|---|---|---|
| **DPDFNet**（Ceva，arXiv 2512.16420） | github.com/ceva-ip/DPDFNet · `resources/DPDFNet` | Apache-2.0【本机】`LICENSE:1-3` | huggingface.co/Ceva-IP/DPDFNet（`onnx/*.onnx`，已下 6 个到 `resources/weights/dpdfnet/`，sha256 见 exp001） | Apache-2.0【官方】模型卡；训练数据未披露、训练代码未开源 | 要比的 | 逐条核查 | 活跃：push 2026-10-20，release v0.6.0（2026-07-05），维护者回 issue【本机】`gh` |
| **DeepFilterNet3 / libDF** | github.com/Rikorose/DeepFilterNet · `resources/DeepFilterNet` | MIT 或 Apache-2.0【官方】`README.md:323-328` | 仓内 `models/DeepFilterNet3_onnx.tar.gz`、`DeepFilterNet3_ll_onnx.tar.gz` | **未写明**：README 只说 "All code" 双许可；issue #697（2026-07-15）、#712（2026-09-21）问权重能否商用，维护者无回复【本机】`gh issue view` | 要比的 + 零件（STFT） | 逐条核查 | **停更**：最后提交 2024-10-17，最后 release v0.5.6（2023-08-31）；67 个 open issue 无人回【本机】 |
| **GTCRN**（ICASSP 2024） | github.com/Xiaobin-Rong/gtcrn · `resources/gtcrn` | MIT【本机】`LICENSE:1-3` | 仓内 `stream/onnx_models/gtcrn.onnx`（DNS3 权重）；`checkpoints/*.tar` | **MIT，作者书面确认**覆盖预训练 ONNX、可商用与再分发【官方】issue #104（作者回复，2026-10-09 关闭）；训练数据许可未写 | 要比的 | 逐条核查 | 低频活跃：最近提交 2026-08-03，无 release，作者仍回 issue |
| GTCRN 的 Rust LADSPA 插件 | github.com/biglinux/gtcrn · `resources/biglinux-gtcrn` | MIT【本机】`ladspa/Cargo.toml:6` | 同 GTCRN，另导出 `gtcrn_vctk.onnx` | 同上 | 要学的（Rust 逐帧 + ort 范本） | 读架构 + 关键行 | 只有 1 个提交（2026-04-27） |
| **UL-UNAS**（GTCRN 同组，TASLP 2026）· 新增候选 | github.com/Xiaobin-Rong/ul-unas · `resources/ul-unas` | MIT【本机】`LICENSE:1-3`（2026-02-09 才建，权重 2026-02-01 已上传） | 仓内 `ulunas_onnx/onnx_models/ulunas_stream_simple.onnx`（789 KB） | 仓库 MIT 覆盖，但**无 #104 那样的权重声明**；训练用了 DiDiSpeech 普通话，许可未核【传言】 | 要比的（GTCRN 的升级位） | 逐条核查 | 最近提交 2026-06-17 |
| **ort**（ONNX Runtime 的 Rust 绑定） | github.com/pykeio/ort | MIT 或 Apache-2.0【本机】Cargo.toml | — | — | 零件：**用** | 一两行 | 很活跃，但只有 rc：2.0.0-rc.15（2026-10-19）【本机】crates.io |
| **tract**（纯 Rust 推理） | github.com/sonos/tract | MIT 或 Apache-2.0【本机】 | — | — | 零件：**备用** | 一两行 | 活跃：0.23.9（2026-10-15）；libDF 锁在 0.21.4 |
| **nnnoiseless**（RNNoise 的 Rust 移植） | github.com/jneem/nnnoiseless · `resources/nnnoiseless` | BSD-3-Clause【本机】`Cargo.toml:6` | 仓内 `src/weights.rnn`（RNNoise 原版） | 推断同 BSD-3（COPYING 未单独提权重）【本机推断】 | 零件：**只参考 + 垫底基线** | 一两行 | 低频：0.5.2（2025-12-18） |
| aTENNuate | arXiv 2409.03377；官方 GitHub Brainchip-Inc/aTENNuate **现返回 404**【本机，2026-10-29】 | Apache-2.0（PyPI `attenuate`、HF PeaBrane/aTENNuate） | HF PeaBrane/aTENNuate | Apache-2.0 | 排除 | 一两句 | 官方仓失联 |
| FRCRN（阿里） | ClearerVoice-Studio；HF alibabasglab/FRCRN_SE_16K | Apache-2.0 | HF | Apache-2.0 | 排除 | 一两句 | — |
| MP-SENet | arXiv 2305.13686 | MIT | — | — | 排除 | 一两句 | — |

### 逐个核查要点（全部带出处；被怀疑派推翻的已改）

#### DPDFNet
- **A 延迟**：因果；窗 20 ms、hop 10 ms，`df_lookahead=2`、`conv_lookahead=2`【本机】`onnx_model/dpdfnet.py:547-548`。默认 `mask_method='before_df'`（`dpdfnet.py:553、783-787`）让深度滤波作用在**已经乘过掩码**的频谱上，两段延迟因此**串联**（DFN3 的 `df_op(spec.clone())` 用的是没乘掩码的频谱，两段并联取较大值，`DeepFilterNet/df/deepfilternet3.py:431、442`）：mask 缓冲延 2 帧（`onnx_model/layers.py:20-25、98-104、377-383`），DF 用 5 帧缓冲取中间那帧、系数再延 2 帧（`multiframe.py:162-176、221`）→ **输出比输入落后 4 帧**。训练版 `model/dpdfnet.py:545-555` 同样先掩码后 DF，换权重也压不下来（怀疑派独立复推确认）。官方推理脚本自己按 4 个 hop 对齐（`infer_dpdfnet_onnx.py:18` `ATTN_LIMIT_NOISY_FRAME_OFFSET = 4`、`:177-181`、`:231-232` 裁掉 `win_len*2`）。
  - 算法延迟：按「窗 + lookahead」口径 **60 ms**；按「(窗−hop) + lookahead」口径 50 ms，与第三方 CoreML 模型卡「fixed algorithmic delay 2,400 samples / 50 ms」一致【传言】huggingface.co/iky1e/DPDFNet8-48kHz-HR-CoreML；按 DNS 口径（窗 + hop + lookahead）70 ms。
  - 官方说法不全：README/模型卡「首个输出约 20 ms」只是攒满第一个窗【官方】`README.md:118-120`；维护者在 issue **#14**（不是 #7）说「overall latency of about 40 ms」，并说零 lookahead 变体「possible」但质量有取舍、无时间表【官方】。
- **B 采样率**：8 k / 16 k（baseline、2、4、8）/ 48 k 全频带（2_48khz_hr、8_48khz_hr）【官方】`README.md:34-56`。
- **C 算力**（参数 M / GMACs）：baseline 2.31/0.36 · dpdfnet2 2.49/1.35 · dpdfnet4 2.84/2.36 · dpdfnet8 3.54/4.37 · 48k_hr2 2.58/2.42 · 48k_hr8 3.63/7.17【官方】模型卡。**本机实测**（M5，ORT 单线程，真实语音，模型 + STFT 合计 max ms/帧）：0.56 / 0.89 / **1.40** / 2.52 / 1.25 / 3.99【本机】exp001。
- **D ONNX**：单帧图，输入 `spec[1,1,F,2]` + `state_in[S]`，输出 `spec_e` + `state_out`；GRU、DPRNN、卷积缓存、DF 缓冲、归一化统计量**全部在一条显式 state 向量里**；ERB 特征与归一化在图内，**STFT/iSTFT 在图外**；opset 17；元数据带 `state_size` 与归一化初值【本机】`export_dpdfnet_to_onnx.py:26-30、64-88`，`infer_dpdfnet_onnx.py:47-67、87-109`；state 尺寸 baseline 38256 … dpdfnet4 52592 … 48k_hr8 90228【本机】exp001。已发布 ONNX 里**没有 Einsum**（已降成 MatMul），有原生 GRU 节点【本机】`grep -a`。
- **E Rust 可复用**：官方仓无 Rust。libDF 的 STFT/iSTFT 可复用（vorbis 窗与 wnorm 公式相同，`libDF/src/lib.rs:127-133、356-427` vs `DPDFNet/model/utils.py:153-167`），ONNX 包装层已经在图内**输入乘 wnorm、输出乘 1/wnorm**（`export_dpdfnet_to_onnx.py:26-30`），所以**自写 STFT 时直接喂未缩放的 rfft，irfft（realfft 逆变换不归一化，要 ÷N）后乘窗做普通 OLA，不要再乘 wnorm**；vorbis 窗 50% 重叠满足 w²+w²=1，不用除包络（官方 `package/src/dpdfnet/stream.py:119-153` 就是这么做的）。只有借 libDF 的 `frame_analysis` / `frame_synthesis`（libDF 只在分析侧乘 wnorm，`lib.rs:389-392`；逆变换未归一化，`:401-409`）时，才要「进图前 ÷wnorm、出图后 ×wnorm」。libDF 的 ERB/归一化/tract 运行时**用不上**（都在 DPDFNet 图里，且 alpha 是 0.98 不是 0.99）。第三方 Rust + ort 实现 HushMic（github.com/Fovty/HushMic，自称 80 ms）【传言，未复核】。
- **F**：代码与权重 Apache-2.0；EvalSet 也是 Apache-2.0【官方】。
- **G 低 SNR 证据**：作者自建低 SNR 集（12 语种，SNR 0/5/10 dB，9 种场景，含 pub≈babble）上，PESQ / SI-SNR：DPDFNet-4 **3.18 / 14.11**，DPDFNet-2 3.14 / 13.72，DFN3 2.76 / 12.10，DFN2 2.59 / 11.95，GTCRN 2.21 / 9.11【官方】arXiv 2512.16420v3 Table 1。**没有**分 SNR 档或 babble 单列；全文不出现 babble；只有 NPU RTF（0.97），无 CPU 数字。
- **ONNX 元数据键**（本机 `strings` 核对已发布文件）：`sample_rate`、`n_fft`、`hop_length`、`window_type=vorbis`、`center=1`、`pad_mode=reflect`、`state_size`、`erb_norm_state_size`、`spec_norm_state_size`、`erb_norm_init`、`spec_norm_init`（`export_dpdfnet_to_onnx.py:64-88`）。`center`/`pad_mode` 描述的是离线路径，流式实现忽略；state 各段长度按型号不同（16k erb 段 32、48k_hr 481），只能从元数据读。输入是 [−1, 1] 浮点，不乘 32768（`infer_dpdfnet_onnx.py:284-290`）。
- **I 坑**：state 不能全零起步，要按元数据填归一化初值（`infer_dpdfnet_onnx.py:47-67`；exp001 第二次跑即按此，0 个 NaN 帧）；干湿混合/衰减下限要把原声延后——频域延后 4 帧，时域（OLA 之后）延后 4 × hop + (窗 − hop)，16 kHz 下 800 样本；导出的 DPRNN 块数必须和权重一致（issue #8）；流式包 `center=False` 与离线 `center=True` 不逐位一致（`package/src/dpdfnet/stream.py:19-27`）→ Rust 实现的对拍基准用 `StreamEnhancer`（`stream.py:117-156`），不用 `infer_dpdfnet_onnx.py`；`StreamEnhancer` 每块调用无状态的重采样（`stream.py:112`），会有块边界伪影，别照搬，对拍时直接喂模型采样率的音频。

#### DeepFilterNet3 / libDF
- **A 延迟**：48 kHz，窗 20 ms、hop 10 ms。默认版 `conv_lookahead=2`、`df_lookahead=2` 取**较大值**（并联，`libDF/src/tract.rs:309-316`）→ 论文口径 **40 ms**【官方】arXiv 2305.08227 §3；**_ll 版两者都是 0 → 20 ms**【本机】两个 tar.gz 里的 `config.ini`；`ladspa/README.md:3`。DNS 口径：默认 50 ms，_ll 30 ms。
- **_ll 不是「去掉 lookahead 的同一模型」**：结构更大、单独训练（emb/df 隐层 512 vs 256、df 3 层 vs 2 层、`convt_depthwise=True` 等），按文件大小估约 9.75 M 参数 vs 默认约 2.15 M【本机，估算】`config.ini` diff。**官方没有 _ll 的 MACs 或 RTF**；默认版只有 RTF 0.19（i5-8250U 单线程）【官方】。
- **D ONNX**：拆成 enc / erb_dec / df_dec 三个子图，**状态不显式**，时间维符号化，靠 tract-pulse 转成逐帧（`tract.rs:797、864、965`）；STFT、ERB、归一化、深度滤波全在 Rust 侧【本机】。→ 预判「很多开源 ONNX 没把状态做显式」**对 DFN3 成立**。
- **E Rust 可复用**：最完整——STFT `lib.rs:356-427`、ERB 滤波器组 `lib.rs:68-101`、归一化 `lib.rs:206-259`、深度滤波 `tract.rs:724-767`、逐帧主循环 `tract.rs:509-640`、C API `capi.rs:83、161`【本机】。但**不实时安全**：每帧深拷贝 `spec_buf` 两次（`tract.rs:541-542`）、`into_tensor().permute_axes` 复制（`:462`）、`Tensor::zero`（`:491`）【本机】；锁在 tract 0.21.4（现 0.23.9）。
- **I 坑（对本项目是硬伤）**：lsnr < −10 dB 时**整帧输出静音**，两个解码器都不跑；lsnr > 30 dB 原样输出；连续 6 帧静音后连编码器也跳过（`tract.rs:176-185、441-505、513-524、658-672`）【本机】——低 SNR / babble 正是 intent 的「真难题 1」，默认阈值会直接「吃字」，必须改或关。C API 只暴露 `set_atten_lim`、`set_post_filter_beta`，改阈值要动源码。被跳过的解码器状态不前进、与编码器错帧（推断，未实测）。
- **G**：只有 VB-DEMAND 整体 PESQ 3.17【官方】Table 1；无分 SNR、无 babble。DPDFNet 论文在其低 SNR 集上测得 DFN3 PESQ 2.76（见上）。
- 社区分叉：grazder/DeepFilterNet（torchDF，单图流式 ONNX、状态显式，但 hop 改成 512 并微调过，许可 NOASSERTION）；Shuichi346/DeepFilterNet3-VST3（Rust，MIT，push 2026-10-12）【传言，未克隆】。

#### GTCRN（+ biglinux Rust 插件）
- **A**：16 kHz，窗 512（32 ms）、hop 256（16 ms），sqrt-Hann，**lookahead 0**；时间卷积左补零，跨帧 GRU 单向【本机】`stream/gtcrn_stream.py:371、455`，`gtcrn.py:112、145`。窗口径 32 ms ✅，DNS 口径 48 ms ❌（DNS 原文就拿 32/16 举例为不合格，见下）。biglinux 插件另有 VAD 前瞻端口默认约 20 ms（`ladspa/src/lib.rs:45-46、150-155`），照搬要设 0。
- **C**：48.2 K 参数、33 MMACs/s（README 口径；论文 23.7 K / 39.6 M）【官方】`README.md:14、19`；**本机实测最坏一帧 0.62 ms / 16 ms 帧**（含 STFT）【本机】exp001。
- **D**：流式 ONNX 现成，`mix[1,257,1,2]` + `conv_cache` / `tra_cache` / `inter_cache` **显式**；STFT 在图外，ERB、掩码在图内，无跨帧归一化统计量【本机】`gtcrn_stream.py:321-350、412-418`；exp001 已按此逐帧回灌跑通（真实语音，0 个 NaN 帧）。
- **E**：biglinux 是 **Rust + ort 2.0.0-rc.11**：状态管理与单帧推理 `ladspa/src/model.rs:97-128、248-319`，sqrt-Hann STFT `stft.rs:326-460`【本机】。附带启发式（warm-up 20 帧、回退到参考状态 `model.rs:138-149、228-271`；输入 RMS 归一化最多放大 3 倍 `plugin.rs:47-73`；双模型切换），**都没有评测**。
- **G**：VCTK PESQ 2.87；DNS3 合成测试集 PESQ 2.10 / SI-SNR 11.44；DNS3 盲测 DNSMOS P.808 3.44（RNNoise 3.15）【官方】`README.md:25-42`；无分 SNR、无 babble。「伤人声」只有盲测集 DNSMOS-SIG 3.00 < 含噪 3.20 这一条弱证据（RNNoise 也是 3.00）。issue #102 是提问啸叫抑制，**不是**流式不稳定的 bug（核查员原判被推翻）。
- **结论**：同算力档里最强一类，但绝对效果明显低于 DPDFNet / DFN3（DPDFNet 论文低 SNR 集 PESQ 2.21 vs 3.18）。

#### UL-UNAS（新增）
- 与 GTCRN 同框架（512/256 Hann、LA 0、16 kHz），171 K 参数、35 MMACs/s【官方】arXiv 2503.00340 Table VI；流式 ONNX 状态显式（3 个打平的 cache），仓内 ONNX 已载入 DNS3 训练权重【本机】`ulunas_onnx/stream/ulunas_stream.py:285-291、347-358`。
- DNS3 合成测试集 PESQ 2.30 / SI-SNR 12.30（GTCRN 2.10 / 11.44，DFN 2.13 / 12.20），但 DNSMOS OVRL 3.05 略低于 DFN 3.07；VCTK 上 SI-SNR 略低于 GTCRN【官方】Table VI、VII。
- 坑：Hann 50% 重叠两次加窗平方和不恒定，自写 OLA 要按包络归一化；权重无专门声明；未实测耗时。

#### 零件
- **ort**：macOS arm64 默认下载静态链接 ORT 1.31.0（压缩约 9.7 MB），带 CoreML EP；有 IoBinding 预分配输出、单线程设置【官方】ort.pyke.io；**普通 `run` 每帧返回新张量**，音频回调里不能直接调，要放推理线程 + IoBinding，并实测 malloc。只有 rc 版，要锁版本。
- **tract**：注册了 GRU/LSTM/Conv/ConvTranspose/STFT 等算子【本机】`gh api` 读源码；pulse 可把整段模型转逐帧；具体模型能否加载需冒烟。
- **nnnoiseless**：48 kHz、帧 480、窗 960、约 10 ms 延迟【本机推断】`src/lib.rs:36-41`、`features.rs:262-275`；逐帧 API `DenoiseState::process_frame`（`denoise.rs:95`），输入 ±32768；静音帧跳 RNN（`denoise.rs:101`）。价值：Rust 逐帧 API 的写法范本 + 最轻垫底基线。

#### 排除项
- **aTENNuate**：基础版 PreConv 是中心卷积，延迟 46.5 ms；无 PreConv 变体 16 ms 但 DNS1 PESQ 2.98→2.59【官方】arXiv 2409.03377v3 Table 2、附录 C.3；无 ONNX，官方仓 404。
- **FRCRN**：全频带版 12.3 GMACs/s 超预算；发布版 16k 配置是 40/20 ms 帧【官方】arXiv 2206.07293；ClearerVoice-Studio `FRCRN_SE_16K.yaml`。
- **MP-SENet**：时频 Conformer 全序列注意力，非因果【官方】arXiv 2305.13686 §3.1。
- 另查过 LiSenNet（官方无权重）、Fast-ULCNet（无权重）、FSPEN（无官方仓）——都不合格。

#### 延迟口径的原文（影响命门 1）
DNS Challenge：「The total algorithmic latency allowed including the frame size T, stride time Ts, and any look ahead must be ≤ 40ms」；例：20/10 ms 帧 = 30 ms 合格，32/16 ms 帧 = 48 ms 不合格【官方】`resources/DNS-Challenge/docs/ICASSP_2021_DNS_challenge.pdf` 第 2 节（evaltool 线克隆）；2022 版同措辞。DFN 论文用的是「窗 + lookahead」，比 DNS 少算一个 hop。

---

## 两类都不算数的地方
- **babble / 分 SNR 档的效果**：所有候选都没有公开数字 → 只能在 004 评测里自己测（见 gap.md「它们没做的」）。
- **DFN3 _ll 的算力**：无官方数字，ONNX 不显式状态，本次没测 → 待实测（要编译 libDF，属重活）。
