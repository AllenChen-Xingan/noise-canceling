# 现状页 · 手下 rs-shm（批次 20261102-b8-rust）

- **手上**：vdn-shm（新）· vdn-rt-thread（新）· xtask（闸门）
- **做到哪**：读完交代与规约，开始写 vdn-shm
- **下一步**：Bus 接口编译通过 → 通知大总管、rs-entry、rs-io

## 流水
- 2026-11-02 报到；读 010 第十 / 十一节、规约 §2–§7、009 decision 相关行、现有 xtask 与 ci.yml
- 2026-11-02 本机工具：cargo 1.99.1；Miri 不可用（stable 1.99.1 无 miri 组件，没装 nightly，按交代不硬装）；cargo-deny 未装（按交代不自己装）
- 2026-11-02 依赖：libc 0.2（MIT OR Apache-2.0，已在 Cargo.lock）· mach2 0.6.0（BSD-2-Clause OR MIT OR Apache-2.0，本地缓存有）——均与 GPL-3.0 兼容
- 2026-11-02 vdn-shm 建成：layout.rs（头部布局 + 编译期断言）· region.rs（全部 unsafe：堆 / mmap）· bus.rs（Bus / Ring，无 unsafe）；测试 ring.rs（8 过 1 忽略 loom）· posix.rs（3 过，含两线程 10 万块）· no_alloc.rs（1 过）
- 2026-11-02 vdn-rt-thread 建成：promote_current / check_current（mach2）· join_audio_workgroup → Unimplemented；测试 4 过（含起线程提升后读回）
- 2026-11-02 签名发给大总管、rs-io、rs-entry（state/handoff/rs-shm-api.md）；大总管认可 check_current 返回 Result，并让我在根 Cargo.toml 加 vdn-shm / vdn-rt-thread 两行 workspace 依赖（已加）
- 2026-11-02 xtask：新增 `ci`（ci.rs）· `todo`（todo.rs）；tidy 加 SAFETY 检查（unsafe=true 的 crate）；deny.toml 建好（未实跑：本机没装 cargo-deny）；ci.yml 改调 `xtask ci`
- 2026-11-02 Miri：未实跑（stable 1.99.1 无 miri 组件，没装 nightly）。loom：本批不做（ring.rs 里 #[ignore = "待实现：loom…"]）
