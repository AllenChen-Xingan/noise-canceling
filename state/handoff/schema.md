# 现状页 · schema（批次 20261102-b9，issue #57）

> 手下 `schema`，worktree `../noise_canceling-schema`，分支 `feat/schema-crosscheck`。大总管 `noise-canceling-99`。

## 做到哪
- [x] 读交代 · 报到；机制已报大总管（文件命名 + 导出命令）
- [x] 机制：`vdn-schema` crate（tool 档，`src/schema/`）· `cargo run -p vdn-schema [-- --check]` · 逐字比对测试（随 `cargo test`）
- [x] 共享内存头：`vdn_shm::desc` 字段表 → `shm-header.layout.json`；版本登记表 `shm-header.versions.json`（改布局不升版本 → 红）
- [x] Config：`config.schema.json`；示例 `specs/schema/examples/config.*.json`；往返测试；范围与总线名逐例比对
- [x] 评测清单：`vdn_eval::files` 类型成为格式正本（解析器与两个写入口都用它）；3 份 schema；4 份文件过 schema；`task-schema.md` 改成指针
- [x] 规约 ADR 004 表 · ADR 004 修订注 · cog.md 一行 · specs/README.md 子目录表
- [x] 全量 CI（`xtask ci`）全绿；`xtask eval ci` 门禁通过（ref：回归 13/17、能力 1/10、探针 5/5）
- [x] codex 第 1 轮（P1×2 P2×3，全部属实、全部已改，记录 `docs/reviews/011_schema-crosscheck-codex.md`）
- [x] 全量 CI（第 1 轮改动后）全绿；`xtask eval ci` 门禁通过
- [x] codex 第 2 轮（P1×1 P2×2，全部属实、已改：对照 main 的版本历史守卫 · 总线名只许可见 ASCII · 往返比较整数精确）
- [ ] 锁解除后：全量 CI → codex 第 3 轮

## 技术决定（我定，理由）
- 导出库 `schemars` 1.2（MIT）：派生自 serde 属性，Rust 类型即正本。加进 control · eval · schema 三个 C / tool 档 crate
- 校验库 `boon` 0.6（MIT OR Apache-2.0）：只做 `vdn-schema` 的测试依赖。它和导出器是两家实现，能抓到 schema 与 serde 实际行为不一致
- 依赖体积（Cargo.lock 新增）：schemars 一侧 6 个 crate（schemars · schemars_derive · serde_derive_internals · dyn-clone · ref-cast · ref-cast-impl）；boon 一侧约 40 个（url / idna / icu 一族 · regex · ahash 等），只在测试构建里。cargo-deny 许可检查通过，没加新许可
- 共享内存头：核心 crate 不引 serde，字段表在 `src/shm/src/desc.rs`，偏移全由 `offset_of!` 算；小宏穷举解构，漏字段或类型写错就编译失败（已实测两种都编译不过）。`layout.rs` 里手写偏移数字的编译期断言删了，改由导出文件和测试钉住；结构性约束（每行 64 字节、wake 按 4 对齐等）保留
- 版本守卫：布局指纹（FNV-1a 64）登记在 `src/schema/src/shm.rs` 的常量 `HISTORY`（codex #1 后从 json 文件改来，导出工具不改它）。当前 (版本, 指纹) 必须是最后一行，否则 `write` 一个文件都不写、测试红
- 缺省值：评测清单的缺省值正本是 `files.rs` 的 `defaults` 常量，解析与 schema 的 `default` 都读它（codex #3）
- 基线文件存在但格式不对 → `vdn-eval ci` 报错退出，不再当空基线（codex #2）
- Config 往返的口径（语义等价）：① 原文件写了的键，写出后值相同（按 JSON 值比，不比空白、键顺序）；② 没写的键等于默认值；③ 写出再读回得到同一个 `Config`
- 评测清单改成 `deny_unknown_fields`：多出的键报错（原来静默忽略）。任务里的 `note` 收进类型。`pause_s` / `f0` 不是两个数的数组时，原来静默取默认值，现在报错
- 枚举名字只由 serde 定：删了 `NoiseKind::parse` · `InterfMode::parse` · `Probe::parse` · `Level::parse` · `Set::parse`；保留的 `as_str` 由测试与 serde 名字逐个比对

