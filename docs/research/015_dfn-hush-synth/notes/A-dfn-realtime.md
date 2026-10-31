# A · DeepFilterNet 一系的实时版：延迟 · 许可 · 自然度旋钮 · 两段式证据

> 2026-10-31 · 015 线手下 A · 只读调研，**本文没有跑任何推理**；凡「每帧耗时」都是别人的数或推算，已标等级。
> 本机副本：`resources/` 在主检出 `noise_canceling/resources/`（worktree 里只有 README）。模型包解到 scratchpad 读 `config.ini`、用 `onnx` 数参数。

## 结论（5 行）

1. **延迟都够**：DFN3 `_ll`、DFN2 `_ll`、Hush 三个都是**零前瞻**（`df_lookahead = conv_lookahead = 0`），按 002 口径 ① = 20 ms，③ 端到端约 26–30 ms，比 GTCRN（38–42 ms）还低；DFN3 默认版 ③ 约 46–50 ms，**超 D32**。
2. **但 `_ll` 不是「小号 DFN3」**：参数约 **9.7 M**（DFN3 默认 2.1 M 的 4.6 倍），GRU 宽度 256→512、DF 解码器 2→3 层；每帧耗时**没人测过**（我也没测）。
3. **许可**：DFN 权重**仍未澄清**——2026-07 至 10 月 5 个 issue 问，**维护者 0 回复**，上游最后一次 push 是 2024-10-17。Hush 权重 HF 卡与仓库都写 Apache-2.0（可并入 GPL-3.0）；训练数据里有 ESC-50（CC BY-NC）等，属灰区但不改变权重的标示许可。
4. **「−10 dB 整帧静音」只在 `RuntimeParams::default()` 里**；C API、命令行、LADSPA、Hush 全都改成 −15/35/35，等于关掉。`atten_lim_db` 就是「对齐后的原声按比例混回」——和我们的「背景保留」是同一件事，只是做在频域、全程固定。
5. **建议**：Hush 进下一轮盲听当第三候选；DFN3_ll 只当「听感参照」，许可不清不进产品。两者共用一个 libDF 式适配器，增量成本约等于一个。

---

## 1 · 实时版一览

延迟口径同 002：① 窗 + 前瞻；③ = 管线直通 6–10 ms + ①（`docs/research/002_model-baseline/decision.md` 延迟一节）。

| 变体 | 采样率 · 窗/hop | 前瞻（源码） | ① | ③ 端到端附加 | 参数（数 ONNX）| 状态是否显式 | ≤45 ms |
|---|---|---|---|---|---|---|---|
| DFN3 默认 | 48 k · 20/10 ms | conv 2 · df 2，取较大值 → 2 帧【源码】`DeepFilterNet3_onnx/config.ini:49,52`；`libDF/src/tract.rs:309-316` | 40 ms | 46–50 ms | 2.13 M【推算】数 initializer + Constant；与论文 2.13 M 一致 | 否：三张子图，时间维符号 `S`，靠 tract-pulse 转逐帧 | ❌ 略超 |
| **DFN3 `_ll`** | 48 k · 20/10 ms | **0 · 0**【源码】`DeepFilterNet3_ll_onnx/config.ini:51,54` | **20 ms** | **26–30 ms** | **9.74 M**（enc 1.73 / erb_dec 3.20 / df_dec 4.81）| 否（同上）| ✅ |
| DFN2 `_ll` | 48 k · 20/10 ms | 0 · 0【源码】`DeepFilterNet2_onnx_ll/config.ini:45,48` | 20 ms | 26–30 ms | 2.30 M | 否 | ✅，但现行 libDF 拒载 DFN2（`tract.rs:310-312` 直接 bail，要回退 v0.3.1）|
| **Hush** | **16 k** · 20/10 ms（320/160）| 0 · 0【源码】`Hush/deployment/models/*.tar.gz` 内 `config.ini`；`Hush/configs/default.ini:12,54` | **20 ms** | **26–30 ms** | **2.28 M**（README 写 ~1.8 M，对不上）；`conv_ch=16`、DF 只到 64 bin ≈ 3.2 kHz | 否（同 DFN3 三图格式）| ✅ |
| DPDFNet-2 | 16 k · 20/10 ms | 两段串联 4 帧（002 已核）| 60 ms | 66–70 ms | 2.49 M | **是**（单帧图 + state 向量）| ❌ 已出局 |
| GTCRN（对照）| 16 k · 32/16 ms | 0 | 32 ms | 38–42 ms | 48 K | 是 | ✅ |

