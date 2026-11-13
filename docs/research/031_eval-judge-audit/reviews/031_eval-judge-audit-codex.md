# 031 评估器体检：代码修复与结论的 codex 评审

> 对象：`git diff b2e05478`（research/031-eval-judge-audit；`src/eval/` 定标去重 · 过关线混淆矩阵 · 随机抽查，`docs/experiments/exp022-judge-tpr-tnr/`，`docs/research/031_eval-judge-audit/`）· 类型 diff · effort medium · 模型 gpt-6-astra（日志头）· 只读沙箱
> 本记录放在调研目录里（手下 evalaudit 的可写范围）；`docs/reviews/README.md` 的索引行由大总管补。
> 停止线：连续两轮无 P0/P1。

## 第 1 轮（2026-11-13 12:11，59 s）· P0×0 P1×1 P2×2 · 总评：修改

| # | 级 | 发现 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | 吃字题没答（字段缺、null、空列表）被当成「人判没吃」，污染真值 | 属实：exp003 明确「都没有」记为 `["none"]`，另一份旧答卷整份没有该字段 | `judge.rs` 的 `Item.eaten` 改 `Option<bool>`，没答 → 判不了并计数；Python 同改。测试：`judge_tests::unanswered_eaten_is_skipped_not_pass`、`test_stats.MissingEaten`。exp022 结果不变（exp003 每屏都答了） |
| 2 | P2 | 送人耳的段入样概率不等，等权累计的混淆矩阵不能当总体率 | 属实 | `judge.rs` 文件头与 `scorecard.md` 第六节写明「这批答卷上的率」，要估总体须按入样概率加权或分层；exp022 盲听页揭盲表记分层与抽样比例。加权实现留到有答卷后（随 issue 草稿 p2-7） |
| 3 | P2 | J = TPR + TNR − 1 < 0 时 Rogan-Gladen 并非无解 | 属实（例 TPR 0.2、TNR 0.3、观测 0.55 → 0.3） | `stats.rogan_gladen` 只在 |J| 近 0 时返回 None；加 `test_negative_j_still_recovers`；issue 草稿 p2-7 验收条件同步 |
| 抽查不符 | — | audit.md 称 L0–L3 只看方向与 τ，但 L3 另要求 rmse*、对抗样本、重测 | 属实 | 改为「L1、L2 只看这两个；L3 另有要求，但也没有过 / 不过口径」 |

## 第 2 轮（2026-11-13 12:18，61 s）· P0×0 P1×1 P2×2 · 总评：修改

第 1 轮四条，codex 抽查全部「核实属实」，修复到位。另：隐私扫描把 `listen-files.sha256` 里哈希的数字串当手机号拦下；该文件与揭盲表里的音频哈希重复，逐条核对一致后删掉，页面哈希并入揭盲表（唯一编码），未用 `--force`。

| # | 级 | 发现 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | 选题先抽判不过层、再跨层排除同源片段，揭盲表的「抽样比例」不是真实入样概率，加权回不了总体 | 属实 | 改设计：两层各不放回简单随机抽样，整套有同源片段就整套重抽；按同一设计模拟 200 000 次估每条入样概率（附蒙特卡罗误差），抽样框写进揭盲表。测试 `Inclusion.test_shared_source_lowers_probability` 用可手算的小例（1/7 · 2/7）核对。新 build `20261113-122017-3126`：判不过层入样概率 0.095–0.165（n/N 近似是 0.134），旧 build `20261113-121429-ba6b` 标作废、未交人 |
| 2 | P2 | 洗牌后前半段若只有一层，重测原题查找会抛 StopIteration | 属实 | 抽出 `place_retests`：两层原题都换到前半段再接重测；测试 `test_retest_when_first_half_is_one_stratum` |
| 3 | P2 | 揭盲表没事先登记人耳真值切法与本轮用途 | 属实 | 揭盲表加 `prereg`（宽切为主、严切敏感性；unsure 判不了；本轮 = 定标第 2 轮验证集；逆概率加权；重测只量一致性），`make_key` 缺字段即报错，测试 `Key.test_prereg_and_required_fields` |

## 第 3 轮（2026-11-13 12:21，70 s）· P0×0 P1×0 P2×3 · 总评：修改（连续第 1 轮无 P0/P1）

第 2 轮三条 codex 抽查「核实属实」。本轮 P2 三条都属实，代价小，直接修：

| # | 级 | 发现 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P2 | 新答卷是 `answers[{q,…}]`，calib 读 `trials[].by_model`，直接喂会静默出零条 | 属实 | 新增 `convert_answers.py`（答卷 + 揭盲表 → calib 三份输入 + 入样概率 `weights.json`；build 不符、题号不在揭盲表、转出零条都报错；听不出 = 判不了；重测题不进）。测试 `test_convert_answers.py`；另用造的答卷跑通 `vdn-eval calib`（18 条，重测 2 题已排除） |
| 2 | P2 | 揭盲表必填校验只看顶层键，prereg 子字段缺了也能生成 | 属实 | `check_key` 查顶层与 `prereg` 每个子字段非空；测试逐个删字段 / 置空断言报错；现有 build 揭盲表过新校验 |
| 3 | P2 | 缺一类人耳样本时 TPR/TNR 为 NaN，Rogan-Gladen 截断后伪造成 0 | 属实 | 输入非有限值返回 None；测试 `test_nan_inputs_give_none` |

## 第 4 轮（2026-11-13 12:24，45 s）· P0×0 P1×0 P2×2 · 总评：修改 · **停止线满足**（第 3、4 轮连续无 P0/P1）

第 3 轮三条 codex 抽查「核实属实」。本轮两条 P2 属实、改动小，直接修，不再开第 5 轮：

| # | 级 | 发现 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P2 | `prereg` 子字段为 null 时 `str(None)` = "None" 能通过校验；测试的 None 分支实为删字段 | 属实 | 改为「必须是非空字符串」；测试补显式 null 与非字符串 0 |
| 2 | P2 | audit.md 说漏判「一条也量不到」超出证据：听感最差 3 段即使过线也送人耳 | 属实 | audit.md 汇总表与第 3 条改为「只覆盖最差的一小撮段，估不出总体漏判率」 |

**汇总**：4 轮共提出 P1×3 P2×9 + 抽查不符 1，全部属实、全部采纳；没有留 issue 的 codex 条目。实际模型 gpt-6-astra，只读沙箱，effort medium。
