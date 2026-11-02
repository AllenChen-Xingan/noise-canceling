# 大总管看板

> 只有大总管跟人说话。手下做完、卡住、要名额，只报给大总管。
> 大总管会话名：`noise-canceling-47`（换班后在这里改）。tmux 工作区：`vdn-rust`（另一位大总管的手下在 `vdn`，账本 `state/coord.md`）。

## 当前大事

**Rust 工程骨架整条链路**：麦克风 → 收音 → 共享内存 → 降噪 → 虚拟麦克风，声音送得进、出得来（方案与接口约定 `docs/plan/010_rust-skeleton.md` 第九–十一节）。分支 `feat/rust-skeleton`（worktree `../noise_canceling-skeleton`）；四个手下各在子分支，大总管合回。

## 手下

| 窗口 | 任务 | 交代文件 | 现状页 | 状态 |
|---|---|---|---|---|
| rs-core | 48 kHz 处理层 · 实体：vdn-dsp · vdn-engine · vdn-backend-ort · vdn-models · `feat/rust-skeleton-core`（`../noise_canceling-rs-core`） | `_tmp/20261102-b8-rust/rs-core.md` | 其 worktree `state/handoff/rs-core.md` | 已报到（noise-canceling-rs-core-2d） |
| rs-shm | 共享内存总线 · 实时线程 · 闸门 · 实体：vdn-shm · vdn-rt-thread · xtask · `feat/rust-skeleton-shm`（`../noise_canceling-rs-shm`） | `_tmp/20261102-b8-rust/rs-shm.md` | 其 worktree `state/handoff/rs-shm.md` | 已报到（noise-canceling-rs-shm-dc） |
| rs-io | 音频进出 · 虚拟麦克风读出端 · 实体：vdn-io-macos · vdn-hal · `feat/rust-skeleton-io`（`../noise_canceling-rs-io`） | `_tmp/20261102-b8-rust/rs-io.md` | 其 worktree `state/handoff/rs-io.md` | 已报到（noise-canceling-rs-io-7b） |
| rs-entry | 接线 · 实体：vdn-control · vdn-capture · vdn-denoised · vdn-cli · `feat/rust-skeleton-entry`（`../noise_canceling-rs-entry`） | `_tmp/20261102-b8-rust/rs-entry.md` | 其 worktree `state/handoff/rs-entry.md` | 已报到（noise-canceling-rs-entry-c3） |

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
- rs-shm：Bus / rt-thread 签名定（check_current 改返回 Result，认可），已转 rs-entry；在写测试；Miri / cargo-deny 未装、不装
- rs-io：io-macos / hal 签名定（回调 ≤ block 帧；HalSource 对 SampleSource 泛型），已转 rs-entry；Out 环 → SampleSource 的包装归 rs-entry
- rs-entry：链路缓冲余量 lead 定为参数，默认 1440 样本（30 ms）；GTCRN 时 L_total 64 ms，超 45 ms 预算 → 记已知缺口，后续批次压
- rs-core：Pipeline 签名定（不设 max_block；Identity 延迟 863、GTCRN 1631 样本 @48k），已转 rs-entry；在写测试与拆 backend-ort
- 16:xx 四个手下已起（tmux `vdn-rust`）；四个都已报到

## 发现

- 2026-11-02 · rs-entry：跨进程缓冲余量 30 ms（预算 3 ms），骨架期接受、记已知缺口

## 待定清单（攒着等人一次看完）

- [ ] 两个进程（008 #4）与工程细节（第五组）按建议推进，待人确认 —— 来源 010 第九节
