# vdn-speaker · 认出你的声音

P2 离线：VAD 驱动的快开慢关 · Kaldi 兼容 fbank · 声纹（ONNX）· 零设置注册 · 不对称门控。设计见 `docs/plan/003_tech-design.md` 第二节，实验见 `docs/experiments/exp005-p2-speaker/README.md`。

## 模块

| 模块 | 回路 | 干什么 |
|---|---|---|
| `vad` | 实时 | earshot 1.2.2：每 256 样本（16 ms @16 kHz）一个人声概率；别的 hop 内部重分块，不延迟声音 |
| `envelope` | 实时 | `VadEnvelopeStage`：p1 快开慢关换成 VAD 判据；`EnergyEnvelopeStage`（p1 能量版，只为对比）；`FloorStage` |
| `gate` | 实时 | `AsymGate`：新人声起始立刻放开；判决窗整个在起始之后的 Other 判决攒够 `confirm_n`（默认 2）份才慢收（0.5 s）；下限 −12 dB |
| `fbank` | 判断 | 80 维 log-mel，Kaldi 算法（对拍基准 kaldi-native-fbank） |
| `embed` | 判断 | WeSpeaker ONNX 声纹，L2 归一化（`onnx` feature；ORT Environment 由调用方传入） |
| `vecmath` | — | 归一化、余弦等向量小工具 |
| `enroll` | 判断 | 在线聚类 + 「时长 × 近讲」打分 + 稳定才锁 + 可改判 |
| `judge` | 判断 | 1.5 s 窗 / 0.5 s 一次：挑清楚单人片段喂注册器、对主人打分、发 `SpeakerVerdict` |
| `types` | — | engine 接口镜像，逐字段照 align 的 `vdn-types` 草案 |

实时级 `latency_samples() = 0`、零前瞻；判断回路只改参数、不碰声音。

## 依赖与模型（许可）

| 东西 | 版本 / 来源 | 许可 | 备注 |
|---|---|---|---|
| earshot | crates.io `=1.2.2`（pykeio） | MIT OR Apache-2.0 | **发布阻断**：内嵌权重的训练数据来源未说明（009 §5）。纯 Rust、零分配 |
| realfft | 3.5 | MIT | fbank FFT |
| ort | `=2.0.0-rc.15`（同 workspace） | MIT OR Apache-2.0 | 声纹推理 |
| **声纹模型 ECAPA512** | HF `Wespeaker/wespeaker-voxceleb-ecapa-tdnn512` @ `89426e111abc9ffcc40279baf808e087693c6e9b`，文件 `voxceleb_ECAPA512.onnx` | 模型卡标 **Apache-2.0**；训练数据 VoxCeleb（**CC BY 4.0，需署名**，005 拍板 #2 默认 A） | 24 861 931 字节，sha256 `f901ae3571912bad4e11e157744cb8b7d77a0b46923b83c410bdbb6c77d69736`；放 `~/vdn-data/models/wespeaker/`，不进仓。192 维；输入 `feats [1,T,80]`、输出 `embs` |

- 模型下载：WeSpeaker 文档里的 wenet.org.cn 链接 2026-10-30 证书过期（`resources/wespeaker/docs/pretrained.md:53`），改从同一组织的 HF 仓库取，钉提交号
- 同目录另存了模型卡 `README.hf.md`（sha256 `4bcf87ecfbbb8e07a01b21415a970c8b53a5283bf6872b657040d3f45c9241f7`）与训练配置 `config.yaml`（sha256 `7515e0d1a73accd160f85b05b4a76d505a3c545457eb05b6fc61e7614db50d5a`）
- 查找顺序：`--speaker-onnx` → 环境变量 `VDN_SPEAKER_ONNX` → `~/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx`
- 换 WeSpeaker CAM++（512 维）：同前端，直接换文件；换 3D-Speaker CAM++：改 `FbankConfig`（povey 窗、输入不放大）

## feature

- 默认不带任何 feature：核心只依赖 `vdn-dsp`、earshot、realfft（规约 §2.2 规则 1）
- `onnx`：声纹推理（ort），过渡期例外，拆出推理后端 crate 后移走
- 没有 `offline` feature 了：离线实验命令行在 `src/speaker-lab/`（2026-10-31 p2fix），本 crate 的依赖闭包里不再有 engine、models、hound、serde_json（check-deps 守）

## 档位

`metadata.vdn.tier = "B"`：`envelope` / `gate` / `vad` 的 `process` / `reset` 每 hop 跑在降噪线程（规约 §4 的 B 档线程），这些 `impl` 块升了禁用清单（§5.3）；判断回路 `fbank` / `embed` / `enroll` / `judge` 是 C 档，模块头 `#![allow(clippy::indexing_slicing)] // NRT`。规约 §2.1 表里写的是 C，只算了声纹那半

## 接入注意（下一批接 engine 时）

1. **ORT Environment 一个进程只许一个**（`ort-2.0.0-rc.15/src/environment.rs:514-516`，取当前环境的函数是 `pub(crate)`）。`vdn-models` 现在自己建一个（`src/backend-ort/src/lib.rs:12-18`）；`Embedder::open(&env, …)` 已改成由调用方传入。接入时由宿主（或 align 的推理后端层）建一个，vdn-models 也改成接收它
2. **时钟**：`SampleClock` 按 IO 采样率计。`GateParams::clock_rate` 设成 IO 采样率；`Judge::push_audio(wet, dry, start)` 的 `start` 传本块首样本的 IO 时钟，`JudgeParams::clock_per_sample` = IO 采样率 / 16000
3. **VAD 只跑一份**：链上谁先算 earshot 概率就填进 `FrameCtx::voice_prob`，包络和门控都优先读它
4. **两级下限会叠加**：保守链是「包络（混回原声，下限 −12 dB）→ 门控（× g，下限 −12 dB）」，门控收到底时原声残留约 −24 dB、降噪部分 −12 dB。激进链不混回原声，只有门控下限
5. 判断回路要两路输入：降噪输出（算声纹）和对齐的原声（量电平；近讲线索与门控同一口径）
6. 类型镜像 `types.rs` 逐字段照 vdn-types；换成 `use vdn_types::…` 时注意 `EnrollOutcome`（本 crate）≠ `EnrollEvent`（vdn-types 决策日志，`Locked/Relocked/Reset`），需要映射；`Stage::set_params(&ParamSet)` 接入时补

## 发布前（本批不发布）

- NOTICE：VoxCeleb 署名原文（CC BY 4.0）+ WeSpeaker 模型 Apache-2.0 声明
- earshot 内嵌权重来源未说明 = **发布阻断**（009 §5）

## 命令行（离线实验）

已挪到 `src/speaker-lab/`（`vdn-speaker-lab`，tier = tool），见那里的 README。
