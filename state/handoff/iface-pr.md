# PR 草稿 · iface（issue #65）

标题：参数描述导出 schema · 模型接口三方一致 · FE-M 名字只留一个（#65）

---

## 做了什么

1. **参数描述导出**：`vdn-schema` 新模块 `params.rs`，把参数描述表（58 条）导出成 `specs/schema/params.table.json`。导出时对 `ParamDesc` 穷举解构，描述表加字段不导出就编译失败。浮点按 f32 最短写法写出。
2. **文本格式与描述表比对**：
   - 修 bug：「我的配置」（`memory.rs`）读文本时原来只查有限、不查范围。先写了会红的测试，现在按描述表越界拒收。
   - `params.table.json` 写出文本里能用的键（由 `ParamField` 派生）和模型名（由 `ModelId::NAMES` 派生）。测试核对：列出的键两个解析器都认，没列出的都拒。
   - 加两份示例 `specs/schema/examples/{scene-table,profile}.example.txt`。测试扫 `git ls-files` 里所有这两种文件，逐个解析。
3. **模型接口三方一致**：
   - 声明放在 `src/models/src/interface.rs`（输入输出的名字、顺序、形状），导出到 `specs/schema/models.interface.json`。
   - 两个适配器打开文件时按声明核对，不一致打不开。gtcrn.rs 手写的名字表和形状表删掉了。
   - FE-M 的缓存形状是本机用真权重读出来的。
4. **FE-M 名字只留一个**：
   - 正本是 `ModelId::as_str` 写出的 `fastenhancer-m`。理由：命令行、配置 JSON、评测结果文件（`baseline-ci.json` 81 个键）都已用连字符；下划线写法只在 `as_str` 一处。
   - 旧写法 `fastenhancer_m` 只在 `ModelId::parse` 兼容（`ModelId::LEGACY`）。写出一律用正本。
   - cli、denoised、eval 的 system / report，以及两个适配器的 `Model::name()`，都改成从 `ModelId` 取。FE 适配器原来自报 `fastenhancer`，也统一了。
   - xtask tidy 新规则 R-MODEL（`src/xtask/src/model_name.rs`）：正本文件之外，字符串里写死这些名字就红。
   - 已提交的历史结果文件不改写。
5. **拆分了哪些文件**（每个代码文件 ≤ 600 行，人定）：
   - `src/types/src/desc.rs`（930）→ `desc/{mod,speaker,polish,scene,engine}.rs`（最大 481）。对外路径 `vdn_types::desc::X` 不变；「全部登记进 ALL」的检查仍在 `mod.rs` 一处。
   - `src/types/src/policy.rs`（688）→ `policy.rs`（401）+ `policy_tests.rs`（289）
   - `src/schema/tests/crosscheck.rs`（870）→ `schema_mechanism.rs` · `shm_layout.rs` · `config.rs` · `eval_suite.rs` + `common/mod.rs`，新增 `text_formats.rs` · `models.rs`
   - `src/xtask/src/single_source.rs`（789）→ 按规则命名（人定）：`no_mirror.rs`（R-MIRROR）· `param_range.rs`（R-PARAM，改为按 `desc/` 目录读描述表）· `scan.rs`（共用的扫描小工具）

## 关联 issue
Closes #65

## 测试（必填，CLAUDE.md 第五节第 6 条）
- 新增的测试：
  - vdn-types：`memory::rejects_out_of_range_values_per_desc_table`（先红后绿）· `params::model_names_single_source`
  - vdn-schema：`tests/text_formats.rs` 5 条（表与描述表逐字段一致 · 改表不导出报红 · 键与解析器一致 · 越界拒收 · 仓里文本文件全过）· `tests/models.rs` 2 条（声明 ↔ 导出文件 · 改声明不导出报红）· `tests/config.rs::config_model_names_follow_model_id`
  - vdn-models：`interface::tests` 3 条（含 `check_read_rejects_any_difference`：元素类型 · 顺序 · 形状 · 名字任一不同都拒） · `gtcrn::tests::declaration_agrees_with_stft` · `fastenhancer::tests::declaration_agrees_with_declared_hop_and_latency` · `tests/interface.rs` 2 条（要权重，`#[ignore]`）
  - xtask：`model_name::tests` 6 条（R-MODEL：读正本名的各种写法 · 写死名字的负例 · 测试项与跨行 / 转义字符串 · 整仓通过）
  - vdn-types：`desc::tests::unlisted_detects_missing_entries`（描述表漏登记 ALL 的负例）
  - vdn-cli：`model_arg_follows_model_id` · vdn-denoised：`model_choice_roundtrips` · vdn-eval：`system_names_follow_model_id`
  - 搬移的测试一条不少：crosscheck 25 条 → 4 个文件共 25 条；xtask 原 R-MIRROR / R-PARAM 测试全保留
- 跑了哪些（命令 + 结果）：
  - `cargo run -p xtask -- ci`：全绿（fmt · clippy · test 428 过 / 0 败 / 7 忽略 · tidy · check-deps · cargo-deny）
  - `cargo run -p vdn-schema -- --check`：与 Rust 正本一致
  - `cargo run -p xtask -- eval ci`：与改动前相同（ref：回归 13/17 · 能力 1/10 · 探针 5/5，门禁通过）
  - 本机真权重：`VDN_REQUIRE_MODELS=1 cargo test -p vdn-models -- --include-ignored`，17 条全过（含接口比对 2 条 · 冒烟 4 条 · 不分配 1 条）
- 测不了的（`#[ignore]` + 人工步骤）：`src/models/tests/interface.rs` 两条要仓外权重，人工步骤见该文件头

## 唯一真相源（CLAUDE.md 第五节第 5 条）
- 本次新增的第二处表示及其校验：
  - `specs/schema/params.table.json` ← 描述表：`vdn-schema` 逐字比对 + `text_formats.rs` 逐字段比对
  - `specs/schema/models.interface.json` ← `ModelKind` 声明：逐字比对 + `tests/models.rs`
  - `src/models/src/interface.rs`（声明）↔ ONNX 文件：适配器 `open` 时比对 + `tests/interface.rs`（`#[ignore]`）
  - `vdn_control::ModelChoice` 的 serde 名（vdn-control 不依赖 vdn-types）↔ `ModelId::NAMES`：`tests/config.rs`
  - `specs/schema/examples/*.example.txt` ← 两个文本解析器：`text_formats.rs` 逐个解析

## 换谱系评审
codex 3 轮（`docs/reviews/012_param-model-schema-codex.md`）：P1×1 · P2×8，全部属实、全部已改；第 2、3 轮连续无 P0/P1，停止线满足；没有留 issue。

🤖 Generated with [Claude Code](https://claude.com/claude-code)
