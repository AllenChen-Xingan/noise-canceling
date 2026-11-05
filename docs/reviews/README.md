# reviews — 对抗性评审记录

> **异质双谱系**：找两条不同来路的「框架 + 模型」分别评审，别都用同一家。
> 谱系＝一整套装置（框架配模型），换谱系才换得掉偏见——同一家的两个会话，权重那一半没换。
> **只查事实、逻辑、合规——风格的事它说了不算，你说了算。**
> 难题的经验量级：两个旗舰模型攻关三轮，之后才实际执行，执行完再评审一次。
> 编号与 `plan/` 同号对应，便于对照。

| 编号 | 日期 | 评审对象 | 采纳 / 提出 |
|------|------|---------|------------|
| 010 | 2026-11-02 | [exp010 推理后端基准纯 Rust 一侧](010_exp010-rust-codex.md)（README + bench/src，codex，5 轮，已达停止线） | 采纳 27 / 提出 27（P1×7 P2×20） |
| 019 | 2026-11-02 | [019 推理后端总方案与判据](019_apple-inference-codex.md)（decision + standard + apple-rs，codex gpt-6-astra，8 轮，已达停止线） | 采纳 41（其中 2 条 P2 列进待办）/ 提出 41 |
| 011 | 2026-11-02 | [删镜像副本 + 参数描述表 + tidy 新规则](011_types-single-source-codex.md)（`git diff bfe6266`，issue #56，codex，3 轮，已达停止线） | 采纳 14 / 提出 14（P1×1 P2×13） |
| 011 | 2026-11-02 | [schema 导出机制与第一批 schema](011_schema-crosscheck-codex.md)（diff，codex，5 轮，停止线满足） | 采纳 15 / 提出 16（P1×5 P2×11；1 条 P2 留作 issue） |
| 062 | 2026-11-02 | [代码卫生：600 行上限 · 采样率只写一处 · 评测阈值正本](062_code-hygiene-codex.md)（`origin/main...HEAD`，issue #62，codex，9 轮，已达停止线） | 采纳 21 / 提出 21（P1×11 P2×10；其中测试模块识别相关条目随功能删除作废） |
| 020 | 2026-11-02 | [研究 020 手写 GTCRN 引擎提速](020_fastpath-codex.md)（decision + rewrites + exp012 README + bench/src，codex gpt-6-astra，3 轮，已达停止线） | 采纳 14 / 提出 14（P1×3 P2×10 + 抽查不符 1；3 件 P2 后续留作 issue） |
| 011 | 2026-11-02 | [处理图描述 + Pipeline 交叉校验 + 会话记录记图](011_processing-graph-codex.md)（`git diff 0604143...HEAD`，issue #64，codex gpt-6-astra，2 轮，已达停止线） | 采纳 4 / 提出 4（P2×4；1 条留作 issue） |
| 012 | 2026-11-02 | [参数描述导出 · 模型接口三方一致 · FE-M 名字唯一 · 模型节点带权重版本](012_param-model-schema-codex.md)（diff，codex，7 轮，停止线满足） | 采纳 18 / 提出 19（P1×3 P2×16；另 1 条 P1 部分接受；无 issue） |
| — | 2026-11-02 | 项目难度与机会判断（codex 评审 + 联网调研，`outlook-codex-20261102/`） | 采纳：Mac 非空白、评测方法非首创 → D77 · 待拍板 6-1 |
| 071 | 2026-11-03 | [tidy 规则迁到 syn 语法树](071_tidy-syn-codex.md)（`origin/main...HEAD`，issue #71，codex，8 轮；未达停止线，人定 2026-11-03 提前合并） | 采纳 36 / 提出 37（P1×14 P2×23；1 条 P1 绕过写法按人定记入后续 issue） |
| 021 | 2026-11-03 | [021 开会场景目录](021_meeting-scenes-codex.md)（catalog · coverage · decision · sources · check_catalog.py，codex gpt-6-astra，5 轮，已达停止线） | 采纳 23 / 提出 23（P1×5 P2×18；无 issue） |
| 022 | 2026-11-03 | [022 Mac 内置麦调研结论](022_mac-mic-codex.md)（devices · simulation · selftest · decision · sources · tunings · tools，codex gpt-6-astra，5 轮，已达停止线） | 采纳 22 / 提出 22（P1×7 P2×15；无 issue） |
| 023 | 2026-11-03 | [开会场景真实录音数据集调研：结论层与卡片检查脚本](023_meeting-datasets-codex.md)（decision · matrix · pick · sources · check_cards.py，issue #82，codex gpt-6-astra，5 轮，已达停止线） | 采纳 30 / 提出 30（P1×10 P2×20；无 issue） |
| 087 | 2026-11-03 | [场景成绩单 + 新评分项 + 评分模型接入](087_scorecard-codex.md)（`origin/main...HEAD`，issue #87，codex gpt-6-astra，8 轮，已达停止线） | 采纳 17 / 提出 17（P1×8 P2×9；无 issue） |
| 013 | 2026-11-03 | [exp013 场景测试集：生成器 · 设备链 · 实测房间](013_scene-suite-codex.md)（exp013 目录 + scenes.json，issue #86，codex gpt-6-astra，4 轮，已达停止线） | 采纳 13 / 提出 15（P1×4 P2×11；2 条 P2 留作 issue） |
| 014 | 2026-11-04 | [exp014 GTCRN 第一张场景成绩单：结论](014_scorecard-gtcrn-codex.md)（findings.md · analyze.py，issue #92，codex gpt-6-astra，3 轮，已达停止线） | 采纳 12 / 提出 12（P1×2 P2×10；无 issue） |
| 015 | 2026-11-04 | [exp015 认人耗时估算与计时命令](exp015-speaker-shared-spectrum-codex.md)（exp015 README + speaker-lab bench.rs，issue #95，codex gpt-6-astra，4 轮，已达停止线） | 采纳 10 / 提出 10（P1×1 P2×9；无 issue） |
| 094 | 2026-11-04 | [每帧只算一次频谱：参数正本 · 借谱接口 · 场景 / 修音共用](013_frontend-once-codex.md)（`990e7843...HEAD`，issue #94，codex gpt-6-astra，3 轮，已达停止线；另有多视角自审 `013_frontend-once-selfreview.md`） | 采纳 8 / 提出 8（P1×1 P2×7；无 issue） |
| 024 | 2026-11-04 | [个性化降噪（先认人再提取）调研结论](024_pse-landscape-codex.md)（decision · sources · notes/01–05，issue #98，codex gpt-6-astra，4 轮，已达停止线） | 采纳 20 / 提出 20（P1×6 P2×14；无 issue） |
| 102 | 2026-11-04 | [统一频谱模块支持 Kaldi 分帧 · 认人 fbank 改调 · R-FFT 规则](102_frontend-speaker-codex.md)（`d23bfe66..HEAD`，issue #102，codex gpt-6-astra，2 轮，已达停止线） | 采纳 2 / 提出 3（P2×3；驳回 1；无 issue） |
| 025 | 2026-11-04 | [成绩单评分方法对照业界做法](025_eval-best-practice-codex.md)（matrix · decision · sources · notes/group-A–F · notes/scripts，issue #105，codex gpt-6-astra，9 轮，已达停止线） | 采纳 35 / 提出 35（P1×15 P2×20；无 issue） |
| 104 | 2026-11-04 | [评测器 4 处频谱计算改调统一模块 · 评分逐位不变](104_frontend-eval-codex.md)（`f2782cf4...HEAD`，issue #104，codex gpt-6-astra，2 轮，已达停止线） | 采纳 4 / 提出 4（P2×4；无 issue） |
| 111 | 2026-11-04 | [成绩单评分方法修正：新汇总规则 v2 · #109 · #110](111_scorecard-v2-codex.md)（`4493f0ce..HEAD`，issue #111，codex gpt-6-astra，中途 1 · 中途 2 各 2 轮；收尾 2 轮，连续两轮无 P0/P1 停） | 采纳 19 / 提出 19（P1×7 P2×12；无 issue） |
| 016 | 2026-11-04 | [exp016 候选模型上同一张成绩单：① 外部离线输出入口](016_candidates-codex.md)（`4493f0ce...HEAD`，issue #112，codex gpt-6-astra，中途 2 轮，P0/P1 清零即停） | 采纳 6 / 提出 6（P1×1 P2×5；2 条 P2 留作 issue） |
| 027 | 2026-11-05 | [调研 027 GTCRN 微调方案 + exp018 MPS 冒烟](027_gtcrn-finetune-codex.md)（research，codex gpt-6-astra，进行中） | 采纳 6 / 提出 6（P1×2 P2×4） |