## 大总管要核的三件（2026-11-02，读 cargo tree / 源码，未编译）
1. 许可与依赖范围：`cargo deny check` 在锁建立前全过（licenses ok，没加新许可）。`cargo tree -e normal -i boon` 为空，boon 只在 vdn-schema 的 dev 依赖里
2. 宏：`describe!` 16 行，没有 `#[macro_export]`，不导出，符合规约 §6「≤30 行、不导出」
3. 核心 crate：types · dsp · engine · models · shm · speaker 的 forbid 都含 `serde`。schemars 依赖 serde，闭包检查同样挡住它；check-deps 通过
- 顺带改了一处（锁解除后待编译验证）：schemars 原经 vdn-control 进了 vdn-capture · vdn-denoised · vdn 三个产品程序的依赖。现改为 vdn-control 的可选特性 `schema`，只有 vdn-schema 打开

## 改了哪些文件
- 新：`src/schema/`（Cargo.toml · src/lib.rs · src/shm.rs · src/eval.rs · src/main.rs · tests/crosscheck.rs）· `src/shm/src/desc.rs` · `src/eval/src/files.rs` · `specs/schema/`（README + 5 份导出 + examples/2 份）· `docs/reviews/011_schema-crosscheck-codex.md`
- 改：`Cargo.toml` · `Cargo.lock` · `src/shm/src/{lib,layout}.rs` · `src/shm/tests/layout_doc.rs` · `src/control/{Cargo.toml,src/lib.rs}` · `src/eval/{Cargo.toml,src/lib.rs,src/suite.rs,src/main.rs,src/synth.rs,src/probes.rs}` · `specs/rust/architecture.md`（§6 表一行）· `specs/README.md` · `.42cog/cog.md`（一行）· `docs/research/008_rust-architecture/adr/004-ast-guard-not-generate.md`（修订注）· `docs/experiments/exp007-eval-suite/task-schema.md`

## 加了哪些测试
- `src/schema/tests/crosscheck.rs` 16 条：仓里文件一致 · 任一导出过期报红 · 孤儿文件报红 · 共享内存头逐项比对 + 3 个锚点直接用 `offset_of!` · 改布局不升版本报红（含 write 拒写）· 版本已登记 · Config 示例过 schema · 往返 · 默认示例 = `Config::default()` · 范围边界 20 例与 `validate` 一致 · 总线名 11 例与 `validate_bus_name` 一致 · Config 加字段不导出报红 · 4 份清单过 schema · suite 目录 json 全登记 · 坏任务 10 例 schema 与解析器判定一致 · 清单类型改了不导出报红
- `src/eval/src/files.rs` 3 条：`as_str` 与 serde 名字一致 · `ALL` 列全 · 缺省值不变
- `src/shm/src/desc.rs` 1 条：字段有序、不重叠、不越界

## 转交
- **`.github/workflows/ci.yml` 改了一行**：checkout 加 `fetch-depth: 0`（vdn-schema 的测试要对照 main 上的布局文件，codex 第 2 轮 #1）。这是共用文件，合并冲突请大总管处理
- **总线名规则收窄**（我的实体 vdn-shm / vdn-control，codex 第 2 轮 #2）：前缀之后只许可见 ASCII、不含 `/`，空格和非 ASCII 字符不再许可。大总管让改动方补测试：已给 vdn-denoised 的 `bus_name_rule_matches_vdn_shm` 加 5 个样例（空格、中文、9 个中文、末尾 /、可见符号），只加测试、没改逻辑。PR 正文要写这条行为变化

## 待拍板
（暂无，全是技术题，已自定）

## 小失误（如实记）
- 删旧登记表时用了 `git rm`（应该用普通 `rm`，规矩 3：手下不做 git 写操作）。效果只是把删除放进暂存区，随后由 hook 提交，没有别的影响
