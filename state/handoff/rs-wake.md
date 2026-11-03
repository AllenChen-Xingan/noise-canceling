# 现状页 · 手下 rs-wake（批次 20261102-b8-rust）

- **手上**：vdn-shm · vdn-rt-thread · xtask（大总管 2026-11-02 加派）
- **做到哪**：notify / wait · 头部版本 2 · OwnedBus · tidy 补查 bin 根 已完成；下一步对照实验、cargo-deny、Miri
- **下一步**：申请名额 → 装 cargo-deny 并跑 → 装 Miri 并跑 → release 对照实验 → 自审

## 流水
- 2026-11-02 报到；读交代、common、latency-fix、009 decision、rs-shm 现状页、规约 §4–§5
- 2026-11-02 本机 macOS 27.0.1；libc 0.2.191 已有 `os_sync_*` 声明，但**不直接链接**：改用运行期 `dlsym` 找，否则低于 14.4 的系统上整个程序启动失败、没机会退回轮询（读 SDK 头文件 `os/os_sync_wait_on_address.h` 的可用性标注推断，未在 14.2 实测）
- 2026-11-02 头部版本 1 → 2（`layout.rs`）：生产方行加 `wake`（u32，偏移 16）；消费方行加 `wait_fallbacks`（偏移 16）· `gaps`（偏移 24）；配置行加 `lead`（32）· `denoise_block`（36）· `session`（40）。各行仍是 64 字节，其余偏移不变
- 2026-11-02 新文件：`wake.rs`（全部 FFI unsafe，带 SAFETY）· `wait.rs`（`Bus::wait`，整模块非实时）· `owned.rs`（`OwnedBus`，按 rs-entry 原件思路自写：flock 文件锁 `$TMPDIR/vdn-shm_<名>.lock`）
- 2026-11-02 `Bus::notify` 放在实时 deny 块里；`no_alloc` 测试已覆盖 notify（不分配）
- 2026-11-02 测试：`tests/wake.rs` 10 个（立刻返回 · 超时 · 数据不够被叫醒后接着等 · heap 两线程 2 万块不丢唤醒 · POSIX 两份映射 2 万块不丢唤醒 · 头部新字段 · 会话 ID · 缺口 · OwnedBus 三个）· `tests/fallback.rs` 1 个（假装不支持 → 轮询照样工作并计数）。`cargo test -p vdn-shm` 25 过 1 忽略；clippy 无警告；tidy 过
- 2026-11-02 签名发给大总管与 rs-entry：`state/handoff/rs-wake-api.md`
- 2026-11-02 xtask tidy：crate 根检查从「只查第一个存在的根」改为查全部编译根（lib.rs · main.rs · src/bin/*.rs · src/bin/*/main.rs），加单测；`cargo test -p xtask` 10 过；tidy 过

- 2026-11-02 对照实验程序 `src/shm/examples/wake_bench.rs`：两个真进程（父 = 生产者，实时优先级；子 = 消费者），每 2 ms 推 96 帧，时间戳编进样本；统计延迟 p50/p99/max、系统计的唤醒次数（`proc_pid_rusage` 空闲 + 中断唤醒）、消费者 CPU。debug 冒烟 1 s：唤醒 p50 约 10–18 µs；500 µs 轮询 p99 约 44 ms（debug、1 s，仅冒烟，以 release 表为准）
- 2026-11-02 发现：macOS 的 `getrusage` 不填上下文切换数（读回 0），改用 `proc_pid_rusage`（libc 有结构体没声明函数，实验程序里自写一行 extern）
- 2026-11-02 Miri 准备：Miri 下唤醒走「不支持」分支（不调 dlsym）；POSIX / 真实时钟测试标 `cfg_attr(miri, ignore)`；heap 两线程测试 Miri 下 200 块
- 2026-11-02 xtask 新增 `miri` 子命令（`miri.rs`，默认 vdn-shm，不进 `ci`）；本机尚无 nightly，命令报「先装」并退出 1
- 2026-11-02 名额排队中（rs-entry 在做 V7 延迟测量）；自审 3 个子代理已派（睡死 · 丢唤醒 · 版本不兼容）

## 新增依赖
- 无新外部依赖（libc 已有）。vdn-shm 加 dev-dependency `vdn-rt-thread`（只给实验程序提优先级；check-deps 只看普通依赖，过）

## 待人拍板
（实验后填）
