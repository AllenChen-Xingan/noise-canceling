# 现状页 · iface（批次 20261102-b9 第二阶段，issue #65）

> 手下 `iface`，worktree `../noise_canceling-iface`，分支 `feat/param-model-schema`。大总管 `noise-canceling-99`。
> 管的实体：参数描述（`desc.rs` 的导出）· 场景参数表 / 我的配置的文本格式 · 模型接口（`vdn-models`）。

## 做到哪
- [x] 读交代 · 报到（大总管同意顺序）
- [x] 读代码，定方案（见下）
- [x] 拆 `desc.rs`（930）→ `src/types/src/desc/{mod,speaker,polish,scene,engine}.rs`（最大 481 行），对外路径不变
- [x] 拆 `src/schema/tests/crosscheck.rs`（870）→ `schema_mechanism.rs` · `shm_layout.rs` · `config.rs` · `eval_suite.rs` + `common/mod.rs`（25 条测试一条不少，全过）
- [x] 拆 `src/xtask/src/single_source.rs`（789，8d 同意归我；人定按规则命名）→ `no_mirror.rs`（R-MIRROR）· `param_range.rs`（R-PARAM，按 `desc/` 目录读描述表）· `scan.rs`（共用扫描）；xtask 28 条测试全过，tidy 通过
- [ ] 拆 `policy.rs`（688）
- [ ] 全仓改 `single_source` 的文档引用
- [ ] 参数描述 → `specs/schema/params.table.json`
- [x] 我的配置越界拒收：先写测试（红）→ `memory.rs` 读值时按描述表 `check`（绿）
- [ ] 仓里文本文件过检查（示例文件 + 扫描）
- [ ] 模型接口：`vdn-models` 声明输入输出名与形状 → `specs/schema/models.interface.json`；ONNX 实读比对（`#[ignore]`，本机实跑）
- [x] FE-M 名字只留一个：正本 `ModelId::as_str` = `fastenhancer-m`，旧写法只在 `ModelId::LEGACY`/`parse`；cli（8d 同意）· denoised · eval 的 system/report 改成从 `ModelId` 取；tidy 新规则 R-MODEL（`src/xtask/src/model_name.rs`）
- [ ] `ModelChoice`（vdn-control 的 serde 名）↔ `ModelId::NAMES` 比对测试（放 vdn-schema）
- [ ] 全量 CI · eval ci 对比 · codex 评审 · PR 草稿

## 方案（我定的技术题，理由）
1. **参数描述导出**：`vdn-schema` 新模块 `params.rs`，把 `desc::ALL` 转成 `params.table.json`（不是 JSON Schema，按 README 规矩 1 命名为 `<实体>.<种类>.json`）。导出函数里对 `ParamDesc` 做穷举解构：描述加字段没导出 → 编译失败。浮点按 f32 最短写法写出（`0.002` 不变成 `0.0020000000949…`）。文件还写出「文本格式里能出现的参数键」（= `ParamField::ALL` 的键），由同一张表派生
2. **文本格式**：仓里目前**没有**已提交的场景参数表 / 我的配置文件（`git grep` 文件头为空）。加两份示例进 `specs/schema/examples/`，测试遍历仓里所有带这两种文件头的已跟踪文件，逐个解析（越界、未知键即红）。以后谁提交这类文件都会被查到
3. **模型接口**：声明（采样率 · hop · 延迟 · 输入输出名与形状）放 `vdn-models` 的 `ModelKind`，适配器打开文件时按声明核对（替换 gtcrn.rs 里手写的名字表）。导出 `models.interface.json`；CI 比对声明 ↔ 导出文件；ONNX 实读 ↔ 声明写成 `#[ignore = "要仓外权重"]`
4. **FE-M 名字正本 = `fastenhancer-m`**（连字符）。理由：命令行、配置 JSON、评测结果文件（`baseline-ci.json` 81 个键）、denoised 都用它；下划线写法只出现在 `ModelId::as_str` 一处，仓里没有用它写的场景表 / 会话记录文件。旧写法 `fastenhancer_m` 只在 `ModelId::parse` 一处兼容

## 转交 / 等大总管
- cli 已删 `ModelArg`（#62 把它搬到 args.rs，合并冲突由 8d 处理）
- tidy.rs 只改了调用处 3 行 + 文档表 1 行

## 待拍板
（无）
