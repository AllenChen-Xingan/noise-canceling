# ICASSP 2023 Speech Signal Improvement Challenge 真实测试集（SIG 2023）

- **名称**：ICASSP 2023 Speech Signal Improvement Challenge test / blind set（SIG 2023）
- **链接**：https://github.com/microsoft/SIG-Challenge/tree/main/ICASSP2023
- **许可**：README 原句「The datasets are provided under the original terms that Microsoft received such datasets」，测试集本身没写许可名；仓库 LICENSE 为 MIT（GitHub API），是否覆盖音频未写 → 结论：未查到【官方文档】
- **大小**：测试集 500 条、盲测集 500 条【论文 §V】【论文 2309.07385】；`ICASSP2023/test_data` 500 个 wav 共约 414 MB（GitHub API 文件大小合计），约 1.2 h【推断：按 48 kHz、16 位、单声道折算】；盲测集大小未查到
- **场景键**：other（通话发送端真实录音；环境逐条不同，未分类）
- **设备键**：non-laptop-unknown（论文只写「PCs and mobile devices」，每条设备不同）
- **笔记本段**：可能有，但不能筛。论文写 500 条「each using a unique device, environment, and person speaking」，来自 PC 与移动设备；PC 里有没有笔记本、有没有逐条设备标签，未查到【论文 §V】
- **干净参考**：没有（真实录音，无干净轨）【论文】
- **转写**：部分（2024 版 README：「we release the transcripts for the test set」，供算 WER；`test_data` 目录里未见转写文件，位置未查到；盲测集转写未查到）【官方文档】
- **语种**：英语约 80%，另有德、荷、法、西【论文 §V】
- **语体**：未查到（通话发送端说话；有无笑声 / 附和未标）
- **获取方式**：直接下载（GitHub 仓库 `ICASSP2023/test_data`、`blind_data`）
- **与 D62 的关系**：真实录音（真人对 PC / 手机说话，非回放）。按 P.804 维度（信号、噪声感、音色、断续、响度、混响）分层抽样。人评分：挑战用扩展版 P.804 众包评 9 队 × 500 条，但逐条人评分**未见公开**。若开发阶段拿来评测：用测试集 500 条 + 转写做「识别吃字检查」与无参考分的真实旁证；约 414 MB
- **出处**：https://arxiv.org/abs/2303.06566 §V、Table 13、Figure 1【论文】; https://arxiv.org/abs/2309.07385【论文】; https://github.com/microsoft/SIG-Challenge【官方文档】

## 备注
- 本路原稿漏了这份。它给出了**真实录音上无参考分对人评分的独立数字**（Table 13，9 队 × 500 条盲测）：DNSMOS P.835 OVRL 对 P.804 整体，单条 PCC 0.695、按队 0.884；SIG 对信号 0.656 / 0.799；NISQA 整体 0.681 / 0.766；NISQA 断续 0.478 / 0.310。论文原话：这说明仍需主观测试【论文】。
- 论文：82% 的片段信号质量低于背景质量（SIG < BAK），与 DNS 系列以噪声为主不同【论文「Distribution of dimensions」一节】。
- 同仓库 `ICASSP2024/` 有 2024 版盲测集（500 个文件以 LFS 指针存放）与 SigMOS 无参考模型（ONNX）【官方文档】。
- 怀疑派复核：新补卡（原稿漏）。
