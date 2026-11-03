# Rust 骨架对照审查：语法树 · 最佳实践（2026-11-02，只读审查员）

范围：skeleton（`feat/rust-skeleton` 093d293：core · shm · io 已合入）+ rs-entry（6ab0088）+ rs-shm（eae998a，跨进程唤醒进行中）。
路径前缀：`S/` = `noise_canceling-skeleton`，`E/` = `noise_canceling-rs-entry`，`H/` = `noise_canceling-rs-shm`。

## 1. 一句话结论
- **「用代码语法树守规矩」（ADR 004）：沿用了，且做得较全。** 每个新 crate 都有机读档位表、`forbid` 头、panic 类 lint 和实时块禁用清单；xtask 的 tidy / check-deps / ci 能跑。缺口在 cargo-deny 直接边、RTSan、loom。【读源码】
- **「领域超级语法树」（005 第三节 A：处理图 + 会话记录）：只做了一半。** 会话记录、版本号、内容 ID 已有（在早先的 `vdn-types` 里，本批没动）。处理图没有：`Pipeline` 是写死的一条链，不认识 `Stage`，不能挂节点。005 第五节第 1 题人也还没答。【读源码】【读文档】
- **009 的 Rust 最佳实践：大方向沿用，有几处有理由的偏离，几处没理由或还没做。** 推理设置、零分配、自写 SPSC 共享内存、用 cpal 过渡都照 009 做。重采样没用 rubato 而是自写，理由是延迟更低。进程内的 rtrb / triple_buffer 还没用上（本批没有控制线程到实时线程的通道）。FTZ 守卫、Audio Workgroup、loom 都没做。【读源码】

## 2. 表一 · 代码语法树守规矩（规约 §5 · §6 · ADR 004）

