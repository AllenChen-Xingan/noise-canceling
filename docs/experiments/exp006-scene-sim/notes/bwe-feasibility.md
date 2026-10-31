# libopus BBWENet（OSCE BWE）用于 vdn 16 kHz → 48 kHz 带宽扩展：可行性调研

日期 2026-10-30 · 只调研不实现 · 标注：【官方】官方文档/发布说明 ·【源码】xiph/opus 源码 ·【论文】论文 ·【推断】我的推断（与前三类分开看）

取材：
- xiph/opus `main` 浅克隆，HEAD `bafd40dcbd`（2026-10-29），位于 `resources/opus/`（worktree 下，已被 `.gitignore:12 resources/**` 忽略，未入库）。下文 "源码" 行号均指此 HEAD。
- 官方发布包 `opus-1.6.tar.gz`（sha256 `b7637334527201fdfd6dd6a02e67aceffb0e5e60155bbd89175647a80301c92c`，36 MB），解在 scratchpad `opus16/`。
- crates.io 下载 `opusic-sys 0.7.6` 源码包，解在 scratchpad `crates/opusic/`。
- 没能做 CPU 实测：在 scratchpad 里 `configure --enable-osce` 编译 `bwe_demo` 的命令被权限分类器拦下（外部代码），所以 CPU 只有论文给的 FLOPS 数和推断。

---

## 1. 它是什么、在哪个版本、是不是公开 API

| 结论 | 来源 |
|---|---|
| libopus **1.6**（2025-12-15）发布说明列出 "A new wideband-to-fullband bandwidth extension (BWE) module" | 【官方】https://opus-codec.org/release/stable/2025/12/15/libopus-1_6.html |
| 当前最新标签 **v1.6.1** | 【源码】`gh api repos/xiph/opus/tags` |
| 引入提交：`44f448b2ff` 2025-03-15 "BBWENet C implementation + decoder integration" | 【源码】GitHub commits API（path=`dnn/bwe_demo.c`） |
| 定位：实验性的宽带→全带语音增强器，用 0–8 kHz 生成 8–20 kHz，**无旁信息**（blind）；输出 48 kHz；只对 SILK 宽带模式生效；不处理窄带→宽带 | 【官方】https://opus-codec.org/demo/opus-1.6 |
| 对**未编码**语音，BWE 大约补上宽带与全带之间一半的质量差 | 【官方】同上 demo 页 |
| 论文：Büthe & Valin, "A lightweight and robust method for blind wideband-to-fullband extension of speech", WASPAA 2025 | 【论文】https://arxiv.org/abs/2412.11392 |
| 参数量 **~370 K**，复杂度 **~140 MFLOPS（~70 MMACS）** | 【论文】同上，摘要 |
| 发布包权重：`dnn/bbwenet_data.c` 由 checkpoint `bbwenet_v2.pth` 自动生成；按数组统计约 37.8 万个参数（与论文 370 K 一致） | 【源码】`opus-1.6/dnn/bbwenet_data.c:1`；统计为【推断】 |
| 输入 16 kHz int16，**每次必须 160 或 320 样本**（10/20 ms），输出 3 倍长度 48 kHz int16 | 【源码】`dnn/osce.c:1478-1497`（`celt_assert(xq16_len == 160 \|\| xq16_len == 320)`） |
| 结构：特征（20 ms Hann 窗、10 ms 跳的 STFT：32 带对数谱 + 41 bin 瞬时频率 = 114 维）→ 特征网 Conv1d×2 + 转置卷积 + GRU(128) → 信号网：AdaConv(16 kHz, 1→3 通道) → IIR 2× 上采样 → AdaShape + 非线性 → AdaConv(32 kHz) → FIR 1.5× 插值 → AdaShape → AdaConv(48 kHz, 3→1)。AdaConv 的卷积核按 200 Hz（5 ms）更新 | 【源码】`dnn/osce_config.h:59-64`、`dnn/osce_features.c:461-500`、`opus-1.6/dnn/bbwenet_data.h`；【论文】§2 |
| 训练数据：多个高质量 TTS 语料，900+ 说话人 34 种语言；20% 片段加平稳宽带噪声，"教模型不扩展噪声、只扩展语音" | 【论文】§3.1 |
| **只在解码器路径**：`silk/dec_API.c:398-431` 里，当 `osce_extended_mode == OSCE_MODE_SILK_BBWE` 时用 `osce_bwe()` 替代 `silk_resampler()`，切换时 10 ms 交叉淡化 | 【源码】 |
| 公开 API 只有一个开关：`OPUS_SET_OSCE_BWE(x)` / `OPUS_GET_OSCE_BWE`（4054/4055），条件是解码采样率 48 kHz；解码复杂度须 ≥ 4 | 【源码】`include/opus_defines.h:176-177, 819-829`、`src/opus_decoder.c:451-452, 1076-1094`；【官方】demo 页 |
| **不存在**"喂任意 PCM 进来做 BWE"的公开函数；`osce_bwe()` / `osce_bwe_reset()` / `osce_load_models()` 声明在内部头 `dnn/osce.h:92-103`，状态结构 `silk_OSCE_BWE_struct` 在内部头 `silk/structs.h:252` | 【源码】 |
| 但仓里有独立示例 `dnn/bwe_demo.c`：读 16 kHz int16 raw PCM、每 320 样本调一次 `osce_bwe()`、写 48 kHz——正是"任意 16 kHz PCM"用法 | 【源码】`dnn/bwe_demo.c:50-92`；Makefile 目标 `Makefile.am:335-337`（noinst） |

