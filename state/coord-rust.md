# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-47`（换班后在这里改）。tmux 工作区：`vdn-rust`（另一位大总管的手下在 `vdn`，账本 `state/coord.md`）。

## 当前大事

**Rust 工程骨架整条链路**：麦克风 → 收音 → 共享内存 → 降噪 → 虚拟麦克风，声音送得进、出得来（方案与接口约定 `docs/plan/010_rust-skeleton.md` 第九–十一节）。分支 `feat/rust-skeleton`（worktree `../noise_canceling-skeleton`）；四个手下各在子分支，大总管合回。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| rs-core | 48 kHz 处理层 · 实体：vdn-dsp · vdn-engine · vdn-backend-ort · vdn-models · `feat/rust-skeleton-core`（`../noise_canceling-rs-core`） | `_tmp/20261102-b8-rust/rs-core.md` | 其 worktree `state/handoff/rs-core.md` | 完成，已合入 feat/rust-skeleton（3e2992e），窗口已关 |
| rs-shm | 共享内存总线 · 实时线程 · 闸门 · 实体：vdn-shm · vdn-rt-thread · xtask · `feat/rust-skeleton-shm`（`../noise_canceling-rs-shm`） | `_tmp/20261102-b8-rust/rs-shm.md` | 其 worktree `state/handoff/rs-shm.md` | 完成，已合入 feat/rust-skeleton（449f40d），窗口已关 |
| rs-io | 音频进出 · 虚拟麦克风读出端 · 实体：vdn-io-macos · vdn-hal · `feat/rust-skeleton-io`（`../noise_canceling-rs-io`） | `_tmp/20261102-b8-rust/rs-io.md` | 其 worktree `state/handoff/rs-io.md` | 完成，已合入 feat/rust-skeleton（a9d4c83），窗口已关 |
| rs-entry | 接线 · 实体：vdn-control · vdn-capture · vdn-denoised · vdn-cli · `feat/rust-skeleton-entry`（`../noise_canceling-rs-entry`） | `_tmp/20261102-b8-rust/rs-entry.md` | 其 worktree `state/handoff/rs-entry.md` | 已报到（noise-canceling-rs-entry-c3）· 三家已合进其分支（65e62a9），接线中 |
| rs-wake | 跨进程唤醒替掉 sleep 轮询 + 唤醒 / 轮询对照实验 + 总线头加 lead / 块长 / 会话 ID / 缺口 + tidy 查 main.rs · 实体：vdn-shm · vdn-rt-thread · xtask · `feat/rust-skeleton-shm`（`../noise_canceling-rs-shm`） | `_tmp/20261102-b8-rust/rs-wake.md` | 其 worktree `state/handoff/rs-wake.md` | 已报到（noise-canceling-rs-shm-86） |
| rs-core2 | 处理链补齐：Stage 串进 Pipeline（处理图第一步）· 送原声改用 48 kHz · 延迟断言改 Pipeline · FTZ 守卫 · 实体：vdn-types · vdn-dsp · vdn-engine · vdn-models · `feat/rust-skeleton-core`（`../noise_canceling-rs-core`） | `_tmp/20261102-b8-rust/rs-core2.md` | 其 worktree `state/handoff/rs-core2.md` | 完成，已合入 feat/rust-skeleton（77f46e6），窗口已关 |

## 规矩

