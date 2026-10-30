# 个人语音降噪项目：设计与参考汇总

整理自 2026-10-29 的对话"Mac 降噪麦克风方案"。
标注说明：**【你定的】** 是你本人提出或决定的；其余是对话里 Claude 给的建议和调研结论，尚未拍板。

---

## 目录

1. 背景与问题
2. 产品目标与定位
3. 用户痛点
4. 现有方案：谁做到了什么
5. 技术路线：模型
6. 系统架构
7. 工程解法：逐个痛点
8. 主模型的弱点与辅助模块
9. 音色：舒服的声音怎么做
10. 零录制个性化与环境自适应
11. 延迟与资源
12. 训练与微调
13. 评测体系与飞轮
14. 开源相关事项
15. 工作量估算
16. 建议的开发顺序
17. 短期应急方案
18. 参考项目与资料（含链接）

---

## 1. 背景与问题

在咖啡馆开会时，对方完全听不清你说话。macOS 的 Voice Isolation 解决不了。

**物理上限。** MacBook 内置麦克风离嘴约半米。咖啡馆的噪音和人声几乎一样大地到达麦克风。任何算法都只能二选一：留噪音，或者伤人声。麦克风离嘴的距离永远排第一。

**Voice Isolation 为什么不行。**

- 它假设你正对着麦克风。不正对内置麦克风时，声音发闷。
- 它只有开和关，不能调强度。苹果 API 只能检测模式，开发者不能设置。
- 它只为语音设计，只处理音频接口的第一个输入通道。
- 典型症状：机器人声、水下声、断续的门限效果、音量突变。

**最难的噪音是别人的说话声。** 咖啡馆里的噪音叫 **babble noise**。通用模型学的是"人声和非人声"的区别，所以会把邻桌一起保留。要压掉别人，必须知道哪个声音是你：这就是 **personalized speech enhancement (PSE)** 或 **target speaker extraction (TSE)**。

---

## 2. 产品目标与定位

### 你定的目标

- **【你定的】** 自己做一个 Mac 上的实时麦克风降噪 / PSE 产品。
- **【你定的】** 核心引擎用 Rust：低延迟、低资源占用、足够底层。用 Apple Silicon 加速。
- **【你定的】** 以开源实现为起点，但很多部分要深度优化。
- **【你定的】** 不需要用户录音，直接自动注册（zero-recording auto-enrollment）。
- **【你定的】** 面向个人用户的成品：上手快，开会时声音自然，效果好。
- **【你定的】** 目标用户是职场人、remote worker、WFH 的人。覆盖多种环境，不只是咖啡馆。
- **【你定的】** 理想音色：保留一定混响；清晰、完全音量；圆润、通透。
- **【你定的】** 边说边学用户的声音，像索尼耳机那样自动适应环境。
- **【你定的】** 可以做成完全开源的产品。
- **【你定的】** 专业音频工具（iZotope RX 等）的效果是参照标准。

### 定位（建议）

> Mac 上的开源语音处理引擎，让每个人开会时都有录音棚的声音。

把 Teams 的个性化、CEDAR 的低延迟、NVIDIA Studio Voice 的音色目标，做进一个 Mac 全系统可用、装上就能用的工具。

### 真正的对手

对普通远程工作者来说，对手不是 Krisp，而是 Zoom、Teams、Meet 自带的免费降噪。用户会问：会议软件已经有降噪，为什么还要装你的？

所以差异必须来自四点：

1. **全系统可用**：任何会议软件，同一个声纹，同一套效果。
2. **零注册**：不用朗读，开口说话就认出你。
3. **音色**：自然、圆润、通透，而不只是"去掉了噪音"。
4. **可靠**：不吃字，出问题时你能知道。

### 三条设计原则（借鉴专业音频圈）

1. **零设置**：自动注册声纹，不用录音，不用调参。
2. **重新平衡，而不是删除**：分层控制人声、环境声、混响，默认保留一点环境声。
3. **自动调参**：用实时测量替代坐在旁边调旋钮的音频工程师。这是相对专业工具和大众产品的真正差异点。

---

## 3. 用户痛点

**【你定的】** 你最初列的四点：声音不干净、需要提前训练、声音不自然、声音很奇怪。

### 重新整理

- **"不干净"和"不自然"是同一根绳子的两头。** 压得越狠越干净，也越不自然。目标是停在对的位置。
- **"不自然"和"奇怪"要分开。** 不自然是太干净：背景变真空、噪音底忽大忽小。奇怪是伪影（artifacts）：机器人声、水下声、发颤。
- **"需要提前训练"** 用自动注册解决，但带来新风险：锁错人。

### 漏掉的痛点

1. **吃掉字。** 用户评价里最集中的抱怨。漏一声杯子响，对方不在意；吃一个词，对方听不懂。设计原则是 **fail open**：不确定时宁可放噪音过去。
2. **说话的人听不到自己。** 出错时只有对方知道。代价由听的人付，选择却由说的人做。
3. **管线不可靠。** 用 15 到 30 分钟后对方突然听不到；虚拟麦克风选不上或自动关闭；Krisp 在 Discord 里误判 CPU 过载而自关；16kHz 输入时效果很差。这些是工程问题，却是流失主因。
4. **吃掉笑声和附和声（backchannel）。** 对方听到你一直沉默，对话变僵。几乎没有 benchmark 测它。
5. **延迟。** 多出 40 到 50ms 会打乱轮流说话的节奏。
6. **设置麻烦。** 每个 app 单独选麦克风；耳机一连，路由就乱。

### 排序（建议）

1. 吃掉字
2. 管线不可靠
3. 说话的人没有反馈
4. 伪影和不自然
5. 吃掉笑声和附和声
6. 设置麻烦

前三个决定用户会不会继续用。后三个决定用户会不会推荐。模型优化主要解决第 1、4 项；第 2、3、6 项靠工程和交互设计。

### 为什么大众产品普遍"差一口气"

- 默认参数按"平均用户"调，极端场景（低信噪比、人声噪音）就不行。
- 演示能展示"背景安静了多少"，展示不了"吃没吃字"，所以默认偏向压得狠。
- 反馈回路断了：用户说不清哪个词、什么噪音出了问题。
- 平台约束：Voice Isolation 要在所有 Mac 上省电运行，又不给开关。
- 商业重心转移：降噪成了大路货，Krisp 的营销转向会议转录和纪要。
- 没人为盲人用户设计：状态反馈都是视觉的。

