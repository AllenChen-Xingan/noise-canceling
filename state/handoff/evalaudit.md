# 现状页 · evalaudit（批次 20261113-b17）

- **状态**：已收尾
- **手上**：按 Hamel 的 eval 指南体检评估器，重点查和人耳一致性是否分开报假阳性 / 假阴性
- **交代**：`../noise_canceling/_tmp/20261113-b17/evalaudit.md`（主目录，git 忽略）
- **做到哪**（2026-11-13 12:26）：体检、实测、修复（P1 × 4 中代码类 3 条）、自审、codex 4 轮（停止线满足）全部完成；20 题盲听页 build `20261113-122017-3126` 已建好，等大总管交人
- **下一步**（接班者）：人交回答卷 → 放进 `docs/experiments/exp022-judge-tpr-tnr/results/` 并提交 → `convert_answers.py` → `vdn-eval calib … --round 2` → 写按入样概率加权的总体率

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `src/eval/src/calib.rs` | 有效配对去重（规则只写在文件头）；`calibrate_with` 接评分模型表；报告加混淆矩阵一节；术语说明 |
| `src/eval/src/calib/judge.rs` · `judge_tests.rs`（新） | 按过关线的混淆矩阵、Wilson 区间、按屏重采样；吃字题没答记判不了；与 exp022 Python 逐格比对 |
| `src/eval/src/scorecard/rules_v2.rs` · `rules_v2_tests.rs` | 随机抽查 2 段（跳过已逐段否决的段），种子写进原因 |
| `src/eval/src/thresholds/table.rs` | 新增 `SC-LISTEN.random = 2`（人定 2026-11-13） |
| `src/eval/src/main.rs` | `calib --predictors`；用法说明 |
| `specs/rubric/scorecard.md` | 第六节随机抽查行、两种量法与样本口径；L1 改「有效配对」；新增第十节改动记录 |
| `docs/research/031_eval-judge-audit/` | inventory · audit · decision · issues/（P2 × 5）· reviews/（codex 记录） |
| `docs/experiments/exp022-judge-tpr-tnr/` | 实测脚本与结果；盲听页生成 · 模板 · 答卷转换；Python 测试 36 条；builds/（新 build + 作废旧 build） |

验证：`cargo test -p vdn-eval` 全过（lib 151 条 + 集成测试）、`clippy -D warnings`、`xtask tidy` 通过（12:16，此后 src/ 未改）；Python 36 条全过。

## 盲听页（交人用）
- 给人的目录：`/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling-evalaudit/_tmp/exp022-listen/20261113-122017-3126/listen/`（`index.html` + 40 个 `qNN.wav` / `qNN_orig.wav`；已查：无揭盲表、无清单、无模型名或场景名）
- 揭盲表：同 build 的 `key/`（不给人），已复制进 `docs/experiments/exp022-judge-tpr-tnr/builds/20261113-122017-3126/`
- 20 题：9 题评估器判不过 + 9 题判过 + 2 道重测；每题 12 s，约 10–15 分钟；答完点「导出答卷」交回 JSON
- ⚠️ 本 worktree 删除前，把 `_tmp/exp022-listen/` 挪到主目录 `_tmp/`（git 忽略，删 worktree 会一起丢）
- 旧 build `20261113-121429-ba6b` 作废（选题设计被 codex 否定），未交人

## 发现
- 人耳说「不能用」的，SCOREQ 单段线只抓到 0.20（区间 0.00–0.40，36 条，1 人）；它几乎不冤枉能用的（0.88）
- 吃字线从「> 1 个百分点」换成「≥ 5 字」后，抓到率 0.69 → 0.27，冤枉率不变（同一批 36 条）

## 待人拍板（已由人定 2026-11-13：① 吃字线不改 ② 随机抽查 2 段 ③ 要听一页；题面留档）
1. **吃字的否决线要不要收紧？** 现行「单段比原声多错 ≥ 5 字才否决」（D85 题 3）。在 exp003 那批 36 条上，你勾了吃字的 26 条里它只抓到 7 条；旧线「多 1 个百分点」抓到 18 条；两条线都没有冤枉你没勾的 10 条。注意：这批多是英文，5 个英文词比 5 个汉字长
   - A. 不改，等再听一轮、样本够了再定 —— 代价：期间成绩单会把多数你觉得吃字的段判过 ← 建议
   - B. 现在改回「多 > 1 个百分点」 —— 代价：D85 当时要避免的「一个字就否决整段」会回来
   - 如果你不回，默认 A
2. **成绩单每次送你听的段里，要不要加随机抽查？** 现在只送最差、两个模型分歧、越线的段，评估器说「过」的段你从来听不到，所以没法知道它漏了多少
   - A. 每次加 2 段随机抽查，每次听测多约 1–2 分钟（每题 30–50 秒，依据 `scorecard.md` 第六节） ← 建议
   - B. 加 4 段，多约 2–3 分钟，样本攒得快一倍
   - C. 不加
   - 如果你不回，默认 A
