# 评审记录：加训场景按地点键表查（codex，read-only，effort medium）

范围 `da9008e1..HEAD`（分支 feat/trainer-target-scenes，issue #209）。重点：`place_keys` 是否只把 `specs/schema/eval-suite.schema.json` 的 `$defs.MeetingPlace.enum` 当正本、去掉兜底键 `other`；target_scenes 各种坏输入是否都报出、不误报；仓里矩阵是否只剩两处路径占位。代码由 pi `trainer-target-scenes` 写（提交 d9c5f527..3afe9e9c）。

## 第 1 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×0 P2×1。
- **P2｜地点键表读不了时整个检查中断**（第 1 轮 #1）：`place_keys()` 在文件缺失、JSON 损坏或缺 enum 时抛异常，`problems()` / `main()` 不接，已经收集的两处路径占位被 traceback 盖掉。核实属实。不进停止线，开 issue 留后处理：改成把读不了正本记为一条拦路问题、其余照查，补缺失 / 损坏 / 缺字段三条测试。现在的行为是大声失败、不会放行开训，所以不挡本 PR。
- 抽查引用 4 条属实、3 条「无法核实」（地点键表、formal.yaml、D113 原文不在允许补读的清单里）。trainer 补核：`specs/schema/eval-suite.schema.json` 的 MeetingPlace 含这 4 个场景；`formal.py check` 实跑只报两处路径占位；D113 来自大总管 c4 2026-11-26 的转达。