| 规则 | 状态 | 证据 / 说明 |
|---|---|---|
| R1 不分配 | 做到 | dsp / engine / shm / io-macos / hal 都有 `tests/no_alloc.rs`，capture、denoised 有计数测试（`E/src/capture/src/tests.rs`）【读源码】 |
| R2 不加锁 | 做到（静态） | 根 `clippy.toml:33` 禁用类型；实时块 deny（如 `S/src/io-macos/src/pump.rs:67`、`H/src/shm/src/bus.rs:258`）。cpal 内部 `try_lock` 已列进已知违规清单（`S/state/handoff/rs-io.md:82,84`）【读源码】 |
| R3 不做 IO / sleep | 部分 | tidy 查 A/B 档（`H/src/xtask/src/tidy.rs:10`）。降噪循环空闲时 `thread::sleep(500µs)`（`E/src/denoised/src/main.rs:218`），整文件标「非实时」放行（`E/state/handoff/rs-entry.md:64`）；rs-shm 正在加唤醒来替换【读源码】 |
| R4 不 panic | 做到 | 所有 A/B 档 crate 根有 `cfg_attr(not(test), deny(unwrap_used…))`（如 `H/src/shm/src/lib.rs:29`、`S/src/backend-ort/src/lib.rs:8`）；tidy 查实时块里的 `assert*!`【读源码】 |
| R5 最坏情况有界 · bench | 没做 | 无 `benches/` 目录；降噪进程只打印单块最长耗时（`E/src/denoised/src/main.rs:226`）【读源码】 |
| R6 跨线程只用 §3.4 原语 | 部分 | 跨进程按规约自写 `AtomicU32` 环（`H/src/shm/src/lib.rs:14`）✓；但 loom 是空测试 `#[ignore]`（`H/src/shm/tests/ring.rs:171`），Miri 未跑（`H/state/handoff/rs-shm.md:15`）【读源码】 |
| R7 实时线程不 drop | 未涉及 | 本批没有换模型 / 回收队列【读源码】 |
| R8 实时优先级 + Workgroup | 部分 | `promote_current` 已用（`E/src/denoised/src/main.rs:207`）；Workgroup 返回 `Unimplemented`（`H/src/rt-thread/src/lib.rs:6,84`），010 第十节约定如此【读源码】 |
| R9 FTZ 守卫只包 DSP | 没做 | dsp 里查不到 FTZ 守卫（grep `ftz` 无结果）；规约 §2.1 把它列为 dsp 积木【读源码】 |
| R10 unsafe 白名单 | 做到 | 非白名单 crate 有 `#![forbid(unsafe_code)]`（如 `S/src/io-macos/src/lib.rs:11`）；白名单 shm / rt-thread / hal 局部 allow 并写 SAFETY（`H/src/shm/src/wake.rs:90`、`S/src/hal/src/factory.rs:23`）；tidy 查 SAFETY【读源码】 |
| R11 出错送原声 + `Status` | 部分 | 进程级做了：心跳 100 ms 不动切原声（`E/src/capture/src/lib.rs:125`）；hal 有 `#[must_use] RenderStatus`（`S/src/hal/src/source.rs:30`）。模型级仍在 16 kHz 的 Engine 里，`Pipeline::process` 不回报 `Status`（`S/src/engine/src/pipeline.rs:23-24` 自注「归后续批次」）【读源码】 |
| R12 依赖方向 | 部分 | 每个 crate 有 `forbid`，check-deps 查闭包并抓到过一次（`E/state/handoff/rs-entry.md:122`）；**cargo-deny 的直接边规则空着**（`S/deny.toml:45`「本批未做」）；tidy 不查 deny.toml 与 forbid 一致（规约 §5.4 要求）【读源码】 |
| R13 延迟有账 | 部分 | `Pipeline::latency_samples()` 有逐样本实测断言（`S/src/engine/tests/pipeline.rs:85-88`）；但 ≤45 ms 的总断言仍按模型 + 预算算（`S/src/models/tests/latency.rs:3`），规约要求接入 Pipeline 后改断言 Pipeline【读源码】 |
| R14 构建产物不进库 | 做到 | tidy 第一条查 `/target/`（`H/src/xtask/src/tidy.rs:5`）【读源码】 |
| §5.3 只一份 clippy.toml、实时块 deny | 做到 | 只有根 `clippy.toml`；workspace 设 allow（`S/Cargo.toml` lints 段），实时 impl 块升 deny【读源码】 |
| §5.4 `[package.metadata.vdn]` | 做到 | 13 个被审 crate 全有 tier / unsafe / forbid；tidy 校验存在性【读源码】。小瑕疵：capture 是 A 档却直接依赖 hound 与 clap（`E/src/capture/Cargo.toml:21-22`），规约 §2.2 列的 capture 闭包里没有它们 |
| §6「用」：crate/trait 拆分 · clippy · deny+check-deps · tidy | 做到（deny 直接边除外） | 见上【读源码】 |
| §6「用」：rtsan `#[nonblocking]` | 没做 | 全仓 grep `nonblocking` / `rtsan` 无结果【读源码】 |
| §6「用」：cbindgen | 不适用 | HAL 改用 Rust cdylib（`S/src/hal`），不需要 C 头【读源码】 |
| §6「不用」：自写 derive · build.rs · 宏 DSL | 守住 | 无 build.rs；`macro_rules!` 只有旧的 `each!`（`S/src/models/src/lib.rs:142`）【读源码】 |
| §6 借 Unison：sha256 当 ID | 偏离（旧代码） | `ContentId` 是 FNV-1a 64 位（`S/src/types/src/version.rs:4`），不是 sha256；非本批改动【读源码】 |
| §7 闸门 | 部分 | `xtask ci` 跑 G1 G2 G4 G5 G3(check-deps)；cargo-deny 没装就跳过（`H/src/xtask/src/ci.rs:39`）；G6 有随机切块测试（`S/src/engine/tests/pipeline.rs:128`）；G8 G9 G10 缺。CI 只有 macOS 一个 job（`S/.github/workflows/ci.yml:10`），规约要核心放 Linux【读源码】 |

## 3. 表二 · 领域统一语法树（005 第三节 A）

