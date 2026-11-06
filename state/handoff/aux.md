# 交接 · 手下 aux（调研 030 · #124 · 实验 exp021）

> 2026-11-05 · 分支 `research/030-gtcrn-aux` · worktree `../noise_canceling-aux`

## 做完了什么

- 结论：`docs/research/030_gtcrn-aux/decision.md`（第一节比较卡）· `README.md` · `sources.md`
- 实验：`docs/experiments/exp021-gtcrn-aux/`
  - `driver/`：重混（真引擎，只换混回参数）· 生成 GTCRN 原始输出（带预热段）· 派生关保命测试集 · 诊断。Rust 测试 7 条
  - `analyze.py`（核对 · 配对差 · 按场景挑上限 · 平均）· `plot.py`（取舍曲线 SVG）· Python 测试 13 条 · 复用核对测试 9 条（`test_sweep_reuse.sh`）
  - `sweep.sh`：一个模型扫 8 档 + 不混回
  - `runs/`：DFN3 核对 · GTCRN 核对（补零版不过、带预热段版过）· 口径差 · 两次扫描

## 没做 / 留给后面

- 听感分（Distill-MOS · SCOREQ）没测：比较卡第一个最小实验要带评分模型重测（约 77 分钟 × 2，要名额）
- 「说话时 −9 · 停顿 −9」组合没扫，叠加效果是推断
- 判场景小模型：只查了已定 / 已做，没训练（缺数据，D87 · D88）
- 外部入口预热段补零的口径差：大总管已开 #129；hook 第 131 行的问题：#133

## 复现要点

- 音频都在主目录 `_tmp/030/`（不进仓）；GTCRN 权重用 `VDN_GTCRN_ONNX` 指向主目录 `resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`
- 内置模型做重混必须带预热段（`raw` 写的 `<out>.head`，`remix --head`），否则降噪量差约 0.1 dB
- zsh 里别用不加引号的变量传多个参数（不会拆开）
- 超过 10 MB 的 `clips.jsonl` 用 gzip 压缩后入库，`analyze.py` / `plot.py` 能直接读 `.gz`
