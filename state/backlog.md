# 以后再办（积压清单）

> 大总管一次只做好一件大事，每个手下只管 2–4 个实体（CLAUDE.md 第五节第 4 条）。不属于当前大事的事按**实体**分组记在这里；换入某个实体时把它下面的事一起取走，打勾写去向，**不删**。
> 要人拍板的题不放这里，放 `state/pending-decisions.md`。

## 处理链的统一表示（下一件大事，D72 · 011）

- [ ] **处理图 + schema 交叉校验 + 前端只算一次（D71）**：STFT / fbank / 场景特征每帧只算一次，降噪 · 认人 · 场景 · VAD 共用（规约 §3.1 `SpectrumTap` 已留口）；同一功能只在一处实现；处理链表示成「节点带类型 · 参数 · 版本」的图，会话记录同一套表示；Rust 类型为源头导出 schema、CI 比对；清掉 speaker / polish / scene 的 `vdn-types` 镜像副本（12 处）与手写 JSON；后处理级非有限检查与 Engine classify 规则合成一处；非有限值检查在 Pipeline 与送音端各一处（进程边界，有意）统一口径 · 出处 `docs/plan/011_schema-crosscheck.md` · Rust 骨架审查 `docs/plan/010_rust-skeleton-audit.md`

- [x] **三技能扫描的 4 处新发现（011 未覆盖）** → 去向：#62 `feat/code-hygiene`（手下 hygiene，2026-11-02 派）；镜像副本部分已由 #60 清掉，建议并进这件大事一起做：①采样率常量收进 `vdn-types`（`16000/48000` 散在 37 个文件）②三个 `Verdict` 同名不同物、`StopOnDrop` 两份 ③`eval/suite/ci.json` 阈值手抄自 012，缺比对 ④`xtask tidy` 加 600 行上限 + 基线棘轮（现有 11 个文件超线，最大 `eval/src/verifiers.rs` 1153 行）· 出处 `docs/reviews/three-skills-scan-20261102/README.md`

## Rust 骨架的遗留（b8-rust 批次）

- [x] ~~空闲时推理块约 3 ms：追性能核 / 提频、压 lead 到 144~~ → 人定 D74 用低频，lead 留 4 ms
- [ ] **资源判据按 D75 改**（e3 转人定：「我觉得是实际耗电和对性能的影响」）：链路 verifier 加「功耗 mW（powermetrics，人输密码或免密规则）+ 对其他程序性能的影响」，CPU% 只作参考；现有数据在 `docs/experiments/exp-v7-latency/results/20261102-power/`
- [ ] 共享内存权限 0600 与 coreaudiod 下的虚拟麦克风能否打开（装驱动那批实测；补 009 待拍板 3）
- [ ] 虚拟麦克风真驱动接口 + 安装（D05，要人输密码、断音几秒）；构建部署目标 macOS 15（D78，改 D73）
- [ ] 麦克风流无缝重接（回调里换设备，不断声）
- [ ] `vdn-eval` / `vdn-speaker` / `vdn-speaker-lab` 改走推理后端，不再直接依赖 ort
- [ ] ContentId 用 FNV → 规约原写 sha256
- [ ] RTSan 检查（规约 G8）
- [ ] 删掉只为非 macOS 存在的分支（`vdn-shm` 的 POSIX 替身与轮询兜底里的非 macOS 部分、`vdn-rt-thread` 的「本平台不支持」分支）——D76 后不再被编译
- [ ] `.claude/workflows/` 的 vdn-review 不认 args 指定的审查对象（e3 报，给 019 文件它审了 src/）
- [ ] `vdn-rt-thread` 的 JoinToken 常数照 SDK 头文件抄，缺自动比对（唯一真相源）
- [ ] Ring 拆 Producer / Consumer 句柄（类型挡住「一端一线程」）

## 虚拟麦克风（HAL 插件）

- [ ] **dev-4 实时管线 + Rust 虚拟麦克风**：第一步是 D05 空插件加载与崩溃实验（D54 · D55）；要人输密码、断音几秒 · 出处 `docs/plan/001_decisions-20261029-b1.md` D54 / D55

## 模型

- [ ] **查 Silero VAD 能否纯 Rust、实时线程零分配地跑，能就替换 earshot**（D59，发布阻断）· 出处 `docs/plan/009_model-map.md` 第二节

## 认人（声纹 · 判断回路）

- [ ] **认人减负办法**（目标 1.2% → 0.15%，约 8 倍）：①锁定后降频 0.5 s → 2 s（插话晚约 1.5 s 被压，**要人定**）②只在有人声时判 ③声纹模型换更快后端（已并入 019）④更小声纹 / D47 辅助小模型逐帧判、大模型偶尔确认 ⑤**共用频谱前端（人选，D71，归 47 的 Rust 骨架）**
- [ ] **⚠️ CPU 超线（D69）**：认人判断回路约 1.2% 核（exp005 README:155，每 0.5 s 一次、每次约 6 ms），D69 整机 ≤ 1%、019 分摊给认人 0.15% → 要降约 8 倍（降频 · 换小模型 · 只在说话段跑等）· 出处 019 `standard.md` §6.1（engine worktree）
- [ ] **STFT · 重采样 · 采集输出 · 修音 四项 CPU 都没量过**（019 分摊合计 0.17%）

