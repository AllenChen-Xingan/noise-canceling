# URGENT 2025 SQA（增强后语音的人评分 + 多指标集）

- **名称**：urgent2025-sqa（URGENT 2025 挑战提交结果的语音质量评估集）
- **链接**：https://huggingface.co/datasets/urgent-challenge/urgent2025-sqa
- **许可**：HF 数据卡 YAML `license: cc-by-nc-sa-4.0`；卡内正文另写挑战数据「publicly available with varying licenses」，源语料许可表标「to be updated」→ 结论：只评测【官方文档】
- **大小**：默认子集约 168k 行：blind_test_mos 23.4k（卡内原句：26 个系统 × 900 条 = 23,400 条全部有人评分）· blind_test 44.1k · nonblind_test 92k · validation 8k；GB 未查到【官方文档】
- **场景键**：未查到（源自 URGENT 2025 盲测，014 卡称来源未点名）
- **设备键**：unknown
- **笔记本段**：未查到；无逐条设备标签【官方文档】
- **干净参考**：部分（模拟一半有参考；表内对部分行给了 PESQ、ESTOI 等全参考分，说明这些行有参考【推断】）
- **转写**：未查到（表头无转写列；有 `sbert` 列）【官方文档】
- **语种**：英、中、日、德、西、法六种（README YAML `language` 字段）【官方文档】
- **语体**：未查到
- **获取方式**：直接下载（Hugging Face；数据页未见「同意后访问」提示）
- **与 D62 的关系**：是 26 个系统（`system_id`）对 URGENT 2025 测试集的增强结果；盲测集约一半是真实录音（014 `urgent2025-blind-real.md`），本集 MOS 部分覆盖整份 900 条盲测（每系统 900 条），所以真实段比例与盲测相同，约一半【推断】。每条带 8 名 MTurk 评分人按 P.808 打的原始分（`raw_ratings`）与均值 `mos`【官方文档】，同时预先算好 DNSMOS OVRL、NISQA、SCOREQ、UTMOS、Distill-MOS、SIGMOS 各维、说话人相似度等列。若开发阶段拿来评测：只取 `blind_test_mos` 里的中文行（约 26 × 150 = 3,900 条【推断：每语种 150 条，见 014 卡】），直接算「各 verifier 与人评分的相关」而不必自己跑模型；是否有现成语种列未核；大小未查到
- **出处**：https://huggingface.co/datasets/urgent-challenge/urgent2025-sqa【官方文档】; https://arxiv.org/abs/2507.11306【论文】; https://arxiv.org/abs/2505.23212【论文，未读】

## 备注
- 对本路最有用的一份：**中文 + 增强后语音 + 人评分 + 现成多 verifier 分数**，能直接做 verifier 校准表。
- 评分协议：配套论文 2507.11306 写的是 P.808 ACR，并把 P.808 文本与音频提示本地化到多语种【论文摘要】；每语种评分人数未查到。
- 该论文提醒：生成式增强会「幻觉」，单靠 ACR MOS 和 DNSMOS / NISQA 抓不到，建议配音素保真度指标【论文摘要】。
- 怀疑派复核：改了 5 处——「许可」补卡内「varying licenses」原句；「大小」补 26 × 900 原句；「语种」从「另 3 种未显示」补齐为六种；「与 D62 的关系」把「真实比例未查到」改为「覆盖全部盲测，约一半真实【推断】」，中文行估 3,900 条【推断】；「获取方式」去掉「未核」。注意：卡内一段已注释掉的旧文字写「All speech samples in this dataset are in English」，与 YAML 冲突，以 YAML 为准。
