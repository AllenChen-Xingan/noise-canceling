> workflow `wf_16dd3e65-ebc` · sim 路调研员原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

## 要点

1. **Apple 内置麦的真实频响查不到。** MicIRP 只有 23 只老式话筒（2017 年页面），没有 Apple 笔记本，也没有笔记本或手机。也没找到任何人公开测过 MacBook 内置麦的频响曲线。【已核实网页】
2. **DR-VCTK 里那台 MacBook Air 应该是 2017 年及更早的 Intel 双麦机型，不是 Apple 芯片的三麦机型。** 依据：README 写 `RELEASE November 2017`、2 个声道；Apple 2017 款规格页写 `Dual microphones`。用它反推的频响只能当「Intel Air」的旁证。而且它包含了回放音箱和房间的影响，只到 8 kHz。没查到有人用它算过 MacBook 频响。【推断，可信度高】
3. **Apple 芯片 Air 的三支麦，几何参数第一次查到了。** 开源项目 Asahi 给 M1 Air、M2 13 英寸、M2 15 英寸用的是同一组：边长约 2 cm 的等边三角形（`mic2_x 0.02 / mic3_x 0.01, mic3_y 0.0173`，单位米）。波束器之后接一个 120 Hz 高通，总增益约 +40.6 dB。注意这组是 Linux 社区的调参值，不是 Apple 的设计数据。【源码】
4. **按阵列理论估算（推断，未实测），2 cm 三麦很难对付低频。** 在白噪声增益（WNG，衡量自噪声被放大多少）≥ −10 dB 的约束下，指向增益约为：250 Hz 1.9 dB、500 Hz 3.9 dB、1–4 kHz 约 6 dB。正侧方或背后的声音约被压 12 dB，前方 ±45° 内只压约 2 dB。普通延迟求和（DAS）在 2 kHz 以下几乎不压（< 1 dB）。
5. **业界的「设备仿真」做法都差不多：随机均衡 + 带宽限制 + 削波 + 电平。** 没有哪家用了笔记本专门的数据。
   - URGENT 2025：削波取信号的分位点（下限 0–0.1，上限 0.9–1.0）；带宽限制从 8 / 16 / 22.05 / 24 / 32 / 44.1 kHz 中均匀抽；每条样本的失真个数 0 / 1 / 2 / 3 的概率为 0.25 / 0.40 / 0.20 / 0.15。
   - DeepFilterNet：随机二阶滤波，系数在 ±3/8 内均匀取；另有随机双二阶滤波（biquad），增益 ±15 dB。
   - Braun 和 Tashev（Microsoft）：同一个滤波系数范围；电平按高斯分布取，均值 −28 dBFS。【源码 / 论文】
6. **Apple 芯片 Air 没有风扇，不用加风扇噪声。** M1 Air 的声明噪声是 3 dB（操作者位置）；Intel 2020 款空闲 4 dB、上网 3 dB；满载时未标。底噪模型用 MEMS 麦的通用关系：等效输入噪声 = 94 dB − 信噪比。厂商常见信噪比 65 dBA 时，等效输入噪声为 29 dBA SPL。Apple 用的是哪颗麦、什么指标，未查到。【官方文档】【推断】
7. **对现有假设的纠错：**
   - 016 评测基准里的「7 kHz 低通」没有依据。macOS 给的是 48 kHz，Asahi 输出也到 48 kHz。对 16 kHz 模型来说这个低通基本无害，但不该当成「Mac 特征」。
   - exp006 设削波概率 0.2 偏高（推断）。
   - ±1.5 dB/倍频程的频响倾斜，比 Braun、DeepFilterNet 的范围窄，可以保留，但要补一个高通和一两个峰值滤波。
8. **我们走 HAL 采集，拿不到「语音突显」。** 主线不要模拟它，可以另开一个可选分支。【沿用 macos-mic.md】

## 逐条证据

