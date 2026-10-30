# 现状页 · pipeline（批次 20261029-b1）

- **状态**：进行中
- **手上**：Mac 实时音频管线与虚拟麦克风的开源参考调研（003）
- **交代**：`_tmp/20261029-b1/pipeline.md`
- **做到哪**（2026-10-29）：must-know、sources、assign 已写；对抗核实 Workflow 进行中
- **下一步**：核实结果回来后写 gap.md、decision.md，修 sources 里的「待核」两条，再自审收尾

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| state/handoff/pipeline.md | 新建现状页 |
| docs/research/003_mac-audio-pipeline/sources.md | 填自有 / 他人积累登记 |
| docs/research/003_mac-audio-pipeline/assign.md | 末尾补本线实际使用的核查表 |
| docs/research/003_mac-audio-pipeline/must-know.md | 起草三条命门、决定一句话、排除项、预判 P1–P5（AI 起草，待人改） |

## 发现
- 大总管实为 noise-canceling-69（交代已改）

## 待人拍板
（暂无）

## 转交（范围外，留给大总管）
（暂无）

## 流水
- 20:41 读交代、共同规矩、手下协议 → 完成
- 20:41 向 5-app-89 报到失败（名单无此名）→ 写进本页首行，并试发 noise-canceling-69
- 20:45 大总管确认身份；克隆 12 个仓库进 resources/（共约 50 MB，已被 git 忽略）→ 全部成功
- 20:45 写统一核查表（scratchpad/checklist.md），派 5 个子代理分头核查 → 进行中
- 20:45 起草 must-know.md，核对设计稿行号并修正 4 处差一行 → 完成
- 20:45 人新要求（经大总管）：用 Workflow 做深调研 → 回报计划，获批；起 Workflow 补 4 条空白（签名 / IPC / 崩溃 / 先例）→ 进行中
- 20:45 大总管追加：带上 macOS 26/27 的变化 → 排进第二个 Workflow
- 20:46 BackgroundMusic 核查回来 → 只借鉴架构（scratchpad/out-backgroundmusic.md）
- 20:46 libASPL+roc-vad 核查回来 → libASPL 可改造（MIT），roc-vad 只借鉴架构；roc-vad 把 DSP 放在 coreaudiod 内，是 P1 的反例（scratchpad/out-libaspl-rocvad.md）
- 20:46 MetalVoice+AudioCap 核查回来 → MetalVoice 是反例（回调里分配内存加锁、无漂移补偿）；AudioCap 可改造（BSD-2）；P3 部分成立（多一道系统录音权限，与麦克风时钟对齐未验证）
- 20:46 Rust 零件核查回来 → 6 个都可直接用；P4 改判：异步推理线程不该加入设备 workgroup，应自建 work interval，Rust 无现成封装（scratchpad/out-rust-parts.md）
- 20:46 BlackHole+Apple 示例核查回来 → BlackHole M0–M2 直接用、最终只借鉴；P2 成立带 4 个坑；发现 AudioServerPlugIn.h:34 称插件跑在独立进程，可能改写命门 1 前提，排进对抗核实（scratchpad/out-blackhole-apple.md）
- 20:47 本机只读实测（macOS 27.0.1）：/Library/Audio/Plug-Ins/HAL 下有第三方 ParrotAudioPlugin.driver；进程表无独立「Core Audio Driver」宿主，只有 coreaudiod(570) 与 com.apple.audio.DriverHelper(585)；无 sudo 无法 lsof 确认插件被谁加载 → 交对抗核实
- 20:50 补检索 Workflow 完成（4/4）→ 签名：给别人用需 Developer ID+公证；DriverKit 排除；IPC：开源先例都不用 shm 传音频；先例：无更好的，登记 tympan-aspl
- 20:50 起第二个 Workflow（5 代理：macOS 版本差异 · 插件宿主进程专项 · 怀疑派 · 实现派 · 产品派）→ 进行中
- 20:50 写 sources.md（自有 + 他人四类登记）、assign.md 末节补实际核查表 → 完成
