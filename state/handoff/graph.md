# 现状页 · graph（批次 20261102-b9 · 第二阶段 · #64）

> 手下 `graph`，分支 `feat/processing-graph`，大总管 `noise-canceling-99`。
> 管的实体：处理图描述（`vdn-types`）· Pipeline（`vdn-engine`）· 会话记录 · 修音链频谱级归属。

## 做到哪

1. ✅ 处理图描述：`src/types/src/graph.rs`。包括节点种类登记表 `KINDS`（6 种）、`ProcessingGraph`（模型 + 频域段 + 时域段）、规范文本与内容 ID、`match_chain` 比对
2. ✅ 「级 ↔ 种类」只写一处：`Stage::name()` → `Stage::kind() -> &'static NodeKind`，名字从种类取；`SpectralStage` 同样改
3. ✅ Pipeline 构造期比对：`with_stages(engine, io, block, &描述, stages)`，不一致返回 `PipelineError::Graph`；实时 `process` 不变
4. ✅ 参数清单交叉校验：speaker（包络 · 门控）、polish 现有参数表测试里各比一次节点的参数清单
5. ✅ 拆文件：`src/engine/src/pipeline.rs`（607 行）拆出 `pipeline/error.rs` 与 `pipeline/rt.rs`；`src/engine/tests/stages.rs`（601 行）拆出 `tests/fallback.rs` 与 `tests/common/mod.rs`
6. ✅ 会话记录：`SessionHeader`/`SessionRecord` 从 `record.rs` 搬到 `src/types/src/session.rs`；头加 `graph`（处理图内容 ID）· `model`，格式 1 → 2；格式 1 照常能读（两项为空）；头部键正本 `HEADER_KEYS`（穷举解构）
7. ✅ schema 导出：`processing-graph.kinds.json` · `session-header.fields.json`；测试 `src/schema/tests/graph.rs`（8 条），含「编号永不复用」对照 main 基线（`graph::RETIRED`）。基线读取函数加了文件名参数（`baseline*`），原 `shm_baseline*` 保留为包装
8. ✅ 规约 §3.3 · §3.7.5、`.42cog/cog.md`、`specs/schema/README.md` 已同步
9. ✅ `xtask ci` 全绿；`eval ci` 与 main（0604143）对比：门禁结论 · 通过数相同，`report.json` 只有一项计时指标（R02 单帧耗时，debug 构建受机器负载影响）不同
10. ⏳ codex-review

## 纯技术题，我定了（理由）

- **不列 Pipeline 固定段**（重分块、重采样、混合）：它们不随配置变，改它们是改程序本身，不是换图
- **参数值不进图**：节点只列读哪些参数（引用 `desc.rs`）；值归场景参数表和我的配置，它们各有版本，会话头已记
- **模型节点不在 Pipeline 里比对**：Pipeline 只认 `Model` trait，不认模型 ID。图由调用方用 `Pipeline::graph(model)` 拼出来。模型名和模型 ID 的对应归 iface 线（转交）
- **`SpectralStage` 留在 polish**：搬进零依赖的 `vdn-types` 要自带复数类型，每帧还要和 `realfft` 互转（复制，或者用 `unsafe`，而 types 禁 unsafe）。现在它只有一个实现，也还没有调用方。图里它挂在模型节点上（`spectral` 段），种类正本照样在 `vdn-types`
- 测试里的假级编号 ≥ `0xF000`，用 `graph::test_kind` 造（编译期查），不进登记表

## 转交

- iface：模型节点与 `Model::name()` 的比对（要等模型接口定下来）
- 以后接频域级的人：模型适配器调用频域级时，在适配器构造处照 `match_chain` 比对 `ProcessingGraph::spectral`

## 顺带发现（不在我范围，报大总管）

- `src/types/src/policy.rs` 688 行、`src/types/src/desc.rs` 930 行，超 600 行上限

## 待拍板

（无）
