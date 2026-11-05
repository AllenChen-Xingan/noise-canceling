# 027 · 训练数据下载清单（待大总管汇总问人；本线不自行下载）

> 2026-11-05 · 手下 finetune。许可「复核」= 主会话亲手读了出处页；其余是子代理读到的（`notes/data-licenses.md`）。
> 判据：D63（测试份永不进训练）· D64（BY-SA 可训练，权重按 BY-SA 共享并署名；NC 只评测）· D57（评测音频不进训练）。
> 为什么要下：本机能合法训练的只有 DEMAND 6 个环境（食堂 · 露台 · 餐馆 · 厨房 · 街道 · 公交）每段前 60%，和 LibriSpeech test-clean 偶数位 20 人（约 2.7 小时）。车内 · 地铁 · 候车厅的 DEMAND 被测试份 / 评测集占用，ESC-50 是 NC，本机三套房间冲激响应全被评测集用了（exp013 README 第二、四节）。

## 第一批：第一个最小实验够用（约 16.1 GB，解压峰值约 32 GB）

| # | 数据集 | 类别 | 补哪些弱场景 | 许可（原文 · 出处） | 能否训练 | 大小 | 时长 / 条数 | 和评测集撞源？ |
|---|---|---|---|---|---|---|---|---|
| A1 | LibriSpeech train-clean-100 | 干净人声（英） | 全部（主说话人） | "CC BY 4.0"（openslr.org/12，**复核**） | 能 | 6.3 GB | 100 小时 · 251 人 | 不撞：评测只用 test-clean |
| A2 | QUT-NOISE（CAFE · CAR · STREET · HOME · REVERB） | 噪声实录 | 咖啡馆 · 车内 · 街边 | "The QUT-NOISE data itself is licensed CC-BY-SA"（QUT SAIVT 官网，**复核**） | 能（BY-SA） | 约 7.7 GB（五个包 1.4–1.7 GB） | 未核 | 不撞 |
| A3 | TUM 风噪集 | 风噪（手机实录 + 合成） | 街边 + 风 | "Creative Commons Attribution 4.0 International"（Zenodo 6687982，**复核**） | 能 | 0.14 GB | 未核 | 不撞 |
| A4 | OpenSLR 28 RIRS_NOISES | 房间冲激响应（实测 RWCP · REVERB 2014 · Aachen AIR + 模拟）+ 点声源噪声（取自 MUSAN） | 全部（混响） | "License: Apache 2.0"（openslr.org/28，**复核**；内部各来源的许可页面未列，未逐个核） | 能（内部来源待核） | 1.3 GB | 未核 | 不撞：评测用 MIT · ACE · McVAMPIRE |

| A5 | LibriSpeech dev-clean + test-other | 干净人声（英），**只给终测** | 终测的主说话人与旁人 | "CC BY 4.0"（openslr.org/12，**复核**） | 能（这里只测不训） | 约 0.33 + 0.33 GB | 各约 5 小时 | 不撞：说话人与 test-clean、train-clean-100 都不同 |

第一批合计约 16.1 GB。每个来源下载后**先按 `decision.md` 第六节的表切训练 / 开发 / 测试三份**，切法与哈希进仓，再生成数据。

本机已有、可直接用：DEMAND 6 个环境前 60%（0 GB）。另加风噪模拟器 `audiolabs/SC-Wind-Noise-Generator`（MIT，代码，几 MB）。

## 第二批：补车站 / 地铁 / 公交、街边交通（约 67 GB 下载）

| # | 数据集 | 类别 | 补哪些弱场景 | 许可（原文 · 出处） | 能否训练 | 大小 | 时长 / 条数 | 和评测集撞源？ |
|---|---|---|---|---|---|---|---|---|
| B1 | CochlScene | 场景实录（手机，单声道 44.1 kHz） | 公交 · 地铁车厢 · 地铁站 · 咖啡馆 · 餐馆 · 小汽车 · 街道 · 拥挤室内（13 类里用 8 类） | "Creative Commons Attribution Share Alike 3.0 Unported"（Zenodo 7080122，**复核**） | 能（BY-SA），需筛人声 | 53.9 GB | 76,115 条 × 10 s ≈ 211 小时；每类约 5,800 条（论文表 1，**复核**） | 不撞 |
| B2 | SONYC-UST v2 | 城市声实录（纽约） | 街边：引擎 · 喇叭 · 警报 · 施工 | "Creative Commons Attribution 4.0 International (CC BY 4.0)"（Zenodo 3966543，**复核**） | 能，需按标签去掉人声类 | 13.3 GB | 18,515 条 × 10 s ≈ 51 小时 | 不撞 |

- **盘量**：B1 下载后先转 16 kHz、只留 8 类，推算落盘约 12 GB（53.9 × 8/13 ÷ 2.75）；但解压时 zip 与解压目录同时在，峰值约 66 GB（推断）。本机剩约 125 GB；如果 D65 的 40 GB（解压峰值约 80 GB）也要下，两者不能同时解压
- **CochlScene 的人声**：论文写验证者拒掉了能听清或涉及隐私的人声，但「有些提交可能含录音者本人的说话声」（论文 §数据收集，**复核**）。当噪声用前要过一遍语音检测，筛掉有人声的段（推断：否则模型会学着把人声当噪声压掉）

## 第三批（可选）：中文主说话人

| # | 数据集 | 许可 | 大小 | 说明 |
|---|---|---|---|---|
| C1 | AISHELL-3 | "Apache License v.2.0"（openslr.org/93，子代理读到） | 19 GB | 约 85 小时 · 218 人；评测集的中文主说话人只有 2 人，中文训练数据先不做第一个实验的前提 |

## 不建议下（查过不能训练或风险大）

WHAM! noise（CC BY-NC）· TAU Urban 2020（非商用）· CHiME-3/4（LDC 付费）· SINS（两处许可冲突，一处写仅限学术）· BBC Sound Effects（仅限个人 / 教育 / 研究）· Sonniss GDC（明文禁止 AI/ML 训练）· DNS5 noise（58 GB，内含 DEMAND 整段、和评测集撞源；AudioSet 部分的音频版权未覆盖）· MAGICDATA · ST-CMDS（NC-ND）。

## 时长

- 下载速度未量化。参照本机上一次从 Zenodo 下 DEMAND：326 MB 约 16 分钟，约 0.34 MB/s（`~/vdn-data/download.log` 的文件时间戳，推断）
- 第一批 15.4 GB：按 0.34 MB/s 约 12.6 小时；按 5 MB/s 约 50 分钟。第二批 67 GB：约 2.3 天 / 约 3.7 小时（同两档）
- 建议先下第一批，同时实测速度，再决定第二批
