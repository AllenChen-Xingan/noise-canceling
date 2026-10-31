# FastEnhancer-M 场景怎么修音（缺口调研，只读，不下效果结论）

> 2026-10-31 · 人问「FE-M 场景修音怎么办」，大总管转。只写事实与测量计划，**不下效果结论**（D46）。
> 标注：【官】官方文档 ·【源】源码 ·【测】本机计算 ·【推】推断。

## 背景
修音链的频谱节点（平滑 / 下限 + 舒适噪声 / 去发闷）要吃**模型自己的 STFT** 才零延迟。GTCRN 适配器本来就做 512/256 STFT，挂得上；
我们现在用的 FE-M 是 wav2wav 图（STFT / iSTFT / 重叠相加都在图里），拿不到频谱——所以本批路由表给 FE-M 场景（babble、音乐）只开 AGC（tone 22 / 25）。

## ① 官方有没有「STFT 在图外」的 FE-M —— 有
- 官方 release `onnx-dns-v1.0.0`（2025-10-10）同时有 `fastenhancer_m.onnx`（wav2wav，2 033 752 字节，我们现用）和 **`fastenhancer_m.spec.onnx`**（spec2spec，2 029 871 字节）【官：`gh release view onnx-dns-v1.0.0 -R aask1357/fastenhancer`】
- spec2spec = 「STFT 和 iSTFT 在 PyTorch 里做，ONNX 只算网络」；论文的 RTF 就是用 spec2spec 量的【官：`resources/fastenhancer/docs/docs/onnx.md:7-10`】
- 接口：每次喂一帧 `spec_in`（`[B, F+1, 1, 2]`，实虚部），缓存 `cache_in_*` 回灌，出 `spec_out`【源：`resources/fastenhancer/scripts/test_onnx_spec.py:44-61`、`export_onnx_spec.py:135-140`】
- FE-M 的 STFT：`n_fft 512 · hop 160 · win 512 · window hann`（不是 sqrt-Hann）【源：`resources/fastenhancer/configs/fastenhancer/m.yaml:17-20`】；分析 `torch.stft(..., window=hann, normalized=False)`，合成 `torch.istft(..., window=hann)`【源：`test_onnx_spec.py:19-27, 63-67`】

### 换用它之后
| 问题 | 答案 |
|---|---|
| 能否与 GTCRN **共用同一次** STFT？ | **不能**：hop（160 vs 256）和窗（Hann vs sqrt-Hann）都不同，帧网格对不上【源】。能共用的是 `vdn-dsp::StreamingStft` 这个**积木**（它按 n_fft / hop / 窗参数化），各自一套实例 |
| 频谱修音链能否照搬？ | 代码上可以：`SpectralPolish::new(n_fft, hop, sr)` 本来就按 hop 算时间常数，FE-M 用 `new(512, 160, 16000)` 即可【源：`src/polish/src/spectral.rs`】；预设数值是在 GTCRN 网格上起的，换网格要重标【推】 |
| 额外延迟 | **0**：spec2spec 的流式延迟 = n_fft − hop = 352 样本 = 22 ms，与现用 wav2wav 相同（官方文档：wav2wav 也是 `n-h` 的延迟）【官：`onnx.md` 「More information about wav2wav」；推：两者同一网格】。修音链挂在中间不再加 |
| ⚠️ Σw² 归一化 | Hann 窗分析 + Hann 窗合成、hop 160：重叠相加的窗平方和**不是常数**——本机算得 1.1905–1.2101，**每 10 ms 起伏 0.14 dB**（即 100 Hz 的调制）【测】；sqrt-Hann / hop 256 是严格的 1.0【测】。`torch.istft` 会按窗包络逐样本除掉【官：PyTorch istft 文档】，而 `vdn-dsp::StreamingStft::synthesize` 只做「iFFT × 窗 → 叠加」、不除包络【源：`src/dsp/src/stft.rs` synthesize】。直接拿来用 FE-M 会出 exp003 说的「Σw² 没归一的 16 ms 嗡声」那类伪影。**修法**：合成端除以预先算好的周期包络（周期 = hop 160，长度固定，不加延迟），或换成对偶合成窗 `w / Σw²`——要改 `vdn-dsp`（align 线的 crate），本线不动 |

