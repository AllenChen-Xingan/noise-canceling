# 评审记录：正式训练改用第五批 build + 正式执行单（codex，read-only，effort medium）

范围 `origin/main...HEAD`（分支 feat/trainer-formal-b5，Refs #212）。重点四条：
1. 四份配置都改指第五批 build `df759bb8d606a209` 和 `/workspace/shards/batch5`
2. `tests/_shipped.py` 让测试里的期望值只写一处
3. 执行单 `plan-h100-formal.md` 的前提、闸门、时长和花费
4. 执行单里对另外两份执行单的引用

代码由 pi `trainer-b5-pin` 写（dd5ed3e4、2b0cd25a，花 0.0106 美元）；执行单由 trainer 写。

## 验收前 trainer 自查（2026-11-26）

- 红灯：trainer 在 dd5ed3e4 的复制件上复现（只有测试、配置还是旧的），9 败 95 过，9 条都是比对仓里配置的用例。
- 绿灯：合入后 trainer 全部测试 531 过、2 跳过。两个 check 在真记录上都退出 0，打印「可以开训」。

## 第 1 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×0 P2×1。核实属实。
- **P2｜开跑前提没写磁盘余量**（第 1 轮 #1）
  - 问题：`data/cloud-batch5.md` 第 98 行写着第五批落盘后约剩 22 GB，低于正式训练要的 30 GB，要先删 batch4。执行单没写这一条。
  - 改法：trainer 在第一节加第 6 条：开机后第一步实测 `df -h /workspace`，不足 30 GB 就停；删 batch4 照 cloud-batch5.md 第六节由 cloudrun 做，本单不删东西。
- 抽查 5 条属实；「无法核实」1 条：build 记录、plan-h100-smoke.md 第十二节和 11-25 的耗时文件不在允许读的清单里。build 记录由两个 check 实测核过。

## 第 2 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×0 P2×1。连续第 2 轮无 P0/P1，**达停止线**（脚本判「满足停止线」）。上一轮 1 条已修。核实属实，抽查 8 条也都属实。
- **P2｜耗时依据都算成了第四批**（第 2 轮 #1）
  - 问题：3 个 B 同时跑的 0.39 s、158 s、2.2 s 是 2026-11-25 在 build f032a75e（first-batch）上测的，不是第四批。
  - 改法：trainer 当场改了第四节，分两条写来源，标明 B 的读数跨了数据批次，换到第五批是推断。改动只一行，所以不开 issue。
