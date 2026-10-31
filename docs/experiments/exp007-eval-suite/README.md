# exp007 · 评测体系：verifier 写成代码 + 任务集 + 定标

> 2026-10-31 · evalimpl（批次 20261031-b4）· 分支 `feat/eval-verifiers`
> 人原话：「那rust 或者指标的验证开始写了吗」「以及好的完整的eval 以及之类的东西」「具体eval 的设计等等其实可以看anthropics 那篇讲 eval 设计的那个」「就是做好eval 以及所有的varifier」。相关决策：D44（主观项拆成带置信度的 verifier）· D46（验收只看 verifier，CLAUDE.md 第五节第 3 条）· D48（虚拟听众）。
> 依据：`docs/research/012_eval-standards/verifiers.md`（清单、L0–L3、§3.4 模拟真值）· `decision.md` · `specs/rubric/denoise.md` 第 2、7 节 · Anthropic《Demystifying evals for AI agents》（2026-01，https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents）。

## 结论先说

RESULTS_PLACEHOLDER

## 怎么跑

```bash
cargo run -p xtask -- eval ci                 # CI 层：纯 DSP、无模型、几分钟；门禁不过退出码 1（已挂进 .github/workflows/ci.yml）
cargo run -p xtask -- eval ci --audio         # 同上，另写抽查音频到 _tmp/exp007/<run>/audio/（仓外）
cargo run -p xtask -- eval qa                 # QA 层：本机 Qwen3-ASR + DNSMOS + 真实片段（要名额）
cargo run -p vdn-eval -- calib --record       # 用 exp003 答卷定标，结果进 calib/
cargo run -p vdn-eval -- list                 # 列任务与 verifier
```

报告：`_tmp/exp007/<run-id>-<层>/report.{json,md}`；加 `--record` 复制进本目录 `results/`（进仓）。

## 设计

### 三层（D46 / CLAUDE.md 第五节第 3 条）

| 层 | 谁判 | 跑在哪 | 挡什么 | 本实验交付 |
|---|---|---|---|---|
| CI 层 | DSP verifier，确定性 | 每个 PR（`ci.yml` 的 `eval ci` 一步） | 合并 | 16 条 verifier · 30 个任务 |
| QA 层 | 模型 verifier（ASR、DNSMOS）+ 真实片段 | 本机 | 发版 | 2 条 verifier · 11 个真实片段 |
| 人工层 | 作者盲听 | 听测页 | 定标 | 定标脚本 `vdn-eval calib`（读答卷 → 一致率 / τ → 级别） |

### 被测系统

一律是真 `vdn_engine::Engine`（三件保命）包一个模型，逐 hop 调 `process`——和实时同一份代码（`.42cog/cog.md`「eval 和实时用同一份处理代码」）。频谱分析用 `vdn_dsp::StreamingStft`。

- `ref`：crate 内的参考降噪器（判决引导维纳 + 最小值跟踪噪声估计，STFT 512/256，与 GTCRN 同 hop、同延迟）。**它不是产品模型**——CI 没有模型权重，用它让 Engine 的保命逻辑、门控、延迟账在真信号上被检验
- `gtcrn` / `fastenhancer-m`：真模型，找得到权重才跑（`ModelKind::default_path`），找不到就跳过并在报告里写明

### 任务集（`src/eval/suite/`）

| 集合 | 个数 | 作用 | 门禁 |
|---|---|---|---|
| 回归 `regression` | CI 16 · QA 2 | 已经做好的不许退步；应接近 100% 通过 | L3 不过阻断；L0 感知量只报警但**不许比冻结基线差** |
| 能力 `capability` | CI 9 · QA 9 | 攻坚（邻座、人群、−5 dB、激进压制）；起点通过率低，给爬坡空间 | 从不阻断 |
| 探针 `probe` | CI 5 | verifier 自测：劣化版必须被判不过 | 没抓住就阻断 |

合计 41 个（Anthropic：起步 20–50 个）。每个任务写了 `origin`（取自哪次真实失败或哪条规约）：exp003 人耳勾吃字 / 金属声 / 背景忽大忽小的片段、exp005「所有的都没降噪」、exp005 的邻座先说（other_first）、rubric 第 7 节正负例。**正反两面**：`polarity` = suppress（该压的：停顿噪声、邻座）· preserve（不该压的：主讲人独白、轻声说话人、音色）· robust（保命与物理量）。

CI 任务全是**确定性合成场景**（`synth.rs`：声门脉冲 + 三共振峰 + 音节包络 + 擦音 + 爆破起音的类语音；粉 / 白 / 风扇 / 6 人 babble / 咖啡馆杯碟 / 和弦音乐噪声；三轨分存 + 逐音节真值）。参数 + 种子即全部，音频不进仓。每个任务 3 个种子 = 3 次试验。

### verifier（`src/eval/src/verifiers.rs` 的 `REGISTRY` 是正本）