## 人耳（评测器）

- [ ] **降噪效果大事的起点**（若 E-1 选 A）：难场景（−5 dB · 6 人 babble · 中文目标 + 英文邻桌，K9）上比 GTCRN · DFN3 `_ll`（离线）· NoNoise-Mac（D77）· DPDFNet-2；合成测试（016 D2 级 + 隔离）+ 盲听夹众人锚点（D66）；看清 GTCRN 差在哪再定训练 / 改进 / 换模型；改输出的提速（fp16 / int8 · 剪枝 · 蒸馏，D79）也在这里评

- [ ] **下一轮盲听**：①多留背景（下限 −6 dB）的两组红线通过率最高、评分模型却最低——评分模型判不了，要人耳（018 exp011 results.md 第 4 节）②夹 4–6 道众人锚点题（D66）③016 4-9 第 3 题「覆盖 ≥ 8 个系统」一起排 ④**放进 DFN3 `_ll`**，检验「GTCRN 够不够好」（人 2026-11-02「好」：GTCRN 第三方评测少，主模型选择当假设继续验证）· 出处 018 · 017 · 016 · D33 · D49

- [ ] **NoNoise-Mac 进评测当对照组**（D77）：先看能否离线跑（CLI / 直接调 DFN3 Core ML），不能就装驱动回录；与本项目同场景比吃字率 · 评分 · 延迟 · 耗电；可选加 MetalVoice、Krisp（只对照不进 CI）· ⚠️ 装第三方驱动要人同意 · 出处 `docs/reviews/outlook-codex-20261102/research.md`

## 数据

- [ ] **exp008 第②段续跑**：FRCRN（179/348）· MossFormerGAN · URGENT-BSRNN · GMAP-SE 断点续跑再打分，回填 016 mvp 2.3；补充，不改结论 · 出处 016 现状页「转交」
- [ ] **LibriSpeech dev-clean + train-clean-100（约 6.6 GB）下载**：挂在待拍板 4-9「换素材池」与 D65 触发条件上

- [ ] **⏰ 触发：要开始训练、或要用真实原料合成时** → 先全量下载公开真实数据集（约 40 GB，解压峰值约 80 GB，不含 ECHI / CHiME-4），当场按说话人 / 场次切测试份与训练份，切法与哈希进仓；DNS · ACE · DR-VCTK 许可核清前只评测 · 出处 D63 · D64 · D65，清单见 `docs/research/014_real-datasets/decision.md` 第一节
- [ ] **DEMAND 署名**：按 D64，SA 与 CC BY 都可训练，场景小模型不必因许可重训；发布模型卡时写明 DEMAND 来源与许可（exp006 README 第 34 行写 CC BY 4.0，015-5 写 BY-SA，以 Zenodo 原页为准）· 出处 D63 · D64
- [ ] 016 benchsynth 交代补一条：合成原料可用公开集训练份，与测试份按说话人 / 场次隔离（D63）

## 仓库与工作区

- [ ] 清理 b8 已合并的 worktree：`../noise_canceling-engine` · `../noise_canceling-rustbe` · `../noise_canceling-fastpath`（及 b7 的三个，见下）

- [ ] **清理**：删已合并的 worktree `../noise_canceling-prefs` · `../noise_canceling-benchsynth`（016，PR #47）· `../noise_canceling-ears`（017，PR #48）· `../noise_canceling-judge-loop`（018，PR #49）· `../noise_canceling-rustbe`（exp010，PR #55）· `../noise_canceling-engine`（019，PR #59）· `../noise_canceling-fastpath`（020，PR #66）；删 `../noise_canceling-dfnhush`（015 已随 PR #40 合并，D61）；本地 main 同步到远端（落后 77）；推送 `chore/coord-20261031-b6`；`_tmp/` 8.3 GB 看哪些能删

## 引擎（engine）

- [ ] **开会负载下的测速与耗电**（E-5）：分纯音频通话（音频通话不等于空闲）与视频会议（+ 摄像头 ± 屏幕共享）两档，先量会议软件自身占用，v4i 按 16 ms 节奏的最坏帧 · 超时帧 · 功耗增量 · 会议软件自身是否变卡；进 / 出会议启停（StopIO 后 CPU 回 0）· 参照 DAFx26 争用测试

- [x] **codex-review 工具问题**：codex 输出以「P1」「P2」开头时，`file` 命令把它误判为 PBM / PGM 图片，第 2 轮起 `--previous` 被隐私扫描拦下；绕法：文件开头加一行 HTML 注释 · 出处 fastpath 收尾报告 · 2026-11-05 timbre（026）第二次撞上，要转 skill 维护者 · 2026-11-05 已修（手下 skillfix）：根因 macOS 27 iconv 跨第 1024 字节失败，改 perl 严格校验，23 条测试，报告 _tmp/skillfix/report.md；剩 1 条 P2（perl 坏时报错文案不准，仍会拦）

