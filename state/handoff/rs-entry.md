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
