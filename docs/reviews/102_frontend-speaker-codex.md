# 评审 102 · 统一频谱模块支持 Kaldi 分帧，认人 fbank 改调它（codex · gpt-6-astra · read-only）

> 对象：`git diff d23bfe66..HEAD`（分支 `feat/frontend-speaker`，issue #102，含 `docs/plan/013_frontend-once.md` 第 12 节）。
> 自审（手下 fe2，交 codex 之前）：改动文件 ≤ 600 行（最大 `tidy.rs` 593）；产出里没有本机路径；`xtask ci` 6 项全绿；认人 fbank / 声纹与第一段 4 份对拍在本机真比、逐位相同。

## 第 1 轮（2026-11-04）· 总评「修改」

原文：草稿目录 `_tmp/20261104-b13/reviews/102-frontend-speaker-codex-1.md`（不入库）。effort medium，耗时 51 秒。行号指评审时的版本。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮 · 留作 issue 的 P2：无

- **P2｜`src/speaker/tests/golden.rs:108` 平台不匹配时对拍直接跳过，没有强制模式**（第 1 轮 #1）
  - 核实：现象属实，驳回改法。CI 是 macos-15（`.github/workflows` 的 `runs-on`），基准是 `aarch64-macos-27`，CI 上认人对拍会打印原因后跳过。
  - 理由：这是全仓对拍的既定做法（#99「平台标签加系统主版本，跨系统版本跳过并打印原因」，场景 / GTCRN 对拍同样如此）。ln / cos / powf 的末位由系统数学库定，换系统大版本就可能变；「改前代码在 macos-15 上的基准」本机生成不了。CI 上真比的是不依赖系统数学库版本的那几条：窗公式逐位、整段 vs 流式、补零、f64、不分配、R-FFT 规则测试。PR 正文写明了哪些跳过。
- **P2｜`src/xtask/src/fft_site.rs:123` 白名单整文件豁免，超出声明的例外；白名单文件解析失败被吞**（第 1 轮 #2）
  - 核实：属实。场景文件只因逆 FFT 获准，旧实现下在那里新加 `plan_fft_forward` 也放行；白名单文件解析失败时问题列表非空，被当成「仍有命中」吞掉。
  - 处理：`ALLOW` 每条加第二列「只许出现的标识符」：评测器两处只许 `RealFftPlanner` + `plan_fft_forward`，场景只许 `RealFftPlanner` + `plan_fft_inverse`；不在列里的照样报红。解析失败无条件上报。判断移进 `fft_problems` 一处。
  - 测试：`fft_site_tests.rs::whitelist_is_scoped_per_identifier`：场景文件只有逆 FFT 不红；加正向计划红 1 条且点名 `plan_fft_forward`；解析失败红；评测器文件出现逆 FFT 红。
- **P2｜`src/dsp/src/fft_tests.rs:34` 「逐位」测试实际用浮点 `==`**（第 1 轮 #3）
  - 核实：属实。`Complex32` 的 `==` 分不出 +0.0 / −0.0。
  - 处理：加 `bits32` / `bits64`，整段 vs 流式、补零、钩子顺序、f64 四处一律比 `to_bits()`。测试信号每 97 个样本插一段 −0.0 与 +0.0。

### 引用抽查

- codex 列的「核实属实」：4 条。
- 无法核实：
  - 基准来自 `d23bfe66`：生成脚本在草稿目录不入库，做法写在 `src/speaker/tests/golden.rs` 文件头与 013 第 12 节（`git archive d23bfe66`、单独 target、只换 `old_names` 两行、`VDN_BLESS=1 VDN_REQUIRE_MODELS=1`）。
  - 窗参照逐字来自旧源码：speech / estoi 不在 codex 可读范围；参照与 `git show d23bfe66:<文件>` 逐字一致由作者核对。

### 是否再审

无 P0/P1，开第 2 轮确认。
