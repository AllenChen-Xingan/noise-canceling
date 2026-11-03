## 做了什么

建立 schema 机制：Rust 类型是正本，导出到 `specs/schema/` 并提交进仓。测试会重新导出并逐字比对，不一致即红。第一批 schema 覆盖 3 个实体：

- **机制**：新工具 crate `vdn-schema`（`src/schema/`，tool 档），不改 xtask「不依赖任何 vdn crate」的约定。`cargo run -p vdn-schema` 负责导出，加 `-- --check` 只比对。说明在 `specs/schema/README.md`，内容包括命名、登记和规矩
- **共享内存头**（vdn-shm，版本 2）：字段表写在 `src/shm/src/desc.rs`，偏移和大小全由 `offset_of!` / `size_of` 算出。小宏对每个结构穷举解构，漏写字段或类型写错都编译失败。导出到 `shm-header.layout.json`
  - 版本历史 `HISTORY` 写成代码常量，改布局不升版本就红；完整历史也导出进布局文件
  - 测试还对照可信基线上已提交的布局文件，要求它的整份历史原样是 `HISTORY` 的前缀（只许末尾追加）。基线在 CI 里由环境变量 `VDN_SCHEMA_BASE` 给：push 用推送前的提交，PR 用目标分支。基线读不出来 → 红，只有基线里确实没有这份文件才跳过
  - `layout.rs` 里手写偏移数字的编译期断言删了，结构性约束（每行 64 字节、wake 按 4 对齐等）保留
- **Config**（vdn-control）：派生出 `config.schema.json`，取值范围由 transform 从 `Config` 的常量写入；所有整数字段导出时都补上 Rust 类型的上下界。schemars 是 vdn-control 的可选特性 `schema`，产品程序不带。示例放在 `specs/schema/examples/`
- **评测任务清单**（vdn-eval，4 份 JSON）：新增格式类型 `src/eval/src/files.rs`，作为格式正本
  - 解析器和两个写入口（`ci --write-baseline`、`calib --write-levels`）都改用这些类型
  - 缺省值收成 `defaults` 常量，解析补缺省值和 schema 的 `default` 都从它取
  - 导出 3 份 schema；`docs/experiments/exp007-eval-suite/task-schema.md` 里的手写字段表改成指针
- 规约 `specs/rust/architecture.md` §6 表一行改了；ADR 004 加修订注；`.42cog/cog.md` crate 表加一行
- CI：`.github/workflows/ci.yml` 改两处：checkout 加 `fetch-depth: 0`；rust gate 一步加环境变量 `VDN_SCHEMA_BASE`

**行为变化（请留意）**
- **总线名不再允许空格和中文**：前缀 `/vdn-` 之后只许可见 ASCII（`!`–`~`），且不含 `/`。vdn-shm、vdn-control、schema 三处一起改，默认名 `/vdn-bus`、`/vdn-out` 不受影响
- 评测清单多出的键改为报错，原来是静默忽略。`pause_s` / `f0` 不是两个数时也报错，原来是悄悄取默认值。仓里 4 份文件都能过
- 冻结基线文件存在但格式不对时，`vdn-eval ci` 报错退出，不再当成空基线

## 关联 issue
Closes #57

## 测试（必填，CLAUDE.md 第五节第 6 条）
- 新增的测试：
  - `src/schema/tests/crosscheck.rs`（25 条）：
    - 机制：仓里文件与导出一致；任一导出过期报红；孤儿文件报红
    - 共享内存头：逐项比对，另有 3 个锚点直接用 `offset_of!` 核；改布局不升版本报红（含 write 拒写）；当前版本已登记；对照基线只增不改；改写、删除、插入旧行报红（含已发布多个版本）；基线读取失败不当成「首次引入」（假 git 注入读树、读文件失败）
    - Config：示例过 schema；往返语义等价；默认示例等于 `Config::default()`；范围边界 20 例、总线名 14 例都与 Rust 判定一致；加字段不导出报红
    - 评测清单：4 份文件过 schema；suite 目录的 json 全登记；坏任务 10 例 schema 与解析器判定一致；改类型不导出报红（3 份）；往返语义等价；写入口产物过 schema、坏元数据报错；整数精确比较；整数字段都有上下界，越界值 schema 与解析器都拒
  - `src/eval/src/main.rs`（1 条）：冻结基线只有「文件不存在」算空，内容坏、读不了都报错
  - `src/eval/src/files.rs`（3 条）：`as_str` 与 serde 名字一致；`ALL` 列全；缺省值不变（含干扰者）
  - `src/shm/src/desc.rs`（1 条）：字段有序、不重叠、不越界
  - `src/shm/tests/posix.rs`、`src/denoised/src/lib.rs`：总线名加空格、中文、多字节等负例
- 跑了哪些（命令 + 结果）：〔编译后填〕
- 测不了的（`#[ignore]` + 人工步骤）：无。对照基线那条测试在本机制第一次合并前会跳过，因为 main 上还没有布局文件，合并后生效
- 已知差异（留作 issue）：schema 把 `3.0` 算整数，serde 不收；方向是 schema 宽、Rust 严。写在 `specs/schema/README.md`

## 唯一真相源（CLAUDE.md 第五节第 5 条）
- 本次新增的第二处表示及其校验：
  - `specs/schema/*.json`（6 份导出）↔ Rust 类型：重新导出逐字比对
  - `shm-header.layout.json` ↔ `layout.rs` 结构：逐项比对 `offset_of!` / `size_of`；版本 ↔ `HISTORY` ↔ 可信基线上的布局文件（整份历史前缀）
  - `vdn_shm::desc` 字段表 ↔ 结构：穷举解构，编译期保证
  - `specs/schema/examples/config.default.json` ↔ `Config::default()`：测试比对
  - `bus_name_pattern()` 正则 ↔ `validate_bus_name` ↔ vdn-shm `checked_name`：逐例对拍
  - Config schema 的 min / max ↔ `Config` 常量 ↔ `validate`：边界逐例对拍
  - 枚举的 `as_str` ↔ serde 名字：逐变体比对；删掉了 5 个手写的 `parse` 函数
  - 评测清单缺省值：`files::defaults` 一处，解析与 schema 都读它
- 换谱系评审：codex 5 轮，记录在 `docs/reviews/011_schema-crosscheck-codex.md`。共提出 16 条（P1×5、P2×11），全部属实；15 条已改，1 条 P2 留作 issue。第 4、5 轮连续没有 P0/P1，停止线满足

🤖 Generated with [Claude Code](https://claude.com/claude-code)
