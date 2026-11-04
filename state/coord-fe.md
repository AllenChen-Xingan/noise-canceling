# 大总管看板 · noise-canceling-a2

> 本大总管的账本（另一位大总管 5-app-5e 的在 `state/coord.md`，不改它）。检出 `../noise_canceling-coordfe`，分支 `chore/coord-20261104-fe`。交代在主目录 `_tmp/20261104-b13/`。tmux 工作区 `vdn-fe`。
> 占用：批次 b13 · 实验 exp015 · 方案 `docs/plan/013_frontend-once.md`；5-app-5e 已确认（它下一个实验从 exp016 起；最近决策号 D81）。

## 当前大事（2026-11-04 起）

**前端只算一次**（D71）：每帧频谱只算一次，降噪 · 场景 · 修音 · 认人共用。人在 2026-11-04 选「B 前端只算一次」。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| fecore | 共用频谱前端 · #94 · `feat/frontend-once` · `../noise_canceling-fecore` | 频谱前端 · 降噪模型适配 · 场景特征 · 修音频谱级 | 已报到（noise-canceling-fecore-c7） |
| fespk | exp015 认人吃共享频谱 + 耗时分解 · #95 · `exp/015-speaker-shared-spectrum` · `../noise_canceling-fespk` | 认人前端 · 声纹模型（只评）· 认人判断回路 | 已报到（noise-canceling-fespk-45） |

## 进度板

- 2026-11-04 盘点没做完的事：`_tmp/20261104-inventory.md`（主目录）。人同意删 16 个已合并的旧 worktree，已删（硬盘可用 137 → 153 GB）
- 2026-11-04 读源码发现：场景特征与 GTCRN 同为 512 / 256 / sqrt-hann，可直接共用；认人用 Kaldi 400 / 160，且吃降噪后的声音 → 认人单开实验线
- 2026-11-04 名额被 5-app-5e 的成绩单正式跑占着（约 2 h 35 min，heavy.lock）；两人在此期间只读、设计、写代码，不编译不测试。5-app-5e 要求一条 GTCRN 逐样本对拍测试，已写进 fecore 交代
- 2026-11-04 fespk 早报（读源码估算，未实测）：认人每次约 6 ms 里声纹模型推理占约 90–97%，fbank 约 0.1–0.4 ms，共用频谱最多省约 5%。大总管定：fespk 收窄为「实测耗时分解 + 待人拍板」，认人不接共享频谱，不改 src/speaker/；已通知 fecore 不为认人设计输出谱
- 2026-11-04 fecore 设计稿 `docs/plan/013_frontend-once.md`（fecore worktree）大总管点头：STFT 参数正本放 vdn-types `frontend.rs`；借谱接口改为带噪输入功率谱；新增 `vdn_dsp::SpectrumFrontend`；场景加借谱入口。大总管定：SpectralStage 不挪、GTCRN 不带修音槽（接线不在本大事）。名额放开后 fecore 先在未改的 gtcrn.rs 上生成对拍哈希
- 2026-11-04 fespk：exp015 README 估算与三方案写完；计时命令 `vdn-speaker-lab bench-split`（未编译）；判据大总管定「fbank 占 embed p50 < 10% → 共用不值得」；codex 第一轮放行；名额顺序 fecore 对拍哈希 → fespk 实测

## 待人拍板（本大事）

- fespk 3 题（多久判一次 · 一次听多长 · 量化或换小模型实验），见 fespk worktree `state/handoff/fespk.md`；等实测坐实后一起问

