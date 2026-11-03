# 现状页 · 手下 rs-wake（批次 20261102-b8-rust）

- **手上**：vdn-shm · vdn-rt-thread · xtask（大总管 2026-11-02 加派）
- **做到哪**：已收尾（见文末「已收尾」）；只剩 tidy「每个 crate 至少一条测试」抓到 3 个别人的 crate，待大总管定
- **下一步**（按序）：自审修完 → 名额 ① 对照实验 ② cargo-deny ③ Miri → 加派三件：loom 测试 + deny.toml 直接边 + tidy 校验 deny 与 forbid 一致 · rt-thread 加入 Audio Workgroup · CI 拆 Linux + macOS（paths 过滤）
- **本批不改**：共享内存权限 0600（要真装驱动实测，大总管 2026-11-02 定）

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
- 2026-11-02 名额排队中（rs-entry 在做 V7 延迟测量）
- 2026-11-02 自审（3 个子代理，只读）：睡死 · 丢唤醒 · 版本不兼容三视角均无 P0。已修：
  - P1 新旧建方锁文件不同、互相锁不住，可能删掉对方的总线 → `OwnedBus` 过渡期两把锁都拿，加测试 `legacy_owner_lock_is_respected`
  - P2 唤醒改用 `os_sync_wake_by_address_all`（单等待者时代价相同；残留的第二个等待者不会让真消费方漏醒）
  - P2 单次系统调用最多睡 1 小时（防极大超时换算溢出后忙等）
  - P2 锁步 ping-pong 测试（`lockstep_heap` / `lockstep_posix_two_mappings`）；**反向验证**：把读序对调后它没抓到（窗口太窄），于是加单元测试 `wait::tests::widened_race_window_does_not_lose_wakeup`（只在单元测试里在两次读之间停 3 ms），对调后失败、改回通过——实测
  - P3 连续 3 次暂时性错误（EINTR 等）只睡一次、不计 `wait_fallbacks` · 等待端改用 `WAIT_*` 常量 · 名字检查改为不映射别人的总线 · 新增 `Bus::push_notify` · 补 `Poll(0)` 超时测试
  - 未改：多个等待者只是约定（单消费者），文档写明
- 2026-11-02 `cargo test -p vdn-shm` 30 过 1 忽略（loom 占位）；clippy 无警告；tidy 过
- 2026-11-02 xtask ci 加 `--platform linux|macos`（不给 = 全部，本机用法不变）：linux 排除 vdn-io-macos / vdn-capture；macos 只跑 io-macos · shm · rt-thread · hal 的 clippy（aarch64-apple-darwin）与测试

## 新增依赖
- 无新外部依赖（libc 已有）。vdn-shm 加 dev-dependency `vdn-rt-thread`（只给实验程序提优先级；check-deps 只看普通依赖，过）

## 流水（续）
- 2026-11-02 名额 ①：对照实验（结果见下节）；名额已还
- 2026-11-02 名额 ②：cargo-deny 0.20.2 装到 `~/.cargo/bin`；`cargo deny check` 通过（advisories · bans · licenses · sources ok）。警告只剩重复版本：thiserror / thiserror-impl / syn / jni-sys 各两个版本，都经 cpal 进来（不处理，大总管定）。按大总管：删掉 deny.toml 里没用到的 4 个许可（GPL-3.0 · CC0-1.0 · BSL-1.0 · MPL-2.0）与 1 个例外（assert_no_alloc），复跑仍通过。`xtask ci`：没装 cargo-deny 改为失败；ci.yml 的 Linux job 先用 `taiki-e/install-action` 装它
- 2026-11-02 名额 ③：loom（release，`LOOM_MAX_PREEMPTIONS=3`）2 项过，2.5 s；debug 不设上限跑 5 分钟没跑完，已停。反向验证：快照读序对调后 loom 报「唤醒丢了」，改回通过（实测）。Miri（nightly 2026-11-01）：进程内测试全过、无未定义行为报告；no_alloc 一项 Miri 下 352 s。Miri 暴露并修了两处：会话 ID 取系统时间（Miri 下不取）· 锁步测试的计时上限（Miri 下不查）
- 2026-11-02 D67：`wake.rs` 改为直接调 libc 的 `os_sync_*`；测试钩子改为 `force_wake_error_for_test`（模拟系统调用出错）。注意：本仓没设 `MACOSX_DEPLOYMENT_TARGET`，Rust 默认 11.0；低于 14.4 的系统上程序会因找不到符号而启动失败（符合 D67），要不要在 `.cargo/config.toml` 里写死 14.4 由大总管定（不是我的文件）

