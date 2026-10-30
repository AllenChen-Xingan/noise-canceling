# 现状页 · model（批次 20261029-b1）

- **状态**：**已收尾**（2026-10-29），等大总管验收
- **手上**：起步用哪个降噪模型与代码作为基线（002）
- **交代**：`_tmp/20261029-b1/model.md`
- **结论一句话**：DPDFNet 效果最好但延迟约 30 ms 更多（算法 60 ms、端到端约 66–70 ms）；命门草稿下三条全过的只有 GTCRN。暂定首选 DPDFNet-2 + ort（需人放宽延迟上限），备选/默认 GTCRN，两者同接口接入
- **下一步**：等人拍板（见下）；拍板 #1 选 D 的话，离线 AB 听测可在下一批做

## 待人拍板（详见 `docs/research/002_model-baseline/decision.md` 末尾）
1. 端到端附加延迟上限定哪档：≤30 / ≤45（= 命门草稿，GTCRN）/ ≤75（DPDFNet，越过 G.114、对 intent 破例）/ D 先离线 AB 听测再定。建议 D；默认 ≤45
2. 设计稿 §16「M1 用 libDF 原样跑 DFN3」还做不做。建议跳过；默认跳过
3. 训练数据未披露的 DPDFNet 能否作开源默认模型。建议可以；默认可以
4. `must-know.md` 三条命门（AI 起草）

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| docs/research/002_model-baseline/must-know.md | 起草三命门、排除项、预判表 + 核实结果 |
| docs/research/002_model-baseline/sources.md | 两类来源登记（许可证与权重来源单列）+ 逐候选核查要点 |
| docs/research/002_model-baseline/gap.md | 候选 × 命门差距表、它们没做的 |
| docs/research/002_model-baseline/decision.md | 取舍卡：暂定首选/备选、四选一、三列延迟、单线程预算、失败条件、拍板 3 条 |
| docs/research/002_model-baseline/README.md | 状态栏 |
| docs/experiments/exp001-model-frame-time/{README.md,bench.py,raw-output.txt} | 每帧耗时实验（跑了两次，第二次修正方法；旧版见提交 89bf562） |
| resources/{DeepFilterNet,DPDFNet,gtcrn,biglinux-gtcrn,nnnoiseless}/ | 浅克隆（不入库）|
| resources/weights/dpdfnet/*.onnx | DPDFNet 6 个 ONNX（不入库，sha256 见 exp001） |
| state/handoff/model.md | 本页 |

## 流水
- 2026-10-29 报到（5-app-89 不可达，按更正发给 noise-canceling-69）
- 2026-10-29 查到官方仓库：DPDFNet = github.com/ceva-ip/DPDFNet；GTCRN = github.com/Xiaobin-Rong/gtcrn；DFN = github.com/Rikorose/DeepFilterNet
- 2026-10-29 克隆 5 个仓库；must-know.md 起草
- 2026-10-29 按人新要求跑 workflow wf_a4112186-a18（5 核查 + 5 怀疑派，只读）
- 2026-10-29 下载 DPDFNet 6 个 ONNX（约 70 MB，sha256 与 HF LFS oid 一致）
- 2026-10-29 exp001 第一次跑（名额借还）；大总管验收
- 2026-10-29 写 sources / gap / decision 初稿；加前瞻与端到端延迟表
- 2026-10-29 三视角自审：怀疑派 P0（命门草稿下 GTCRN 全过）、产品派 P0（拍板题面改成延迟上限、intent 破例要授权）、实现派 P1（wnorm 说法、对拍基准、GTCRN 接口差异、bench 随机 state）
- 2026-10-29 exp001 第二次跑（名额借还）：真实语音 + 元数据初始化 state + NaN 检查；结论不变；大总管抽查通过
- 2026-10-29 按自审意见改完四份文件；收尾
