# 评审 016 · exp016 候选模型上同一张成绩单（codex · gpt-6-astra · read-only）

> 本线（issue #112）按人定 2026-11-04 插三次 codex：① 外部入口开小 PR 前（中途，1–2 轮，P0/P1 修掉即停）· ② 第一个候选跑 480 段前审离线推理脚本（中途，同上）· ③ 收尾审 findings（连续两轮无 P0/P1）。
> **最新：① 外部入口第 2 轮（2026-11-04）· 总评「修改」（仅 1 条 P2，已补）· 中途评审按人定 1–2 轮、P0/P1 清零即停 → 停**。

## ① 外部离线输出入口

> 对象：`git diff 4493f0ce...HEAD`（`src/eval/src/system.rs` 分发 · `src/eval/src/system/external.rs` · `src/eval/tests/external_e2e.rs` · `src/eval/src/main.rs` 一行 · `docs/experiments/exp016-candidates/external-entry.md`）。重点：evalfix 提的 3 条公平性检查，尤其截半查偷看未来能否被无心绕过。
> 累计：P1×1、P2×5 逐条回源码核实，属实 6、驳回 0；改 4 条，2 条 P2 留作 issue（待大总管开）。

### 第 1 轮（2026-11-04）· 总评「修改」

原文：主目录 `_tmp/exp016/reviews/diff-4493f0ce-head-codex-1.md`（不入库）。effort medium，60 秒。行号指评审时的版本。
本轮 P0/P1：1 条 · 留作 issue 的 P2：#3 · #4

- **P1｜external.rs:132 不同目录同名系统被视为相等，入口也不拒绝**（第 1 轮 #1）
  - 核实：属实。`PartialEq` 按名字比，`load` 不查重；两个同名目录同时参评时成绩单 `system` 列分不清来源
  - 处理：`SystemKind::external` 改返回 `Result`，进程内名表查重，第二个同名目录报「已被另一个目录用过」；加测试 `duplicate_external_name_is_rejected`
- **P2｜external.rs:331 延迟 ≥ 半段时截半比较为空仍通过**（第 1 轮 #2）
  - 核实：属实。`saturating_sub` 得 0，最大差恒为 0
  - 处理：比较区间短于段长 1/4 报「截半自检无法验证」；加测试 `huge_declared_latency_cannot_be_verified`
- **P2｜external.rs:323 单段单切点不能可靠排除离线处理的未来依赖**（第 1 轮 #3）
  - 核实：属实（切点落静音时有限前视可能漏判；峰值在前半段的整段归一化可完全一致；`bg` 未查）
  - 处理：说明里写明「抽样证据，不是因果性证明」并列出漏判情形（`external-entry.md` 第 4 节）；多切点 / 有声区切点 / `bg` 截半**留作 issue**
- **P2｜external.rs:337 超差直接归因于未来依赖，未排除数值不可复现**（第 1 轮 #4）
  - 核实：属实（Core ML / Neural Engine 可能有运行间抖动）
  - 处理：说明里写明会误判；「同输入重复运行对照」**留作 issue**。实际跑候选时若被拒，先手工重复跑一次判是否抖动
- **P2｜external_e2e.rs:265 缺 bg 会让整段没跑成，不是只背景一路**（第 1 轮 #5）
  - 核实：属实（`runner.rs` 的 `prepare_clip` 一路失败即整段失败；`runner` 归 evalfix，不改）
  - 处理：改说明契约为「整段进没跑成」，并写明正式跑一律给 `bg/`

#### 引用抽查

- 「不符」1 条：`external-entry.md:47` 关于缺 bg 的表述 → 已按 #5 改
- 「核实属实」4 条
- 无法核实：Engine 故障复位后回放位置、计时项最终判定（实现文件不在 codex 可读范围）→ 由测试 `declared_latency_is_reinserted_and_timing_is_empty` 与逐位一致测试覆盖

#### 是否再审

有 P1 → 改完开第 2 轮。

### 第 2 轮（2026-11-04）· 总评「修改」

原文：主目录 `_tmp/exp016/reviews/diff-4493f0ce-head-codex-2.md`（不入库）。effort medium，56 秒。
本轮 P0/P1：0 条 · 停止线：中途评审（人定 2026-11-04：1–2 轮，P0/P1 修掉即停）→ **停** · 留作 issue 的 P2：无新增

上一轮处理的验证：codex 抽查确认 #1（同名查重）、#2（比较区间下限）、#5（缺 bg 契约与实现一致）已修；#3 #4 只披露、延期，本轮不重复列。

