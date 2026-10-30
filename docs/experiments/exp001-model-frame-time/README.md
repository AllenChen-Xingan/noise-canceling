# exp001 · 流式 ONNX 每帧推理耗时（本机实测）

> 2026-10-29 · 手下 model · 批次 20261029-b1

## 假设
DPDFNet-4 的 ONNX 在 M 系列单核上每 10 ms 帧能在 2 ms 内跑完（命门 1，`docs/research/002_model-baseline/must-know.md`）。

## 做法
- 机器：Apple M5（`sysctl machdep.cpu.brand_string`），macOS Darwin 27.0.0
- 运行时：onnxruntime 1.31.1（Python，`uv run --no-project --with onnxruntime --with numpy` 临时环境），CPUExecutionProvider，intra/inter op 线程都设 1，顺序执行，图优化全开
- 脚本：[bench.py](bench.py)。每个模型先空跑 200 帧，再计时 6000 帧；只计 `session.run` 一次的时间（**不含 STFT/iSTFT 与特征计算**）；输入是固定随机数，状态输出每帧回灌到状态输入
- 命令（项目根下）：
  ```bash
  W=resources/weights/dpdfnet; G=resources/gtcrn/stream/onnx_models
  uv run --no-project --with onnxruntime --with numpy python docs/experiments/exp001-model-frame-time/bench.py \
    $W/baseline.onnx $W/dpdfnet2.onnx $W/dpdfnet4.onnx $W/dpdfnet8.onnx \
    $W/dpdfnet2_48khz_hr.onnx $W/dpdfnet8_48khz_hr.onnx $G/gtcrn.onnx $G/gtcrn_simple.onnx
  ```
- 原始输出：[raw-output.txt](raw-output.txt)（去掉了 uv 的安装日志）

## 权重（不入库，在 `resources/weights/dpdfnet/`）
来源：`https://huggingface.co/Ceva-IP/DPDFNet/resolve/main/onnx/<名>.onnx`（Apache-2.0，模型卡）。sha256 与 HF 的 LFS oid 一致：

| 文件 | 字节 | sha256 |
|---|---|---|
| baseline.onnx | 8722707 | 371d26182aff0e1e0d31354e24c81f79cd57458f5a7dd003fc10a7ebf64255e0 |
| dpdfnet2.onnx | 10178747 | 4f0ee28935b4a32abecc717d745416976565834d839601acf43031094b4dc94c |
| dpdfnet4.onnx | 11633337 | 42a386d3601922f474c4309927e1f471cce422e9bd8e3fd3967bb943bac36c01 |
| dpdfnet8.onnx | 14542517 | 899d4f23f3ff86edbffa8c537e4bcbdc49da1b4e84e0ef390611e0604a3b26cb |
| dpdfnet2_48khz_hr.onnx | 10493337 | 7f0575a5cec0ba4ffd8f8bd657e06d007e4ccdd955d76faab922b9d3291dc14b |
| dpdfnet8_48khz_hr.onnx | 14857107 | 7b3afbb260a08fe9af3d16e3bda992971be1e7e951d1dee7c2d235f5c43f5631 |

GTCRN 用官方仓库自带的 `resources/gtcrn/stream/onnx_models/gtcrn.onnx`、`gtcrn_simple.onnx`（MIT，github.com/Xiaobin-Rong/gtcrn）。

## 结果（毫秒 / 帧）

| 模型 | 输入（ONNX 图） | 一帧代表 | p50 | p99 | p99.9 | max | 占帧时长（p99） |
|---|---|---|---|---|---|---|---|
| DPDFNet baseline（16k） | spec [1,1,161,2] + state_in [38256] | 10 ms | 0.285 | 0.316 | 0.355 | 0.484 | 3% |
| dpdfnet2（16k） | spec [1,1,161,2] + state_in [45424] | 10 ms | 0.745 | 0.783 | 0.823 | 1.152 | 8% |
| **dpdfnet4（16k）** | spec [1,1,161,2] + state_in [52592] | 10 ms | **1.214** | **1.263** | 1.300 | 1.869 | 13% |
| dpdfnet8（16k） | spec [1,1,161,2] + state_in [66928] | 10 ms | 2.143 | 2.205 | 2.243 | 3.240 | 22% |
| dpdfnet2_48khz_hr | spec [1,1,481,2] + state_in [56436] | 10 ms | 1.083 | 1.124 | 1.150 | 1.799 | 11% |
| dpdfnet8_48khz_hr | spec [1,1,481,2] + state_in [90228] | 10 ms | 3.142 | 3.242 | 3.926 | 5.725 | 32% |
| GTCRN | mix [1,257,1,2] + 3 个 cache | 16 ms（hop 256 @16k，`gtcrn_stream.py:456`） | 0.397 | 0.434 | 0.488 | 0.983 | 3% |
| GTCRN simple | 同上 | 16 ms | 0.396 | 0.430 | 0.496 | 0.608 | 3% |

一帧代表的时长：DPDFNet 16k 的 161 频点 = n_fft 320、hop 160；48k 的 481 频点 = n_fft 960、hop 480（hop = win/2，`resources/DPDFNet/onnx_model/infer_dpdfnet_onnx.py:196-199`），都是 10 ms。

## 结论
1. **命门 1 的算力部分：dpdfnet4 成立**（p99 1.26 ms < 2 ms，max 1.87 ms 也在内）。dpdfnet2_48khz_hr 同样过线（p99 1.12 ms）。**dpdfnet8 两版都超 2 ms**（16k p99 2.21，48k p99 3.24、max 5.7）。
2. **预判「ONNX 状态不显式」对 DPDFNet 与 GTCRN 都不成立**：两者的流式 ONNX 都把状态做成显式输入输出（DPDFNet 是一个扁平 float 向量，GTCRN 是 3 个 cache 张量），本实验正是靠回灌状态逐帧跑的。
3. 耗时分布很紧（p99.9 / p50 ≤ 1.25），max 偶有 1.5× 左右的尖峰，后续 Rust 实时线程里要看真实 p99.9 与最大值。

## 局限（别过度解读）
- 只量模型本体，不含 STFT/ERB/iSTFT（DPDFNet 的 STFT 在模型外，`infer_dpdfnet_onnx.py:202-232`）；这些是 O(n log n) 小活，但要在 Rust 里一并量
- Python 调用有固定开销，Rust `ort` 直接调用通常更低；没测 CoreML EP、没测 int8
- 随机输入、未核对输出音质；只说明「算得快」，不说明「效果好」
- 机器是 M5，比 M1/M2 快；在老机器上 dpdfnet4 余量会变小，需再测

## 改变了哪一个取舍
把「DPDFNet-4 能实时」从「按 GMACs 估算」升级为「本机实测」；把 dpdfnet8 从候选里排到「超预算」。