`_ll` 相对默认版改了什么（两份 `config.ini` 逐行 diff）【源码】：前瞻 2→0；`emb_hidden_dim` 256→512、`df_hidden_dim` 256→512、`df_num_layers` 2→3；`conv_kernel` 1,3→**2,3**（时间核变 2，只看过去一帧，用来补回被拿掉的前瞻）；`convt_depthwise` 改 True；训练侧还加了 `p_interfer_sp = 0.01`（少量竞争说话人）、`p_clipping/p_zeroing = 0.05`。**换句话说，作者用「加宽网络」换「零前瞻」，_ll 比默认版重得多**——issue #628 的提问者也观察到「_ll 更大、看起来更慢」【二手】。

### 1.1 每帧算力（全部未实测）

| 变体 | 官方/别人的数 | 我的推算 |
|---|---|---|
| DFN3 默认 | 0.344 GMACs/s 量级【论文】arXiv 2305.08227；论文写 tract 单线程 RTF 0.19（i5-8250U）| — |
| DFN3 `_ll` | **无** | 参数 4.6×，MAC 大约 1.1–1.6 GMACs/s【推算】；与 DPDFNet-2（1.35 GMACs，本机 ort 单线程 max 0.89 ms，exp001）同档，**ort 下大概率 ≤ 2 ms/帧**，tract 下未知 |
| Hush | 「< 1 ms / 10 ms 帧」【官方】`Hush/README.md:36`；issue #9 维护者表：1 会话平均 0.586 ms/帧（机型未写）【二手】| — |
| GTCRN | 33 MMACs/s【官方】；本机 max 0.62 ms/16 ms 帧（exp001）| — |

### 1.2 运行时与社区实现

| 实现 | 是什么 | 延迟 / 实时性做法 | 能直接搬吗 |
|---|---|---|---|
| libDF `DfTract`（tract）| 官方 Rust 逐帧运行时：STFT、ERB 特征、归一化、DF 全在 Rust，三张子图交 tract | 前瞻按 `max(conv, df)` 对齐（`tract.rs:309-316`）；每帧有若干 `clone()`（`:461-462、541-542、637`），未复核分配量 | 代码 MIT/Apache，可借；tract 锁 0.21.4 |
| LADSPA 插件 | PipeWire 虚拟麦用 | **默认编进 `_ll`**（`ladspa/Cargo.toml:14`）；推理在独立线程，用 `Mutex<VecDeque>` 交接，**额外垫 1 hop 输出延迟**（`ladspa/src/lib.rs:213-220`），③ 再 +10 ms → 36–40 ms；工作线程按 hop/5 睡眠轮询（`:221`）| 只借思路；Mutex 交接不符合我们的实时规约 |
| C API `df_create` | 给 C/C++/JUCE 用 | 2024-09 的修复把阈值改成 −15/35/35（`libDF/src/capi.rs:27-31`）| 同 libDF |
| Shuichi346/DeepFilterNet3-VST3 | **macOS Apple Silicon** 的 VST3/CLAP（Rust，nih-plug 系），MIT，2026-10-12 仍在推 | 默认 `_ll`；自报「模型固有延迟 480 样本（10 ms，即窗−hop）」，48 kHz、最大块 128 时上报 30 ms【二手】README；无锁回调 + 迟到帧回退对齐原声 | 最接近我们形态的参照；未克隆 |
| svaningelgem/RoomMute | Windows 纯 Rust 虚拟麦，DFN3 + tract | 发行的是官方 DFN3 权重重导出（issue #700 本人承认）；自训两次「把近讲声也压过头」【二手】| 不能当「干净权重」来源 |
| kylehowells/DeepFilterNet-mlx、HF `*-CoreML` | Swift + MLX / 有状态 Core ML 逐 hop | Core ML 稳态 0.264 ms/hop（未按节拍）【二手】；006 已记「按真实节拍 p99 9 ms、漏拍」 | 006 已判「只看看」 |
| Hush `libweya_nc` | Hush 自带 C ABI 库，**内含 DFN `df` v0.5.3 裁剪副本**（`Hush/native/weya_nc_build/README.md`）| 多会话共享一份编译模型（PR #10）；阈值写死 −15/35/35（`weya_nc/src/lib.rs:32-41`）| 可从源码自建 arm64；别用预编译二进制 |
| grazder/torchDF | 单图、状态显式的流式 ONNX | 但 hop 改 512 且重新微调，许可 NOASSERTION（002 已记）| 不适合 |

