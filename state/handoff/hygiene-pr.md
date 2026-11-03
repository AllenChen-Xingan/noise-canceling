# PR 草稿 · #62 代码卫生（hygiene 写，大总管推送前过目）

> 状态：可以开 PR（2026-11-03）。codex 评审已达停止线（9 轮）；#65 已合并进来（合并提交 f0addc43，8d 提交）；R-SIZE 已改成硬上限；`xtask ci` 全绿。

## 做了什么

三件事：每样东西只写一处、代码文件不超过 600 行、评测阈值有正本。

1. **单文件 ≤ 600 行进闸门（R-SIZE，`src/xtask/src/file_size.rs`）**
   - 扫 `src/` 与 `docs/experiments/` 下的代码文件。扫描目录收成常量 `SCAN_ROOTS`，扩展名收成常量 `CODE_EXTS`：rs · py · sh · js · ts · swift · c · h · m · mm
   - 硬上限：超 600 行即红，不设基线、不留豁免（人定）。过渡期用过只降不升的基线，存量拆完后连同 `size-baseline` 子命令、放宽机制一起删掉
   - 仓里现在没有超过 600 行的代码文件【测】
2. **拆大文件（按职责，不机械切）**
   - vdn-cli：`main.rs` 852 → 340 行，`chain.rs` 825 → 508 行，`live.rs` 750 → 517 行
   - vdn-eval：`verifiers.rs` 1168 → 399 行（拆出物理量 / 感知量 / 模拟真值三类），`runner.rs` 629 → 424 行（拆出门禁判定、读真实片段）
   - split 线（另一位手下）拆了 capture、denoised、speaker-lab，见 `state/handoff/split.md`
3. **cli 同名不同物**
   - 三个 `Verdict` 含义不同。vdn-eval 那份保留；chain 的改名 `ChainVerdict`，live 的改名 `LiveVerdict`
   - 两份 `StopOnDrop` 合成 `src/cli/src/stop.rs`
4. **采样率只写一处**
   - 正本 `src/types/src/rate.rs`：`IO_RATE` = 48 kHz、`MODEL_RATE` = 16 kHz
   - vdn-control、vdn-io-macos 改为转出 `IO_RATE`，为此新增对 vdn-types 的依赖。architecture.md §2.2 加了规则 5
   - shm、rt-thread、dsp 加 vdn-types 为 dev 依赖（forbid 只管普通依赖闭包；shm 已有先例）
   - 约 150 处 16000 / 48000 字面量改成引用常量，测试代码也改了
   - 新 tidy 规则 R-RATE（`src/xtask/src/rate_literal.rs`）：代码里不许再写采样率字面量，测试、示例都算；单行放行要写理由
5. **评测阈值正本**（`src/eval/src/thresholds.rs`）
   - 16 个评测器的分档与默认阈值收进一张表，每个数带 012 / rubric 原文摘录或设计理由；verifiers 改成从表里取
   - 比对测试 `src/eval/tests/thresholds.rs` 查三件事：
     - 摘录逐字在文档里，数字带符号对上
     - 分档顺序按好坏方向检查
     - ci.json / qa.json 的每个阈值都在表里登记
   - 待定清单 `PENDING` 必须和找到的不一致完全相等：多一条红，修好一条也红
   - 每个评测器只许用自己的分档
6. **V-SPLVL 分档改为 −1 / −2 / −4 dB**（原来 C/D 线是 −3）
   - 起因：codex 查出量法描述错。代码不是 P.56，而是按有声帧做投影能量损失，与 rubric C3 口径接近
   - 据此按铁律 5（标准文档是正本）改成 C3 的数（大总管 8d 定）
   - 改后 `xtask eval ci`：V-SPLVL 48 条，等级变化 0 条

## 阈值不一致 7 处（待定标）

详见 `state/handoff/hygiene-thresholds.md`：来历（rubric 与代码各是哪次提交）、后果复算、建议。全部是建议值，挂在 012 拍板 #8 · D44 下，不新开题。

