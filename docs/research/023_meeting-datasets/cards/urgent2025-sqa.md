# URGENT 2025 SQA（增强后语音的人评分 + 多指标集）

- **名称**：urgent2025-sqa（URGENT 2025 挑战提交结果的语音质量评估集）
- **链接**：https://huggingface.co/datasets/urgent-challenge/urgent2025-sqa
- **许可**：HF 数据卡标签 cc-by-nc-sa-4.0（卡内许可正文本路未读到）→ 结论：只评测【官方文档】
- **大小**：默认子集约 168k 行：blind_test_mos 23.4k · blind_test 44.1k · nonblind_test 92k · validation 8k；GB 未查到【官方文档】
- **场景键**：未查到（源自 URGENT 2025 盲测，014 卡称来源未点名）
- **设备键**：unknown
- **笔记本段**：未查到；无逐条设备标签【官方文档】
- **干净参考**：部分（模拟一半有参考；表内对部分行给了 PESQ、ESTOI 等全参考分，说明这些行有参考【推断】）
- **转写**：未查到（表头无转写列；有 `sbert` 列）【官方文档】
- **语种**：标签含英语、中文、日语，另 3 种未显示【官方文档】
- **语体**：未查到
- **获取方式**：直接下载（Hugging Face；是否需点同意未核）
- **与 D62 的关系**：是 26 个系统（`system_id`）对 URGENT 2025 测试集的增强结果；盲测集约一半是真实录音（014 `urgent2025-blind-real.md`），本集 MOS 部分里真实 / 模拟各占多少未查到。每条带 8 个原始人评分（`raw_ratings`）与均值 `mos`，同时预先算好 DNSMOS OVRL、NISQA、SCOREQ、UTMOS、Distill-MOS、SIGMOS 各维、说话人相似度等列。若开发阶段拿来评测：只取 `blind_test_mos` 里的 `language=zh` 行，直接算「各 verifier 与人评分的相关」而不必自己跑模型；大小未查到
- **出处**：https://huggingface.co/datasets/urgent-challenge/urgent2025-sqa【官方文档】; https://arxiv.org/abs/2507.11306【论文】; https://arxiv.org/abs/2505.23212【论文，未读】

## 备注
- 对本路最有用的一份：**中文 + 增强后语音 + 人评分 + 现成多 verifier 分数**，能直接做 verifier 校准表。
- 评分协议：配套论文 2507.11306 写的是 P.808 ACR，并把 P.808 文本与音频提示本地化到多语种【论文摘要】；每语种评分人数未查到。
- 该论文提醒：生成式增强会「幻觉」，单靠 ACR MOS 和 DNSMOS / NISQA 抓不到，建议配音素保真度指标【论文摘要】。
