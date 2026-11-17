# 034 · 来源清单

> 2026-11-16 查。「读源码」= 读了原文件；「摘要」= 只读了摘要或检索摘录，没读全文。

## 一、skill / 插件

| 编号 | 名字 | 链接 | 读到什么程度 |
|---|---|---|---|
| S1 | `huggingface/skills` 仓库首页（24 个 skill 列表、安装方法） | https://github.com/huggingface/skills | 读页面 |
| S2 | `huggingface-llm-trainer` 的 `SKILL.md` | https://raw.githubusercontent.com/huggingface/skills/main/skills/huggingface-llm-trainer/SKILL.md | 读全文（经摘要工具） |
| S3 | `krasserm/ml-plugins`（`ml-research` 插件，HF `ml-intern` 的 Claude Code 版） | https://github.com/krasserm/ml-plugins · https://github.com/huggingface/ml-intern | 读 README |
| S4 | Orchestra Research `AI-Research-SKILLs`（`0-autoresearch-skill` · `09-infrastructure` · `13-mlops`） | https://github.com/Orchestra-Research/AI-Research-SKILLs | 读 autoresearch 的 `SKILL.md` 前 70 行；其余只看目录 |
| S5 | Karpathy `autoresearch` | https://github.com/karpathy/autoresearch | 检索摘要 |
| S6 | Anthropic 官方 `anthropics/skills`（`skills/` 目录 19 项） | https://github.com/anthropics/skills | 读目录 |
| S7 | 第三方目录示例：claudemarketplaces.com（W&B · MLflow）· skills.sh（`ml-hyperparameter-tuning`）· claudedirectory.org | https://claudemarketplaces.com/skills/davila7/claude-code-templates/weights-and-biases · https://www.skills.sh/josiahsiegel/claude-plugin-marketplace/ml-hyperparameter-tuning · https://claudedirectory.org/plugins/hyperparameter-tuner | 只看检索摘要，未审代码 |

## 二、同类模型训练写法

| 编号 | 名字 | 链接 | 读到什么程度 |
|---|---|---|---|
| S8 | SEtrain（GTCRN 作者的训练模板：`configs/cfg_train.yaml` · `train.py` · `dataloader.py` · `scheduler.py` · `loss_factory.py`） | https://github.com/Xiaobin-Rong/SEtrain | 读源码 |
| — | GTCRN 官方仓库（本仓副本 `resources/gtcrn/`） | https://github.com/Xiaobin-Rong/gtcrn | 读 README 与 `loss.py` |
| S9 | UL-UNAS 仓库与论文 | https://github.com/Xiaobin-Rong/ul-unas · https://arxiv.org/abs/2503.00340 | 只看名字与 HF 卡引用 |
| S10 | HF：`Arm/ulunas-speech-enhancement-fp32-litert` | https://huggingface.co/Arm/ulunas-speech-enhancement-fp32-litert | 读模型卡与文件列表 |
| — | DeepFilterNet（本仓副本 `resources/DeepFilterNet/`：`README.md` · `models/DeepFilterNet3.zip` 内 `config.ini` · `libDF/src/dataset.rs` · `libDF/src/augmentations.rs` · `DeepFilterNet/df/loss.py` · `df/train.py`） | https://github.com/Rikorose/DeepFilterNet | 读源码 |
| S11 | SpeechBrain DNS 增强 recipe：`recipes/DNS/enhancement/hparams/sepformer-dns-16k.yaml` | https://github.com/speechbrain/speechbrain/tree/develop/recipes/DNS/enhancement | 读源码 |
| S12 | HF：`speechbrain/sepformer-dns4-16k-enhancement` | https://huggingface.co/speechbrain/sepformer-dns4-16k-enhancement | 读模型卡头部与文件列表 |
| S13 | SpeechBrain Voicebank MetricGAN recipe：`recipes/Voicebank/enhance/MetricGAN/hparams/train.yaml` | https://github.com/speechbrain/speechbrain/tree/develop/recipes/Voicebank/enhance/MetricGAN | 读源码（关键字段） |
| S14 | ESPnet `egs2/dns_ins20/enh1/conf/tuning/train_enh_dccrn.yaml` | https://github.com/espnet/espnet/tree/master/egs2/dns_ins20/enh1 | 读源码 |
| S15 | ESPnet `egs2/TEMPLATE/enh1/README.md`（现场加噪说明，PR #4321） | https://github.com/espnet/espnet/blob/master/egs2/TEMPLATE/enh1/README.md · https://github.com/espnet/espnet/pull/4321 | 读相关段落 |
| S16 | ESPnet `espnet2/train/preprocessor.py`（`EnhPreprocessor`） | https://github.com/espnet/espnet/blob/master/espnet2/train/preprocessor.py | 读相关行 |
| S17 | Interspeech 2025 URGENT 挑战赛总结 | https://arxiv.org/abs/2505.23212 · https://urgent-challenge.github.io/urgent2025/baseline | 读全文（经摘要工具） |
| — | ClearerVoice-Studio（本仓副本 `resources/ClearerVoice-Studio/train/`） | https://github.com/modelscope/ClearerVoice-Studio | 读配置与目录 |
| S18 | HF Hub 搜索 `gtcrn`（13 个仓库） | https://huggingface.co/models?search=gtcrn | 读 API 列表 |
| S19 | HF：`Vasubuha/gtcrn-speech-enhancement` | https://huggingface.co/Vasubuha/gtcrn-speech-enhancement | 读模型卡头部 |
| — | HF：`AXERA-TECH/H-GTCRN.AXERA`（NPU 移植，非流式） | https://huggingface.co/AXERA-TECH/H-GTCRN.AXERA | 读模型卡头部 |

