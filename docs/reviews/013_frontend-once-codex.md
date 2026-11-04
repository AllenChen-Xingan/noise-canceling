# 评审 013 · 每帧只算一次频谱（codex · gpt-6-astra · read-only）

> 对象：`git diff 990e7843...HEAD`（分支 `feat/frontend-once`，含 `docs/plan/013_frontend-once.md`）。**最新一轮：第 2 轮（2026-11-04）· 总评「修改」· 待第 3 轮（连续第 1 轮无 P0/P1）**。
> 累计处理：P1×1、P2×6 逐条回源码核实，属实 6、部分属实 1、驳回 0。

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

## 第 2 轮（2026-11-04）· 总评「修改」

原文：草稿目录 `reviews/diff-990e7843-head-codex-2.md`（不入库）。effort medium，耗时 65 秒。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1 · 留作 issue 的 P2：无

上一轮处理的验证：codex 抽查确认第 1 轮 #1、#2、#4 已按记录修好（`stft.rs` 的 `feed_partial`、`features.rs` 的退回条件和注释、对拍截到整 hop）。

（另：第 1 轮原文开头是「P1」两个字符，`file` 命令把它认成位图，隐私扫描当成非纯文本拦下。在原文前加了一行标题再发，内容不变。）

- **P2｜`src/scene/tests/frontend.rs:122` 切换测试测不出漏推缓冲**（第 2 轮 #1）
  - 核实：属实。切换点在第 6 秒，前面第 4–6 秒是静音段，没推进的全零缓冲正好等于正确的历史。
  - 处理：改成「借谱 → 3 s 换自算 → 7 s 再借谱」，两个切换点都在有声段中间，前端每帧都跑。比的是整份摘要，要求逐位相同。
  - 变异检查（实测）：把借谱入口的 `self.stft.feed(x)` 注释掉，这条测试变红；恢复后变绿（文件逐字节核对过已恢复）。复位后再借谱的情况没有单独测：`reset` 会同时清零两边的缓冲。
- **P2｜013 第 8 节说的「真 GTCRN 每帧 FFT 计数」没实现**（第 2 轮 #2）
  - 核实：属实。
  - 处理：加了 `models/tests/tap.rs::gtcrn_plus_scene_borrowing_does_no_extra_fft`：真 GTCRN 借谱给场景，要求场景自算 FFT 0 次，判断（35 维特征按位比、生效场景）与自算逐位相同。GTCRN 内部那 1 次由构造保证，没有对外计数；第 8 节照实改写。
- **P2｜013 第 11 节拟定的 tidy 规则会误报场景的逆 FFT**（第 2 轮 #3）
  - 核实：属实（`features.rs` 求自相关用的逆 FFT 计划）。
  - 处理：规则限定为正向 FFT 的创建（`plan_fft_forward`）。逆 FFT 要收进来就另加验收。

### 引用抽查

- 「不符」：无。
- 「核实属实」：5 条。
- 无法核实：
  - CI 是否真比：`.github/workflows/ci.yml:12` 是 `runs-on: macos-15`（arm64），与基准平台 aarch64-macos 相同。
  - 基准是否来自改前源码：见现状页的生成方法。
  - 第二段：留给第二段的手下。

### 是否再审

本轮无 P0/P1，但只是连续第 1 轮，再审一轮确认。
