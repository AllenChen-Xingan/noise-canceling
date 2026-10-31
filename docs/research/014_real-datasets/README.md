# 014 · 真实录音公开数据集（真实回归与 sim-to-real 校准）

> 2026-10-31 · 手下 datasets。人的原话：「这个实际上的东西或者真实效果的东西，帮我找公开数据集」。

## 结论

**没有任何公开集是「笔记本内置麦 + 咖啡馆 + 有参考」。** 最贴近的是 DNS Challenge（ICASSP 2022 / 2023）真实测试集（笔记本或台式机录、1–3 m、邻座重叠，但无干净参考、无逐帧标签）。首选组合：DNS 真实测试集 · CHiME-6 · AliMeeting · DR-VCTK · CHiME-4 咖啡馆段（付费，待拍板）或 ACE 笔记本档。详见 [decision.md](decision.md)。

## 文件

| 文件 | 内容 |
|---|---|
| [must-know.md](must-know.md) | 三条命门：真实录音 · 可对齐参考 · 许可可得；预判 |
| [sources.md](sources.md) | 来源登记与时效 |
| [assign.md](assign.md) | 这次怎么派活（Workflow 的分组与核查项） |
| [gap.md](gap.md) | 格子对照：哪些格有数据、哪些格公开数据补不上 |
| [decision.md](decision.md) | 首选组合、用途、获取步骤、怎么接 013 / 012、待人拍板、卡片冲突裁定 |
| [cards/](cards/README.md) | 51 张数据集卡（索引按接近度排序） |
| [notes/](notes/) | 五路调研的「未找到 · 预判核验 · 怀疑派更正 · 漏项」原始记录 |

**本线没有下载任何数据集本体，也没有运行任何程序**；所有大小、许可都来自网页 / 论文 / LICENSE，逐条出处在卡片里。
