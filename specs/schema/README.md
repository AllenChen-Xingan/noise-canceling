# specs/schema：结构的机读描述（由 Rust 导出，勿手改）

> 依据：CLAUDE.md 第五节第 5 条「唯一真相源 + 交叉校验」· `docs/plan/011_schema-crosscheck.md` · 规约 `specs/rust/architecture.md` ADR 004 那张表。

## 一句话

**Rust 类型是正本。** 这里的文件全由 `cargo run -p vdn-schema` 从 Rust 类型导出、提交进仓。
`vdn-schema` 的测试重新导出并逐字比对，不一致即红（随 `cargo test --workspace` 与 `cargo run -p xtask -- ci` 跑）。

## 现有文件

| 文件 | 正本（Rust） | 管哪些数据文件 | 交叉校验（测试在 `src/schema/tests/crosscheck.rs`） |
|---|---|---|---|
| `shm-header.layout.json` | `src/shm/src/layout.rs` 的结构，经 `src/shm/src/desc.rs` 的字段表 | 跨进程的共享内存（给虚拟麦克风 / C 侧对表） | 每个字段的名字 · 偏移 · 大小与 `offset_of!` / `size_of` 逐项比对；版本号 = `layout::VERSION`；布局指纹必须等于 `src/schema/src/shm.rs` 的 `HISTORY` 最后一行——改布局不升版本、或升了不登记，导出拒绝写、测试红（历史故意写在代码里，导出工具不改它）；完整历史也写进导出文件；测试读可信基线（CI：push 用推送前的提交、PR 用目标分支，经环境变量 `VDN_SCHEMA_BASE` 传入；本机：与 main 的分叉点）上已提交的布局文件，要求它的整份历史原样是 `HISTORY` 的前缀，只许在末尾追加。基线提交不存在或读不出 → CI 红；只有基线的树里确实没有这份文件才跳过（CI 要 `fetch-depth: 0`） |
| `config.schema.json` | `src/control/src/lib.rs` 的 `Config` | `examples/config.*.json`（以后的用户配置同格式） | 示例过 schema；往返语义等价；范围边界与 `Config::validate` 逐例一致；总线名正则与 `validate_bus_name` 逐例一致（名字只许可见 ASCII，字符数 = 字节数） |
| `eval-suite.schema.json` | `src/eval/src/files.rs` 的 `SuiteFile`（缺省值正本是同文件的 `defaults` 常量，解析与 schema 的 `default` 都读它） | `src/eval/suite/ci.json` · `qa.json` | 文件过 schema；往返语义等价；坏任务（拼错键、未知 verifier、未知噪音…）schema 与解析器判定一致 |
| `eval-levels.schema.json` | 同上 `LevelsFile` | `src/eval/suite/levels.json` | 文件过 schema；`calib --write-levels` 序列化同一类型 |
| `eval-baseline.schema.json` | 同上 `BaselineFile` | `src/eval/suite/baseline-ci.json` | 文件过 schema；`ci --write-baseline` 序列化同一类型 |

## 命令

```sh
cargo run -p vdn-schema             # 改了类型后：重新导出（只写有变化的文件），再提交
cargo run -p vdn-schema -- --check  # 只比对，不写；不一致退出码 1
cargo test -p vdn-schema            # 全部交叉校验
```

## 规矩（新加一份 schema 照这个做）

1. **命名**：`<实体>.schema.json`（JSON Schema）；不是 JSON Schema 的描述用 `<实体>.<种类>.json`（如 `shm-header.layout.json`）。全小写、连字符，不带版本号（版本走提交链；跨进程布局的版本号写在文件内容里）
2. **有 serde 的类型**（C 档、tool 档）：派生 `schemars::JsonSchema`，draft 2020-12。范围、枚举等要写进 schema 的数字，用 `#[schemars(transform = …)]` 从 Rust 常量写进去，不在属性里另写数字
3. **核心 crate（A / B 档）不引 serde、不引 schemars**（规约 §2.2 规则 1）：在本 crate 写一张常量字段表（照 `vdn_shm::desc`：小宏穷举解构，漏字段或类型写错编译失败），由 `vdn-schema` 转成 JSON
4. **登记**：在 `src/schema/src/lib.rs` 的 `export()` 里加一行；有数据文件的，在对应模块的 `DATA_FILES` 登记（目录里没登记的 json 测试会报）
5. **测试**：每份 schema 至少一条正例（仓里文件一致 / 数据文件过 schema）和一条负例（改了 Rust 不重新导出 → 红）。数据格式有第二处表示（文档里的字段表、`as_str` 名字表）就加比对，或删掉第二处改成指针
6. **校验器**用 `boon`（只在测试里），与导出用的 `schemars` 是两家实现，能抓到「schema 与 serde 实际行为不一致」
7. **整数字段**：导出时按 schemars 写的 `format`（`uint64` 等）补上 Rust 类型的 `minimum` / `maximum`（`int_bounds`），已有的更窄范围不动。校验器不按 format 判范围，不补的话超出类型的数 schema 放行、serde 拒
8. 导出文本的键顺序依赖 `serde_json` 不开 `preserve_order`（默认有序 Map）。哪天开了，导出会整体变化、测试会红，届时重导一次即可

## schema 不管什么

schema 只管单个字段的格式与范围。跨字段规则（如 `lead_frames ≥ block`、两个总线名不同、探针的基础任务存在、任务 id 唯一）只在 Rust 的校验函数里，schema 判「过」不等于能用。
