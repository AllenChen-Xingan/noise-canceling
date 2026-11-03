# 评审 011 · schema 导出机制与第一批 schema（codex · gpt-6-astra · read-only）

> 对象：`git diff bfe6266...HEAD`（分支 `feat/schema-crosscheck`，issue #57；diff 模板，附 `docs/plan/011_schema-crosscheck.md` 与重点文件）。**最新一轮：第 5 轮（2026-11-02）· 总评「修改」· 停止线满足（第 4、5 轮连续无 P0/P1），评审结束**。
> 累计处理：P1×5、P2×11，逐条回到源码核实，16 条全部属实；15 条已改，1 条（第 5 轮 #1）写进 README、留作 issue。共享内存版本守卫前后改了三轮：第 1 轮改成代码常量，第 2 轮加 main 基线，第 3 轮补齐基线的读取失败、多提交推送和整份历史。codex 抽查引用时列出的 1 条「不符」也属实，并入第 1 轮 #4 一起改了。新冒出的需要人定的事：没有。
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

第 2 轮改动的验证（2026-11-02 锁解除后，实测）：`xtask ci` 全过（fmt · clippy · test · tidy · check-deps · cargo-deny），`vdn-schema --check` 一致，`xtask eval ci` 门禁通过（ref 回归 13/17 · 能力 1/10 · 探针 5/5，与前同）。`same_keeps_integers_exact` 第一次跑红：手写样例没写 `defaults`，往返写出多一个空对象 `{}`，错在样例；样例补上 `defaults` 后转绿，比较口径不变。

## 第 3 轮（2026-11-02）· 总评「修改」

原文：`_tmp/20261102-schema-review/diff-bfe6266-head-codex-3.md`（不入库）。gpt-6-astra，effort medium，耗时 72 秒（`result: P0×0 P1×2 P2×2 P3×0`）。第 2 轮输出同样以 `P1` 开头，照第 2 轮的办法加标题行另存为 `round2-previous.md` 传入；第 2 轮本有 P1，停止线判断不受影响。
本轮 P0/P1 共 2 条 · 停止线：未满足 · 留作 issue 的 P2：无。
上一轮处理的验证：#2 总线名、#3 往返比较，codex 抽查确认已落实；#1 版本守卫还有三处口子（本轮 #1–#3）。
改动期间主目录又有计时基准锁，代码先写好，锁解除后再编译、重新导出、跑全量 CI。

- **P1｜src/schema/src/lib.rs:214 读不到基线被当成「首次引入」**（第 3 轮 #1）
  - 核实：属实。`git show` 的任何失败（提交不在、读错）都返回 `Absent`，测试只打印提示，CI 里也不红
  - 处理：新函数 `shm_baseline_at`：先确认基线提交存在，再用 `git ls-tree` 看树里有没有这份文件。只有「提交在、树里确实没有」才算 `Absent`；提交不存在、读树或读文件出错一律 `Unavailable`（CI 里红）。负例 `shm_baseline_failures_are_not_absent`：不存在的提交、全零提交 → Unavailable；仓库第一个提交 → Absent；HEAD → Found
- **P1｜src/schema/src/lib.rs:204 main 上 push 时基线可能是同批未验证的提交**（第 3 轮 #2）
  - 核实：属实。一次推两个提交，第一个改布局并改写同版本的 `HISTORY`，CI 只在最后一个提交上跑、取 `HEAD^1`，正好拿到被改过的布局
  - 处理：`.github/workflows/ci.yml` 的 rust gate 一步加环境变量 `VDN_SCHEMA_BASE`：push 用 `github.event.before`（推送前的 main），PR 用 `github.event.pull_request.base.sha`。`shm_baseline` 先读它，没有才按本机规则（分叉点 / `HEAD^1`）。全零（新建分支推送）算 Unavailable。测试经 `shm_baseline_from` 传入，验证指定的基线优先于本机规则。多提交推送本身没法在单元测试里重演；测试只验证指定的基线优先于本机规则
- **P2｜src/schema/src/shm.rs:144 「只增不改」只检查了基线那一个版本**（第 3 轮 #3）
  - 核实：属实。main 到版本 3 后，改写或删掉版本 2 的行仍能通过
  - 处理：完整历史写进导出的 `shm-header.layout.json`（`history` 数组，来自 `HISTORY`）；main 上那份文件就成了已发布历史的可信记录。`guard_baseline` 要求基线的整份历史原样是当前 `HISTORY` 的前缀：改写报「被改写」，删掉报「删掉」，在已发布行之间插行报「插进」；基线自己的布局指纹不在它的历史里报「自相矛盾」。没有 `history` 的旧格式按只有当前一行算。负例 `shm_history_whole_prefix_is_checked`。README 与现状页的说法同步改了
- **P2｜specs/schema/eval-suite.schema.json:619 整数字段没有上界**（第 3 轮 #4）
  - 核实：属实。schemars 对 `u64` 只写 `minimum: 0` 和 `format: uint64`，校验器不按 format 判范围，`seed` 超出 u64 时 schema 放行、serde 拒
  - 处理：导出时统一补界（`vdn_schema::int_bounds`，在 `with_comment` 里调）：按 `format` 填 Rust 类型的 `minimum` / `maximum`，transform 写入的更窄范围不动。测试 `integer_bounds_agree_with_rust_types`：四份 schema 里每个整数字段都有上下界；`seed` 取 u64::MAX 两边都过，取 1e20、-1 两边都拒

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条
- 无法核实：1 条（CI 结果是运行结果）

### 是否再审

有 P1 → 锁解除、全量 CI 绿之后开第 4 轮。

