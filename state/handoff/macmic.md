# 现状页 · 手下 macmic（022 Mac 内置麦克风）

状态：进行中（交付已写完，在做评审）
分支 `research/022-mac-mic` · issue #81 · 管的实体：麦克风（设备链）· 设备链模拟

## 做到哪
5 份交付写完（`docs/research/022_mac-mic/`：devices · simulation · selftest · decision · sources）；工具 2 个带 10 条测试已过。正在做自审与 codex 评审。

## 流水
- 2026-11-03 · 读交代、共同规矩、手下协议 · 已报到
- 2026-11-03 · 读 011 macos-mic / 003 / 016 F8 / exp006 device / mtgdata 转交 · 已知结论并入草稿
- 2026-11-03 · 写设备键表草稿 v0 · 发 scenes、mtgdata、大总管
- 2026-11-03 · 写调研计划 `plan.md`（4 路 + 4 怀疑派）· 大总管批，补三点（Air 单列、嘈杂环境录音抱怨、可执行模拟步骤）
- 2026-11-03 · 设备键 v1：应 mtgdata 加 mac-unknown 与 non-laptop 8 个子键 · 已通知；scenes 场景键升 v1
- 2026-11-03 · mtgdata 的 check_cards.py 改为直接读 devices.md 第一列 · 已在 devices.md 抬头注明表格式约束
- 2026-11-03 · workflow `wf_16dd3e65-ebc` 完成（8 代理，0 失败）· 原始报告存 `findings/`
- 2026-11-03 · 亲手只读核本机系统调音文件 · 写 `findings/tunings.md`：标准模式已做每麦降噪 + 固定波束 + 均衡
- 2026-11-03 · 写 `tools/read_tunings.py`、`tools/check_devices.py` + `test_tools.py` · 10 条测试过，设备键 21 个 0 处问题
- 2026-11-03 · 写 devices v2 · simulation · selftest · decision · sources
- 2026-11-03 · 给 mtgdata 发 DR-VCTK 改标签 · 失败：mtgdata 已下线，转大总管

## 待人拍板
1. 要不要在你的 Mac 上实测一次内置麦（只读档 5 分钟 / 全套 60–90 分钟 / 等发版前）。题面与默认见 `docs/research/022_mac-mic/decision.md` 第三节。如果你不回，默认等发版前。

## 转交
- mtgdata：DR-VCTK 设备键 `mba-builtin` → `mac-intel-builtin`；MODALITY、DEBATE 两条数据线索（`findings/data.md`）
- D65 回写：DR-VCTK 许可已核清为 CC BY 4.0
- 011 卡：cpal 行号按版本写（0.18.2 是 `device.rs:237`）
- scenes：ETSI 桌面免提布置已用进 simulation 第 1 步