- **P2｜016_candidates-codex.md:36 测试没覆盖故障复位与计时项最终判定**（第 2 轮 #1）
  - 核实：属实（原有两条测试都关了保命或没注入故障）
  - 处理：加测试 `faults_and_resets_keep_replay_on_the_same_time_axis`：保命开，注入 NaN · 后端报错 · 幅度爆炸三种故障，内置 passthrough 与回放的输出 · 送原声 · 门控 · 复位次数逐位一致；并断言成绩单标准不含计时项（所以「计时给空」对成绩单无影响）。变异检查：把 `Replay::reset` 改成回卷，这条测试变红；改回后全绿

#### 引用抽查

- 「不符」1 条：即本轮 #1，已补测试
- 「核实属实」5 条
- 无法核实：Engine 故障时是否每 hop 都调模型（实现不在 codex 可读范围）→ 由上面的故障测试实测覆盖

#### 是否再审

不再审（中途评审，P0/P1 已清零）。收尾时 ③ 按连续两轮无 P0/P1 跑。

## ② 离线推理脚本（跑 480 段前）

> 对象：`docs/experiments/exp016-candidates/{_ext.py,run_dpdfnet2.py,run_dfn3ll.py,tests/test_ext.py}`。重点：采样率转换 · 流式 / 低延迟模式与声明是否一致 · 权重哈希与可追溯。
> 冒烟实测（第 1 段）：DPDFNet-2 滞后 0、截半前视 532 ≤ 声明 960；DFN3 低延迟版滞后 0、前视 160 ≤ 声明 284。

### 第 1 轮（2026-11-04）· 总评「修改」

原文：主目录 `_tmp/exp016/reviews/ext-codex-1.md`（不入库）。effort medium。
本轮 P0/P1：3 条 · 留作 issue 的 P2：无

- **P1｜_ext.py:212 分片记录建在输出目录里，入口拒绝没列的子目录**（第 1 轮 #1）
  - 核实：属实（`external.rs` 载入时拒绝 run.json 没列的子目录）。codex 交卷前我自查也发现了
  - 处理：分片记录与指纹移到输出目录旁边的 `<run-id>.shards/`；`finalize` 查输出目录里不许有入口不认的子目录或 `.part` 文件；加测试 `test_finalize_writes_run_json_and_rejects_stray_dirs`。「生成完整目录后由 Rust 载入」的集成在正式跑后打分那一步实地发生（载入失败即停）
- **P1｜run_dfn3ll.py:104 截半前视不等于实时链路延迟**（第 1 轮 #2）
  - 核实：部分属实。截半只量算法前视，确实不含攒块与 soxr 内部缓冲的等待
  - 处理：不改算法，改说明：`latency_samples` 按模型接口正本定义只含算法前视、不含攒 hop 的等待，内置 GTCRN 同口径，因此可比；DFN3 低延迟版的 284 写明是下界，实时延迟引用 019（README「延迟口径」）。交第 2 轮看这条理由
- **P1｜_ext.py:200 续跑会混入旧实现的输出**（第 1 轮 #3）
  - 核实：属实
  - 处理：第一次运行写 `<run-id>.shards/fingerprint.json`（项目内全部已载入代码的 sha256 · 模型信息 · 运行配置 · 依赖版本 · 片段清单哈希）；每个分片、续跑、`--finalize` 都比对，不一致报错要求换 run-id；指纹进 `builds/…/params.json`；加测试 `Fingerprint.test_mismatch_is_refused`
- **P2｜_ext.py:153 半截 wav 被当完成**（第 1 轮 #4）
  - 核实：属实（自查同时发现）
  - 处理：先写 `.part` 再原子改名；续跑跳过前校验能读 · 等长 · 有限；加测试
- **P2｜_ext.py:237 归档不足以复现**（第 1 轮 #5）
  - 核实：属实
  - 处理：并入 #3 的指纹（推理时记 ORT · soxr · numpy · soundfile · libsndfile 版本与平台）
- **P2｜test_ext.py:59 输入一致性没跨语言核对**（第 1 轮 #6）
  - 核实：属实（exp013 输入全是 32 位浮点，实际碰不到整型差异，但测试应覆盖）
  - 处理：加 `RustWavParity`：比对 `wav.rs` 的归一公式与取第一声道写法仍在；PCM_16 · FLOAT · 双声道三种文件的读值逐位符合该公式

#### 引用抽查

- 「核实属实」6 条；「无法核实」3 条（libDF reset / atten 语义、整型归一、soxr 尾部）→ 前两条由冒烟与 `RustWavParity` 覆盖；尾部由 `process` 的长度检查与截半自检兜底

#### 是否再审

有 P1 → 第 2 轮。
