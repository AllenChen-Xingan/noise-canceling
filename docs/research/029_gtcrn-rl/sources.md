# 调研 029 · 来源登记

> 文献部分由手下 rl 派的只读调研子代理（2026-11-05）检索整理，rl 抽核了编号与结论方向；「未查到」照实写。表号 / 节号以子代理读到的原文为准，未逐一复核的标「二手」。

## 一、用不可微指标当奖励的做法

| 做法 | 出处 | 模型 | 相对基线的收益 | 已报告的失败方式 |
|---|---|---|---|---|
| 策略梯度：离散二值掩码当动作，奖励 = 识别错字率之差 | Shen 2018，arXiv 1811.04224 | DNN，参数量未查到 | 0 / 5 dB 下错字率相对降约 12–19%（二手） | 未查到 |
| PPO：掩码加高斯噪声（σ = 0.05）当随机策略，加 KL 与 SI-SNR 锚定，不用评论家网络 | arXiv 2603.13952 §3 表 1 | 视听模型，参数量未写 | PESQ 1.20 → 1.25，STOI 0.48 → 0.58；听测比 DNSMOS 奖励版多胜 67.6% | 未做过优化检验，未报方差 / 种子 |
| MetricGAN+（学一个可微的指标代理） | arXiv 2104.03538 表 1 | BLSTM | PESQ 3.15，比 MetricGAN 高约 0.3 | 只报被优化的 PESQ；PESQ 可被刷见 arXiv 2406.03460 |
| MetricGAN-U（只用带噪语音，奖励 DNSMOS） | arXiv 2110.05866 表 1–2 | BLSTM | 相对带噪输入 PESQ +0.16，DNSMOS +0.103 | PESQ 与 DNSMOS 不总是正相关 |
| MetricGAN 原文：策略梯度 vs 判别器 | arXiv 1905.04874 表 1 | — | 只比监督高约 0.07–0.1 PESQ；策略梯度那行 STOI 低于监督 | 两行起点不同（013 已登记） |
| GRPO，流匹配生成式增强 | FlowSE-GRPO，arXiv 2601.16483 表 2 | 46.18M，LoRA 训 1.57M | DNSMOS OVRL 3.373 → 3.549 | 只用 DNSMOS 一项：分涨、音质降；改多指标按标准差归一加权 |
| GSPO，语言模型式增强 | arXiv 2606.21458 表 1–3 | 未写 | OVRL +0.05 到 +1.40 | 21 人听测：只用 DNSMOS 奖励 Elo 1335 < 监督 1476；三项合成奖励胜率 71% |
| DPO，偏好对按 UTMOS 挑 | arXiv 2507.09929 表 1–2 | 12 层 1024 维 | UTMOS 2.03 → 3.18（有混响） | 说话人相似度 0.691 → 0.651 |
| TTS 上 GRPO 与 DPO 对比 | arXiv 2509.18531 | TTS | 字错率降 | 韵律被压平 |
| CHiME-7 UDASE | arXiv 2402.01413 §5.4 §6.2.5 | — | — | 拿 DNSMOS 训练：DNSMOS 最高、听测最差（013 已登记） |
| PESQetarian | arXiv 2406.03460 | — | — | 只改首个采样点 PESQ 就到 3.46（013 已登记） |
| 奖励过优化 | Gao 2022，arXiv 2210.10760 §3.6 | — | — | KL 惩罚只决定走多远，挡不住讨好（013 已登记） |

## 二、小于 1M 参数的因果实时降噪模型上做强化学习 / 偏好优化

**未查到。** 最接近的两条：
- arXiv 2603.13952 的「掩码 + 高斯噪声当策略」，结构上能套到 GTCRN 这类掩码模型；
- GTCRN 已有的后续工作只有监督联合微调（arXiv 2508.19583）和结构搜索（arXiv 2503.00340），都不是强化学习。

## 三、可微替代损失的已知问题

| 出处 | 结论 |
|---|---|
| Fu 2018，arXiv 1709.03658 §IV-B 表 II 图 9 图 12 | 只用 STOI 损失：STOI 升、PESQ 降；STOI 不管静音段和高频，高频被整段丢，识别错字率比 MSE 还差。STOI + MSE 最好 |
| Kolbæk 2018，arXiv 1802.00604 §4–5 表 2–6 | 近似 STOI 损失 ≈ 短时包络 MSE；相对 MSE 的可懂度收益「至多中等」；只有客观指标 |
| Kolbæk 2020，arXiv 1909.01019 表 III–IV | 6 种损失横比：按谁训谁最高；PMSQE（PESQ 近似）没拿到最高 PESQ，SI-SDR 损失反而最高 |
| Iwamoto 2022，arXiv 2201.06685 | 伤识别的主因是增强伪影，不是残留噪声；按比例加回带噪原声，识别变好（和本项目引擎混回同向） |

## 四、样本效率（都在大模型上；小模型未查到）

- FlowSE-GRPO：每个输入采 10 个；只用 DNSMOS 约 1.5k 步收敛，多指标 5k 步
- GSPO：每组 4 个，有效批量 8，3000 步；单 GPU 10–22 小时
- DPO：400 步，批量 128，每个输入 32 个候选
- 掩码式 PPO：σ = 0.05，KL 系数 1e-4
- 策略梯度方差的定量数字：未查到

## 五、仓内来源

| 用到什么 | 在哪 |
|---|---|
| 调研 013 全部结论与待定题 | `docs/research/013_rl-environment/decision.md` |
| verifier 定标级别（全部感知类 L0） | `src/eval/suite/levels.json`（2026-11-03） |
| verifier 定义与已知失效 | `src/eval/src/verifiers.rs` 的 `REGISTRY` |
| GTCRN 成绩单逐段读数（含原声直通、参考降噪器） | `docs/experiments/exp014-scorecard-gtcrn/runs/20261105-005916-4ccf0638/clips.jsonl` |
| DFN3 低延迟版 · DPDFNet-2 逐段读数、弱项 / 守住项 | `docs/experiments/exp016-candidates/runs/`（cands 分支，合并后进 main） |
| 决定 D44 · D57 · D81 · D85 · D86 | `docs/plan/001_decisions-20261029-b1.md` |
| GTCRN 单步训练耗时（MPS 批量 32 一步 0.821 s，每墙钟小时约 156 小时音频） | `docs/experiments/exp018-gtcrn-finetune/runs/20261105-145149-step/`（finetune 分支） |
| 识别单次 2.6–4.4 s；成绩单带识别一个系统约 77 分钟 | 项目记忆「成绩单估时」；大总管转述 DFN3 那组实测 |
| 本线实测 | `docs/experiments/exp020-gtcrn-rl/`（计时 · 离线排序模拟） |
