# 调研笔记原件（Workflow wf_ec6b3516-162，2026-10-30）

8 个子代理的原始笔记，作为 `specs/rust/architecture.md` 与 `adr/` 的出处。**读法**：

- r1–r6 是六块检索；**r7（怀疑派）与 r8（完整性批评）对它们做了更正，冲突时以 r7 / r8 为准**。最重要的更正：
  - cargo-deny `wrappers` 只查**直接**依赖（r7 C10），r1 / r2 / r6 里「wrappers 证明 engine 不碰设备层」不成立
  - crate 目录下的 clippy.toml 整份替换上层，不合并（r7 C7）
  - `unsafe_code = "forbid"` + 局部 `#[allow]` 写不通（r7 C9）
  - rustc 内置 RTSan（PR #147935）已合入 nightly 1.93，仍 unstable（r7 C3）
  - ADC 2024 RTSan 演讲讲者是 David Trevelyan 与 Chris Apple（r7 C17）
  - `resources/fastenhancer` 就是 FastEnhancer 官方源码（r7 C25）
  - ort 每帧约 1554 次 malloc、376 对 mutex（r8 §3.1，证据原件见 `../appendix-probes/`）
- 笔记里的 `S/` = 当时的 scratchpad（已不在），探针已复制到 `../appendix-probes/`；`R/` = 主目录 `resources/`。
- 这是过程材料，不再修改；结论以规约和 ADR 为准。