- 2026-11-02 vdn-rt-thread 加 `WorkInterval`（`workgroup.rs`）：降噪进程属 `AudioWorkInterval.h` 头注释第 3 种情况（实时线程与设备 IO 不同步、在别的进程），用 `AudioWorkIntervalCreate` 自建工作区间，**不需要设备 IOProc**；`join` 返回 `Member`（不能跨线程，drop 时离开），每块 `begin(截止期 ns)` / `finish()`，实时可调。另加 `set_qos_user_interactive()`（对照用）、`RtError::Unavailable`。`join_audio_workgroup`（设备的 IO 工作组）仍返回 Unimplemented，并写明它只在送音进程里拿得到、属 vdn-io-macos。新链接 AudioToolbox 框架（系统自带）
- 2026-11-02 测试 `tests/workgroup.rs` 4 项过（实测）：建 · 加入 · 100 块 begin/finish · 加入后 time-constraint 仍在 · 错误路径（没 begin 就 finish、连着两次 begin、同一线程重复加入，系统都返回错误，我们原样报出）· QoS
- 2026-11-02 task 3：提优先级对醒来延迟有影响（对照实验实测）→ 补测试 `promote.rs::promotion_cuts_wakeup_lateness`（`#[ignore]`，依赖机器负载，人工步骤写在注释里）
- 2026-11-02 实验程序 `src/rt-thread/examples/rt_bench.rs`：固定计算量代替推理（约 700 µs，启动时标定），每 16 ms 一块，四种设置 none / tc / tc_wg / qos；记块耗时 p50 / p99.9 / max、醒来迟到、策略降级次数、工作区间报错数、CPU %。time-constraint 的 computation 取 2.1 ms（块计算量的 3 倍、不超过周期一半）
- 2026-11-02 耗电脚本 `../noise_canceling-skeleton/_tmp/20261102-b8-rust/power/run.sh`（人用 `sudo bash run.sh` 跑）：基线空跑 + none / tc / tc_wg 各 60 s，powermetrics 每 1 s 采 CPU 功耗；只有 powermetrics 用 root；大总管已审过。等名额构建 rt_bench
- 2026-11-02 xtask tidy 新规则（代码已写，等名额编译验证）：① 每个 crate 至少一条测试；② deny.toml 直接边规则与真实依赖、forbid 一致（`denycheck.rs`）。deny.toml 补两条直接边：ort 只许 backend-ort · eval · speaker · speaker-lab；cpal 只许 io-macos。注意：审查原话是「ort 只许 vdn-backend-ort」，但 eval（规约 §7.1 第 8 条登记的例外）、speaker、speaker-lab 现在都直接用 ort，我按现状列名单；要不要收紧到只剩后端，归它们的负责人。hound / clap 的直接边本批不加（rs-entry 的 vdn-capture 直接依赖它们，加了合并就红）
- 2026-11-02 唯一真相源：`tests/layout_doc.rs`（等名额编译验证）——头注释里的字段清单、版本历史由测试与结构比对；偏移由 `offset_of!` 编译期断言钉死

