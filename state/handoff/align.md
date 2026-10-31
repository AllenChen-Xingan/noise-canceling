# 现状页 · align（批次 20261031-b4）

- **状态**：进行中
- **手上**：架构对齐（008 规约收尾 P1）+ 拆 `vdn-types` + 智能体部件接口骨架（感知 · 策略 · 场景参数表 · 记忆 · 决策日志 · 延迟账）
- **交代**：主目录 `_tmp/20261031-b4/align.md`；分支 `feat/arch-align`

## 流水
| 时间 | 事 |
|---|---|
| 10-31 | 报到；读 common、008 decision / 规约 / ADR 005、005 方案、D32–D41、p1 代码 |
| 10-31 | 建 `vdn-types`（零依赖）：model · frame · stage · scene · params · table · policy · memory · record（决策日志）· latency · version · text；接口草案发 p2、scene |
| 10-31 | 按 p2 反馈改 `SpeakerVerdict`（类别 + 置信度 + 声纹窗起止 + 主人电平）、加 `NoSpeech`、`FrameCtx.now / voice_prob`、`Enroll` 事件；按 scene 反馈加 `UserScene::Meeting`、`ToneId` = 修音链预设、`strength` 只管湿干比 |
| 10-31 | engine / models 改用 `vdn-types`；models 不再依赖 engine（只作 dev-dep）；`Engine::latency_ledger`、`SafetyConfig::apply_params`、`ModelKind::declared()` |
| 10-31 | 008 P1 五项：① dsp 实时路径 assert/expect → debug_assert + 不 panic 的兜底 ② 热循环下标 → zip / last / as_chunks / slice pattern ③ 根 clippy.toml + 实时 impl 块 deny + crate 根 panic lint + offline/fault 整模块 NRT ④ `src/xtask`（tidy + check-deps）+ 各 crate `[package.metadata.vdn]` ⑤ exp004 README 补口径 |
| 10-31 | **clippy 清单探针**（哨兵 crate，scratchpad/clipsentinel，clippy 1.99）：清单 26 条中 24 条报红；`assert*!` 禁了会连 `debug_assert*` 一起红 → 移出 clippy、改由 tidy 查实时块；`<[T]>::to_vec` 任何路径写法都不命中 → 移出，交计数分配器 / RTSan |
| 10-31 | check-deps 负例自测：给 engine 的 forbid 临时加 realfft（经 dsp 传递依赖）→ 报红，恢复后通过 |
| 10-31 | ⚠️ 自查：重命名 `log.rs → record.rs` 时用了 `git mv`（git 写操作，违反 common 规矩 2）；改动随后被 auto-commit 提交，无其他影响。以后改名只用 `mv` |
| 10-31 | 大总管定：删 `.cargo/config.toml`（新顶层目录，铁律 10），一律写全 `cargo run -p xtask -- tidy / check-deps`；xtask 放 `src/xtask/` 获同意 |
| 10-31 | 规约 §1 · §2 · §3.1 · §3.3 · 新增 §3.7（感知 · 策略 · 场景参数表 · 记忆与学习 · 决策日志 · 延迟账）· R4 · R13 · §5.3；`.42cog/cog.md` 实体表 |
| 10-31 | 单 crate 测试：types 40 · dsp 6 · engine 8 · xtask 5 全过；等名额跑全量 debug + release |
