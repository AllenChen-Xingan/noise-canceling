# 笔记 05 · ≤100 万参数的因果 PSE / TSE / 个性化 VAD（子代理原始交回，pse 未逐条核；核过的在 sources.md 标出）

> 2026-11-04 · 只读论文页面，未复现。

## 结论
- **TSE**：≤100 万最好的是 **TF-MLPNet（约 50.9 万参数）**，LibriSpeech 两人混合 SI-SDR 12.37 dB，高于 321.8 万参数的 pDCCRN（10.71 dB）
- **PSE**：唯一同时给 DNS 盲测主观分和单线程耗时的是 **PVQE-S（107 万，略超线）**
- **个性化 VAD**：Personal VAD ET 版 13 万参数
- **GTCRN 量级（约 5 万参数）上查不到任何带公开数字的个性化模型**
- 估计（推断）：GTCRN 内部加 FiLM 多约 0.5–2 万参数、计算量 +<1–10%；声纹提取器另算，15 万（TinyECAPA）到 620 万（ECAPA），只在注册时跑

## 逐条
1. **TF-MLPNet（TSE 版）** https://arxiv.org/html/2508.03047v1
   - 50.9 万（盲分离版 49.3 万）；小版 +2F：TSE 23.1 万、盲分离 21.5 万；INT8 约 483 kB
   - GAP9 370 MHz：每 6 ms 块 3.6 ms，55–80 mW；未报 MACs
   - 因果，算法延迟 10 ms；编码器后一层 FiLM，d-vector
   - LibriSpeech test-clean SI-SDR（非 SI-SDRi）：12.37 · +2F 11.77 · TFG-LN+2F（21.3 万）12.22 但 8.9 ms · pDCCRN（321.8 万）10.71 · TinyDenoiser（105.6 万）8.15
   - 无代码链接，许可未知
2. **PVQE-S** https://arxiv.org/html/2406.09928v1
   - 107 万；i7-10700K 单线程每帧 0.135 ms；窗 20 / 帧移 10 ms、无前瞻
   - 声纹 = 模型最后一层 GRU 后表示在注册音频上取平均，无需另配提取器；线性映到 240 维与编码器特征拼接
   - DNS5 盲测个性化 P.835：内部声纹 2.96 · 外接声纹 2.88 · E3Net 2.87 · 带噪 2.06；无代码
3. **pDCCRN 学生（蒸馏）** https://ar5iv.labs.arxiv.org/html/2204.00771
   - 150 万，RTF 0.094（Xeon W-2133 单线程）；教师 396 万 RTF 0.195；窗 32 / 帧移 16 ms
   - VCTK TS1（WER / DNSMOS / TSOS 秒）：学生 MTL+KD 37.20 / 3.37 / 0.60；教师 38.24 / 3.36 / 14.61
4. **Sivaraman & Kim 个性化 ConvTasNet** https://ar5iv.labs.arxiv.org/html/2104.02017（代码 github.com/IU-SAIGE/pse，许可未查）
   - SDR 通用 → 最佳零样本个性化：13.9 万 8.92→9.91（+1.0）· 22.4 万 9.77→9.94 · 43.8 万 10.59→10.89 · 103.7 万 11.23→11.06
   - 同架构不同规模下「通用 vs 个性化」唯一对照：**越小个性化收益越大**；靠自监督微调，不靠声纹条件

### 超 100 万、作参照
5. pDeepFilterNet2：231 万、0.33 GMACs（与 DFN2 同，拼接声纹几乎不增开销）；双编码器 271 万 0.40 G；ECAPA 192 维；40 ms 前瞻；无代码 https://arxiv.org/html/2404.08022v1
6. 续作 2601.16235：加 15 万参数 TinyECAPA 在线修正声纹，223 万 0.33 G → 238 万 0.374 G（+44 MMACs/s，比 GTCRN 全模型还大）；DNS5 盲测 PDNSMOS OVRL 耳机 3.078→3.126、外放 2.777→2.829；arXiv 写「ICASSP 2025」但提交 2026-01，待核
7. VoiceFilter-Lite：3 层 256 单向 LSTM，INT8 2.2 MB（推断约 220 万）；加性人声干扰 WER 56.5→31.4% https://ar5iv.labs.arxiv.org/html/2009.04323
8. Personalized PercepNet 850 万、i7-8565U 单核 4.7% · SpeakerBeam-SS 793 万 RTF 0.36 延迟 20 ms SDR 11.58 · TargetVoice（Interspeech 2025）提取块 12 MB 6 G 乘加 延迟 10 ms SI-SDRi 8–10 dB
9. DNS5 个性化：NPU-Elevoc 1249 万（ResNet34 声纹 663 万）8.5 G/s RTF 0.48 · TEA-PSE 3.0 2224 万 19.66 G/s RTF 0.46 · E3Net 基线 662 万 11.08 G；总结论文没列各队复杂度

