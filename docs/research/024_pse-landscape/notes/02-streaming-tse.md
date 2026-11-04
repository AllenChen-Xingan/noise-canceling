# 笔记 02 · 流式 / 因果 / 端侧目标说话人提取（子代理原始交回，pse 未逐条核；核过的在 sources.md 标出）

> 2026-11-04 · 读文档、论文、模型卡；未跑任何模型。

## 结论
截至 2026-11-04，纸面上同时满足「因果 · 附加延迟 ≤ 75 ms · 每 10 ms ≤ 2 ms · 许可与 GPL-3.0 兼容」的开源权重候选有 1 个：**Alkove Focus（`alkove-focus-1-preview`，Apache-2.0）**。数字全是厂商自测，无论文、无第三方复测；训练数据未公开；无负 SIR 分档数字；预览版，权重与接口可能改。VoxLock 效果更好但算力超标。其余缺开源权重、太慢、或要多麦 / 视频。

## A. 开源、有权重
1. **Alkove Focus** https://github.com/alkove-ai/alkove · https://huggingface.co/alkove-ai/alkove-focus-1-preview（原表 `bench/results/focus.json`）
   - 因果；每步 16 ms 另加 1 步延迟，合计约 32 ms；约 0.8M 参数，int8 文件 1.1 MB
   - 单线程 M1 Max 每秒音频 56 ms（≈每 10 ms 0.56 ms）；Galaxy Z Fold8 每秒 39 ms；Rust 引擎，测于 2026-10-20
   - 代码与权重 Apache-2.0；**从 GTCRN 的 DNS3 检查点（MIT）改来**；训练数据未公开
   - Alkove 自测：0 dB 同响旁人，有声纹 SI-SDR 7.14 · 无声纹 4.46 · 原始 3.37 · VoxLock 11.74；旁人单独说话残留 有声纹 −22.6 dB · VoxLock −28.6；AMI 头戴麦 WER 16.1%（原始 20.8%、VoxLock 15.1%），旁人漏词每分钟 0.5（VoxLock 0.4）
   - 已知缺陷：新人刚开口前约 150 ms 漏过（比原声低 25–35 dB）；只适合近距离麦
   - **会中自动学声纹：约 6 s 用户语音，在下一次静音时生效**
2. **VoxLock**：见 005；新增 Pipecat 实测端到端 104–184 ms；无 CoreML
3. **REAL-TSE 官方因果基线**（2607.15198）：窗 32 / 跳 8 ms，算法延迟 24 ms；主办方测 EVAL-2 TER 0.811 / 0.808、SpkSim 0.370 / 0.391；修正后排行榜 TER 0.712–0.734；仓库许可未写、权重 Google Drive 许可未证实
4. **WeSep 新版**（2607.27436）：因果 BSRNN 理论 32 ms；预训练「coming soon」；Libri2Mix-100 因果 SI-SDRi：说话人嵌入 6.87 · USEF 13.05 · TF-Map+上下文 12.98 · USEF+上下文 14.15；「准确率」（SI-SDRi > 1 dB 占比）嵌入法 79.9%、其余约 95–96% →【推断】嵌入法约 20% 样本失败或提错人
5. **AFW**（2609.20463，Mesgarani 组）：每块 250 ms；2.9M 因果 TCN + 41k 记忆；RTF CPU 0.235（机型未报）；代码 github.com/ym2976/anchor-fast-weight 许可未写；旁人比用户响 10 / 15 dB 时 SI-SNRi 13.6 / 14.0；提错人 4%；用户 30 s 不说话后 6.2 dB；仿真
6. **CARTSE**（REAL-TSE 在线第 1，CyberAgent）https://real-tse.github.io/assets/pdf/CARTSE-Track1.pdf：因果 BSRNN，算法 24 ms、实测有效前瞻 22.9 ms；未开源；仿真预训练 → 真实会议 + 非因果教师伪标签微调；**训练时 35% 概率故意给错误声纹、加入用户缺席片段**；TER 0.592、SpkSim 0.503

## B. 无开源权重、数字可参考
7. TargetVoice（Interspeech 2025）：流式 20 / 10 ms；提取模块 12 MB、6 GMACs；SIR −10~20 dB 训练 + 真实微调；旁人响 0–10 dB 时 SI-SDR 提升 8–10 dB（无表）
8. SpeakerBeam-SS（2407.01857；ICASSP 2026 实时演示）：因果 20 ms；7.93M；单核 EPYC 7502P C++ RTF 0.36；SDR 11.58（非因果 13.63、pDCCRN 10.70、E3Net 9.83）；SIR −5~5
9. 正负注册对比 TSE（2502.16611）：因果 TF-GridNet 1.88M；Xeon 1 s 音频 0.365 s；2 人 SI-SNRi 9.90、6 人 3.22
10. 双模训练实时 TSE（CyberAgent，APSIPA 2025）：7 麦阵列，不适用
11. 自声消除 OVC（2606.23332）：因果 2 ms；Mamba-MinGRU 小版 2.17M、0.18 GMAC/s；i7-13700 单线程每块 1 ms RTF 0.82；SDR 约 11.2–11.5（架构参考）
12. clarity-tse（个人原型）：多麦助听；约 2M、252 MMAC/s；M5 Pro 每 8 ms 一跳中位 1.66 ms；SI-SDRi 3.43
13. StarTSE / DFAPO（2604.19635、2607.10191）：生成式，分块 560 ms，延迟远超
14. EvoTSE：见笔记 03；代码 github.com/IiuZiKai/Evo_TSE 许可未写
15. Falcon（2610.32749）：音视频，排除
16. 主动助听（2511.11473）：以佩戴者声音为锚的思路
17. 其他：DSINet（无数字）· D-LGTSE · TripleC · MeanFlow-TSE · CHiME-9 ECHI（多通道眼镜）· ClearerVoice 2025 起无新因果 TSE

## 负 SIR 数字
没有真实录音上按 SIR 分档的因果小模型结果。TargetVoice 0–10 dB 提升 8–10（无表）· AFW 10 / 15 dB 13.6 / 14.0（仿真）· SpeakerBeam-SS 只报 −5~5 平均 · Alkove 只测 0 dB

## 提错人：发生率与解法
- 发生率：WeSep 因果嵌入法约 20%（SI-SDRi ≤ 1 dB）、USEF 约 4% · AFW 4% · EvoTSE 跨域 23.9→8.1%
- 解法：训练时故意给错误声纹 + 加用户缺席片段（CARTSE 等）· 说话人增广（2505.21805）· 声纹一致性损失（2507.09510）· 高置信历史输出迭代更新注册（EvoTSE、AFW）· 负样本注册（2502.16611）· 说话人对比损失（Falcon）· 新声纹静音时才生效、刚开口 150 ms 允许漏（Alkove 工程做法）

## 检索记录
截至 2026-11-04。关键词：causal / streaming / real-time / on-device / lightweight TSE 2025/2026 · speaker / target confusion · negative / low SIR · Look Once to Hear 后续 · ICASSP 2026 · CHiME-9 · REAL-TSE · personalized SE interfering speaker · GitHub / HF 因果检查点 · 中文「目标说话人提取 实时 流式 因果」（几乎无直接结果）。站点：arXiv · ISCA Archive · APSIPA 2025 · NTT ICASSP 2026 页 · SSRN（403）· real-tse.github.io 与 3 份参赛报告 · GitHub · HF。未证实：DSINet 数字 · DeepSound 结构 · REAL-TSE 基线权重许可 · Alkove 训练数据。
