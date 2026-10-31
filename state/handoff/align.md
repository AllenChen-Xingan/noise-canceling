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
| 10-31 | 三视角自审（实时安全 · 接口设计 · 机器检查与延迟账）。修：GTCRN 长度不符改返回 `Err(Shape)`；两模型出错即清缓存（防半更新）；irfft `InputValues` 不再整帧清零；**滞回按 (用户层, 细场景) 比较（P0：只出用户层的分类器原来永远切不过去）**；低置信打断确认；锁定冻结「表+配置」、旋钮仍生效；用户层以表为准；`Policy::observe`、`Stage::set_params`；日志加 `SceneObserved`、`PreferA/B`、`DeviceChanged{rate}`；参数表 `[user]` 必须在 `[fine]` 前；我的配置拒重复小节与 NaN；删 `params_for` |
| 10-31 | **延迟账改为逐项硬加**：原「IO 缓冲并入攒 hop」推断对 FE-M 不成立（480 帧 hop 不被 256 整除），已删。009 目标（IO 256 帧）47.00 ms 超 2 ms，钉成已知缺口；提议 IO 块 96 帧（两模型 hop 公约数）→ 43.67 ms，测试按它断言 ≤45 |
| 10-31 | tidy 加固：`expect(`/`cfg_attr(allow` 也要理由、`allow(dead_code)` 不再豁免整文件、`assert !(`/同行 `{`/模块头 deny 都查、`#[cfg(test)]` 只在紧跟 `mod` 时截断；check-deps 用 `--all-features`；核心 crate forbid 加 coreaudio/objc2（types 另加 libc）；crate 根加 `unreachable`/`todo` |
| 10-31 | ondevice 需求 ①②③ 落类型（`ParamBlock`、`Transition` 限速与手动斜坡、`FrameFeatures`），④⑤⑥ 写进规约 §3.7.4b；R11 补 NaN 硬切。ondevice 会话已结束、回信未送达，经大总管转 |

## 自审未修（P2，留给下一批）
- `Engine::process` 长度不符分支送的是未对齐原声（比正常早 latency 个样本、无接缝），目前只计数；接 Pipeline 时一并改
- `ToneId` 无范围校验（预设个数归 polish 定）；手动模式下 `SceneCommitted` 记门判结果、`ParamsApplied` 记 0，日志两条细场景不一致
- `copy_from_slice` 长度不等会 panic，现各调用点都先查长度，但没有机器守；tidy 的 crate 根检查是文本包含
- deny.toml / cargo-deny 未建（E-deny）

## 待人拍板（经大总管）
| # | 题面 | 选项 | 代价 | 建议 | 默认 |
|---|---|---|---|---|---|
| 1 | 延迟账按 009 目标（IO 256 帧）逐项硬加 47 ms，超 D32 2 ms。把「IO 块 96 帧（2 ms）」定为 P3 调参目标吗？ | A 定 96 帧为目标 · B 保留 256、超出部分等 P3 实测再走 008 ⟨拍板 6⟩ | A：每秒约 500 次回调，多唤醒、略增 CPU / xrun 风险，设备不一定支持；B：账面先超 | A（都是推断，P3 脉冲法裁决） | A 写进账与测试，P3 前不改 |
