# 现状页 · 手下 rs-entry（批次 20261102-b8-rust）

> 大总管：`noise-canceling-47`。交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-entry.md`。
> 管的实体：`vdn-control` · `vdn-capture` · `vdn-denoised` · `vdn-cli`（可执行 `vdn`）

## 手上
- 延迟修复 A · C · D · E + V7（大总管 2026-11-02 加派，见 `../noise_canceling-skeleton/_tmp/20261102-b8-rust/latency-fix.md`）

## 判据结果（2026-11-02，本机 debug 构建，机器判；降噪块长 96，定块长 Pipeline）

| # | 判据 | 命令 | 结果 |
|---|---|---|---|
| V1 | 直通逐样本对齐 | `vdn chain-test --mode passthrough` | **过**：相关 1.000000（线 ≥ 0.999）· 实测延迟 2208 = L_total · 虚拟麦欠载 0 |
| V2 | 降噪死掉仍出声 | `vdn chain-test --kill-denoiser-at 2` | **过**：110.0 ms 切原声（线 ≤ 150）· 最长静音 0 ms；GTCRN 下同样 110.0 ms |
| V3 | 真模型出声 | `vdn chain-test --mode denoise --model-path <gtcrn_simple.onnx>` | **过**：均方根 0.1151（线 > 0.01）· 非有限 0 · 长度 240000 / 240000 |
| V4 | 两个真进程 | 见下 | **过**：相关 1.000000 · 实测延迟 2208 · 读出欠载 0 · 实时提升成功 · 结束后 unlink 成功 |
| V5 | 真麦克风（人判） | 见「V5 人工步骤」 | 待人做 |

- **延迟分项**（`with_block` 合入后）：
  - 直通：L_total = 2208 样本（46.00 ms）= Pipeline 768（16.00 ms）+ 环余量 1440（30.00 ms）
  - GTCRN：L_total = 2976 样本（62.00 ms）= Pipeline 1536（32.00 ms）+ 环余量 1440（30.00 ms）
  - 合入前（块长 480、`Pipeline::new`）：直通 2303（47.98 ms）· GTCRN 3071（63.98 ms）；V1–V4 当时也全过
- **加测 lead = 1 块（480，不作过线条件，只计数）**：
  - 降噪块长 96：直通 3 次「处理后欠载」1728 / 0 / 0 样本；GTCRN 3 次 480 / 480 / 1920 样本（缺的用延迟原声补，不是静音；虚拟麦欠载都是 0）。GTCRN 时 L_total = 2016 样本（42.00 ms）
  - 降噪块长 480（合入前）：直通、GTCRN 各 3 次全 0
  - 推断（未核实）：GTCRN 第一次推理慢（预热），10 ms 余量不够；debug 构建、降噪线程空闲时睡 0.5 ms 也有影响
- 坏块长：`--denoise-block 100` 启动即报错「块长 100 不整除 hop 768」【测】
- V3 用的权重：`../noise_canceling/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`（sha256 前缀 `b4718df6228e7bdf`；本工作区 `resources/` 里没有）
- V4 命令：
  ```
  ./target/debug/vdn-denoised --passthrough --bus /vdn-v4b --seconds 9 &
  ./target/debug/vdn-capture --source sine --sink file _tmp/20261102-rs-entry-v4/out2.wav --bus /vdn-v4b --seconds 5
  ./target/debug/vdn chain-check _tmp/20261102-rs-entry-v4/out2.wav --l-total 2208
  ```

## 延迟修复 · V7 基线（2026-11-02，release，降噪仍是 0.5 ms 轮询、未接唤醒）

做了什么：IO 块默认 480 → 96（A）· 降噪进程启动预热 64 块 + 每块耗时计数（C）· 全部在 release 下量（D）· chain-test 三个线程提 time-constraint 优先级 · `vdn v7` 子命令（N 次 × T 秒，空闲 + 压力两组，可扫 lead）

| lead | 组 | 有欠载的次数 | 处理后欠载合计 | 虚拟麦欠载 | 单块最长（中位 / 最大） | 平均每块 |
|---|---|---|---|---|---|---|
| 96（2 ms） | 空闲 | 10 / 10 | 222 432 样本 | 0 | 3.09 / 3.29 ms | 0.18 ms |
| 96 | 压力（7 个忙线程） | 1 / 10 | 864 | 0 | 1.43 / 2.17 ms | 0.10 ms |
| 144（3 ms） | 空闲 | 10 / 10 | 100 368 | 0 | 2.97 / 3.10 ms | 0.17 ms |
| 144 | 压力 | 0 / 10 | 0 | 0 | 0.64 / 0.81 ms | 0.05 ms |

- 正式跑 10 次 × 10 s；开始时 1 分钟负载 10.68（核数 10，有外部负载），结束 7.88
- 空闲加扫（3 次 × 10 s，负载 6.66 → 2.97）：lead 192 / 240 / 288 / 384 全部无欠载。只有 3 次，不作 V7 结论
- 更早的粗扫（3 次 × 5 s，负载 8.99 → 12.53）：96–480 全部无欠载——负载高时 CPU 频率高，反而干净
- **结论（实测）**：V7 不过。最小无欠载 lead：空闲下约 192（4 ms，3 次），压力下 144（10 次）
- **卡在哪（推断，未核实）**：空闲时每块处理慢约 2–3 倍（平均 0.18 vs 0.05 ms，推理那块 3.1 vs 0.8 ms），像是 CPU 降频或降噪线程落到能效核。对应审查清单第 5 条（推理线程没加入 Audio Workgroup、不提核心频率），归 vdn-rt-thread。轮询和余量不是主因
- 结果文件（不入库）：`_tmp/20261102-rs-entry-v7/{sweep-poll,v7-poll,idle-sweep-poll}.{json,txt}`
- 复测命令：`./target/release/vdn v7 --model-path <gtcrn_simple.onnx> --leads 96,144,192 --runs 10 --seconds 10 --json <路径>`

## 延迟修复 · 接线（2026-11-02，唤醒合入之后）

清单用途：每条写「做了什么 → 怎么证明」。

- **唤醒（B，rs-wake 的接口）**：送音端推完 Raw 就 `notify`；降噪进程 / chain-test / `vdn run` 改 `bus.wait(Raw, 降噪块, 20 ms)`，删掉 0.5 ms 轮询 → 本机「唤醒 可用 · 退回轮询 0 次」【测】
- **OwnedBus**：改用 `vdn_shm::OwnedBus`，denoised 里那份已删（同一段代码只留一份）
- **Out 总线拆分**：送音端自己建 `/vdn-out`（`Config.out_bus_name`）；插件 / 扬声器 / 文件都按名字连它。没有降噪总线就送延迟原声、后台 100 ms 重试；会话 ID 变了就换 → 两进程三种起停顺序（降噪先起 · 送音先起 2 s 后才起降噪 · 第 3 s SIGKILL 再起新的）文件全段相关 1.000000、延迟 1056、读出欠载 0、静音 0【测，debug】
- **降噪落后时跳过 Raw 积压**：可读量 > lead + 2 块就丢旧的只留一块（`Ring::discard`，O(1)），记 `note_gap`；在输出流「现在 + L_pipe」处推等长零占位，账不错位 → 单测：降噪停 100 块后恢复，不静音、跳变 < 0.13、0.2 s 内回到处理后的并逐样本对齐【测】

### 已修（大总管指出的两条规约红线，2026-11-02）
1. **音频线程不许 `try_lock`（§3.4）**：换总线改成 rtrb 交接（`vdn_capture::feeder`）：监视线程建好新总线 → 交接队列 → 音频线程在块边界接手 → 旧对象经回收队列送回监视线程释放；回收队列满就留在本地槽位、这块不接新的 → 单测 2 条（交接 / 回收满）；新增文本检查测试 `src/capture/tests/realtime_rules.rs`：capture 全部源文件没有 `Mutex` · `RwLock` · `Condvar` · `try_lock` · `.lock(` · `parking_lot` · `mpsc`
2. **不在声音里用 NaN 做记号**：跳过积压的占位改成零，告知走带外（Raw 环缺口计数）；送音端看到缺口计数涨了，就把接下来 L_total + 2 块当心跳过期（淡到原声、回滞后淡回），零占位不会播出。另加防御：处理后的样本不是有限值就当没有（计数 `nonfinite`，单测里 = 0）

### 新增的第二处表示及其校验（CLAUDE.md 第五节第 5 条）
| 第二处 | 正本 | 校验 |
|---|---|---|
| `vdn_control::validate_bus_name`（配置文件早报错） | vdn-shm 的名字规则（`Bus::open` 报 `InvalidName`） | 单测 `bus_name_rule_matches_vdn_shm`（12 个样例逐个比对） |
| 总线头里的 lead · 降噪块长 · L_total · 采样率 | `vdn_control::Config`（lead）+ `Worker`（管线延迟、块长）；只经 `vdn_denoised::denoise_bus_config` 写 | 单测 `io_rate_single_source`（写进去 = 读回来）；chain-test 每次运行比对送音端读回的 lead / 降噪块长，不一致报错 |
| chain-test 的采样率常量 | `vdn_control::IO_RATE` | 直接引用（`const RATE = vdn_control::IO_RATE`），不再手写 48000 |
| 送音端「没有降噪总线时」的延迟线长度 | 不设第二处：取配置里的 lead（不手抄管线延迟）；接上后改用总线头的 L_total | —（代价：第一次接上降噪进程时延迟跳一次） |

## V5 人工步骤（给大总管转人）

⚠️ 这一步会开麦克风、从扬声器出声。**请戴耳机**：用外放时麦克风会收到扬声器的声音，可能啸叫。

1. 在仓库根目录跑 `./target/debug/vdn devices`（只列设备，不开麦、不弹框），确认默认输入、输出设备是你想用的
2. 戴上耳机，跑 `./target/debug/vdn run --passthrough --seconds 5`
3. 第一次跑时 macOS 会弹「终端想使用麦克风」，请点「允许」（点了之后要重跑第 2 步）
4. 5 秒内对着麦克风说话

**算过**：耳机里听到自己的声音，比嘴上晚一点点（链路内约 48 ms，另加设备缓冲），没有断续、没有爆音。
**算不过**：没声音、断断续续、有咔哒声或啸叫。结束时终端会打印「输出欠载 N 样本」，请把那一行发回来。

## 自审（2026-11-02，三个对抗视角的子代理；报告副本在 `_tmp/20261102-rs-entry-review/`，不入库）

清单用途：每条写「问题 → 怎么处理 → 怎么证明」。

| 视角 | 级别 | 问题 | 处理 | 证明【测】 |
|---|---|---|---|---|
| 故障 | P0 | 降噪进程重启后送音端不重连（插件若按名字开总线 → 静音） | 送音端监视循环：心跳停 ≥ 1 s 且同名能开到心跳不同的总线 → 换过去；正弦路径 `Forwarder::rebind` 保留延迟线，原声不断；麦克风 / 扬声器流重开（有一小段断，未量化） | 两进程：第 3 s SIGKILL 降噪 A，第 4.5 s 起 B；10 s 文件全段相关 1.000000、静音 0 ms、欠载 0 |
| 故障 | P1 | 处理后 ↔ 原声 全是 1 样本内硬切 | 缺样本时硬切 + 接缝偏移（规约 R11）；回滞：连续 `fade` 个准时样本才淡回 | 新单测（假降噪 = 0.1 × 原声）：死掉 / 卡 50 ms / 断续停 三种场景相邻样本跳变 < 0.13 |
| 故障 | P1 | 长期偏慢时一直送原声、计数看不出 | 新计数 `slow_episodes`（心跳在动但连续缺 > 超时） | 单测：吞吐 2/3 时 `slow_episodes ≥ 1`、`stale_switches = 0` |
| 故障 | P2 | 重新对账后留下的两块直接满权重播 | 全丢 + 强制硬切 + 回滞淡回 | 单测 `resync_does_not_jump` |
| 故障 | P2 | Out 环没有积压上限 | `OutSource` 读前丢掉超过 200 ms 的旧积压，计数 | 单测 `out_backlog_is_capped` |
| 故障 | — | （复测发现）新总线开头的占位零被当成声音淡入 → 重接时 25 ms 静音 | 推进序号 < 0 的处理后样本标成占位，照用原声 | 同上两进程测试静音 0 ms |
| 残留 | P1 ×2 | 同名第二个建方会删掉别人在用的总线（启动时 / 退出时） | `OwnedBus`：每个名字一把文件锁（`File::try_lock`，SIGKILL 也放锁），拿到锁才清残留、才建；Drop 删名字一次 | 同名起两个：第二个报「正被占用」退出码 1，第一个仍可接；单测 2 条 |
| 残留 | P2 | SIGTERM / SIGHUP 不删名字 | ctrlc 开 `termination`；先装处理再建总线 | `kill -TERM` 后总线已删 |
| 残留 | P2 | `--seconds -1` 等在建总线后 panic | `vdn_control::parse_seconds`（有限、0–1e6），所有秒数参数都走它 | 单测 |
| 残留 | P2 | chain-test 被 Ctrl-C 打断后名字永远残留（带 pid） | 默认名固定 `/vdn-chaintest`，下次拿到锁时清掉 | — |
| 规则 | P1 | tidy 红：两个 bin 里有 sleep | bin 整文件标 NRT（理由写在 allow 里）；`OwnedBus` 挪进 NRT 模块 | `xtask tidy` 通过 |
| 规则 | P2 | bin 根没有 panic 类 deny | 两个 `main.rs` 加同样的 `cfg_attr(not(test), deny(...))` | clippy 0 警告 |
| 规则 | P2 | 丢弃迟到样本逐个读 | `Pop::discard`，真环转调 O(1) 的 `Ring::discard` | — |
| 规则 | P2 | 不分配测试只覆盖空环 | 新单测：真进程内总线 + 定块长管线，走 正常 → 降噪停（Raw 溢出、过期）→ 恢复 → 重新对账，Forwarder + Worker 都在 `assert_no_alloc` 里；先确认守卫会因 10 字节分配 abort | 通过 |
| 规则 | P2 | `vdn run` 不查容量、不看 prime 返回值 | 加检查，块长 / 余量走 `Config::validate` | — |
| 规则 | P3 | `copy_within` 靠不变式 · shm 适配没挂 deny · 正弦每块分配 · `IO_RATE` 三份 | 切片写法 · 挂 deny · `fill_tone` 就地写 · denoised 改 `pub use vdn_control::IO_RATE` | — |

**没修、要大总管定的（涉及接口约定或别人的 crate）**：
1. **Out 环放在降噪进程建的总线上**：降噪进程一重启，插件那头就换了总线。建议 Out 环改由送音端自己建一块共享内存，插件只连它（P0 的根因；本批靠送音端重接兜住）
2. **降噪落后时不跳过 Raw 积压**：建议积压超 lead 就丢旧的、并在总线头记缺口（要改 vdn-shm 头布局）
3. **总线头里没有 lead / 降噪块长 / 代号**：送音端没法校验「lead ≥ 送音块 + 降噪块」，换总线时只能靠「心跳不同」判断是另一条（两条心跳都是 0 的边角情况判不出）
4. **麦克风流重接要重开设备**：会断一小段；要无缝得在回调里无锁换总线（ArcSwap 类，新依赖）
5. tidy 只查 `lib.rs` 不查 `main.rs` 的 crate 根；cargo-deny 本机没跑过（xtask / CI，大总管的）
6. 规约 §2.2 图里 `vdn-capture` 的依赖要补 `vdn-dsp` · `vdn-hal` · `clap` · `hound`（规约不归我改）

**留着的小问题**：送音端等总线最多 10 s 就退出（没有总线就没有 Out 环可写，和第 1 条一起改）；心跳超时实际比配置多最多一块（V2 实测 110 ms，在 150 ms 线内）

**负载影响（发现）**：机器 1 分钟负载 23.7 时跑的一次 V3（GTCRN）出现一次降噪停顿 > 100 ms（切原声 1 次、虚拟麦欠载 960 样本），负载降到 16 后重跑全 0。chain-test 的三个线程没提实时优先级，这一项在延迟修复里一起改

## 改了哪些文件
- 新：`src/control/**` · `src/capture/**` · `src/denoised/**` · `src/cli/src/{chain,live,verify}.rs`
- 改：`src/cli/src/main.rs`（新子命令 devices · run · chain-test · chain-check）· `src/cli/Cargo.toml`（依赖；forbid 去掉 cpal，留注释）
- 根 `Cargo.toml`：members 加三项；workspace 依赖加 `vdn-control` · `vdn-capture` · `vdn-denoised` · `serde` · `ctrlc`

## 新 crate 实体行（收尾时由大总管写进 `.42cog/cog.md`）
- `vdn-control`：配置文件与控制消息 · 人写配置文件、bin 读 · 是 `vdn-capture` / `vdn-denoised` / `vdn` 的输入
- `vdn-capture`：送音进程（lib `Forwarder` + bin）· 麦克风回调驱动 · 输入原声，输出 Out 环给虚拟麦克风；读 Processed 环与心跳
- `vdn-denoised`：降噪进程（lib `Worker` + bin）· 建总线 · 输入 Raw 环，输出 Processed 环与心跳；依赖 `Pipeline`

## 设计要点
- 对齐账只用 `L_total`（总线头里有）：Processed 第 j 个样本在输出时刻 `A(j − L_total) + L_total` 播出。迟到的丢、缺的用延迟原声补、Raw 环满的缺口单独记——降噪慢、卡、死、同进程恢复都逐样本对齐（单测覆盖）
- 心跳按音频样本计时（4800 样本 = 100 ms），回调里不读系统时钟
- `vdn-denoised` 不依赖 `vdn-capture`（后者带 cpal）；`Worker` 直接用 `vdn_shm::Bus`，单测用 `Bus::heap`

## 发现（本批不查，大总管记进 backlog，压延迟那批用 release 构建 + 预热核实）
- **现象**：环余量 = 1 块（480 样本，10 ms）且降噪块长 96 时，送音端「处理后欠载」（处理后的声音没按时到，用延迟原声补上；不是静音）
- **次数**：6 次里 4 次——直通 3 次：1728 / 0 / 0 样本；GTCRN 3 次：480 / 480 / 1920 样本。虚拟麦欠载 6 次都是 0
- **对照**：降噪块长 480 时同样 6 次全 0；默认环余量 1440 时全 0【测】
- **推断，未核实**：GTCRN 第一次推理慢（预热）吃掉 10 ms 余量；debug 构建、降噪线程没活时睡 0.5 ms 轮询也有影响
- 默认环余量保持 1440（大总管定）

## 已知缺口
- 环余量 30 ms，预算 3 ms：GTCRN 时 L_total 64 ms > 45 ms 预算（D32）。待实时线程 + 小块后压（大总管定）
- 降噪进程**换了个进程**重启：新流和旧对齐账对不上，靠「积压超限 → 重新对账」兜底，不保证逐样本对齐；新进程重建同名总线时送音进程要重开总线（本批不做自动重开）
- 降噪线程没活时睡 0.5 ms 轮询（B 档；后续换跨进程唤醒）
- `--sink hal`：本批没有插件读 Out，环满后计溢出

## 依赖许可
- `serde` 1（MIT OR Apache-2.0）· `ctrlc` 3（MIT OR Apache-2.0）——与 GPL-3.0 兼容（子代理复核中）

## 待人拍板
- 无

## 流水
- 2026-11-02 报到；三个零件手下的 worktree 还没有新代码
- 2026-11-02 收到三家签名；lead 定为参数、默认 1440（大总管）
- 2026-11-02 `Forwarder` 第一版「单个净账」在 Raw 溢出 + 处理后缺失同时发生时错位（单测红）→ 改成按输出时刻逐样本记账 + 缺口表，转绿
- 2026-11-02 大总管合入三家零件（65e62a9）；接线：capture 的 shm/hal 适配、两个 bin、vdn devices / run / chain-test / chain-check
- 2026-11-02 check-deps 报 vdn-denoised 闭包有 cpal（经 vdn-capture）→ Worker 改直接用 Bus，capture 改 dev 依赖，复跑通过
- 2026-11-02 V1–V4 过；lead = 1 块加测 6 次欠载 0；Ctrl-C 退出后总线已删【测】
- 2026-11-02 大总管合入 `Pipeline::with_block`（fd26a1d）；`Config.denoise_block` 默认 96，vdn-denoised 与 chain-test 用定块长，`vdn run` 仍用 `Pipeline::new`；V1–V4 重跑全过
- 2026-11-02 4 个 crate 27 个单测过（含 1.2 s 版 V1 / V2 真共享内存测试），clippy 0 警告，fmt 过