**上游与 fork 活跃度**【本机 gh，2026-10-31】：Rikorose/DeepFilterNet 最后 push 2024-10-17、最后提交 2024-09-25、最后 release v0.5.6（2023-08-31），67 个 open issue；542 个 fork 里近半年有推送的多是 0 星个人 fork，最高 ismailivanov/DeepFilterNetPlus（17★）。**没有接棒的活跃 fork**；活跃的是下游应用（上表）和 Hush。

> 坑（给适配器的）：issue #713（2026-10-28）——libDF 喂纯数字静音约 78 s 后输出 NaN：静音保护 `skip_counter` 每遇 `gains = Some` 就清零，实际从不触发（`tract.rs:516-525` 对 `:579`），归一化状态按 0.99 指数衰减，开了 flush-to-zero 后变 0，`0/sqrt(0)`【二手，未复现】。**我们开 FTZ（no_denormals），必须在适配器里给归一化状态设下限或静音旁路。**

---

## 2 · 权重许可现状

| 对象 | 标示 | 证据 | 判断 |
|---|---|---|---|
| DFN 代码 | MIT 或 Apache-2.0 | 【官方】`DeepFilterNet/README.md:323-330`「All code in this repository…」| 清楚 |
| DFN2/DFN3 权重（含 `_ll`）| **未写明** | README 只说 "All code"；`models/` 里无单独许可；DFN3 论文写「published under a permissive license」并给仓库链接【论文】arXiv 2305.08227 §3 脚注 2——指仓库，没区分权重；issue #697、#700、#709、#712（2026-07-15 至 10-18，含有人邮件两次）**维护者无一回复**【本机 gh】；GitHub 识别为 NOASSERTION | **仍不明**。宽松解读有依据（论文那句 + 权重放在双许可仓库里），但没有书面确认 |
| 第三方转发 | Intel/deepfilternet-openvino 标 MIT；soniqo、iky1e 标 MIT；aufklarer、ferdinandl007 标 Apache-2.0【本机 HF API】| 转发者自标，不是原作者授权 | 不能当依据（#700 提问者也这么说）|
| Hush 权重 | **Apache-2.0** | HF 卡 `license: apache-2.0`（`weya-ai/hush`，2026-03-31 更新）；仓库 `LICENSE` 为 Apache-2.0 全文（缺附录署名行，故 GitHub 显示 NOASSERTION）【本机】| **标示属实**；Apache-2.0 → GPL-3.0 单向兼容 |
| Hush 是否继承 DFN3 权重 | 否（推断）| 结构与 DFN3 发布版不同（16 k、`conv_ch=16`、`nb_df=64`），`train.py:1117-1119` 的 `--init_from` 是 `strict` 加载，DFN3 48 k 权重形状对不上；版本串 `full_finetune_fastsafe_100ep_20260220`（内部起点不明）【推算】| DFN3 权重的不明许可**大概率不传染** Hush |
| Hush 训练数据 | 混合 | `Hush/DATASETS.md`：LibriSpeech/VCTK CC BY 4.0、Common Voice CC0、**ESC-50 CC BY-NC 3.0**、DNS（自写「Microsoft Research License」）、AudioSet（YouTube 来源）| NC 数据训出的权重是否受 NC 约束，**法律上未定**；业界常见做法是把权重许可与数据许可分开看。对 GPL 开源发布风险低–中；若将来有商业分发需人定 |

---

## 3 · 阈值分支与「自然度 / 舒适底噪」旋钮

### 3.1 参数逐个说清

DFN 编码器每帧输出一个局部 SNR 估计 lsnr，经 sigmoid 映射到 [−15, 35] dB（`DeepFilterNet/df/deepfilternet3.py:162-164,184`）。libDF 用它决定这一帧跑哪几段（`tract.rs:658-673`）：

