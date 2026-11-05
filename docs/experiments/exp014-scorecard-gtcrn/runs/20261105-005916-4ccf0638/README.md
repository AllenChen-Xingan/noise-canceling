# run 20261105-005916-4ccf0638 · 新规则（v2，人定 D85）全量重跑

exp014 的成绩单按新汇总规则重跑一次：三个系统（原声直通 · 参考降噪 · GTCRN）× 480 段，带评分模型（`--qa`）。issue #111 · #109 · #110。

| 项 | 值 |
|---|---|
| run-id | `20261105-005916-4ccf0638` |
| 规则版本 | `v2`（`--rules v2`；正本 `src/eval/src/scorecard/rules_v2.rs`，标准 `specs/rubric/scorecard.md`） |
| 代码提交 | `4ccf0638`（分支 `feat/scorecard-v2`） |
| 场景集 | `src/eval/suite/scenes.json` · exp013 build `20261103230756-17f5` |
| 段数 | 1440（480 × 3），没跑成的段 0 |
| 耗时 | 9986 s（00:59:15 → 03:45:41，2026-11-05），`caffeinate -i`，供电 `ac`（`meta.power`） |
| GTCRN 权重 | `gtcrn_simple.onnx`，sha256 `b4718df6…`（主目录 `resources/gtcrn/stream/onnx_models/`，经 `VDN_GTCRN_ONNX`） |
| 评分模型 | SCOREQ `adapt_nr_telephone.onnx` sha256 `d493ba8acfe37d0aa0acceb85a41f22405fc4b9a5101359a6363d919307300b4` · Distill-MOS sha256 `b8e2dc186286377ddd5145698446ce949933c356a306a758eb986c0f881d2928` |
| 识别（吃字项） | `qwen3-asr-plus:1.7b`，每段识别 2 次 |

命令（在 worktree 根目录）：

```
caffeinate -i ./target/release/vdn-eval scorecard --suite src/eval/suite/scenes.json \
  --system passthrough,ref,gtcrn --qa --rules v2
```

## 文件

- `scorecard.json` · `scorecard.md`：成绩单（180 个「场景 × 档 × 设备 × 系统」组合）
- `clips.jsonl`：逐段读数；可用 `vdn-eval scorecard --from-clips` 换规则重汇总，不必重跑音频
- `compare.py` → `compare.md`：和旧 run `20261104-000444-e4511abc`（旧规则 v1）逐项对照、B3 改前改后、新项基线
- 音频与评分模型缓存只在主目录 `_tmp/scorecard/20261105-005916-4ccf0638/`，不进仓

## 读数核对（实测）

- 确定性：背景稳不稳（V-BGSTAB）的改前读数 `uncorr_db` 与旧 run 主值逐段相同，三系统各 329 段最大差 0（`compare.md` B3 节）
- 延迟（V-LAT）：三系统读数与旧 run 相同（直通 27.67 ms、参考与 GTCRN 43.67 ms，各 480 段）；旧 run 的 2 个不过组合在新的互相关算法下变为过（#109）

结论见 `../../findings.md` 末节「新规则下的结论」。
