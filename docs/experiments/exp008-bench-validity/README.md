# exp008 · benchmark 格子的效度检查（MVP）

> 2026-10-31 · 手下 benchsynth（016 线，issue #41）。设计见 `docs/research/016_bench-synth/{benchmark,synth-standard,mvp}.md`。
> 证据等级：【测】本机实测 ·【推】推断。数据侧检查（S1–S3、S6）不用名额；排名效度（S7 / S8）用系统阶梯，第一轮结果见下文末节与 `016/mvp.md` 2.2。

## 跑了什么

| 文件 | 做什么 |
|---|---|
| `cells.py` | 生成 4 条单类轴（S-SNR 5 档 · S-NOISE 6 种 · S-RT60 4 档 · S-SIR 6 档）+ 2 个多类格（M-SNRxNOISE · M-SIRxSNR），两套独立生成器 A / B，每格每档 6 段 × 8 s；**配对设计**（同轴各档共用素材，只改一个因素）；三轨分存 + 逐 hop 四类标签 + 实测 T60 / DRR |
| `checks.py` | S1 可复现 · S2 可控 · S3 真值完整 · S6 噪声素材统计对齐（对本机真实录音） |
| `builds/<build>/` | 清单 `manifest.json`（参数、脚本哈希）+ 真值 `truth.jsonl`（每段条件、素材出处、逐文件样本哈希、逐帧标签）。音频在主目录 `_tmp/exp008/<build>/`（仓外，约 685 MB float32） |
| `results/<build>-checks.json` | 检查结果 |

复现：`PY=<主目录>/_tmp/exp002-venv/bin/python`；`$PY …/cells.py --build <id> --n 6` → `$PY …/checks.py <id> --n-align 32`（合计约 1 分钟，纯 numpy）。

## 结果（build `20261031-102406-0c67`，348 段）【测】

| 检查 | 结果 |
|---|---|
| S1 可复现 | 重生成 S-SNR 全部 60 段，**样本哈希 60/60 相同** |
| S2 可控 | 4 条轴 × 12 组（A / B 各 6）：未被扫的轨逐样本不变 12/12；实测因素随名义值单调 12/12；实测 SNR / SIR 与名义值最大偏差 0.0 dB |
| S3 真值完整 | 三轨相加对 mix 残差最差 −148.8 dB；标签长度全对 |
| S6 统计对齐 | 见下表 |

**S6 噪声素材对齐**（统计量：1/3 倍频程长时谱形状 · 四带包络调制谱 · 20 ms 电平分布；线 = 真实留出样本到真实类中心距离的 95 分位；每格 32 段 × 4 s）

| 名义类型 | 真实参照（本机） | 生成器 A 素材 → 判 | 生成器 B 素材 → 判 | 读法 |
|---|---|---|---|---|
| cafe | DEMAND PCAFETER + SCAFE | DEMAND PCAFETER（真实）→ 过 | DEMAND SCAFE（真实）→ 过 | **阳性对照**：真实素材本来就在参照池里，必须过 |
| traffic | STRAFFIC + TBUS | STRAFFIC（真实）→ 过 | TBUS（真实）→ 过 | 阳性对照 |
| clatter | DKITCHEN | DKITCHEN（真实）→ 过 | **exp006 `synth_dishes`（合成）→ 不过**：长时谱距离中位 10.4 dB（线 5.3）、电平分布 11.6（线 1.2），过线 0% | 合成杯碟的谱形和起伏都不像真实厨房 |
| music | 真实钢琴录音（只有 1 首，Maple Leaf Rag） | **exp006 `synth_music`（合成）→ 不过**：长时谱 9.0 dB（线 2.5）、调制谱 6.2（线 4.0），过线 0% | 钢琴录音（真实）→ 过 | 只有一首曲子当参照，判据很窄 |
| babble | PCAFETER + SCAFE + PRESTO | 合成 6 人（LibriSpeech + exp006 RIR）→ **临界**：长时谱过线 69% | 合成 6 人（另一组说话人 + Polack RIR）→ 过（最差一项 88%） | 合成 babble 接近真实食堂，但长时谱偏差在边上 |
| stationary | **本机没有**真实空调 / 风扇录音 | `synth_hvac` → D0 | 有色噪声 → D0 | 没有参照就没有对齐可言 |

**这一步说明什么**（只说数）：exp006 训练 / 评测都在用的三种合成噪声里，杯碟和音乐的统计量离真实录音很远，babble 在边上；稳态类连判的参照都没有。**它不说明这些格子上的降噪排名错了**——那要 S7 / S8（排名效度）才能判。

