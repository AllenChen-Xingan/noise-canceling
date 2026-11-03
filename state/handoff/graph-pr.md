## 做了什么

处理图有了描述数据，并且和运行时交叉校验（D72 · `docs/plan/011_schema-crosscheck.md` 第二阶段）。运行时照旧是编译期定死类型的级串（`StageChain`）：不引运行时图库，也不改成 `Vec<Box<dyn Stage>>`。图只作描述。

- **处理图描述**（新文件 `src/types/src/graph.rs`，零依赖）
  - 节点种类登记表 `KINDS` 有 6 种：两种包络、衰减下限、门控、修音链的 AGC 与频域部分
  - 每种节点有稳定编号 · 名字 · 版本 · 挂点（时域 / 频域）· 声明延迟 · 读哪些参数。参数引用 `desc.rs` 的正本，不另抄范围
  - `ProcessingGraph` = 模型 + 频域段 + 时域段。内容 ID 是规范文本的 FNV-1a：名字转义，文本无歧义；不含说明文字
  - 删掉的编号进 `RETIRED`，永不复用
- **「级 ↔ 种类」只写一处**：`Stage::name()` 换成 `Stage::kind() -> &'static NodeKind`，名字从种类取；`SpectralStage` 同样改。speaker 的 3 个包络级和门控、polish 的修音链都跟着改了
- **Pipeline 构造期比对**：`Pipeline::with_stages(engine, io, block, &描述, 级串)` 多了「描述」参数
  - 构造时用 `match_chain` 逐级比对级数 · 种类 · 延迟，不一致返回 `PipelineError::Graph`
  - 实时 `process` 不多一步；`new` / `with_block` 对应空描述
  - 新增 `Pipeline::stage_kinds()` 和 `Pipeline::graph(model)`
- **会话记录**：`SessionHeader` / `SessionRecord` 从 `record.rs` 搬到新文件 `src/types/src/session.rs`
  - 头部加 `graph`（处理图内容 ID）和 `model`，格式号 1 → 2；`set_graph` 从同一张图一起填这两项
  - 头部键的正本是 `HEADER_KEYS`，对 `SessionHeader` 穷举解构
- **schema**：新增导出 `specs/schema/processing-graph.kinds.json` 和 `session-header.fields.json`
  - 种类表里带一份代码生成的规范文本样例和它的内容 ID，外部实现可以拿来核对
  - `src/schema/src/lib.rs` 的基线读取函数加了文件名参数（`baseline*`）；原 `shm_baseline*` 保留为包装，行为不变
- 规约 `specs/rust/architecture.md` §3.3 · §3.7.5、`.42cog/cog.md`（实体表加「处理图描述」、crate 表两行）、`specs/schema/README.md` 已同步

**拆分了哪些文件**（600 行上限，按职责拆，纯搬移）
- `src/engine/src/pipeline.rs`（607 行）→ 构造与延迟账留原处（约 300 行）；构造错误拆到 `pipeline/error.rs`；实时路径拆到 `pipeline/rt.rs`
- `src/engine/tests/stages.rs`（601 行）→ 48 kHz 送原声的测试拆到 `tests/fallback.rs`；公共小工具拆到 `tests/common/mod.rs`
- `src/types/src/record.rs`（577 行，加字段会超）→ 会话头与会话记录拆到 `src/types/src/session.rs`

**行为变化（请留意）**
- `Stage` trait 去掉了 `name()`，必须实现 `kind()`。仓外没有实现方
- `Pipeline::with_stages` 多一个参数。仓里只有 engine 的测试在调，已改
- 会话记录写出格式 2。格式 1 的文件照常能读，`graph` / `model` 为空；格式 1 里出现这两个键算坏文件
- 会话记录头的 rate · table · profile 缺了、或任一头部键写两次，现在报错。原来静默补默认值