---

## 4. 现有方案：谁做到了什么

### 商业与内置方案

| 方案 | 强项 | 局限 |
|---|---|---|
| Microsoft Teams Voice Isolation | 真正的 PSE，能去掉旁边人的声音；管理策略里已有 **PassiveVoiceEnrollment**（被动注册） | 只在 Teams 里；默认要朗读约 30 秒 |
| Zoom Personalized Audio Isolation | 2024 年 4 月 Zoom Workplace 6.0 推出的 PSE；官方针对办公室、咖啡馆 | 只在 Zoom 里；要朗读注册；找不到可靠的独立评测 |
| Zoom 普通降噪（Auto/Low/Medium/High） | 免费、零设置 | Auto 几乎没效果；High 会吃字；档位听不出区别；删掉音乐和音效 |
| NVIDIA Broadcast | 盲听常赢 Krisp | 只支持 Windows + RTX 显卡 |
| NVIDIA Maxine Studio Voice | 目标就是录音棚音色；有低延迟版 | 要 NVIDIA 显卡；面向企业开发者，付费 |
| CEDAR Voxis | 2026 年 8 月发布；延迟低于 10ms；同时去噪和去混响；分低中高三频段控制 | 专业插件，要宿主软件；$269；没有个性化 |
| Krisp | 普及最好；系统级；处理噪音、背景人声（BVC）、回声 | 吃字、失真、长时间运行后断掉 |
| macOS Voice Isolation | 全系统、免费 | 效果差、不能调 |
| Discord 内置降噪 | 用 Krisp 的模型 | 只在 Discord 里 |

### 专业音频插件（音色标杆）

- **iZotope RX Dialogue Isolate**：RX 11 起有实时版，降噪和去混响合一，延迟 2048 采样（48kHz 下约 43ms）。注意：做降噪的是 RX，不是 Ozone；Ozone 是母带工具。
- **Waves Clarity Vx**：一个旋钮，后期工程师日常用。
- **Acon Digital Extract:Dialogue**：有混音师认为调出来比 RX 更自然。
- **Supertone Clear**：三个旋钮，分别控制人声、环境声、人声混响。
- **CEDAR VoicEx / Voxis**：老牌标杆。2000 年就有延迟低于 0.2ms 的硬件对白降噪器。
- Gearspace 上一位工程师的排序：CEDAR VoicEx > Acon > iZotope > Supertone > Waves（个人偏好，仅供参考）。

### 免费开源方案

| 方案 | 做到的 | 缺的 |
|---|---|---|
| MetalVoice | Mac 上开源免费，DFN3 跑在 Neural Engine 上 | 要手动配 BlackHole；无个性化、音色、回声处理；个人小项目 |
| DeepFilterNet 本身 | 模型开源，带实时插件 | 实时插件只支持 Linux PipeWire；不是产品 |
| Linux 工具（NoiseTorch-ng、noise-suppression-for-voice、Easy Effects） | Linux 上可用 | Mac 上没有对应物 |

### 结论

- **零件全部存在**，大多开源：DeepFilterNet、DPDFNet、WeSpeaker、WebRTC AEC3、BlackHole。
- **组合不存在**。最接近的免费 Mac 方案 MetalVoice，大约只完成你设想的五分之一。
- **空白在三处**：Mac 上的零注册个性化；"重新平衡而不是删除"的自然音色；对普通人足够可靠的系统级管线。
- 每一块都有人做到了商业水准，所以方向已被验证。空白在组合与产品化。
- 注意：不能证明绝对没有人做，只是在当天能查到的范围里没有。

---

## 5. 技术路线：模型

### DeepFilterNet 的算法（基线）

作者 Hendrik Schröter 等，合作方是助听器公司，所以从一开始就为低算力、低延迟设计。论文：*DeepFilterNet3: Perceptually Motivated Real-Time Speech Enhancement*；相关：*Deep Multi-Frame Filtering for Hearing Aids*。

两步处理，都在频域：

1. **ERB 增益。** STFT 后把频谱合并成 32 个 **ERB bands**（模仿人耳的频率分辨率）。每个频带预测一个 0–1 的 mask，恢复 **spectral envelope**。很便宜。
2. **深度滤波（deep filtering）。** mask 只改幅度不改相位。所以对低频 96 个频点，每个预测一个 5 抽头的复数滤波器，跨相邻帧滤波，恢复谐波（harmonics）的精细结构和相位。

网络：可分离卷积 + **Squeezed GRU** 编码器，两个解码器。CoreML 版约 210 万参数，FP16 4.2MB，10ms 一帧。

**已知弱点：** 低信噪比时过度压制（over-attenuation）；长时间连续运行不够稳定。有 **attenuation limit** 参数：设在 12–20 dB，留一点底噪，声音自然得多。

### 实时模型对比

PESQ 在 VoiceBank+DEMAND 上测，论文间条件不同，只能粗比。

| 模型 | 参数量 | 计算量（每秒） | PESQ | 说明 |
|---|---|---|---|---|
| RNNoise (2018) | 0.06M | 0.04 GMACs | 2.29 | 老基线，最轻最弱 |
| GTCRN (2024) | 0.05M | 0.03 GMACs | 2.87 | 南京大学；和 RNNoise 一样轻，效果好很多 |
| DeepFilterNet3 | 2.13M | 0.344 GMACs | 3.16 | 延迟 40ms；实时模型公认基线；性价比甜点 |
| FRCRN | 6.9M | 38.1 GMACs | 3.21 | 比 DFN3 高一点，算力多 100 倍 |
| DPDFNet-2/4/8 | 2.49–3.54M | 1.35–4.37 GMACs | 自有评测集 | 16kHz |
| DPDFNet-8 HR | 3.63M | 7.17 GMACs | 自有评测集 | 48kHz 全频带；固定延迟 50ms |
| MP-SENet | — | — | 3.50 | 学术高分，幅度相位并行；不适合实时 |
| aTENNuate | — | — | — | Interspeech 2025；state space model 直接处理波形 |

### DPDFNet（推荐的主模型候选）

- 芯片公司 Ceva，2025 年 12 月发布。
- 在 DeepFilterNet2 编码器里加了 **Dual-Path RNN**。
- 训练加了两样：防止过度衰减的损失项；针对"一直开着"场景的微调阶段。正好修 DFN 的两个弱点。
- 提升主要在低信噪比，正是咖啡馆场景。
- 有 ONNX 和 TFLite 版本，Apache-2.0。官方推荐 CPU 上跑 ONNX。
- 文档要求在独立片段之间 reset 状态。

