# r5 · 本机盘点：exp011 最小离线模拟的现成积木

> 2026-11-02 · 018 判断回路线 · 只读盘点，没编译、没跑模型、没下载。路径缩写：`J/` = 本 worktree（含 PR #38，`03d93c8` 已在 origin/main）· `M/` = 主目录 `noise_canceling/` · `E/` = `noise_canceling-ears/` · `B/` = `noise_canceling-benchsynth/`。
> 证据等级：【源码】读了代码 ·【本机】看了文件 / 目录 ·【官方】【论文】【二手】·【推算】。

## 一句话结论

**评分模型只有 DNSMOS 在本机能直接用；SCOREQ 不在本机（没权重、没包、没 venv）；Qwen3-ASR 1.7B 已在本机 42model 服务里跑着。** 合成格首选 **exp006 留出 fed2**（真值场景与路由表一一对应），exp008 当「有干扰者分轨 + A/B 两套生成器」的补充。门控 / 下限 / 修音 / strength 全是「GTCRN 输出之后」的事，**GTCRN 每段只跑一次、其余对缓存重算**成立；但 vdn-eval 现在**没有**「对任意 wav 对 + 任意参数」打红线分的入口，exp011 要补一个小 Rust 驱动。

## 证据表

### 1 · 评分模型

| 项 | 状态 | 出处 |
|---|---|---|
| DNSMOS P.835 | 本机有 `sig_bak_ovr.onnx`（1.16 MB）、`model_v8.onnx`；Rust 自写推理，9.01 s 窗 1 s 跳，不够长自我重复 | 【本机】`M/resources/DNS-Challenge/DNSMOS/DNSMOS/`；【源码】`J/src/eval/src/qa/dnsmos.rs:1-20` |
| DNSMOS 怎么调 | 单独命令 `vdn-eval dnsmos <模型.onnx> <音频.wav>`，只收 16 kHz，输出一行 JSON `{sig,bak,ovrl,windows}`；QA 层在子进程里调它（ort 一个进程只许一个 Environment） | 【源码】`J/src/eval/src/main.rs:327-342`；`qa/mod.rs:35-37, 58-66, 182-205` |
| SCOREQ | **不在本机**：所有 venv 都没有 `scoreq`；ears 线计划的 `exp009-venv` 不存在；`~/.cache/huggingface`、`~/vdn-data` 下无权重。exp009 的 SCOREQ 数来自上游元数据里**预先算好的** `scoreq.scp`，不是本机推理 | 【本机】`~/vdn-data/public-mos/vmc2026-track1-meta/ebe09d8…/*/test/scoreq.scp`；`E/docs/experiments/exp009-more-ears/README.md:33`（「SCOREQ 首次下载 ONNX 约 378 MB」）；`score_author.py:40-41`（`scoreq.Scoreq(data_domain="natural", mode="nr")`） |
| SCOREQ 对众人 | URGENT24 语句 SRCC 0.789 / 系统 0.926；CHiME-7 0.744 / 0.9；DNSMOS CHiME-7 语句 0.232 | 【本机统计，分数是上游预算】【已核改】相关系数是 ears 在本机算的，但 SCOREQ / DNSMOS / Distill-MOS / UTMOS / NISQA 各分全读自上游 `vmc2026-track1-meta` 的 `<指标>.scp`（`E/…/exp009-more-ears/crowd_ears.py:33, 82, 97`），**不是本机推理**；DNSMOS 0.232 也不是本仓 Rust DNSMOS 的分。`E/…/exp009-more-ears/results/crowd_ears.md:10-16` |
| NISQA | 源码 + 权重 `nisqa.tar` 1.05 MB / `nisqa_mos_only.tar` 0.91 MB；要 torch（exp008-torch-venv 有 torch 2.15，但没装 pandas）。CHiME-7 语句 SRCC 0.424 | 【本机】`M/resources/NISQA/weights/`；`crowd_ears.md` nisqa 行 |
| UTMOS22 / UTMOSv2 / SpeechMOS / URGENT-PK / versa | 只有源码，**无权重** | 【本机】`M/resources/` |
| venv | `M/_tmp/exp002-venv`（441 MB）：numpy · scipy · librosa · soundfile · onnxruntime 1.31.1 · pesq · pystoi；`M/_tmp/exp008-torch-venv`（1.1 GB）：另加 torch 2.15.0 · torchaudio 2.11.1。**两者都没有 scoreq / nisqa 包 / transformers / qwen**；`E/` 没有 `_tmp/` | 【本机】`*/lib/python3.12/site-packages` |
| Qwen3-ASR（D58 吃字） | **在本机**：42model 服务 `127.0.0.1:11520` 运行中（`~/.42model/endpoint.json` running=true），模型目录 `Qwen3-ASR-Plus-1.7B`（2.3 GB）与 `-0.6B`（2.7 GB）；vdn-eval 用 `qwen3-asr-plus:1.7b`、curl 调、按 sha256 缓存。另有别的应用下的 `Qwen3-ASR-0.6B-F16.gguf`（1.5 GB，HF 缓存）不归本项目 | 【本机】`~/.42model/models/`；【源码】`J/src/eval/src/qa/asr.rs:1, 15-16, 39-56` |
| 现成 ASR 缓存 | exp006 92 段的转写缓存 | 【本机】`M/_tmp/20261031-exp006/20261030214530-44df/ab_work/asr_cache.json` |