| 参数 | `RuntimeParams::default()`（`tract.rs:176-186`）| C API / CLI / LADSPA / Hush | 含义 | 是不是自然度旋钮 |
|---|---|---|---|---|
| `min_db_thresh` | **−10** | **−15**（`capi.rs:28`；`bin/enhance_wav.rs:43`；`ladspa/src/lib.rs:36-37`；`weya_nc/src/lib.rs:37`）| lsnr 低于它：**ERB 增益全置 0（`tract.rs:491-492`），DF 不跑 → 整帧静音** | 否，是省算力的门。设 −15（lsnr 下限）即永不触发 = **可通过参数关掉** |
| `max_db_erb_thresh` | 30 | 35 | lsnr 高于它：两段都不跑，原样输出（延迟对齐）| 否。设 35 关掉 |
| `max_db_df_thresh` | 20 | 35 | lsnr 高于它：只跑 ERB，跳过 DF | 否。设 35 让 DF 永远跑 |
| `atten_lim_db` | 100（=不限）| 调用方给 | `lim = 10^(−dB/20)`（`:288-297`）；**输出 = (1−lim)·增强 + lim·带噪**，带噪取的是和增强帧对齐的那一帧（`:599-610、625-629`）；可运行时改（`:387-397`）；<0.01 dB 直接直通 | **是**：最大压制深度 = 背景留多少 |
| `post_filter_beta` | 关（0.02 备用）| 0（LADSPA 范围 0–0.05）| 对增益再做一次非线性压低（`libDF/src/lib.rs:446-`），**方向是更狠** | 反向旋钮；DFN2 论文里加它 PESQ 3.08→3.03、CSIG 4.30→3.72【论文】arXiv 2205.05474 Table 1 → 建议关 |

另外两处硬切换：lsnr 落在不同档时整帧在「全压 / 只 ERB / 直通」间跳，**没有平滑**；论文就是这样设计的（−10 / 20 dB 两道门，arXiv 2305.08227 §3）。在 −15/35/35 下这些跳变全消失，只剩一条路径。

**结论**：「−10 dB 以下整帧静音」是真的，但只出现在直接用 `RuntimeParams::default()` 的 Rust 调用里；**002 把它写成「默认阈值」需要加一句「官方 C API / CLI / LADSPA 已改为不触发」**。它不是自然度设计，而是吃字风险（安静人声 + 大噪声时 lsnr 估计偏低就整帧清零）。

### 3.2 `atten_lim` 和我们的「背景保留」

| | libDF `atten_lim` | 我们规划的「背景保留」（`docs/plan/003_tech-design.md:48`；011 decision:26 默认 0.3）|
|---|---|---|
| 做法 | 原声按延迟对齐、按固定比例混回 | 同 |
| 在哪做 | 频域、iSTFT 之前，模型内部的对齐帧 | 管线里、门控之后（时域，按模型延迟对齐）|
| 和模型的关系 | 绑死 DFN 运行时 | 与模型无关，GTCRN / FE-M / Hush 共用 |
| 随内容变吗 | 不变（全程一个 lim）| 可按场景 / 门控状态变（013 参数搜索）|
| 底噪是什么 | 真实背景的 −X dB 副本，不是合成舒适噪声 | 同 |
| 副作用 | 增强语音也乘 (1−lim)，但带噪里的语音同相补回，净效应≈不变 | 同 |

→ **它就是同一个旋钮，不是新东西**；所以文章说的「留舒适底噪」我们已经有了。真正差别在「放在哪」：我们放在模型外，能换模型、能随场景调——「因此不必搬 libDF 的实现」是假设，尚无 verifier。有一点可借：libDF 在 `atten_lim` 生效时，原来的「整帧静音」也会变成「−X dB 原声」，说明**背景保留同时就是吃字兜底**——这条应写进我们背景保留的验收项。

---

## 4 · 两段式 deep filter 与「不机器人味」的证据