**建议：** Rust 路线用 `ort` + DPDFNet-4 ONNX，CPU 推理，效果与算力最平衡。

### 关于 benchmark 的内行要点

- **VoiceBank+DEMAND** 已刷饱和；合成数据上的高 PESQ 预测不了真人对真实录音的评分。
- **DNS Challenge**（微软，2020–2023）用真实录音 + DNSMOS；有 real-time track，总算法延迟 ≤ 40ms。还有个性化赛道。
- **URGENT Challenge**（ICASSP 2026）：universal speech enhancement，七种失真，十几个指标，最后真人 MOS 定名次。客观第一 GAP-URGENet 有 5.68 亿参数，不能实时。
- **排行榜顶部和你能用的模型是两个世界。** 通话降噪必须 **causal**，CPU 预算只有几毫秒。
- **不同论文的分数不能直接比。** 要在同一测试集上自己跑。

---

## 6. 系统架构

### 分工

- **训练和微调**：PyTorch。
- **推理和信号处理**：全部 Rust。
- **唯一接口**：导出的 ONNX，RNN 状态是显式的输入和输出。
- **一致性测试（parity test）**：同一段音频，Python 和 Rust 输出逐采样对比。STFT 窗函数、归一化差一点不会报错，只会悄悄变差。

### 模块划分（建议）

```
voice-engine/
├── dsp/        STFT、ERB、深度滤波、增益平滑、舒适噪音、衰减下限
├── engine/     逐帧调度、模型后端、掉帧回退到原声
├── speaker/    fbank、声纹、自动注册、带滞回的门控
├── io-macos/   CoreAudio、Audio Workgroup、设备监听、时钟漂移补偿
├── hal-plugin/ 虚拟麦克风设备
├── control/    命令行、状态查询、提示音
└── eval/       离线评测，跑和线上完全相同的代码
```

### 关键设计

1. **模型后端是可替换的 trait**：`tract`、`ort` CPU、`ort` CoreML。同一测试集上直接比速度和功耗。
2. **虚拟设备和推理分两个进程。** AudioServerPlugIn 跑在 `coreaudiod` 里，崩了整台电脑没声音。插件只从共享内存环形缓冲读数据。插件写得越笨，系统越稳。
3. **离线评测与实时运行用同一套代码。** `eval` 只是把 WAV 逐帧喂给同一个 `engine`。
4. **声纹在另一个线程。** 每 0.5–1 秒更新门控决策，用原子变量传给推理线程。推理线程从不等它。

### 完整信号链（建议）

```
             ┌─ 说话人验证 ─┐
             ├─ VAD ────────┤
麦克风 ──┬──→├─ 信噪比估计 ──┤──→ 控制器 ──→ 调节主模型参数
         │   ├─ 混响估计 ───┤          （衰减下限、干湿比、
         │   ├─ 输入分类器 ─┤            门控、模式）
         │   └─ 按键事件 ───┘
         │
         └──→ AEC ──→ 主降噪模型 ──→ 带宽扩展 ──→ 成品处理链 ──→ 输出
                                                       │
                                                质量监测（DNSMOS）
```

成品处理链（voice finishing）：

```
降噪 → 部分去混响 → 带宽扩展 → 自动 EQ（贴近目标曲线）→ 去齿音 → 轻度压缩 → 响度归一化 → 限幅
```

主模型负责"怎么处理"。辅助模块负责"现在是什么情况"。控制器据此调整处理强度。

### 关键 Rust crate

| crate | 用途 |
|---|---|
| `cpal` | 音频输入输出，底层 CoreAudio |
| `ort` | ONNX Runtime，可开 CoreML execution provider；用 IoBinding 预分配张量 |
| `tract` | 纯 Rust 推理，不依赖 C++ |
| `rtrb` | 无锁环形缓冲 |
| `rubato` | 重采样、时钟漂移补偿 |
| `realfft` | STFT |
| `libDF` | DeepFilterNet 官方 Rust 实现：STFT、ERB、深度滤波 |
| `nnnoiseless` | RNNoise 的纯 Rust 移植（参考，效果弱） |
| `webrtc-audio-processing` | WebRTC AEC3 绑定 |
| `assert_no_alloc` | 调试时检查音频线程不分配内存 |
| `objc2` | 直接调用 CoreML |

### macOS 系统层要点

- **Audio Workgroups**（macOS 11+）：推理线程加入音频设备的 workgroup，系统优先放到性能核。比盲目上 Neural Engine 更能解决卡顿。
- **process tap**（macOS 14.2+）：直接截取系统或某程序的输出声音，作为 AEC 的参考信号。
- **AudioServerPlugIn**：自己的虚拟设备，名字清楚（例如 "Allen Mic (Clean)"），可设为系统默认输入。程序不能替 Zoom 选麦克风，但大部分 app 跟随系统默认。
- **CoreAudio 属性监听器**：设备变化、采样率变化、蓝牙切 16kHz 时热重建音频流。

---

## 7. 工程解法：逐个痛点

### 吃掉字

- **衰减下限（attenuation floor）**：最多压 15 dB 左右，永不压到零。
- **快开慢关（fast attack, slow release）**：语音结束后保持 100–200ms 再压，保护词尾清辅音。电信行业叫 **hangover**。
- **增益时间平滑**：避免忽大忽小。
- **混入一点原声（dry/wet mix）**：5–10%。少量真实噪音掩盖伪影。

### 管线不可靠

- **掉帧时输出原声，而不是静音。**
- **用缓冲区水位控制重采样**：本质是 PI 控制器，标准的时钟漂移解法。
- **监听设备变化**，热重建。
- **定期清理 RNN 状态**：长静音时轻轻 reset。
- **按推理耗时判断过载，不看 CPU 占用率。**
- **soak test**：连续 8 小时，中途插拔耳机、切设备、睡眠唤醒。

### 说话的人听不到自己

- **不要实时监听处理后的声音。** 40ms 以上会形成 **delayed auditory feedback**，人会结巴。
- **一键回放**：回放"对方刚才听到的最后 5 秒"。
- **提示音**：判断有语音时却压掉大量能量，播放轻微提示音。对 VoiceOver 用户尤其自然。
- **后台质量检查**：每隔几十秒 DNSMOS 打分，掉分时语音提醒。

