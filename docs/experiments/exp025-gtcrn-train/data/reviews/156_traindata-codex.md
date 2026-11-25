# 评审 156 · exp025 训练数据：下载 · 切分 · 混音 · exp013 核对（codex · gpt-6-astra · read-only）

> 对象：`docs/experiments/exp025-gtcrn-train/data/` 下 29 个代码、规则与说明文件（数据产物 `splits/` · `rebuild/*.log` · `builds/` · `inventory.txt` 不发）。**最新一轮：第 1 轮（2026-11-25）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×3、P2×3 逐条回源码核实，属实 6、驳回 0。1 条 P2 留作 issue（待大总管开）。
> 记录放在本目录，因为 `docs/reviews/` 不是 traindata 能写的文件；索引那一行请大总管加。
> 之前做过一轮多视角自审（外行 · 老手 · 挑刺，2026-11-25）：1 条 P1（判据 2 认不出云上自产音频）已修，见第 1 轮之前的提交。

## 第 1 轮（2026-11-25）· 总评「修改」

原文：`_tmp/reviews-traindata/exp025-data-codex-1.md`（不入库）。effort high，耗时 144 秒。行号指评审时的版本。
本轮 P0/P1：3 条 · 停止线：未满足 · 留作 issue 的 P2：#6（待开）

- **P1｜loader.py:45 训练 build 身份漏了实际输入**（第 1 轮 #1）
  - 核实：属实。`CODE` 只有 mix · loader · shard，真正在 GPU 上混音的 `gpu_mix.py` 不在；分片索引（成员偏移）只记了分片 tar 的哈希，索引本身没进 build_id
  - 处理：`CODE` 加 gpu_mix · leakcheck · split；`parts` 加 `index_sha256`。先写会红的测试 `test_mix_loader.EndToEnd.test_build_id_covers_index_and_gpu_code`（改一个成员偏移，id 必须变）。已登记的 build `49349c158a4defdc` 因此要在云上重跑 `register`（README 第七节写明）
- **P1｜loader.py:59 交集检查漏掉终测原料池的人声来源**（第 1 轮 #2）
  - 核实：属实。register 只查分片索引里的来源，`librispeech-dev-clean-test-other` 不进分片就没查，和 README 第三节「三份全部原料」不符。本机补查全部 6 个来源：83,460 行，0 处交集（实测）
  - 处理：register 按规则里的全部来源加载切分表，缺表即停；分片用到的表照旧核哈希；build.json 记 `leak_checked_sources`。测试 `test_leak_check_covers_every_rule_source` · `test_register_refuses_missing_table`（夹具加了 4 个 dev-clean / test-other 说话人）
- **P1｜rebuild_exp013.py:139 第 1 版判据的报告仍能放行**（第 1 轮 #3）
  - 核实：属实。`drift_covered` 只看 ok 与代码哈希
  - 处理：报告带 `criterion`，`drift_covered` 只认 `CRITERIA` 里的；第 1 版报告没有这个字段，不再算。测试里加了第 1 版报告必须不算的反例
- **P2｜exp013_inputs.py:89 原料记录不全也能报「原料全对」**（第 1 轮 #4）
  - 核实：属实。空记录也返回无错
  - 处理：`check_record_complete` 先核记录条数、路径不重复、索引等于旧 build 的 material_index；抽样改为显式参数 `sample`。测试 `test_incomplete_record_red`（空 · 少一条 · 多一条重复都要红）。云上那次核对用的是仓里完整的 1,994 条记录，结论不受影响
- **P2｜tests/test_products.py:35 产物整体丢失不报红**（第 1 轮 #5）
  - 核实：属实
  - 处理：产物已进仓，缺了改成断言失败，不再跳过
- **P2｜README.md:157 改规则重切没有可执行路径**（第 1 轮 #6）
  - 核实：属实。`split.write` 拒覆盖，三步共用 `splits/`
  - 处理：留作 issue（要给 split · shard · register 加贯穿的目录参数）。README 6.3 写明这条路还没打通

### 引用抽查

- 「不符」README.md:67（交集检查只查分片引用的来源）：同 #2，已改
- 「核实属实」：5 条
- 无法核实：mix.py:30 `early_rir` 语义（rooms.py 不在发送清单里）。它是 evalcmp 的正本，已随 main 合入，有 `test_mix_loader.Mix.test_target_is_early_reverb_from_rooms` 对拍

### 是否再审

有 P1，改完开第 2 轮，附本记录。