| 005 要的东西 | 现状 | 证据 | 离「能挂新节点」还差什么 |
|---|---|---|---|
| 处理图的数据结构（节点带类型 · 参数 · 版本） | 没有 | 全仓 grep `Graph` / `Node` 在 engine、types 无结果；`Pipeline` 是写死的「重分块 → 降采样 → Engine → 升采样 → 混合」（`S/src/engine/src/pipeline.rs:4-8`）【读源码】 | 缺一个描述链路的数据（节点表），以及从它造出 Pipeline 的入口 |
| 节点统一接口 | 有接口，没接上 | `trait Stage`（`S/src/types/src/stage.rs:6`）已有 5 个实现（speaker 的门控 / 包络、polish 修音链）；但 `Engine` / `Pipeline` 不持有任何 `Stage`（grep `dyn Stage` 只在 speaker-lab）【读源码】 | Engine 或 Pipeline 要能持有一串 `Stage` 并逐 hop 调用 |
| 版本号 | 有（表、配置） | `TableVersion` / `ProfileVersion` / `ContentId`（`S/src/types/src/version.rs:11-31`）【读源码】 | 没有「处理图版本」；会话头只记表和配置版本（`S/src/types/src/record.rs:385-390`），不记链路形状、模型 ID |
| 会话记录 | 有 | `SessionRecord` + `DecisionEvent`，文本可往返（`S/src/types/src/record.rs:69,394`）【读源码】 | 本批的 capture / denoised 没有写会话记录（grep 无 `SessionRecord` 使用）【读源码】 |
| 热更新参数 | 没有（只有构造期） | `SafetyConfig::apply_params` 注明「构造期，非实时」（`S/src/engine/src/engine.rs:74-77`）；`Stage::set_params` 存在但无人调用；控制命令 `Cmd` 定义了（`E/src/control/src/lib.rs:48`）但没送进实时线程的通道【读源码】 | 缺控制线程 → 实时线程的通道（规约 §3.4：rtrb / `AtomicU32`） |
| 反馈事件总线 | 没有 | 决策事件只是类型；009 §6「定长事件经 rtrb 送日志线程」未实现【读源码】 | 缺实时侧发事件、非实时侧落盘的那条队列 |
| Pipeline 能否表示成图 | 不能 | 同第一行；009 §6 明确建议「不要图，自写固定管线」【读文档】 | 两份文档有张力：009 不要运行时图库，005 要统一表示。可兼容：图只作「描述数据」，运行时仍是固定的 Stage 数组【推断】 |

## 4. 表三 · 009 最佳实践逐条（只列本批相关的）

| 009 推荐 | 本批用了什么 | 判定 | 偏离理由 |
|---|---|---|---|
| 麦克风：M0 用 cpal 0.18.2，M1 换自写 IOProc | cpal 0.18.2（`S/src/io-macos/Cargo.toml`），已知违规 6 条写成清单 | 一致 | 010 第十节：本批用 cpal【读文档】 |
| 虚拟麦克风：照 voice-strip 自写 Rust cdylib | `vdn-hal` cdylib，工厂返回空指针 | 一致（骨架） | 010：真驱动归后续批次 |
| 跨进程共享内存：shm_open+mmap、`#[repr(C)]` 全原子、voice-strip「时间轴 + 分槽序号」环 | shm_open+mmap+全原子 ✓；环是普通读写索引，没有时间轴 / 分槽序号（`H/src/shm/src/lib.rs:14-18`）【读源码】 | 部分偏离 | 现状页未写理由；漂移靠消费方 `discard` 重对齐（`lib.rs:18`） |
| 共享内存安全交接：xpc_shmem 交给驱动 | 命名 POSIX 共享内存，权限 0600（`H/src/shm/src/region.rs:145`） | 偏离 | 未写理由。0600 只有本用户能开，coreaudiod 下的 HAL 驱动大概率打不开【推断】 |
| 跨进程唤醒：`os_sync_wait_on_address` / wake，旧系统退轮询 | rs-shm 正在加：dlsym 取符号，<14.4 退 200 µs 轮询（`H/src/shm/src/wake.rs:1,57`）；denoised 仍 sleep 0.5 ms | 一致（进行中） | — |
| 实时优先级 + Workgroup | 优先级 ✓（mach2）；Workgroup 未做 | 部分 | 010 第十节约定本批 `Unimplemented` |
| 进程内环 rtrb · 最新值 triple_buffer · 单值 `AtomicU32` | 都没引入（全仓 grep 无 rtrb / triple_buffer） | 未涉及 | 本批没有控制线程 → 实时线程的通道；`Cmd` 只定义未接线【读源码】 |
| 断言不分配：计数分配器 | assert_no_alloc / stats_alloc（dev） | 一致 | — |
| RTSan（可选 feature） | 没有 | 未做 | 未查到理由 |
| FTZ 守卫只包 DSP | 没有 | 未做 | 未查到理由 |
| NaN / Inf 防线 | Engine 三件保命 + Pipeline 混合时 `!o.is_finite()` 切原声（`S/src/engine/src/pipeline.rs:325`） | 一致 | — |
| 不要 MPMC / seqlock crate | 没用 | 一致 | — |
| FFT：realfft | realfft（`S/src/dsp/Cargo.toml`） | 一致 | — |
| 重采样：rubato，目标每跳 ≤2 ms | 自写 97 抽头多相 FIR，每跳 1 ms（`S/src/dsp/src/resample.rs:1-13`） | 偏离（有理由） | 延迟比 009 目标还低、实时不分配；代价是阻带未优化，文件自注「待按 009 换掉或调参」【读源码】 |
| 漂移：rubato `Slip` + 自写估计 | 没有 | 未涉及 | 后续批次 |
| 推理：ort rc.15 · intra=1 · 关 spinning · PreferCPU | 全部照做（`S/src/backend-ort/src/lib.rs:52-62`） | 一致 | — |
| engine 生命周期：clack 式类型分开 activate/process/deactivate | `Pipeline::new / with_block` + `process` + `reset`，没有用类型把「可分配期」和「实时期」分开 | 部分 | 未写理由；靠注释和 clippy 块约束【读源码】 |
| 参数：每参数一个原子 + 稳定 ID + Smoother | 没有运行时参数 | 未涉及 | — |
| 延迟只在激活时定 | `latency_samples()` 在构造时定（`S/src/engine/src/pipeline.rs:225`） | 一致 | — |
| 日志：原子计数器 + 定长事件经 rtrb | 原子计数器有（shm 头、`EngineStats`）；事件队列无 | 部分 | — |
| 控制面 IPC：std `UnixStream` | 无（只读 JSON 配置文件，grep `UnixStream` 无结果） | 未涉及 | — |
| 工具链：cargo-deny · nextest · proptest · loom+Miri · criterion | cargo-deny 有配置但直接边空、没装就跳过；用 `cargo test` 不是 nextest；切块测试是手写随机不是 proptest；loom 空测试；无 criterion | 部分 | rs-shm 现状页：本机无 nightly、不硬装（`H/state/handoff/rs-shm.md:9,15`）；账本显示人已同意装 cargo-deny 与 Miri（093d293） |

