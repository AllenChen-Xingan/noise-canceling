# E-distill · 怀疑派核对

> 2026-10-31 · Workflow wf_113bb80a-4ec · 只读调研，未运行程序。

## 核对表

- **VoxLock 27M、因果、80 ms 块 int8 单核占 61%、L4 上比实时快 40 倍** → ✅【源码 voxlock/README.md:111、141-145】。⚠️ 有两处要补：
  - 它是**目标说话人提取**模型，要先录 3 s 注册语音【同:111】，不是通用降噪老师。
  - 权重是从 REAL-TSE 基线微调来的，说话人嵌入用 WeSpeaker ECAPA【同:190-191；NOTICE】。这两份上游的**训练数据许可没有核**，按 D26 要查清。
- **DNS 解包约 1 TB、URGENT25 Track1 要 1.3 TB、DNS5 语音 225 GB、本机剩 177 GiB** → ✅【DNS-Challenge/README.md:91-92；urgent2025_challenge/README.md:61、68；`df`】。
- **arXiv 2309.08144** → ✅ Nathoo、Kegler、Stamenovic 2023；−5 dB、63 倍压缩、SDR +0.9 / +1.1 dB。⚠️ 「学生 62k，是老师的 3.3%」摘要里没有，未能核实。
- **E3Net arXiv 2204.00771** → ✅ 学生快 2–4 倍；可用没有干净参考的噪声数据训练；加了 ASR 损失。漏写了一点：它是**个性化增强**（要说话人注册）。
- **CNNC-Distill** → ✅ Park 等，IEEE SPL 31:1129–1133，DOI 10.1109/LSP.2024.3388956。❌ 要改两处：
  - 学生 RT-SENet 是从 MANNER 改的，「老师是 MANNER-B」没有核到。
  - 只在 VoiceBank（Valentini）上评测；而且是**越大的学生蒸馏收益越大**【二手 韩国大学实验室摘要】，和结论 2「收益集中在极小学生」的方向相反。
- **arXiv 2312.08979 的表** → ✅ 数字都对（表 1：原始输入 OVR 2.84 / BAK 2.92 / SIG 3.48；RemixIT-VAD 2.84 / 3.62 / 3.28）。⚠️ 注意这篇是 Close 等人的 Multi-CMGAN+/+，不是 RemixIT 原文。
- **「DNSMOS 当判别器做到 OVR 3.51」** → ✅ 数字属实【ISCA CHiME 2023 close23】。❌ **由此推出的结论反了**：UDASE 的听测论文【arXiv 2402.01413 v2】写明，CMGAN 系统 DNSMOS 最高，但**主观听测最差**，原文是 "learned to maximize the metrics while providing … unsatisfactory quality"。四个参评系统的 SIG **全部显著低于原始输入**。
- **arXiv 2309.02432** → ✅ 作者 Xu 等（Fingscheidt 组）。方法是 PESQ-DNN 无参考损失：合成测试集 PESQ +0.32，真实集 DNSMOS +0.05。增益很小。
- **GTCRN 48.2K / 33M** → ✅【gtcrn/README.md:14、19】。但 FastEnhancer 表里 GTCRN 一行是 24K / 40M【fastenhancer/README.md 表 1】，笔记表格把两处来源混在一行。
- **FastEnhancer 的 RTF 与 PESQ（T / B / M、LiSenNet 流式版）** → ✅ 与表 1 逐项一致。⚠️ 两点：
  - 这是**只用 VoiceBank 训练**的成绩，对咖啡馆低信噪比 babble 场景证据很弱。同一 README 里有 DNS 表（第 260 行起），应改用那张。
  - 「LiSenNet 官方无权重」→ ❌ FastEnhancer 的 release 里有流式 LiSenNet 的 ONNX（只用 VoiceBank 训练）【006 sources.md:22】。