| # | 结论 | 原文或数字 | 出处 | 来源 | 样本量 |
|---|---|---|---|---|---|
| 1 | MicIRP 没有 Apple 或笔记本条目 | 23 只话筒：STC 4035、AKG D12、RCA KU3a 等；没写许可 | https://micirp.blogspot.com/ | 【官方文档】（项目页） | 1 个站点 |
| 2 | MicAugment 用 MicIRP 的 70 条话筒冲激响应和 200 个带通滤波；带通的低端 50–150 Hz、高端 3–8 kHz | 「a set of 70 microphone IRs from the MICrophone Impulse Response Project, complemented with a set of 200 band-pass filters with cutoff frequencies uniformly sampled from [50 Hz, 150 Hz] and [3 kHz, 8 kHz]」 | arXiv 2010.09658 §2.2 | 【论文】 | — |
| 3 | MicAugment 的设备模型：设备冲激响应 → 按频带功率阈值截断（模拟非线性处理）→ 加滤波白噪声 → 软削波 | 公式 (1)–(4)；阈值在 [−2, 3] 内均匀取；削波门限在信号峰值的一半到峰值之间；五步的施加概率依次为 0.8 / 0.9 / 0.6 / 0.9 / 0.1；信噪比 5–30 dB | arXiv 2010.09658 §2.1–2.2 | 【论文】 | — |
| 4 | **MicAugment 的「真实」测试集就是 MacBook Pro 自放自录（扬声器放、内置麦录）**；用 15 s 配对音频学设备模型，最多补回 70% 的准确率损失（88.3 → 90.3，理想上限 91.0） | 「recaptured playing the audio clips with MacBook Pro loudspeakers and recording them with the built-in microphone」，表 2 | 同上 §3.2 | 【论文】 | 1 台 MacBook Pro，型号年份未写 |
| 5 | 设备冲激响应增强（声学场景分类）：66 条 MicIRP 冲激响应，施加概率默认 0.6、可用范围 0.2–0.6；在没见过的设备上准确率 +0.7 到 +4.6 个百分点 | 表 I | arXiv 2305.07499 | 【论文】 | 3 个模型 × 2 个数据集 |
| 6 | DR-VCTK 的 MacBook Air 是 2 声道，2017 年 11 月发布，录音时 48 kHz、发布版 16 kHz，回放音箱 Bose SoundLink III，距离 2 m | README 原文 | https://datashare.ed.ac.uk/bitstream/handle/10283/3038/README.txt | 【官方文档】 | 1 台 |
| 7 | 2017 款 13 英寸 Air 是双麦；2018 款写 `Three microphones`；2020 Intel 款与 M1 款都写 `Three-mic array with directional beamforming` | — | https://support.apple.com/kb/SP753 ；https://support.apple.com/kb/SP783 ；https://support.apple.com/kb/SP813 ；https://support.apple.com/en-us/111883 | 【官方文档】 | — |
| 8 | 由 #6 和 #7 推出：DR-VCTK 那台是 Intel 双麦 Air（2017 年及以前） | — | — | 【推断】可信度高 | — |
| 9 | DR-VCTK 的论文没给逐设备的频响，也没给 MacBook 单独的分数 | 只有混合后的 PESQ：带噪 1.24 | arXiv 1911.03952 | 【论文】 | — |
| 10 | 用 DR-VCTK 反推频响的可行做法：用 Welch 法求传递函数 H₁ = Pxy/Pxx（MacBook 声道对干净 VCTK）；再除以 Apogee MiC 声道的同一估计，以抵消音箱和房间的影响 | 只到 8 kHz（发布版 16 kHz）；开窗和关窗两种条件要分开算 | — | 【推断】可信度中；没查到有人做过 | — |
| 11 | Apple 芯片 Air 的三麦几何（Asahi 的值）：J313（M1）、J413（M2 13 英寸）、J415（M2 15 英寸）都是 mic2 (0.02, 0)、mic3 (0.01, 0.0173)，单位米；看向的方向 J313 为 120°，J413 和 J415 为 80°；之后接 120 Hz、Q 0.707 的高通；每路 ×2，再 ×7.25 两次（约 +40.6 dB） | `firs/j313/mic.json` 等 | https://github.com/AsahiLinux/asahi-audio（GPL-2.0） | 【源码】 | 3 个 Air 机型；MacBook Pro J314 同为三角形，J293 是 0 / 2 / 4 cm 直线排列 |
| 12 | iFixit 拆机：Late 2020（M1）Air 的三支麦嵌在上壳的凹槽里，一条排线、一个 ZIF 接口，其中一支「closest to the center」；没写左右位置和间距 | 第 29–31 步 | https://www.ifixit.com/Guide/MacBook+Air+13-Inch+Late+2020+Microphone+Array+Replacement/142560 | 【论坛 / 博客，个人说法】（拆机指南） | 1 台 |
| 13 | 2 cm 三角阵的理论指向性（自由场、水平面内、3D 扩散噪声相干、不计机身遮挡）：<br>• 延迟求和：≤ 2 kHz 时指向增益 < 1 dB；4 kHz 2.4 dB；8 kHz 4.9 dB<br>• 超指向（WNG ≥ −10 dB）：250 Hz 1.9 dB、500 Hz 3.9 dB、1–4 kHz 约 6 dB、8 kHz 4.9 dB；正侧方 −12 dB，±45° 处 −2 dB<br>• WNG ≥ 0 dB：250 Hz 0.5 dB、1 kHz 2.1 dB、2 kHz 4.5 dB | 计算脚本：`/private/tmp/claude-501/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-macmic/a5b0364a-7ef2-4d59-b924-8888b7e88ca5/scratchpad/arr2.py` | 理论上限：3 麦最大指向因子 9.5 dB（Benesty & Chen 2013，见 beamform.md D1） | 【推断】可信度低（Apple 用什么算法、看向哪里都未知） | 0 台实测 |
| 14 | 机身遮挡会让「从机身另一侧来的高频」明显变弱：Vision Pro 的左侧和背后来声在 16 kHz 频带「as much as 40 dB lower」；iPhone 底麦偏向自己那一侧接收 | — | https://www.faberacoustical.com/blog/wp-json/wp/v2/posts/2286 ；https://www.faberacoustical.com/blog/?p=2349 | 【论坛 / 博客，个人说法】（消声室实测） | 各 1 台；不是 Mac |
| 15 | URGENT 2025 的参数：<br>• 削波：下分位 [0, 0.1]、上分位 [0.9, 1.0]<br>• 编解码：mp3 质量 1–10；ogg 用 vorbis 或 opus，质量 −1 到 10<br>• 丢包：20 ms 一包、丢包率 0.05–0.25、最多连丢 10 包<br>• 信噪比 −5 到 20 dB；混响概率 0.5<br>• 每条失真个数 0 / 1 / 2 / 3 的概率：0.25 / 0.40 / 0.20 / 0.15 | `conf/simulation_train.yaml` | https://github.com/urgent-challenge/urgent2025_challenge ；本机 `resources/urgent2025_challenge` | 【源码】 | — |
| 16 | URGENT 2025 的带宽限制：从 8 k / 16 k / 22.05 k / 24 k / 32 k / 44.1 k 中、低于原采样率的那些里均匀抽；重采样方法随机 | `simulation/generate_data_param.py` | 同上 | 【源码】 | — |
| 17 | URGENT 2026 先给语音做一次 70 Hz 高通（FIR，过渡带 15 Hz）再仿真；失真配置与 2025 相同 | `simulate_data_from_param.py:29-56,461` | `resources/urgent2026_challenge_track1` | 【源码】 | — |
| 18 | DNS Challenge 合成器不做设备仿真：只把电平随机到 −35 到 −15 dBFS、信噪比 −5 到 20 dB；`add_clipping` 只被导入，没被调用 | `noisyspeech_synthesizer.cfg:39-43`；`audiolib.py:94-98` | `resources/DNS-Challenge` | 【源码】 | — |
| 19 | Braun & Tashev（Microsoft）：随机频谱整形 H(z) = (1 + r₁z⁻¹ + r₂z⁻²) / (1 + r₃z⁻¹ + r₄z⁻²)，rᵢ 在 [−3/8, 3/8] 内均匀取，语音和噪声各自独立；电平按高斯分布，均值 −28 dBFS、方差 10 dB | §6.1 | arXiv 2008.06412 | 【论文】 | — |
| 20 | DeepFilterNet 的默认设置：<br>• 随机二阶滤波，系数 ±3/8，概率 0.25<br>• 随机双二阶滤波：1–3 个，增益 ±15 dB，Q 0.5–1.5；高通 40–400 Hz、低通 4–8 kHz、峰值 40 Hz–4 kHz；滤完恢复原 RMS；默认概率 0<br>• 削波：门限为峰值的 0.05–0.9<br>• 带宽限制截止频率：4 / 6 / 8 / 10 / 12 / 16 / 20 kHz | `libDF/src/augmentations.rs:137-170,348-386,560-564,1107`；`dataset.rs:644-670` | `resources/DeepFilterNet` | 【源码】 | — |
| 21 | WebRTC 自动增益二代（AGC2）的自适应数字增益默认值：余量 5 dB、最大增益 50 dB、初始增益 15 dB、增益最快变 6 dB/s、输出噪声上限 −50 dBFS；电平估计约 400 ms 进入可信；限幅器从 −1 dBFS 起作用、压缩比 5；限幅器的电平估计瞬时起音、20 ms 释放 | `sonora/src/config.rs:255-271`；`sonora-agc2/src/common.rs:26,40,61-65`；`fixed_digital_level_estimator.rs:11-17`；`saturation_protector.rs:8-12` | `resources/sonora`（移植自 WebRTC） | 【源码】 | — |
| 22 | WebRTC 一代（AGC1）默认值：目标电平 −3 dBFS（`target_level_dbfs: 3`）、压缩增益 9 dB | `webrtc-audio-processing-config/src/lib.rs:300-301` | `resources/webrtc-audio-processing` | 【源码】 | — |
| 23 | Apple 的语音处理（VPIO）默认开自动增益；我们走 HAL，没有这一层 | `AudioUnitProperties.h:2624-2649` | 沿用 macos-mic.md #8 | 【官方文档】头文件 | — |
| 24 | MEMS 麦：等效输入噪声 = 94 dB − 信噪比；数字麦 94 dB SPL 对应 −26 dBFS（120 dB SPL 对应满量程）；信噪比 65 dBA 时等效输入噪声 29 dBA SPL，数字底噪约 −91 dBFS(A)（未加系统增益前） | ADI AN-1112 | https://www.mouser.lt/pdfDocs/AN-1112-v11-3.pdf | 【官方文档】（厂商应用笔记） | Apple 的麦型号未查到 |
| 25 | 风扇：M1 起的 Air 没有风扇；M1 Air 声明噪声空闲和上网都是 3 dB；Intel 2020 Air 空闲 4 dB、上网 3 dB，满载未声明 | ECMA-109 声明表 | https://support.apple.com/en-us/111883 ；https://support.apple.com/kb/SP813 ；无风扇见 https://macrumors.com/2020/11/10/apple-silicon-mac-unveiled | 【官方文档】；无风扇一条是【论坛 / 博客，个人说法】 | — |
| 26 | Libri-Adapt 的 MacBook Pro 只是外接 Shure MV5 的主机；MobiPhone 是 21 部手机，没有笔记本；ACE 里的笔记本是 Chromebook | — | mtgdata 转交 `r1-device-macmic-handoff.md`；arXiv 2010.09658 §3.1 | 【官方文档】【论文】 | — |

