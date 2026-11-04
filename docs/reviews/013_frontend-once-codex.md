# 评审 013 · 每帧只算一次频谱（codex · gpt-6-astra · read-only）

> 对象：`git diff 990e7843...HEAD`（分支 `feat/frontend-once`，含 `docs/plan/013_frontend-once.md`）。**最新一轮：第 1 轮（2026-11-04）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×1、P2×3 逐条回源码核实，属实 3、部分属实 1、驳回 0。

## 第 1 轮（2026-11-04）· 总评「修改」

原文：草稿目录 `reviews/diff-990e7843-head-codex-1.md`（不入库）。effort medium，耗时 60 秒。行号指评审时的版本。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue 的 P2：无

- **P1｜`src/scene/src/features.rs:209` 短输入改变了场景数值行为**（第 1 轮 #1）
  - 核实：属实。改前按 `min(hop, len)` 移位、不补零（`git show 990e7843:src/scene/src/features.rs` 的 `push_hop`）。改后经 `StreamingStft::feed` 移一整个 hop 并补零，空输入也会推进。实时路径每次都是整 hop，碰不到，但属于数值红线。
  - 处理：`vdn-dsp` 加 `feed_partial` / `analyze_partial`，按 `min(len, hop)` 移位，与改前逐行同义；`feed` / `analyze` 不变。场景自算改走 `analyze_partial`。借谱入口遇到 `x` 不是整 hop 就退回自算。
  - 测试：
    - `dsp` 单测 `feed_partial_shifts_by_input_length`：与手算参照比，覆盖 100、0、hop+50、1 几种长度。
    - 新对拍 `scene/tests/golden.rs::ragged_hops_match_golden`：输入长度按表循环（含不足一个 hop、空、超过一个 hop），基准在 `990e7843` 上生成，31 次判断逐位相同。
    - `scene/tests/frontend.rs::ragged_input_with_power_falls_back_to_self_compute`。
- **P2｜`src/scene/src/features.rs:53` 采样率断言是同源自比**（第 1 轮 #2）
  - 核实：部分属实。`FRONTEND.sample_rate` 取自 `MODEL_RATE`；`MODEL_RATE` 本身改了，这条断言拦不住。
  - 处理：注释改成实话：断言只保证「正本不再取 `MODEL_RATE`」时编译失败。`MODEL_RATE` 本身改了会让所有模型一起作废，归采样率正本管。不另写 16000 字面量，因为仓库规则不许写采样率字面量。
- **P2｜`src/scene/tests/common/mod.rs:104` 非基准平台没有改前参照；第 8 节说的「旧实现留在测试里」没有做**（第 1 轮 #3）
  - 核实：属实（文档与实现不符）。
  - 处理：改前参照用在改前提交上生成的基准，不抄 300 多行旧实现进测试。CI 是 macos-15（arm），与生成基准的平台相同，CI 上真比。013 第 8 节第 2 条改成照实写。另加了不整齐长度的那份基准（见 #1）。
- **P2｜`src/models/tests/golden.rs:59` 48000 点里最后 128 点没过模型**（第 1 轮 #4）
  - 核实：属实（48000 = 187 × 256 + 128，`chunks_exact` 丢了尾巴）。
  - 处理：输入截到整 hop（47872 点），在 `990e7843` 上重生成基准。013 第 8、10 节同步改。

另：大总管要求补一条引擎最终输出的对拍，因为成绩单测的是它。加在 `src/models/tests/golden.rs::pipeline_output_matches_golden`：GTCRN 走 48 kHz `Pipeline`，默认安全参数，块长轮换，144000 点，基准在 `990e7843` 上生成，逐位相同。

### 引用抽查

- codex 列的「不符」：`013:91` 的 48000 点 → 已改为 47872（同 #4）。
- codex 列的「核实属实」：3 条。
- 无法核实：
  - 基准的生成过程：生成脚本在草稿目录，不入库。做法写在 `state/handoff/fecore.md`：`git archive 990e7843`，只放对拍测试，单独 target 目录。
  - 第 11 节第二段：还没实现，留给第二段的手下做对拍。

### 是否再审

有 P1，改完开第 2 轮。

### 修后复跑（实测）

- 四份基准在 `990e7843` 原样源码上重生成：
  - 场景：整 hop 24 次判断，不整齐长度 31 次判断。
  - GTCRN：47872 点。
  - 引擎：144000 点。
- 五个 crate 的 `cargo test` 全绿，四份对拍都是真比，没有跳过。
- `xtask ci` 6 项全绿。
