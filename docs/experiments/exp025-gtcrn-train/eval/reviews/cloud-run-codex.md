# codex 评审记录：评测上云执行单 `eval/cloud-run.md`（PR #230）

评审方：codex，模型 gpt-6-astra，只读沙箱，effort medium，类型 doc。作者 evalcmp 逐条回源码核实后改。

## 第 1 轮（2026-11-26）· P0×0 P1×5 P2×1 · 总评：修改

| # | 级 | 问题 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | ONNX 放在 `/workspace/eval-formal/onnx/`，离线脚本的 `rel()` 只认工作副本或主目录里的路径，连探针都过不了 | 属实（`run_gtcrn_offline.py` `rel()`） | ONNX 改放仓库内 `$REPO/_tmp/eval-formal/onnx/`，并写明软链接也不行 |
| 2 | P1 | 第 6 步把 `R` 改成成绩单目录，下一个模型 `cd $R` 就进错目录；`LEDGER` 没定义 | 属实 | 固定变量块：`REPO` · `RUNS` · `LEDGER` 等分开命名，每步显式 `cd $REPO` |
| 3 | P1 | 1.2 美元只靠「每 30 秒看账本」，没有可执行的命令，跨机的批次起点也没存 | 属实（程序只有 15 美元一道闸，`asr_or.rs` `BUDGET_USD`） | 加闸门脚本 `gate.py`（随每份成绩单启动、越线 `pkill`）；批次起点存在网络盘 `batch-start.json`，机 2 沿用；写明最多越线约 0.01 美元（推断） |
| 4 | P1 | 去掉了 `eval-plan.md` 要求的「提交变了就重跑内置原版」回归检查 | 属实 | 恢复为第 1.5 步：先比 ab0f4f0a..EVAL_COMMIT 的 `src/` 改动，只动了 `accept` 才跳过，否则重跑内置原版并用 `parity.py` 对拍，过了才往下 |
| 5 | P1 | 缓存复制失败仍会启动付费任务；缓存键在甲路是否相同未核 | 属实（前半）；缓存键已读源码核实：键是送识别音频的 sha256，干净人声 / 原声取自清单（`qa/mod.rs:113`、`153–154`），与系统无关 | `set -euo pipefail` + 每片 `test -f` · `cmp`，失败就在启动前停 |
| 6 | P2 | 噪声对循环靠 `set -- $p` 分词，在 zsh 下会拆错 | 属实 | 改成 `0-1` 形式加 `${p%-*}` / `${p#*-}`；全部命令块注明用 bash；10 个块 `bash -n` 通过 |
