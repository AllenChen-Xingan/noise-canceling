# vdn-speaker · 认出你的声音

P2 离线：VAD 驱动的快开慢关 · Kaldi 兼容 fbank · 声纹（ONNX）· 零设置注册 · 不对称门控。设计见 `docs/plan/003_tech-design.md` 第二节，实验见 `docs/experiments/exp005-p2-speaker/README.md`。

## 模块

| 模块 | 回路 | 干什么 |
|---|---|---|
| `vad` | 实时 | earshot 1.2.2：每 256 样本（16 ms @16 kHz）一个人声概率；别的 hop 内部重分块，不延迟声音 |
| `envelope` | 实时 | `VadEnvelopeStage`：p1 快开慢关换成 VAD 判据；`EnergyEnvelopeStage`（p1 能量版，只为对比）；`FloorStage` |
| `gate` | 实时 | `AsymGate`：新人声起始立刻放开；判决窗整个在起始之后、连续两次 Other 才慢收；下限 −12 dB |
| `fbank` | 判断 | 80 维 log-mel，Kaldi 算法（对拍基准 kaldi-native-fbank） |
| `embed` | 判断 | WeSpeaker ONNX 声纹，L2 归一化（`onnx` feature） |
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
- 同目录另存了模型卡 `README.hf.md`（sha256 `4bcf87ec…41f7`）与训练配置 `config.yaml`（sha256 `7515e0d1…0d5a`）
- 查找顺序：`--speaker-onnx` → 环境变量 `VDN_SPEAKER_ONNX` → `~/vdn-data/models/wespeaker/voxceleb_ECAPA512.onnx`
- 换 WeSpeaker CAM++（512 维）：同前端，直接换文件；换 3D-Speaker CAM++：改 `FbankConfig`（povey 窗、输入不放大）

## 命令行（离线实验）

```sh
vdn-speaker run   <noisy.wav> <out_dir> [--model-path gtcrn.onnx] [--speaker-onnx ecapa.onnx] [--preload owner.json] [--save-owner owner.json]
vdn-speaker fbank <in.wav> <out.f32>
vdn-speaker embed <in.wav>
```

`run` 一次出 `raw / floor / energy / vad / personal` 五个版本和一份 `log.json`（判断回路每次判决、注册事件、逐 hop 增益与耗时）。
