# 现状页 · model（批次 20261029-b1）

- **状态**：进行中
- **手上**：起步用哪个降噪模型与代码作为基线（002）
- **交代**：`_tmp/20261029-b1/model.md`
- **做到哪**（2026-10-29）：四份文件初稿完成；自审中
- **下一步**：按自审意见修 → 收尾报大总管

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| state/handoff/model.md | 新建现状页 |

## 流水
- 2026-10-29 报到（5-app-89 不可达，按更正发给 noise-canceling-69）
- 2026-10-29 查到官方仓库：DPDFNet = github.com/ceva-ip/DPDFNet；GTCRN = github.com/Xiaobin-Rong/gtcrn；DFN = github.com/Rikorose/DeepFilterNet
- 2026-10-29 克隆 DeepFilterNet / DPDFNet / gtcrn / biglinux-gtcrn / nnnoiseless 进 resources/
- 2026-10-29 must-know.md 起草完（三命门待人改）
- 2026-10-29 按人新要求跑 workflow wf_a4112186-a18（5 核查 + 5 怀疑派，只读）
- 2026-10-29 下载 DPDFNet 6 个 ONNX 到 resources/weights/dpdfnet/（共约 70 MB，sha256 与 HF LFS oid 一致）
- 2026-10-29 写 exp001 bench.py，已申请名额，等批
- 2026-10-29 exp001 跑完并归还名额；结果 docs/experiments/exp001-model-frame-time/README.md + raw-output.txt
- 2026-10-29 workflow 完成（10 代理）；回源码抽查 DPDFNet 4 帧延迟成立
- 2026-10-29 写完 sources.md / gap.md / decision.md 初稿；按大总管要求加「前瞻 + 管线总延迟」表
- 2026-10-29 派 3 个对抗视角自审（怀疑派 / 实现派 / 产品派），等结果
