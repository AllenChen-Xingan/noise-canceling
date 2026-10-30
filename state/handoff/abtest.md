# 现状页 · abtest（批次 20261030-b2）

状态：进行中

## 手上
exp003：DPDFNet-2 对 GTCRN 的离线 AB 听测（交代 `_tmp/20261030-b2/abtest.md`）

## 做到哪
读完前置材料；正在下载合成素材（LibriSpeech test-clean 347 MB · DEMAND PCAFETER_16k 107 MB · DKITCHEN_16k 111 MB · SCAFE_48k 326 MB，均 <500 MB，放 `~/vdn-data/`）。

## 下一步
找背景音乐与中文素材 → 写合成 / 处理 / 打分脚本 → 申请名额跑 → 写对比页 → README → 自审

## 待人拍板
（暂无）

## 流水
- 2026-10-30 读交代、common、002 decision、exp001、exp002、004 AB 行；向大总管报到
- 2026-10-30 盘点：现成带噪 6 段（urgent casino ×3 有干净参考、DFN noisy_snr0 有参考、gtcrn mix、PESQ bab 仅 3.1 s）；whisper-cli 已装但无正式 ggml 模型；`_tmp/exp002-venv` 可复用（onnxruntime/librosa/pystoi/pyloudnorm）
- 2026-10-30 DEMAND 在 Zenodo 标 cc-by-4.0（交代写 CC BY-SA，以 Zenodo 为准，仍按 D26 只评测）；SCAFE 只有 48k 版
- 2026-10-30 后台开始下载 LibriSpeech test-clean + DEMAND 三套
