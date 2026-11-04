# 现状页 · scorecard（批次 20261103-b11）

- **状态**：进行中（方案已批）
- **手上**：场景成绩单——评分模型接入 · 补评分项 · 成绩单标准与报告（issue #87）
- **交代**：主目录 `_tmp/20261103-b11/scorecard.md`
- **做到哪**（2026-11-03）：交代第 2–6 步写完；ci 门禁过、全量测试 83 组全过；codex 第 1 轮评审在跑
- **下一步**：逐条核实 codex 意见 → 第 2 轮 → 收尾

## 一、方案（大总管已批，2026-11-03）

来源标注：【读源码】【读文档】【本机查】；没有出处的数一律「暂定」进过关线检查器的待定清单（`src/eval/src/thresholds.rs` 的 `PENDING`）。

### 1. 评分模型怎么接

| | SCOREQ（主） | Distill-MOS（交叉） | DNSMOS |
|---|---|---|---|
| 本机有没有 | 有：`~/.cache/scoreq/onnx-models/adapt_nr_telephone.onnx`，378,049,307 B，哈希与 exp009 记录一致待核【本机查】 | 有：exp009 的 Python 环境里 `distill_mos_v7.pt`，16.9 MB【本机查】 | 有 |
| 许可 | 代码 MIT；权重 CC BY 4.0（017 decision 第 3 节）【读文档】 | 代码 MIT；权重许可 018 写「未核」，我再核一次 | — |
| 怎么跑 | Rust `ort` 直接读 ONNX。预处理很简单：16 kHz 单声道 → 尾部补零到 320 的整数倍 → 一次前向出一个分【读源码 scoreq.py:15、200】。和 DNSMOS 一样放子进程 `vdn-eval scoreq`（一个进程只许一个 ort 环境，`qa/mod.rs:36`） | 上游只有 PyTorch 权重。**做法 A（建议）**：用 exp009 环境一次性导出成 ONNX（固定 122,880 样本输入，分段与求均值在 Rust 里做，照 `sqa.py:126-157`），导出脚本进仓、导出物放 `~/vdn-data/models/`，记哈希。**做法 B（兜底）**：Python 旁路进程，`vdn-eval` 调它 | 降为冒烟：只查「能跑、分数是有限数」，不进成绩单的过线判定 |
| 每段耗时 | exp009 实测：48 段 × 约 10.6 s，三个模型合计 41 s CPU，约 0.85 s / 段（三者合计）【读文档 exp009 README:36】；单独 SCOREQ 的数在对拍时量 | 同左 | — |
| 对拍上游 | 选 6 段（含 exp009 已打过分的段），上游 Python 结果存成小 JSON 进仓（`src/eval/tests/fixtures/`），测试要求差 ≤ 0.01。模型文件不在时测试标 `#[ignore = "要本机模型，人工跑"]` | 同左 | — |

**判定规则**（只打分，不当训练奖励，018 结论 4）：
- 每个模型各自报「输出分」和「比带噪原声高多少」（Δ）
- **两者都说好才算听感过**：两个 Δ 都 ≥ 过线值。过线值暂定 0，即「不比原声差」（012 §2.3 第 4 条「不比基线差」的约束口径）；绝对分线没有出处，不设
- **分歧送人耳**：两个 Δ 方向相反，或 |Δ_SCOREQ − Δ_Distill| ≥ 0.3 → 该段标「送人耳」。0.3 出自 rubric 第 1 节第 3 条（「SCOREQ 比原声高 ≥ 0.3」而另一个「低 ≥ 0.3」，标〔建议值〕），所以登记为「rubric 出处，但原文是建议值」
- 名字：`V-NRMOS-SCOREQ` · `V-NRMOS-DISTILL`（进 REGISTRY，QA 层、感知类、L0）

### 2. 要补的评分项（每个带单元测试 + 「必须判不过」的探针）

