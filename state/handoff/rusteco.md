# 现状页 · rusteco（批次 20261030-b3）

状态：**进行中**

## 手上
009 Rust 生态调研：GitHub 上哪些 Rust 项目可直接用、可学。交代 `_tmp/20261030-b3/rusteco.md`（主目录）

## 做到哪
decision.md 初稿已写（每模块首选+备选 · 5 个设计 · 待人拍板 5 题）；正在派 3 个对抗视角自审。

## 待人拍板
（暂无）

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