### 个性化 VAD
10. Personal VAD ET：13 万，2 层 64 LSTM，输入拼 256 维 d-vector；AP 0.932，加权损失 0.955；声纹模型本身 488 万 https://ar5iv.labs.arxiv.org/html/1908.04284
11. Personal VAD 2.0：Conformer，INT8 1.0 MB，9.58 MFLOPs；**FiLM 优于拼接**（WER 11.3 vs 15.3）https://ar5iv.labs.arxiv.org/html/2204.03793
12. HyWA（2510.12947）：8.5 万参数小 PVAD，mAP 拼接 89.7 · FiLM（LSTM 前）89.7 · 超网络 91.6；FiLM 放 LSTM 前比后高 1.5 点
13. In-Ear-Voice（2309.02393）：约 5000 参数，骨导麦个性化，不用声纹，2.64 mW

### 声纹提取器大小
ECAPA C=1024 1465 万（CAM++ 论文）；C=512 约 620 万（本轮未核）· CAM++ 718 万 1.73 GFLOPs · ResNet34 663 万 · PVAD d-vector LSTM 488 万 · ECAPA-TDNN-Lite 31.8 万 EER 2.31% · TinyECAPA 15 万 EER 4.26% · PVQE 内部表示 0

## 估计推理链（推断）
1. TF-MLPNet 加一层 FiLM 两种规模都多约 1.6 万参数。GTCRN 瓶颈约 16 通道，每通道 γ/β 从 192 维映射约 2×16×192 ≈ 6 千；γ/β 注册时算一次，每帧多逐元素乘加约 0.1 MMACs/s → 合计约 5.4–7 万参数，计算 +<1–5%
2. 若拼进 GRU 输入：192 维直拼每频带约 3×16×192×33×100 ≈ 30 MMACs/s（计算翻倍）；先投影到 8–16 维再拼约 +5–10%。PVAD 2.0 与 HyWA 显示 FiLM 不差于拼接
3. 声纹提取器只在注册时跑；若学 2601.16235 在线修正，每秒多约 44 MMACs（> GTCRN 全模型 33 MMACs/s）；PVQE 式内部声纹零成本但要重训改结构
4. 效果：Sivaraman 13.9 万时个性化 +1.0 dB、100 万时 ≈0；TF-MLPNet 23 万仍 11.8 dB → FiLM 在亚百万有效；**5 万量级尚无判据，待测**

## 没找到
pGTCRN / 个性化 GTCRN（GTCRN 只在 D-LGTSE 里当降噪前端）· ULCNet / FSPEN / LiSenNet / FastEnhancer 个性化版 · 5 万量级公开个性化结果 · 亚百万参数、Libri2Mix 报 SI-SDRi 的因果 TSE · 多数许可未核

## 检索记录
截至 2026-11-04。关键词：PVQE · pDCCRN · E3Net · Personal VAD 1/2 · VoiceFilter-Lite · pDeepFilterNet2 · TinyECAPA · personalized/speaker-conditioned GTCRN · ULCNet · FSPEN · LiSenNet · TinyML PSE · TF-MLPNet · SpeakerBeam-SS · TargetVoice · LGTSE · HyWA · DNS4/5 personalized · FiLM vs concat · CAM++ · ECAPA-Lite；中文「轻量 个性化语音增强」「GTCRN 个性化 声纹」。站点 arxiv · ar5iv · isca-archive · research.google · microsoft.com。
