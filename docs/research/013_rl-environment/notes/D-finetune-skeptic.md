# D-finetune · 怀疑派核对

> 2026-10-31 · Workflow wf_113bb80a-4ec · 只读调研，未运行程序。

## 核对表

**第 1 节（RL 与不可微指标）**
- MetricGAN 表 1：L1 2.033、PG 2.098、MetricGAN(P) 2.133 → ✅。但有两处漏报：① PG 那一行标了星号「Pre-trained from another model」，**起点和 MetricGAN 不同**，严格说不算对照；② PG 的 STOI 是 0.743，**比 L1 监督的 0.753 还低**（MetricGAN(P) 是 0.760）。只优化 PESQ 会伤可懂度，这正是吃字的信号【论文 1905.04874 表 1】
- 表 2 的 2.86/3.99/3.18/3.42、2 层 BLSTM、掩码下限 0.05、加大 λ 出现「unpleasant artifacts」→ ✅【同上】
- MetricGAN+ PESQ 3.15 → ✅【摘要】。摘要没写结构，「非因果」仍是推算
- MetricGAN-OKD：ICML 2023（PMLR 202）→ ✅。因果版 PESQ 3.12 只有二手来源，而且**因果版有 0.82M 参数，是 GTCRN 的约 17 倍**，笔记漏了这一点【二手 PwC】
- CMGAN 许可 → ✅ 是 MIT【官方 GitHub】。CHiME-7「按 DNSMOS 训、听测最差」→ ⚠️ 转引自 007，2402.01413 的 PDF 没能解析，未复核
- GSEPF：❌ 无混响集只用 DPO 时 UTMOS 是 3.91→**4.21**，不是 4.17。OVRL 3.41→3.44、混响集 2.03→3.18、SECS 0.691→0.651、400 步 / batch 128 / 单张 A40 → ✅。「加上 CE 后回到 0.667」⚠️ 未核到【2507.09929 表 1、§4.1】
- FlowSE-GRPO：46.18M、LoRA、GRPO 5k 步 vs DPO 20k 步、OVRL 3.373→3.549 和 3.115→3.356、HiFi-GAN vocoder → ✅。补一条：只用 DNSMOS 当奖励时约 1.5k 步就收敛并开始讨好【§4.1.1】
- Berdoz：作者、Interspeech 2026、奖励 DNSMOS+WER+UTMOS、GSPO → ✅。❌「只用 DNSMOS 降到 SFT 以下」**摘要里没有**。摘要只说单指标会出现讨好，人评中复合奖励优于任一单指标

**第 2 节（训练代码与设备）**
- `gtcrn/loss.py:16-30`、`:24`、`:30` → ✅。`README.md:53` 指向 SEtrain → ✅
- `gtcrn.py:81`、`:163-164`、`:193`、`:197`，`stream/gtcrn_stream.py:146-147`、`:213` → ✅
- ❌「GRU 一共 5 处」：源码里只有 **3 行** `nn.GRU`，实例化后有 **14 个**（编码 + 解码共 6 个 TRA，加上 2 个 DPGRNN × 2 个 GRNN × 2）【`gtcrn.py:81,163-164,234-236,251-253,285-286`】
- LiSenNet：`model.py:130-141`（生成器）、`:145-166`（判别器回归 PESQ，`.cpu().numpy()`）、`config.yaml:48-51`、lr 5e-4、100 epoch、MIT → ✅。笔记漏了：batch 4、片段 2 s、`accelerator: gpu`（`config.yaml:2,11-12`）
- UL-UNAS `README.md:30-31` → ✅
- FastEnhancer：`losses.py:150-162`、`m.yaml:66,78-99` → ✅。补一条：每个 epoch 只有 16384 段（`m.yaml:41`），500 epoch ÷ batch 64 ≈ **12.8 万步**完整训练【推算】
- PyTorch #94691 → ✅ 仍 open。2025-01 加了「correctness (silent)」标签；LSTM 已修，GRU 没修【官方】

## 逻辑问题

