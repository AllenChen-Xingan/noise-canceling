# 大总管看板 · noise-canceling-a2

> 本大总管的账本（另一位大总管 5-app-5e 的在 `state/coord.md`，不改它）。检出 `../noise_canceling-coordfe`，分支 `chore/coord-20261104-fe`。交代在主目录 `_tmp/20261104-b13/`。tmux 工作区 `vdn-fe`。
> 占用：批次 b13 · 实验 exp015 · 方案 `docs/plan/013_frontend-once.md`；5-app-5e 已确认（它下一个实验从 exp016 起；最近决策号 D81）。

## 当前大事（2026-11-04 起）

**前端只算一次**（D71）：每帧频谱只算一次，降噪 · 场景 · 修音 · 认人共用。人在 2026-11-04 选「B 前端只算一次」。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| fecore | 共用频谱前端 · #94 · `feat/frontend-once` · `../noise_canceling-fecore` | 频谱前端 · 降噪模型适配 · 场景特征 · 修音频谱级 | 完成，验收过（大总管亲手跑 xtask ci 6 项全绿 · 对拍 4 份逐位相同含引擎最终输出 · 路径清零 · 文件 ≤ 557 行），PR #99 已开（合并冲突已解），窗口已关 |
| fespk | exp015 认人吃共享频谱 + 耗时分解 · #95 · `exp/015-speaker-shared-spectrum` · `../noise_canceling-fespk` | 认人前端 · 声纹模型（只评）· 认人判断回路 | 完成，验收过（大总管复跑 11 测试 · 9 文件在范围内 · src/speaker/ 未改 · 路径清零），PR #96 已开，窗口已关 |
| pse | 调研 024 个性化降噪：有没有人做过 · 为什么难 · #98 · `research/024-pse-landscape` · `../noise_canceling-pse`（人点名派，属「个性化降噪」大事的前置调研） | 个性化降噪 / 提取模型 · 自动注册 · PSE 训练数据与评测集 | 已报到（noise-canceling-pse-8b） |

## 进度板

