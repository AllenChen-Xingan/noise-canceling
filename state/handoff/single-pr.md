## 做了什么

删掉 `vdn-types` 的三份手抄镜像，并让每个可调参数的范围与默认值只写一处。

1. **删镜像**：`src/speaker/src/types.rs` · `src/polish/src/mirror.rs` · `src/scene/src/mirror.rs` 删除，speaker / speaker-lab / polish / scene 改直接用 `vdn-types`。9 处分歧以正本为准（明细见 `state/handoff/single.md` 分歧表）；唯一可见变化：scene 路由表文本里 FE-M 的名字从 `fastenhancer-m` 变成正本的 `fastenhancer_m`。polish 的频域扩展 trait `SpectralStage` 正本里没有，按大总管定的方案 A 留在 polish（`spectral_stage.rs`）
2. **参数描述表** `src/types/src/desc.rs`：58 条常量，每条 稳定编号 · 键名 · 单位 · 最小 · 最大 · 默认 · 说明；零依赖、无 serde，第二阶段可直接导出 schema。使用方：
   - `vdn-types`：`ParamField` 的范围与键名、`ParamSet::DEFAULT`、`Hysteresis`、`Transition`
   - speaker：`GateParams` · `EnvelopeParams` · `JudgeParams` · `EnrollParams` · `FloorStage`
   - polish：`PolishParams`（`OFF` / `DEFAULT`）· `PolishChain`（过渡时长、收参数）
   - scene：`set_tuning`（滞回、后验平滑）
   - engine（只改引用）：`SafetyConfig::default`、`SafetyConfig::clamped`（`Engine::new` 先夹）、Pipeline 淡化时长
   - cli · speaker-lab · 两个 example：默认值取表、越界报错
   - 每个使用方两件套：`validate()` 越界拒收（穷举解构，结构加字段不登记就编译失败）；`clamped()` 夹进范围（构造与收参数时先夹，实时侧不会见到越界值）
3. **tidy 两条新规则**（`src/xtask/src/single_source.rs`）：R-MIRROR（不许 `mirror.rs`；不许在 `vdn-types` 外定义同名类型）· R-PARAM（参数范围不许在描述表外写成数字字面量）。挡得住 / 挡不住的写法见 `state/handoff/single.md`
4. 文档：`.42cog/cog.md` 四行（types · speaker · polish · scene）、`src/speaker/README.md`
5. 顺带：polish AGC 的私有底噪跟踪变量 `floor_db` → `noise_floor_db`（撞参数名，被新规则抓到；行为不变）

## 关联 issue
Closes #56

## 测试（必填，CLAUDE.md 第五节第 6 条）
- 新增的测试：
  - `vdn-types`：`desc::tests`（6 条：每条形式合法、编号键名唯一、全部登记进 `ALL`、按编号 / 键名查找、check / clamp 按表、报错文字）· `params::fields_follow_desc_table` · `policy::hysteresis_and_transition_follow_desc_table`
  - speaker：`gate_params_follow_desc_table` · `new_clamps_out_of_range_floor` · `envelope_params_follow_desc_table` · `judge_params_follow_desc_table` · `enroll_params_follow_desc_table`
  - speaker-lab：`gate_params_rejects_out_of_range_by_desc_table` · `judge_params_parse_and_reject_by_desc_table`
  - polish：`params_follow_desc_table` · `every_preset_is_in_range` · `chain_clamps_params_by_desc_table`
  - scene：`table_text_names_parse_back_via_vdn_types`（路由表里的模型名 / 场景名都能被 `vdn-types` 解析回来）· `calibrated_hysteresis_matches_vdn_types_default` · `calibrated_ema_matches_desc_default` · `set_tuning_clamps_by_desc_table`
  - engine：`safety_defaults_follow_desc_table` · `desc_boundary_values_are_accepted_by_engine` · `engine_clamps_config_by_desc_table`
  - cli：`safety_args_follow_desc_table`
  - xtask：R-MIRROR / R-PARAM 正例、负例、词法剥离、描述表解析、整仓通过（`repo_passes`）
  - 每条「同表」测试都是：默认值逐字段等于表 → 取表的边界外一点造输入 → `validate` 报的是那一条 → `clamped` 夹到边界
- 跑了哪些（命令 + 结果）：`cargo run -p xtask -- ci`（fmt · clippy -D warnings · test --workspace · tidy · check-deps · cargo-deny）：<收尾时填最后一次结果>
- 测不了的：无

## 唯一真相源（CLAUDE.md 第五节第 5 条）
- 正本：参数范围与默认值 = `src/types/src/desc.rs`；接口类型 = `vdn-types`
- 本次新增的第二处表示及其校验：
  - scene `weights.rs` 里训练脚本写出的滞回参数与后验平滑常数 → 测试比对描述表默认值
  - scene 路由表文本里的名字 → 测试经 `ModelId::parse` / `UserScene::parse` 解析回来
  - xtask 按文本解析 `desc.rs` 的键名 → 「键名数 = 常量数」检查 + 测试
- 删掉的第二处表示：3 份镜像；各 crate 手写的默认值与范围（`clamp(0.0, 1.0)` 等）
- 已知未统一（不在本 PR 实体内，已转交）：命令行 / 配置 / 评测里 FE-M 仍叫 `fastenhancer-m`

换谱系评审：codex 3 轮，已达停止线（第 2、3 轮无 P0/P1），14 条全部属实已改，记录 `docs/reviews/011_types-single-source-codex.md`。

🤖 Generated with [Claude Code](https://claude.com/claude-code)
