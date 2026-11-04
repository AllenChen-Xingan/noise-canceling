# 现状页 · 手下 scenes（021 开会场景目录）

- 大事：降噪效果（D80）第一步——补评测维度
- 管的实体：会议场景 · 噪声类别 · 场景声学条件
- worktree `../noise_canceling-scenes` · 分支 `research/021-meeting-scenes` · issue #80
- 状态：**收尾中**：产出已齐，在做换模型评审（codex）（2026-11-03）

## 产出（都在 `docs/research/021_meeting-scenes/`）

| 文件 | 是什么 |
|---|---|
| `catalog.md` | 场景键表 v2：15 个开会地点 + 兜底 `other`，每个地点的声级 · T60 · 旁人说话 · 突发 · 难点 · 评测覆盖 · 用户占比 · 公开真实录音；末节回答「有没有人做过会议场景分类」 |
| `coverage.md` | 地点 × 9 个随地点变的因素（016 F1–F16 中的 9 个）矩阵：43 个空格；补的先后分 4 批；与用户层 5 类、噪音 7 类的对应（只提议） |
| `decision.md` | 结论 · 我定的技术事项 · 待人拍板 4 题 · 转交 |
| `sources.md` | 64 条出处，按怀疑派判定标了是否核过原文 |
| `check_catalog.py` + `test_check_catalog.py` | 检查 K1–K6（键唯一、字段齐、带单位的数字有出处、出处编号存在、覆盖矩阵与主表一致）；18 条测试全过【实测】 |
| `raw/` | 调研 workflow 的 5 份原始报告 + 2 份怀疑派判定（供追溯） |

复现检查：`python3 docs/research/021_meeting-scenes/check_catalog.py` → `ok 主表 16 个键，问题 0 处`；`python3 docs/research/021_meeting-scenes/test_check_catalog.py` → `Ran 18 tests … OK`。

## 待人拍板（题面全文在 `decision.md` 第三节）

1. 多人围着一台 Mac 开会，要不要算进评测范围？默认：先不算
2. 用户能选的场景要不要多分出「家里有人」？默认：先不改代码
3. 开放工位、候车厅、街边没有公开真实录音，开发阶段要不要破例自录？默认：不破例（守 D62）
4. 补评测先补哪里？默认：先补家里客厅和开放工位

## 转交

- **016 评测基准**：加 F17 风噪；F6 混响网格加到 8 s 以上；F3 声级与 SNR 联动没有任务实现；F3 斜率来自老人被试；「香港 73.9 dBA」出处是 To & Chung 2015 摘要
- **macmic**：最接近 MacBook 放桌上的标准布置是 ETSI TS 103 224「桌面免提」（表 8A）；3GPP 没有桌面免提噪声测试
- **mtgdata**（它的会话已不在，消息没发出，请大总管转）：catalog 升 v2，键与第一列不变，只往右加 8 列；测试里有一条按它的读法复读主表并比对，已过。补空格线索：DCASE 2016 TUT 声景（library / train / tram / metro station，无主说话人）；AliMeeting 会场参数可能含 RT60
- **大总管**：噪音 7 类缺 4 种声音（外放人声、小孩和狗、键盘与提示音、风噪），是否改分类器属于场景分类器那件事