### 背景变成真空

- **comfort noise generation (CNG)**：VoIP 老技术，静音时注入轻微平滑噪音。配合衰减下限。

### 吃掉笑声和附和声

- 微调数据里放入笑声、"嗯"、"对"。
- 对很短的发声降低放行门槛。

### 设置麻烦

- 自己的 AudioServerPlugIn 虚拟设备，设为系统默认。
- 提示用户在 Zoom 打开"原声"模式，避免二次处理。

### 实时编程纪律

- 音频线程里不分配内存、不加锁、不做系统调用。
- **打开 flush-to-zero (FTZ)**：安静时 RNN 数值进入 **denormal** 范围，CPU 慢几十倍，恰在静音时卡顿。
- 所有缓冲区启动时分配，固定大小。
- 热点路径用 SIMD（`std::simd` 或 NEON）。

### 已知的坑

- **时钟漂移**：麦克风和输出设备是两个独立时钟，几分钟后缓冲溢出或耗尽。
- **蓝牙通话模式掉到 16kHz**，管线要能处理采样率变化。
- **WeSpeaker 输入是 80 维 fbank**，要和 Kaldi 行为一致，否则声纹偏。写完用 Python 版对比。
- **DPDFNet 的 ONNX 是否包含 STFT**，要先确认；不包含就复用 libDF。
- **很多开源 ONNX 导出没把 RNN 状态做成显式输入输出**，每次重算整段窗口，浪费算力。

---

## 8. 主模型的弱点与辅助模块

| 问题 | 主模型的弱点 | 辅助模块 |
|---|---|---|
| 别人的说话声 | 人声都保留 | 说话人验证（WeSpeaker ECAPA / ResNet）做门控或条件 |
| 回声 | 不处理；还会破坏 Zoom 的 AEC（AEC 假设线性路径） | WebRTC **AEC3**；神经方案 **DeepVQE**（AEC + 降噪 + 去混响合一）。参考信号用 process tap |
| 房间混响 | DFN 类主要针对噪音 | 部分去混响模块，或用带混响数据微调 |
| 键盘声 | 瞬态，实时模型不能看未来 | **监听系统按键事件**，在按键时刻精确压低。需"输入监控"权限；只用时间点，不记录按键内容 |
| 蓝牙窄带声音 | 只能清理，不能补回高频 | **带宽扩展 (BWE)**；URGENT 2026 冠军系统也加了这步，扩到 48kHz |
| 削波 | 不修 | **去削波 (declipping)** |
| 笑声和附和声 | 当成噪音 | 含非语言发声的 **VAD** |
| 极低信噪比、轻声说话 | 过度压制 | **信噪比估计器**，自动放宽衰减下限 |
| 已被处理过的输入（AirPods、Voice Isolation） | 叠加后变怪 | 小分类器判断是否已处理，减轻自己的处理 |

### 远程工作者场景的常见噪音（按普遍程度）

1. 键盘声
2. 房间混响
3. 家里其他人的声音（孩子、伴侣、电视）
4. 持续机械噪音（风扇、空调、洗碗机）——最容易，开源模型已做得很好
5. 突发噪音（狗叫、门铃、装修、车）
6. 咖啡馆和共享办公室——最难

另外两个易漏的问题：麦克风千差万别；输入可能已被处理过。

### 辅助模块优先级（远程工作者）

1. AEC
2. 说话人门控
3. 去混响
4. 键盘按键事件
5. 带宽扩展

---

## 9. 音色：舒服的声音怎么做

**【你定的】** 一定的混响；清晰、完全音量；圆润、通透。这是播客和广播行业追求的声音。

### "一定的混响"：保留早期反射，砍掉尾巴

- **早期反射（early reflections）**：约 50ms 内到达，和直达声融合，给厚度和空间感。
- **晚期混响（late reverberation）**：50ms 以后的尾巴，糊住音节，像在浴室。
- 指标 **C50**：早期能量与晚期能量之比。DNS Challenge 数据附带每段的 C50。
- 研究惯例：训练目标是**直达声 + 早期反射**，而不是完全干声。要的是 **partial dereverberation**。

### "完全音量"：响度稳定

- **响度归一化**，单位 **LUFS**。
- **轻度压缩**，比例别太高，避免抽吸感。
- **限幅器**防爆音。
- 别让降噪把人声本身拉低。

### "圆润通透"：频谱平衡

- **圆润**：100–300 Hz 的温暖与厚度。笔记本和蓝牙麦克风恰恰缺这段。300–500 Hz 过多会发闷。
- **通透**：2–5 kHz 的 **presence**（咬字）；8 kHz 以上的 **air**（亮度）。
- **齿音**：5–8 kHz，用 **de-esser** 压。
- **上游方法：LTASS**（long-term average speech spectrum）。最常引用的是 Byrne 等 1994 年跨 12 种语言的国际 LTASS。持续测量输出的长期平均频谱，自动 **match EQ** 贴近目标曲线。iZotope Ozone 的 Tonal Balance Control 是同一思路。

### 把"好听"变成可测指标

找几段你觉得最舒服的播客，测出 C50、响度、频谱曲线，作为默认目标。"好听"就变成了可以回归测试的数字。

---

## 10. 零录制个性化与环境自适应

### 为什么"录什么都行"在技术上成立

PSE 的核心是声纹向量（**speaker embedding**，常叫 **d-vector**）。说话人识别模型训练成 **text-independent**，只看嗓音，不看内容。产品要求念指定句子，是为了控制录音质量，不是技术必需。

真正影响效果的两点：录音要干净（只有你）；录音条件接近使用条件（同一麦克风、正常音量）。

### 已有的零录制/鲁棒注册研究

- **open-voicefilter-lite**：复现 Google VoiceFilter-Lite。LiveKit 插件在通话开头自动累计约 5 秒你的声音。
- **Pärnamaa & Saabas (2024)**：不用单独的说话人模型，用降噪模型内部表示当声纹，实时自动注册。
- **三星的 cross-attention 方法**：固定声纹有缺陷（音高、情绪一直变），用 cross-attention 从注册录音里动态挑帧；模型减半，效果更好。
- **USEF-PNet / DSEF-PNet (2025)**：通用降噪和个性化降噪合一；训练模型忽略注册录音的情绪和内容，随机长度注册效果略好。
- **Microsoft Teams PassiveVoiceEnrollment**：大厂也在往被动注册走。

