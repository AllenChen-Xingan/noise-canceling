# 三个通用技能扫本项目 · 可优化处（2026-11-02）

> 我（大总管窗口，旁支）用 `single-source-truth` · `complexity-budget` · `verify-first` 三个技能的脚本扫了 `main` 上的 `src/`（commit 87958eb）。只读，没改代码。
> 证据标注：【测】脚本实测 ·【读】读文件 ·【推】推断，未核实。

## 结论

可优化的有 7 处。已在下一件大事（处理图 + schema 交叉校验，`docs/plan/011_schema-crosscheck.md`）里的有 3 处。**011 没覆盖的新发现有 4 处**，已记进 `state/backlog.md`。

| # | 发现 | 技能 | 数 | 已在 011？ | 建议 |
|---|---|---|---|---|---|
| 1 | `vdn-types` 的镜像副本 | 唯一真相源 | 3 个 crate、19 个类型名重复【测】 | 是 | 照 011 删镜像；另加一道「不许新增同名类型」的 CI 棘轮 |
| 2 | 衰减下限 `floor_db` 散在多个文件 | 唯一真相源 | 21 个文件【测】 | 是 | 照 011 |
| 3 | 手写 JSON 没有 schema（评测任务 4 份等） | 唯一真相源 | 4 份【读】 | 是 | 照 011；schema 带版本号 + 内容哈希 |
| 4 | **采样率常量各处自己写** | 唯一真相源 | `16000/48000` 字面量出现在 37 个 `.rs` 文件；至少 6 处各自定义 `const SR` / `IO_RATE` / `VAD_SR`【测】 | **否** | `vdn-types` 导出 `IO_RATE`、`MODEL_RATE` 两个常量，其余引用；测试里的也改引用 |
| 5 | **同名不同物 / 重复小工具** | 唯一真相源 · 一个东西一个名字 | `Verdict` 有 3 个定义（`cli/chain.rs` 是 struct，`cli/live.rs` 和 `eval/lib.rs` 是 enum）；`StopOnDrop` 在 `chain.rs` 和 `live.rs` 各写一份【测】 | **否** | 先判断三个 `Verdict` 是不是同一个意思【推：大概率都是「过线 / 不过线」】；是就统一用 `vdn-eval` 那份，不是就改名；`StopOnDrop` 收进 cli 的一个公共模块 |
| 6 | **评测阈值从文档手抄进 JSON** | 唯一真相源 | `src/eval/suite/ci.json` 自述「阈值凡 012 标〔建议值〕的，这里照抄」【读】 | **否** | 查有没有比对测试【未查】；没有就二选一：JSON 当正本、文档表从 JSON 生成；或加一条测试比对 |
| 7 | **11 个源文件超过 600 行，CI 没有行数门槛** | 降复杂度 | 最大 `src/eval/src/verifiers.rs` 1153 行，其次 `denoised/src/lib.rs` 870、`cli/src/chain.rs` 825、`cli/src/main.rs` 789、`cli/src/live.rs` 750【测】 | **否** | 在 `xtask tidy` 里加行数检查 + 基线棘轮（存量只许变小，新文件守 600）；拆分优先 `verifiers.rs`（按 verifier 种类分文件）和 cli 的 `chain.rs` / `live.rs`（两者有重复代码，见第 5 条） |

## 已经做得好的（`verify-first` 角度）

- CI 两道门都在：`xtask ci` 6 步（fmt · clippy · test · tidy · check-deps · cargo-deny）和 `xtask eval ci`（回归集、冻结基线、防作弊探针）【读 `.github/workflows/ci.yml`】
- 评测分了 CI 层和 QA 层（`src/eval/suite/ci.json`、`qa.json`），并有给 verifier 自测的探针【读】
- 加测试已是规矩（CLAUDE.md 第五节第 6 条）

可补的一处（不急）【推】：人工层（盲听）目前不在任何清单里登记。可以在评测任务清单里加「人工层」条目，写明谁判、怎么判、结果放哪，这样「哪些维度还没有机器判据」一眼可见。

## 复现

```bash
K=~/xingan-chen/B-MyCreate/2-aartifacts/skills
python3 $K/single-source-truth/scripts/find-duplicates.py . --only types
python3 $K/complexity-budget/scripts/check-file-size.py .
grep -rln "floor_db" src | wc -l
grep -rln "16_000\|16000" src | grep '\.rs$' | wc -l
```