- [ ] **019 待重测（下次名额）**：耗电 3 轮（人 sudo，约 35 min）· BE-IMPACT 新测法 · rt 下 10 min BE-DEADLINE · BE-SIZE / BE-RT-SAN · 出处 019 `decision.md` §7
- [ ] **apple-rs 与 bench 合成一个共用 crate**（JSON 字段唯一真相源）· 出处 019 §7

- [ ] **手写 GTCRN 接进产品**（随「接起来」大事；默认候选改为 **native-v4i**，PR #66；另 src/dsp 重采样点积改多累加器）：补齐挡线（POWER 同场 · IMPACT · SIZE · RT-SAN · DEADLINE@rt 10 min）+ rustbe 转交 6 件（禁 panic · Model 适配器与 CI 测试 · 输入防护 · 权重校验 · 对 PyTorch 对拍 · 含长静音长跑）· 出处 exp010 README（PR #55）

- [ ] **输入 NaN/Inf 时输出 NaN 且 seam 永不恢复**（issue #53，读源码确认、未复现）· 已转 47 的 Rust 骨架线
- [ ] **vdn-review 工作流不认 args 里的审查对象**（applebe 给 019 文件，它审了 src/）· 已告 47

- [x] → 2026-11-02 换入当前大事（b8 applebe / rustbe）**019 推理引擎对比续做**（issue #45 · worktree `../noise_canceling-engine` · 2026-10-31 会话中断，只写了部分笔记）：不在当前大事内，以后单开一件大事（「轻快」那把尺子）

- [ ] **修 engine 停顿门控误开**（GTCRN 纯噪声停顿门控开 0.99；验收 = R17 由 xfail 转通过）· 出处 board 第四批「下一批建议 ②」
- [ ] **接入线**：p2 认人 / scene 路由接进 engine · 出处 board 第四批「下一批建议 ①」

## 待拍板清单

- [ ] **待拍板清单瘦身**：约 35 题按实体重新分组，过时的（BlackHole 相关）划掉 · 出处 `state/pending-decisions.md`

## 降噪效果大事 b10 转交（2026-11-03）

- [ ] 场景分类器噪音 7 类缺 4 种声音：外放人声（电视 / 视频）· 小孩和狗 · 键盘与提示音 · 风噪；是否改分类器归「场景分类器」那件事 · 出处 021 `decision.md` 转交
- [ ] 真实数据补空格线索（023 后续）：DCASE 2016 TUT 声景（library / train / tram / metro station，无主说话人，只作合成原料或统计参照）· AliMeeting 会场参数可能含 RT60 · 出处 021 现状页转交
- [ ] 开放工位 · 候车厅 · 街边的公开候选待核（scenes 评审后提出，mtgdata 已收尾未核）：DNS 2020 实录 · Krisp 测试集——核地点标签、设备、许可后补进 023 `matrix.md` · 出处 021 现状页转交
- [ ] `vdn-eval ci` / `qa` / `calib --record` 报告用 `std::fs::write`，同目录跑第二次会覆盖旧报告（违第五节 1 条）；照 `scorecard::card::write_new` 改为只新建 · 出处 scorecard 现状页转交
- [ ] 成绩单按语种（`lang`）分组比较（旧 G4 中文组的替代）· 出处 `specs/rubric/scorecard.md` 第七节
- [ ] DFN3 离线链路换产品重采样器（vdn_dsp Interpolator3 / Decimator3）复测：前视 284 → 约 192 样本，看输出效果变不变 · 方案 exp016 `plan-dfn3-resampler.md`（cands，2026-11-05，大总管定这批不做）
- [ ] NoNoise-Mac 进成绩单（D77）：命令行文件降噪模式可用，要下载未公证的 release 程序约 7.5 MB；人未选，默认推迟 · 出处 exp016 `state/handoff/cands.md` 待人拍板 1
- [ ] 模型（候选对比）：成绩单在引擎「原声混回」后打分（说话时约一半原声，engine.rs mix_out），模型本身的差约被压一半（推断）。如人选「改进 / 训练」，用 safety: false 再跑一轮看模型本身；改场景集参数归 evalfix，要名额。来源 exp016 runs/dfn3-ll-20261105-103000-4cf04ac1/notes.md 第三节
- [ ] **旁人说话 → 另外的小模型（D89 · D83）**：降噪模型不负责去掉旁人声音（成绩单 V-INTF 不过的主因，所有模型都大面积不过）；按调研 024 走「会中认出用户 → 以声纹为条件只留用户」。降噪效果大事收口后换入；起点 024 decision（Alkove Focus 从 GTCRN 改来 · 约 80 万参数 · Apache-2.0，只有厂商自测）
- [ ] **麦克风 / 输入清晰度（D91）**：人听明标对比觉得「a（带噪原声，外接麦克风合成链）本身就很不清楚」。待查：是合成链（房间混响 · 麦克风模拟）过重、还是真实 Mac 麦克风也这样；可能的手段：去混响 · 频响补偿 · 带宽扩展。实体：虚拟麦克风 / 输入链（调研 022）。降噪效果大事收口后换入
