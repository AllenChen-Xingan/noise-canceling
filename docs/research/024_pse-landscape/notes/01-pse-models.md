# 笔记 01 · 2025 以后的 PSE / TSE 新模型（子代理原始交回，pse 未逐条核；核过的在 sources.md 标出）

> 2026-11-04 · 只读论文与模型卡，未跑任何模型。所有性能数字作者自测，除 REAL-TSE 挑战赛排名（组织方统一打分）。

## 结论
截至 2026-11-04，**没找到** 2025 以后同时满足「因果 · 开源权重 · 许可与 GPL-3.0 兼容 · 单线程每 10 ms ≤ 2 ms · 真实录音第三方证据」的 PSE / TSE。

最接近的新候选：**Scicom-intl/voice-isolator-cpu**（arXiv 2610.35272《The Voice on the Line》）
- 满足：因果；**会中自动注册**；ONNX 权重；真实会议录音评测（作者自测）
- 不满足：2210 万参数；作者自测 10 ms 帧占单核 54%（fp）/ 34%（int8），约 5.4 / 3.4 ms 每 10 ms（机器未写）；权重许可「other」无条款；训练数据许可不全
- 可借鉴：自动注册规则

## 候选（已知清单以外）
1. **voice-isolator / voice-isolator-cpu**（Husein Zolkepli，Scicom，2026-10-26）https://arxiv.org/html/2610.35272v1 · https://huggingface.co/Scicom-intl/voice-isolator-cpu
   - 因果零前瞻；论文 20 ms 帧 / 10 ms 跳、延迟 20 ms；CPU 版卡写 10 ms 帧、落后 10 ms
   - 大版 5.55 亿（含因果化 WavLM-Large），约 47.7 GMAC/s；CPU 学生 C22 2210 万（8 层宽 384 Transformer、回看 10 s、帧 LSTM + 子带 LSTM），注册编码器 87 万
   - 耗时（作者自测，CPU 未写）：10 ms 帧 54% / 34%；20 ms 帧 33% / 23%；**同机 GTCRN 约 5%，VoxLock 65–264%**
   - 许可「other」；LiveKit 插件
   - 训练：2761 h 干净语音（Common Voice、MLS 等 131 语）；噪声 MUSAN、FSD50K、AudioSet、DEMAND、ESC-50；真实串音 AMI、NOTSOFAR、AliMeeting 620 h；NOTSOFAR CC BY 4.0；ai-coustics 通话 CC BY-NC 只评测
   - 真实录音 WER（作者自测）：AMI 60 s 窗 16.13%（VoxLock 23.66%）；NOTSOFAR 7.67%（8.84%）；Krisp 基准打平；只有来电者一人 5.58%（VoxLock 2.92%，更差）
   - **自动注册**：开始空声纹；第 3 s 起取「到目前为止最响的 3 s」，15 s 后固定；作者承认旁人更响会锁错；可选「语音机器人模式」用 LibriTTS-R 诱饵声
2. **tse-conv-tasnet-48k**（个人，penta2himajin）：模型卡称因果流式、零前瞻；145 万参数（二手）；ONNX 约 0.6 + 5.3 MB；许可两说未证实；要外部 192 维 ECAPA；页面现 401
3. **REAL-TSE 挑战赛（SLT 2026）在线赛道** https://real-tse.github.io/challenge · arXiv 2607.15198
   - 规则：算法延迟 ≤ 100 ms、允许有限前瞻、不限算力；基线因果 BSRNN（ECAPA 条件 / TF-Map 条件），窗 32 / 跳 8 ms，算法延迟 24 ms
   - 代码 github.com/REAL-TSE/wesep-real-tse 无 LICENSE；因果权重邮件发报名队，是否公开未证实
   - 评测集 REAL-T（AISHELL-4、AliMeeting、AMI、DipCo、CHiME6 真实会议），许可未写；组织方统一打分，12 队
   - 发现：只用合成训练不如真实适配，前 3 名都用了真实录音；因果基线在 14 个系统里排第 11、14；在线第一 CARTSE 细节未查到
   - 都要单独注册语音
4. **SonicAGI SwiftNet-Lookahead**（2607.11083）：有界前瞻 + 因果迭代分离，总延迟 96 ms（超 75 ms）；无发布；在线赛道约 2–3 名
5. **WeSep 新版**（2607.27436，Interspeech 2026）：因果 BSRNN 理论延迟 32 ms、可导 ONNX（搜索摘要）；新版「将公开」
6. **TargetVoice**（Meeami，Interspeech 2025 展示）：20 ms 窗 10 ms 跳，自称延迟 10 ms；抽取块约 12 MB、6 GMAC；声纹编码器约 10 MB；专有；需 ≥ 3 s 注册
7. **Contrastive KD 声纹在线细化**（Serre 等，2601.16235）：15 万参数小编码器在线细化声纹、跟踪变化（见笔记 05）
8. **TripleC + LGTSE**（2512.04945）：D-LGTSE 续作；D-LGTSE 613 万、8.53 GMAC/s、8 kHz Libri2Mix 合成
9. **AV-PVQE**（Rashed、Filipi、Cutler，2609.30631，2026-09-24）：PVQE 加嘴部视觉；因果、算法延迟 20 ms；**起点的纯音频个性化模型即使有干净注册，双人混合中 46% 认错目标；加视觉后 1.6%**（作者自测）
10. **PAS-SE**（2509.20875，ICASSP 2026）：耳机，注册 + 耳内辅助麦；Mac 单麦不适用
11. **《Don't Listen to Me》**（2606.23332，Interspeech 2026）：反任务（去掉已注册者）；Mamba-MinGRU 时域，算法延迟 2 ms；结构可参考
12. **EvoTSE**（2604.06810）：见笔记 03；WSJ0-2mix（LDC）8 kHz；作者承认增加延迟
13. **Enroll-on-Wakeup**（2602.15519）· **Unmixing the Crowd**（2604.03219）：见笔记 03
14. **Proactive Hearing Assistant**（UW Gollakota 组，2511.11473，EMNLP 2025）：以佩戴者语音为锚自动推断对话对象、无注册；流式每 12.5 ms + 慢模型；双耳输入；作者实录 11 人 6.8 h
15. 其他：NTT SpeakerBeam-SS 实时演示（ICASSP 2026，笔记本 CPU，约 10 s 注册，无代码）· SSRN 7478491 轻量因果 TSE（403，未证实）· TTT for SE（2508.01847）· 真实会话多说话人提取（2609.25948，注册与目标不匹配、静音段影响）

## 已知清单补充
- VoxLock：首个第三方对比（voice-isolator 论文）：AMI / NOTSOFAR / ai-coustics 上被超过，来电者独白更好；同机占单核 65–264%
- PVQE 2406.09928 是 Interspeech 2024
- URGENT 2026 无个性化赛道（2601.13531）；DNS 2023 后无个性化后续

## 检索范围
截至 2026-11-04。关键词：personalized speech enhancement 2025/2026 · speaker-conditioned SE · personalized noise suppression · pDNS · TSE causal/streaming/real-time · enrollment-free / self-enrollment / online enrollment · Interspeech 2025 PSE · ICASSP 2025/2026 PSE/TSE · URGENT personalized · REAL-TSE · GTCRN personalized · Mamba/SSM TSE · hearables personalized · 中文「个性化语音增强」「目标说话人」。站点：arXiv · ISCA Archive · ICASSP 2026 程序页 · SLT 2026 / REAL-TSE 官网 · HF · GitHub · SSRN（被拦）· PubMed（被拦）。局限：没逐篇翻 Interspeech 2025 与 ICASSP 2026 全部录用列表。
