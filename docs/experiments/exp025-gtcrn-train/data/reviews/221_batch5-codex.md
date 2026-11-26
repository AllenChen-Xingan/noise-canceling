# 评审 221 · exp025 第五批训练数据（Room112 单音补测后换来源 -b5 · 冻结旧证据页）（codex · gpt-6-astra · read-only）

> 对象：`docs/experiments/exp025-gtcrn-train/data/` 下 screen_tone.py · tests/fixtures_pages.py · tests/test_screen_splice.py · tests/test_but_real.py · tests/test_build_evidence.py · split_rules.json · recipe.json · scene_notes.json · scene-gaps.md · cloud-batch5.md（feat/data-batch5，PR Closes #221）。

## 第 1 轮（2026-11-26）· 总评「修改」

原文：主仓 `_tmp/reviews-traindata-b5/data-batch5-codex-1.md`（不入库）。effort medium，耗时 57 秒。
本轮 P0/P1：0 条 · 停止线：连续第 1 轮无 P0/P1

- **P2｜screen_tone.py:387 不匹配行并非逐字节带过**（第 1 轮 #1）
  - 核实：属实（读源码）。现有旧表格式恰好相同，所以真文件不受影响（`--check` 一直一致）。
  - 处理：pi traindata-b5codex 改为按原文写回不匹配的行；加了不同空白、键序、`\u` 转义的测试。真文件 `--check` 仍逐字节一致（traindata 亲手跑）。
- **P2｜tests/test_build_evidence.py:350 「未建表」豁免过宽**（第 1 轮 #2）
  - 核实：属实（读源码）。
  - 处理：同上 pi。当前配方已有 build 时，两页都必须在、都逐字节重算，不许豁免；没有 build 时，只有当前配方真用到的来源在「未建表」里才豁免。加了无关来源不豁免、有 build 缺页即红两条测试。
- **P2｜cloud-batch5.md:25 进目录后 python 相对路径失效、失败被掩盖**（第 1 轮 #3）
  - 核实：属实。
  - 处理：traindata 改为 `PY=$PWD/.venv/bin/python`、`set -o pipefail`，测试失败即停（364653a6）。第五批已跑完，cloudrun 当时自己处理了路径；改动只影响以后照抄这份的人。

### 引用抽查

- codex 列的「核实属实」6 条，「不符」无。「无法核实」2 条：两张新页重算（由 test_build_evidence / test_scene_table 覆盖，本机全量 494 条通过）；旧文件历史字节不变（traindata 用 `git diff feat/data-batch5` 核过为空，冻结页的 sha256 等于 main a6bb1ced 上那两份）。