| id | 测什么 | 类 | 级别 | 过线 |
|---|---|---|---|---|
| V-LAT | 延迟账 ≤ 45 ms + 实测算法延迟 = 声明 | 物理 | L3 | D32 |
| V-FRAME | 最坏一帧 ÷ 周期 | 物理 | L3 | ≤ 0.5（CI / debug 下不过 → 判不知道） |
| V-NAN | NaN / Inf / 幅度 > 4 | 物理 | L3 | 0 |
| V-MUTE | 数字静音输入的输出峰峰值 | 物理 | L3 | ≤ −90.3 dBFS〔建议值〕 |
| V-SWITCH | 送原声切换跳变 | 物理 | L3 | ≤ 2×〔建议值〕 |
| V-SIM-GATE | 门控漏开（字头 200 ms 单列） | 模拟真值 | L3（仅模拟） | ≤ 5% / ≤ 20%〔建议值〕 |
| V-SIM-FA | 只有邻座说话时门控误开 | 模拟真值 | L3（仅模拟） | ≤ 10%〔建议值〕 |
| V-SIM-SWLAT | 开口到门控打开 p90 | 模拟真值 | L3（仅模拟） | ≤ 32 ms〔建议值〕 |
| V-BGSTAB | 背景跳变（P.1120 §11.14.5 式两次处理对比） | 感知 | L0 | ≤ 10 dB |
| V-ONSET | 吞字头 | 感知 | L0 | ≤ 3 dB |
| V-NR | 停顿噪声电平降低（TNLR）+ SNRI | 感知 | L0 | 任务各定 |
| V-INTF | 邻座压低 | 感知 | L0 | ≤ −10 dB〔建议值〕 |
| V-SPLVL | 人声被压低 | 感知 | L0 | ≥ −3 dB〔建议值〕 |
| V-MUSIC | 残留频谱峰度比 + 孤立亮点 | 感知 | L0 | ≤ 2〔建议值〕 |
| V-VACUUM | 停顿衰减深过 −40 dB | 感知 | L0 | 只报警 |
| V-TIMBRE | 四频带音色偏移 | 感知 | L0 | ≤ 6 dB〔建议值〕 |
| V-EAT-ASR | 吃字 Δ(del + sub)（QA） | 感知 | L0 | rubric V7 |
| V-DNSMOS | DNSMOS SIG / BAK / OVRL（QA） | 感知 | L0 | 不比带噪输入差 |

012 清单里**本批没做**的：V-WEAK、V-DSN、V-ESTOI、V-SNG（TS 103 281，待拍板 #1）、V-NRMOS（SCOREQ / UTMOS）、V-ROBOT（无算法）、V-PUMPV、V-CLICK、V-HALLU、V-CPU、V-DROP、V-CONV、V-EAT-H（人耳）。

## 与 Anthropic 文章的逐条对照

| 文章原则 | 本实现怎么做 | 做到没有 |
|---|---|---|
| 起步 20–50 个任务，取自真实失败 | 41 个任务，每个有 `origin`；`suite.rs` 测试断言总数在 20–50 | ✓ |
| 先把手工检查的东西变成任务 | rubric 第 7 节正负例、负例护栏三种劣化、exp003 / exp005 人耳反馈逐条成任务 | ✓ |
| 任务无歧义：两个专家独立判同一结论；有参考解 | CI 任务阈值写死在清单里、真值来自合成器；「参考解」= 干净主讲人轨（V-SPLVL / V-ONSET / V-TIMBRE 对它比）。**感知类阈值是〔建议值〕，专家未必同意**——这是定标要解决的 | 部分 |
| 0% 通过率多半是任务坏了 | 能力集通过率低是设计（爬坡）；但每个能力任务都写了「为什么现在过不了」（如 C09：衰减下限 −12 dB 封顶），区分「任务坏」与「系统还不会」 | ✓ |
| 正反两面都测（该触发 / 不该触发） | `polarity`：suppress vs preserve；探针 P04（什么都不做）vs P05（全静音）互为反面 | ✓ |
| 稳定、干净的环境，每次试验从干净状态开始 | 每次试验新建 Engine 与模型；合成器按种子确定性；QA 片段核 sha256 | ✓ |
| 评分器优先确定性代码，模型评分器其次，人工校验 | CI 层全是 DSP；QA 层才用 ASR / DNSMOS；人工层只做定标 | ✓ |
| 评结果不评过程 | verifier 只看输出波形（V-SIM-* 看门控状态，那是子模块的「结果」） | ✓ |
| 多组件任务给部分分 | 每个 grader 独立报分与档位（A–D）；任务通过 = 全部 grader 过 | ✓ |
| 模型评分器要与人校准、允许答「不知道」 | 所有 verifier 都能返回 Unknown（带原因）；`calib` 读人耳答卷算一致率、τ、bootstrap 区间，回写级别 | ✓ |
| 每个维度独立评 | 一条 verifier 一个维度；不出总分 | ✓ |
| 能力评测与回归评测分开；能力集毕业进回归集 | 两个集合分开汇总、门禁规则不同；毕业 = 把任务的 `set` 从 capability 改成 regression（需人拍板，见下） | ✓（毕业流程待定） |
| 非确定性：多次试验，pass@k 与 pass^k | 合成任务 3 个种子；ASR 每个文件转 2 次（不一致即剔除）；报告逐任务、逐集合报 pass@k 与 pass^k | ✓ |
| 读原始记录 | 报告附抽查样本路径（每个失败任务的第一次失败 + 每 5 个任务固定抽 1 个），音频在仓外 | ✓ |
| 防评分器被钻空子 | 5 个探针：削词首、非语音段 −40 dB、1 kHz 低通、什么都不做、全静音；成对制衡（V-NR ↔ V-SPLVL / V-VACUUM）；P02 / P05 证明 V-NR 单独会被刷 | ✓ |
| 防饱和 | 能力集故意定高线；回归集到 100% 只说明没退步，改进看能力集 | ✓ |
| 放进 CI，每次改动都跑 | `ci.yml` 加 `eval ci`；回归集 L3 不过、比冻结基线差、探针没抓住 → CI 红 | ✓ |
| 评测集要有主人、持续维护 | 规约 §7.1 写了维护规则；级别表只由 `calib` 改；基线有意变化要在 PR 里说明 | ✓ |