### 2 · 合成格

| 项 | exp006 留出 `fed2` | exp008 `20261031-102406-0c67` |
|---|---|---|
| 规模 | test 280 段 + seq 16 条，10 s；有目标说话人约 88% | 348 段 × 8 s = A / B 两套生成器各 174 |
| 分轨 | 只有 `<id>_clean.wav` + `<id>_noisy.wav`（邻座干扰混在噪声里）；【已核改】无目标说话人的 36 段**没有** `_clean.wav`（目录 524 个文件 = 244×2 + 36×1） | `mix / owner / interf / noise` 四轨，float32 |
| 格档名 | 每段 `fine`（7 类：traffic 58 · babble 52 · neighbor 43 · quiet 39 · music 36 · clatter 27 · hvac 25）、`user`、`device`（【已核改】取值是 `none` 166 / `bt_narrow` 55 / `clip` 59，没有 `normal`）、`snr_db`、`rt60_mid` | 目录 `<A|B>/<格>/c<NN>_<档>_<轨>.wav`；格 = S-SNR（−5…20）· S-NOISE（stationary/cafe/babble/clatter/traffic/music）· S-RT60（car/office/cafe/hall）· S-SIR（none,20…−6）· M-SNRxNOISE · M-SIRxSNR |
| 真值 | `truth.jsonl`（场景真值，可直接喂路由表）；另有 Rust 场景特征 `feat_test_final.jsonl`（每 0.5 s 一行、35 维 `feat`） | `truth.jsonl` 每段 `cond` + 逐 hop（256）四类标签 0 无人声 / 1 主讲人 / 2 干扰 / 3 重叠 + 素材出处 + PCM 哈希 |
| 出处 | 【本机】`J/docs/experiments/exp006-scene-sim/builds/20261030235558-fed2/{build.json,truth.jsonl}`【已核改：是 build.json】；特征 `M/_tmp/20261031-exp006/20261030235558-fed2/feat_test_final.jsonl`（仓外，5600 行 = 280 × 20）；音频 `M/_tmp/20261031-exp006/20261030235558-fed2/test/`；README §1 表 `J/…/exp006-scene-sim/README.md:12-35` | 【本机】`B/docs/experiments/exp008-bench-validity/builds/20261031-102406-0c67/{manifest.json,truth.jsonl}`；`cells.py:150-160`（标签规则）；音频 `M/_tmp/exp008/20261031-102406-0c67/` |
| 坑 | fed2 **从没参与任何调参**（README:53）；但修音预设是在 44df 上初调的 | **exp008 不在 main、远端也没有这条分支**（只在本机 benchsynth worktree）；S-NOISE 没有 quiet / neighbor 类，映不全路由表；合成杯碟 / 音乐与真实录音统计不对齐（README S6 表）；A 的素材与 exp006 训练集重叠（局限 6） |

