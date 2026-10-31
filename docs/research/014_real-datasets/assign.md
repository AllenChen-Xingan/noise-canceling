# 派活记录 · 014

> 2026-10-31。Workflow `wf_5c707fee-0a3`，10 个代理：5 路调研，每路接 1 个怀疑派（流水线，不等齐）。

## 分组（按数据集族，同类同批）

| 路 | 范围 | 预判（让它推翻） |
|---|---|---|
| A 会议 | AMI · ICSI · AliMeeting · AISHELL-4 · NOTSOFAR-1 · LibriCSS | NOTSOFAR 含单设备 / 笔记本、可商用评测 |
| B CHiME 与可穿戴 | CHiME-5/6/7/8 · DiPCo · Mixer 6 · CHiME-9 ECHI · MISP · EasyCom | CHiME-5/6 禁商用需签协议 → **被推翻**（2024 起 CC BY-SA 4.0） |
| C 降噪真实测试集 | DNS 各届（含个性化）· AEC · URGENT · REAL-M · VOiCES · CHiME-7 UDASE · Clarity | DNS 盲测无转写只能听 → **部分推翻**（盲测已转写未公开；dev 有 WAcc 表；个性化 dev 有邻座） |
| D 咖啡馆 / 设备麦搜罗 + 中文 | 全网找「真实公共场所 + 设备麦」；CHiME-3/4 · Common Voice · ARTE · AISHELL-5 · Libri-Adhoc40 等 | 没有笔记本麦 + 咖啡馆 → **成立** |
| E 设备链 / RIR | DAPS · BUT ReverbDB · ACE · MIRaGe · AIR · VoiceHome-2 · DIRHA 等 | ACE 是唯一含笔记本麦的 RIR → 成立（Chromebook Pixel）；怀疑派另找到 DR-VCTK 含 MacBook Air |

## 每个代理的四件事

- **逐条核查**：卡片模板 12 项（做什么 · 设备与距离 · 环境 · 人数与重叠 · 语言 · 参考 · 大小 · 许可原文名 · 获取 · 接近度 · 用途 · 出处），另加 6 个结构化字段（真实录音 / 参考用「有 · 部分 · 没有」回答）
- **证据格式**：每条带 URL 与证据等级；查不到写「未查到」
- **字数上限**：每卡 ≤ 600 字
- **硬禁令**（写死在提示里）：不许构建、运行程序，不许下载数据集本体，只读网页 / 论文 / LICENSE
- **怀疑派**：逐卡回官方页复核许可、获取、设备、参考、大小；默认怀疑二手、过时、把回放当真实、把 SA 当可训练；压高估的接近度；列漏项

## 主线补做

- 合并跨路重复卡（NOTSOFAR、AliMeeting、AISHELL-4、AMI、ECHI、DAPS、DiPCo、EasyCom、CHiME-5/6、VOiCES），冲突裁定写进 decision 附录
- 三个怀疑派都点了 RealMAN：主线直接核 GitHub 与 arXiv 2406.19959，建卡；另补 NISQA LIVETALK
