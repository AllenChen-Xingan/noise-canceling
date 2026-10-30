# exp002 · 评测打分工具冒烟

> 2026-10-29 · evaltool（批次 20261029-b1）· 名额经大总管批准，跑完已归还

## 假设

最轻的一组打分工具（DNSMOS ONNX + PESQ + STOI/ESTOI + SI-SDR + LUFS）能在本机 Apple Silicon 上纯 CPU、离线跑通，速度够进 CI。

## 做法

- 机器：Apple M5 · 32 GB · macOS 27.0.1 arm64
- 环境：`uv venv _tmp/exp002-venv --python 3.12`（3.12.14）；装 `onnxruntime 1.31.1` · `librosa 1.0.0` · `numpy 2.5.4` · `pesq 0.0.4` · `pystoi 0.4.1` · `pyloudnorm 0.2.0` · soundfile · pandas · tqdm · requests。venv 406 MB，留在 `_tmp/`，不进仓
- 模型：`resources/DNS-Challenge/DNSMOS/DNSMOS/sig_bak_ovr.onnx`（1.16 MB）+ `model_v8.onnx`（P.808，225 KB），都直接在仓里，不是 git-lfs 指针
- 样例（都是 `resources/` 里已有的公开样例，没另下载）：gtcrn 的含噪 / 增强对（16 kHz）；DeepFilterNet 的 clean / noisy_snr0（48 kHz，先转 16 kHz）
- 命令（在项目根目录下）：`_tmp/exp002-venv/bin/python docs/experiments/exp002-eval-smoke/smoke.py > docs/experiments/exp002-eval-smoke/results.json`
- 脚本：[smoke.py](smoke.py)；原始输出：[results.json](results.json)

## 结果（本机实测）

| 片段 | 时长 s | SIG | BAK | OVRL | P808 | DNSMOS 耗时 s | LUFS |
|---|---|---|---|---|---|---|---|
| gtcrn_mix（含噪） | 9.77 | 3.61 | 2.64 | 2.57 | 2.96 | 0.229（含首次预热） | −23.3 |
| gtcrn_enh（增强后） | 9.76 | 3.40 | 4.00 | 3.09 | 3.89 | 0.051 | −24.0 |
| dfn_clean（干净） | 10.60 | 3.55 | 4.17 | 3.29 | 3.90 | 0.048 | −29.7 |
| dfn_noisy_snr0（含噪） | 10.60 | 3.60 | 2.68 | 2.58 | 2.98 | 0.048 | −24.0 |

| 有参考对 | PESQ-WB | STOI | ESTOI | SI-SDR dB | PESQ 耗时 s | STOI+ESTOI 耗时 s |
|---|---|---|---|---|---|---|
| noisy_snr0 对 clean | 1.36 | 0.948 | 0.857 | 6.21 | 0.046 | 0.051 |
| clean 对 clean（自检） | 4.64 | 1.000 | 1.000 | 自比 | 0.057 | 0.101 |

模型加载 0.007 s，全部跑完 0.66 s（`time` 墙钟 1.36 s，含 Python 启动和 import）。

## 结论

1. **假设成立**：这组工具纯 CPU、离线在 M5 上跑通，DNSMOS 预热后每段约 10 秒的音频只要约 0.05 s（实时倍率约 0.005）。由此推算，1 小时音频打一遍 DNSMOS 约 20 s，**能进 CI**（推算值，没有实测长音频）。
2. **意外**：`resources/DNS-Challenge/DNSMOS/dnsmos_local.py:53` 用位置参数调 `librosa.resample(aud, input_fs, fs)`，librosa 0.10 起会报 `TypeError`，所以非 16 kHz 输入会直接崩。绕法：先自己转成 16 kHz（smoke.py 里就这么做的）。正式评测代码要么锁定 librosa 版本，要么自己做重采样。
3. **一个小信号，不能当结论**（每类只有 1 段）：两段含噪样例的 SIG（3.60–3.61）**反而高于**干净样例（3.55），gtcrn 增强后 SIG 从 3.61 降到 3.40，BAK 却从 2.64 升到 4.00。这说明 DNSMOS 的 SIG 对「人声有没有被伤」可能不灵敏、甚至方向相反，BAK 才主导 OVRL。这和意向书「无参考 MOS 只作筛选，不作结论」一致，也说明「吃字率 / 语音帧能量损失」这类专项尺子不能省。
4. 干净样例的 LUFS（−29.7）比含噪的（−24.0）低近 6 LU，说明合成时没有做响度对齐。以后评测「人声被压低」要先按语音段做响度归一化再比。

## 改变了哪一个取舍

- 确认 DNSMOS + PESQ/STOI/SI-SDR + LUFS 这一组可以作为「每次改动都跑」的 CI 层，**不需要 GPU，也不需要 PyTorch**（只要 onnxruntime）
- 加了一条实现约束：评测代码自己负责重采样，不依赖 `dnsmos_local.py` 的重采样路径

## 没做的（待实测）

NISQA、UTMOS/UTMOSv2、URGENT-PK（都要 PyTorch，venv 会更大）· Whisper 系吃字率的速度 · 长音频（≥10 分钟）的实际耗时 · CoreML 执行器比 CPU 快多少（本次用的是 onnxruntime 默认执行器）
