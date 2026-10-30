# 现状页 · pse（批次 20261030-b2）

- **状态**：进行中
- **手上**：005 个性化降噪（PSE）：模型 · 自动注册声纹 · 评测
- **交代**：`_tmp/20261030-b2/pse.md`
- **做到哪**：六份已填完，三视角自审中（怀疑派 · 实现派 · 产品派）
- **下一步**：按自审意见修改 → 收尾

## 待人拍板
（调研完填）

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| docs/research/005_pse/*（六份） | 骨架 + 填满：must-know 三命门与核实结果、sources、assign 派活记录、gap、decision（6 条待人拍板） |
| docs/research/README.md · docs/plan/README.md | `research.sh` 自动各登记一行（交代范围外，已报大总管） |
| resources/{voxlock,Hush,ID-PSE,D-LGTSE,G-MaP-SE,DENSE,SEF-PNet,USEF-TSE,wesep,ClearerVoice-Studio,speakerbeam,speaker_extraction_SpEx,wespeaker,3D-Speaker,diart,...} | 子代理浅克隆（不入库） |
| state/handoff/pse.md | 本页 |

## 流水
- 2026-10-30 报到 noise-canceling-69
- 2026-10-30 读完 common / 交代 / 设计稿 §5 §9 §10 §11 §13 / 002 must-know·decision·gap / 004 babble 行
- 2026-10-30 `research.sh` 建 `docs/research/005_pse/`（自动提交 fdc142c）
- 2026-10-30 起 workflow wf_7240073b-611（先 18 代理，按大总管要求停掉改 12 代理续跑）；填 assign.md 派活记录
- 2026-10-30 workflow 完成（14 代理，含被停掉那次的 2 个）；pse 回源复核 VoxLock README/NOTICE/core.py/CHANGELOG、WeSpeaker pretrained.md:19-23、3D-Speaker #268
- 2026-10-30 发现：workflow 期间 hook 把根目录 LICENSE（6e0d8c2）扫进提交，非 pse 所写，已报大总管
- 2026-10-30 写 sources / gap / decision、补 must-know 核实结果与 README 状态；报大总管 exp-A 需求
- 2026-10-30 派三视角自审
