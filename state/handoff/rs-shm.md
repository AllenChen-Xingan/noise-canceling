# 现状页 · 手下 rs-shm（批次 20261102-b8-rust）

- **手上**：vdn-shm（新）· vdn-rt-thread（新）· xtask（闸门）
- **做到哪**：已收尾（见文末）
- **下一步**：等大总管验收

## 流水
- 2026-11-02 报到；读 010 第十 / 十一节、规约 §2–§7、009 decision 相关行、现有 xtask 与 ci.yml
- 2026-11-02 本机工具：cargo 1.99.1；Miri 不可用（stable 1.99.1 无 miri 组件，没装 nightly，按交代不硬装）；cargo-deny 未装（按交代不自己装）
- 2026-11-02 依赖：libc 0.2（MIT OR Apache-2.0，已在 Cargo.lock）· mach2 0.6.0（BSD-2-Clause OR MIT OR Apache-2.0，本地缓存有）——均与 GPL-3.0 兼容
- 2026-11-02 vdn-shm 建成：layout.rs（头部布局 + 编译期断言）· region.rs（全部 unsafe：堆 / mmap）· bus.rs（Bus / Ring，无 unsafe）；测试 ring.rs（8 过 1 忽略 loom）· posix.rs（3 过，含两线程 10 万块）· no_alloc.rs（1 过）
- 2026-11-02 vdn-rt-thread 建成：promote_current / check_current（mach2）· join_audio_workgroup → Unimplemented；测试 4 过（含起线程提升后读回）
- 2026-11-02 签名发给大总管、rs-io、rs-entry（state/handoff/rs-shm-api.md）；大总管认可 check_current 返回 Result，并让我在根 Cargo.toml 加 vdn-shm / vdn-rt-thread 两行 workspace 依赖（已加）
- 2026-11-02 xtask：新增 `ci`（ci.rs）· `todo`（todo.rs）；tidy 加 SAFETY 检查（unsafe=true 的 crate）；deny.toml 建好（未实跑：本机没装 cargo-deny）；ci.yml 改调 `xtask ci`
- 2026-11-02 Miri：未实跑（stable 1.99.1 无 miri 组件，没装 nightly）。loom：本批不做（ring.rs 里 #[ignore = "待实现：loom…"]）
- 2026-11-02 拿名额跑 `cargo run -p xtask -- ci`：全绿，23 s（fmt · clippy · test · tidy · check-deps 过；cargo-deny 跳过，未装）；名额已归还
- 2026-11-02 自审（子代理，内存序 · 跨进程布局 · 欠载溢出三视角）：无 P0。已修：`available()` 读序（P2）· 新增 `pop_or_zero`（余下填 0 并计欠载，P2）· 文档写明「环满丢最新、消费方按 l_total 用 discard 重对齐」并加测试（P2）· 补布局偏移断言与 64 位原子断言、改 MAGIC 注释（P3）。未修、报大总管：`Ring` 是 Copy、两端靠约定（拆 Producer / Consumer 要改接口）；0600 权限下 coreaudiod 里的驱动可能打不开（P1，见待人拍板）；对端写坏索引无告警计数、创建方崩溃后驱动需按心跳重新 open（P3，后续批次）

## 待人拍板
| 题 | 选项 | 人能感知的后果 | 建议 | 不回的默认 |
|---|---|---|---|---|
| 虚拟麦克风驱动怎样打开共享内存（与 009 待人拍板 3 是同一件事，这里只补一条新发现） | A 现在就换成 Mach 服务交接 · B 开发期把权限放宽到 0644 · C 保持 0600，等装驱动那批再定 | 0600 时：驱动（跑在系统音频服务的用户身份下）很可能打不开总线，会议软件里虚拟麦克风没声音（推测，未实测）。0644 时：同一台机器上任何程序都能读到你的麦克风声音，不能发布 | C：本批没有真驱动，不影响「送声音出声音」；装驱动那批先实测一次 | C |

## 已收尾
- 改了哪些文件：`src/shm/**`（新）· `src/rt-thread/**`（新）· `src/xtask/src/{main.rs,tidy.rs,ci.rs,todo.rs}` · `deny.toml`（新）· `.github/workflows/ci.yml` · 根 `Cargo.toml`（两个成员；libc · mach2 · vdn-shm · vdn-rt-thread 四行 workspace 依赖）· `state/handoff/rs-shm{,-api}.md`
- 新 crate 实体行（供大总管写进 `.42cog/cog.md`）：
  - `vdn-shm`：跨进程共享内存总线（头部 + Raw / Processed / Out 三个环 + 心跳 + 计数器）。由降噪进程 `create`；收音进程与虚拟麦克风 `open`。Raw 是降噪进程的输入，Processed 是收音进程的输入，Out 是虚拟麦克风（开发期扬声器）的输入
  - `vdn-rt-thread`：把线程提成实时优先级并读回自检。收音进程的回调线程与降噪进程的处理线程在启动时调用；不产出数据
- 未实跑：Miri（无 nightly）· cargo-deny（未装）· loom（本批不做）。CI 的 Linux / macOS 拆分本批不做