## 模拟流水线（干净语音 → 「像 MacBook Air 内置麦、标准模式、经 HAL」）

这条流水线用来生成训练和评测数据，顺序就是执行顺序。

| 步 | 做什么 | 参数：范围（默认） | 依据 | 可信度 |
|---|---|---|---|---|
| 1 | 房间与距离：卷积房间冲激响应，说话人在屏幕前方偏上 | 距离 0.3–1.0 m（0.5）；混响时间 0.2–0.6 s（0.35）；混响概率 0.5–0.7 | URGENT 混响概率 0.5（#15）；距离是坐姿常识 | 中（距离分布无出处） |
| 2 | 阵列的空间效应（只作用在干扰声和噪声上，目标说话人不衰减）：<br>• 按方位做频率相关衰减：前方 ±45° 内 0 到 −2 dB；侧方和背后：250 Hz −2、500 Hz −4、1–6 kHz −12、8 kHz −10 dB<br>• 扩散噪声按频带降信噪比：250 Hz −2、500 Hz −4、1–4 kHz −6、8 kHz −5 dB | 整体强度系数 0–1（0.5） | #13 理论估算 + #14 机身遮挡 | **低**（Apple 的算法、看向哪里、是否自适应都未知） |
| 3 | 设备线性频响：<br>• 2 阶高通<br>• 1–2 个峰值或搁架滤波<br>• 频响倾斜 | 高通 80–150 Hz（120），Q 0.7；峰值增益 ±(0–6) dB（±3）、Q 0.5–1.5、频率 1–6 kHz；倾斜 ±1.5 dB/倍频程（保留 exp006 的值） | Asahi 120 Hz 高通（#11）；MicAugment 带通低端 50–150 Hz（#2）；DeepFilterNet（#20）；Braun（#19） | 中（形状有出处，Apple 的数值没有） |
| 4 | 带宽：**不用固定的 7 kHz 低通**。16 kHz 模型直接重采样即可；如需随机，低通 7.5–8 kHz、概率 ≤ 0.1 | （无低通） | macOS 给 48 kHz（macos-mic.md #2）；Asahi 允许 48 k 输出（#11） | 中 |
| 5 | 麦自噪声：加白噪声，略带粉红化 | 相对说话人声压的信噪比 30–45 dB（38）；依据是说话人声压 60–70 dB SPL、等效输入噪声 29–32 dBA | #24 | 中（关系式可信，Apple 的麦型号未知） |
| 6 | 风扇或机身噪声：**Apple 芯片 Air 跳过**；Intel Air 或 Apple 芯片 MacBook Pro 以一定概率加风扇噪声 | Intel：概率 0.2，信噪比 15–35 dB；Apple 芯片 Air：概率 0 | #25 | 中（是否有风扇有出处；满载风噪的 dB 未查到） |
| 7 | 系统增益与自动增益（代理）：<br>• 先做电平随机<br>• 再以一定概率套 AGC2 式慢增益 | 电平：均值 −28 dBFS、σ 约 10 dB（高斯）；AGC 概率 0.2–0.3（0.25）；余量 5 dB、最大增益 30 dB（默认 50 太大）、≤ 6 dB/s、噪声上限 −50 dBFS | #19、#21、#22 | 低（HAL 路径上到底有没有自动增益，未知） |
| 8 | 系统降噪（代理，可选）：平稳噪声谱减或维纳滤波，最大压 6–12 dB | 概率 0–0.3（0.15） | MicAugment 的频带阈值（#3）只是类比 | **低**（标准模式下 HAL 有没有降噪，未知） |
| 9 | 限幅与削波（放在增益之后，模拟 ADC 满量程）：<br>• 软限幅：−1 dBFS 起、压缩比 5<br>• 硬削波：驱动 6–16 dB，或用分位法 | 硬削波概率 0.05–0.1（建议从 0.2 降到 0.08）；分位法下 0–0.1、上 0.9–1.0 | #15、#20、#21 | 中（方法有出处；Mac 上真实削波的频率未知，降概率是推断） |
| 10 | 输出：重采样到 16 kHz，用 float32 或 16 位量化。**不要**把蓝牙窄带和编解码放进这条链（它们不是内置麦），保持为另外的设备分支 | — | — | 高 |

