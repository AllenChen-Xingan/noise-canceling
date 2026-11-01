# 现状页 · 手下 prefs（批次 20261031-b4）

分支 `feat/eval-prefs` · worktree `noise_canceling-prefs` · 大总管 `noise-canceling-69`

| 时间 | 做了什么 |
|---|---|
| 10-31 | 报到；读完交代、common、exp003 / exp005 / exp006 README 与答卷、rubric §4、012 verifiers §2/§5、D38–D48 |
| 10-31 | 与 evalimpl（`noise-canceling-evalimpl-7f`）对齐任务格式：用 vdn-eval 的 JSON 清单（`id / set / polarity / origin / real / graders / xfail`），verifier 用名字 id（V-EAT-ASR 等），未知字段会被忽略，溯源字段可加 |
| 10-31 | 发现：exp003 处理后音频（`_tmp/20261030-exp003/`）本机已找不到，只有输入哈希 → 记缺口；exp005 README 写主人 3570，`clips.json` 写 `owner_speaker: 7127`（3570 是 c03 邻座）→ 记缺口，不改别人的文件 |
| 10-31 | `build_dataset.py` → `trials.jsonl`（29 屏）· `pairs.jsonl`（219 对）· `summary.json`；`test_build_dataset.py` 13 项全过（揭盲数与各实验 README 人工汇总逐项一致；exp006 假答卷能直接并入） |
| 10-31 | 发现：exp005 c06、c08 里「激进」与「只有 GTCRN」逐字节相同（c06 无邻座、c08 没锁）；c08 人开会选了其中一份 → 该对标 invalid。exp006 盲听页 12 屏里 10 屏「路由开」=「路由关」同一文件，路由效果只在 te0039 / te0011 / te0028 三屏可测 |
| 10-31 | `make_tasks.py` → `tasks/human-tasks.json`（回归 8 · 能力 12）；verifier id 对 main 注册表（PR #38 已合并，19 个）校验通过，与 ci.json / qa.json 无 id 冲突 |
| 10-31 | 会话因休眠中断后恢复；大总管改名 `noise-canceling-c4`；对应 issue #37 |
| 10-31 | 写完 `scene-defaults.md`（10 行、2 题待人拍板）· `schema.md`（含「同文件配对 = 无效票，不进定标」）· `README.md`（含 exp006 只有 3 屏可测路由）· 规范 `specs/rubric/preferences.md`（K1–K9 · U1–U10 · 偏好↔verifier · 每轮更新流程） |
| 10-31 | 自审：派 3 个对抗视角（数据怀疑派独立重算揭盲 · 证据审计 · 接入派对 vdn-eval 解析器），等结果 |
| 10-31 | Self-review, three viewpoints, findings all verified and fixed. Data skeptic: all 219 pairs recomputed and match; P1: answer to the all-identical trap swallowed by `same` (fixed → `identical#k`). Evidence auditor: 14 items, conservative-version residual range corrected to −0.4 to −3.8 dB; "background retention tracks noise-floor level" downgraded to a hypothesis (K1's clips and c05 share the same noise floor); added the "quietest" answers for c05/c07 (both picked GTCRN-only); K9 → very low; c08 removed as evidence for "after judged Other". Integration: exp006 all-identical trap always judged as passed (fixed), legacy pages have no kind (fixed), the 9 exp003 synthetic-clip tasks can't run in vdn-eval (split into `human-tasks-blocked.json`), missing registry now errors out |
| 10-31 | Final state: self-test 18/18; `build_dataset.py --check`, `make_tasks.py --check` consistent; registry validated against origin/main (19 verifiers, 43 task ids) |

## For human decision
1. "Background retention" for now: A fixed per scene (café −12 dB) · B a "more / less background" knob in "My config", default −12 dB · C wait until the next listening round clarifies what makes someone judge a place "already quiet". Recommend B + C; default A (`docs/experiments/human-prefs/scene-defaults.md` final section)
2. Should the next round specifically test background retention (−6 / −12 / −20 / −30 dB, 4 clips × 4 levels, plus the question "did this place already feel quiet to you")? Recommend A (test it), about 10 minutes; default B

## For the coordinator (not this line's files, not changed)
- `docs/experiments/exp006-scene-sim/unblind.py` has the same name-collision bug: in the all-identical trap, whichever slot the person picks gets unblinded as `same` and judged "passed"
- `state/pending-decisions.md`: please add the two decisions above
- exp005 README says owner 3570; `clips.json` says 7127

**Done**
