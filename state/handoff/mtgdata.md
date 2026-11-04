# 现状页 · mtgdata（批次 20261103-b10）

- **状态**：进行中
- **手上**：023 开会场景的真实录音数据集（issue #82）
- **交代**：主目录 `_tmp/20261103-b10/mtgdata.md`
- **做到哪**（2026-11-03）：已报到；014 盘点完（51 张卡，缺场景键 / 设备键 / 单列转写）；大总管已批准，调研 workflow（8 个代理）后台在跑；卡片检查脚本已写，14 条测试过
- **下一步**：等 workflow 回来，合并成 addendum · matrix · pick · decision · sources

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `docs/research/023_meeting-datasets/plan.md` | 调研计划 |
| `docs/research/023_meeting-datasets/card-template.md` | 数据卡模板（暂定场景键 / 设备键，检查脚本的正本） |
| `docs/research/023_meeting-datasets/check_cards.py` · `test_check_cards.py` | 卡片字段检查 + 14 条测试 |

## 发现
- 021 场景键表、022 设备键表还没出，先用暂定键

## 待人拍板
（暂无）

## 转交（范围外，留给大总管）
（暂无）

## 流水
- 读交代、common、手下协议 → 完成
- 盘点 014 卡片字段 → 51 张；场景键 0/51、设备键 0/51、转写单列 1/51
- 向 5-app-5e 报到 → 已发
- 写调研计划 plan.md → 完成
- 大总管回「可以」，加 3 点（笔记本段可否按设备筛并转交 macmic · 找笑声附和情绪语音 · 与 D62 关系只列事实）→ 写进提示
- 起 workflow wf_98de0537-23d（4 路 + 4 怀疑派）→ 后台运行
- 写 card-template.md · check_cards.py · 测试 → 14 条测试过【实测】