- 2026-11-02 D68 落进 vdn-rt-thread：`ThreadPolicy::default()`（实时优先级开、工作区间不加入，开关都在）· `RtParams::from_measured(周期, 窗口总耗时 p99.9, MEASURED_MARGIN = 1.5)`。周期按 16 ms 报（大总管二选一定 A）：computation 要盖住每 16 ms 窗口的总工作量（1 块推理约 3 ms + 7 块轻处理），约 4 ms → 6 ms；不按 2 ms 报，因为推理那一块单独就超过 2 ms。测试 `default_policy_follows_d68` · `from_measured_adds_margin_and_clamps`（未编译）
- 2026-11-02 发现（推断，未核实）：读回策略可能看不到系统内部的降级——耗电对照里声明 computation 2.1 ms、实际每块 3.0 ms，每秒读回仍是 time-constraint、降级计数 0。所以降级计数只当下限，直接判据是醒来迟到与处理超时（已写进 `RtStatus::time_constraint` 文档；rs-entry 写进 chain-test / V7 输出说明）
- 2026-11-02 `qos_set` 实验按 D68 取消，未跑；rt_bench 留作 example（ci 只编译不运行）
 vs 周期轮询（实测，2026-11-02）

**测量条件**：本机 macOS 27.0.1，10 核（4 性能核 + 6 能效核）；release；`./target/release/examples/wake_bench all 10`（`src/shm/examples/wake_bench.rs`）。
两个真进程走真 POSIX 共享内存：父进程 = 生产者（提成实时优先级，模拟送音回调），每 2 ms 推 96 帧，推之前取时间戳编进样本；`wake` 时推完 `notify`。
子进程 = 消费者（模拟降噪进程），`wait(Raw, 96, 500 ms)` 返回后立刻取时间。「从推到醒」= 醒来时间 − 时间戳（同一个全系统时钟 `CLOCK_UPTIME_RAW`）。
每组 10 s，去掉前 100 块，剩 4900 块；每组只跑 1 次（未重复，没有置信区间）。
「后台压力」= 另一个进程跑满 8 个核。「每秒醒来次数」取自 `proc_pid_rusage` 的空闲唤醒 + 中断唤醒（活动监视器「空闲唤醒」那一栏的来源）；
**唤醒方式读成 0 是因为这一栏不计「被别的线程叫醒」**，按构造它每块醒 1 次，约 500 次/秒。「消费者 CPU %」= 消费者进程用户 + 系统时间 / 墙钟时间。

| 方式 | 消费者实时优先级 | 后台压力 | 块数 | p50 µs | p99 µs | max µs | 每秒醒来次数 | 消费者 CPU % | 退回轮询 |
|---|---|---|---|---|---|---|---|---|---|
| wake | 否 | 无 | 4900 | 17.0 | 29.5 | 9351.2 | 0 | 0.49 | 0 |
| poll200 | 否 | 无 | 4900 | 124.8 | 258.9 | 4532.7 | 5785 | 1.65 | 0 |
| poll500 | 否 | 无 | 4900 | 330.5 | 635.0 | 654.9 | 2360 | 1.50 | 0 |
| wake | 是 | 无 | 4900 | 9.9 | 22.0 | 40.6 | 0 | 0.41 | 0 |
| poll200 | 是 | 无 | 4900 | 102.5 | 203.8 | 210.9 | 7354 | 1.62 | 0 |
| poll500 | 是 | 无 | 4900 | 249.7 | 505.8 | 527.0 | 2902 | 1.30 | 0 |
| wake | 否 | 有 | 4900 | 6.4 | 2524.0 | 11247.4 | 0 | 0.24 | 0 |
| poll200 | 否 | 有 | 4900 | 137.7 | 2049.6 | 9538.7 | 3671 | 1.43 | 0 |
| poll500 | 否 | 有 | 4900 | 318.4 | 1585.4 | 6486.0 | 1548 | 0.66 | 0 |
| wake | 是 | 有 | 4900 | 4.3 | 5.5 | 7.4 | 0 | 0.21 | 0 |
| poll200 | 是 | 有 | 4900 | 101.1 | 200.7 | 203.8 | 4938 | 1.03 | 0 |
| poll500 | 是 | 有 | 4900 | 251.7 | 497.7 | 502.8 | 1988 | 0.48 | 0 |