1. **结论 2 自相矛盾**：说 RL 论文「都没报告 WER」，而 Berdoz 的奖励里本身就有 WER。
2. **「判别器 vs PG」的对照不成立**：起点不同，数据只有 TIMIT，模型是 BLSTM。能读出来的是：两条路都只比监督高约 0.07–0.1 PESQ，而且 PG 让 STOI 掉到监督以下。证据量撑不起「部分证实」，只能写「证据不足以区分」。
3. **把吃字代理交给判别器去学，路线绕了**：语音段能量比本身可微，直接写成损失就行。只有 ASR 删除率这种不可微的量才需要判别器或 PG。SCOREQ 也是神经网络、可微，可以冻结后直接当损失用，但讨好风险更高。
4. **步数估算借错了量级**：参照的是 46M 参数的 DiT 和自回归 LM，和 48K 的掩码模型不可比。更合适的锚点是 FastEnhancer 完整训练约 12.8 万步【推算】。
5. **「MPS 推翻」说过头了**：#94691 的复现是 Embedding+GRU，GTCRN 的小 GRU 会不会出 NaN 没验证过。应改成「有已知的静默错误，未证可行，需要冒烟」。另外 FastEnhancer 写死了 `autocast("cuda")`，LiSenNet 每步在 CPU 上算 PESQ，这两处在 Mac 上都是瓶颈。
6. **FiLM 方案 (a) 也不是完全连续**：换 γ/β 后 GRU 状态的分布同样会变，也有过渡。方案 (b) 交叉淡变时算力短时翻倍，在单核 4% 的预算下其实可以接受。笔记否定 (b) 的理由不足。

## 遗漏

- **D42 的场景缺口**：exp006 的设备效果只有倾斜、蓝牙窄带、削波，**没有模拟内置麦阵列波束成形或 AGC**，分场景专家没有对应的训练数据。
- **babble 的特殊性**：所列论文没有一篇针对咖啡馆 babble。用 WER 当奖励时，babble 被转写进来会变成插入错误，必须有目标说话人参考。笔记没有和 005（PSE）、011（多人说话）挂钩。
- **D26 许可**：只核了代码许可，没核**数据与评测器**的许可，包括 DNS3 各子集、DEMAND（CC BY-SA）、DNSMOS / UTMOS / SCOREQ 权重、PESQ（ITU P.862 有专利与授权条款）、torch_pesq。
- **最直接的证据没用上**：LiSenNet 本身就是几十 K 参数的 MetricGAN 式模型，它论文里的 PESQ 和因果性没有引用。这是对「48K 模型能否稳定做判别器训练」最直接的证据。
- 微调后其他场景会不会退化（灾难性遗忘），没有讨论。

## 修正后的结论

1. 对 48K 量级的模型，目前**没有证据**说明判别器比 PG 稳。MetricGAN 表 1 里两者都只比监督高约 0.1，而且只优化 PESQ 会伤 STOI。预判「稳得多」应改为「未证实」。
2. 现实组合修正为：**监督锚点（`gtcrn/loss.py:30`）+ VoiceFilter-Lite 式不对称损失**（公式已核，对 S_cln−S_enh>0 的部分乘 α>1；α 的取值未核）**+ 直接可微的语音段能量项**。判别器只留给 ASR 删除率、SCOREQ 这类不可微或不便直接优化的量，并另留一个裁判指标不参与训练。
3. 现成模板：LiSenNet 的 GAN 循环（MIT）和 FastEnhancer 的在线混合（MIT）。两者都默认 CUDA，在 Mac 上训练有 GRU 静默 NaN 的风险，要先冒烟。不行就走 MLX 移植、CPU 训练或云 GPU（云 GPU 待人拍板）。
4. 分场景专家 vs 加条件：没有同参数对照（2005.08128 摘要只说「更少参数」）。共用骨干加 FiLM 和交叉淡变两条路都可行，要用实验定。
5. 先补两块再谈训练：exp006 的内置麦与 babble 设备模型，以及数据和评测器的许可清单。

来源：
- [MetricGAN 1905.04874](https://arxiv.org/html/1905.04874v1)
- [MetricGAN+ 2104.03538](https://arxiv.org/abs/2104.03538)
- [MetricGAN-OKD（PwC）](https://paperswithcode.com/paper/metricgan-okd-multi-metric-optimization-of)
- [MetricGAN-OKD（PMLR）](https://proceedings.mlr.press/v202/shin23b.html)
- [CMGAN](https://github.com/ruizhecao96/CMGAN)
- [GSEPF 2507.09929](https://arxiv.org/html/2507.09929v2)
- [FlowSE-GRPO 2601.16483](https://arxiv.org/html/2601.16483v1)
- [Berdoz 2606.21458](https://arxiv.org/abs/2606.21458)
- [专家混合 SE 2005.08128](https://arxiv.org/abs/2005.08128)
- [VoiceFilter-Lite 2009.04323](https://arxiv.org/pdf/2009.04323)
- [PyTorch issue #94691](https://github.com/pytorch/pytorch/issues/94691)