**意外收获**：exp006 调参集 44df 的 244 段已缓存**原始 GTCRN 输出**（`*_base.wav`，Python 流式、未过 Engine）、带噪、FE-M 路由混合、修音各版——`M/_tmp/20261031-exp006/20261030214530-44df/ab_work/raw/`（1708 个文件，899 MB）【本机】。

### 3 · 离线跑链路与「只跑一次 GTCRN」

| 环节 | 现状 | 能否对缓存重算 | 出处 |
|---|---|---|---|
| GTCRN | Python `exp003/enhance.py` `stream()`（ORT，输出与输入逐样本对齐）；或 Rust `vdn denoise`、vdn-eval `SystemKind::Gtcrn` | —（只跑一次） | 【源码】`J/docs/experiments/exp003-offline-ab/enhance.py:136, 164`；`J/docs/experiments/exp006-scene-sim/offline_ab.py:126-157` |
| 下限 / 快开慢关 | `out = w + m·(d − w)`，`m` 由 `OpenCloseEnvelope` 按**模型输出 wet** 的 hop 能量判；`apply_params` 只写 floor/open/detect/release/hangover | **能**：只依赖 wet 与延迟对齐的 dry | 【源码】`J/src/engine/src/engine.rs:7-8, 77-83, 351-378`；`J/src/dsp/src/envelope.rs:82-98` |
| strength | 定义「输出 = 原声 + strength·(降噪 − 原声)」，**全仓还没有实现**（只在 types / memory / policy 出现；【已核改】另有 `scene/src/mirror.rs:107, 119` 一个同名字段与 `engine.rs:76` 注释，均无混合计算） | **能**（纯湿干比）；与下限的先后顺序未定 | 【源码】`J/src/types/src/params.rs:40, 117-150`；`engine.rs:76` |
| tone 修音 | `polish_offline <dry> <wet> <out> <tone|时间线> [--params smooth,floor,comfort,demuffle,agc]`，对 dry/wet 各做一次 512/256 STFT 模拟「挂在模型 STFT 上」 | **能**：exp006 就是对缓存 GTCRN 输出跑的 | 【源码】`J/src/polish/examples/polish_offline.rs:1-8, 70-85`；预设表 `J/src/polish/src/params.rs:50-86` |
| model | GTCRN / FE-M 二选一 | 不能「叠加」，FE-M 要另跑一次 | `J/src/types/src/params.rs:11` |
| vdn-cli | `vdn denoise` 有 `--floor-db --open-db --hangover-ms --release-ms --no-floor…`，**无 `--detect-db`、无 strength**，每次重跑模型 | 否 | 【源码】`J/src/cli/src/main.rs:64-98, 235-245` |

路由表（`J/src/scene/src/route.rs:92-120`）【源码】：**只改 model 与 tone，其余参数全取 `ParamSet::DEFAULT`**（strength 1 · floor −12 · open −6 · detect 9 · release 0.3 · hangover 0.15，`types/src/params.rs:141-150`）。quiet / hvac → GTCRN + tone 1；babble → **FE-M** + 22；clatter → GTCRN + 2；neighbor → GTCRN + 3；traffic → GTCRN + 4；music → **FE-M** + 25；窄带蓝牙 tone +10；未判出 → tone 9。013 已写「GTCRN 输出与门控参数无关，每段只跑一次并缓存降噪前后两路」【二手·本仓】`M/docs/research/013_rl-environment/decision.md:128`。