| # | 评测器 | 不一致 |
|---|---|---|
| 1 | V-TIMBRE 分档 | 代码 2 / 4 / 6 dB，rubric N4 1.5 / 3 / 6。两边量法不同（频带、分帧都不同），保留代码的数 |
| 2 | V-SPLVL 分档 | 已按 C3 改成 −1 / −2 / −4，**已移出待定**（见上文第 6 条） |
| 3 | V-SPLVL 过线值 −3 dB | 没有出处 |
| 4 | V-ONSET 分档 | 只有 3 dB 有出处，1 / 2 dB 没有 |
| 5 | V-NR max | −8、−10 dB 没有出处（−6、−20 是 rubric C2 档线） |
| 6 | V-SIM-GATE max 0.05 | 没有出处 |
| 7 | V-SIM-GATE onset_max 0.2 | 没有出处 |

分档只写进报告的等级字段，不参与过不过和门禁。

## 关联 issue

Closes #62

## 测试（必填）

- **新增的测试**：
  - xtask：
    - R-SIZE：超线、基线棘轮、重复路径、路径写法、扫描出错、docs/experiments 与非 Rust 文件
    - R-RATE：各种写法与后缀、区间写法、注释字符串不算、空理由、测试代码也查
  - vdn-cli：`stop.rs` 2 条
  - vdn-types：`rate.rs` 1 条
  - vdn-eval：`tests/thresholds.rs` 共 8 条：真仓比对 1 条，查表与绑定各自评测器 1 条，负例 6 条（新不一致、修好未移出、摘录缺失 / 未登记 / 无出处、同一参数新值、符号与顺序、同值不同原因）
- **跑了哪些**（2026-11-03，本机实测，提交 e70b338d）：
  - `cargo run -p xtask -- ci`：**全绿**。fmt · clippy · test · tidy · check-deps · cargo-deny 六步都过；测试 491 过 / 0 败 / 7 忽略
  - `cargo run -p xtask -- eval ci`：回归 13/17 · 能力 1/10 · 探针 5/5 · 门禁通过，和改前一致。只有参考降噪器，本机没有 GTCRN 权重
  - `cargo run -p vdn-schema -- --check`：schema 与 Rust 正本一致
  - `cargo test -p vdn-cli`：改前 21 过 / 2 忽略，拆完 23 过 / 2 忽略；合并 #65 后多 1 条（`model_arg_follows_model_id`，从 main.rs 搬进 args.rs）
- **测不了的**：无新增

## 唯一真相源

本次新增的第二处表示及其校验：

| 第二处表示 | 正本 | 校验 |
|---|---|---|
| `vdn_control::IO_RATE`、`vdn_io_macos::IO_RATE` | `vdn_types::rate` | `pub use` 转出，不是副本 |
| `VAD_SR`、`synth::SR` 等保留的本地名字 | `vdn_types::rate` | `= vdn_types::MODEL_RATE`，等号引用 |
| 012 / rubric 文档里的阈值，ci.json / qa.json 里的阈值 | `src/eval/src/thresholds.rs` | `src/eval/tests/thresholds.rs` |
| 字面量不许再出现 | `src/types/src/rate.rs` | tidy R-RATE |
| 行数上限 600 | `src/xtask/src/file_size.rs` 的 `MAX_LINES` | tidy R-SIZE（CLAUDE.md 第五节第 7 条是文字说明，数字只在代码里一处） |

## 未做

- **自动识别测试模块**：试了 7 轮评审（codex 第 2–7 轮），每轮都有新绕过点，最终放弃，改用显式标注（文件头整模块 NRT 标注）。以后真要做，用 `syn` 解析语法树（AST 报告扩展方向第 8 条，已开 #71，设计见 `docs/plan/012_tidy-syn.md`），不要再用文本匹配。
- R-RATE 查不到算出来的值（`3 * 16_000`），也不管 `docs/experiments/` 下的独立实验程序（它们不依赖 vdn-types）；前者留给 #71。
- #65 新加的 `src/xtask/src/test_scope.rs` 也是文本猜测试范围，这次不动，记进 #71 设计（`docs/plan/012_tidy-syn.md`）。
- 7 处阈值不一致只登记、不改数（见上表），等 012 拍板 #8。

## 评审

codex 换谱系评审 9 轮，第 8、9 轮连续无 P0/P1，已达停止线。共提出 21 条（P1×11 · P2×10），全部属实。记录在 `docs/reviews/062_code-hygiene-codex.md`。

🤖 Generated with [Claude Code](https://claude.com/claude-code)
