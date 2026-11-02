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

## 规矩

- 共同规矩 `_tmp/20261102-b8-rust/common.md`（在本 worktree 下）
- 大总管只派活、验收、合并，不写实现（人定 2026-11-02：「记住，你的工作是这一批的大总管，而不是干活的人 / 你要给各种手下派活」）
- 每个手下只管 2–4 个实体（CLAUDE.md 第五节第 4 条）

## 名额

| 名额 | 当前持有 |
|---|---|
| 重编译 / 全量测试（1 个） | 空 |

排队：（空）

等人许可的弹窗 / 长时间前台运行：（空）

## 进度板

进度（HH:MM）
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
- [ ] 两个进程（008 #4）与工程细节（第五组）按建议推进，待人确认 —— 来源 010 第九节

## 收尾时大总管要做

- [ ] V6：release 测试 + 全量 `xtask ci`
- [ ] 改过时说法：根 Cargo.toml 第 1 行注释 · speaker README / Cargo.toml · 规约 §2.2 · §3.7.6（细节见 rs-core 现状页）
- [ ] 新 crate 实体行写进 `.42cog/cog.md`（各现状页有行）
- [ ] backlog：Ring 拆 Producer / Consumer 句柄；共享内存权限 0600 与 coreaudiod（装驱动那批实测，补 009 待拍板 3）；cargo-deny / Miri 未装；models 不直接依赖 ort 尚无机器检查
