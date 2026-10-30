# 现状页 · pipeline（批次 20261029-b1）

- **状态**：进行中
- **手上**：Mac 实时音频管线与虚拟麦克风的开源参考调研（003）
- **交代**：`_tmp/20261029-b1/pipeline.md`
- **做到哪**（2026-10-29）：六份文件都已填完；交付前自审进行中
- **下一步**：按自审意见修完 → 报大总管「pipeline 可以收尾」

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| state/handoff/pipeline.md | 新建现状页 |
| docs/research/003_mac-audio-pipeline/sources.md | 填自有 / 他人积累登记 |
| docs/research/003_mac-audio-pipeline/assign.md | 末尾补本线实际使用的核查表 |
| docs/research/003_mac-audio-pipeline/gap.md | 填两类发现、候选横向表 |
| docs/research/003_mac-audio-pipeline/decision.md | 取舍卡、分阶段方案、8 条待拍板 |
| docs/research/003_mac-audio-pipeline/README.md | 五份状态勾上 |
| docs/research/003_mac-audio-pipeline/must-know.md | 起草三条命门、决定一句话、排除项、预判 P1–P5（AI 起草，待人改） |

## 发现
- 大总管实为 noise-canceling-69（交代已改）

## 待人拍板
共 8 条，题面 · 选项 · 代价 · 建议 · 默认见 `docs/research/003_mac-audio-pipeline/decision.md` 末节表格。最急的两条：
1. **本机现在装 BlackHole 吗**（M0 前提，本机未装）：A 现在 `brew install blackhole-2ch` 并重启 ← 建议 / B 推后 M0
2. **批准插件加载与崩溃实验**（要写 `/Library/Audio/Plug-Ins/HAL`、`killall coreaudiod`，越过本批红线）：A 批准，放在 M3 开头 ← 建议 / B 等开发者账号 / C 不做
其余：何时办 Apple Developer Program（建议 M3 前）· 出事时送原声还是静音（建议原声）· 回声消除用不用 process tap（建议默认关）· BlackHole 阶段给不给外部用户（建议不给）· 装完 killall coreaudiod 还是重启（建议前者）· must-know 命门按核实改写（建议照改）

## 转交（范围外，留给大总管）
- 取舍卡确认后的结论应整理进 `vault/truth/`（本手下无写权限）
- 002 模型线须知：附加延迟能否到 20 ms 取决于模型前瞻；管线侧直通推算约 15–20 ms（含 10 ms 帧），见 `gap.md` 2a
- 004 评测线须知：参考仓库都没有附加延迟实测，M0 用 cpal 两端回调时间戳实测
- 本机缺 meson（webrtc AEC3 的 bundled 构建要用）

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
- 20:59 对抗核实 Workflow 完成（5/5）→ 更正：插件在独立宿主进程（本机开机日志实测）；ParrotAudioPlugin 是 Apple 的；本机未装 BlackHole；断供送静音违背「从不吃字」；ad-hoc 签名能否加载查不实
- 20:59 写 gap.md（第〇节更正表 + 逐项核对 + 候选横向表 + 没做的）、decision.md（分阶段方案 + 8 条待拍板）；修 sources.md 待核两条与本机条目；must-know 加更正注记 → 完成
- 20:59 交付前自审：派 3 个子代理（怀疑派 · 实现派 · 产品派+放行测试）→ 进行中
- 21:00 自审·产品派+放行测试回来 → 放行三问答得出；3 条 P1（卡片把待拍板第 4/6/8 条和 arm64-only 写成已定）+ 6 条 P2/P3，待合并修
