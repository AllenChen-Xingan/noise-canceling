# 现状页 · traindata（issue #156，批次 20261116-b19）

状态：进行中（2026-11-24 的三件活已做完，待收尾自审与 codex 评审）

## 手上
实体：训练数据集 · 数据切分 · 混音配方。产出都在 `docs/experiments/exp025-gtcrn-train/data/`，说明见那里的 `README.md`。

## 做到哪
混音已改成 GPU 批量做：CPU 只出 plan（读片段 + 抽随机数），`gpu_mix.render_batch` 在 GPU 上混。CPU 版留作对拍。
同一批 plan 两边逐样本差 ≤ 1e-4，写成了测试。本机 75 条测试全过（torch CPU 后端）。下载脚本已备好，没在云上跑。

## 下一步
1. 等大总管放行：云上 `bash cloud_steps.sh setup → test → head → fetch`
2. 下完：`split`（先看 QUT / TUM 的清单）→ `shard` → `register`；切分表与 build.json 拷回进仓
3. exp013 重建：`bash cloud_steps.sh rebuild`
4. 收尾：多视角自审 + codex 评审

## 依赖
- 早期混响 100 ms 的正本在 `docs/experiments/exp013-scene-suite/rooms.py` 的 `EARLY_REVERB_MS` 与 `early_rir`。它们在 evalcmp 的分支 `feat/eval-paired-compare` 上，还没合进 main。合并前本分支的混音测试会红；本机验证用的是那个分支的 rooms.py
- 那个分支合并后，`test_rebuild.test_code_unchanged_since_build` 可能会红：exp013 的 rooms.py 变了，要在云上确认旧的 4,540 个文件仍逐位一致

## 待人拍板
（暂无）

## 发现
- QUT-NOISE 的服务器没声明支持断点续传（HEAD 响应里没有 `Accept-Ranges`）。脚本遇到这种情况会从头重下
- 官方 SEtrain 混音代码 `alpha = 10**(-snr*1.5/20)`：它的实际信噪比是名义的 1.5 倍（−7.5～22.5 dB）。本配方按真实信噪比取，已写进 `recipe.json` 的出处

## 流水
- 2026-11-16 报到；读 014 · 027 · 034 · exp013 的 leak.py / build.py / pools.py / rirs.py
- 2026-11-16 用 HEAD 请求核下载地址与 md5：OpenSLR 12 / 28 · QUT-NOISE 6 个 zip · TUM Zenodo · DEMAND Zenodo · Commons 3 个文件
- 2026-11-16 和 evalcmp 定早期混响正本（rooms.py）；和 trainer 定加载接口（区间由我给，掩码由 trainer 做；第 k 批只由 seed 与 k 决定）
- 2026-11-16 记下 exp013 原料 1,994 个文件的 sha256（`exp013_inputs.json`），原料索引和正式 build 一致
- 2026-11-16 写 sources.json · fetch.py · split_rules.json · split.py · leakcheck.py · recipe.json · mix.py · shard.py · loader.py · rebuild_exp013.py，各带测试
- 2026-11-24 恢复。按 D100 / D102：混音拆成 plan + render；新写 gpu_mix.py；加对拍测试（GPU 版对 CPU 版 ≤ 1e-4，电平与区间对 exp013 dsp.py 逐个相同）
- 2026-11-24 loader 改成 worker 只出 plan_batch，主进程调 `render_on_gpu`；DataLoader 用 2 个 worker 加 start_batch 测过
- 2026-11-24 fetch.py 加了两样：开下前先查盘（含解压峰值），以及 `--head`（只核大小，不下载）。本机发 HEAD 核了 A3，大小一致
- 2026-11-24 写 requirements.txt（版本钉死，和 exp013 旧 build 一致）· cloud_steps.sh · data/README.md · exp025 README 的数据一节
- 2026-11-24 建本 worktree 的 `.venv`（已被 git 忽略）：torch 2.15.0 · numpy 2.5.4 · scipy 1.18.1 · soundfile 0.14.0
