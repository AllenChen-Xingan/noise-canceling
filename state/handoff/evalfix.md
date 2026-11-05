# 现状页 · 手下 evalfix（#111 成绩单评分方法修正）

状态：已收尾 · 等大总管合 main、开 PR（`Closes #111 #109 #110`）
分支 `feat/scorecard-v2` · worktree `../noise_canceling-evalfix` · 管的实体：评测器 · 成绩单（规则与报告）· 定标等级

## 做到哪

代码全部完成；全量重跑 `20261105-005916-4ccf0638` 跑完（1440 段，0 没跑成，9986 s）。结果 `docs/experiments/exp014-scorecard-gtcrn/runs/20261105-005916-4ccf0638/`，结论 `findings.md` 第十一节。收尾 codex 评审 2 轮，连续两轮无 P0/P1，已停（`docs/reviews/111_scorecard-v2-codex.md`）。

PR 正文要写：① `spectrum_golden` 改为冻结法（FROZEN 清单 + 不许少的测试；冻结后哈希 = 原基准 c493e149），基准由 a2 那条线（#99 #107）建；② 背景稳不稳主值修正单独提交 `6e8c66d7`，证明见本页流水；③ 加了哪些测试 · 跑了哪些（vdn-eval 全部 · 大总管全量 `--no-fail-fast` 0 红 · tidy）。

## 人定 D85（2026-11-04，替换计划里的「默认」）

G-1 B 均值 + 95% 区间三态（X1 X2 X4），最差 1–3 段送人耳 · G-2 B 人声起伏 / 音色 / 轻声字改在干净人声单独过一遍模型上算，照判，报告写明「只量到安静时的人声损伤」 · G-3 C 吃字单段多错 ≥ 5 字才否决，不做场景合并（E1 取消，E2 只整理正本）· G-4 A 门控项换成量输出（D1–D5）· G-5 4 dB · G-6 0.1 分 · G-7 咔嗒一直只报不判。代码与文档标「人定 D85」；`--rules v1|v2` 开关照留。

## 计划（依据：调研 025 `decision.md` 第 ①④ 节；G-1..G-7 按默认推进，待人确认）

所有改判定的地方做成一个开关 `--rules v1|v2`（默认 v2），旧规则原样保留，人改主意可切回；每处标「G-x 按默认，待人确认」。

### 第 1 组 · 只汇总（不重跑音频，不要名额，约 1 天）

| 项 | 改哪些文件 | 测试 |
|---|---|---|
| 新增「从 `clips.jsonl` 重新汇总」入口 `vdn-eval scorecard --from-clips <路径>` | `scorecard/mod.rs`（读回 `ClipRow`）· `main.rs` | 读回再写出逐字段一致 |
| X1 均值 + 按源片段成组的配对 bootstrap 95% 区间三态；物理项（V-NAN · V-LAT · V-MUTE · V-FRAME · V-SWITCH）仍逐段否决（G-1） | 新文件 `scorecard/aggregate.rs`（bootstrap · 三态）· `rules.rs` 调用 | 已知分布区间覆盖率 · 区间全在线内 / 外 / 跨线三例 · 物理项一段坏即否决 · 种子固定可复现 |
| X2 听感：每个评分模型 Δ 均值区间对 −δ 三态，δ = 0.1（G-6） | `rules.rs` 听感部分 | 两模型组合 4 种情形 |
| X4 Holm 校正，只在「不过」方向，每项一族（G-1） | `aggregate.rs` | 对照手算例 |
| X3 重估：哪些项在组合层多半落「待测」 | 只出报告，不改判定 | — |
| B4 真空感（V-VACUUM）进成绩单，只报不判，取 V-NR 现有数 | `standard.rs` · `card.rs` | 列存在且不进过不过 |
| E1 吃字合并字数 + 按句成组重采样；线 全集 1 pp · 场景×档 3 pp，标「暂定」（G-3） | `rules.rs` 吃字特判 · `thresholds/table.rs` | 合并公式对拍 jiwer 式累加 · 单段不再否决 |
| E2 吃字口径正本放 `thresholds/table.rs`，删 + 替 | `table.rs` · `qa/mod.rs` · `standard.rs` | 一致性测试 |
| E3 ESTOI 对拍容差 0.01 → 1e−4，补「整段零」样例 | `tests/estoi_parity.rs` · `verifiers.rs` 注册表说明 | 本身即测试 |
| G-2 人声起伏 / 音色 / 轻声字改「只报不判」；G-5 人声被压线 3 → 4 dB；G-7 咔嗒维持只报不判；D2a V-SIM-FA 撤下 | `standard.rs` · `table.rs` | 现有 `scorecard_standard` 扩充 |
| 产出：「新规则下的 exp014」对照表（旧 vs 新，每项 过 / 不过 / 待测 组合数） | `docs/experiments/exp014-scorecard-gtcrn/runs/<新 run-id>/` | — |

