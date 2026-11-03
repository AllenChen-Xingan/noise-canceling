# 评审 011 · schema 导出机制与第一批 schema（codex · gpt-6-astra · read-only）

> 对象：`git diff bfe6266...HEAD`（分支 `feat/schema-crosscheck`，issue #57；diff 模板，附 `docs/plan/011_schema-crosscheck.md` 与重点文件）。**最新一轮：第 1 轮（2026-11-02）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×2、P2×3，逐条回到源码核实，5 条全部属实，全部已改。codex 抽查引用时列出的 1 条「不符」也属实，并入第 1 轮 #4 一起改了。新冒出的需要人定的事：没有。
> 作者：手下 schema。评审期间主目录有计时基准锁（`_tmp/heavy.lock`），codex 只读代码、没有编译；改完的代码等锁解除后再跑全量 CI。

## 第 1 轮（2026-11-02）· 总评「修改」

原文：本 worktree 的 `_tmp/20261102-schema-review/diff-bfe6266-head-codex-1.md`（不入库）。effort medium，耗时 91 秒（脚本末行：`result: P0×0 P1×2 P2×3 P3×0`）。行号指评审时的版本。
本轮 P0/P1 共 2 条 · 停止线：未满足 · 留作 issue 的 P2：无（3 条都已改）。隐私扫描干净；`src/eval/suite/baseline-ci.json` 是纯数值表，用 `--exclude` 没发给 codex。

- **P1｜src/schema/src/lib.rs:154 登记表丢了会自动重建，能绕过「必须升版本」**（第 1 轮 #1）
  - 核实：属实。改布局、不升 `VERSION`、删掉 `shm-header.versions.json` 再导出，`write` 会按当前布局重新登记，`check` 随后通过
  - 处理：删掉这份 json 登记表，版本历史改成 `src/schema/src/shm.rs` 的常量 `HISTORY`（导出工具从不改它）。守卫要求：版本号严格递增；当前 (版本, 指纹) 必须正是最后一行。否则 `write` 一个文件都不写、`check` 红，提示「升 VERSION 并在 HISTORY 末尾加 (v, 指纹)」。改写旧行仍做得到，但那是一处显眼的代码改动，评审能看到。codex 建议的「CI 对照 main 上的登记表」没做：CI 检出深度是 1，拿不到 main。负例覆盖：没升版本、升了没登记、版本往回降、历史乱序、空历史、端到端 write 拒写
- **P1｜src/eval/src/main.rs:93 基线文件格式不对时被静默当成空基线**（第 1 轮 #2）
  - 核实：属实。改成严格反序列化后，缺 `doc` / `commit` / `run_id` 或多一个键，`.ok()` 都会把它变成空基线，门禁就不再和基线比
  - 处理：`load_baseline` 返回 `Result`：文件不存在 = 空基线（同旧行为）；存在但格式不对 → 报错退出。解析放进 `BaselineFile::parse`。测试：values 合法、元数据坏的 3 例都报错，schema 也判不过
- **P2｜src/eval/src/suite.rs:153 评测缺省值还有没校验的第二处表示**（第 1 轮 #3）
  - 核实：属实。缺省数字一份在 `suite.rs` 的 `unwrap_or`，一份在 `files.rs` 的注释（会进 schema 的说明），改一处另一处不跟
  - 处理：缺省值收成 `files.rs` 的 `defaults` 常量（18 个）。`suite.rs` 补缺省值、schema 的 `default` 都从它来（transform 写入），注释里删掉数字。`defaults_unchanged` 测试补了干扰者、末尾静音、没写主讲人三种情况
- **P2｜src/schema/tests/crosscheck.rs:335 「类型变了」负例混进了无关差异**（第 1 轮 #4，含抽查「不符」一条）
  - 核实：属实。`ConfigOld` / `SuiteOld` 和真类型的标题、注释、字段都不同，就算字段漏了也会因为注释不同而报红
  - 处理：改成拿当前导出结果删掉一个字段（及 `required` 里的它）当作仓里的旧文件，其余不变；断言只报这一份、放回去就绿。共 4 例：Config 的 `denoise_block`、TaskFile 的 `xfail`、LevelsFile 的 `updated`、BaselineFile 的 `run_id`
- **P2｜src/schema/tests/crosscheck.rs:344 评测文件缺往返校验**（第 1 轮 #5）
  - 核实：属实。交代要求「反序列化再序列化与原文件等价」，原来只对 Config 做了
  - 处理：新测试 `eval_files_roundtrip_semantic`：4 份文件反序列化成各自的格式类型再写出，语义等价，再往返一次不变。口径：null 与不写等价，空数组与不写等价，数字按 f64 比。新测试 `eval_writer_outputs_pass_schema`：两个写入口序列化的类型，写出来的东西过 schema、能读回

### 引用抽查

- codex 列的「不符」：1 条（`crosscheck.rs:326` 说「多一个字段」，实际换成了只有 block 的类型），并入 #4 改了
- codex 列的「核实属实」：3 条
- 无法核实：1 条（CI 全绿是运行结果；codex 按要求没跑测试）。改完后的全量 CI 等锁解除再跑，结果记在现状页

### 是否再审

有 P1 → 锁解除、全量 CI 绿之后开第 2 轮，提示词附本记录。