**结论（实测，单次）**：
- 唤醒比轮询快：中位数快约 0.1 ms（对 200 µs 轮询）到 0.25–0.33 ms（对现在的 500 µs 轮询）；消费者提了实时优先级时，最长一次快 0.17–0.49 ms
- CPU：唤醒 0.2–0.5 %，轮询 0.5–1.65 %（差约 1 个百分点）；轮询每秒醒 1500–7400 次，唤醒约 500 次
- **实时优先级比唤醒方式影响更大**：消费者不提优先级时，三种方式都有几毫秒的尖峰（最长 4.5–11.2 ms，后台压力下唤醒 p99 2.5 ms）；提了之后唤醒最长 7–41 µs、轮询最长 ≈ 间隔本身。→ 降噪线程必须提实时优先级（交给 rs-entry 接线；vdn-rt-thread 补测试见下）
- 都在 1 ms 以内：按 009 待人拍板 1 的规则（「差不到 1 ms、耗电差不明显就选 B」）应选 B。但本实现在运行期找系统函数：14.4 以上自动用唤醒，14.2–14.3 自动退回 200 µs 轮询，所以**不提最低系统版本也能在新系统上拿到唤醒**

- 2026-11-02 rt_bench 改成组合式设置（qos · tc · wg · wg@<截止期 µs>，用 + 连），每块记是否在性能核上（`pthread_cpu_number_np`；「核号 ≥ 能效核个数 = 性能核」按本机 powermetrics 的编号规律，换机器要核对）；`qos_set` 一次跑 8 档。未编译
- 2026-11-02 排队（大总管定）：e3 基准约 17:56 结束 → ① 我的 debug 编译加测试（rt-thread 新测试 · xtask tidy 新规则 · shm 布局交叉校验）→ ② rs-entry 测试 ×5 → ③ 我的 release rt_bench + `qos_set 20`（约 3 分钟，判据：块耗时能否回到约 0.7 ms）

## 耗电对照（人跑 powermetrics，2026-11-02 17:03–17:08，每档 60 s、只跑 1 次）

正本（表 · 测量条件 · 文件）：`docs/experiments/exp-v7-latency/results/20261102-power/README.md`，这里不再抄表。
一句话（实测，单次）：提实时优先级把醒来迟到中位从 2.6 ms 降到 22 µs；三档每块都约 3.0 ms（能效核低频），工作区间没提速、功耗中位多约 27 mW。
→ 人定 D68：不追性能核 / 提频；实时优先级默认开、工作区间默认不加入（保留开关）；`qos_set` 实验取消

## 待人拍板
- （已定）最低系统 14.2 → 14.4：人定 D67（2026-11-02）。已改：系统函数直接链接，去掉按版本退回；只留系统调用出错时的轮询兜底

## 已收尾（2026-11-02）

**验收（名额 ①，实测）**：`cargo clippy -p vdn-shm -p vdn-rt-thread -p xtask --all-targets -- -D warnings` 过；`cargo test -p vdn-shm -p vdn-rt-thread` 全过；
`xtask tidy` 只剩 3 条：vdn-backend-ort · vdn-cli · vdn-speaker-lab 没有任何测试（新规则按设计报红，不归我改，待大总管定）。
另外实跑过：loom（release，抢占上限 3）· Miri（nightly，进程内测试）· cargo-deny · 对照实验 · 耗电对照（人跑）。

**改了哪些文件**
- `src/shm/**`：`layout.rs`（版本 2）· `bus.rs` · 新 `wait.rs` · `wake.rs` · `owned.rs` · `sync.rs` · `region.rs`（loom 版）· `lib.rs` · `Cargo.toml`（feature loom、example 要求 posix、dev 依赖 vdn-rt-thread）· `examples/wake_bench.rs` · 测试见下
- `src/rt-thread/**`：新 `workgroup.rs` · `lib.rs`（ThreadPolicy · from_measured · MEASURED_MARGIN · Unavailable）· `examples/rt_bench.rs` · 测试见下
- `src/xtask/**`：`tidy.rs`（查全部编译根 · 每 crate 有测试 · 接 denycheck）· 新 `denycheck.rs` · 新 `miri.rs` · `ci.rs`（--platform · cargo-deny 必跑）· `deps.rs`（direct_any）· `main.rs`
- `deny.toml`（删 4 个没用到的许可 + 1 个例外；加 ort / cpal 直接边）· `.github/workflows/ci.yml`（Linux）· 新 `.github/workflows/ci-macos.yml`
- `docs/experiments/exp-v7-latency/results/20261102-power/`（新）· `state/handoff/rs-wake.md` · `state/handoff/rs-wake-api.md`
- 仓外：`noise_canceling-skeleton/_tmp/20261102-b8-rust/power/run.sh`（大总管的 `_tmp`）