### 自动认出你（建议的设计）

1. **冷启动用通用降噪**，先不压别人的声音。
2. **用物理线索判断主人：**
    - 离麦克风最近：更响、直达声比例高、混响少。
    - 对方没说话时你在说话：程序拿得到会议输出声音。
    - 同一个声音反复出现：声纹在线聚类，最频繁、最近场的簇就是主人。
3. **持续更新声纹**：高置信度片段加权平均融入；存多个原型（正常、大声、轻声、笑声）。
4. **个性化强度跟着置信度走**：开头很轻，几分钟后到位。用户只觉得"越来越干净"。

### 自动适应环境（像索尼耳机）

- 持续测量：底噪、信噪比、混响长度、噪音类型。
- 控制器调参：衰减下限、去混响程度、干湿比、个性化强度。
- 按地点记住参数：家、咖啡馆、办公室。
- **调节必须慢、必须平滑**：长时间常数 + 滞回（hysteresis）。突然跳变用户马上听得出。

### 要防的坑

- **锁错人**：开头几秒是邻桌在说话，就会锁定邻桌、删掉你。只在确信只有一人说话时注册。
- **压掉你自己**：声纹对不上时 PSE 削弱主人，比普通降噪失败更严重。DNS Challenge 有专门指标衡量目标说话人被过度压制。
- **声纹污染**：经常和伴侣同屋开会，可能把伴侣学进去。只用最高置信度近场片段；限制更新速度；保留初始声纹作锚点，偏离太远就拒绝。
- **换麦克风失准**：每个麦克风单独存一份声纹。
- **一键重置**：快捷键或命令清空重学。
- **隐私**：只存声纹向量，不存录音，全部本机。开源代码可证明。

### 不建议

- **test-time adaptation**（使用中更新模型权重）：可能慢慢学坏，难排查。只更新声纹向量就够了。

### 最简可行版本：说话人门控（speaker-aware gating）

先用 DPDFNet 去环境噪音，再逐段算当前声音与声纹的相似度，低就压低音量。不需要训练。缺点：分不开重叠语音。一个周末能做出来，Rust + `ort` 即可。完整版去看 **WeSep**（TSE 工具包，带预训练模型，可导出 ONNX；确认选因果模型）。

---

## 11. 延迟与资源

### 延迟从哪里来

主要来自**窗口长度和 lookahead**，不是计算慢。DFN3 的 40ms、DPDFNet-8 HR 的 50ms 几乎全是这两项。M 系列上计算远小于 1ms。

### 最大陷阱：模块串联会叠加延迟

降噪、去混响、带宽扩展各做一个模型，各有窗口和 lookahead，延迟三次相加，很容易破 100ms。

**解法：**

- 所有频谱处理共用一次 STFT。
- 多任务合成一个模型（DeepVQE 思路）。
- EQ 用最小相位 IIR（biquad），几乎零延迟；不用线性相位 EQ。

### 压低延迟的技术

- **非对称窗口（asymmetric windows）**：分析长窗、合成短窗。上游是 Mauler & Martin 2007 年前后的工作，低延迟助听器广泛采用。分帧延迟可压到几毫秒。
- **时域模型**（如 aTENNuate），无需 STFT 窗口。
- **缩短 lookahead**：需重新训练。

### 多低才够

- ITU-T **G.114** 建议单向总延迟 ≤ 150ms；网络和编解码已占 100ms 以上。
- 目标：**附加延迟 20ms 左右**。
- **稳定比极低更重要**：20ms 从不掉帧，好过 5ms 偶尔卡。

### 推理性能要点

- 预算：每 10ms 一帧，目标 2ms 内处理完。
- 预分配张量（`ort` IoBinding），每帧不分配内存。
- intra-op 和 inter-op 线程数都设 1。
- **看 p99.9 和最大值，不看平均值。** 一万帧里一帧超 10ms 就是一次爆音。
- **Neural Engine 不一定更快**：每帧调度 ANE 的开销可能超过计算本身。ANE 在这里主要是省电。先用 CPU，实测再说。
- 试 int8 量化，量化后在测试集上确认效果。

### 线程分工

- **推理线程**：加入 Audio Workgroup，性能核，只做降噪。
- **声纹线程**：utility QoS，可在能效核，每 0.5–1 秒算一次。
- **评分线程**：DNSMOS，最低优先级，偶尔跑。

### 功耗

- 一直开着的程序，耗电比速度重要。
- 用 `powermetrics` 测，目标整体 CPU 占用低于 2–3%。
- **真正耗电的是每 10ms 唤醒一次 CPU。** 每次处理 2 帧，用很小的延迟换明显更低的功耗。

### 估算的整条链路计算量

- 主模型（降噪 + 去混响合一）：0.5–2 GMACs/秒
- 带宽扩展：量级相近或更小
- 声纹验证：每秒一次，摊下来很少
- 经典 DSP：可忽略

---

## 12. 训练与微调

- **训练和推理是两件事。** 训练在 GPU 上跑几小时到几天，得到固定权重。开会时只做推理，权重不变。
- **自动注册也不是训练**，只是用固定的声纹模型算一个向量。
- **用户永远不需要训练。**
- **起步可以完全不训练**：先用现成 DPDFNet 权重，把工程做好。

### 什么时候微调

用场景矩阵找出通用模型弱的格子，再针对性收集数据微调，不盲目训练。

### 可能的微调方向

- 带混响数据 → 部分去混响
- 缩短 lookahead → 降延迟
- 带宽扩展
- 笑声、附和声保留
- 保留 DPDFNet 的防过度衰减损失项；一直混入通用数据防过拟合

### 公开数据来源

- DNS Challenge 训练数据
- 噪音：**FSD50K**、**MUSAN**
- 房间混响：公开 **RIR** 数据集
- **注意许可证**：有些数据集禁止商用，影响你发布的权重能否被商用。

---

## 13. 评测体系与飞轮

思路：**机器做筛选，人只听机器挑出来的少数片段。**

### 第一层：合成测试数据

- 干净语音 + 噪音 + 卷积 RIR。信噪比、混响时间、C50 精确可控。
- 模拟设备：麦克风频响、降采样到 16kHz、削波、自动增益。
- 场景矩阵用程序生成。参考 URGENT Challenge 的七种失真数据管线。