## 2. 延迟与 CPU

| 结论 | 来源 |
|---|---|
| 论文：帧长 10 ms，前瞻（lookahead）**0.27 ms**，即 48 kHz 下 13 个样本（来自上采样滤波器） | 【论文】摘要、§1、§2 |
| libopus 实现里输出再加一个固定延迟 `OSCE_BWE_OUTPUT_DELAY = 21`（48 kHz 样本 = **0.4375 ms**） | 【源码】`dnn/osce_config.h:64`、`dnn/osce.c:1520-1535` |
| 这个 21 恰好等于 SILK 自带重采样器 16→48 kHz 的输入延迟 7 样本@16 kHz（`delay_matrix_dec[16][48] = 7`），目的是让 BWE 与普通重采样输出对齐、可交叉淡化 | 【源码】`silk/resampler.c:63-68`；"目的"为【推断】 |
| 特征提取只用过去 160 样本历史 + 当前 160 样本，无未来样本 | 【源码】`dnn/osce_features.c:488-497` |
| **成帧（framing）**：必须攒满 10 ms 才算一次，所以纯算法延迟 = 10 ms 成帧 + ~0.27–0.44 ms | 【论文】§1 "lookahead of 0.27 ms in addition to a framing delay of 10 ms"；【源码】同上 assert |
| 对 vdn：**FastEnhancer-M hop = 160**，与 BWE 的 160 帧完全对齐，成帧不额外加延迟；**GTCRN hop = 256（16 ms）**，要重新分块成 160，计算得最坏额外延迟 **128 样本 = 8 ms**（20 ms 帧则 16 ms） | 本仓 `src/models/src/fastenhancer.rs:3`、`src/models/src/gtcrn.rs:17`；8 ms 为【推断】（逐样本模拟 256→160 重分块，最大可用时刻差） |
| 若 BWE **替换**现有 16→48 kHz 重采样器（它本身就是个上采样器），净增延迟 ≈ 0.44 ms 减去现有重采样器的延迟，≤ 1 ms 预算内 | 【推断】 |
| CPU：官方/论文**没有**给出 CPU 占用或实时因子实测，只有 ~140 MFLOPS / ~70 MMACS | 【论文】；【官方】demo 页无 CPU 数字 |
| 粗估：70 MMAC/s 在 Apple Silicon 单核（libopus 有 NEON/int8 点积路径 `dnn/vec_neon.h`）上约 **0.5–2 % 单核**——比 vdn 其他环节（~0.1 %）高一个数量级，与 GTCRN 本身同量级甚至略高 | 【推断】，必须实测 |

