# exp001 · 流式 ONNX 每帧推理耗时（本机实测）

> 2026-10-29 · 手下 model · 批次 20261029-b1。本页是第二次跑的结果；第一次的方法有缺陷，见文末「与上一次的差别」。

## 假设
DPDFNet-4 的 ONNX 在 M 系列单核上每 10 ms 帧能在 2 ms 内跑完（命门 1，`docs/research/002_model-baseline/must-know.md`）。

## 做法
- 机器：Apple M5（`sysctl machdep.cpu.brand_string`），macOS Darwin 27.0.0
- 运行时：onnxruntime 1.31.1（Python，`uv` 临时环境），CPUExecutionProvider，intra/inter op 线程都设 1，顺序执行
- 输入：真实带噪语音 `resources/gtcrn/test_wavs/mix.wav`（16 kHz，约 9.8 s，循环拼接到够长）；48 kHz 模型用线性插值升采样（只为计时，不评音质）
- 每帧：numpy 做 STFT（DPDFNet 用 vorbis 窗、n_fft/hop 从 ONNX 元数据读；GTCRN 用周期 sqrt-Hann 512/256）→ `session.run` → iSTFT + 重叠相加
- 状态：DPDFNet 按 ONNX 元数据 `erb_norm_init` / `spec_norm_init` 初始化，其余为 0；GTCRN 三个 cache 全零。每帧把状态输出回灌
- 检查：每帧输出是否全为有限值；输出与输入频谱能量比（确认模型没在输出静音）
- 先空跑 200 帧，再计时 6000 帧；分别记「模型 run」和「STFT + iSTFT」
- 脚本：[bench.py](bench.py)；原始输出：[raw-output.txt](raw-output.txt)

命令（项目根下）：
```bash
uv run --no-project --with onnxruntime --with numpy python docs/experiments/exp001-model-frame-time/bench.py
```

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

GTCRN 用官方仓库自带的 `resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`（MIT，github.com/Xiaobin-Rong/gtcrn）。

## 结果（毫秒）

「合计 max」= 模型的 max 与 STFT/iSTFT 的 max 各自取最大后相加，两者不一定出现在同一帧，所以是**保守上限**；「合计 p99.9」同理。

| 模型 | 周期 | 模型 p50 | 模型 p99 | 合计 p99.9 | 合计 max | max 占周期 | ≤ 2 ms 线 | ≤ 周期一半（5 ms / 8 ms） | NaN 帧 |
|---|---|---|---|---|---|---|---|---|---|
| DPDFNet baseline | 10 ms | 0.287 | 0.312 | 0.37 | 0.56 | 6% | ✅ | ✅ | 0 |
| **dpdfnet2** | 10 ms | 0.746 | 0.779 | **0.84** | **0.89** | **9%** | ✅ | ✅ | 0 |
| dpdfnet4 | 10 ms | 1.218 | 1.252 | 1.32 | 1.40 | 14% | ✅ | ✅ | 0 |
| dpdfnet8 | 10 ms | 2.148 | 2.197 | 2.26 | 2.52 | 25% | ❌ | ✅ | 0 |
| dpdfnet2_48khz_hr | 10 ms | 1.084 | 1.118 | 1.18 | 1.25 | 13% | ✅ | ✅ | 0 |
| dpdfnet8_48khz_hr | 10 ms | 3.178 | 3.233 | 3.47 | 3.99 | 40% | ❌ | ✅ | 0 |
| **GTCRN simple** | 16 ms | 0.400 | 0.422 | **0.48** | **0.62** | **4%** | ✅ | ✅ | 0 |

STFT + iSTFT（numpy）每帧 p99 只有 0.02–0.04 ms，可以忽略。所有模型输出与输入的能量比都在 −2.6 到 −2.8 dB：模型在工作（没有原样输出，也没有输出静音），但这个数**不说明音质**。

## 结论
1. **命门 1 的算力部分：dpdfnet2 / dpdfnet4 / dpdfnet2_48khz_hr / GTCRN 都过**（最坏一帧也不到 2 ms）；dpdfnet8 两版超 2 ms，但仍在「周期一半」线内。
2. **单线程推理预算**（供 003 定线程优先级）：首选 dpdfnet2 最坏一帧 0.89 ms，占 10 ms 周期 9%；备选 GTCRN 0.62 ms，占 16 ms 周期 4%。离 pipeline 建议的「≤ 周期一半（约 5 ms）」线都有 5 倍以上余量。
3. **DPDFNet 与 GTCRN 的 ONNX 状态确实显式、可用**：按元数据初始化后，用真实语音逐帧回灌 6000 帧，0 个 NaN 帧。
4. 耗时分布很紧（p99.9 / p50 ≤ 1.15），这次最大值也稳。

## 局限（别过度解读）
- 测的是 Python 里的 ORT 1.31.1；Rust `ort` 2.0.0-rc.15 捆绑的 ORT 是 1.31.0（`sources.md` 零件节），版本相近但不是同一个，要在 Rust 里复测
- 没测 CoreML EP、int8；没测 M1/M2 等老机器（M5 更快，老机器余量会变小）
- 只量了算得多快，没量效果；没跑真实 CoreAudio 回调，没测推理线程与音频线程之间的交接
- 48 kHz 输入是线性插值升上来的，高频是空的；只影响计时代表性，不影响「能不能跑完」

## 改变了哪一个取舍
把「DPDFNet 能实时」从「按 GMACs 估算」升级为「本机实测」；把 dpdfnet8 从候选里排到「超 2 ms 预算」；给 003 提供了单线程预算的实测余量。

## 与上一次的差别
第一次（同日早些时候，脚本见 git 历史提交 `89bf562`）有两处缺陷，由自审实现派指出：

| | 第一次 | 这一次 |
|---|---|---|
| 输入 | 随机数 × 0.01 | 真实带噪语音逐帧 STFT |
| 状态 | 随机数起步（DPDFNet 的谱归一化里有开方，负数状态可能出 NaN） | 按 ONNX 元数据初始化 |
| 输出检查 | 没有，「跑通」只说明没崩 | 每帧查有限值 + 能量比 |
| STFT 耗时 | 没算 | 单独计时 |

数字变化：模型本体 p50 / p99 几乎不变（例：dpdfnet2 p99 0.783 → 0.779，dpdfnet4 1.263 → 1.252）；最大值更稳（dpdfnet2 1.152 → 0.842，dpdfnet8_48khz_hr 5.725 → 3.887），第一次的尖峰多半是随机输入或机器上其他进程的干扰，未细查。**结论没有变**，但「状态显式可用」现在才有证据。
