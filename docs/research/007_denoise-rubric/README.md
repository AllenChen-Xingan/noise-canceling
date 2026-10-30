# 007 · 降噪 rubric：评测降噪效果与人耳听感，并用它筛现成实现

> 2026-10-30 · 手下 rubric · 批次 20261030-b2 · 分支 `research/007-denoise-rubric`

## 问的是什么

人的原话：「需要来做一个基于降噪或者之类的 rubric，来评测降噪效果，以及实际上人的听感效果；就是来筛选我们用哪些可以用的已有的实现」。

004 已经选好了**量具**（拿什么量）；本线定的是**判据**（量出来的数怎么判：哪级、否决没有、怎么加权、人耳和机器打架信谁）。

## 文件

| 文件 | 装什么 |
|---|---|
| [`../../../specs/rubric/denoise.md`](../../../specs/rubric/denoise.md) | **rubric 正文**（标准，被引用的那份） |
| [`decision.md`](decision.md) | 筛选表：用 rubric 给已知候选打分 + 待人拍板 |
| [`must-know.md`](must-know.md) | 设计 rubric 前必须知道的判断（调研结论） |
| [`sources.md`](sources.md) | 来源登记 |
| [`gap.md`](gap.md) | rubric 要的数据 × 现在有没有；要补的实验 |
| [`dryrun-exp003.md`](dryrun-exp003.md) | 用 exp003 临时机器分预演一遍 rubric（**作者听完 AB 再看**） |

## 在三层验证里的位置（CLAUDE.md 第三节）

- 规则判（脚本）：否决项里能机器判的、指标落档 → `specs/rubric/denoise.md` 第 8 节给了脚本规格
- **判据判（本 rubric）**：AI 按写下的锚点与裁决规则代判
- 实效判（人）：人耳部分只吃作者在 exp003 AB 页上的答案；真实会议听感仍是最高真相源，rubric 判不了
