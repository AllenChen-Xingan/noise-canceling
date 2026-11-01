# human-prefs · 人耳偏好数据集 · eval 任务 · 分场景默认

> 意图：人原话「所以说，让一个手下吧一些测试好的东西也变成eval或者之类的……，或者偏好什么的吧 / 以及让大家都同步」。D44（verifier 用人耳定标）· D46（验收只看 verifier）· D48（虚拟听众）· D40 / D38（场景路由 · 我的配置）。issue #37。
> 只做数据整理与统计：**不跑模型、不读音频、不占重活名额**。音频不进仓，只记 sha256 与生成方式。

## 局限（先读）

**一个人 · 合成素材 · 带人原话的偏好各 1–3 屏、全部只来自一个会话 · exp003 会话打折（有 C 位偏好）· exp005 有一票是在两份相同文件里选的（已标无效）。** 这里的任何数字都不是效果结论（D46），只是 verifier 定标与下一轮听测的起点。完整列表：`specs/rubric/preferences.md` 第 0 节。

## 文件

| 文件 | 是什么 | 怎么来 |
|---|---|---|
| `build_dataset.py` | 揭盲各实验答卷、统一格式 | 读 `docs/experiments/exp00{3,5,6}-*/results/` 与 `builds/` |
| `trials.jsonl` | 每屏一条（29 条：正式 22 · 检验 3 · 旧 build 无法揭盲 4） | 生成，别手改 |
| `pairs.jsonl` | 两两比较（219 对，其中 1 对无效票） | 生成，别手改 |
| `summary.json` | 计数、会话有效性、位置分布 | 生成，别手改 |
| `schema.md` | 字段说明 · 硬规则 · 每条记录喂给哪个 verifier / 虚拟听众 | 手写 |
| `test_build_dataset.py` | 自测 13 项：揭盲数与各实验 README 人工汇总逐项一致；exp006 假答卷能并入；同文件配对标无效 | 手写 |
| `make_tasks.py` | 从 `trials.jsonl` 生成 eval 任务，并对 verifier 注册表校验 | 手写 |
| `tasks/human-tasks.json` | 20 个任务（回归 8 · 能力 12），vdn-eval 清单格式 | 生成，别手改 |
| `scene-defaults.md` | 分场景默认参数初稿（每个数字带出处、票数、置信度） | 手写 |
| 规范 `specs/rubric/preferences.md` | 已知偏好 K1–K9 · 未知 U1–U10 · 偏好 ↔ verifier · 每轮更新流程 | 手写 |

## 怎么跑

```sh
python3 docs/experiments/human-prefs/build_dataset.py          # 重建 trials / pairs / summary
python3 docs/experiments/human-prefs/test_build_dataset.py     # 自测
python3 docs/experiments/human-prefs/make_tasks.py             # 重建任务清单并校验（注册表默认 src/eval/src/verifiers.rs）
python3 docs/experiments/human-prefs/build_dataset.py --check  # CI 用：输出与仓里不一致则退出码 1
python3 docs/experiments/human-prefs/make_tasks.py --check
```

纯 Python 标准库，确定性输出。

## 数据集现在有什么

| 会话 | 有效性 | 正式屏 | 检验题 | 备注 | 问法 |
|---|---|---|---|---|---|
| exp003 `20261030102301-68ab`（旧，两候选） | 未验证（中止、揭盲表丢失） | 2 可揭盲 + 4 不可 | 0 | 0 | 最清楚 · 最自然 · 吃字 · 开会评级 · 标签 |
| exp003 `20261030110755-f8c3`（三候选） | **打折** | 12 | 2（全同没过 · 劣化过） | 3 | 同上 |
| exp005 `20261030224448-ab48`（个性化三版） | 有效（只有一道检验题） | 8 | 1（过） | 8 | 最安静 · 吃字 / 变轻 · 最自然 · 开会选 |
| exp006 `20261030235644-6a99`（场景 4 候选） | 答卷未交 | — | — | — | 最清楚 · 最自然 · 吃字 · 开会评级 · 标签 |

## exp006 答卷到了怎么并