### 4 · 红线 verifier（main 上，PR #38 后）

| verifier | 位置 | 需要 | 级别 |
|---|---|---|---|
| V-LAT | `verifiers.rs:56` / `v_lat` 371 | `RunOut`（引擎跑记录） | L3 |
| V-NAN | `:82` / 440 | 只要输出 | L3 |
| V-MUTE | `:95` / 459 | 输入数字静音段 | L3 |
| V-ONSET 吞字头 | `:134` / 613 | 干净参考（主讲人轨），≥3 个字头 | L0 |
| V-SPLVL 人声被压低 | `:173` / 729 | 干净参考 + owner/interf 掩码，主讲人独说 ≥0.5 s；线 −3 dB | L0 |
| V-SIM-GATE 门控漏开 | `:225` / 992 | **分轨（tracks=true）+ RunOut 的逐 hop 门控**；线 漏开 ≤5%、字头 200 ms 漏开 ≤20% | L3（仅模拟） |
| V-EAT-ASR 吃字 | `:277`；`qa/mod.rs:101-150` | 干净参考的 ASR 转写当参考；Δ(删+替) | L0 |
| V-DNSMOS | `:290` | 无参考；过线 = 不比带噪 OVRL 差 | L0 |

级别正本 `J/src/eval/suite/levels.json:5-80`【源码】。**命令行单独调用**：只有 `dnsmos` 能对任意 wav 跑；其余只能经 `vdn-eval ci|qa` 跑任务集（`main.rs:1-13, 374-382`），被测系统固定为 `SafetyConfig::default()`（`system.rs:249-274`），**不能注参数、不能吃外部 wav 对**。但任务集支持 `real.kind = "exp005"`：读一个 clips.json，每段 `files.{mix,owner,interf,noise}.{file,sha256}`，sha256 可空（`runner.rs:248-305`）——exp008 正好是这四轨，但**不是「换个清单就能进」**【已核改】【源码】：① 任务集是 `include_str!` 编进二进制的（`suite.rs:16-17`），加任务要改 `suite/qa.json` 并重编；② 每个 task 只指一个 `clip`（`suite.rs:85-89`），348 段 = 348 条 task；③ exp008 清单字段是 `path` / `pcm_sha256`，loader 要 `file` / `sha256`，且后者是整文件 `shasum`（`wav.rs:45-53`）不是 PCM 哈希，照搬会校验失败，只能留空或重算；④ 被测系统仍固定 `SafetyConfig::default()`，所以这条路只能评默认参数，不能扫参。

### 5 · 耗时参照

| 项 | 数 | 出处 |
|---|---|---|
| GTCRN 每 16 ms 帧 | Python ORT 均值 0.40 ms，p99 0.42 | 【本机实测】`J/docs/experiments/exp001-model-frame-time/README.md:49`；`M/_tmp/exp001-run.txt` |
| 修音链 | 约 0.03–0.04% 个核 | 【本机实测】exp006 README §3；`ab_scores.json` `cpu_share_pct_of_one_core` 0.04 |
| exp008 排名 | 12 系统 + 理想掩码 × 348 段 = 6287 s（含 DNSMOS、大模型） | 【本机实测】`B/…/exp008-bench-validity/README.md:49` |
| vdn-eval QA 全跑 | 约 12 分钟（ASR 有缓存更快） | 【本机实测】`J/docs/experiments/exp007-eval-suite/README.md:28` |

## 对本项目的含义（全部「假设」）