**分机型：**
- **Apple 芯片 MacBook Air（M1 / M2 / M3…）：** 第 2 步用三角阵估算；第 6 步关掉。
- **Intel MacBook Air：** 2018–2020 款是三麦，但几何未查到；2017 年及以前是双麦，频响可以用 DR-VCTK 校准（#8、#10），第 2 步建议强度取 0–0.3。第 6 步打开。
- **MacBook Pro（对照）：** J293 是 0 / 2 / 4 cm 直线排列（#11），有风扇。

## 只有实测才能定的参数

1. 标准模式经 HAL 拿到的信号：真实频响（高通转折频率、峰值、是否到 20 kHz），以及有没有降噪、自动增益或限幅。
2. Apple 波束器看向哪里、指向图有多强，是固定还是自适应；离轴说话人真实被压多少 dB。
3. 默认输入音量下，底噪是多少 dBFS；正常说话（0.5 m）的电平是多少 dBFS；多久削波一次。
4. Intel 与 Apple 芯片、M1 / M2 / M3 之间的差异。
5. 风扇满载时的噪声频谱与声压（Intel Air、MacBook Pro）。
6. DR-VCTK 反推出的 Intel Air 频响（数据要下载，1.67 GB，超过本次 50 MB 的限制，没做；许可也待核）。

## 没查到 / 仍不确定

- Apple 用的麦克风型号、厂家、信噪比指标：未查到。
- 2018–2020 Intel 三麦 Air 的几何：未查到。Asahi 的 2 cm 等边三角形是社区调参值，它的 v3.2 改过一次 J313 的几何，所以不保证是物理实测。
- iFixit 拆机没给麦的位置和间距。
- MicIRP 现在有没有新增条目（页面是 2017 年的）：没逐页核。
- 有没有人用 DR-VCTK 反推 MacBook 频响的论文或代码：未查到。
- 微软 DNS 系列论文里有没有专门的「设备 / 笔记本」仿真：只查到 Braun & Tashev 的随机频谱整形；DNS 官方合成器里没有。
- Braun 论文里频谱增强带来的 PESQ 增益：原文只说 slightly，具体数字我没抄录。
- WebRTC AGC1 的起音和释放时间常数：没查。AGC2 的值已列在 #21。
- 第 13 条的理论估算没计入机身、键盘面和屏幕的反射，也没计入声源仰角。实际值可能和估算差几 dB 以上。