| 证据 | 内容 | 等级 | 能证明什么 |
|---|---|---|---|
| DFN1 Table 1（VCTK/DEMAND）| 完整：1.8 M、0.35 GMACs、PESQ **2.81**、SI-SDR **16.63**；去掉第二段：0.9 M、0.25 GMACs、PESQ **2.57**、SI-SDR **13.81** | 【论文】arXiv 2110.05588 | DF 段 +0.24 PESQ、+2.8 dB SI-SDR，代价 +0.9 M 参数、+0.10 GMACs |
| DFN1 Fig. 3 | DF 与普通复数比值掩码（CRM，= 阶数 1 的 DF）在 5–30 ms 窗下逐一比：DF 全面更好；**CRM 在窗 ≤ 20 ms 时掉得厉害**（频率分辨率不够分开谐波），窗到 30 ms 两者差距缩小 | 【论文】同上 §3 | DF 的收益主要来自「短窗下补频率分辨率」 |
| DFN1 Fig. 4 | 语谱图示例：DF 复原了带噪谱里看不见的谐波 | 【论文】定性 | 只是一张图 |
| DFN3 设计说明 | DF 只做到约 5 kHz（浊音能量主要在下面）| 【论文】arXiv 2305.08227 §3 | — |
| 「不机器人味」的主观证据 | **没有找到**：DFN1–3 都只报 PESQ/CSIG/SI-SDR，无 MOS/MUSHRA 自然度消融 | — | 文章那句是合理推论，**不是测出来的** |

**对 GTCRN 的含义**：GTCRN 用的是 CRM（`gtcrn/gtcrn.py:265-272`），但窗是 32 ms（16 k 下 31.25 Hz 分辨率，DFN 是 50 Hz）。按 DFN1 Fig. 3，**窗越长 CRM 越接近 DF**——GTCRN 多花的那 12 ms 延迟某种程度上就是在买 DF 想补的那份分辨率【推算】。所以「GTCRN 缺第二段」的损失，应小于 DFN1 那 +0.24 PESQ。

**便宜的补法**（按 D45「能不训练先不训练」排序）：

| 办法 | 成本 | 延迟 | 风险 | 判断 |
|---|---|---|---|---|
| ① 按说话人音高的梳状 / 基音滤波（D45 已列）| 零模型；参照 RNNoise 的 pitch filter：把基音延迟后的信号按每带强度加回再归一能量（`nnnoiseless/src/features.rs:220-258`，强度由增益与基音相关度算）| 0（只用过去样本）| 基音估错时自己会制造嗡嗡的「机器味」→ 必须按浊音置信度门控、只在低 SNR 开 | **先做**，和 D47 的「说话人音高」共用一个音高跟踪 |
| ② 只借 DF 第二段：在 GTCRN 输出上加一个小 DF 头（阶数 3–5、只覆盖 ≤ 4 kHz、只看过去帧）| 要训练（系数无法手推）；DFN1 的 DF 段 0.9 M 参数，比 GTCRN 本体大 20 倍，需压到几万参数【推算】| 0（`_ll` 已证明 DF 可零前瞻）| 训练线，归 013/D45 慢线 | 后备；①不够再做 |
| ③ 直接换 Hush / DFN3_ll 当主模型 | 见第 5 节 | 更低（20 ms）| 许可、算力 | 盲听决定 |

---

## 5 · 建议：要不要进下一轮盲听

**Hush：进。DFN3_ll：作参照进，不作产品候选。**

| | Hush | DFN3_ll |
|---|---|---|
| 延迟 | ③ 26–30 ms ✅ | ③ 26–30 ms ✅ |
| 采样率 | 16 k，和 GTCRN 同管线 | 48 k，Mac 设备原生率，**省掉两次重采样** |
| 许可 | Apache-2.0 ✅（数据 NC 灰区）| 不明 ❌ |
| 独特价值 | 训练时 60% 样本带竞争说话人（SIR 12–24 dB），正对 011/D47 | DFN 正统音质参照；验证「两段式是否真更自然」 |
| 算力 | 自报 < 1 ms/帧 | 未知，4.6 倍参数 |
| 接入成本 | 三图 + Rust 侧 ERB/DF：需写一个 libDF 式适配器（可借 Hush 自带的 MIT/Apache 运行时源码）| 同一个适配器，换配置即可 |

