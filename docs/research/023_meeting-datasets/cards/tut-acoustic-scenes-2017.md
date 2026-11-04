# TUT Acoustic Scenes 2017（DCASE 2017 Task 1）

- **名称**：TUT Acoustic Scenes 2017, Development dataset（TUT-AS-2017）
- **链接**：https://zenodo.org/records/400515
- **许可**：Zenodo 许可字段「Other (Non-Commercial)」，正文写「No further description」→ 结论：只评测【官方文档】
- **大小**：15 个场景 × 312 段 × 10 s，每场景 52 min，共 4,680 段约 13 h；Zenodo 文件合计 10.7 GB【官方文档】；段数合计与小时数由 15 × 312、15 × 52 min 算得【推断】
- **场景键**：cafe, open-office, home-family, car-passenger, transit-hub, street-outdoor, other（公交、电车、火车车厢；杂货店、图书馆；森林、湖边、公园、住宅区）
- **设备键**：non-laptop-studio（Soundman OKM II 双耳入耳麦 + Roland Edirol R-09 录音机；模板无双耳麦键，暂记 non-laptop-studio）
- **笔记本段**：没有；全集只有一种设备
- **干净参考**：没有（场景分类集，无目标语音）
- **转写**：没有
- **语种**：无目标语音；背景人声录于芬兰，语种未标（推断多为芬兰语）
- **语体**：无目标语体；cafe/restaurant、office、metro station 等场景里应有背景交谈，未标注【推断】
- **获取方式**：直接下载（Zenodo，按 zip 分包）
- **与 D62 的关系**：真实录音（环境声），无目标说话人，只能当合成用的噪声原料或场景对照。若开发阶段拿来评测：取 cafe/restaurant、office、home、car、metro station、city center 6 类当混合噪声，每类 52 min；6 类约 4.3 GB【推断：按 10.7 GB × 6/15】
- **出处**：https://zenodo.org/records/400515【官方文档】; https://dcase.community/challenge2017/task-acoustic-scene-classification【官方文档】

## 备注
- 怀疑派补卡：r2-place 把「DCASE TUT 2016–2019 场景集」列为本轮未查；它是本路唯一同时有 office（办公室）、cafe/restaurant、home、car 四类按地点切好的真实背景集，补上。
- 「office」是否开放办公未写明（DCASE 页只写 Office (indoor)），场景键 open-office 属推断。
- 录于 2015-06 至 2017-01，芬兰；原始录音 3–5 min，切成 10 s。
- 怀疑派复核：新卡，许可、大小、设备均读官方页。
