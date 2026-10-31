# 两类来源 · 设备端低功耗实时自适应 + 可靠性

> 2026-10-31。只列支撑结论的关键来源；每条全文与更多链接在 `notes/` 对应笔记。数字以 `notes/s1-numbers-check.md` 核对后的为准。
> 本线**没有克隆仓库**：GTCRN 源码只在线阅读（github.com/Xiaobin-Rong/gtcrn），主目录 `resources/` 未新增。

## 一、自有积累（可照搬，先确认时效）

| 是什么 | 在哪 | 还算数吗 |
|---|---|---|
| GTCRN 本机最坏一帧 0.62 ms / 16 ms 帧（含 STFT），48.2K 参数、约 33 MMACs/s | `docs/research/002_model-baseline/sources.md:63`（exp001）| 算：同机、同权重 |
| exp003 人耳：赌场段 FastEnhancer-M 好、邻桌说话段 GTCRN 略好 | `docs/experiments/exp003-offline-ab/results/README.md` | 算，但证据弱（D33）|
| 依赖选型：`triple_buffer` · `rtrb` 等 | `docs/research/009_rust-ecosystem/decision.md` | 算 |
| 送原声两层、实时三档 | `specs/rust/architecture.md` §3.6、§4 | 算 |

## 二、他人积累（只读借鉴）

| 主题 | 来源 | 等级 | 笔记 |
|---|---|---|---|
| Phonak 7 类、每 0.4 s、35 维特征、92% 用自动（n=149,675）| Phonak AutoSense OS 白皮书；Rakita & Jones, Hearing Review 2015 | 官方 | r1 |
| ReSound 7 类、4 s 判定、1 dB/s、222 ms 双耳交换 | gnhearing.com environmental-classifier 白皮书 | 官方 | r1 |
| Learning VC：每场景 EMA 偏移、一致性门控 | Leenen et al., ISAAR 2007 p512 | 论文 | r1 |
| 分类正确率 41–98%（平均约 67%）| Groth & Cui 2017，经 Yellamsetty 2021 综述 PMC7989119 转引 | 论文 | r1 |
| SoundSense Learn：成对比较 10–20 次、≤ 4 参数 | Nielsen et al., IEEE TASLP 23(1):162, doi:10.1109/TASLP.2014.2377581 | 论文 | r1 · r5 |
| 持续 TTA 塌缩：CCC 基准 Tent / CoTTA / RDumb | arXiv 2306.05401 | 论文 | r2 |
| 降噪 TTA 增益小、可能不如原模型 | arXiv 2509.04280 · 2601.14770 · 2508.01847 | 论文 | r2 |
| 护栏：SAR · CoTTA · WiSE-FT · L2-SP · EATA | arXiv 2302.12400 · qinenergy/cotta · 2109.01903 · 1802.01483 · 2204.02610 | 论文 / 源码 | r2 |
| GTCRN 22 BN + 4 LN | github.com/Xiaobin-Rong/gtcrn `gtcrn.py`（L101/118/124/128/196/200）| 源码（s1 已核）| r2 |
| E 核频率与功耗（M1–M5）| eclecticlight.co 2022/01/03 · 2022/10/05 · 2024/02/26 · 2026/04/13 · 2026/06/29 | 二手（powermetrics 实测）| r3 |
| E 核浮点流水线 | dougallj.github.io/applecpu | 源码·微基准 | r3 |
| Gboard 训练条件：空闲 · 充电 · 不计流量；条件变就中止 | Google AI Blog 2017；arXiv 1902.01046 §3 · 1811.03604 §5.2 | 官方 / 论文 | r3 |
| Hey Siri 注册 5、上限 40；Face ID 高质量才更新；Photos 充电时聚类 | machinelearning.apple.com（personalized-hey-siri · recognizing-people-photos）；FaceID_Security_Guide.pdf | 官方 | r3 |
| `NSBackgroundActivityScheduler` · `thermalState` · `isLowPowerModeEnabled` · `BGTaskScheduler` 平台 · QoS 与核 | developer.apple.com 对应页；WWDC20-10686；power_efficiency_guidelines_osx CentralizedTaskScheduling | 官方 | r4 |
| DAS 打分不公开 | eclecticlight.co 2023/01/23 | 二手 | r4 |
| `IOPS*` 通知 | IOKitUser `IOPowerSources.h` | 源码 | r4 |
| 最小统计 / IMCRA 噪声估计 | Martin 2001 TSAP 9(5):504；Cohen 2003 | 论文 | r5 |
| DNSMOS 相关 0.94–0.98、9.01 s 窗、泛化差 | arXiv 2110.01763；`dnsmos_local.py`；2502.05356 · 2402.01413 | 论文 / 源码 | r5 |
| 保守老虎机、SafeOpt | arXiv 1602.04282；ICML 2015 | 论文 | r5 |
| Chrome 音频看门狗、补零超时、唤醒推迟 | chromium `media/audio/audio_thread_hang_monitor.cc`（L34/37）· `services/audio/sync_reader.cc`（L82–86）| 源码（s1 已核行号）| r6 |
| WebRTC FTZ/DAZ、AEC NaN、field trial 结束日期 | webrtc 提交 0441bb62 · 54cf754d · e994058e；`g3doc/field-trials.md` | 源码 | r6 |
| Krisp 设备切换 / 唤醒问题 | krisp.launchnotes.io 发版说明 | 官方 | r6 |
| AudioServerPlugIn 在 coreaudiod 沙箱；Audio Workgroups | Apple QA1811；WWDC20-10224 | 官方 | r6 |
| OSS-Fuzz 音频 fuzzer 清单 | google/oss-fuzz `projects/opus/build.sh`；webrtc `test/fuzzers/` | 源码 | r7 |
| pluginval 测试级别 | Tracktion/pluginval `Source/tests/BasicTests.cpp` | 源码 | r7 |
| RTSan | clang.llvm.org/docs/RealtimeSanitizer.html；lib.rs/crates/rtsan-standalone | 官方 / 二手 | r7 |
| Developer ID 拿不到 Organizer 崩溃报告；MetricKit | help.apple.com/xcode dev9a80ab71d；developer.apple.com MXMetricManager | 官方 | r7 |

**没找到的**（明确写出，不填版面）：Phonak OS 5.0 类目表、助听器滞回秒数、Widex 分类器、语音降噪里冻结主干只训 FiLM 的先例、影子比对的语音文献、风扇起转阈值、Krisp/Zoom/Discord 事故复盘、浸泡测试公开阈值、客户端混沌工具、Rust DSP 的 cargo-fuzz 先例。

**怀疑派 A 没来得及核的**：Vyas 2022 中位数 25 次、Krisp 后台 CPU < 2–3%、Face ID / Photos 原文、Oticon 1200 万场景。