- **UL-UNAS 171K / 35M，表 VI** → ✅【arXiv 2503.00340 表 VI】。
- **huge_noncausal.yaml:1、b.yaml、4 卡训练** → ✅（4 卡出处是 fastenhancer.md:27-29，笔记写的 :29 大致对）。
- **每 epoch 约 9.1 h、共约 4550 h、约 3 PFLOP** → ✅ 算术正确。
- **RunPod 价格** → ✅（今日抓取官方页）：L4 0.44 / 0.59；A100 80G 1.19–1.79；H100 2.69 / 3.99。
- **AWS 价格「2026-10-31 更新」** → ❌ 晚于今天（10-30），日期不可信，需重抓。
- **Lambda 价格** → ⚠️ 官方页这次也没打开。
- **MUSAN、AISHELL-1、DEMAND、VoxCeleb2 的许可** → ✅【DNS-Challenge/README.md:271-285】。
- **「MLS / CommonVoice（CC0）」** → ❌ 要拆开：MLS 是 CC BY 4.0，只有 CommonVoice 是 CC0【二手】。
- **VCTK 许可** → DNS 写 ODC-By【DNS-Challenge/README.md:279-280】。VCTK 0.92 在 Edinburgh DataShare 上的许可为 CC BY 4.0【二手，未打开】。
- **URGENT26 去掉 WSJ、排除 ESD** → ✅【urgent2026_challenge_track1/readme.md:33、69】。
- **`~/vdn-data` 目录与大小** → ✅ 本机 `du` 一致。

## 逻辑问题

1. **结论 4 自相矛盾**：总价写 150–600 美元，但同一节表里光训老师就是 240–840 美元。「4×A100 跑 2–3 天」本身也没有出处。
2. **「伪标签 + 不可微奖励比只用伪标签强」证据不成立**：DNSMOS 分高恰恰是**奖励被钻了空子**（reward hacking）。这正是 D43「评测当奖励」的头号风险。奖励只能用多尺互相牵制（WER / 吃字率 + SIG + 人工听测抽检），不能单用 DNSMOS。
3. **VoxLock 当老师有三处不对位**：
   - 它是因果模型，算不上「大的非因果老师」。
   - 它是 TSE，要注册语音。
   - 它训练时用的是耳麦录音，用户场景是笔记本内置麦（D42），本身就有 sim-to-real 差距。
4. **非因果老师 → 因果学生**会让学生去学「没有未来帧就做不到」的目标，词首最容易被压，正撞「从不吃字」这条一票否决。笔记没提，需要实验验证。
5. **「几百小时真实录音本机一两天跑完」**：算力推算没问题（约 180 核·时，M5 多核约一天）。但**录音从哪来、许可如何**完全没说。用户录音受「无云端」与隐私约束，不能拿来训练。CHiME-5 的许可也需另查（未能核实）。
6. **「同延迟档、算力 GTCRN 量级」说得太宽**：FastEnhancer-B 的 MAC 是 GTCRN 的约 7–8 倍，M 是约 88 倍。M 档单线程 RTF 0.039，还没算 STFT，已经贴着 4% 单核上限。

## 遗漏

- **训练奖励里没有 WER**。FastEnhancer 表 1 有 WER 列：GTCRN 3.6、B 3.2、M 2.8，可以直接拿来当吃字代理。
- **FastEnhancer DNS 权重的数据许可没逐项拆**。它用的 DNS3 里含 VoxCeleb2、DEMAND（BY-SA），只有拆清楚才能判断能不能拿来当老师或起点。
- **MPS 训练 RNN 的问题**（pytorch #83144）没有核原文。
- **策略搜索的打分耗时**：「Whisper 10–20 h」要求 RTF ≈ 0.03–0.05，用哪个模型、哪个后端都没写。
- **存储方案没给**：外置盘、NAS 都要人拍板。

## 修正后的结论

1. 本机能做：GTCRN 量级学生训练（MPS 可能不比 CPU 快，要先做一次有界冒烟实验）、bandit 策略搜索、VoxLock 伪标签。云上：只有训练非因果大老师需要，成本**未能可靠估算**，待人拍板。
2. 本机先卡在**硬盘**（177 GiB，对比 DNS 解包约 1 TB）和**真实录音的来源与许可**上，不在算力。
3. 蒸馏收益的证据零散，而且都在 VoiceBank 或模拟数据上：两步蒸馏在 −5 dB 有约 1 dB SDR 增益；CNNC 显示学生越大收益越大。到咖啡馆 babble、笔记本内置麦上能剩多少，需要我们自己做实验。
4. **头号风险是钻奖励的空子**：伪标签自训练和 DNSMOS 奖励在真实录音上都会伤 SIG，CMGAN 是「分数最高、听感最差」【arXiv 2402.01413】。D43 的奖励必须有吃字 / WER 和 SIG 两条硬底线，外加人工听测。
5. **许可要另核**：VoxLock 上游（REAL-TSE 基线、ECAPA）的训练数据许可、DEMAND 的 BY-SA 对权重是否生效、MLS 改记为 CC BY 4.0。