### 场景矩阵

- 噪音：键盘、混响、他人说话、机械噪音、突发噪音、咖啡馆
- 麦克风：内置、AirPods、USB 耳机、专业麦克风
- 语言：至少英语和中文

另外带上你的麦克风去咖啡馆录真实场景，固定为测试集。

### 第二层：每个痛点一个专用指标

| 痛点 | 自动指标 |
|---|---|
| 吃字 | Whisper 分别识别参考和处理结果，比较掉了多少词 |
| 人声被压低 | 参考有语音的帧里损失了多少能量；单独看词头词尾 |
| 漏过别人的声音 | 残留部分与干扰者声纹的相似度 |
| 背景忽大忽小 | 残留噪音电平随时间的波动 |
| 伪影 | NISQA 分项分数（噪音、音色失真、不连续、响度） |
| 音色 | 长期平均频谱与目标 LTASS 的偏差、LUFS、C50 |
| 延迟 | 输入输出互相关；每帧耗时 p99.9 |
| 笑声和附和声 | 专门的小测试集 |
| 回声兼容 | 输出后接 WebRTC AEC3 模拟 Zoom，测回声消除量 |

整体质量：无参考 MOS 预测模型 **DNSMOS**、**NISQA**、**UTMOS**。上游信息：URGENT 2026 有专门的"增强语音 MOS 预测"赛道，官方基线 **URGENT-PK** 开源在 GitHub。

### 第三层：真机自动测试

- 一个小音箱当"嘴"，另一个放噪音，真实麦克风收音，走完整管线（含虚拟设备）。每晚自动跑，换不同麦克风。
- 长时间压力测试放在 GitHub Actions 的 Apple Silicon 机器上：掉帧、内存增长、延迟漂移。

### 第四层：降低人耳确认成本

- 只听机器挑出的退步最大的 10 段，或指标互相矛盾的片段。
- **AB 对比**，不打绝对分。
- 用你的选择校准指标：一致的加权，不一致的降权。
- 需要更多人时用众包：微软开源的 **ITU-T P.808** 众包听测工具。
- 音频大模型当评委目前不可靠，只作参考。

### 第五层：回归门禁

- 每次改动自动跑完整合成测试集。
- 任何格子退步超阈值，阻止合并。
- 每个格子有历史曲线。
- 开源后让用户自愿上报诊断数据：只有分数、掉帧次数、压制统计，没有录音。

### 警惕

**不要只优化一个指标。** 模型会学会讨好 DNSMOS。多指标一起看，定期用耳朵校准。

### 飞轮

```
改动 → 合成测试自动跑 → 专项指标打分 → 自动挑出最差片段 → 你 AB 听 10 段 → 保留或回退
```

搭好后一轮几十分钟，你本人约十分钟。

---

## 14. 开源相关事项

### 为什么适合开源

- **信任**：程序一直听麦克风。开源证明全部本机处理、没有网络请求。
- **Mac 上有空白**：Linux 有成熟工具，Mac 几乎没有。
- **社区补场景矩阵**：各种麦克风、房间、macOS 版本。
- **先例**：OBS 就这样成为直播标准。

### 开源前要理清

- **许可证链条**：DeepFilterNet（MIT / Apache 2.0）、DPDFNet（Apache 2.0）代码没问题；模型权重看各自模型卡；微调数据要允许发布权重。
- **签名和公证**：虚拟音频设备装进系统目录、跑在 `coreaudiod` 里。没有签名公证，Gatekeeper 拦截，普通人装不上。需要 Apple Developer Program（每年 99 美元）。
- **维护负担**：每次 macOS 大版本都可能坏东西。从第一天起做诊断工具，一条命令导出设备信息和日志。

### 怎么做成好的开源项目

- 评测工具和代码一起开源，公开场景矩阵和打分方法。
- 模型后端可插拔，社区能接入新模型。
- **无障碍从第一天做好**：命令行完全可用；状态用声音反馈；菜单栏 app 对 VoiceOver 友好。这在开源音频工具里很少见，也是你最有资格做好的部分。
- 可持续性：GitHub Sponsors，或提供签名好的安装包。

---

## 15. 工作量估算

**AI 之前：** 可靠的 v1 约 **25–40 人月**，3–4 人干 9–12 个月。误差约 ±50%。

| 模块 | 人月 | 难点 |
|---|---|---|
| 信号处理核心与成品处理链 | 2–3 | 可复用 libDF；EQ、压缩、舒适噪音是经典 DSP |
| macOS 实时音频管线 | 2–3 | 时钟漂移、设备热切换、蓝牙、Audio Workgroup |
| 虚拟麦克风设备 | 2–4 | AudioServerPlugIn 文档少、调试难、要签名公证 |
| 回声消除接入 | 1–2 | 系统输出作参考信号（process tap 能省不少） |
| 自动声纹与门控 | 2–4 | 在线聚类、近场判断、防污染，偏研究 |
| 环境估计与控制器 | 3–5 | 多个小模型加大量调参 |
| 模型微调 | 4–8 | 数据管线、部分去混响、缩短 lookahead、带宽扩展、许可证 |
| 评测体系 | 2–3 | 场景矩阵录音、自动打分、盲听，持续进行 |
| 产品外壳 | 2–3 | 菜单栏 app、无障碍、安装包、自动更新、诊断工具 |
| 跨设备测试 | 2–3 | 各种麦克风、各版本 macOS、长时间压力测试 |

**MVP**（降噪 + 虚拟麦克风 + 基础成品处理，不含个性化、AEC、自适应）：一个有经验的工程师约 2–3 个月。

**需要的三种能力**：音频 DSP 与系统工程；语音机器学习；macOS 应用开发。过去最大的成本是找到这几种人。

**AI 时代：** 写代码的部分压缩到原来的 1/3 到 1/2。几乎不变快的：用耳朵听、采集真实录音、跨设备真机测试、苹果的签名公证流程。瓶颈变成评测和调音循环的速度，所以先搭评测体系。

---

## 16. 建议的开发顺序

### 起点

先写 `dsp`、`engine`、`eval`。用 libDF 现成代码和 DPDFNet ONNX，在 WAV 文件上离线跑通。验证整条链路的效果，不碰任何实时问题。效果对了再接 `io-macos`。`hal-plugin` 最难，最后做。

### 里程碑

