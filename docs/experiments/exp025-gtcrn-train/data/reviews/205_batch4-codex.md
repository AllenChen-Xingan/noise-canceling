# 评审 205 · exp025 第四批训练数据（单音检查 · 场景表 · 消融配方 · 开训证据）（codex · gpt-6-astra · read-only）

> 对象：`docs/experiments/exp025-gtcrn-train/data/` 下 diagnose.py · screen_tone.py · scene_table.py · ablation_recipes.py · build_evidence.py · split_rules.json · recipe.json · scene_notes.json · screen_rules.json，以及 test_screen_tone / test_build_evidence / test_scene_table / test_but_real（feat/data-batch3，PR Closes #205）。
> **最新一轮：第 3 轮（2026-11-26）· 总评「修改」· 停：剩下的 P1 是同一条第三次重提，已由裁决人（大总管 noise-canceling-meta）定**。
> 累计处理：P1×5、P2×8 逐条回源码与数据核实：属实 9（其中 Room112 那 1 条提了 3 次，交大总管定）、驳回 1、延期开 issue 2（#221 · #222，#222 被重提 3 次）。

## 第 1 轮（2026-11-26）· 总评「修改」

原文：主仓 `_tmp/reviews-traindata-b4/data-batch4-codex-1.md`（不入库）。effort medium，耗时 67 秒。行号指评审时的版本。
本轮 P0/P1：3 条 · 停止线：未满足 · 留作 issue：#221（Room112）· 第 1 轮 #3 → 见下

- **P1｜diagnose.py:167 不足一秒的尾段漏检**（第 1 轮 #1）
  - 核实：代码属实，影响不属实。实测：BUT 底噪文件长 60.002 秒（`screen/office-3mic-tone.jsonl` 的 `seconds`），尾段只有 2 毫秒（32 个采样），比 1 Hz 分辨率的 1 秒块短 500 倍，测不出 250 Hz。
  - 驳回理由：对现有数据没有影响；改 diagnose.py 也不会改变任何已带回的结果。大总管 noise-canceling-meta 同意驳回（2026-11-26）。
- **P1｜split_rules.json:99 第四批没有强制经过单音合并**（第 1 轮 #2）
  - 核实：属实。实测：筛查表里 Room112 的 248 条都没有 `measures.tone_where`，其中 keep 的 209 条在 build bc0e9e7b566421f0 的训练里。
  - 处理：交大总管定（选 A）。照现状开训；补测并进 trainer 甲段 CPU 机（`cloud-room112-tone.md`）；判法写定：≤ 5% 照用 bc0e9e7b，超过就出新 build。限制写进 `scene-gaps.md` 第 3 节与 `scene_notes.json`。issue #221。冻结的 split.py 不改。
- **P1｜screen_tone.py:133 只按路径关联，认不出旧诊断**（第 1 轮 #3）
  - 核实：属实，但降为 P2。现有缓解：原始证据 `office-3mic-{screen,tone}.jsonl` 都进了仓，测试用它们重跑 merge 逐字节复现 402 行；音频在云盘上不会原地替换。
  - 处理：不在本 PR 改。留待下次改 diagnose 输出格式时一起加音频与判据哈希，并入 issue #221 的后续。
- **P2｜build_evidence.py:178 交集与目标场景只读记录**（第 1 轮 #4）
  - 核实：属实。
  - 处理：pi traindata-codexfix 加了「交集重算一致」列：按记录里的来源重跑 `leakcheck.check`，与记录逐键比较；同时报出本机 exclude.json 的 sha256 前 12 位（b7292768d7f9）。目标场景列改名「目标场景（build 记录）」，并写明分片索引在云上、本机只能核记录。加了测试：记录说通过、重算命中时整页判「否」。
- **P2｜screen_tone.py:110 未知单音状态默认放行**（第 1 轮 #5）
  - 核实：属实。
  - 处理：`where` 只许 none · first_only · later_too；`present_seconds` 必须是整数列表，并与 where 一致（正本 `diagnose.tone_verdict`）。对不上就停。每种情况各加了测试。

