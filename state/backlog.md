# 以后再办（积压清单）

> 大总管一次只做好一件大事，每个手下只管 2–4 个实体（CLAUDE.md 第五节第 4 条）。不属于当前大事的事按**实体**分组记在这里；换入某个实体时把它下面的事一起取走，打勾写去向，**不删**。
> 要人拍板的题不放这里，放 `state/pending-decisions.md`。

## 虚拟麦克风（HAL 插件）

- [ ] **dev-4 实时管线 + Rust 虚拟麦克风**：第一步是 D05 空插件加载与崩溃实验（D54 · D55）；要人输密码、断音几秒 · 出处 `docs/plan/001_decisions-20261029-b1.md` D54 / D55

## 模型

- [ ] **查 Silero VAD 能否纯 Rust、实时线程零分配地跑，能就替换 earshot**（D59，发布阻断）· 出处 `docs/plan/009_model-map.md` 第二节

## 认人（声纹 · 判断回路）

- [ ] **认人减负办法**（目标 1.2% → 0.15%，约 8 倍）：①锁定后降频 0.5 s → 2 s（插话晚约 1.5 s 被压，**要人定**）②只在有人声时判 ③声纹模型换更快后端（已并入 019）④更小声纹 / D47 辅助小模型逐帧判、大模型偶尔确认 ⑤**共用频谱前端（人选，D71，归 47 的 Rust 骨架）**
- [ ] **⚠️ CPU 超线（D69）**：认人判断回路约 1.2% 核（exp005 README:155，每 0.5 s 一次、每次约 6 ms），D69 整机 ≤ 1%、019 分摊给认人 0.15% → 要降约 8 倍（降频 · 换小模型 · 只在说话段跑等）· 出处 019 `standard.md` §6.1（engine worktree）
- [ ] **STFT · 重采样 · 采集输出 · 修音 四项 CPU 都没量过**（019 分摊合计 0.17%）

## 人耳（评测器）

- [ ] **下一轮盲听**：①多留背景（下限 −6 dB）的两组红线通过率最高、评分模型却最低——评分模型判不了，要人耳（018 exp011 results.md 第 4 节）②夹 4–6 道众人锚点题（D66）③016 4-9 第 3 题「覆盖 ≥ 8 个系统」一起排 ④**放进 DFN3 `_ll`**，检验「GTCRN 够不够好」（人 2026-11-02「好」：GTCRN 第三方评测少，主模型选择当假设继续验证）· 出处 018 · 017 · 016 · D33 · D49

- [ ] **NoNoise-Mac 进评测当对照组**（D77）：先看能否离线跑（CLI / 直接调 DFN3 Core ML），不能就装驱动回录；与本项目同场景比吃字率 · 评分 · 延迟 · 耗电；可选加 MetalVoice、Krisp（只对照不进 CI）· ⚠️ 装第三方驱动要人同意 · 出处 `docs/reviews/outlook-codex-20261102/research.md`

## 数据

- [ ] **exp008 第②段续跑**：FRCRN（179/348）· MossFormerGAN · URGENT-BSRNN · GMAP-SE 断点续跑再打分，回填 016 mvp 2.3；补充，不改结论 · 出处 016 现状页「转交」
- [ ] **LibriSpeech dev-clean + train-clean-100（约 6.6 GB）下载**：挂在待拍板 4-9「换素材池」与 D65 触发条件上

- [ ] **⏰ 触发：要开始训练、或要用真实原料合成时** → 先全量下载公开真实数据集（约 40 GB，解压峰值约 80 GB，不含 ECHI / CHiME-4），当场按说话人 / 场次切测试份与训练份，切法与哈希进仓；DNS · ACE · DR-VCTK 许可核清前只评测 · 出处 D63 · D64 · D65，清单见 `docs/research/014_real-datasets/decision.md` 第一节
- [ ] **DEMAND 署名**：按 D64，SA 与 CC BY 都可训练，场景小模型不必因许可重训；发布模型卡时写明 DEMAND 来源与许可（exp006 README 第 34 行写 CC BY 4.0，015-5 写 BY-SA，以 Zenodo 原页为准）· 出处 D63 · D64
- [ ] 016 benchsynth 交代补一条：合成原料可用公开集训练份，与测试份按说话人 / 场次隔离（D63）

## 仓库与工作区

- [ ] **清理**：删已合并的 worktree `../noise_canceling-prefs` · `../noise_canceling-benchsynth`（016，PR #47）· `../noise_canceling-ears`（017，PR #48）· `../noise_canceling-judge-loop`（018，PR #49）；删 `../noise_canceling-dfnhush`（015 已随 PR #40 合并，D61）；本地 main 同步到远端（落后 77）；推送 `chore/coord-20261031-b6`；`_tmp/` 8.3 GB 看哪些能删

## 引擎（engine）

- [ ] **输入 NaN/Inf 时输出 NaN 且 seam 永不恢复**（issue #53，读源码确认、未复现）· 已转 47 的 Rust 骨架线
- [ ] **vdn-review 工作流不认 args 里的审查对象**（applebe 给 019 文件，它审了 src/）· 已告 47

- [x] → 2026-11-02 换入当前大事（b8 applebe / rustbe）**019 推理引擎对比续做**（issue #45 · worktree `../noise_canceling-engine` · 2026-10-31 会话中断，只写了部分笔记）：不在当前大事内，以后单开一件大事（「轻快」那把尺子）

- [ ] **修 engine 停顿门控误开**（GTCRN 纯噪声停顿门控开 0.99；验收 = R17 由 xfail 转通过）· 出处 board 第四批「下一批建议 ②」
- [ ] **接入线**：p2 认人 / scene 路由接进 engine · 出处 board 第四批「下一批建议 ①」

## 待拍板清单

- [ ] **待拍板清单瘦身**：约 35 题按实体重新分组，过时的（BlackHole 相关）划掉 · 出处 `state/pending-decisions.md`
