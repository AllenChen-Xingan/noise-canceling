# 交接 · split（批次 20261102-b8 · #62 拆文件部分）· 2026-11-02

## 做了什么（手下 split 做，hook 自动提交）
3 个超 600 行的文件按职责拆开，只搬代码、不改行为；公开接口靠 `pub use` 保持原路径。

| 原文件（行数） | 拆后 |
|---|---|
| `src/denoised/src/lib.rs`（878） | `lib.rs` 165（Processor、建管线、调度与总线配置）· `worker.rs` 360（Worker 与计数器、预热）· `tests.rs` 366 |
| `src/capture/src/lib.rs`（672） | `lib.rs` 131（Push/Pop/Heartbeat 与替身）· `config.rs` 63（参数与计数器）· `forwarder.rs` 487（转发器） |
| `src/speaker-lab/src/main.rs`（659） | `main.rs` 45（分发）· `wav.rs` 37 · `params.rs` 146（含参数测试）· `verdict.rs` 66（含往返测试）· `run.rs` 311 · `cmds.rs` 104 |

顺手做（大总管加派）：R-RATE 采样率字面量 15 处换成正本引用——capture 12、denoised 2 用 `vdn_control::IO_RATE`，speaker-lab 的 `SR` 用 `vdn_types::MODEL_RATE`。

## 测试（实测）
- 拆前（导出提交 `ffdd7e2` 到 scratchpad，共用 target 跑）：capture 23 + 1 · denoised 13 · speaker-lab 6 = **43 过**
- 拆后（worktree）：同样 **43 过，0 失败**
- `cargo clippy -p vdn-denoised -p vdn-capture -p vdn-speaker-lab --all-targets -- -D warnings`：无告警
- `xtask tidy`：三个 crate 只剩 `file-size-baseline.txt` 里 3 条「已降到 ≤600，删掉这条基线」（该文件归 hygiene）
- 拆前拆后代码行排序比对：差别只有模块声明、导入、`pub(crate)`、新文件首行说明、采样率替换

## 要注意
- denoised 的 `tests.rs` 搬出后被 tidy R3 当实时代码扫（里面有 `thread::sleep`）。照 `capture/src/testing.rs` 的先例，在文件头加了整模块 NRT 标注。更好的修法是 tidy 认出 `#[cfg(test)] mod x;` 引入的文件，属 hygiene 的 xtask，建议它评估