## 三、训练方法

| 编号 | 名字 | 链接 | 读到什么程度 |
|---|---|---|---|
| S20 | Nathoo, Kegler, Stamenovic. Two-Step Knowledge Distillation for Tiny Speech Enhancement (2023) | https://arxiv.org/abs/2309.08144 | 摘要 |
| S21 | Thakker 等. Fast Real-time Personalized Speech Enhancement: E3Net and Knowledge Distillation (2022) | https://arxiv.org/abs/2204.00771 | 摘要 |
| S22 | Taherian, Eskimez, Yoshioka. Breaking the trade-off in personalized speech enhancement with cross-task knowledge distillation (ICASSP 2023) | https://arxiv.org/abs/2211.02944 | 检索摘要 |
| S23 | Braun, Tashev. Data augmentation and loss normalization for deep noise suppression (SPECOM 2020) | https://arxiv.org/abs/2008.06412 | 摘要；参数范围未核 |
| S24 | Défossez 等. Real Time Speech Enhancement in the Waveform Domain (Demucs, 2020) | https://arxiv.org/abs/2006.12847 | 检索摘要 |
| S25 | Multi-Loss Convolutional Network with Time-Frequency Attention for Speech Enhancement (2023) | https://arxiv.org/abs/2306.08956 | 检索摘要（消融数字） |
| S26 | Eskimez 等. Personalized Speech Enhancement: New Models and Comprehensive Evaluation (2021) | https://arxiv.org/abs/2110.09625 | 检索摘要 |
| S27 | Fu 等. MetricGAN+ (2021) | https://arxiv.org/abs/2104.03538 | 只引名字，未读 |
| S28 | Li 等. Aligning Generative Speech Enhancement with Perceptual Feedback / Human Preferences via DPO (ICASSP 2026) | https://arxiv.org/abs/2507.09929 | 摘要 |
| S29 | Multi-Metric Preference Alignment for Generative Speech Restoration (2025) | https://arxiv.org/abs/2508.17229 | 摘要 |
| S30 | Karpathy. A Recipe for Training Neural Networks (2019) | https://karpathy.github.io/2019/04/25/recipe/ | 未在本次重读，按已知内容引用 |
| S31 | Google Research. Deep Learning Tuning Playbook | https://github.com/google-research/tuning_playbook | 读前 10 万字符（经摘要工具） |
| S32 | Serre 等. A lightweight dual-stage framework for personalized speech enhancement based on DeepFilterNet2 (2024) | https://arxiv.org/abs/2404.08022 | 摘要 |