### 引用抽查

- codex 列的「不符」：无。「无法核实」2 条（旧规则逐字节、10aa3a9f 可读），由 `tests/test_but_real.py` 的旧规则哈希测试覆盖。
- codex 列的「核实属实」：6 条。

## 第 2 轮（2026-11-26）· 总评「修改」

原文：主仓 `_tmp/reviews-traindata-b4/data-batch4-codex-2.md`（不入库）。effort medium，耗时 56 秒。
本轮 P0/P1：1 条 · 停止线：未满足 · 留作 issue：#222

- **P1｜split_rules.json:99 单音必经检查仍未落实**（第 2 轮 #1，即第 1 轮 #2 重提）
  - 核实：事实属实，已由大总管定：先训后测，≤ 5% 照用 bc0e9e7b（#221）。
  - 处理：同一条第二次提出，按评审规则交裁决人。裁决人是大总管 noise-canceling-meta，已定 A，不再改。这一条的风险写在开训前清单第 2 项的限制里（`scene-gaps.md` 第 3 节）。
- **P2｜build_evidence.py:174 交集来源没核齐**（第 2 轮 #2）
  - 核实：属实（来源列表只取自记录）。
  - 处理：pi traindata-codexfix2 加了「交集来源齐全」列：记录里的来源必须正好等于 `split_rules.json` 的全部来源，不许为空，build 用到的切分表来源都要在里面。加了少一个来源、空列表两条测试。实测基线的 9 个来源和规则的 9 个相等。
- **P2｜screen_tone.py:141 没核逐秒证据**（第 2 轮 #3）
  - 核实：属实。
  - 处理：逐秒证据的条数要等于片段的整秒数，`prominence_db` 必须是有限数；再用 `diagnose.tone_verdict` 加 json 里的 `present_db` 重算，结论必须和存的一致。夹具改成真的 10 秒。实测仓里 1022 条单音结果（C236 620 条、办公室 402 条）全部通过新的核对；办公室 402 行仍逐字节复现。
- **P2｜screen_tone.py:161 旧诊断关联仍在**（第 2 轮 #4，即第 1 轮 #3）
  - 处理：延期，开 issue #222（单音检查结果记音频与判据的哈希）。

## 第 3 轮（2026-11-26）· 总评「修改」

原文：主仓 `_tmp/reviews-traindata-b4/data-batch4-codex-3.md`（不入库）。effort medium，耗时 57 秒。
本轮 P0/P1：1 条，是同一条的第三次重提 · 停止线：脚本判未满足；按评审规则「同一条第二次被驳回就停下交人裁决」停，裁决已有 · 留作 issue：#221 · #222

- **P1｜split_rules.json:99 单音必经检查**（第 3 轮 #1，同第 1 轮 #2、第 2 轮 #1）
  - 处理：交裁决，已定（大总管 noise-canceling-meta 选 A：先训后测，判法定在 `cloud-room112-tone.md`，issue #221）。不再改。
- **P2｜screen_tone.py:196 缺 seconds 静默放行，两表时长不比**（第 3 轮 #2）
  - 核实：属实。
  - 处理：pi traindata-codexfix3 改成缺 `seconds` 就停；筛查行也有时长时，两边相差超过 0.001 秒就停。仓里 1022 条真单音行都有 `seconds`，全部通过（实测）。
- **P2｜screen_tone.py:223 旧诊断关联**（第 3 轮 #3，第三次提）
  - 处理：仍延期，见 issue #222。
- **P2｜screen_tone.py:200 条数核对会误报整秒边界**（第 3 轮 #4）
  - 核实：属实（上一轮的修改引入的）。diagnose 把 `seconds` 四舍五入到毫秒，逐秒块数却按采样数向下取整。
  - 处理：容许半毫秒的舍入窗口，常量指向 diagnose.py 的 `round(..., 3)`。加了 159999 个采样的边界往返测试。