**前提条件**（进盲听前要满足）：
1. 适配器统一用 −15/35/35 阈值（关掉整帧静音与硬切换）、`atten_lim` 关（=100），背景保留交给管线外的统一旋钮，和 GTCRN/FE-M 同一档位比。
2. 响度对齐、同素材（含 011 的竞争说话人片段）；Hush 是 ASR 取向，要专门看它会不会把「对方想听到的旁人插话」也压掉。
3. 先离线（Python `libdf` 或 Rust tract 均可）出盲听样本，**同时实测每帧耗时**（M 系列单核，补上 1.1 节的空格）；DFN3_ll 超 2 ms/帧就只留作参照。
4. 适配器处理 issue #713 的 FTZ + 长静音 NaN。
5. DFN3_ll 的样本只内部听，**不进任何发布物**，直到作者书面确认许可。

---

## 要人拍板的

| # | 题 | 选项 · 代价 | 建议 | 默认（不拍板时）|
|---|---|---|---|---|
| 1 | Hush 进下一轮盲听吗 | 进：写一个 libDF 式适配器（约 1–2 天，未估准）· 不进：竞争说话人只靠 D47 辅助模型 | **进** | 进 |
| 2 | DFN3_ll 怎么用 | 当听感参照（只内部听）· 当产品候选（需先等许可确认，可能永远等不到）· 不用 | **参照** | 参照 |
| 3 | 训练数据含 NC 的 Hush 权重能否随 GPL 发布 | 接受（标示 Apache-2.0，风险低–中）· 不接受（只拿 Hush 训练配方自训）| 开源发布接受；商业分发另议 | 接受，在 NOTICE 里写明数据来源 |
| 4 | GTCRN 补谐波先走哪条 | ① 音高梳状滤波（零训练）· ② 小 DF 头（训练）| **①先，②后备** | ① |

## 来源

- 【源码】`resources/DeepFilterNet/libDF/src/tract.rs:70-92、110-186、288-316、387-420、436-505、509-641、658-673`；`libDF/src/capi.rs:22-31、80-104`；`libDF/src/bin/enhance_wav.rs:36-52`；`libDF/src/lib.rs:446-`；`ladspa/src/lib.rs:30-47、172-235、268-331`；`ladspa/Cargo.toml:13-14`；`libDF/Cargo.toml:81-82`；`DeepFilterNet/df/deepfilternet3.py:162-185`；`README.md:323-332`
- 【源码】模型包 `resources/DeepFilterNet/models/{DeepFilterNet3_onnx,DeepFilterNet3_ll_onnx,DeepFilterNet2_onnx_ll}.tar.gz` 内 `tmp/export/config.ini`（行号见第 1 节）；参数数自三张子图的 initializer + Constant【推算】
- 【源码】`resources/Hush/README.md:30-41、90-106、320-360`；`DATASETS.md`；`configs/default.ini`；`deployment/models/advanced_dfnet16k_model_best_onnx.tar.gz`（config.ini、version.txt）；`native/weya_nc_build/README.md`；`native/weya_nc_build/weya_nc/src/lib.rs:25-41`；`training/train.py:1115-1121`；`LICENSE`
- 【源码】`resources/gtcrn/gtcrn.py:265-272`、`gtcrn/README.md:19`；`resources/nnnoiseless/src/features.rs:220-258`
- 【论文】Schröter 等，DeepFilterNet，arXiv 2110.05588（Table 1、Fig. 3–4）；DeepFilterNet2，arXiv 2205.05474（Table 1）；DeepFilterNet3 demo，arXiv 2305.08227（§3、Table 1）
- 【官方/本机 gh】github.com/Rikorose/DeepFilterNet issues #628、#697、#700、#709、#712、#713；repo 元数据（pushed_at 2024-10-17）；github.com/pulp-vision/Hush issues #2、#9、PR #10；huggingface.co/weya-ai/hush（HF API cardData）
- 【二手】HF API：Intel/deepfilternet-openvino、soniqo/DeepFilterNet3-ONNX、iky1e/DeepFilterNet3-MLX、aufklarer/DeepFilterNet3-CoreML、ferdinandl007/DeepFilterNet3-Streaming-CoreML 的自标许可；Shuichi346/DeepFilterNet3-VST3、svaningelgem/roommute、kylehowells/DeepFilterNet-mlx 的 README（未克隆）
- 本项目：`docs/research/002_model-baseline/decision.md`、`sources.md:26、52-59、92`；`docs/research/006_sota-landscape/gap.md:27`；`docs/plan/001_decisions-20261029-b1.md` D32/D33/D45/D47；`docs/plan/003_tech-design.md:48`