## 3. 能否独立用在任意 16 kHz PCM 上？需要什么

- **可以，但走的是内部函数**。`bwe_demo.c` 已证明调用序列：`opus_select_arch()` → `osce_load_models(model, NULL, …)`（NULL = 用编译进库的权重）→ `osce_bwe_reset(state)` → 循环 `osce_bwe(model, state, out48, in16, 320, arch)`。【源码】`dnn/bwe_demo.c:59-90`、`dnn/osce.c:1427-1475`
- 构建开关：autotools `--enable-osce`（同时定义 `ENABLE_OSCE` 与 `ENABLE_OSCE_BWE`）或 CMake `-DOPUS_OSCE=ON`。不能与 `--enable-fixed-point` 同用。【源码】`configure.ac:883-898`、`CMakeLists.txt:433-436`
- 权重：发布包里**已带** `dnn/bbwenet_data.c`（9.6 MB C 源，编进库）；从 git 构建则需 `autogen.sh` 调 `dnn/download_model.sh <sha256>` 下载 135 MB 的 `opus_data-a5177ec6….tar.gz`（含全部 DNN 模型）。也可用 `USE_WEIGHTS_FILE` 运行时从 blob 加载。【源码】`autogen.sh:12`、`dnn/download_model.sh`、`dnn/osce.c:1449-1466`；大小为 HTTP HEAD 实测
- 要做的事：①写一个十几行的 C 薄壳（shim），对外只暴露不透明句柄 `vdn_bwe_new / vdn_bwe_process(160 in → 480 out) / vdn_bwe_free`，内部 include `dnn/osce.h`、`silk/structs.h`，从而避免在 Rust 里镜像内部结构体布局；②float↔int16 转换（`osce_bwe` 只吃 int16）；③16 ms→10 ms 重分块（仅 GTCRN 需要）。【推断】
- 内部符号在静态库里存在（只是没进公开头），静态链接到 Rust 可用；但它们**不是稳定 API**，跨版本可能改签名（如 1.6→main 已有 "fix error in error handling" 等改动 `5481f9d26a`）。【推断】+【源码】
- 另一条"只用公开 API"的绕路：把 16 kHz 输出先用 Opus SILK-WB 编码再用开 BWE 的解码器解——要承受编码器延迟与有损编码，**不可取**。【推断】

## 4. 许可

| 结论 | 来源 |
|---|---|
| libopus 为 3 条款 BSD（`COPYING`），文件头同为 BSD；`osce_config.h` 版权 Amazon（Jan Buethe）2023，同 BSD 文本 | 【源码】`opus-1.6/COPYING`、`dnn/osce_config.h:1-26` |
| 权重以 C 源形式随发布包分发，受同一 `COPYING` 约束（无单独模型许可） | 【源码】`opus-1.6/dnn/bbwenet_data.c`；"无单独许可"为【推断】（未见其他 LICENSE 文件） |
| BSD-3 与 GPL-3.0-or-later 兼容 | 【官方】FSF 许可证列表 https://www.gnu.org/licenses/license-list.html#ModifiedBSD |
| 专利：`COPYING` 指向 Xiph、Microsoft、Broadcom 等的 Opus 免版税专利许可，这些许可的范围是"实现 Opus 规范"。**把 BWE 拆出来单独处理非 Opus 音频，是否仍在专利授权范围内不确定**；BWE 本身也不在 RFC 6716 规范内（IETF 草案正把增强方法写成对 RFC 6716 的更新） | 【源码】`COPYING` 尾部；【官方】https://datatracker.ietf.org/doc/draft-ietf-mlcodec-opus-speech-coding-enhancement/ ；范围判断为【推断】，需人/法务定 |

## 5. Rust 选项