## 5. 偏离清单（按影响排序）

1. **处理图不存在，`Stage` 没接进 Pipeline。** 后果：门控、修音链写好了却进不了实时链路；以后每加一种处理都要改 Pipeline 代码，正是 005 担心的「中后期复杂度失控」。另外 005 第五节第 1 题（超级语法树指哪种）人还没答。归 `vdn-engine`（Pipeline 持有 Stage 串）+ `vdn-types`（链路描述与版本）。【读源码】【读文档】
2. **跨进程环没有自动纠漂移，HAL 交接权限 0600。** 后果：两个设备时钟不同步时，延迟会慢慢涨到环满并停在最大（`H/src/shm/src/lib.rs:17-18`），用户感到对方听见自己越来越晚；真 HAL 驱动接上时可能打不开共享内存，虚拟麦克风没声【推断】。归 `vdn-shm`（布局）+ `vdn-capture`（消费方重对齐）。
3. **loom / Miri / RTSan / cargo-deny 直接边都还没跑。** 后果：自写无锁环的并发正确性只靠普通测试，偶发错样本或断声在现场才会暴露；依赖方向只有闭包检查一道。归 `vdn-shm`（loom + `sync` 门面）· `xtask`（deny 直接边、tidy 查 deny.toml 与 forbid 一致）。
4. **降噪循环 sleep 0.5 ms 轮询。** 后果：已观察到 10 ms 余量下「处理后欠载」，用延迟原声补上（`E/state/handoff/rs-entry.md:99-102`，原因未核实）；用户听到降噪时有时无。归 `vdn-denoised`（接 rs-shm 的唤醒）。
5. **FTZ 守卫与 Workgroup 未做。** 后果：极小信号时 DSP 可能变慢；推理线程不提核心频率，可能偶发超时转原声【推断，未量化】。归 `vdn-dsp`（FTZ）· `vdn-rt-thread`（Workgroup）。
6. **模型级送原声仍在 16 kHz Engine，Pipeline 不回报 `Status`；≤45 ms 总断言没改成断言 Pipeline。** 后果：维护上两层送原声口径不一；延迟账可能和真实链路不一致。归 `vdn-engine` · `vdn-models`（latency 测试）。
7. **控制命令没有实时通道（rtrb 未引入）。** 后果：运行中切「直通 / 降噪」只能重启进程。归 `vdn-control` + `vdn-denoised`。
8. **小项：** capture（A 档）直接依赖 hound / clap；`ContentId` 用 FNV 不是 sha256；CI 只有 macOS 一个 job（费用约为 Linux 的 10 倍，规约 §7）。归 `vdn-capture` · `vdn-types` · CI。

审查方式：只读，grep + 抽读关键段；未跑构建与测试，表中「做到」指代码与测试存在，不代表本次实跑通过。