- **M0 直通**：麦克风 → BlackHole，不处理。测基础延迟，连续一小时无爆音。
- **M1 接入 DFN3**：libDF 原样跑，CPU。作为基线。
- **M2 换 DPDFNet**：`ort` 加载 ONNX。
- **M3 说话人门控**：声纹提取 + 逐段相似度 + 滞回。（你的方向是零录制自动注册，见第 10 节。）
- **M4 打磨**：自动更新声纹、衰减下限、CoreML/ANE 的延迟与功耗实测。
- **之后**：自己的 AudioServerPlugIn 虚拟设备，摆脱 BlackHole。

### 最先做的三件事（不到一百行代码）

1. 掉帧时输出原声
2. 衰减下限
3. 快开慢关

它们解决"吃字"和"断掉"这两个最致命的问题。

### 优化的顺序

先搭评测（固定测试集 + DNSMOS + 盲听）。然后按回报排序：微调 → 调度（Audio Workgroup）→ 延迟 → Neural Engine。

### 无障碍

先做命令行版本：设备选择、开关、状态全用命令。VoiceOver 最顺。菜单栏 app 以后再做。

---

## 17. 短期应急方案

### 硬件（第一优先：让麦克风靠近嘴）

- **Shokz OpenComm2 UC**（推荐）：骨传导，不堵耳朵，听得到环境和 VoiceOver，过马路也安全。降噪麦克风杆（Qualcomm cVc），Loop120 USB-C 接收器即插即用。
- **DJI Mic Mini + Mobile Receiver**：纽扣大小领夹麦，实体三档降噪开关（摸得到）。注意有人反映插 Mac 输入音量偏小，先试录。
- **Shure MoveMic + MoveMic Receiver**：做工更讲究，但没有实体降噪开关。
- **带 boom mic 的耳机**（Jabra Evolve2 等）：最清楚，但像客服耳机。
- **AirPods Pro**：beamforming，比内置麦克风好。

### 软件

- **Zoom 个性化音频隔离**：免费，正对咖啡馆场景，最快的改善。
- **Krisp**：
    1. 官网下载 Mac 版，装虚拟音频驱动（可能要管理员密码和麦克风权限）。
    2. 登录。
    3. 在菜单栏 Krisp 里选真实的麦克风和扬声器，开降噪。
    4. Zoom 音频设置里麦克风选 **Krisp Microphone**。
    5. 或设为系统默认输入。
- **MetalVoice + BlackHole**：开源，带 CLI 版，对 VoiceOver 可能更顺；未签名，首次要"仍要打开"。

### 一定要做：只留一层降噪

- macOS 麦克风模式设"标准"，不用 Voice Isolation。
- Zoom 背景噪音抑制调"低"，不同时开个性化音频隔离。
- 用 Zoom 个性化音频隔离时，就关 Krisp。

### 限制

- Krisp 免费版多数来源说每天 60 分钟；Pro 每月 8 美元起（年付）。以 app 内为准。
- Krisp 的背景人声消除在 Mac 上是否可用不确定，装好后查看。
- Krisp、CEDAR 等界面对 VoiceOver 是否友好不确定，需要实测。

### 做第一组标杆

同一个咖啡馆、同一个麦克风，分别录 Krisp、Zoom 个性化音频隔离（以及可能的 CEDAR Voxis 14 天试用，通过 Audio Hijack 加载）。哪个不吃字、更自然，就是你产品要超过的线。

---

## 18. 参考项目与资料

### 模型与 Rust 引擎