**本次新增的测试**
- vdn-shm：`tests/wake.rs`（立刻返回 · 超时 · Poll(0) 超时 · 容量钳制 · 两线程 / 两份映射 2 万块不丢唤醒 · 锁步 heap / POSIX · 头部 v2 字段与会话 ID · 缺口计数 · lead / 块长越界 · OwnedBus 忙 / 删名 / 清残留 / 旧锁 / 坏名字）· `tests/fallback.rs`（系统调用出错 → 轮询并计数）· `tests/loom.rs`（SPSC 回绕 · 不丢唤醒，均反向验证过）· `tests/layout_doc.rs`（2 项）· `src/wait.rs` 单元测试（撑大窗口，反向验证过）· `no_alloc.rs` 加 notify / gaps
- vdn-rt-thread：`tests/workgroup.rs`（6 项：建 / 加入 / begin-finish · 错误路径 · 坏名字 · 嵌套逆序离开 · 多线程共用 · QoS）· `promote.rs` 加 `default_policy_follows_d68` · `from_measured_adds_margin_and_clamps` · `promotion_cuts_wakeup_lateness`（`#[ignore]`，人工步骤在注释里）
- xtask：`every_crate_root_is_checked` · `parses_one_line_bans` · `crate_without_tests_is_detected`

**新增的第二处表示及其校验**（唯一真相源规矩）
| 第二处 | 正本 | 怎么校验 |
|---|---|---|
| `layout.rs` 头注释里的字段清单 | `layout.rs` 里的 Rust 结构 | `tests/layout_doc.rs`：逐行比字段名与顺序 |
| `VERSION` 上方的版本历史 | `VERSION` 常量 | `tests/layout_doc.rs`：最后一条编号 = VERSION |
| 头部各字段偏移 | Rust 结构 | `layout.rs` 编译期 `offset_of!` 断言（改结构不改断言编译不过） |
| `deny.toml` 的直接边名单 | 各 crate 的真实依赖 + `metadata.vdn.forbid` | xtask tidy（`denycheck.rs`）三条检查；cargo-deny 也查 |
| `JoinToken` 的大小与对齐 | SDK `os/workgroup_base.h` | 编译期断言 40 字节 / 对齐 4（数字照头文件抄，**不会随 SDK 自动更新**，SDK 变了要人核） |
| `ThreadPolicy::default()` 的取值 | 决策 D68 | 测试 `default_policy_follows_d68` |
| `state/handoff/rs-wake-api.md` 的签名 | 源码 | **无自动校验**；文件头已写明「快照，以源码为准」 |
| 耗电结果表 | `docs/.../20261102-power/README.md` | 现状页已删掉抄的表，只留指针 |

**待人拍板**：无新的。（最低系统 14.4 已由 D67 定；耗电与调度默认值已由 D68 定。）
**交给大总管定的技术事**：① tidy 抓到的 3 个无测试 crate（A 负责人补 · B 临时名单）；② 要不要在 `.cargo/config.toml` 写死 `MACOSX_DEPLOYMENT_TARGET = 14.4`；③ ort 直接边名单要不要收紧到只剩 backend-ort（现在按现状列了 4 个）；④ hound / clap 的直接边（rs-entry 的 vdn-capture 直接用它们）；⑤ 共享内存 0600 权限（本批不改）
**本批没做 / 没验证**：`ci.yml` 的 Linux job 没在 GitHub 上真跑过（ort 下载、Linux 下各 crate 能否编译未实测）；rt_bench 的 `qos_set` 按 D68 取消；跨进程加入设备 IO 工作组（属 vdn-io-macos）
