# 现状页 · iface（批次 20261102-b9 第二阶段，issue #65）

> 手下 `iface`，worktree `../noise_canceling-iface`，分支 `feat/param-model-schema`。大总管 `noise-canceling-99`。
> 管的实体：参数描述（`desc.rs` 的导出）· 场景参数表 / 我的配置的文本格式 · 模型接口（`vdn-models`）。

## 做到哪
- [x] 读交代 · 报到（大总管同意顺序）
- [x] 读代码，定方案（见下）
- [x] 拆 `desc.rs`（930）→ `src/types/src/desc/{mod,speaker,polish,scene,engine}.rs`（最大 481 行），对外路径不变
- [x] 拆 `src/schema/tests/crosscheck.rs`（870）→ `schema_mechanism.rs` · `shm_layout.rs` · `config.rs` · `eval_suite.rs` + `common/mod.rs`（25 条测试一条不少，全过）
- [x] 拆 `src/xtask/src/single_source.rs`（789，8d 同意归我；人定按规则命名）→ `no_mirror.rs`（R-MIRROR）· `param_range.rs`（R-PARAM，按 `desc/` 目录读描述表）· `scan.rs`（共用扫描）；xtask 28 条测试全过，tidy 通过
- [x] 拆 `policy.rs`（688）→ `policy.rs`（401）+ `policy_tests.rs`（289；8d 要改的 `const SR` 随测试搬到 `policy_tests.rs:90`）
- [x] 文档引用：只有 `specs/schema/README.md` 在用，已改；已收尾线的现状页（single · schema · rs-entry）是历史记录，不改
- [x] 参数描述 → `specs/schema/params.table.json`（58 条；改表不导出 → 红）
- [x] 我的配置越界拒收：先写测试（红）→ `memory.rs` 读值时按描述表 `check`（绿）
- [x] 仓里文本文件过检查：示例 `specs/schema/examples/{scene-table,profile}.example.txt`；`tests/text_formats.rs` 扫 `git ls-files` 里所有这两种文件
- [x] 模型接口：声明 `src/models/src/interface.rs` → `models.interface.json`；适配器 `open` 按声明核对；ONNX 实读比对 `#[ignore]`，本机两份真权重实跑全过（2026-11-02）
- [x] FE-M 名字只留一个：正本 `ModelId::as_str` = `fastenhancer-m`，旧写法只在 `ModelId::LEGACY`/`parse`；cli（8d 同意）· denoised · eval 的 system/report 改成从 `ModelId` 取；tidy 新规则 R-MODEL（`src/xtask/src/model_name.rs`）
- [x] `ModelChoice` ↔ `ModelId::NAMES` 比对测试（`src/schema/tests/config.rs`）
- [x] 全量 CI 全绿（425 过 / 0 败 / 7 忽略）· `vdn-schema --check` 一致 · `eval ci` 同改动前（13/17 · 1/10 · 5/5，门禁通过）
- [x] codex 评审 3 轮（P1×1 P2×8，全部属实、全部已改；第 2、3 轮连续无 P0/P1，停止线满足）→ `docs/reviews/012_param-model-schema-codex.md`
- [x] 最终验证：`xtask ci` 全绿（428 过 / 0 败 / 7 忽略）· `vdn-schema --check` 一致 · `eval ci` 同改动前 · 真权重 vdn-models 17 条全过
- [x] PR 草稿 `state/handoff/iface-pr.md`
- [x] 大总管合 main（4f622582）后修两处红：会话测试的旧模型名 · R-MODEL 认测试模块文件
- [x] #68：图加制品内容 ID、格式号升 2、`check_model`、`Pipeline::graph(model, artifact)`、`artifact_id`；CI 全绿（468 过）· eval ci 同前 · 真权重 18 条全过
- [ ] codex 第 4 轮（看 #68 与合并修复）

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