1. **评分模型分工**：本机今天能当奖励候选的只有 DNSMOS（D58 后是旁证）。要按 D58 用 SCOREQ 当奖励，exp011 第 0 步得新建 venv 装 `scoreq`（权重约 378 MB，超铁律 8 的 50 MB 线 → 放 `~/vdn-data/models/`，仓里只留路径 + 哈希），**下载需人点头**。建议：**SCOREQ = 调参奖励，DNSMOS + NISQA = 留出防讨好**（三者对众人的一致率差得多，留出那方偏弱，这点要写进结论）；吃字 / 人声被压低 / 吞字头 / 门控漏开一票否决。
2. **合成格**：fed2 test 按 7 个 fine 类各取 12 段（只取 device=`none`【已核改：没有 `normal` 这个值，按字面过滤只剩 bt_narrow】与 bt_narrow，有目标说话人；可选段数 traffic 44 · babble 39 · neighbor 28 · quiet 26 · music 23 · clatter 17 · hvac 14，每类 ≥12 够取）≈ **84 段 ≈ 14 分钟音频**；再从 exp008 取 S-SIR + M-SIRxSNR 的 A 组（【已核改】实为 36 + 24 = **60 段**，不是 ≈ 24；要 24 段需再抽样）补干扰者红线（V-INTF / V-SIM-FA），B 组同档当「换生成器」留出。fed2 清单按 exp005 格式写：interf 置零、noise = noisy − clean。
3. **对照**：「路由表查表」要喂 fed2 的真值 fine，babble / music 要用 FE-M → 每段额外跑一次 FE-M（或另报一个「只换 tone 的 GTCRN 版查表」）；「固定默认」= `ParamSet::DEFAULT` + tone 9；「事后最优」= 每段全网格里奖励最高且红线全过的那组。
4. **要补的积木（Rust，一个 example 即可）**：读 dry + 缓存 wet + 参数组 → 复用 `OpenCloseEnvelope` 与混合公式 + strength 湿干比 → 调 `PolishChain` → 输出 wav + 逐 hop 门控；再把 V-ONSET / V-SPLVL / V-SIM-GATE / V-NAN 以库函数方式对它评分（`verifiers::eval` 是 pub，`Ctx.run` 可用伪造的 `RunOut`）。
5. **CPU 估算**【推算】：GTCRN 84 段 × 10 s ≈ 52 k 帧 × 0.4 ms ≈ 0.4 分钟（Python 开销算进去约 2 分钟）；FE-M 同量级几分钟。参数网格设 48 组（floor 4 × open/detect 3 × release/hangover 2 × strength 2，tone 取路由预设 + 默认）：84 × 48 ≈ 4000 次后处理，每次 < 0.2 s → 约 10 分钟。SCOREQ 速度未测，按 wav2vec 级编码器 RTF 0.05–0.1 粗估：4000 × 10 s × 0.1 ≈ **67 分钟**，是大头 → 先用 DNSMOS + 物理红线筛到每段前 8 组再上 SCOREQ（≈ 11 分钟）。吃字只对候选前沿跑：84 段 × 3 版（默认 / 查表 / 选中）× 2 次转写，若每条 1–3 s 约 15–25 分钟。**合计约 1–1.5 小时单机**。

## 没查到 / 存疑

- SCOREQ 在本机 CPU 的实际速度、ONNX 能否离线加载：**未查到**（本机没装）；对 0.5 s 短窗打分是否稳定：未查到，倾向窗口 ≥ 2–4 s。
- strength 与下限混合的先后顺序：规约只说「Pipeline 混合时用」（`engine.rs:76`），没有实现可对照。
- exp006 `*_base.wav` 是 Python 版 GTCRN、未过 Engine；与 Rust 版逐样本是否一致看 exp004（未在本笔记复核）。
- exp008 只在本机 worktree：清单若丢，348 段无法复核——建议先推送 016 分支。
- fed2 的 `clean` 是带房间混响的目标还是干声：README 说经 RIR，未逐行核 `sim.py`。
- 「事后最优」要先定「最优」由哪个分判：若用奖励分，就和被测方同源，偏向奖励；需人拍板。

## 怀疑派核对

