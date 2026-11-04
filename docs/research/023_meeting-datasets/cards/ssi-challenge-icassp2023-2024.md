# Speech Signal Improvement Challenge 测试集（ICASSP 2023 / 2024，Microsoft）

- **名称**：ICASSP 2023 / 2024 Speech Signal Improvement Challenge test / blind sets（SSI、仓库名 SIG-Challenge）
- **链接**：https://github.com/microsoft/SIG-Challenge
- **许可**：仓库 LICENSE 为 MIT（只覆盖代码）；README 原句「The datasets are provided under the original terms that Microsoft received such datasets.」写在「Training data」一节下，说的是训练数据；test / blind 集所在的「Datasets」一节没有任何许可句 → 结论：未查到【官方文档：README.md、ICASSP2023/README.md、LICENSE】
- **大小**：ICASSP2023/test_data 500 条（约 414 MB）· ICASSP2023/blind_data 500 条（约 432 MB）· ICASSP2024/blind_data 500 条（git LFS，大小未查到）· 2023 test 转写 500 个 txt【实测：GitHub git tree API 读文件表与字节数】
- **场景键**：other（通话发送端；按「噪声 / 混响 / 失真 / 音色 / 断续 / 响度」分层抽样，不按地点标）
- **设备键**：laptop-unknown, non-laptop-desktop, non-laptop-phone, non-laptop-unknown（论文只说「PCs and mobile devices」，每条一台不同设备）
- **笔记本段**：可能有，但不能筛。论文原句「each using a unique device, environment, and person speaking… captured from both PCs and mobile devices」；文件名为 GUID，无设备字段【论文 §V】【实测：文件名】
- **干净参考**：没有（真实发送端录音）
- **转写**：部分（2023 test 集 500 条有转写，2024 年随挑战发布，可算字准确率；无时间戳；2023 / 2024 blind 未见转写）【官方文档】【论文 2401.14444 §2】
- **语种**：英语约 80%，另有德、荷、法、西语【论文 §V】
- **语体**：未查到（论文未写朗读还是自由说；2024 blind 另含少量 Common Voice 朗读片段）【论文 2401.14444 §3.1】
- **获取方式**：直接下载（GitHub 仓库目录，2024 blind 走 git LFS）
- **与 D62 的关系**：真实录音（众包真人在自己设备上录，方法同 ICASSP 2022 AEC 挑战）。开发阶段若拿来评测：2023 test_data 500 条 + 转写 = 现成的「真实设备 + 字准确率」回归集，约 414 MB；P.804 人评分（SIG 等维度）只在论文图里有分布，逐条分数未见公开
- **出处**：https://arxiv.org/abs/2303.06566 §V【论文】; https://arxiv.org/abs/2401.14444 §2、§3.1【论文】; https://github.com/microsoft/SIG-Challenge（README、ICASSP2023/README.md、LICENSE）【官方文档】

## 备注
- 和 DNS 盲测集同一团队、同一采集方法，但 SSI 的抽样目标是「信号失真」（音色、断续、削波）而不只是噪声，适合查降噪对设备本身失真的副作用【推断】。
- 仓库另附 SigMOS（P.804 维度的无参考估计器，ONNX），见 `ICASSP2024/sigmos/`【官方文档】；交给 r3 路的 verifier 清单参考。
- 2024 blind 与 2023 blind 有同名 GUID 文件（如 `000e74c5-…wav` 两处都有），是否同一条录音未核【实测：文件名】。
- 怀疑派复核：重读 README 与 LICENSE，用 git tree 接口重数文件（2023 test 500 条 414.35 MB、2023 blind 500 条 432.05 MB、2024 转写 500 个、2024 blind 500 个 LFS 指针，均与原卡一致）；回 arXiv 2303.06566 §V 与 2401.14444 §3.1 核「500 条 · 每条一台设备 · PCs and mobile · 约 80% 英语」，一致。改 1 处：原卡把 README 那句许可话当成测试集的条款，原文它在「Training data」一节下，已改写；结论仍为未查到。另记：2023 论文 §VI.F 称 blind 集 WER 用人工转写算，但转写未公开。
