# TAU Urban Acoustic Scenes 2020 Mobile（DCASE 2020 Task 1a）

- **名称**：TAU Urban Acoustic Scenes 2020 Mobile, Development dataset（TAU-UAS-2020-Mobile）
- **链接**：https://zenodo.org/record/3819968
- **许可**：Zenodo 许可字段「Other (Non-Commercial)」，页面未给正文 → 结论：只评测【官方文档】
- **大小**：64 h，10 s 一段；Zenodo 文件合计 27.4 GB【官方文档】
- **场景键**：transit, street, other（购物中心、公园、广场；另有电车 / 公交 / 地铁车厢内）
- **设备键**：phone, studio, unknown
- **笔记本段**：没有。真机 A = Soundman OKM II 双耳麦 + Zoom F8，B = Samsung Galaxy S7，C = iPhone SE；S1–S10/S11 为用设备 A 录音加脉冲响应模拟的「虚拟设备」，脉冲响应不公开。有逐段设备标签（文件名含设备号），可按设备筛【官方文档】
- **干净参考**：没有（场景分类集，无目标语音）
- **转写**：没有
- **语种**：无目标语音；背景人声来自 12 个欧洲城市，语种未标【官方文档】【推断】
- **语体**：无目标语体；机场、地铁站、广场等场景里应有路人交谈与广播，但未标注（是否含人声官方页未说）【推断】
- **获取方式**：直接下载（Zenodo，Open）
- **与 D62 的关系**：真实录音（环境声），无目标说话人，只能当合成用的噪声原料或场景声学对照。若开发阶段拿来评测：取 airport、metro_station、street_pedestrian、street_traffic、bus、tram、metro 等场景、只取设备 A（或 B/C 手机段）做噪声，按场景每类几百段 × 10 s；全量 27.4 GB，单场景单设备约十分之一【推断】
- **出处**：https://zenodo.org/record/3819968【官方文档】; https://dcase.community/challenge2020/task-acoustic-scene-classification【官方文档】

## 备注
- 这是本路「机场 / 车站」唯一找到的、按地点逐段标好的公开真实录音；但没有「人在机场对着设备说话」的目标语音。
- 有手机真机（S7、iPhone SE）段，可做「手机麦频响 vs 双耳麦」对照，但仍不是笔记本麦。
- DCASE 历年还有 TUT 2016–2019 场景集（含 cafe/restaurant、office、home、car、train 等），本轮未逐个查许可。