### 第 2 组 · 修 bug（不要名额，单测级，约半天）

- #109 延迟测量器：归一化互相关 + 窗放在人声最多的 4 s；先写红测试（合成 1.9 s 底噪 + 150 Hz 浊音、c07 两段回归、已知 300 样本）。文件 `analysis.rs` · `verifiers/physical.rs` · 注册表删「5 种子 = Teams」
- #110 V-WEAK 低采样率：频带上沿截到奈奎斯特，剩不到一半判「不知道」；V-TIMBRE 同理（剩 < 3 带判不知道）。8 / 12 / 16 kHz 测试，16 kHz 读数逐位不变。文件 `verifiers/speech.rs` · `verifiers/perceptual.rs`

### 第 3 组 · 重新打分 / 新运行（要名额）

- 只加旁报、判定不变：A2–A4 · B1 · B2 · B3（扣输入侧偏差，验收原声直通 ≈ 0 dB）· C1 · F1（`#[ignore]` 对拍 + 原声分按内容哈希缓存）· F2（注册表说明）
- 新算法：C2 干净人声单独过一遍模型（只报）· D1 TSOS「用户被误删」+ 原声直通噪声底 · D2b 漏过事件率 · D3 开口后被吞多久（只报）· D4 门控安静项降诊断 · D5 重叠段 SI-SDRi + 失败率
- 文件：`verifiers/sim.rs`（D 组，可能拆新文件 `verifiers/tsos.rs`）· `perceptual.rs` · `speech.rs` · `runner.rs`（C2 多一次处理）
- 名额：先在 1 个场景冒烟（约 10 分钟），报耗时；全量见第 4 组

### 第 4 组 · 重出成绩单 + 自检（要名额）

- passthrough · ref · GTCRN 全 480 段 `--qa`，新 run id：估 2.5 h（025 原估）+ C2 多一次处理约 1/3 → 约 3.3 h，冒烟后再报准数
- 自检：`xtask tidy` · `vdn-eval ci` · 全量 `cargo test`（约 10–20 分钟，申请名额）
- 收尾：codex-review 两轮无 P0/P1 即停；`findings.md` 末尾追加「新规则下的结论」

## 流水

- 2026-11-04 读完共同规矩、交代、025 decision；看了 `scorecard/` 代码与 exp014 run（`clips.jsonl` 1440 行已在仓）；出计划

## 待人拍板（G 题之外新冒出的）

1. **「你的声音被误删」（TSOS，D1）比原声直通多多少算不过？** 025 只说「看比原声直通多多少」，没给数。建议：按场景组合并后多 2 个百分点（025 group-D 粗估：约 49 段才分得出 2 个百分点）。如果你不回，默认只报不判，后果是成绩单上这一项暂时没有机器结论
2. **「旁人漏过」（D2b）：只有旁人说话的 1 秒窗里，压低不到几 dB 算漏过？** 025 写「X 待定」。建议 10 dB（与压旁人 V-INTF 的过关线一致）。如果你不回，默认 10 dB、只报不判

## 转交

