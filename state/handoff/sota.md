# 现状页 · sota（批次 20261030-b2）

- **状态**：进行中
- **手上**：006 降噪前沿全景（GitHub + 论文 + 榜单 + 厂商 + 功耗 + 未好好实现的方法）
- **交代**：主目录 `_tmp/20261030-b2/sota.md`
- **做到哪**：六份已写完，三视角自审中
- **下一步**：Workflow（六块检索 → 怀疑派 → 完整性批评）→ 填 sources / gap / decision → 三视角自审

## 待人拍板
（调研完填）

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| docs/research/006_sota-landscape/*（六份） | `research.sh` 生成骨架 |
| docs/research/README.md · docs/plan/README.md | `research.sh` 自动各登记一行（交代范围外，同 pse 线先例，已报大总管） |
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
