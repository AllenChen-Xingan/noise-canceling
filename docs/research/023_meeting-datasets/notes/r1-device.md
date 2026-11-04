# r1 消费设备路：调研笔记

> 2026-11-03 · 手下 r1-device。管的实体：真实数据集（消费设备录的那一类）。
> 问题：哪些公开真实录音有笔记本 / Mac 内置麦的段，能不能按设备筛出来。
> **汇总时补注（2026-11-03）**：本笔记是调研员当时的原话，其中「没有 / 只能 / 唯一」类说法在汇总时已收窄为「本轮未确认能筛出的匹配子集」；以 `../decision.md` · `../matrix.md` · `../pick.md` 为准。

## 结论（一句话）
公开集里**没有一份**是「Mac 内置麦 + 会议 + 逐条设备标签」。最接近的三条路：DR-VCTK（MacBook Air，回放）· DNS 2021 blind 文件名里的 31 条 Mac 真人录音（自填标签）· 3D-Speaker（笔记本 9.6 万条带标签，普通话，型号不明）【实测 + 官方文档】

## 方法
- 读官网、README、论文（pdftotext 本地抽文字）。
- 新方法：对大 zip 只用 HTTP Range 读末尾文件目录（每次 < 8 MB），对 GitHub / HF 用文件树接口，拿逐条文件名统计设备字段。没有下载任何音频。

## 查了哪些
| 数据集 | 结果 | 落在哪 |
|---|---|---|
| DNS 2020 / 2021×2 / 2022 / 2023 真实测试集 | 014 已有卡；补字段 + 4 处疑误 | `../cards-014-addendum.md` |
| URGENT 2024 / 2025 / 2026 | 014 已有；2024、2026 无设备信息；2025 未补（r3 已补） | `../cards-014-addendum.md` |
| VOiCES · DAPS · DR-VCTK · ACE · AEC · NISQA-LIVETALK · Clarity CEC3 · CHiME-4 | 014 已有；补字段 | `../cards-014-addendum.md` |
| ConferencingSpeech 2021 | 新卡（真实会议室 + 麦阵，需注册） | `../cards/conferencingspeech-2021.md` |
| ConferencingSpeech 2022 | 他路已写 `tencent-corpus-cs2022.md`、`pstn-corpus.md`、`nisqa-corpus.md`、`iu-bloomington-corpus.md`，本路无新增，不追加 | — |
| SSI 挑战 2023 / 2024 | 新卡（500 条真实设备 + 转写） | `../cards/ssi-challenge-icassp2023-2024.md` |
| DDS | 新卡（iPad + 9 环境回放） | `../cards/dds-device-degraded-speech.md` |
| 3D-Speaker | 新卡（笔记本段最多） | `../cards/3d-speaker.md` |
| MOBIO | 新卡（2008 MacBook，需签 EULA） | `../cards/mobio.md` |
| Libri-Adapt | 新卡（6 设备回放，CC BY 4.0） | `../cards/libri-adapt.md` |
| Mac 设备材料 | 转交 | `r1-device-macmic-handoff.md` |

## 查无此集或不合适（理由）
- **Spotify Podcast Dataset**：已停止对新用户开放（2023-12 起，据二手论文 arXiv 2411.07892），不开卡【论文】
- **VoxConverse**：YouTube 辩论 / 新闻音频，设备不明；官网原句「The audio files are currently not available on this website」；与消费设备录音无关，不开卡【官方文档】
- **ICASSP PLC 挑战**：丢包轨迹叠在干净语音上，麦克风段不是真实设备录，不开卡【推断：据挑战设计，未细读】
- **Meta Casual Conversations v2**：视频为主，设备未写，许可为「limited」自定义；不开卡【官方文档，经检索摘要】
- **Apple / Mac 官方录音集**：未找到任何公开的 Mac 内置麦录音数据集【检索未命中】
- **AEC Challenge `fullband_mobile` 目录说明**：README 链接 404，未读到【实测】

## 需要注册 / 签协议（只记，未注册）
- ConferencingSpeech 2021：挑战注册后发链接与密码；现状未查到
- MOBIO：Zenodo 申请 + EULA，签字人须在编，限非商业

## 待怀疑派复核
- DNS 2023 dev Track2 文件名 `Mobile` 413 条，与论文「dev 全部笔记本或台式机」冲突，以谁为准
- DNS 2021 那 31 条 Mac 录音是内置麦还是外接麦（只能人耳或问组织方）
- 3D-Speaker 音频本身的许可（官网字面只给 metadata 许可）
- `Microphone_Array_RealtekR_Audio` 是否即笔记本内置麦（推断）

## 怀疑派复核（2026-11-03）
- 6 张新卡全部改过（每张卡「## 备注」末行写了改了什么）：
  - `3d-speaker.md` 许可结论由「只评测」降为「未查到」；方言 14 → 13 种；笔记本 96,262 条只是训练集。
  - `mobio.md` 设备键改为 `laptop-other`；「有转写」无出处，改为未查到。
  - `libri-adapt.md` 的 7,200 h 有论文原文，去掉【推断】；补 GB；噪声取自 ESC-50（CC BY-NC），许可加限定。
  - `ssi-challenge-icassp2023-2024.md` 那句许可话属于「Training data」一节。
  - `conferencingspeech-2021.md` 的半真实小时数只指 dev；「回放 VCTK」无出处，已删。
  - `dds-device-degraded-speech.md` 补包内 README 的许可原句与目录结构。
- 补充文件改 9 处，详见 `../cards-014-addendum.md` 末节。最要紧的两条：本路原写的「DNS3 情绪集 584 条」是错的，014 的 600 是对的；DNS5 dev Track2 的 600 条里 599 条就是 DNS4 blind 的复用。
- 上面「待怀疑派复核」4 条的答复：
  1. DNS5 dev Mobile 413 条：以文件名为准，理由见补充文件。
  2. 31 条 Mac 录音是不是内置麦：仍只能人耳听或问组织方，未解。
  3. 3D-Speaker 音频许可：仍未查到，卡上结论已降为未查到。
  4. Realtek 阵列是不是笔记本内置麦：仍是推断，未解。
- 找漏补卡：`../cards/speech-commands-v2.md`（众包真人在笔记本 / 电脑 / 安卓手机上录，CC BY 4.0；无设备标签，每条 1 s）。
- 本路漏查的线索，未开卡：DNS1 dev 有 `SurfaceBook` 44 条，已写进补充文件；DNS5 blind `mt-win_` 81 条可按「Windows 电脑」粗筛。
