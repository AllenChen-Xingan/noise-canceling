# 出处总表 · 022 Mac 内置麦克风

> 2026-11-03 · macmic。用途：列出 `devices.md` · `simulation.md` · `decision.md` 里每个数字靠的出处，以及怀疑派复核的结果。
> 「复核」一栏：✅ 怀疑派或我回原文核过 · ⚠️ 核过但要加限定 · ❌ 被推翻（已从结论里拿掉）· — 未复核。
> 各路原始报告与逐条判定在 `findings/`：`apple` · `measure` · `sim` · `data`（各带 `-skeptic`），以及我亲手核的 `tunings`。

## 一、本机系统文件（最硬）

| # | 出处 | 支撑什么 | 复核 |
|---|---|---|---|
| S1 | `/System/Library/Audio/Tunings/AID40/DSP/Graphs/builtin_microphone_general.dspg` | 标准模式处理链（直流隔离 → micSense → 波束 → ToneMeister → 输入音量） | ✅ 我亲读原文 |
| S2 | 同目录 `DSP/DSPDatabase.plist` | General / Telephony / Measurement / Movie 共用 general 图；Voice、VoiceRaw、Dictation、Diagnostic 各自的图 | ✅ 我亲读 |
| S3 | `.../Strips/builtin_microphone_general_aufx_bmfd.plist`、`..._aufx_sens.plist` | 固定波束系数（481 点 × 3 麦）；每麦底噪谱 | ✅ `tools/read_tunings.py` 解码，带测试 |
| S4 | `Config/AID40.plist` | 内置麦输入音量 −12 到 +12，初值 0 | ✅ 我亲读 |
| S5 | `ioreg -l`（「Digital Mic」节点、`lp-mic-device`） | 硬件进系统 3 路 48 kHz；低功耗麦 3 路 16 kHz | ✅ 我亲读 |
| S6 | `system_profiler SPHardwareDataType` | 本机 MacBook Air M5，Mac17,3 | ✅ |

## 二、苹果官方

| # | 出处 | 支撑什么 | 复核 |
|---|---|---|---|
| A1 | support.apple.com 技术规格 111933、111948（Air 2018/2019「Three microphones」）；111991、111883、111867、111346、118551、118552、122209、126320（Air 2020 Intel 至 M5「Three-mic array with directional beamforming」）；111924（Air 2017「Dual microphones」） | 各代 Air 的麦数与原文 | ✅（15″ M4 Air 页未查） |
| A2 | 111932（MBP 16″ 2019 起「Studio-quality three-mic array with high signal-to-noise ratio」）；MBP / iMac / Studio Display 规格页 | Pro 与 Air 的措辞差 | ✅ |
| A3 | Apple Newsroom 2022-06 M2 Air：「advanced beamforming algorithms」「integrated between the keyboard and display」 | 波束是算法；麦在转轴区 | ✅ |
| A4 | Apple Newsroom 2020-11：「fanless design」 | Air 无风扇 | ✅ |
| A5 | SDK `AudioUnitProperties.h:2623-2637`（Bypass、AGC 默认开）；`AVCaptureDevice.h:2715-2746`（三种模式定义、用户选、app 只读） | VPIO 与麦克风模式 | ✅（行号以 macOS 27 SDK 为准） |
| A6 | WWDC23 10235「What's new in voice processing」 | VPIO 内含回声消除、降噪、自动增益；模式按 app 生效 | ⚠️「只对 VPIO 生效」是推断，原话没说其他接口不行 |
| A7 | support.apple.com/guide/mac-help/mchlp2567（15.0 版） | 「环境降噪」开关在 Apple 芯片与 T2 机型上不出现；26 版已删 | ✅ |
| A8 | support.apple.com/102217 | 蓝牙开麦后音质降低 | ✅ |
| A9 | Apple 平台安全指南「Hardware microphone disconnect」 | 合盖硬件断麦，≤ 500 ms | ✅ |

## 三、他人实测

