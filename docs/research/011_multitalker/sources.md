# 两类来源 · 一大堆人同时说话

> 2026-10-30 · 手下 multitalker。取材：Workflow `wf_8be39da4-45d`（7 个只读调研代理，**没有构建或运行任何程序**；唯一的本机查询是 `system_profiler SPAudioDataType` 只读）。
> 七份原始发现（每条带出处、标记）放在 `findings/`，本文只做索引和要点，不抄全文。
> 标记：【官方】官方明载 · 【论文】 · 【源码】源码 / 头文件可证 · 【本机】本机只读查询 · 【推断】 · 【传言】营销稿、论坛、第三方转述。

## 一、自有积累

| 是什么 | 在哪 | 还算数吗 |
|---|---|---|
| exp003：邻桌 6 人 / 旁边有人说话的 8 段，三个通用模型几乎全「不能用」且吃字 | `docs/experiments/exp003-offline-ab/results/` · D34 | 算数，是本题的起点 |
| 005 PSE：自写注册器 + 不对称门控（第 1 步）、VoxLock 超预算、Teams 提示用户自己关 | `docs/research/005_pse/decision.md` · `sources.md:84` | 算数；本线不重复调研这些候选 |
| 006：Apple Voice Isolation 当商业对照（要人手动开） | `docs/research/006_sota-landscape/decision.md:72` | 算数，本线把它并进实验建议 |
| D40 / D41 场景路由与两层场景 | `docs/plan/005_evolution-and-simulation.md` | 算数，本线的「多人同用」识别挂在判断回路上 |

## 二、他人积累（按问题）

| 问题 | 原始发现 | 最要紧的几条证据 |
|---|---|---|
| 1 证据：人数 / 信噪比 / 远场 | `findings/evidence.md` | 2→3→4 人提升量 SI-SDRi 8.75→6.82→5.34 dB（输出 SI-SDR 13.8→7.2→2.9；arXiv 2306.16241 表 1，非因果、只用 2 人数据训练）【论文】；「2 人 + 噪声」比纯 2 人难（SEF-PNet 8 kHz 离线，输出 SI-SDR 13.00 vs 7.54，提升量 13.0 vs 9.6，`resources/SEF-PNet/README.md:38-120`）【源码】；真实会议 2→4 人 TER 43.5→78.7%（REAL-T 表 4）【论文】；有干扰人时通用降噪让 WER 变差 36.4→40.1%（arXiv 2110.09625 表 1）【论文】；人耳对 6–128 人 babble 最难（Simpson & Cooke 2005）【论文】；**没有任何论文把 6 人以上 babble 与单个干扰者放进同一张表** |
| 2 macOS 多路麦克风 | `findings/macos-mic.md` | 内置麦 HAL 只给 1 声道【本机】（Mac17,3）· Apple 规格「三麦阵列 + 定向波束成形」【官方】；VPIO 下混成单声道（`resources/cubeb-coreaudio-rs/src/backend/mod.rs:3921`）【源码】；「Voice Isolation and Wide Spectrum take priority over Bypass」（SDK `AudioUnitProperties.h:2630`）【官方】；麦克风模式只能由用户在控制中心选，app 只能弹面板（`AVCaptureDevice.h:2741-2792`）【官方】；语音突显只给用了 Apple 语音处理的 app（WWDC23 10235）【官方】；cpal 输入走 HAL Output 单元、不用 VPIO（`resources/cpal/src/host/coreaudio/macos/device.rs:67,82-84`）【源码】；ReSpeaker 6ch 固件有 4 路原始（厂商 wiki，已停产）【官方】 |
| 3 近讲 / 远讲 | `findings/nearfar.md` | 单通道盲估 DRR 在 ACE 上 RMSE 约 3.9 dB（arXiv 1606.03365 表 7–8）【论文】；盲估 C50 在实录上 8 个模型只有 3 个胜过「永远输出均值」（Gamper 2020）【论文】；单麦近 / 远分离 1 近 4 远只涨 4.4 dB（arXiv 2207.00562，无代码）【论文】；Hush 训练 SIR 12–24 dB，不覆盖 −5 dB（`resources/Hush/README.md:76,334`）【源码】；双麦 CDR 可逐帧、算力极低，但没有开源实现（arXiv 1502.03784）【论文】 |
| 4 波束成形 | `findings/beamform.md` | 3 cm 双麦、0 dB 干扰说话人（仿真房间）：DAS / GSC / JNF 的 PESQ 2.08→2.09–2.12，神经 CDUNet 2.50（arXiv 2412.18141 表 I）【论文】；双通道 GTCRN 低 SNR STOI +7–13 点、无权重（arXiv 2505.19597 表 2）【论文】；WebRTC 旧波束器 2018 年已删【源码】；sonora 无波束模块（`resources/sonora/crates/sonora/src/audio_converter.rs:5`）【源码】 |
| 5 会议室模式 | `findings/meeting.md` | diart 每步 165 ms（20 核 Xeon），超预算（arXiv 2109.06483 §4）【论文】；Sortformer 117M、按 GPU 标定、最多 4 人【官方】；Teams 检测到「身边有人」时**提示用户**关 voice isolation，不自动切【官方·转引】 |
| 6 商业产品 | `findings/commercial.md` | 默认档全行业都是「去噪、保留人声」；「只留我」靠声纹（Teams、Zoom）或近讲假设（Krisp 要求耳麦，Webex 要求 ≤ 1 m + 2 s 说话人识别）【官方】；Webex 另有「Optimize for all voices」给会议室【官方】；Krisp 对内置麦默认不开 BVC【官方】 |
| 7 提示体验 | `findings/ux.md` | 主流会议软件没有「你那边噪音大」提示【官方 / 未查到】；Meet 一次性提示每场最多一次【官方】；HIG：只告知信息不用 alert、不重复通知【官方】；断点再打断挫败感更低，平均延后约 90 s（Iqbal & Bailey CHI 2008）【论文】；每多一次提醒接受率降 30%（Ancker 2017）【论文】 |

## 三、站不住或要降级的说法

| 说法 | 处理 |
|---|---|
| 「Mac 内置麦 HAL 给的 1 路就是 Apple 波束成形后的信号」 | **【推断】**：由 1 声道【本机】+ 规格页「定向波束成形」【官方】+ Asahi 作者的说法【传言】推出；Apple 没写哪一层做 |
| 「内置麦有 4 声道格式」 | Apple 帮助页一句话提到【官方】，内容不详，要只读查看 Audio MIDI 设置确认 |
| 「AirPods 通话 24 / 48 kHz」 | 【传言】，未见官方 |
| 「Krisp 靠近讲线索判主讲人」 | 【官方博客】，无数字 |
| `tse-conv-tasnet-48k`（HF） | 页面 401，未核，不登记 |