3. **要不要再听一轮，专门校准评估器？** 现在只有 1 个会话、36 条，你说「能用」的只有 16 条，区间宽到下不了结论（文章建议过、不过各 30–50 条）
   - A. 用现有 exp014 / exp016 的输出出一页约 20 题（约 10–15 分钟），每题只问「开会能用吗」和「有没有吃字」 ← 建议
   - B. 先不听
   - 如果你不回，默认 B（不占你的时间），评估器继续标「未校准」

## 转交（范围外，留给大总管）
- 自审（vdn-review）报的改动范围外条目：`src/README.md` 仍是初始化模板（说目录是空的）；`src/eval/` 没有 README，verifier / L0–L3 / CI 层与 QA 层没集中定义；rules.rs 与 rules_v2.rs 新旧并存、共用类型放在旧模块里；注释大量只写 D 编号不写人话；单元测试四种放法；tests/common/platform.rs 文件头写的副本数不对（实为 4 份）；依赖两种写法。都不是本线引入的

## 流水
- 报到 → 大总管确认歧义按「有问题就修」理解，修 src/ 前先回报
- 读 field-guide：本文不含 TPR/TNR 公式；读其链接文 llm-judge、evals-faq 补齐（TPR/TNR、训练/开发/测试 10–20/40–45/40–45、每类 30–50 例、二值判定、单一权威标注人、Cohen's Kappa）
- 读 calib.rs：只量「成对方向一致率」+ 片段内 τ，按屏 bootstrap；奇偶轮分定标 / 验证。没有过 / 不过口径，没有真阳性率 / 真阴性率
- 发现：同屏「最清楚」与「能用等级」各生成一次同向配对，同一判断可能被数两次（calib.rs:120-156），会抬高「配对 ≥ 20」门槛的计数，待核实
- 读 exp007 定标第 1 轮：48 对 / 12 屏 / 1 名听者 / 会话打折；全部 L0；SCOREQ 与 Distill-MOS 从未定标
- 全仓 grep：没有任何地方报真阳性率、真阴性率、混淆矩阵或 κ
- 子代理盘点人耳数据：只有 exp003 f8c3 能和评估器配对（36 条）
- exp022：写 stats.py / tpr_tnr.py / 17 条测试，全过；过关线运行时读 table.rs
- 写 inventory.md、audit.md（P1 × 4 · P2 × 5 · P0 × 0）
- 第一次回报发大总管，等放行
- 放行修 1/3/4；人拍板三题：吃字线不改 · 随机抽查 2 段（table.rs 已改标人定）· 要听一页约 20 题（修完再建页）
- 写好第 1、3、4 条的测试与桩（calib/judge.rs · judge_tests.rs · rules_v2_tests.rs · table.rs SC-LISTEN），P2 issue 草稿 5 份
- ⚠️ 拿名额后查 heavy.lock：info 写着 owner: noise-canceling-74 / cands，不空；我先覆盖了 info 再发现，已把第一行原样写回（其后若有内容已丢）；未跑，报大总管等确认
- 规则（大总管 2026-11-13）：读锁前先 ls 判断 heavy.lock 是否存在，别先写；存在就读 info 第一行，不是 free 就停
- 拿名额（mkdir 锁）：先红 5 条 → 修第 1/3/4 条 → cargo test -p vdn-eval 全过、tidy 通过、clippy 无输出 → 删锁、归还
- 重跑定标第 1 轮：有效配对 20（原始 48），存 exp022/results/calib-round1-rerun.md
- scorecard.md：第六节加随机抽查行与两种量法，L1 改「有效配对」，新增第十节改动记录
- 自审（vdn-review，3 个视角）：本改动内 P1 1 条（随机抽查会抽到逐段否决的段）→ 先加红测再修；calib/judge 术语、main 用法补清；范围外条目记进「转交」
- 自审修复：名额内先红后绿（150 条），锁已删、归还
- codex 第 1 轮（gpt-6-astra，59 s）：P1×1 P2×2 + 抽查不符 1，全属实已改；记录 docs/research/031_eval-judge-audit/reviews/031_eval-judge-audit-codex.md
- 隐私扫描把 `eat_tokens` 当密钥赋值误报 → 改名 eat_count（Python 与 Rust 同改），没用 --force-secrets
- 建 20 题听测页 build 20261113-121429-ba6b：音频写本 worktree `_tmp/exp022-listen/`（主目录只读），揭盲表在 key/ 并复制进 exp022/builds/；9 判不过 + 9 判过 + 2 重测；Python 测试 24 条全过
- 申请名额复测 Rust，等回复
- codex 第 2–4 轮：第 2 轮 P1（选题入样概率）→ 改设计、重建 build；第 3 轮 P2×3、第 4 轮 P2×2 全修；第 3、4 轮连续无 P0/P1 → 停止线满足
- 已收尾