- 2026-11-04 盘点没做完的事：`_tmp/20261104-inventory.md`（主目录）。人同意删 16 个已合并的旧 worktree，已删（硬盘可用 137 → 153 GB）
- 2026-11-04 读源码发现：场景特征与 GTCRN 同为 512 / 256 / sqrt-hann，可直接共用；认人用 Kaldi 400 / 160，且吃降噪后的声音 → 认人单开实验线
- 2026-11-04 名额被 5-app-5e 的成绩单正式跑占着（约 2 h 35 min，heavy.lock）；两人在此期间只读、设计、写代码，不编译不测试。5-app-5e 要求一条 GTCRN 逐样本对拍测试，已写进 fecore 交代
- 2026-11-04 fespk 早报（读源码估算，未实测）：认人每次约 6 ms 里声纹模型推理占约 90–97%，fbank 约 0.1–0.4 ms，共用频谱最多省约 5%。大总管定：fespk 收窄为「实测耗时分解 + 待人拍板」，认人不接共享频谱，不改 src/speaker/；已通知 fecore 不为认人设计输出谱
- 2026-11-04 fecore 设计稿 `docs/plan/013_frontend-once.md`（fecore worktree）大总管点头：STFT 参数正本放 vdn-types `frontend.rs`；借谱接口改为带噪输入功率谱；新增 `vdn_dsp::SpectrumFrontend`；场景加借谱入口。大总管定：SpectralStage 不挪、GTCRN 不带修音槽（接线不在本大事）。名额放开后 fecore 先在未改的 gtcrn.rs 上生成对拍哈希
- 2026-11-04 fespk：exp015 README 估算与三方案写完；计时命令 `vdn-speaker-lab bench-split`（未编译）；判据大总管定「fbank 占 embed p50 < 10% → 共用不值得」；codex 第一轮放行；名额顺序 fecore 对拍哈希 → fespk 实测
- 2026-11-04 fecore：代码与测试写完未编译；基准用 git archive 990e7843 + 单独 target；授权改 `specs/rust/architecture.md` 的 SpectrumTap 说明。fespk：codex 3 轮到停止线（8 条全改）；授权只读 exp005 降噪输出、追加 reviews 索引一行。两条线都等名额；已告 5-app-5e 归还时通知
- 2026-11-04 03:14 a2 从 5-app-5e 接名额、建 heavy.lock。fecore ① 对拍基准生成（990e7843，场景 + GTCRN 两份，约 1 min）。fespk 实测（verifier `vdn-speaker-lab bench-split`，插电 · caffeinate · 158 窗 × 6 遍 = 948 样本 × 2 段）：fbank 占 embed p50 5.2%（阈值 < 10%）→ 过线，认人不共用成立；fbank p50 约 160 µs、推理约 2.9 ms；一次声纹计算约 3.05 ms，只有 exp005 记的 6 ms 一半，原因待查（转交）。fecore ② 各 crate 测试在跑
- 2026-11-04 fespk 收尾：codex 4 轮（2–4 轮无 P0/P1）；PR #96（Closes #95）。转交：声纹计算实测 3.05 ms vs exp005 记 6 ms，原因待查 → backlog「认人」
- 2026-11-04 #96 CI 红（fmt · clippy · tidy 采样率字面量，都在 bench.rs）：fespk 只跑了单 crate 测试、大总管验收也没跑 xtask ci。子代理修复，xtask ci 6 项全绿（大总管借名额约 10 min）。**教训：验收一律亲手跑 xtask ci，不只跑单 crate 测试**
- 2026-11-04 **人改定 D82**：「都要做，认人也要做，所有频谱计算都放在一个地方，这个不是省力的问题」；选「同一处代码，各自切法」。exp015 README 已加说明。已告 fecore（模块设计要容纳不同分帧）
- **第二段（待派）**：认人 fbank · 评测器 FFT（`src/eval/src/verifiers/estoi.rs` · `speech.rs`，eval 归 5-app-5e 在用，派前先商量）· 离线例子，收进统一模块；基于 fecore 分支
- 2026-11-04 人：「不能走ort，ort太慢了」（针对认人声纹模型 ECAPA512 走 ORT）；人定「先记下，以后再定」→ 已记 `state/backlog.md`「推理后端：去掉 ORT」
- 2026-11-04 人定 D83：放弃门控，改为先认人再提取（个性化降噪），倾向自训；人点名「派一个手下查一下……到底有没有人做过，为啥可能很难」→ 派 pse（调研 024，#98，已告 5-app-5e 占 024）。fespk 那 3 道待拍板题随门控作废
- 2026-11-04 fecore 收尾：codex 3 轮（2、3 轮无 P0/P1，P2×7 全修）；对拍 4 份（场景整 hop 24 次 · 场景不整齐 31 次 · GTCRN 47872 点 · 引擎最终输出 144000 点）基准均在 990e7843 生成，改后逐位相同；5-app-5e 复核同意 exp014 仍可比。每帧耗时：场景段中位 2.42 → 2.00 µs，合计在噪声内。大总管验收：亲手跑 xtask ci 全绿；补 reviews 索引；解 docs/reviews/README.md 合并冲突（两行都留）
- fecore 转交 → **第二段**（D82）：认人 fbank · 评测器 FFT（estoi · speech · system.rs 参考降噪器手写 512/256）· 离线例子收进同一模块，接口见 `docs/plan/013_frontend-once.md` 第 11 节（StftSpec 加 win_len · Window 加枚举 · 每帧预处理钩子 · f32/f64 通用 · 每处逐位对拍）；`src/models/Cargo.toml` dev 依赖 vdn-scene 用 path 写法 → 根 workspace.dependencies 补一项
- fecore 转交 → backlog「文档」：src/README.md 仍是模板 · 模块说明内部编号多 · A/B/C 档无定义 · 「档」一词三义 · L0–L3 无逐级说明 · types/src/lib.rs 表头三块表里四行 · 帧长 10/16 ms 不一致

## 待人拍板（本大事）

- fespk 3 题（多久判一次 · 一次听多长 · 量化或换小模型实验），见 fespk worktree `state/handoff/fespk.md`；等实测坐实后一起问

