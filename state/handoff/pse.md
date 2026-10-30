# 现状页 · pse（批次 20261030-b2）

- **状态**：**已收尾**（2026-10-30），等大总管验收
- **手上**：005 个性化降噪（PSE）：模型 · 自动注册声纹 · 评测
- **交代**：`_tmp/20261030-b2/pse.md`
- **做到哪**：六份填完，三视角自审意见已全部改完；VoxLock 接入说明已交 abtest（gap.md 末节）
- **结论一句话**：现成「只留主人声音」的开源模型只有 VoxLock（Apache-2.0、因果、ONNX、自动注册、真实录音证据），但 Xeon 上算力是预算 3–5 倍、M5 未测；暂定分阶段——先「通用模型 + 自写注册器 + 不对称保守门控」（不训练、+0 ms），VoxLock 在 exp003 实测，测完再定换它还是自训小模型
- **下一步**：等大总管验收；等 exp003 的 VoxLock 数字回来再推拍板 #1 #3

## 待人拍板（详见 `docs/research/005_pse/decision.md` 末节）
1. PSE 允许训练吗：A 不训练 · B 允许训小模型（VoxLock 当老师）· C 等 exp003 再定。建议 C，默认 C
2. 接不接受 VoxCeleb（研究数据集，CC BY 4.0）训练的声纹模型：建议接受，默认接受
3. 为「只留你的声音」愿意多耗电吗：建议等 exp003 数字，默认不放宽
4. 「零设置」口径：建议首场会议通用模型兜底、几分钟内生效（默认同）
5. 本机最多存什么（隐私）：建议只存声纹向量，VoxLock 的 3 s 只放内存（默认同）
6. 怎么让用户知道已认出 / 锁错：建议状态显示 + 一键重置 + 被压低提示；默认只做状态 + 重置
7. 能否去 VoxLock 仓开 issue 要书面权重许可（对外发言）：建议等 exp003 证明值得用再发；默认不发
另：must-know 三条命门仍是 AI 草稿，核实后的修改建议写在 must-know.md 末尾，待人改

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| docs/research/005_pse/*（六份） | 骨架 + 填满：must-know 三命门与核实结果、sources、assign 派活记录、gap（含给 abtest 的 VoxLock 接入说明）、decision（7 条待人拍板） |
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
- 2026-10-30 大总管：LICENSE 是其有意加的；VoxLock 实测并进 exp003。写 gap.md 末节「给 abtest 的 VoxLock 接入说明」并回报
- 2026-10-30 三视角自审回来：怀疑派 1 个 P0（VoxLock 延迟推算重复计步长、漏计算时间）+ 多条 P1/P2；实现派 2 个 P0（门控须不对称、切 VoxLock 须预热）；产品派 1 个 P0（默认值合起来把核心功能推后）。全部改完：重写 decision.md，修 gap / sources / must-know
- 2026-10-30 已收尾

## 转交（范围外的发现）
- 002 选型若最终选 48k 版 DPDFNet，个性化切换到 16k 的 VoxLock 会掉带宽，engine 接口要统一采样率（decision.md「并存」一节）
- engine 接口建议在 M1 就留好：每个模型声明 `latency_samples`、注册条件作为模型输入、模型可在通话中切换并预热