第 3 轮改动的验证（实测）：重新导出（布局文件加 `history`、评测清单 schema 整数加上下界），`cargo test -p vdn-schema` 24 条全过，`xtask ci` 全过（含合进来的新 tidy 规则），`vdn-schema --check` 一致。大总管在 6b276e1 上亲手复跑：`xtask ci` 全绿（400 过 / 0 败 / 5 忽略）、`--check` 一致、`eval ci` 门禁通过。

## 第 4 轮（2026-11-02）· 总评「修改」

原文：`_tmp/20261102-schema-review/schema-branch-codex-1.md`（不入库）。gpt-6-astra，effort medium，耗时 71 秒（`result: P0×0 P1×0 P2×2 P3×0`）。
**对象范围改了**：大总管把 main（含 PR #60）合进本分支后，`bfe6266...HEAD` 会带进 264 个文件，其中大部分是别人的改动。本轮起改审 `origin/main...HEAD`，只有本分支的 41 个文件；大总管已同意。脚本不许同名换对象，所以本轮在脚本里叫 `schema-branch` 第 1 轮。第 3 轮原文照旧加标题行另存为 `round3-previous.md` 传入。一开始误用旧范围起跑，发现后中途停掉，那次没有产出。
本轮 P0/P1 共 0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作 issue 的 P2：无（两条都改了，大总管定）。
上一轮处理的验证：#1–#4 codex 抽查确认均已落实（6 条「核实属实」）。

- **P2｜src/schema/tests/crosscheck.rs:215 旧测试写死了只有一个版本**（第 4 轮 #1）
  - 核实：属实。以后正常追加版本 3 并重新导出后，「改写」样例只剩版本 3，守卫先报「删掉版本 2」，断言「被改写」就失败；追加正例同样漏了版本 2
  - 处理：改写、删除、追加都从完整 `HISTORY` 复制后改一行；另加一个已发布两个版本的正例
- **P2｜src/schema/tests/crosscheck.rs:290 读基线失败的测试挡不住回退**（第 4 轮 #2）
  - 核实：属实。现有负例在 `rev-parse` 一步就返回，没有走到读树 / 读文件；把那两处改回 `Absent`，测试照样过
  - 处理：调 git 的那一步做成参数（`shm_baseline_with`），`shm_baseline_at` 传真的 git。新测试 `shm_baseline_read_errors_are_unavailable` 用假 git 逐步注入：读树失败、树里有文件但读失败 → Unavailable；树里确实没有 → Absent；读到 → Found。**变异验证**：临时把读树失败改回 `Absent`，这条测试红；改回后绿
  - 改后实测：`cargo test -p vdn-schema` 25 条全过；`xtask ci` 全过；`--check` 一致；`eval ci` 门禁通过（ref 回归 13/17 · 能力 1/10 · 探针 5/5，与前同）

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：6 条
- 无法核实：1 条（CI 结果是运行结果）

### 是否再审

本轮无 P0/P1 → 开第 5 轮确认。第 4 轮原文以 `P2` 开头，`file` 把它认成灰度图（PGM 的文件头恰好是 `P2`），只能加标题行另存为 `round4-previous.md` 传入。这样脚本认不出它是同一对象的上一轮，`stop-line` 行会写「连续第 1 轮」；连续两轮由人工按本记录判断。

## 第 5 轮（2026-11-02）· 总评「修改」

原文：`_tmp/20261102-schema-review/schema-branch-codex-2.md`（不入库）。gpt-6-astra，effort medium，耗时 49 秒（`result: P0×0 P1×0 P2×2 P3×0`）。对象 `origin/main...HEAD`，第 4 轮原文以 `round4-previous.md` 传入。
本轮 P0/P1 共 0 条 · **停止线：满足**（人工判断：第 4 轮、第 5 轮都没有 P0/P1；脚本的 `stop-line` 行写「连续第 1 轮」，原因见第 4 轮「是否再审」）· 留作 issue 的 P2：1 条（#1）。
上一轮处理的验证：#1、#2 codex 抽查确认已落实；生产入口仍走真 git。

- **P2｜src/eval/src/files.rs:28 整数的写法：schema 收 3.0，serde 不收**（第 5 轮 #1）
  - 核实：属实。JSON Schema 把 `3.0` 算整数；serde 读 `u64` 只收不带小数点的写法。`{"seed": 3.0}` 过 schema、解析报错；Config 的 `block` 同理
  - 处理：**留作 issue**。方向是 schema 宽、Rust 严，坏数据进不来；要改就得给所有整数字段换反序列化器，这是行为改动，超出本轮。已写进 `specs/schema/README.md`「schema 不管什么」；issue 草稿在 `_tmp/20261102-schema-review/issue-int-float.md`，交大总管开
- **P2｜src/schema/tests/crosscheck.rs:858 冻结基线的负例没测到真正的读入口**（第 5 轮 #2）
  - 核实：属实。负例只调 `BaselineFile::parse`；把 `load_baseline` 改回「出错就返回空基线」，测试照样过
  - 处理：`load_baseline` 拆出 `load_baseline_from(路径)`。新测试 `load_baseline_only_missing_file_is_empty`（`src/eval/src/main.rs`），覆盖以下几种情况：
    - 文件不存在：返回空基线
    - 仓里那份基线：能读出来，且不为空
    - 内容不是 JSON、缺元数据、多一个键：都报错
    - 读取失败（路径是目录）：报错

### 引用抽查

- codex 列的「不符」：无
- codex 列的「核实属实」：5 条
- 无法核实：1 条（CI 与变异实测是运行结果）

### 结论

停止线满足，评审结束。最后一轮的改动只补测试、加一个读入口参数，不再送审。全量 CI 结果记在现状页。
