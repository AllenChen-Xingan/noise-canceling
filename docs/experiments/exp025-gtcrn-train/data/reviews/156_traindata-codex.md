# 评审 156 · exp025 训练数据：下载 · 切分 · 混音 · exp013 核对（codex · gpt-6-astra · read-only）

> 对象：`docs/experiments/exp025-gtcrn-train/data/` 下 29 个代码、规则与说明文件（数据产物 `splits/` · `rebuild/*.log` · `builds/` · `inventory.txt` 不发）。**最新一轮：第 4 轮（2026-11-25）· 总评「修改」· 已达停止线（第 3、4 轮连续无 P0/P1）**。
> 累计处理：P1×5、P2×15 逐条回源码核实，属实 20、驳回 0。其中 1 条 P1 按大总管定的规则降为 P2（同一主题往上加层）。留作 issue 的 P2 共 3 件（待大总管开）：① register 时在云上核分片 tar 内容；② split · shard · register 贯通切分表目录参数，打通改规则重切；③ split_rules.json 的说明字段移出规则哈希（自审发现）。
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

## 第 2 轮（2026-11-25）· 总评「修改」

原文：`_tmp/reviews-traindata/exp025-data-codex-2.md`（不入库）。effort high，耗时 156 秒。
上一轮处理的验证：codex 认 #2–#5 已落实；#1 没改全（见本轮 #1）；#6 仍是 issue。
本轮 P0/P1：2 条（其中 #2 降为 P2，见下）· 停止线：未满足 · 留作 issue 的 P2：#2、#5（同第 1 轮 #6）
另：本轮期间 cloudrun 的分支合进 main（df145344），合入后 `test_real_cloud_config_quota` 转绿，全量 135 条全过（实测）。

- **P1｜loader.py:28 build 身份仍漏了 mix.py 直接调的 exp013 模块**（第 2 轮 #1，第 1 轮 #1 没改全）
  - 核实：属实。mix.py:29–31 导入 exp013 的 dsp · rooms · rirs；rooms.py 刚随 evalcmp 合入改过
  - 处理：`code_sha256` 加 `exp013/dsp.py` · `exp013/rooms.py` · `exp013/rirs.py`。loader 不导入 mix（登记不需要混音代码，有测试核），所以在 loader 里另写 `EXP013` 路径，并加测试核它等于 `mix.EXP013`
- **P1→P2｜loader.py:49 登记没核分片 tar 的实际内容**（第 2 轮 #2）
  - 核实：属实（换了 tar 文件，身份不变，加载也不核）
  - 定级：按大总管 2026-11-25 定的规则，同一主题（build 身份）一轮轮往上加层，最高 P2。核 15 GB 分片要在云上做
  - 处理：留作 issue：register 时在云上逐个核 tar 的 sha256，加载时可选抽核
- **P2｜rebuild_exp013.py:100 `--cloud` 误填成本机那份也能过**（第 2 轮 #3）
  - 核实：属实
  - 处理：加「云上与本机须是不同 build」一项，判据升为第 3 版；本机用同样两份日志重出报告 `rebuild/exp013-20261124-195551.json`，四项全过。第 2 版报告 `194001` 不再算数。测试里正例改用仓里真的 aa9f 小文件，另加两端同一 build 必须红的反例
- **P2｜tests/test_products.py:120 build 记录的身份字段被改了也照样全绿**（第 2 轮 #4）
  - 核实：属实
  - 处理：`loader.build_id_of` 按 `ID_KEYS` 由记录的字段重算 id；早期 build 没有 `index_sha256`，缺的字段不进哈希，所以旧记录也能核。`test_products.Builds` 现在重算比对
- **P2｜README.md:158 验收失败仍指向没打通的重切流程**（第 2 轮 #5）
  - 核实：同第 1 轮 #6
  - 处理：同一件 issue

### 引用抽查

- 「核实属实」：7 条
- 无法核实：mix.py:30（同第 1 轮）；README.md:188 的上传与原料日志不在发送清单里（两份日志在 `rebuild/` 里进了仓，sha256 记在报告里）

### 是否再审

有 P1，改完开第 3 轮。

## 第 3 轮（2026-11-25）· 总评「修改」

原文：`_tmp/reviews-traindata/exp025-data-codex-3.md`（不入库）。effort high。
上一轮处理的验证：codex 认 #1、#3、#4 已落实；#2、#5 是 issue，维持 P2。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作 issue 的 P2：#1（同第 2 轮 #2）、#2（同第 1 轮 #6）

- **P2｜loader.py:52 分片内容仍未核**（第 3 轮 #1）：同第 2 轮 #2，issue
- **P2｜README.md:158 重切流程没打通**（第 3 轮 #2）：同第 1 轮 #6，issue
- **P2｜fetch.py:137 分块响应截断不重试**（第 3 轮 #3）
  - 核实：属实。`http.client.IncompleteRead` 是 `HTTPException`，不是 `OSError`
  - 处理：一并捕获 `http.client.HTTPException`，从 .part 末尾接着下。测试 `test_chunked_cut_resumes`（先红后绿）
- **P2｜filecache.py:49 缓存键没绑数据根**（第 3 轮 #4）
  - 核实：属实
  - 处理：缓存行记规范化的数据根，按（数据根 · 相对路径）查；没记数据根的旧行不认。云上已有的缓存因此下次全算一遍（约 1346 s），README 6.2 写明。测试 `test_other_root_not_reused`
- **P2｜loader.py:165 target_scene.frac 不生效**（第 3 轮 #5）
  - 核实：属实。抽样只用 `replay.frac`；`target_scene.frac`（0.667）是同一件事的第二处表示
  - 处理：`load_recipe` 核两者之和为 1，不一致即报错。测试 `test_target_and_replay_frac_agree`

### 引用抽查

- 「核实属实」：8 条（含上一轮三处修复）
- 无法核实：同第 2 轮

### 是否再审

连续第 1 轮无 P0/P1，开第 4 轮确认。全量 138 条全过（实测）。

## 第 4 轮（2026-11-25）· 总评「修改」

原文：`_tmp/reviews-traindata/exp025-data-codex-4.md`（不入库）。effort high。
上一轮处理的验证：codex 认 #3–#5 已落实，反例测试已核；#1、#2 是 issue。
本轮 P0/P1：0 条 · 停止线：**满足**（第 3、4 轮连续无 P0/P1）· 留作 issue 的 P2：#1、#2

- **P2｜分片内容未核**（第 4 轮 #1）：issue ①
- **P2｜重切流程没打通**（第 4 轮 #2）：issue ②
- **P2｜rebuild_exp013.py:102 上传核对的预期清单全信本机 hashes.txt**（第 4 轮 #3）
  - 核实：属实（本机清单与日志同时漏掉 480 条早期混响轨，以前照样过）
  - 处理：加「本机与云上 build 的文件清单相同」一项（两份同代码独立生成），判据升为第 4 版；本机重出报告 `rebuild/exp013-20261124-200416.json`，全过。测试 `test_local_hash_list_must_match_cloud_list`（先红后绿）
- **P2｜tests/test_products.py:47 切分摘要的统计字段被改不报红**（第 4 轮 #4）
  - 核实：属实（正文哈希不覆盖摘要字段）
  - 处理：`test_tables_match_rules` 用 `split.summary` 由正文重算 files · units · per_split · time · seconds，逐项比对

### 是否再审

停。第 4 轮之后的两处改动各带测试，没有再交 codex。全量 139 条全过（实测）。