## 已知局限（如实）

1. **阳性对照两边都是循环的**：A 的 cafe / clatter / traffic 用 DEMAND 前 60% 时段，参照中心也由前 60% 算；B 的 cafe（SCAFE）/ traffic（TBUS）就是定阈值的留出池本身；B 的 music 就是那唯一一首参照曲子——它们过线只证明检查本身没坏。另外「参照均值谱 + 随机相位 + 慢调幅」的纯合成噪声能过 traffic / clatter（红队实测），三组统计量约束力不够
6. **隔离（S10）不过 → 使用限制**：A 的 DEMAND 段与合成函数、B 的说话人都与 exp006 训练集重叠。**本 build 的结果只对没见过 exp006 训练素材的系统成立**（今天阶梯全是第三方预训练模型，全部适用）；**exp006 场景分类器、以及任何按 exp006 / `sim.py` 配方训练的项目模型，不能在本 build 上评**。下一 build 按 D63 换池（`016/synth-standard.md` S10 处理段）。本 build 的 `truth.jsonl` 没记说话人 id 与噪声起点，重叠只能按 `cells.py` 代码逐类核
7. 「无人声段压噪」的标签取自干轨，高 SNR / 强混响格里混着主讲人混响尾；`rank.py` 已加带护栏版 `noise_att_g_db`
2. **线压在噪声底上**：真实录音自己互比，过线率也只有 88–100%；所以设了「临界」档（0.5–0.8）不判。每类只有 1–3 个真实环境，类中心不稳；要 014 的真实集（WHAM! 噪声、ACE、CHiME-4 CAF）才能把参照做厚
3. **只对齐了噪声**：语音素材（LibriSpeech 朗读 vs 会话体）、RIR（T60 / DRR 分布）、设备链、重叠率都还没有真实参照可比——本机没有任何现场录音；按 D62 开发阶段不录，语音 / RIR 参照等 D65 下载公开集（016 decision 第二节）
4. **S6 的三组统计量是自选的**〔建议值〕：没有证据说这三组距离小就等于「对降噪排名的影响小」；这正是 S7 / S8 要回答的
5. 设备链 `laptop`（100 Hz 高通 + 7 kHz 低通）是假设；回放重录按 D62 开发阶段不做，发版前试用时再核

## 排名效度（第一轮：12 系统 + 理想掩码，348 段，6287 s，0 错误）【测】

| 文件 | 做什么 |
|---|---|
| `rank.py` | 系统阶梯 × 每段 → Python 近似指标（SI-SDR · ESTOI · DNSMOS · 压噪 · 主讲人电平 · 邻座段压制）；第二轮系统读 `enh/` 先写好的增强音频（`--tag r2`） |
| `enh/` | 第二轮系统的增强脚本与约定（`enh/README.md`） |
| `rankstat.py` | S7 生成器 A / B 的 Kendall τ_b + bootstrap 区间 + split-half 信度；S8 对 exp003 人耳配对 |
| `results/<build>-scores*.jsonl` · `-rankstat.json` | 分数与统计（只有数） |

- **S7a 单格名次**：OVRL τ_AB 中位 0.85，但 A、B **错配格**也是 0.85 → 单格名次无区分力（宽阶梯在哪都这么排），不单独判过
- **S7b 交互一致性**（主判据，每轴系统 × 档位残差 r、精确置换）：S-SNR / S-NOISE / S-SIR 上 r 0.5–0.9、p ≤ 0.025；S-RT60 上 r −0.08–0.45、p ≥ 0.13 → 混响格依赖 RIR 生成器
- **S8**（D4-add）：合成格预测 exp003 人耳配对 0.25–0.38，屏聚类区间含 0.5；片段自身 0.48–0.56 → 判不了方向
- **S2 重检**（`results/*-checks-s2fix.json`）：room / noise 轴原先是空检，补后仍全过
- **第二轮（2026-11-02）**：RNNoise · WebRTC NS 18 / 6 dB · MossFormer2-48k 读文件打分（`results/*-scores-r2.jsonl`，0 错误），合并统计 `results/*-rankstat-r2.json`（`rankstat.py <build> r2`，不覆盖第一轮）。去天花板与变体 15 个系统：S7b 结论不变；DNSMOS SIG 在 S-SIR 轴 r 0.04 不复现。FRCRN 等 4 个 torch 系统中断待续跑；Hush ×2 + DFN3 未跑（权限缺口）
- 读法与局限见 `docs/research/016_bench-synth/mvp.md` 2.2 / 2.3
