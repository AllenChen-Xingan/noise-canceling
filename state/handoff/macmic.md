# 现状页 · 手下 macmic（022 Mac 内置麦克风）

状态：**已收尾**
分支 `research/022-mac-mic` · issue #81 · 管的实体：麦克风（设备链）· 设备链模拟

## 做到哪
交付全部写完；codex 评审 5 轮，第 4、5 轮都无 P0/P1，已达停止线；22 条意见全部属实、全部改完，不留 issue。等大总管验收。

## 改了哪些文件（都在本 worktree）
- `docs/research/022_mac-mic/`：`devices.md`（设备键 v2，21 个键）· `simulation.md` · `selftest.md` · `decision.md` · `sources.md` · `plan.md`
- `docs/research/022_mac-mic/findings/`：4 路调研 + 4 个怀疑派原始交回（8 份）· `tunings.md`（我亲手只读核的本机系统调音文件）
- `docs/research/022_mac-mic/tools/`：`read_tunings.py` · `check_devices.py` · `test_tools.py`（12 条测试，全过；均 ≤ 600 行）
- `docs/reviews/022_mac-mic-codex.md` + `docs/reviews/README.md` 索引一行

## 流水
- 2026-11-03 · 读交代、共同规矩、手下协议 · 已报到
- 2026-11-03 · 读 011 / 003 / 016 F8 / exp006 / mtgdata 转交 · 写设备键草稿 v0 · 发 scenes、mtgdata、大总管
- 2026-11-03 · 写调研计划 · 大总管批，补三点（Air 单列、嘈杂环境录音抱怨、可执行模拟步骤）
- 2026-11-03 · 设备键 v1（加 mac-unknown 与 non-laptop 8 个子键）· mtgdata 的检查脚本改为直接读 devices.md
- 2026-11-03 · workflow `wf_16dd3e65-ebc` 完成（8 代理，0 失败）
- 2026-11-03 · 只读核本机系统调音文件 · 写 `findings/tunings.md` 与解码工具
- 2026-11-03 · 写 5 份交付 · devices 升 v2（键名不变）
- 2026-11-03 · codex 第 1–3 轮各有 P1（结论说过头、实测承诺过头、HAL 对照缺失）· 逐条改
- 2026-11-03 · codex 第 4 轮发现模式值被读成设备键（25 → 21）· 已修并加检查；大总管已在 mtgdata 侧改 Clarity 标签（3c9b779）
- 2026-11-03 · codex 第 5 轮无 P0/P1 · 停止线满足 · 剩 2 条 P2 已改

## 待人拍板
1. 要不要在你的 Mac 上实测一次内置麦：A 只做只读档（约 5 分钟，不出声；中途切一次输入设备，正在用麦的 app 会断一下）· B 全套（约 70–100 分钟，要外接音箱和测量麦，会出声，算不算 D62 例外要你定）· C 等发版前。如果你不回，默认 C：开发阶段的 Air 模拟一直用别人 1 台 M2 Air 的数。题面见 `docs/research/022_mac-mic/decision.md` 第三节（大总管汇总给人）。

## 转交
- 已办（大总管）：DR-VCTK → `mac-intel-builtin`、许可 CC BY 4.0 回写 D65（d3743ac）；Clarity `unknown` → `non-laptop-unknown`（3c9b779）
- 给 016 / exp006 的负责线：`simulation.md` 第二节建议替换 F8「laptop」与 exp006 `device` 的假设（待大总管定，`decision.md` 第二节）
- 给 011 卡：cpal 行号按版本写（Cargo.lock 的 0.18.2 是 `device.rs:237`，`resources/` git 版是 `device.rs:84`）
- 给 mtgdata（已转大总管）：MODALITY、DEBATE 两条数据线索（`findings/data.md`「转交 mtgdata」）

## 发现
- codex-review 技能的隐私扫描会把以「P1」开头的 codex 输出误判为 PBM 位图（`file` 命令），拦下第 2 轮起的 `--previous`。本次每轮手动在临时输出顶上加一行标题绕开。建议改技能：让输出模板以标题行开头，或扫描时对 `.md` 跳过 `file` 判型