- [DeepFilterNet（含 libDF Rust 实现）– GitHub](https://github.com/rikorose/deepfilternet)
- [DeepFilterNet 镜像与论文列表 – Gitee](https://gitee.com/rsmeng/DeepFilterNet)
- [DeepFilterNet3 VST3 插件（Rust 编译，MIT）– GitHub](https://github.com/Shuichi346/DeepFilterNet3-VST3)
- [DeepFilterNet3 CoreML – Hugging Face](https://huggingface.co/aufklarer/DeepFilterNet3-CoreML/commit/a9ec0b667af0653d31598b459a46fb65527ecfcf)
- DeepFilterNet-mlx（kylehowells，Swift 运行时，CoreML + MLX，支持 DFN 和 DPDFNet）
- [DPDFNet-8 48kHz HR CoreML（stateful）– Hugging Face](https://huggingface.co/iky1e/DPDFNet8-48kHz-HR-CoreML)
- [DPDFNet 论文 – arXiv](https://arxiv.org/abs/2512.16420v2)
- [DPDFNet 模型与 README – Hugging Face](https://huggingface.co/Ceva-IP/DPDFNet/blob/main/README.md)
- [DPDFNet 模型变体 – AlphaNeural](https://app.alphaneural.io/assets/Ceva-IP/DPDFNet)
- [DPDFNet_EvalSet（324 段、约 13.5 小时、12 种语言，附各模型输出）– Hugging Face](https://huggingface.co/datasets/Ceva-IP/DPDFNet_EvalSet/blob/main/README.md)
- [GTCRN Rust 实现（LADSPA）– GitHub (biglinux)](https://github.com/biglinux/gtcrn)
- [aTENNuate – arXiv](https://arxiv.org/pdf/2409.03377) · [Interspeech 2025](https://www.isca-archive.org/interspeech_2025/pei25_interspeech.html)
- [nnnoiseless（RNNoise 的 Rust 移植）– OpenApps](https://openapps.pro/packages/nnnoiseless)
- [MetalVoice – GitHub](https://github.com/Ghostkwebb/MetalVoice)
- [MetalVoice – OBS forum](https://obsproject.com/forum/resources/metalvoice-%E2%80%94-ai-noise-suppression-for-apple-silicon.2308/)
- BlackHole（macOS 虚拟声卡）
- WebRTC AEC3（`webrtc-audio-processing`）
- DeepVQE（微软，AEC + 降噪 + 去混响合一）
- MP-SENet、FRCRN（学术参考）

### 个性化 / 说话人

- [livekit-plugins-voicefilter-lite（open-voicefilter-lite）– PyPI](https://pypi.org/project/livekit-plugins-voicefilter-lite/)
- [PSE without a separate speaker embedding model（Pärnamaa & Saabas 2024）– 摘要](https://powerdrill.ai/discover/discover-Personalized-Speech-Enhancement-clxoatzt60bss0165p94fb9vr)
- [Cross-attention for real-time streaming PSE – arXiv](https://arxiv.org/pdf/2211.04346)
- [Real-time PSE Transformers with dynamic cross-attended speaker representations – Interspeech 2023](https://www.isca-archive.org/interspeech_2023/zhang23r_interspeech.pdf)
- [USEF-PNet / DSEF-PNet – arXiv](https://web3.arxiv.org/abs/2505.12288)
- [WeSep toolkit – arXiv](https://arxiv.org/html/2409.15799v1)
- WeSpeaker（ECAPA / ResNet，ONNX 版）
- [Shuai Wang 关于 WeSpeaker 和 WeSep 的讲座 – Telecom Paris](https://listen.telecom-paris.fr/?p=1141)

### Benchmark 与挑战赛

- [Speech Enhancement – Codesota](https://www.codesota.com/tasks/speech-enhancement)
- [Speech Enhancement on EARS-WHAM – hyper.ai](https://hyper.ai/en/sota/tasks/speech-enhancement/benchmark/speech-enhancement-on-ears-wham)
- [ICASSP 2026 URGENT Challenge – arXiv](https://arxiv.org/html/2601.13531v1)
- [URGENT Challenge 综述 – Moonlight](https://www.themoonlight.io/review/icassp-2026-urgent-speech-enhancement-challenge)
- [GAP-URGENet – arXiv](https://arxiv.org/html/2604.01832v1)
- [Interspeech 2021 DNS Challenge – arXiv](https://arxiv.org/pdf/2101.01902v1)
- [LCT-GAN 轻量 transformer 研究（含对比表）– arXiv](https://arxiv.org/pdf/2505.21057)
- [DPDFNet 综述 – Moonlight](https://www.themoonlight.io/review/dpdfnet-boosting-deepfilternet2-via-dual-path-rnn)
- 指标与工具：DNSMOS、NISQA、UTMOS、URGENT-PK、ITU-T P.808 众包工具、PESQ

### 经典文献（上游方法）

- Byrne 等 (1994)：国际 LTASS
- Mauler & Martin (约 2007)：非对称窗口低延迟分析合成
- ITU-T G.114：单向延迟建议

### 商业产品与专业工具

- [Voice isolation in Microsoft Teams – Microsoft Support](https://support.microsoft.com/office/voice-isolation-in-microsoft-teams-calls-and-meetings-a9756ea9-4cec-44c4-aefb-6f5d17c89427)
- [Voice isolation 管理策略（含 PassiveVoiceEnrollment）– Microsoft Learn](https://learn.microsoft.com/en-us/microsoftteams/voice-isolation)
- [Zoom personalized audio isolation – Stanford UIT](https://uit.stanford.edu/node/29632)
- [Zoom Workplace 6.0 – UC Today](https://www.uctoday.com/?p=66615)
- [Zoom 降噪测评 – Zapier](https://zapier.com/blog/zoom-noise-cancellation.md)
- [Maxine Studio Voice – NVIDIA](https://docs.nvidia.com/nim/maxine/studio-voice/1.0.0/overview.html)
- [CEDAR Voxis](https://cedaraudio.com/plugins/voxis) · [发布说明](https://cedaraudio.com/news/voxis-low-latency-neural-network-voice-isolator)
- [CEDAR Voxis vs iZotope RX 12 – MixingGPT](https://mixinggpt.com/blog/cedar-voxis-vs-izotope-rx-12-real-time-voice-isolation-2026)
- [CEDAR VoicEx 讨论 – Gearspace](https://gearspace.com/threads/cedar-voicex.1436423/latest)
- [iZotope Dialogue Isolate](https://www.izotope.com/en/products/rx/features/dialogue-isolate)
- [RX 11 深度评测 – Production Expert](https://www.production-expert.com/production-expert-1/izotope-rx-11-review-deep-dive-from-post-expert)
- [Waves Clarity Vx](https://www.waves.com/clarity-vx-vocal-noise-reduction)
- [2026 年 AI 人声降噪插件 – RysUp Audio](https://rysupaudio.com/blogs/news/best-ai-vocal-noise-reduction-plugins-2026)
- [Krisp, RNNoise, DeepFilterNet 对比 – Fora Soft](https://www.forasoft.com/learn/ai-for-video-engineering/articles-ai/real-time-noise-suppression-krisp-rnnoise-deepfilternet)
- [Krisp / Maxine / Dolby：自建还是购买 – Fora Soft](https://www.forasoft.com/learn/ai-for-video-engineering/articles-ai/krisp-maxine-dolby-build-vs-buy)
- [Krisp vs NVIDIA Broadcast – SimpleClean](https://simpleclean.app/blog/krisp-vs-nvidia-broadcast)
- Audio Hijack、Loopback（Rogue Amoeba）

### 用户痛点证据

- [Krisp 用户评价 – Trustpilot](https://uk.trustpilot.com/review/krisp.ai?page=7)
- [Krisp 自动关闭 – Discord 支持社区](https://support.discord.com/hc/es/community/posts/1500000524582-Option-to-stop-Krisp-Noise-suppression-turning-off-automatically)
- [Zoom 降噪过于激进 – Zoom Community](https://community.zoom.com/meetings-2/noise-suppression-is-too-aggressive-even-on-low-or-original-sound-i-need-to-hear-the-breath-16415)
- [Voice Isolation 声音差的原因 – Mackie](https://help.mackie.com/hc/en-us/articles/14963709854991-Why-Your-Mackie-USB-Audio-Sounds-Bad-on-macOS)
- [Voice Isolation on Sonoma – Focusrite](https://support.focusrite.com/hc/en-gb/articles/15437020104082)
- [Voice Isolation 开发者讨论 – Apple Developer Forums](https://developer.apple.com/forums/thread/829750)

### 硬件

- [Shokz OpenComm2 UC – EB Games](https://www.ebgames.com.au/product/electronics/331475-shokz-opencomm2-uc-2025-usb-c)
- [DJI Mic Series Mobile Receiver – Syntex](https://www.syntex.tv/dji-mic-series-mobile-receiver)
- [DJI Mic Mini 2 在 Mac 上的问题 – Mac Power Users](https://talk.macpowerusers.com/t/help-with-issues-on-a-new-dji-mic-mini-2-and-a-mac/46980)
- [Shure MoveMic Receiver](https://www.shure.com/movemic-receiver)

---

*没有给出链接的条目，在原对话里只提到了名字，没有附来源。用之前需要再查一遍。*