1. 人导出的 JSON 原样放进 `docs/experiments/exp006-scene-sim/results/` 并提交
2. `python3 docs/experiments/human-prefs/build_dataset.py`——自动发现，不用改代码（适配器 `exp006()` 已用假答卷测过）；再跑 `test_build_dataset.py`、`make_tasks.py`
3. **读结果注意**：12 屏里 10 屏「路由开」（`route`）与「路由关」（`base`）是同一文件，路由效果**只在 te0039 · te0011 · te0028 三屏可测**；那 10 屏的 `base`–`route` 配对在 `pairs.jsonl` 里 `identical_audio = true`，判平是一致性检验，分出胜负会自动标 `invalid`。`polish` 与 `both` 在 12 屏里都不同
4. 按 `specs/rubric/preferences.md` 第 4 节更新偏好与 `scene-defaults.md`

## eval 任务（`tasks/human-tasks.json`）

- 格式与 evalimpl 对齐（`docs/experiments/exp007-eval-suite/task-schema.md`，PR #38 已合并）：`id / set / polarity / origin / real / graders / info / xfail`；verifier 用注册表名字 id。另加三个 vdn-eval 会忽略的溯源字段：`human`（被判的系统、每屏的人判结论与原话、被听那份音频的 sha256）· `threshold_basis`（过线值出处）· `repro`（输入 / 输出怎么复现）
- id 段：回归 `R60–R67`、能力 `C70–C81`（`-h-` = human），与 `src/eval/suite/ci.json`（R01–R17 · C01–C10 · P01–P05）、`qa.json`（Q01–Q11）不冲突；`make_tasks.py` 每次都查
- **回归**：人判「能用 / 最自然」的输出不许变差（exp003 c09 · c10 · c08 · c01；exp005 c06 · c04 · c02 · c01）
- **能力**：人判「不能用 / 吃字 / 别人太吵」，带 `xfail`（exp003 c02–c07 · c11 · c12；exp005 c03 · c05 · c07 · c08）
- 过线值只用 verifier 注册表锚点〔建议值〕或已有实测（R67 的 −6.5 dB 来自 exp005 README §3），不是人定的线
- 接入：evalimpl 合并后（已合并）由 vdn-eval 加载本文件或让 `parse_suite` 读 `tasks/`——这一步属于下一批，不在本线改 `src/eval/`

## 缺口

1. **exp003 被听的输出音频没了**：`_tmp/20261030-exp003/`（abtest worktree 下）本机已找不到，build 的 `files.json` 也没记 sha256。输入有哈希（c01–c03 `noisy_sha256` / `clean_sha256`；c04–c12 由 `make_clips.py` 种子 20261030 合成），输出要重跑 `enhance.py`（需要模型推理，占名额），重跑结果是否与人听的逐字节相同**未核**
2. **exp005 任务测不到被判的系统**：人判的是保守版 / 激进版（vdn-speaker-lab 的个性化管线），vdn-eval `--system` 现在只有 gtcrn / fastenhancer-m；人听的是锁定后 25 s 节选，vdn-eval 跑整段
3. **exp006 不能当 `real` 任务**：vdn-eval 只认 `real.kind` = exp003 / exp005；exp006 答卷到了要先给 vdn-eval 加 `exp006` 来源
4. **`vdn-eval calib` 只读 exp003 格式答卷 + `scores.json`**：exp005 / exp006 要先转格式（转换器未写），且 exp005 没有 `per_clip[clip][model]` 的指标表
5. **「留一点背景」没有 verifier**（K1）：见 `specs/rubric/preferences.md` 第 3 节
6. exp005 README 写「主人 = 说话人 3570」，`clips.json` 写 `owner_speaker: 7127`（3570 出现在 c03 的邻座列表里）——两处不一致，未改（不是本线的文件），数据集只用 `clips.json`
7. exp005 导出没有题序和每题时间戳，`trial_order` 用的是答卷 JSON 键序

## 给各会话的同步句（交大总管放进 common-rules 模板 / CLAUDE.md）

> 开工必读 `specs/rubric/preferences.md` 与 `state/pending-decisions.md`：前者是人耳偏好（已知 · 未知 · 对应 verifier），后者是待人拍板的题。改参数、写 verifier、做路由前先对照；每次听测交回答卷后按 `preferences.md` 第 4 节更新。

## 待人拍板

见 `scene-defaults.md` 末节（2 题：背景保留量按底噪连续变化还是按场景固定 · 下一轮是否专测背景保留量）。