| # | 出处 | 支撑什么 | 复核 |
|---|---|---|---|
| M1 | DXOMARK「Apple MacBook Air 13 (2022)」评测页内嵌数据 | M2 Air 13″ 频响、指向性、THD+N；评语「a bit thin-sounding」 | ⚠️ 数字复现一致；1 台；频响用的 app 与模式未写；跨页曲线纵轴不能比电平（原报告「Air 比 Pro 低 9.5 dB」已删 ❌） |
| M2 | DXOMARK「MacBook Air 15 M2 (2023)」 | 15″ 频响与指向性 | ⚠️ 1 台；没有录音失真图 |
| M3 | DXOMARK MBP 14″ M2 Pro / M3 Pro | Pro 较平；「Noise reduction is disabled by default」 | ⚠️ 各 1 台；与系统文件里 Pro 组有 `ocns` 矛盾，待实测 |
| M4 | Zoom 社区 41632（Zoom 弹窗原文「turn off Voice Isolation」） | Zoom Mac 客户端会触发语音突显 | ✅；15516 / 18153 是同一作者的同一段文字，原报告「≥3 人」「2 个组织」❌ 改为 1 人 1 组织 |
| M5 | WebKit bug 262759 | 稳定音约 5 秒后被压低一半以上——原因是**降噪**，不是自动增益 | ✅ 用途已改 |
| M6 | arXiv 2308.01074（键盘声识别） | MBP 16″ 2021 内置麦 + Zoom 低档降噪，键盘声仍可识别 93% | ✅ |
| M7 | Chromium `media_switches.cc:1583`；`audio_low_latency_input.cc:772-777`；`media/audio/mac/audio_manager_mac.cc:1214-1250` | Chrome 默认不走 VPIO，但会动设备级 `'nzca'` 与输入音量 | ✅ |
| M8 | Mozilla cubeb `backend/mod.rs:3561,3881,3921` | 别的进程开 VPIO 时 HAL 电平很低；VPIO 下混单声道 | ✅ |
| M9 | Asahi Linux 文档与 `asahi-audio` `firs/j313|j413|j415/mic.json` | M1/M2 Air 三路 PDM、无前置放大、48 kHz；2 cm 三角几何 | ⚠️ 只覆盖 M1/M2；几何与 Pro 共用，像模板；增益是绕软件限制的调参 |

## 四、模拟方法

| # | 出处 | 支撑什么 | 复核 |
|---|---|---|---|
| P1 | URGENT 2025 `conf/simulation_train.yaml:58-110`；`generate_data_param.py:14` | 削波分位、编解码、丢包、失真个数概率；降采样 8–44.1 kHz（带宽 4–22 kHz） | ✅（原报告把采样率当带宽 ⚠️ 已改） |
| P2 | URGENT 2026 `simulate_data_from_param.py:29-56` | 70 Hz 高通（默认关，脚本里打开） | ⚠️ |
| P3 | DeepFilterNet `libDF/src/augmentations.rs`、`dataset.rs` | 随机二阶 / 双二阶滤波、削波、带宽截止 | ⚠️ 原报告漏了搁架滤波与「语音削波默认概率 0」 |
| P4 | Braun & Tashev arXiv 2008.06412 §6.1（源自 RNNoise） | 频谱整形系数 ±3/8；电平均值 −28 dBFS、方差 10 dB；频谱增强只多 +0.01 PESQ | ✅ |
| P5 | MicAugment arXiv 2010.09658 | 设备冲激响应 + 带通 + 非线性；MacBook Pro 自放自录作真实测试集（非配对，关键词识别） | ⚠️ 原报告「配对」❌ 改为非配对 |
| P6 | WebRTC AGC2（`resources/sonora`） | 自动增益代理参数 | ⚠️ 限幅拐点 −0.75 dBFS |
| P7 | ADI AN-1112 | MEMS 等效输入噪声 = 94 − 信噪比 | ✅ |
| P8 | ETSI TS 103 224 V1.7.2 §5.3.3（scenes 转交，021 `sources.md` S02/S03） | 桌面免提布置 | — 由 scenes 核 |

## 五、数据

| # | 出处 | 支撑什么 | 复核 |
|---|---|---|---|
| D1 | DR-VCTK README（datashare.ed.ac.uk 10283/3038）与论文 arXiv 1911.03952 | 2013–2017 楔形双麦 Air；CC BY 4.0；MacBook 段只在训练集，3,964 条；各声道句子互不重叠 | ✅ |
| D2 | 3D-Speaker arXiv 2306.15354 表 2 | 全库笔记本 96,262 条（不是训练集），品牌未写 | ✅ 已改 |
| D3 | MODALITY（Springer 10.1007/s10844-016-0438-z） | 笔记本麦 + B&K 参考，只评测 | ⚠️ 许可原句位置未复核 |
| D4 | ASVspoof 2017 v2、EchoFake、LRPD、ReMASC、AVspoof | 都没有 Mac 当录音设备 | ✅ |
| D5 | mtgdata 转交 `r1-device-macmic-handoff.md`（DNS 2021 Mac 众包 31 条等） | `mac-unknown` | — mtgdata 已核 |