**转交**
- `SpectralStage` 归属（交代第 5 条，只定方案）：**trait 留在 polish，节点种类进 `vdn-types`，在图里挂在模型节点上**（`ProcessingGraph::spectral`，挂点 `Hook::Spectral`）
  - 不搬进 `vdn-types` 的理由：那边零依赖、禁 unsafe，搬过去要自带复数类型，每帧还要和 `realfft` 的 `Complex32` 互转复制
  - 现状：只有一个实现，还没有调用方（engine 不做 STFT）
  - 以后：模型适配器开始调用频域级时，在适配器构造处照 `match_chain` 比对 `spectral` 段，届时再看要不要搬
- 模型节点与 `Model::name()` 的比对交给 iface 线：Pipeline 不认模型 ID，图由调用方用 `Pipeline::graph(model)` 拼成

## 关联 issue
Closes #64

## 测试（必填，CLAUDE.md 第五节第 6 条）
- 新增的测试：
  - `src/types/src/graph/tests.rs`（12 条）：登记表合法（编号 · 名字唯一、参数就是 `desc` 那一条、退役编号不复用）；描述与级串不一致的负例（少一级 · 多一级 · 顺序反 · 名字错 / 版本不同 · 延迟不同 · 挂点错）；内容 ID 稳定且规范文本快照固定；模型 · 版本 · 参数 · 延迟 · 顺序 · 级数 · 分段任一改动 ID 就变，只改说明不变；名字不能伪造规范文本（codex 第 1 轮 #1 的反例）
  - `src/types/src/session/tests.rs`（8 条）：含新字段的往返、不含新字段的往返；格式 1 能读；格式 1 带新键拒读；坏头部拒读；`HEADER_KEYS` 与 `to_text` 写出的键逐个一致；`set_graph` 跟着图变；逐个删键 / 重复键（codex 第 1 轮 #2）
  - `src/engine/tests/stages.rs` 新增 5 条，经 Pipeline 构造：描述一致能建成、`graph()` 与空描述；少一级 / 多一级；顺序反；名字错；延迟与描述不同（另查错误的 `source` 与文字）
  - `src/schema/tests/graph.rs`（8 条）：仓里种类表逐条等于登记表；样例内容 ID 可复算；会话头键表与文本格式一致；`since` 与解析器一致；改了不重新导出 → 红（两份文件各一条）；编号永不复用对照 main 基线，负例覆盖改名 · 版本倒退 · 改行为不升版本 · 删了不退役 · 撤回退役 · 名字被别的编号占用
  - 参数清单交叉校验：`vdn-speaker` 的包络与门控参数表测试里各加一段比对；`vdn-polish` 新增 `graph_kinds_list_the_same_params`，逐个挂点比（codex 第 1 轮 #3）
- 跑了哪些（命令 + 结果）：
  - `cargo run -p xtask -- ci`：全绿（fmt · clippy · test · tidy · check-deps · cargo-deny）
  - `cargo run -p vdn-schema -- --check`：一致
  - `cargo run -p xtask -- eval ci`：门禁通过。与 main 0604143 同一命令对比：通过数和门禁结论相同。`report.json` 只有 R02 的单帧耗时（V-FRAME）不同，是 debug 构建计时受机器负载影响，其余数值逐个相同
  - 实时路径不分配：`staged_pipeline_with_faults_does_not_allocate` 与 speaker / polish 的 no_alloc 测试照过
- 测不了的（`#[ignore]` + 人工步骤）：无

## 唯一真相源（CLAUDE.md 第五节第 5 条）
- 本次新增的第二处表示及其校验：
  - Pipeline 的实际级串 ↔ 处理图描述：构造期 `match_chain`，不一致就返回错误（engine 测试 5 条负例）
  - 节点的参数清单 ↔ 各 crate 的参数表：speaker / polish 的测试逐集合比
  - `processing-graph.kinds.json` ↔ `graph::KINDS`：逐字比对（通用测试）+ 逐条内容比对 + 对照 main 基线的编号守卫
  - `session-header.fields.json` ↔ `HEADER_KEYS` ↔ `to_text` / `from_text`：键与顺序、`since` 与解析器行为都比
  - `HEADER_KEYS` ↔ `SessionHeader`、`NODE_FIELDS` ↔ `NodeKind`：穷举解构，漏字段编译失败
- 换谱系评审：`docs/reviews/011_processing-graph-codex.md`

🤖 Generated with [Claude Code](https://claude.com/claude-code)
