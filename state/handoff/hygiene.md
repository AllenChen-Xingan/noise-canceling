# 现状 · hygiene（批次 20261102-b8 · issue #62 · 分支 feat/code-hygiene）

大总管：noise-canceling-8d。管的实体：xtask（tidy 闸门）· vdn-cli · vdn-types（只动采样率常量）· vdn-eval（只动阈值比对）。

## 第一段：完成（2026-11-02）

1. **单文件 600 行上限进 tidy（R-SIZE）**：`src/xtask/src/file_size.rs`
   - 扫 `src/**/*.rs`（交代原文的范围；不扫 `docs/experiments/` 下的基准程序）。基线 14 条比扫描报告的 16 条少，原因：报告扫的是 #60 之前的旧检出，范围也含 `src/` 之外；以本仓现状为准
   - 过渡期用过只降不升的基线与 `size-baseline` 子命令；2026-11-03 存量拆完后删掉，R-SIZE 改成硬上限（人定不设基线）
   - 拆完 cli 后基线 14 → 11 条
2. **cli 的同名不同物**：三个 `Verdict` 意思不同（读源码）——
   - `vdn-eval::Verdict`：评测器三态「过 / 不过 / 不知道」
   - `chain::Verdict`（struct）：一条链路判据的结论记录（编号、名称、过否、细节）→ 改名 `ChainVerdict`
   - `live::Verdict`（enum）：`vdn run --check` 的诊断（没输入 / 全零 / 电平对不上 / 过）→ 改名 `LiveVerdict`
   - vdn-cli 不依赖 vdn-eval，没有为此新加依赖
   - `StopOnDrop` 两份合成 `src/cli/src/stop.rs` 一份（`StopOnDrop<const N>`，统一用 `Release`；原 chain 用 `Relaxed`，读方都只看布尔值，更强的顺序不改行为）
3. **拆 cli 大文件**（按职责，未机械切分）：
   - `main.rs` 852 → 340：命令行参数 → `args.rs`（247）；离线命令 denoise / bench → `offline_cmd.rs`（289）
   - `chain.rs` 825 → 508：判据与汇总行 → `chain/judge.rs`（230）；只给测试的锁步模拟 → `chain/lockstep.rs`（105）
   - `live.rs` 750 → 517：`--check` 判定与全零提示 → `live/check.rs`（166）；输出增益 / 录音分流 → `live/tap.rs`（81）

## 测试（实测，本机 2026-11-02）

- `cargo test -p xtask`：30 过（R-SIZE 新增 8 条单测：超线新文件红、600 行过、基线变大红、变小只注释、作废条目红、基线格式错红、收紧只降不升、真仓基线可解析）
- `cargo test -p vdn-cli`：改前 21 过 / 2 忽略 → 改后 23 过 / 2 忽略（新增 `stop.rs` 2 条；其余测试随代码搬家，条数不减）
- `cargo clippy -p vdn-cli -p xtask --all-targets -D warnings`：无告警
- `cargo run -p xtask -- tidy`：通过（拆完前 R-SIZE 正确报出 3 条作废基线，收紧后通过）
- 全量 `cargo test --workspace` 与 `xtask ci`：未跑，等大总管给名额

## 第二段：完成（2026-11-02）

- 采样率只写一处：`src/types/src/rate.rs`（IO_RATE / MODEL_RATE）；control、io-macos 转出 IO_RATE（新增依赖 vdn-types，architecture.md §2.2 规则 5）；约 115 处字面量改引用（3 个子代理改 + 我复核，纠正 eval/system.rs 1 处语义）；tidy 新规则 R-RATE（`src/xtask/src/rate_literal.rs`）
- ~~tidy 认测试模块~~：试了 7 轮评审后放弃（见 `docs/reviews/062_code-hygiene-codex.md` 末节）；R-RATE 改为测试代码也查，shm / rt-thread / dsp 加 vdn-types dev 依赖
- 评测阈值正本 `src/eval/src/thresholds.rs` + 比对测试 `src/eval/tests/thresholds.rs`；7 处待定见 `state/handoff/hygiene-thresholds.md`
- 拆文件：verifiers.rs 1168 → 399（+ physical / perceptual / sim）；runner.rs 629 → 424（+ gate / real）
- R-SIZE 扫描范围收成一处常量 `SCAN_ROOTS`（现只开 `src`；docs/experiments 等 #58 合并后再开）

## 剩下（等别的线）

- ✅ 2026-11-03：#65 已合并（8d 提交合并 f0addc43）；引用改到 no_mirror / param_range / scan；R-SIZE 改硬上限，删了基线文件、`size-baseline` 子命令、放宽机制
- codex 换谱系评审：第 2 轮进行中（`docs/reviews/062_code-hygiene-codex.md`）

## 第二段原始待办（已办）

- 先办（大总管 8d 定）：merge origin/main 后核对 R-SIZE 基线。#57 改过的 denoised / eval 文件如果变长，按实测写新行数，并在 PR 正文写明原因（唯一允许调大基线的情况）

## 转交

无。