- **与另一位大总管 noise-canceling-e3 的约定（2026-11-02）**：任一方跑「计时基准」前给对方发「基准开始，约 N 分钟」，期间对方不起编译、测试、推理；跑完发「基准结束」。普通编译测试不打招呼。另用锁目录 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/_tmp/heavy.lock`（mkdir 原子，内写 owner · 开始 · 预计结束，跑完 rmdir）；我方由大总管放名额时建、收回时删，手下不碰

- 共同规矩 `_tmp/20261102-b8-rust/common.md`（在本 worktree 下）
- 大总管只派活、验收、合并，不写实现（人定 2026-11-02：「记住，你的工作是这一批的大总管，而不是干活的人 / 你要给各种手下派活」）
- 每个手下只管 2–4 个实体（CLAUDE.md 第五节第 4 条）

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | 空——机器交 e3 跑 applebe（17:10 起约 30–40 分钟），我方期间不编译不跑 |

排队：e3 applebe 结束后 → rs-entry 锁步测试改完连跑 5 次（带 4 个忙进程）→ rs-wake QoS 小实验 → 大总管收尾 V6

等人许可的弹窗 / 长时间前台运行：（空）

## 进度板

进度（HH:MM）
- rs-entry 自审：1 P0 + 5 P1 已修（两进程 SIGKILL 重启：相关 1.000000、静音 0）。6 条交大总管 → 定：①Out 环改由送音端自建（rs-entry）②③总线头加 lead · 降噪块长 · 会话 ID · 缺口计数（rs-wake），积压超 lead 丢旧并记缺口（rs-entry）④无缝换设备 → backlog ⑤tidy 查 main.rs → rs-wake；cargo-deny 本批不装 ⑥规约文字收尾时大总管改
- rs-core2：Pipeline 挂后处理级签名定（`Pipeline<M, S = ()>` · `with_stages` · `stages!` · `process_ctx` · `last_status`；new / with_block 不变）；送原声已改 48 kHz；在做故障注入 / 延迟断言 / 对拍
- rs-core2：FTZ 守卫自写（arm64 FPCR 用 asm），vdn-dsp 进 unsafe 白名单，unsafe 只在 ftz 模块；Miri 与其他平台下空操作（大总管定，不引 no_denormals）
- 对照审查（子代理，只读）：`docs/plan/010_rust-skeleton-audit.md`。守规矩那半沿用较全；处理图没有；009 大方向沿用。8 条偏离分派：①最小一步（Stage 串进 Pipeline）+⑤FTZ +⑥送原声 48k / 延迟断言 → 新起 rs-core2；②纠漂移 +⑦rtrb 实时切模式 +⑧capture 依赖 → rs-entry；③loom / deny 直接边 +⑤Workgroup +⑧CI 拆 Linux → rs-wake；②后半 0600 权限 → 等装驱动实测；ContentId 用 FNV → backlog
- **耗电（人亲手 sudo 跑，第一次因脚本 locale bug 退出，修后重跑）**：功耗中位 基线 74 · 不提 72 · tc 77 · tc + 工作区间 101 mW（各 60 s 1 次，平均 117–125）；醒来迟到 不提 2.6 / 7.0 ms → tc 0.022 / 0.068 ms；每块计算三档都约 3.02 ms（标定约 0.7 ms），E 簇约 1.1 GHz、P 簇未起 → 优先级治醒得晚、不治算得慢；下一步 QoS user-interactive 小实验（rs-wake）
- rs-entry 验收**打回**：并行跑时 v1_short、mode_switch_mid_run 失败（单独跑 3/3 过）＝不稳定测试；改为锁步模拟，验收＝连跑 5 次全过且带 4 个忙进程
- **V7 过（有唤醒，release）**：lead 192 空闲 0/10、压力 0/10 欠载，GTCRN L_total 1728 = 36.00 ms（判据上限）；lead 144 空闲 10/10 欠载。推理块空闲中位 2.9 ms vs 压力 0.6 ms，唤醒没改变卡点（空闲降频，推断）。lead 默认 288 → 192。负载 2.00–7.38（核数 10）
- rs-wake ③：loom（release）2 项过 2.5 s，反向验证（对调读序）会报唤醒丢失；Miri vdn-shm 进程内测试全过、无未定义行为；修 2 处 Miri 暴露的问题
- rs-entry：V1 判据统一为 judge_v1（相关 ≥ 0.999 且两种欠载都 = 0），加反例测试；启动硬下限定为 lead ≥ 一块 IO（正本 `vdn_control::check_lead`，三处调用；大总管认可，不用 IO + 降噪块，否则挡掉 V7 目标 144）；54 过 1 忽略
- rs-entry 测试自查：51 过、1 忽略（V5 写成 #[ignore] 测试）；Out 总线重连从 bin 挪进 lib（Supervisor）并补 4 条真共享内存测试；补测揪出测试工具配错（IO 块 480 配 lead 288 必欠载，旧 V1 短测只判相关没发现）→ 已改；大总管加要求：V1 全部断言欠载 0 · Config 启动校验 lead ≥ IO 块 + 降噪块
- 人定「加测试是默认行为」→ CLAUDE.md 第五节第 6 条 · PR 模板 · common.md 第 16 条 · 记忆；已通知 rs-entry / rs-wake；rs-wake 加 tidy「每个 crate 至少一条测试」
- rs-entry：纠漂移（漂移在 Out 两头；±0.1% 60 s 最大积压 ≤ 480、欠载 0；不纠漂 ±60 ms）· rtrb 运行中切直通（欠载 0、静音 0、相关 1.000000）· capture 库闭包无 hound / clap / cpal；43 测试过
- 大总管抽查（人问「这些所有东西都做测试了吗」）：骨架 `xtask ci` 6 项全绿（含 cargo-deny）、228 测试 0 失败 3 忽略（2 真设备 + loom）；rs-entry 4 crate 43 测试 0 失败、clippy 无警告；V1 重跑过（相关 1.000000、延迟 1056）。未测：真设备 / V5、HAL 真驱动、V7（未过）、loom、Miri（在跑）、release 测试、RTSan、纠漂移与切模式（未写完）
- rs-entry：两处红线已修（换总线改 rtrb 交接 + 回收；NaN 记号删，改零占位 + note_gap + 送音端按心跳过期补原声）；新增文本检查：capture 源文件无锁；新依赖 rtrb 0.4（MIT OR Apache-2.0）；41 测试过、两进程三种起停全过
- 人同意跑 powermetrics（原话「可以直接跑 power matrics」）→ rs-wake 写 `_tmp/20261102-b8-rust/power/run.sh`（三种优先级设置 × 60 s + 基线），大总管审后由人 `sudo bash` 跑
- rs-core2：验收通过（4 crate 115 条测试、clippy、tidy 大总管重跑；vdn denoise 4 场景含故障注入字节一致；3 次变异检查测试会红；自审无 P0 / P1）→ 合入、关窗。延迟账按代码：009 目标 44.98 ms，96 帧块 39.67 ms
- rs-wake：cargo-deny 0.20.2 已装，`cargo deny check` 四项全 ok（警告：cpal 带来的重复版本；deny.toml 未用许可 4 个、例外 1 个 → 删）；xtask ci 改为没装就失败，CI 要先装
- rs-wake 补数（提优先级后，空闲 / 压力，各 1 次 × 4900 块）：最长醒来 唤醒 40.6 / 7.4 µs · 200 µs 轮询 210.9 / 203.8 µs · 500 µs 轮询 527.0 / 502.8 µs；CPU 唤醒 0.41 / 0.21 % · 200 µs 轮询 1.62 / 1.03 % · 500 µs 轮询 1.30 / 0.48 %；每秒唤醒 约 500 · 3671–7354 · 1548–2902。wait 运行期探测，14.2–14.3 自动退回 200 µs 轮询
- rs-entry：唤醒接上（wait 替掉 sleep，退回轮询 0 次）· Out 总线 `/vdn-out` 由送音端自建 · OwnedBus 用 vdn-shm 的、旧份已删 · 两进程三种起停顺序（含 SIGKILL 重启）相关 1.000000、延迟 1056、静音 0（debug）· V7 进 `docs/experiments/exp-v7-latency/`。**打回两处**：音频线程 try_lock（违规约 §3.4）→ 改 rtrb 送入 + 回收；NaN 当带内记号 → 改 0 占位 + note_gap + 送音端按心跳过期路径补原声
- rs-wake 对照实验（release，两真进程，每组 4900 块）：推到醒中位 唤醒 4–17 µs · 200 µs 轮询 101–138 µs · 500 µs 轮询 250–331 µs；不提实时优先级三种都有 4.5–11 ms 尖峰，提了后唤醒最长 7–41 µs → 实时优先级是关键。待补：提优先级后轮询最长值、CPU、唤醒次数（为 009 待拍板 1「最低系统 14.4」）。loom debug 5 分钟未跑完，改 release
- rs-entry V7 基线（release、0.5 ms 轮询）**不过**：lead 96 / 144 空闲 10/10 次欠载；**空闲反而比压力差**（推理块中位约 3.0 ms vs 0.64–1.43 ms），推断降频 / 能效核 → Workgroup 提前做（rs-wake）；空闲加扫 lead 192 起无欠载（仅 3 次）。lead 默认暂定 288（6 ms，GTCRN L_total 38.00 ms）。名额已还；骨架（唤醒）已合进 rs-entry，待它补 BusConfig 新字段
- 011 人定 A：骨架之后，处理图 + schema 交叉校验合成下一件大事；人定新规矩「唯一真相源 + 交叉校验」→ CLAUDE.md 第五节第 5 条，已通知全部手下
- 人定 D66（原话「c」）：「超级语法树」两种都要；处理图统一表示排在骨架之后。对照审查子代理在跑（008 / 005 / 009 三样）
- rs-wake：新签名（头部版本 2 · notify / wait · lead / 降噪块长 / 会话 ID / 缺口 · OwnedBus），vdn-shm 测试全过；BusConfig 多两个字段，rs-entry 要补
- 人定 2026-11-02（原话「还有手下，装」，大总管理解为装检查工具）：装 cargo-deny 并接进 `xtask ci`；装 nightly + Miri 跑 vdn-shm（及 dsp）→ rs-wake
- 追加定案：OwnedBus（按总线名文件锁独占建立）收进 vdn-shm（rs-wake），denoised 里那份到时删；Out 总线固定名 `/vdn-out`（Config 可改），不带 pid
- rs-entry 延迟修复做法：IO 块默认 96 · 预热 + 每块耗时计数 · chain-test 线程提实时优先级 + `--stress N` · `vdn v7` 子命令（可扫 lead）
- **延迟修复（人定 2026-11-02：「帮我看下，为啥有这么多问题 / 直接修复」）**：诊断 `_tmp/20261102-b8-rust/latency-fix.md`（A IO 块 480 · B sleep 轮询 · C 推理突发与预热 · D debug 构建 · E lead 留 3 块）；判据 V7 = release、10 次 × 10 s、空闲 + 压力、欠载 0、lead ≤ 192（目标 144）。rs-entry 管 A C D E；新起 rs-wake 管 B
- rs-entry：with_block 接进降噪进程，V1–V4 重跑全过；L_total 直通 2208（46.00 ms）、GTCRN 2976（62.00 ms）；块长 100 启动即报错。新现象：lead = 1 块 + 块长 96 时 6 次里 4 次「处理后欠载」480–1920 样本（用延迟原声补，非静音），块长 480 时 0 次；推断是首次推理预热 + debug 构建，未核实 → backlog
- 子代理（省 2 ms）：`Pipeline::with_block` 完成，GTCRN 块长 96 时 1536 样本 = 32.00 ms（原 33.98），实测 = 声明；engine 测试 16 条过、clippy 无警告 → 合入骨架（113dd13）与 rs-entry（fd26a1d）
- rs-entry：**V1–V4 全过**（V1 相关 1.000000、延迟 2303 = L_total、欠载 0；V2 杀降噪 110 ms 切原声、最长静音 0；V3 GTCRN 有声无 NaN、L_total 64 ms；V4 两真进程同 V1，Ctrl-C 后 unlink）。lead = 1 块时欠载 0、GTCRN L_total 44 ms（空闲机器）。修了 denoised 经 capture 带进 cpal。自审中，等 with_block
- 三家合并后 `xtask ci` 全绿（test 26.5 s；cargo-deny 未装跳过）；骨架分支已合进 rs-entry（65e62a9）
- rs-core：验收通过（4 crate 测试全过，GTCRN / FE-M 真跑；vdn denoise 改造前后逐字节一致；自审 P0 1 · P1 2 已修）→ 合入、关窗。自报一次 git rm --cached（只涉被删的 session.rs）
- rs-shm：验收通过（测试全过；自审无 P0；新增 pop_or_zero）→ 合入、关窗
- rs-io：验收通过（测试 23 过 · 2 个真设备测试待人 · clippy / tidy / check-deps 绿；自审 P0 0 · P1 2 已修）→ 合入、关窗。已知违规 8 条见其现状页；待接：hal 接 shm 的 Out 环（归 rs-entry）
- rs-shm：名额已还；`xtask ci` 在 shm 分支全绿，23 s（cargo-deny 未装、跳过）；自审中
- rs-io：本机 4 个设备都支持 48 kHz；发现 cli forbid 含 cpal 与 `vdn run` 冲突 → 定：cli 去掉 cpal（rs-entry 改）；在自审
- rs-shm：Bus / rt-thread 签名定（check_current 改返回 Result，认可），已转 rs-entry；在写测试；Miri / cargo-deny 未装、不装
- rs-io：io-macos / hal 签名定（回调 ≤ block 帧；HalSource 对 SampleSource 泛型），已转 rs-entry；Out 环 → SampleSource 的包装归 rs-entry
- rs-entry：链路缓冲余量 lead 定为参数，默认 1440 样本（30 ms）；GTCRN 时 L_total 64 ms，超 45 ms 预算 → 记已知缺口，后续批次压
- rs-core：Pipeline 签名定（不设 max_block；Identity 延迟 863、GTCRN 1631 样本 @48k），已转 rs-entry；在写测试与拆 backend-ort
- 16:xx 四个手下已起（tmux `vdn-rust`）；四个都已报到

## 发现

- 2026-11-02 · rs-entry：跨进程缓冲余量 30 ms（预算 3 ms），骨架期接受、记已知缺口

## 待定清单（攒着等人一次看完）

- [x] Pipeline 省 2 ms 延迟 → **人定 2026-11-02：「可以省」**。做法：新增 `Pipeline::with_block`（块长整除 hop 时输出预填少 95 样本），`new` 不变；派会话内子代理在 `../noise_canceling-rs-core` 做
- [x] 最低系统提到 14.4 → **人定 D67（原话「可以，提高版本」）**；去掉按版本退回轮询的分支，只留调用失败兜底（rs-wake）
- [ ] 正常降噪时混回的原声要不要带 8 kHz 以上（rs-core2 题）：A 只到 8 kHz（现状，略闷）/ B 用 48 kHz（更亮，但高频嘶声按 −12 dB 放回）；建议 A，等衰减下限移成后处理级后 AB 听测；默认 A
- [ ] 两个进程（008 #4）与工程细节（第五组）按建议推进，待人确认 —— 来源 010 第九节

## 收尾时大总管要做

- [ ] V6：rs-entry 合入后，release 测试 + 全量 `xtask ci` + V1–V4 重跑 + V7 复测
- [ ] 规约 R10 unsafe 白名单加 vdn-dsp（FTZ）；§3.7.6 延迟表改为按代码（44.98 / 39.67 ms），并给这张表加与代码的比对（唯一真相源）
- [ ] 改过时说法：规约 §2.2 图里 vdn-capture 补 vdn-dsp · vdn-hal · clap · hound ·根 Cargo.toml 第 1 行注释 · speaker README / Cargo.toml · 规约 §2.2 · §3.7.6（细节见 rs-core 现状页）
- [ ] 新 crate 实体行写进 `.42cog/cog.md`（各现状页有行）
- [ ] **下一件大事**：处理图（D66）+ schema 交叉校验（011），清镜像副本与手写 JSON；后处理级非有限 / 幅度检查与 Engine classify 规则合成一处
- [ ] backlog：ContentId FNV → sha256；麦克风流无缝重接（回调里无锁换总线，要新依赖）；lead 压到 1 块时块长 96 的欠载（release + 预热核实）；Ring 拆 Producer / Consumer 句柄；共享内存权限 0600 与 coreaudiod（装驱动那批实测，补 009 待拍板 3）；models 不直接依赖 ort 尚无机器检查