| 名字 | 管什么 | 算法（出处） | 过关线 | 探针 |
|---|---|---|---|---|
| V-WEAK | 轻声字被吞 | 1/3 倍频程、125 ms 窗、50% 重叠；每带取干净轨与输出有声段电平的第 30、第 99 百分位，500 Hz–4 kHz 各带平均；报 L30 损失与 Δ(P99−P30)；不先扣整体增益（012 §3.1 表 V-WEAK，依 IEC 60118-15 [S31]） | 012 写 L30 ≤ 1 / 2 / 4 dB 为 A / B / C、Δ < −3 dB 标「被压缩」，012 自己标〔建议值〕→ 暂定 | 只把弱音节（低于 P30 的帧）压 10 dB |
| V-PUMPV | 人声忽大忽小 | 有声段 100 ms 包络比（输出 ÷ 干净）的 dB 方差（012 §3.2） | 012 只写「≤ 1 dB² A …」，不完整 → 暂定 | 对输出乘 2 Hz、±6 dB 的增益抖动 |
| V-CLICK | 爆音 / 咔嗒 | 一阶差分超过局部 20 ms MAD 的 k 倍、且带噪输入同位置没有的点，按每分钟计（012 §3.2，k = 8〔建议值〕） | 「每分钟 0 个为 A」（012）；过线暂定 0 个 | 在人声段随机插 5 个单样本脉冲 |
| V-ESTOI | 可懂度 | ESTOI，照 pystoi 口径用 Rust 重写；本机 exp002 环境有 pystoi 0.4.1，可对拍（差 ≤ 0.01） | 用 ΔESTOI（输出 − 原声）：< −0.05 → D、< 0 → C、≥ 0 → B、≥ +0.05 → A（rubric C3 表，有出处） | 1 kHz 低通（复用 P03） |
| V-DSN（按需） | 降噪伤人声 | G.160 附录 II：SNRI − TNLR（012 §3.1） | G.160 II.5 原文数字没读到 → 暂定；本批只在时间够时做 | — |

### 3. 成绩单标准 `specs/rubric/scorecard.md` 的结构

1. **用途与边界**：只说「各场景测过、效果还可以」（D81）；不当训练奖励
2. **场景键**：只认 021 主表的 15 个地点键（`other` 不进成绩单）。Rust 里加场景键枚举，测试直接读 021 主表第一列比对，不一致即红（唯一真相源）。设备键同理对 022 键表
3. **每个场景必看哪几项 + 过关线**：一张「场景 × 项」表，每格写过关线和出处；没出处写「暂定」，进待定清单。例：安静类（`home-quiet` `hotel-room` `phone-booth` `library-classroom`）重点查「不做坏事」（V-WEAK · V-ONSET · V-CLICK · V-MUSIC）；有旁人的（`open-office` `cafe` `canteen` `cowork`）加 V-INTF · V-SIM-FA；交通类加 V-NR 与低频 V-TIMBRE
4. **判定顺序**：先一票否决（吃字 V-EAT-ASR · 人声被压 V-SPLVL · 延迟 V-LAT · 静音 / 坏值 V-MUTE V-NAN），任一不过该场景直接「不过」，后面的分只记数；再查各项过关线；最后听感（两个评分模型都说好）
5. **汇总**：场景内「最差一项决定过不过」，均值只作参考；全局不只给总均值，列出最差的 3 个场景和不过的项
6. **往下拆**：场景不过时，按片段标注里「含哪些声音」分组（含 / 不含某类声音的片段各算一次该项均值），差最大的声音类别列为「拖后腿的声音」；每组片段少于 3 段时只列出、不下结论
7. **人耳校准**：只为置信度不够的项（感知类，全部 L0）排盲听：每次最多 8 屏、每屏约 70 秒，合计 ≤ 10 分钟；优先放「两个评分模型分歧」的片段；结果回灌定标（D44）
8. **旧场景组映射**：G1 非人声噪音 → 安静类 + `cafe` 杯碟片段；G2 单个干扰说话人 → `open-office`；G3 多人 babble → `cafe` `canteen`；G4 中文 → 不是地点，改为跨场景的「语种」维度

### 4. 成绩单命令

`vdn-eval scorecard --suite scenes`：读 scenesuite 的片段清单 → 按场景跑 → 写一张 Markdown 表 + 一份 JSON（场景 × 项 → 分数 · 过不过 · 拖后腿的声音）。端到端测试用 2–3 段自造的合成小片段（不需要模型；评分模型那两项在测试里判「不知道」）。

### 5. 需要名额的步骤（到时再申请）