1. **A3 系统层「固定正弦 + 8 个相位强制切换」检测**（025 A3，先只报）：要给引擎注入强制切换故障，不进本次成绩单，未做
2. **V-SWALLOW 按场景组合并 ≥ 30 个起点取 p90**（025 D3）：本线只给每段 p90，合并未实现（注册表已写明）
3. **codex 评审脚本的纯文本判断**：用 `file` 判断，文件开头是「P1」会被认成 netpbm 位图而拒发（本线中途 2 第 2 轮遇到，加一行标题绕过）。建议脚本改用「能否按 UTF-8 解码」判断
- 2026-11-04 大总管回「计划可以」：先做第 2 组 + 第 1 组不随 G 题部分；随 G 题的等消息
- 2026-11-04 写 #109 红测试（`analysis.rs` 迟到浊音样例）· #110 红测试（新文件 `verifiers/low_rate_tests.rs`）；申请首次编译名额，等回复
- 2026-11-04 回 cands：外部输出入口形状可以；要求 bg/ 保留、按输入哈希的索引做成通用（以后 clean/ 给 C2）、元数据记目录与 run.json 哈希
- 2026-11-04 名额：首次编译 vdn-eval，5 条红测试按预期红（迟到浊音量出 2014 样本；V-WEAK 8 k / 2 k 切片越界 panic；V-TIMBRE 报越界带），已归还
- 2026-11-04 #109 修：`analysis.rs` 归一化互相关 + `busiest_start` 选主讲人能量最多的 4 s 窗；`physical.rs` 调用改；注册表删「5 种子 = Teams」。测试：迟到浊音原声直通 = 0 · 已知 300 样本 · 全零 → None · `tests/lat_c07.rs`（ignore，要 _tmp 音频）c07 两段 = 0，已人工跑过
- 2026-11-04 #110 修：V-WEAK 不计上沿越过奈奎斯特的带、少于一半判不知道、截掉带时旁报 bands；V-TIMBRE 同理、少于 3 带判不知道。`spectrum_golden` 对拍通过（16 kHz 全部 verifier 读数逐位不变）· lib 67 条 · clippy 无警告
- 2026-11-04 cands 入口审完：同意形状；公平性 3 处（延迟只是声明值 · 回放无计时判不知道 · trunc/ 截断查偷看未来），大总管同意
- 2026-11-04 第 1 组完成：`scorecard/load.rs`（clips.jsonl 读回 + `--from-clips`）· `aggregate.rs`（成组 bootstrap · 三态 · Holm）· `rules_v2.rs`（D85 新规则）· `card.rs`（`--rules v1|v2`、区间列）· 阈值表 SC-AGG / SC-NRMOS-DELTA / fail_chars / SPLVL −4 · scorecard.md 与 denoise.md 同步 + 比对测试（故意改错验证会红）· E3 对拍容差 1e−4 + 零输出样例 · 注册表 A4/E3/F2 说明 · V-VACUUM 进共同项只报
- 2026-11-04 校验：v1 重新汇总与原 exp014 成绩单 180 组合逐一相同；新规则对照表 `docs/experiments/exp014-scorecard-gtcrn/runs/20261104-230202-83cafcd0-resum-v2/compare.md`
- 2026-11-04 自己发现并修：重采样 2000 次时 60 组合 Holm 永远拒绝不了 → 10000 次；「有值段不足一半判待测」太严 → 「源片段 ≥ 2 组」；Holm 族把只报 / 无区间组合算进族大小 → 只数真判的。各带会红的测试
- 2026-11-04 codex 中途评审 1 第 1 轮开跑（对象 `4493f0ce..HEAD`）
- 2026-11-04 中途评审 1 共 2 轮完成（P1×4 P2×3 全属实已修），记录 `docs/reviews/111_scorecard-v2-codex.md` + 索引一行；重新汇总新版 `runs/20261104-231355-da5821a4-resum-v2/`（旧版标「已被取代」）
- 2026-11-04 大总管改名 noise-canceling-26（旧名已下线）
- 2026-11-04 第 3 组：量输出四项 `verifiers/pse.rs`（V-TSOS · V-SWALLOW · V-LEAK · V-PSE-OVL，只报不判）+ 测试 7 条；G-2 B `scorecard::run_with` + `runner::prepare_clip(clean_pass)`，v2 下三项在干净人声单独过一遍的输出上算、带 `clean_pass` 标记，v1 不跑；e2e 测试 `clean_pass_items_scored_on_clean_input_under_v2`
- 2026-11-04 旁报 A2（over_1 · first）· A4（over_1）· B1（low_frac · tnlr_excl_low_db；−48 dBov 登表）· B2（minus_floor_db）· C1（resid_below_speech_db）+ 测试 `verifiers/side_report_tests.rs`
- 2026-11-04 spectrum_golden 按大总管定的冻结法改（FROZEN 清单 + 不许少的测试，故意改错验证会红）；证明：冻结后哈希 = 原基准
- 2026-11-04 B3 主值修正单独提交 `6e8c66d7`（只动 V-BGSTAB：扣输入侧偏差，旧值旁报 uncorr_db，p95_db；注册表 should → shall）。证明：去掉 V-BGSTAB 的冻结组哈希改前改后相同（90be3b14…，150 个数）；合成基准片段上 V-BGSTAB 本身也逐位不变（停顿里干净轨是数字零），基准文件未变
- 2026-11-05 中途评审 2 共 2 轮（P1×3 P2×4 全属实已修）：SI-SDR 全零输出 · 干净人声那一路专用上下文 `Prepared::clean_view` · 漏过事件率滑窗 · 字头 200 ms 误删比例 · 重叠段先交集后桥接 · e2e 测试逐项断言出分 · 电源状态 `power.rs` 进 meta.power
- 2026-11-05 F1：DNSMOS 原声分按内容哈希缓存（`qa::content_key` + 测试）；官方脚本对拍 `tests/dnsmos_parity.rs`（ignore，附人工步骤）
- 2026-11-05 冒烟清单 主目录 `_tmp/evalfix/smoke/scenes-cafe.json`（咖啡馆 4 组合 32 段）；申请名额
- 2026-11-05 冒烟（名额，接电源）：咖啡馆 32 段 · 带 --qa · v2。原声直通 + 参考 517 s；GTCRN 单独 431 s（第一次 GTCRN 32 段全「没跑成」：worktree 没有 git 忽略的权重，设 `VDN_GTCRN_ONNX` 指向主目录 resources 那份 sha256 b4718df6… 后正常）；不带 --qa v1 39 s / v2 49 s → 干净人声那一路每 32 段 × 3 系统多 10 s。全量估约 2.5 h（上限 4 h）。名额已归还
- 2026-11-05 冒烟读数（推断，1 个场景）：原声直通「不做坏事」各项全 0；三个系统「旁人漏过」都 100%；GTCRN 在干净人声上「人声忽大忽小」均值 1.54 dB²（线 1）
- 2026-11-05 延迟预算（大总管加的小事）：`src/types/src/latency.rs` 只加 `planned_platform_for` · `needs_resample` · `RESAMPLE_COMPONENTS` + 测试；`system.rs` 改调新函数、加 `SystemKind::native_rate()` + 测试；外部系统字段 `native_rate` 已发 cands
- 2026-11-05 申请全量名额
- 2026-11-05 全量（名额，接电源）：00:59–03:45，9986 s，1440 段 0 没跑成；名额已归还。小文件进 `runs/20261105-005916-4ccf0638/`（README · compare.md · compare.py）；findings 第十一节「新规则下的结论」
- 2026-11-05 大总管验收：全量测试 0 红 · tidy 过；vdn-schema 红（新 4 项没重导出）→ `cargo run -p vdn-schema` 重导出，只多 4 个评分项名，测试绿
- 2026-11-05 findings 补两处（大总管要求）：「同时说话只留用户」均值 0 与失败率 1 不矛盾 · GTCRN 场景总判只重新汇总 vs 重跑差 3 个组合的原因
- 2026-11-05 收尾 codex 评审第 1 轮开跑（对象 `4493f0ce..4ccf0638` + 结论文档作依据）
- 2026-11-05 收尾 codex 第 1 轮 P2×3（v1 拒绝干净路读数 · findings 数字按 compare 分解表改 · 漏旁人 / 同时说话分开写），全属实已修，带先红后绿测试
- 2026-11-05 收尾 codex 第 2 轮 P2×2（run README 一句 · v2 成绩单写明干净路局限），全属实已修，带先红后绿测试；连续两轮无 P0/P1，停
- 2026-11-05 合 main（大总管做合并，#115 外部入口）后红 1 条 `external_e2e` 逐位一致：根因 = v2 干净人声那一路对外部系统查不到输入。和 cands 定：cands 加 `Kind::Clean` · `provides()`（补丁出自 exp/016 `fe09f692`）；我在 `scorecard::run_with` 先问 `provides`，没提供就三项落「只报·旧口径」、成绩单开头点名（大总管加的要求），不记没跑成。测试 2 条（先红后绿）。合并提交 `bdf0ad57`，`cargo test -p vdn-eval --no-fail-fast` 全绿
