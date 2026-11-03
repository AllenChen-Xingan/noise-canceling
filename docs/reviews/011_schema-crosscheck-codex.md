# 评审 011 · schema 导出机制与第一批 schema（codex · gpt-6-astra · read-only）

> 对象：`git diff bfe6266...HEAD`（分支 `feat/schema-crosscheck`，issue #57；diff 模板，附 `docs/plan/011_schema-crosscheck.md` 与重点文件）。**最新一轮：第 2 轮（2026-11-02）· 总评「修改」· 待第 3 轮**。
> 累计处理：P1×3、P2×5，逐条回到源码核实，8 条全部属实，全部已改。第 1 轮 #1 只改了一半：第 2 轮 #1 指出它仍能被绕过，第 2 轮补全。codex 抽查引用时列出的 1 条「不符」也属实，并入第 1 轮 #4 一起改了。新冒出的需要人定的事：没有。
> 作者：手下 schema。评审期间主目录有计时基准锁（`_tmp/heavy.lock`），codex 只读代码、没有编译；改完的代码等锁解除后再跑全量 CI。

## 第 1 轮（2026-11-02）· 总评「修改」

原文：本 worktree 的 `_tmp/20261102-schema-review/diff-bfe6266-head-codex-1.md`（不入库）。effort medium，耗时 91 秒（脚本末行：`result: P0×0 P1×2 P2×3 P3×0`）。行号指评审时的版本。
本轮 P0/P1 共 2 条 · 停止线：未满足 · 留作 issue 的 P2：无（3 条都已改）。隐私扫描干净；`src/eval/suite/baseline-ci.json` 是纯数值表，用 `--exclude` 没发给 codex。

- **P1｜src/schema/src/lib.rs:154 登记表丢了会自动重建，能绕过「必须升版本」**（第 1 轮 #1）
  - 核实：属实。改布局、不升 `VERSION`、删掉 `shm-header.versions.json` 再导出，`write` 会按当前布局重新登记，`check` 随后通过
  - 处理（第 1 轮，**不完整**，见第 2 轮 #1）：删掉这份 json 登记表，版本历史改成 `src/schema/src/shm.rs` 的常量 `HISTORY`（导出工具从不改它）。守卫要求：版本号严格递增；当前 (版本, 指纹) 必须正是最后一行。否则 `write` 一个文件都不写、`check` 红，提示「升 VERSION 并在 HISTORY 末尾加 (v, 指纹)」。改写旧行仍做得到，但那是一处显眼的代码改动，评审能看到。codex 建议的「CI 对照 main 上的登记表」没做：CI 检出深度是 1，拿不到 main。负例覆盖：没升版本、升了没登记、版本往回降、历史乱序、空历史、端到端 write 拒写
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

## 第 2 轮（2026-11-02）· 总评「修改」

原文：`_tmp/20261102-schema-review/diff-bfe6266-head-codex-2.md`（不入库）。effort medium，耗时 103 秒（`result: P0×0 P1×1 P2×2 P3×0`）。
第一次发送被隐私扫描拦下：第 1 轮输出以 `P1 #1` 开头，`file` 命令把它误认成 PBM 位图（PBM 的文件头恰好是 `P1`），判成「不是纯文本」。改法：复制一份、开头加一行标题，再作 `--previous` 传入。第 1 轮本来就有 P1，所以停止线判断不受影响。
本轮 P0/P1 共 1 条 · 停止线：未满足 · 留作 issue 的 P2：无。

上一轮处理的验证：#2 · #3 · #4 codex 抽查确认已落实；#1 仍可绕过（本轮 #1）；#5 的比较口径有漏洞（本轮 #3）。

- **P1｜src/schema/src/shm.rs:109 第 1 轮 #1 的版本守卫仍可绕过**（第 2 轮 #1）
  - 核实：属实。把 `HISTORY` 唯一一行改成 `(2, 新指纹)`、不升 `VERSION`，守卫与检查都通过；「评审看得见」不是机器校验
  - 处理：加可信基线。`vdn_schema::shm_baseline` 取本分支与 main 的分叉点（就在 main 上时取上一个提交），读出那里已提交的 `shm-header.layout.json`，算出 (版本, 指纹)。`guard_baseline` 要求 `HISTORY` 里原样有这一行：改写报「被改写」，删掉报「删掉」，只许在末尾追加。CI 的 checkout 改成 `fetch-depth: 0`，才拿得到 main；CI 里拿不到基线算红，本机拿不到只提示。负例：改写、删除红，追加绿；另核从文件算的指纹 = 从 Rust 算的指纹。限制：本机制第一次合并前，main 上还没有这份文件，对照会跳过（`Absent`），合并后才生效
- **P2｜src/control/src/lib.rs:228 总线名长度：schema 数字符、Rust 数字节，多字节名字两边结论不同**（第 2 轮 #2）
  - 核实：属实。`/vdn-` 加 9 个「中」：14 个字符、32 字节，schema 放行，Rust 拒绝
  - 处理：三处一起收窄，前缀之后只许可见 ASCII（`!`–`~`）且不含 `/`。三处是：vdn-shm 的 `checked_name`（正本）、vdn-control 的 `validate_bus_name`、schema 的正则 `^/vdn-[!-.0-~]+$`。只有 ASCII 时，字符数 = 字节数。测试：交叉校验加 `/vdn-中`、9 个「中」、`/vdn-~!`；vdn-shm 的 `bad_names_are_rejected` 加空格、NUL、多字节三例。行为变化：名字里不再许有空格和非 ASCII 字符（默认名 `/vdn-bus` `/vdn-out` 不受影响）
- **P2｜src/schema/tests/crosscheck.rs:473 往返比较把所有数字转成 f64，会吞掉整数变化**（第 2 轮 #3）
  - 核实：属实。9007199254740992 与 9007199254740993 转 f64 后相等，而 `seed` 是 u64
  - 处理：比较改成 `same`：两边都是整数就精确比，有一边是小数才按 f64 比。新测试 `same_keeps_integers_exact`：相邻大整数判不等、3 与 3.0 判等、大整数种子往返不丢精度

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条
- 无法核实：1 条（CI 结果是运行结果）。本轮改动在计时基准锁期间写成，锁解除后跑全量 CI，结果记在现状页

### 是否再审

有 P1 → 全量 CI 绿之后开第 3 轮。