## ② 若只用时域（不要频谱）能做什么
| 做法 | 零延迟吗 | 管得了 | 管不了 |
|---|---|---|---|
| AGC + 子帧限幅（已实现） | 是（只用本 hop 已在手的样本）【源】 | 忽大忽小 | 其余三种 |
| biquad 均衡：固定或慢慢自适应的高搁架（去发闷）、动态齿音压制 | 无前瞻；最小相位滤波有很小的群延迟（高频段远小于 1 ms，低频搁架更大）【推，需实测】 | 发闷（粗粒度，一两个频段） | 细频段补偿 |
| 滤波塑形的舒适噪声：白噪声过几个 biquad，增益按「带噪输入的频带底噪」定（底噪可用判断回路的 `FrameFeatures.bands_db` 或 scene 的底噪谱形特征，旁路分析，不在音频路径上） | 是（噪声发生器 + IIR）【推】 | 真空感 | — |
| 衰减下限：把与模型对齐好的原声按 g 混回（engine 送原声本来就有对齐好的原声延迟线，规约 §3.6） | 是（原声已对齐到模型延迟，不再加）【源：规约 §3.3「送原声用 48 kHz 原声（Pipeline 的延迟线）」】 | 真空感 | 不挑内容——邻座的话也一起带回来（「只留你的声音」场景不能开大） |
| — | — | — | **音乐噪声（金属声）**：它是逐频点增益乱跳造成的，时域滤波器组拿不到逐频点增益，**管不了**【推】 |

另一条中间路线：**旁路分析 + 时域滤波器组施加**——对 FE-M 输出只做 STFT 分析（不合成），算出若干频带的平滑增益，再用最小相位 IIR 滤波器组在时域施加。无前瞻，但增益落后一个分析窗、频带分辨率粗（音乐噪声是逐频点的，粗频带抹不掉）【推】。

## ③ 在 FE-M 输出上再做一次短窗 STFT 的代价
完整的「分析 → 改 → 合成」至少多 **n_fft − hop** 个样本（输出要等一整窗）【推，与 `StreamingStft::latency_samples` 同式】：

| 窗 / hop（16 kHz） | 额外延迟 | 频率分辨率 |
|---|---|---|
| 512 / 256 | 256 样本 = **16 ms** | 31 Hz |
| 256 / 128 | 128 = **8 ms** | 62.5 Hz |
| 128 / 64 | 64 = **4 ms** | 125 Hz |

都违反「修音链零前瞻」（本批硬约束 1），且会拖慢路由池里所有场景（引擎按池内最大延迟补齐）。

## 测量计划（下一批，要名额）
1. 下载 `fastenhancer_m.spec.onnx`（约 2 MB，记 sha256 进本文件），与现用 wav2wav 在 exp003 / exp006 素材上逐样本对比：Hann 分析 + **包络归一**合成后，差值应在 1e-4 量级；同时测「不归一」时的 100 Hz 调制电平，确认嗡声来源
2. 冲激测延迟：spec2spec 链（我们的 STFT + ONNX + 我们的 iSTFT）对齐滞后应是 352 样本
3. `SpectralPolish::new(512, 160, 16000)` 挂在 FE-M 频谱上，进 exp006 `offline_ab.py` 作第 5 版；客观分 + 盲听，与「FE-M 只开 AGC」对比
4. 算力：spec2spec 与 wav2wav 每帧耗时（官方说论文 RTF 用的是 spec2spec）
5. 需要 align（或下一批 engine 接入）在 `vdn-dsp::StreamingStft` 加「合成端窗包络归一」选项

## 待人拍板（并入 exp006 README）
- FE-M 场景的修音：A 换 spec2spec 版（推荐：零额外延迟、整条修音链可用；代价是改 dsp 的合成归一 + 重标预设 + 重新对拍）· B 维持只开 AGC（零成本，管不了金属声 / 真空感）· C 在 FE-M 上叠时域补丁（滤波舒适噪声 + 高搁架），管不了金属声
