# MOBIO（Idiap 手机与笔记本双模态生物识别库）

- **名称**：MOBIO Database（MOBIO）
- **链接**：https://www.idiap.ch/dataset/mobio ; https://zenodo.org/records/4269551
- **许可**：Zenodo 访问条件原句「Access to the dataset is based on an End-User License Agreement. The use of the dataset is strictly restricted to non-commercial research.」→ 结论：只评测（且需签 EULA）【官方文档】
- **大小**：150 人（99 男 51 女）× 12 个 session（Phase I 每次 21 问、Phase II 每次 11 问）；小时数、GB 未查到【官方文档】
- **场景键**：other（6 个站点、5 个国家，2008-08 至 2010-07；具体环境未写，论文题名称「challenging conditions」）
- **设备键**：mac-intel-builtin（官方原句「the laptop computer was a standard 2008 MacBook」，是 MacBook 产品线，不是 Air 也不是 Pro）, phone（NOKIA N93i）
- **笔记本段**：有，但量小。笔记本只录了第一个 session 的一部分；官方原句「The laptop was only used to capture part of the first session」。能否按设备筛：bob.db.mobio 的 PyPI 说明称「the number of protocols has increased, considering the additional data recorded using laptops, which has led to 8 protocols」，即有用笔记本数据的协议；具体协议名未读到【官方文档】
- **干净参考**：没有
- **转写**：未查到（Idiap 数据页与 Zenodo 记录都没提转写；朗读段文本固定，可自行对照）【官方文档】
- **语种**：英语（母语与非母语者）
- **语体**：问答短答、约 5 s 短自由说、朗读固定段落、约 10 s 自由说；无笑声 / 附和 / 情绪标注
- **获取方式**：签协议（Zenodo 申请表，签字人须为在编人员，限非商业研究）
- **与 D62 的关系**：真实录音（真人对着手机与 2008 款 MacBook 说话，音视频同录）。开发阶段若拿来评测：只有第一个 session 的笔记本段可用，量未查到；需先签 EULA
- **出处**：https://www.idiap.ch/dataset/mobio【官方文档】; https://zenodo.org/records/4269551（访问条件与说明）【官方文档】; https://pypi.org/project/bob.db.mobio/（协议数说明）【官方文档】; Khoury et al., Image and Vision Computing 32(12), 2014, doi:10.1016/j.imavis.2013.10.001【论文】

## 备注
- 价值只在「历史 Mac 内置麦真人录音」这一点；2008 款机型的麦与今天 MacBook Air 的三麦阵列差别大，不能当 Mac 设备响应的代表【推断】。已转交 macmic 路（`../notes/r1-device-macmic-handoff.md`）。
- 签协议类，本路未注册、未申请。
- 怀疑派复核：重读 Idiap 数据页与 Zenodo 记录（许可原句、「standard 2008 MacBook」「only used to capture part of the first session」均一致）。改 3 处：①设备键 `mba-builtin` → `laptop-other`（官方写的是 MacBook，不是 Air）；②「Idiap 曾发布 transcriptions」无出处，两处官方页都没提，改为未查到；③协议出处换成能读到原句的 PyPI 页（pythonhosted 页上没有 laptop 字样）。
