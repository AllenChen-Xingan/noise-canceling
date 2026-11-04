> workflow `wf_16dd3e65-ebc` · sim 路怀疑派原始交回 · 2026-11-03。未经汇总修订，结论以 devices.md / simulation.md 为准。

## 第「sim」路报告审核（怀疑派）

审核方法：我逐条打开了原出处，包括 4 篇 arXiv 论文的 PDF、Apple 规格页、DR-VCTK 的 README 和说明页、MicIRP 的博客数据源、Asahi 的 `mic.json` 及其提交历史、Triforce 的 README、iFixit 的接口数据、Faber Acoustical 的博文，以及本机 `resources/` 里的 URGENT、DeepFilterNet、DNS、sonora、webrtc 源码。我还只读查看了本机 `/System/Library/Audio/Tunings`，没有碰音频设置。报告里的阵列脚本 `arr2.py` 我重跑了一遍，是纯 Python，计算量很小。

### 逐条判定

| # | 结论 | 判定 | 理由与核到的原文 | 修正建议 |
|---|---|---|---|---|
| 要点1 / #1 | MicIRP 只有 23 只话筒，没有 Apple 或笔记本 | **部分推翻** | 「没有 Apple 或笔记本」成立。但「23 只」不对：博客数据源共 60 条，2017 年有 24 条（23 只话筒加一条 "Coming Soon..."），2013 年还有 36 条（RCA 44BX、EV RE20、Coles 4038 等）。报告 #2 自己引用的「70 条冲激响应」和 2305.07499 的「66 条」都和「23」矛盾，报告没察觉 | 改成「约 59 只老式话筒（2013 和 2017 两批）」，结论不变 |
| 要点1 | 没人公开测过 MacBook 内置麦的频响 | 成立（限网页） | Faber 博客搜 macbook 0 条。但它测过 iPhone 16 Pro 话筒的频响（博文 2349），是最接近的 Apple 麦实测，报告只引了它的指向部分 | 补上 iPhone 16 Pro 频响，作为「Apple MEMS 麦」旁证 |
| 要点2 / #6 / #8 | DR-VCTK 那台是 2017 年及更早的 Intel 双麦 Air | 成立，且证据比报告写的强 | README 写「RELEASE November 2017」「Channel 1/2 MacBookAir」「All data was recorded at 48 kHz」「2 meters」。论文 1911.03952 第 118 行原文「MacBookAir's two microhpones」，**直接写了双麦**，报告没引。「2 声道 ⇒ 双麦」本身只是弱推断 | 依据改成论文原句。注明双麦 Air 跨 2013–2017 好几代（推断），说话距离 2 m 远大于实际坐姿 |
| #9 | DR-VCTK 论文没给 MacBook 单独分数；带噪 PESQ 1.24 | **存疑（误导）** | README 写「Channel 3 and 7 … were used as test set」，测试集是 Apogee 和 iPad。**MacBook 只在训练集里**，所以 1.24 和 MacBook 无关 | 写明 1.24 是 Apogee+iPad 测试集的分数 |
| #10 | 用 H₁ 求传递函数，再除以 Apogee 声道，抵消音箱和房间 | **存疑，近乎不可行** | README：每条样本只有一个录音条件（`train_ch_log.txt`）；Apogee（声道 3）只在测试集，说话人是另外 2 个。小样本版里没有「同一句话、MacBook 与 Apogee 同录」的配对，没法逐句做比值。而且 Apogee 自身频响、摆放位置不同，抵消不干净 | 降级为「只能拿 MacBook 对干净 VCTK 求长时平均谱比，结果含 Bose 音箱和房间」。许可已查到：CC BY 4.0，可从「待核」里删掉 |
| #7 | 2017 双麦；2018 三麦；2020 Intel 与 M1 写「Three-mic array with directional beamforming」 | 成立 | 四个页面原文逐字对上；M2 13 英寸（111867）、M2 15 英寸（111346）也是同一句 | — |
| 要点3 / #11 | Asahi 给三款 Air 的几何都是 2 cm 等边三角形，增益每路 ×2、再 ×7.25×7.25，约 +40.6 dB | **部分推翻** | ①坐标逐字对上。②但 J314（MacBook Pro 14 英寸）的坐标**一模一样**；J313 在 2025-03-18 的提交「j313: mics: fix triforce parameters」之前用的是 J293 的直线 0/2/4 cm。三种机身共用一组数，更像模板，不像实测。③J415 的 mixO 是 ×3，不是 ×2；J314 是 3/3/2 和 7.6。④2×7.25² ≈ 105 倍，即 **+40.4 dB**，不是 40.6。⑤这组增益是 2026-08-06「mics: Make the mics much louder」才改成的，是凭耳朵调的值。⑥Triforce README 写明它是窄带 MVDR 自适应波束器（不做宽带分解），「Do not expect it to outperform Apple's implementation」 | 把几何降级为「社区模板值，可信度低」；改正 J415 的增益和总增益数字 |
| 要点4 / #13 | 2 cm 三角阵，WNG≥−10 dB 时指向增益 250 Hz 1.9 / 500 Hz 3.9 / 1–4 kHz 约 6 dB；正侧方或背后压约 12 dB；DAS 在 2 kHz 以下 <1 dB | **部分推翻** | 我重跑 `arr2.py`，指向增益数字能复现。但「背后压 12 dB」不对：脚本里 270°（正背后）的值是 1 kHz −5.9、2 kHz −5.7、4 kHz −4.8、8 kHz −1.3 dB，只有 0° 和 180°（正侧方）才是 −12。前提假设还可能整体错：Apple 自带的波束系数 `builtin_microphone_general_aufx_bmfd.plist`（AID40、AID13）写的是 `Azimuth 0, Elevation 90`，看向可能垂直于阵列平面（推断，坐标约定未知），而脚本假设在平面内看向 90° | 把背后改成 −1 到 −6 dB；指向模型改用 Apple 实际系数重算（见末尾第 1 条） |
| #14 | Vision Pro 左侧和背后来声在 16 kHz「as much as 40 dB lower」 | 成立 | 博文 2286 原文对上，2024-07-31，消声室，1 台。Faber 是测量软件 SignalScope 的厂商，不算个人说法 | 来源标签改为「厂商博客实测」 |
| #2 | MicAugment：70 条冲激响应 + 200 个带通（50–150 Hz / 3–8 kHz） | 成立 | §2.2 原文逐字对上 | — |
| #3 | 阈值 [−2, 3]；削波门限在峰值的一半到峰值之间；五步概率 0.8/0.9/0.6/0.9/0.1；信噪比 5–30 dB | 成立 | 原文对上。注意：这组参数是用来生成「增强网络」训练数据的，不是设备模型本身 | 注明用途 |
| #4 | 真实测试集是 MacBook Pro 自放自录；**用 15 s 配对音频**学设备模型；最多补回 70%（88.3→90.3，上限 91.0） | **部分推翻** | 数字对上，表 2 的补回率是 73.0%。但 MicAugment 本身是**非配对**方法（原文「works in the unpaired setup」），真实场景用的是一段 15 s 的 Librispeech 重录；用配对数据的是上限（Oracle）。任务是关键词识别，不是降噪 | 改「配对」为「非配对」；注明任务；机型年份未写 |
| #5 | 设备冲激响应增强：66 条；概率默认 0.6、范围 0.2–0.6；没见过的设备上 +0.7 到 +4.6 个百分点 | 成立 | 表 I 的增益在 +0.68（PaSST，TAU20）到 +4.60（CP-ResNet，TAU22）之间。正文写 0.4–0.6 稳健，图 2 写 0.2–0.6；CP-ResNet 在 TAU22 上用的是 0.4。任务是声学场景分类 | 补一句「任务是场景分类」 |
| 要点5 / #15 | URGENT 2025 的削波、编解码、丢包、信噪比、失真个数 | 成立 | `conf/simulation_train.yaml:58-110` 逐项对上。漏了风噪：概率 0.05，风噪里还带削波（门限 0.85–1.0，概率 0.75） | 可补风噪 |
| 要点5 / #16 | 带宽限制从 8 / 16 / … / 44.1 kHz 中抽 | **存疑（单位混淆）** | `generate_data_param.py:14` 里是 `SAMPLE_RATES`，是**采样率**，实际带宽是它的一半（4–22 kHz）；列表里还有 48000（被「低于原采样率」过滤掉）。报告把它和 DeepFilterNet 的「截止频率」并列，口径不一致 | 写成「降采样到 8 k…44.1 k，对应带宽 4–22 kHz」 |
| #17 | URGENT 2026 先做 70 Hz 高通；失真配置与 2025 相同 | 存疑 | 高通函数对上（filtfilt，零相位），但代码默认 `--highpass False`，靠 `prepare_train_data.sh:109` 传 True 才开。失真配置不完全相同：ogg 的质量参数 2026 是 [1, 10]，2025 是 [−1, 10] | 两处都改 |
| #18 | DNS 合成器不做设备仿真；`add_clipping` 只导入没调用 | 成立 | cfg 第 39–43 行、`audiolib.py:94` 对上。它会加混响（`rir_choice: 3`） | — |
| #19 / 要点5 | Braun & Tashev：H(z) 系数 ±3/8，电平均值 −28 dBFS、方差 10 dB | 成立；「没抄录 PESQ」不对 | §6.1 原文对上。滤波器是「as proposed in [7]」，出自 RNNoise（DeepFilterNet 源码注释也写「Adopted from RNNoise」）。表 1 里频谱增强只多 **+0.01 PESQ**（3.31→3.32）；电平增强配标准损失反而 −0.02。原文写的是「variance 10 dB」，报告流水线第 7 步写成 σ | 补上 PESQ 数字和源头；方差/σ 统一一种说法 |
| #20 | DeepFilterNet 默认值 | 存疑（有漏项） | 低阶滤波 ±3/8、概率 0.25 对上；双二阶滤波 ±15 dB、Q 0.5–1.5、1–3 个、恢复 RMS、默认概率 0 对上。漏项：①还有低搁架（40–1000 Hz）和高搁架（1000–8000 Hz）；②截止频率列表还有 22050；③语音削波默认概率是 **0**（`train.py:155`），噪声削波概率 0.1、门限 0.01–0.5 | 补全 |
| #21 | AGC2：余量 5、最大增益 50、初始 15、6 dB/s、噪声上限 −50、约 400 ms 进入可信；限幅器 −1 dBFS 起、压缩比 5 | 基本成立，一处混淆 | config.rs 和 common.rs 对上。但 −1 dBFS 是「限幅器电平到这里就开始降 AGC 增益」的阈值；限幅曲线自己的拐点按 `limiter_db_gain_curve.rs` 算是 −0.5 − 1/4 = **−0.75 dBFS** | 改正 |
| #22 | AGC1：目标 −3 dBFS、压缩增益 9 dB | 成立 | lib.rs 第 300–301 行对上。目标指的是峰值或包络 | — |
| #23 / 要点8 | VPIO 默认开 AGC；走 HAL 拿不到语音突显 | 成立，但推论有漏洞 | 头文件 2624–2649 行对上。漏洞：本机的系统调音文件显示，内置麦的 `builtin_microphone_general.dspg` 是 3 进 1 出，48 kHz，依次是直流隔离 → 波束器（`bmfd`）→ 注释写「Noise Reduction」的 `micSense`（`sens`）→ 均衡 `ToneMeister` → 增益。所以走 HAL 拿到的很可能**已经降过噪、做过均衡**（推断：「general」对应标准模式未证实） | 主线流水线第 8 步「系统降噪」不该只当「可选、概率 0.15」，先读调音文件再定 |
| #24 / 要点6 | 等效输入噪声 = 94 − 信噪比；65 dBA → 29 dBA SPL，−91 dBFS(A) | 成立（未重开 PDF，与公式一致） | −26 dBFS 灵敏度是例子，不是通用值。高端 MEMS 麦信噪比 69–70 dBA，等效输入噪声约 24 dBA | 范围写成 24–32 dBA |
| 流水线第5步 | 自噪声信噪比 30–45 dB（默认 38） | 存疑 | 按报告自己的数字算：60–70 dB SPL 减 29–32 dBA，得 28–41 dB，不是 30–45。也没算波束器放大自噪声：Apple 系数的 Σ\|w\|²（自噪声放大倍数）在 AID40 的 100–1000 Hz 是 4.7–11.7，相当于 **+7 到 +11 dB** | 改成按频带设：低频自噪声加 5–10 dB |
| 要点6 / #25 | M1 Air 无风扇；声明噪声 M1 3 dB，Intel 2020 空闲 4、上网 3 | 数字成立，用法存疑 | 数字对上。但所有机型声功率都是 **1.3 B**，有风扇和无风扇一样，说明是测量下限，不能用来推风扇噪声。「无风扇」有官方出处：Apple Newsroom 2020-11「delivers this performance in a fanless design」 | 换成官方出处；注明声明值反映不了风扇 |
| #12 | iFixit：M1 Air 三麦在上壳凹槽里，一条排线、ZIF 接口，一支「closest to the center」 | 成立 | 第 25–31 步对上。它是拆机指南，不算个人说法 | — |
| 要点7a | 016 的「7 kHz 低通」没依据，因为 macOS 给 48 kHz | 结论可留，论证存疑 | 016 `benchmark.md:74` 本来就标了「假设」，不是当成 Mac 特征写的。而且采样率 48 kHz 不等于带宽到 24 kHz；Asahi 的 `allowed-rates` 只是允许的流采样率（2025-03 起还允许 8 k–44.1 k） | 论证改用实测频响；或者用调音文件里的均衡曲线（`EQSelection`，481 点到 24 kHz） |
| 要点7b | exp006 削波概率 0.2 偏高 | 存疑（用途错配） | exp006 是场景分类器的仿真，「none 0.6 / bt 0.2 / clip 0.2」是三个设备类别的配比（`recipes.json:52`），不是在模拟真实削波频率 | 只在拿它训练降噪模型时才适用 |
| #26 | Libri-Adapt / MobiPhone / ACE | 部分核到 | 「MobiPhone 21 部手机」在 2010.09658 §3.1 对上；Libri-Adapt 和 ACE 两条未核 | 标「未核」 |