> 2026-11-02 · 只读核对（ls / grep / 读源码 / 读 jsonl 计数），没编译、没跑模型。判错的已在正文就地改正并标【已核改】。

| # | 断言 | 实际 | 判定 | 出处 |
|---|---|---|---|---|
| 1 | DNSMOS `sig_bak_ovr.onnx` 1.16 MB、`model_v8.onnx` 在本机；Rust 9.01 s 窗 1 s 跳 | 在（1 157 965 B）；另有 `bak_ovr.onnx`、`sig.onnx`。`model_v8`（P.808）在但 **Rust 不跑**，只 Python 版 exp009 用 | 对（补注） | `M/resources/DNS-Challenge/DNSMOS/DNSMOS/`；`J/src/eval/src/qa/dnsmos.rs:1-3, 14-15` |
| 2 | `vdn-eval dnsmos <模型> <wav>`，只收 16 kHz，出一行 JSON | 一致；QA 层子进程调用、理由是 ort 单 Environment | 对 | `J/src/eval/src/main.rs:326-342, 379`；`qa/mod.rs:35-37, 181-205` |
| 3 | SCOREQ 不在本机：无包、无 `exp009-venv`、无权重 | 两个 venv 无 `scoreq`；`M/_tmp/exp009-venv` 与 `E/_tmp` 都不存在；`~/vdn-data`、`~/.cache/huggingface/hub` 只有 `scoreq.scp`（上游预算分），无权重 | 对 | `find ~/vdn-data -iname '*scoreq*'` |
| 4 | SCOREQ 对众人的数是「本机实测」 | 相关是本机算的，**各指标分数全读上游 `.scp`**，不是本机推理 | 错（已改） | `E/…/exp009-more-ears/crowd_ears.py:33, 82, 97` |
| 5 | Qwen3-ASR 在本机 42model 服务里跑着；vdn-eval 用 `qwen3-asr-plus:1.7b` | `endpoint.json` running=true，pid 875 进程活着（`42model.app`）；目录 1.7B 2.3 G / 0.6B 2.7 G；`asr.rs:13-16` | 对 | `~/.42model/`；`J/src/eval/src/qa/asr.rs:1-16, 38-56` |
| 6 | venv：exp002 无 transformers / scoreq；exp008-torch 有 torch 2.15.0 · torchaudio 2.11.1、无 pandas | 一致（exp002 另有 pandas 3.1.0） | 对 | `M/_tmp/exp00{2,8-torch}-venv/lib/python3.12/site-packages` |
| 7 | fed2：test 280 + seq 16；fine 7 类计数；有目标约 88% | 280 / 16；计数逐个吻合；有目标 87.1% | 对 | `J/…/fed2/truth.jsonl` 计数 |
| 8 | fed2 `device` = normal / bt_narrow / clip；含义 2 按 `device=normal` 过滤 | 实际值 `none` / bt_narrow / clip，**没有 normal**；按字面过滤只剩 55 段 bt_narrow | **错（已改）** | 同上 |
| 9 | fed2 每段有 clean + noisy | 36 段无目标的**没有 clean**（524 文件） | 错（已改） | `M/_tmp/20261031-exp006/20261030235558-fed2/test/` |
| 10 | 出处 `fed2/{build,truth}.jsonl` | 是 `build.json` + `truth.jsonl`（另有 files.json、eval_holdout.json、recipes.snapshot.json）；`feat_test_final.jsonl` 在仓外 `M/_tmp/` | 错（已改，轻） | `ls J/…/builds/20261030235558-fed2/` |
| 11 | exp008：348 段、A/B 各 174、四轨 float32 16 kHz；不在远端 | 1392 wav = 348×4；`file` 显示 IEEE Float mono 16000 Hz；各格 A=B（S-SNR 30 · S-NOISE 36 · S-RT60 24 · S-SIR 36 · M-SNRxNOISE 24 · M-SIRxSNR 24）；benchsynth 分支 `research/016-bench-synth` 在 `git branch -r` 里无对应远端，领先 origin/main 109 个提交 | 对 | `M/_tmp/exp008/20261031-102406-0c67/`；`B` 的 `git branch -r` |
| 12 | 含义 2：exp008 S-SIR + M-SIRxSNR 的 A 组 ≈ 24 段 | 36 + 24 = **60 段** | 错（已改） | 同上目录计数 |
| 13 | 「exp008 换个清单就能进」vdn-eval QA | loader 收四轨、可空 sha 属实（`runner.rs:248-305`），但任务集 `include_str!` 编译期嵌入、一 task 一 clip、字段名 `path/pcm_sha256` ≠ `file/sha256`、sha 是整文件 shasum；且系统参数固定 | **错（已改）** | `J/src/eval/src/suite.rs:16-17, 85-89`；`wav.rs:45-53`；`system.rs:249-273` |
| 14 | 门控/下限 `out = w + m·(d−w)`，包络按 **wet** 判；`apply_params` 只写 5 个门控参数 → 可对缓存重算 | `env.process(&self.wet, …)` 属实；`apply_params` 只写 floor/open/detect/release/hangover | 对 | `J/src/engine/src/engine.rs:74-84, 352-378`；`J/src/dsp/src/envelope.rs:82-98` |
| 15 | strength 全仓未实现 | 无任何混合计算；但除 types/memory/policy 外 `scene/src/mirror.rs` 也有同名字段 | 对（补注） | `grep -rn strength J/src` |
| 16 | `vdn denoise` 无 `--detect-db`、无 strength | `DenoiseArgs` 只有 floor/open/hangover/release/deadline/lead_in/no_* | 对 | `J/src/cli/src/main.rs:64-98, 236-246` |
| 17 | `polish_offline <dry> <wet> <out> <tone|时间线> [--params …]`；预设 1–6/9/+10/≥20 | 一致；`--params` 时忽略 tone、ramp 置 0 | 对 | `J/src/polish/examples/polish_offline.rs:7-8, 70-85`；`J/src/polish/src/params.rs:45-86` |
| 18 | 路由表：babble→FE-M+22，music→FE-M+25，窄带 +10，未判出 9，其余参数 DEFAULT | 一致；另有 MEETING → tone 6（笔记未提）；`ParamSet::DEFAULT.tone` 是 0 不是 9 | 对（补注） | `J/src/scene/src/route.rs:92-120`；`types/src/params.rs:141-150` |
| 19 | verifier 行号与级别（V-LAT 56/371 … V-DNSMOS 290） | 8 条 id 行号、`fn v_*` 行号、levels.json 级别逐条吻合 | 对 | `J/src/eval/src/verifiers.rs`；`J/src/eval/suite/levels.json` |
| 20 | 44df `ab_work/raw/` 1708 文件 899 MB、244 段缓存 GTCRN 输出 | 1708 = 244 × 7（noisy · base · route · polish · both · null · tones.txt）；899 M。**纯 FE-M 输出没有单存**，`route` 是按决策权重的 GTCRN/FE-M 混合 | 对（补注） | `J/docs/experiments/exp006-scene-sim/offline_ab.py:146-157` |
| 21 | 「exp006 92 段的转写缓存」 | `asr_cache.json` 652 条（键 = 文件 sha256 | 模型 | 语言 | 次）；按段数未能反推 | 存疑 | `M/_tmp/…/44df/ab_work/asr_cache.json` |

**漏掉的积木（补）**：众人表里有 **Distill-MOS**（CHiME-7 语句 SRCC 0.7 / 系统 0.7；URGENT24 0.757 / 0.772）【本机统计，分数上游预算】`crowd_ears.md:13, 18`——它本身就是「蒸馏出的小评分模型」，与 D60 直接相关；本机同样**没装** `distillmos` 包（`score_author.py:42` 要 `exp009-venv`，不存在）。