1. SCOREQ / Distill-MOS 对拍：6 段，预计 ≤ 2 分钟
2. Distill-MOS 导出 ONNX（做法 A）：预计 ≤ 2 分钟
3. 全量 `cargo test`：收尾前一次
不需要下载：两个评分模型本机都有。

## 改了哪些文件

| 文件 | 改了什么 |
|---|---|
| `src/eval/src/verifiers/speech.rs` · `estoi.rs` | 新评分项 V-WEAK · V-PUMPV · V-CLICK · V-ESTOI；ESTOI 照 pystoi 口径重写 |
| `src/eval/src/verifiers.rs` | 登记表加 6 项（含两个评分模型）；子模块接线 |
| `src/eval/src/qa/nrmos.rs` · `nrmos_v.rs` · `qa/mod.rs` | SCOREQ / Distill-MOS 用 ort 跑；常驻子进程 `vdn-eval nrmos … --serve`；模型哈希正本 |
| `src/eval/src/thresholds.rs` → `thresholds/table.rs` | 表拆出（原文件 581 行，加项会超 600）；加 7 组过关线；待定清单 7 → 8 条（V-PUMPV） |
| `src/eval/src/probes.rs` · `suite/ci.json` | 探针 4 个 + 任务 P06–P09 |
| `src/eval/src/scene.rs` | 片段清单类型（正本）：场景键 · 设备键 · 系统处理模式枚举 |
| `src/eval/src/scorecard/`（`mod` · `standard` · `rules` · `card`） | 成绩单：跑、汇总、拖后腿的声音、Markdown + JSON |
| `src/eval/src/suite.rs` · `runner.rs` · `runner/real.rs` · `main.rs` · `files.rs` | 场景任务来源 `kind = scene`；`scorecard` · `nrmos` 子命令；`calib --write-levels` 补缺项（`LevelsFile::fill_missing`） |
| `src/eval/suite/levels.json` | 用 `calib --write-levels` 重生成：补 6 个新项（L0），原有条目逐项核对未变（只是缩进从 1 格变 2 格） |
| `src/eval/tools/export_distillmos.py` · `gen_nrmos_ref.py` · `tests/fixtures/gen_estoi_ref.py` | 导出与参考分脚本（依赖写在文件头） |
| `src/eval/tests/`（`estoi_parity` · `nrmos_parity` · `scene_keys` · `scorecard_e2e` · `scorecard_standard` · `thresholds`）· `tests/fixtures/` | 对拍、键表比对、端到端、标准文档比对；thresholds 测试改为按目录扫子模块 |
| `src/schema/src/eval.rs` · `lib.rs` · `tests/eval_suite.rs`（大总管批） | 导出 `eval-scene-clip.schema.json`；正例 + 负例 |
| `specs/schema/eval-scene-clip.schema.json` · `eval-suite.schema.json` · `README.md` | 导出物；README 加一行 |
| `specs/rubric/scorecard.md`（新）· `denoise.md`（只加一行指针） | 成绩单标准 |

## 发现

- SCOREQ 的「natural」模式实际读 `adapt_nr_telephone.onnx`（上游 `scoreq.py:93` 把 natural 映射成 telephone）。不是错，成绩单里以文件名为准
- **V-WEAK 灵敏度低**（实测，合成人声）：最弱 30% 的段压 −10 / −20 dB 只读到 1.8 / 3.7 dB，过线 4 dB 只抓得住整段吞掉；输出里残留噪声多时 L30「损失」是负的（passthrough 上 −9.4 dB），等于看不见。要靠人耳
- **V-CLICK 原算法误报**（实测）：平滑增益（4 Hz ±30%）上 1746 个/分钟。改为「输出自身离群 + 干净人声解释不了 ≥ 一半 + 听得见 + 背景轨排除」，加两条负例测试。真实 cafe 10 段 ref 上 ~100 → 28 个/分钟，只有 14% 在门控开关附近、帧内相位均匀，机器分不开 → 成绩单里**只报不判**（大总管定）；跳变最大的 5 处剪在主目录 `_tmp/scorecard/click-check-20261103/`，等 GTCRN 成绩单后再定要不要听
- **P06 拆成两条**（大总管定）：吞掉 30% 弱音节标「已知做不到」（xfail）；吞掉 40% 的 P10 必须判不过
- 1 kHz 低通在合成人声上只让 ESTOI 降 0.02（1.000 → 0.979），所以可懂度探针改用「加噪到 0 dB」
- Distill-MOS 导出：torch 导出器不支持复数 STFT，换成等价的实数卷积（对原模型差 1.4e−6）；导出要的 onnx 包装在主目录 `_tmp/scorecard-onnx-pkgs/`（29 MB，不进仓，不动 exp009 环境）
- 评分模型每段重起子进程要重载 378 MB 模型：改成常驻子进程，480 段从估计约 40 分钟降到只载一次（未实测）
- tidy 的同名检查（R-MIRROR）抓到 `SceneKey` 与 `vdn_types::SceneKey`（用户层 5 类）同名、不同物：本线的改名为 `MeetingPlace`（021 地点键），`Status` 改名 `CardStatus`

