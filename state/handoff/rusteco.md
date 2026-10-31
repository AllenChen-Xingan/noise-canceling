# 现状页 · rusteco（批次 20261030-b3）

状态：**已收尾**

## 手上
009 Rust 生态调研：GitHub 上哪些 Rust 项目可直接用、可学。交代 `_tmp/20261030-b3/rusteco.md`（主目录）

## 做到哪
已收尾。交付：`docs/research/009_rust-ecosystem/`（decision · cards/ 8 份 · answers-p1 · sources · gap · review · must-know · assign · README）

## 结论一句
没有现成降噪框架，但零件齐：进程内直接依赖 ort · realfft · rubato · rtrb · triple_buffer · biquad · earshot · hound；跨进程部分（共享内存环、HAL 驱动、唤醒、workgroup）自写。最值得学 voice-strip（Rust 自写 macOS HAL 虚拟麦 + 共享内存 + 神经降噪），学它的时间轴分槽环——但它判缺填零，D03 要我们自己补原声延迟线。默认参数延迟账约 56 ms 超 D32，重采样与 IO 缓冲必须调参。

## 待人拍板（详见 decision.md §五）
1. 最低系统 14.2 → 14.4？（建议先实验，实验前按 14.2 写）
2. GPL-3.0-or-later 还是 -only（建议 or-later）
3. HAL 驱动拿共享内存：M0–M1 文件方式，发版前换 LaunchDaemon + XPC（建议如此）
4. RustFFT 发版已过 12 个月线，破例当依赖？（建议破例）

## 转交（范围外）
- rustarch 008：推理进程直接写 HAL 环少一跳；panic 策略；cargo-deny 白名单正本统一；nih-plug Params / firewheel 节点 trait 行号已在 cards/arch.md
- p1：已知 FE-M 每帧 7 次分配、奇偶帧轮换未做（它记进 exp004 局限 12）
- 规则建议：research.sh 在手下 worktree 里会按本地编号开号并改索引 README，并行开线时会撞号

## 流水
- 2026-10-30 读交代 + common；报到大总管；问到会话名 p1=noise-canceling-p1-a8 · rustarch=noise-canceling-rustarch-27；已分别发消息对齐
- 2026-10-30 跑 research.sh 起目录：脚本按本 worktree 编号成 008，且改了 docs/plan/README.md、docs/research/README.md（不在可写范围）→ 已手工改名 009、两个 README 还原为 165994a 版（auto-commit 曾提交过一次 008，后续提交会覆盖）。**规则建议**：别的线并行开号时，research.sh 要显式查 main 上已用编号，且不应在手下 worktree 改索引 README
- 2026-10-30 写 must-know（3 条命门：GPL 兼容 · 实时线程零分配零锁 · macOS 维护中）
- 2026-10-30 大总管批准 11 代理计划；Workflow wf_e9ab37af-647 已起（7 检索 → 3 怀疑派 → 1 完整性）
- 2026-10-30 答 p1 六个选型问题（ort rc.15 每帧 Rust 侧仍分配 · assert_no_alloc+rtsan 分工 · realfft · hound · cargo-deny 白名单 · no_denormals），文件 docs/research/009_rust-ecosystem/answers-p1.md
- 2026-10-30 rustarch 分工：项目卡归我；它要 nih-plug Params/Plugin 实时分离、firewheel 节点 trait 与跨线程参数的文件:行号（已写进 arch 组任务）
- 2026-10-30 Workflow 完成：11/11 代理，8 份卡（约 120 张项目卡），怀疑派复核约 450 条、改正约 40 处；新克隆约 45 个小仓进主目录 resources/（均 < 100 MB）
- 2026-10-30 复核发现我给 p1 的 ort 建议可改进：run_with_options + preallocate 全部输出 Rust 侧零分配（优于 run_binding）→ 更正已发 p1；p1 回报实测 GTCRN 每帧 0 次、FE-M 每帧 7 次、CoreML 默认会挂须 PreferCPU
- 2026-10-30 亲自抽查 voice-strip shm.rs 写/读协议、FastEnhancer 16 kHz spec2spec 发布物（官方 release 明载）——属实
- 2026-10-30 写 decision.md、sources.md、gap.md，补 assign.md / README
- 2026-10-30 三视角自审（A 事实 · B 实时架构 · C 许可可读性）：2 P0、10 P1 全改，记录 docs/research/009_rust-ecosystem/review.md；拍板 5→4 题（CoreML 改 AI 定；D06、D07、M0 验收已覆盖的不再问）
- 2026-10-30 已收尾