| crate | 版本（crates.io 查询于 2026-10-30） | 与 BWE 的关系 |
|---|---|---|
| `opusic-sys` | 0.7.6（2026-10-26），BSD-3 | **捆绑 libopus 1.6.1**（`opus/package_version`），有 `osce` feature → CMake `OPUS_OSCE=ON`，捆绑了 `bbwenet_data.c`，导出常量 `OPUS_SET_OSCE_BWE_REQUEST`。只覆盖公开 API，不绑定 `osce_bwe()`。【源码】crate 包 `build.rs:210-211`、`src/lib.rs:59-60`、`README.md` |
| `audiopus` / `audiopus_sys` | 0.2.0 / 0.2.2（2021） | 停更，捆绑旧版 libopus，无 OSCE |
| `opus` | 0.4.0（2026-08） | 高层安全绑定，无 feature 开关 |
| `opus2` / `libopus_sys` | 0.4.0 | 有 bundled/static，未见 OSCE feature |
| `opus-rs`（restsend） | 0.1.40 | 纯 Rust Opus 实现，未见 BWE |

推荐路径【推断】：不依赖上述高层 crate，自己用 `cc` crate 只编 BWE 需要的那几个文件（`dnn/osce.c`、`osce_features.c`、`nnet*.c`、`nndsp.c`、`bbwenet_data.c`、`celt/kiss_fft.c`、`mathops.c` 及 arch 分派文件等）+ C shim；或者依赖 `opusic-sys` 的 `osce` 静态库、在 shim 里调内部符号（省编译清单，但耦合到它的打包方式）。第三条长路：照着 C 源码在 Rust 里重写推理（约 37.8 万参数、结构清楚，纯 Rust 可行，但工作量最大）。

## 6. 替代方案

没有为"≤1 ms 前瞻的轻量实时 BWE"找到其他有可查开源实现与延迟数字的具体来源，这里不展开。经典非神经做法（频谱折叠 / 非线性激励 + 包络整形）原理上可做到近零前瞻、CPU 极低【推断】，可作为降级对照。

## 7. 结论

**可行，但有前提（feasible with caveats）。**

- 前瞻 0.27 ms（论文）/ 实现里 0.44 ms 固定输出延迟，**满足 ≤ 1 ms**；若用它替换现有 16→48 重采样器，净增更少。
- 前提 1：**10 ms 成帧**。FastEnhancer-M（hop 160）零额外延迟；GTCRN（hop 256）要重分块，最坏 **+8 ms**——是否计入"≤1 ms"要人定。
- 前提 2：**CPU** 约 70 MMACS，比其他环节高约一个数量级，"普通用户无感"必须实测。
- 前提 3：调用的是**内部、非稳定 API**，需要 C shim；**专利授权范围**（独立用于非 Opus 音频）不确定。
- 质量：训练在干净 TTS 语音上 + 20% 平稳噪声；降噪后的残留伪影是否会被"扩展"成可闻高频噪声未知（IETF 讨论也提到盲扩展"只保证听着合理，不保证正确"，见 IETF 126 slides https://datatracker.ietf.org/meeting/126/materials/slides-126-mlcodec-speech-coding-enhancement-01）。

**下一步（2–3 件）**
1. 有界小实验（需人放行外部代码编译）：用发布包 `--enable-osce` 编 `bwe_demo`，在本机跑 exp 现有 GTCRN / FastEnhancer 输出，测 ① 单核 CPU %（`time` / Instruments）② 与现有重采样器对齐的实测延迟（脉冲测试）③ 盲听 A/B（沿用揭盲表入库规矩）。
2. 待人拍板：① 成帧延迟算不算进 BWE 的 ≤1 ms 预算（决定 GTCRN 能否配 BWE）；② 专利 / 许可风险是否可接受；③ 走 `cc` + C shim 还是纯 Rust 重写。
3. 若实验通过：在 `exp/<NNN>-bbwe` 分支用 `cc` 编最小文件集 + 安全 Rust 封装（不透明句柄、预分配、实时线程无分配），并锁定 libopus 版本哈希。