## 定标（D44 · 012 L0–L3）

CALIB_PLACEHOLDER

## 局限（如实）

1. **合成语音不是真话**：CI 层的类语音信号够测门控、电平、字头、背景稳定、音乐噪声这些信号层的事，**不够测吃字与好不好听**；那些只在 QA 层用真实录音测。scene 线的模拟器合并后应换掉 `synth.rs`
2. **CI 测的不是产品模型**：CI 没权重，被测的是 Engine + 参考降噪器。GTCRN 只在本机跑（要不要把 535 KB 的 GTCRN 权重放进仓给 CI 用，见待人拍板 #1）
3. **感知类全部 L0**：人耳数据只有 exp003 一轮（12 屏、会话打折），按 012 的判据第 1 轮是定标轮，正式级别只能 L0。所以 CI 对感知量只能做「不比冻结基线差」，不能说「好听」
4. **exp003 的候选音频已不在盘上**：定标只能用 scores.json 里现成的指标（DNSMOS、noise_att_db、Δdel、ESTOI、SI-SDR），本批新写的 DSP verifier（V-ONSET、V-MUSIC、V-BGSTAB、V-TIMBRE）没有人耳配对可比——下一轮听测时要同时导出候选音频
5. **〔建议值〕很多**：阈值、基线容差（dB 1 · × 0.2 · 比例 0.02 · ms 8）、V-MUTE 的 −90.3 dBFS（[S35] 原文「0.0028 dBFS」单位存疑）都没有出处，要人定或定标后调
6. **V-BGSTAB 改了 P.1120 的口径**：用 10 ms 帧功率、3 帧平均，不用 35 ms 一阶平滑（会把语音能量拖进停顿帧）；只在合成场景上能算（要逐样本可重放的同一段噪声）
7. **V-SPLVL 不是 P.56**：输出含噪声时 P.56 会把噪声算进主动电平，改用 G.160 式扣噪声的帧功率比；P.56 本身实现了（`analysis::active_level`），用于合成器定电平
8. **V-FRAME 在 CI 上不可信**：虚拟机与 debug 构建计时偏慢，不过线时判「不知道」（规约 G10 放 L4）
9. **ort 在 vdn-eval 里直接用**（DNSMOS），是登记的例外；拆后端 crate 时并入

## 待人拍板

| # | 题 | 选项 | 代价 | 建议 | 默认 |
|---|---|---|---|---|---|
| 1 | GTCRN 权重（`gtcrn_simple.onnx`，535 KB，MIT）进不进仓，让 CI 层测真模型 | A 进仓（带 LICENSE 与 sha256）· B 不进，CI 只测 Engine + 参考降噪器 | A 仓里多一份第三方权重；B CI 测不到产品模型的回归 | A | B |
| 2 | 能力任务何时「毕业」进回归集 | A 连续 3 次 PR 的 pass^3 = 100% 自动提议、人确认 · B 只由人改 | A 要多写一点脚本；B 容易忘 | A | B |
| 3 | 012 的〔建议值〕（各阈值、基线容差、L1/L2 判据） | 照用试跑两轮 · 人定数 | 同 012 拍板 #8 | 试跑两轮 | 试跑 |
| 4 | 下一轮听测是否同时导出候选音频与 P.835 三量表 | A 是 · B 否 | A 每轮多约 15 分钟；B 新 DSP verifier 永远定不了标 | A | A |