### 报告漏查的重要方向

1. **本机的系统调音文件，是比 Asahi 权威得多的出处**（只读，不碰设备设置）。`/System/Library/Audio/Tunings/AID*/DSP/`（共 45 MB）里有：
   - 每个机型的内置麦处理图（模式有 general、voice、dictation、raw、voice_raw），是纯文本 `.dspg`；
   - **固定波束器系数**，每支麦 481 个频点，48 kHz，含看向方向；
   - 降噪模块：`micSense`、`ambientNoiseSuppressor`（`ocns`）、听写模式的神经降噪（`nnrs` / `nnns`）；
   - 均衡（`ToneMeister`、`nbeq`）。
   有了这些，「标准模式经 HAL 有没有降噪、均衡」「波束看向哪里、指向图多强、自噪声放大多少」都能直接算出来，不必等实测。
2. **Intel 三麦 Air 的数据也可能在同一处**：`Mac-0CFF9C7C2B63DF8D` 等 15 个 Intel 主板目录里有 16–18 个麦相关文件。主板 ID 对应哪款机型，要再查（我推断其中有 MacBookAir8,x 和 9,1）。
3. **要先把 AID 编号对上机型**（例如本机 Mac17,3 是哪个 AID），再选对应的波束系数和处理图。否则报告第 2、3、5、8 步的参数都是凭空设的。
4. DR-VCTK 论文本身写了「two microphones」。还没查完整版的 243 GB 数据集里有没有逐句的多设备平行录音（只有那样才能做按句的比值）。
5. 设备仿真增强的源头是 RNNoise（Valin 2018）。PercepNet 和 DNS-5 的基线也可以顺查一下。

---
核对用的下载件在 `/private/tmp/claude-501/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-macmic/a5b0364a-7ef2-4d59-b924-8888b7e88ca5/scratchpad/dl/`（论文 PDF 和文本、DR-VCTK README、MicIRP 页面）。算系数范数的脚本是 `/private/tmp/claude-501/-Users-allenchen-xingan-chen-B-MyCreate-5-App-noise-canceling-macmic/a5b0364a-7ef2-4d59-b924-8888b7e88ca5/scratchpad/wng.py`，它算出的 WNG 列假设看向正上方、各麦相位一致，只能当粗参考。仓库文件没有改动。