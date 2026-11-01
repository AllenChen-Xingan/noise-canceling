# 现状页 · ears（批次 20261031-b6）

- **状态**：进行中
- **手上**：找一切办法把「更多人耳」接进评测，做成可验证的东西（issue #42）
- **交代**：主目录 `_tmp/20261031-b6/ears.md`
- **做到哪**（2026-10-31）：已报到；实验编号 exp009；起深调研 workflow
- **下一步**：调研回来后挑一个公开人评集做 exp009（众人耳 vs 预测器 vs 单个听众）

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `state/handoff/ears.md` | 新建 |

## 发现
- 

## 待人拍板
（收尾时汇总）

## 转交（范围外，留给大总管）
- 

## 流水
- 读交代、common、协议、intent、待拍板、D44/D48、exp007 calib → 作者一致率基线：DNSMOS OVRL 0.48 [0.26,0.70]（48 对）
- 报到大总管（noise-canceling-c4）→ 给 exp009；名额在 benchsynth 手里
- 起 workflow wf_ad512269-bb9（7 调研 + 1 批评，笔记写 docs/research/017_more-ears/notes/）
- 摸本机：DNSMOS onnx（_tmp/exp002-venv）、NISQA 权重（resources/NISQA/weights）、torch venv（_tmp/exp008-torch-venv，benchsynth 的，只读借用或自建 exp009-venv）；exp007 定标法在 src/eval/src/calib.rs（同屏配对、按屏聚类 bootstrap、种子 20261030）——exp009 照此口径
- 大总管追加 H 线（CPC/DNS5 WAcc/SSI/vmc2026）→ 派子代理写 notes/H-extra-sources.md
