# 现状页 · mtgdata（批次 20261103-b10）

- **状态**：进行中
- **手上**：023 开会场景的真实录音数据集（issue #82）
- **交代**：主目录 `_tmp/20261103-b10/mtgdata.md`
- **做到哪**（2026-11-03）：调研完成，6 份交付物写完（cards 44 张 · addendum 33 张 · matrix · pick · decision · sources）；check_cards.py 0 处问题、20 条测试过；正在做 codex 评审
- **下一步**：codex 评审，连续两轮无 P0/P1 即收尾

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `docs/research/023_meeting-datasets/plan.md` | 调研计划 |
| `docs/research/023_meeting-datasets/card-template.md` | 数据卡模板（暂定场景键 / 设备键，检查脚本的正本） |
| `docs/research/023_meeting-datasets/check_cards.py` · `test_check_cards.py` | 卡片检查（字段读模板，键直接读 021 / 022 正本）+ 20 条测试 |
| `docs/research/023_meeting-datasets/cards/`（44 张）· `notes/`（5 份） | 4 路调研 + 怀疑派产出 |
| `docs/research/023_meeting-datasets/cards-014-addendum.md` | 014 的 33 张卡补字段与疑误（由 4 份分路补充合并，原分路文件已删） |
| `docs/research/023_meeting-datasets/matrix.md` · `pick.md` · `decision.md` · `sources.md` | 覆盖表 · 建议先下 · 结论与待拍板 · 来源 |

## 发现
- 场景键已对齐 021 catalog v1（scenes 应我请求加了 `bus-metro-cabin`、`other`）；设备键已对齐 022 devices v1（macmic 加了 `mac-unknown` 与 8 个 `non-laptop-*` 子键）
- Mac 内置麦真实录音只有 DR-VCTK（回放）与 DNS 2021 自填 mac 约 31 条；开放办公 / 机场车站 / 街边无带主说话人的公开集
- 数据卡模板不抄键，check_cards.py 只认本仓库 021 / 022 正本，合并前用 VDN_KEYS_DIR 显式指；大总管排合并顺序 021、022 先于 023

## 待人拍板
4 题，题面、选项、默认与后果见 `docs/research/023_meeting-datasets/decision.md` 第四节：
1. 开发阶段要不要拿真实录音做评测（默认 A 维持 D62；建议 B 只当观察）
2. 下多少（默认 A 不下；建议随第 1 题）
3. 开放办公 / 机场车站 / 街边要不要自己录（默认 A 不录；建议 B 在家回放重录约 2 h）
4. 要不要申请 Ego4D（默认 A 不申请）

## 转交（范围外，留给大总管）
- Mac 设备相关材料 → macmic：`docs/research/023_meeting-datasets/notes/r1-device-macmic-handoff.md`（大总管已转）
- check_cards.py 依赖 021 / 022 正本；大总管已定合并顺序 021、022 先于 023
- 要注册 / 签协议的 17 项清单在 decision.md 第三节；本线未注册任何一项

## 流水
- 读交代、common、手下协议 → 完成
- 盘点 014 卡片字段 → 51 张；场景键 0/51、设备键 0/51、转写单列 1/51
- 向 5-app-5e 报到 → 已发
- 写调研计划 plan.md → 完成
- 大总管回「可以」，加 3 点（笔记本段可否按设备筛并转交 macmic · 找笑声附和情绪语音 · 与 D62 关系只列事实）→ 写进提示
- 起 workflow wf_98de0537-23d（4 路 + 4 怀疑派）→ 后台运行
- 写 card-template.md · check_cards.py · 测试 → 14 条测试过【实测】
- 4 路调研完成（8 代理，595 次工具调用，未下载音频）→ 新卡 44 张，怀疑派记 71 条改动
- 合并 4 份 014 补充为 cards-014-addendum.md（33 张卡，逐行核对无丢失）→ 删 4 份分路文件
- 场景键 v1、设备键 v1 到 → 迁移全部卡；检查脚本改为直接读 021 / 022 正本 → 20 条测试过、0 处问题
- 写 matrix · pick · decision · sources → 完成
- 大总管要求：不再去旁边 worktree 找正本 → 改为只认本仓路径 + VDN_KEYS_DIR + `--research` 参数（测试用），测试用内置小键表 → 23 条测试过；不设变量时退出码 2 并提示变量名【实测】
