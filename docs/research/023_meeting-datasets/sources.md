# 来源登记

> 2026-11-03 · 手下 mtgdata · issue #82。用途：说清这批结论从哪来、怎么查的、哪里能复核。
> **URL 不在这里重抄**：每份数据集的出处在它卡上的「出处」字段（`cards/*.md`），014 已有集在 `cards-014-addendum.md` 各节。本文只记方法与读过的类别，避免两处 URL 对不上。

## 一、两类来源

| 类别 | 是什么 | 在哪 |
|---|---|---|
| 自有积累 | 014 的 51 张卡、gap、decision；016 `benchmark.md` 第三、五节；决策 D62 · D65 · D80 | `../014_real-datasets/` · `../016_bench-synth/benchmark.md` · `../../plan/001_decisions-20261029-b1.md` |
| 他人积累 | 数据集官网、LICENSE / 数据卡原文、论文（arXiv / Interspeech / ICASSP）、GitHub / HuggingFace / Zenodo / OpenSLR 页面、LDC 目录页 | 各卡「出处」字段 |
| 兄弟线 | 场景键正本 021 `catalog.md`（scenes）· 设备键正本 022 `devices.md`（macmic） | 合并前在 worktree `noise_canceling-scenes` / `noise_canceling-macmic` |

## 二、怎么查的

- **workflow `wf_98de0537-23d`**：4 路调研员 + 4 个怀疑派，2026-11-03 跑完，共 595 次工具调用【实测：workflow 返回】。每路的提示词与返回值在会话目录 `subagents/workflows/wf_98de0537-23d/journal.jsonl`（不入库）
- **读文件名代替下载**：对大 zip 只用 HTTP Range 读末尾的文件目录（每次 < 8 MB），对 GitHub / HF 用文件树接口，按文件名数设备字段。**没有下载任何音频**；单次读取都 < 50 MB【实测：调研员与怀疑派自报，r1 笔记「方法」一节】
- **许可**：一律要求读到原文（LICENSE、数据卡、条款页）；读不到写「未查到」，不按常见说法填。口径：CC BY / CC0 / MIT / CDLA-Permissive → 训练可；SA / NC / 仅研究 / 自定义 → 只评测；LDC 付费 → 不可
- **怀疑派**：默认对方错，逐条回原文推翻；改动写在各卡「## 备注」末行和各路笔记末节

## 三、各路笔记

| 路 | 笔记 | 管什么 |
|---|---|---|
| ①消费设备 | `notes/r1-device.md` · `notes/r1-device-macmic-handoff.md`（已转交 macmic） | 笔记本 / 手机 / 平板录的真实噪声语音；能否按设备筛 |
| ②按地点 | `notes/r2-place.md` | 7 类地点各有哪些真实录音；笑声、附和、情绪 |
| ③无参考 | `notes/r3-noref.md` | 没有干净参考时能算的 13 个打分器；带人评分的校准集 |
| ④中文 | `notes/r4-zh.md` | 中文会议 / 对话真实录音 |

## 四、没读到原文的（结论已按「未查到」降级）

各卡「许可」字段里写「未查到」的都属此类，`check_cards.py` 要求它们仍写「结论：未查到」。重点几份：3D-Speaker 音频许可 · MISP2021 许可与入口 · DNS 各届测试集单列许可 · SIG / SSI 挑战测试集许可 · Tencent / PSTN 语料许可 · TMHINT-QI 数据许可 · MSP-Podcast 官网（证书校验失败）。
