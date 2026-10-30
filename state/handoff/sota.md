# 现状页 · sota（批次 20261030-b2）

- **状态**：**已收尾**
- **手上**：006 降噪前沿全景（GitHub + 论文 + 榜单 + 厂商 + 功耗 + 未好好实现的方法）
- **交代**：主目录 `_tmp/20261030-b2/sota.md`
- **做到哪**：006 六份写完，三视角自审修完
- **下一步**：等大总管验收；拍板见下

## 待人拍板（详见 `docs/research/006_sota-landscape/decision.md` 末节）
1. 做不做「延迟能不能被察觉」实验（D30），顺带确认人答的「5」是什么意思 —— 建议 A 下一批做；默认 A
2. FastEnhancer 并入 exp003 当第三候选 —— 建议 A；默认 A
3. 拿 RX（试用 / 借用）离线处理当 AB 天花板参照 —— 建议拿得到就 A；默认 B
4. 去 xiph / CoFi-Lite / Scicom 开 issue 问许可（对外发言）—— 建议只问 xiph；默认都不问

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| docs/research/006_sota-landscape/*（六份） | `research.sh` 生成骨架 |
| docs/research/README.md · docs/plan/README.md | `research.sh` 自动各登记一行（交代范围外，同 pse 线先例，已报大总管） |
| docs/research/006_sota-landscape/*（六份） | 填满：must-know · assign · sources · gap · decision · README |
| 主目录 resources/fastenhancer | 浅克隆（MIT，67 MB，只读） |
| state/handoff/sota.md | 本页 |

## 流水
- 2026-10-30 报到 noise-canceling-69
- 2026-10-30 读完 common / 交代 / intent / 路线图 / 28 条拍板 / 002 must-know·decision / 005 decision·sources / 设计稿 §4 §5 §7–§9 §11 §18
- 2026-10-30 `research.sh` 建 `docs/research/006_sota-landscape/`（自动提交 ae31338）
- 2026-10-30 Workflow 计划报大总管并获批；起 wf_5b5f8eb3-fe1（6 检索 + 3 怀疑派 + 1 批评者），中间产物在 scratchpad/wf/
- 2026-10-30 大总管转人新要求（训练放最后、现成模型 + 跨行业让延迟不可感、能耗低）：加第⑦块，起 wf_b412f601-4d1（1 检索 + 1 怀疑派）；⑥与前 5 件按「不训练优先」排
- 2026-10-30 主 Workflow（10 代理）与第⑦块完成；大总管转人追问「降噪后补偿 / RX」→ 起第⑧块 wf_55ec3686-951（总代理数 14，已报）
- 2026-10-30 亲自核 FastEnhancer：MIT、状态显式（`scripts/test_onnx_spec.py:44-61`）、算法延迟约 32 ms（`docs/docs/onnx.md:48-50`）；浅克隆进主目录 `resources/fastenhancer`
- 2026-10-30 写 sources.md（按怀疑派改正）；等第⑧块再写 gap / decision
- 2026-10-30 第⑧块完成；写完 gap.md · decision.md · README 状态；起三视角自审 wf_d48a7498-40f（怀疑派 · 实现派 · 产品派）
- 2026-10-30 三视角自审（wf_d48a7498-40f）：P0 ×2（exp003 分帧不做 Σw² 归一化会冤枉 FastEnhancer；拍板 #1 默认与 D30 冲突）、P1 若干（端到端延迟没算缓冲、48k 版许可、失败条件无阈值、耗电挂 D31、一句话没答人问的事等）→ 全部修进 decision / gap / sources
- 2026-10-30 已收尾，报大总管