## 待人拍板

1. **暂定过关线先用着，还是先盲听校准再出成绩单？** 现在 V-PUMPV（人声忽大忽小）、V-SPLVL（人声被压低）、门控误关三处过关线没有出处。参考降噪器在 cafe 10 段上，V-PUMPV 有 9 段不过。
   - A. 先用暂定值出第一张成绩单，标「暂定」，盲听结果回来再改 ← 建议
   - B. 先盲听校准这三项，再出成绩单（每次 ≤ 10 分钟，第一张成绩单推迟到校准后）
   - 如果你不回：默认 A，后果是第一张成绩单里这三项的「不过」可能偏严或偏松

## 转交（范围外，留给大总管）

- scenesuite 合并进本分支后：`src/schema/src/eval.rs` 的 `DATA_FILES` 加 `("src/eval/suite/scenes.json", SUITE_FILE)` 一行（否则 `eval_data_files_registered` 红）；`repo_scene_suite_parses_and_expands` 测试那时才真正生效
- scenes.json 只有 `mba-builtin` 的 30 条任务，240 段 `external-mic` 片段没有任务展开
- 按语种（`lang`）分组比较还没做（旧 G4 中文组的替代），`scorecard.md` 第七节已写
- 成绩单输出默认在主目录 `_tmp/scorecard/<run_id>/`；要进仓（CLAUDE.md 第五节第 1 条）时放哪个目录，由大总管定

## 流水
- 片段清单类型 + schema（与 scenesuite 对齐两轮，加 4 个字段）· 成绩单标准 + 比对测试 · 成绩单命令 + 端到端测试 · 常驻评分子进程 · tidy / clippy 过
- 用 scenesuite 第一版 build（20261103214702-b22d）实测：30 条任务展开 240 段；cafe 常见档 10 段 ref 约 1.7 秒出表
- 新评分项 V-WEAK · V-PUMPV · V-CLICK · V-ESTOI（`src/eval/src/verifiers/speech.rs` · `estoi.rs`）+ 探针 P06–P09 → 单元测试全过
- ESTOI 对拍 pystoi 0.4.1，5 段公式信号，最大差 0.00000（`tests/estoi_parity.rs`）
- V-WEAK 灵敏度低（实测）：弱音节压 −10 / −20 dB 只读到 1.8 / 3.7 dB，过线 4 dB 只抓得住整段吞掉 → 探针改为整段置零，登记表写明
- 1 kHz 低通在合成人声上只让 ESTOI 降 0.02，抓不住 → ESTOI 探针改为加噪到 0 dB
- 名额：Distill-MOS 导出 ONNX（复数 STFT 换成等价实数卷积，对原模型差 1.4e−6）· 上游参考分 · 对拍：SCOREQ 差 0.00000、Distill-MOS 0.00225 → 通过，名额已还
- 级别表：`calib --write-levels` 加「补缺项」（`LevelsFile::fill_missing`，带测试），撤掉手加条目后用命令重生成；原有条目逐项核对未变
- 读交代、共同规矩、手下协议 → 向大总管报到
- 读 `src/eval/`（REGISTRY、thresholds 的 7 条待定、qa、probes、suite）· 016 §1 · 017 / 018 decision · 012 §3 新评分项定义 · rubric C3 与第 1 节 · 021 主表 · 022 键表 · exp009 怎么跑 SCOREQ → 写方案（本页第一节）
