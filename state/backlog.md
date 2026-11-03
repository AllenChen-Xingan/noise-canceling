# 以后再办（积压清单）

> 大总管一次只做好一件大事，每个手下只管 2–4 个实体（CLAUDE.md 第五节第 4 条）。不属于当前大事的事按**实体**分组记在这里；换入某个实体时把它下面的事一起取走，打勾写去向，**不删**。
> 要人拍板的题不放这里，放 `state/pending-decisions.md`。

## 处理链的统一表示（下一件大事，D72 · 011）

- [ ] **处理图 + schema 交叉校验 + 前端只算一次（D71）**：STFT / fbank / 场景特征每帧只算一次，降噪 · 认人 · 场景 · VAD 共用（规约 §3.1 `SpectrumTap` 已留口）；同一功能只在一处实现；处理链表示成「节点带类型 · 参数 · 版本」的图，会话记录同一套表示；Rust 类型为源头导出 schema、CI 比对；清掉 speaker / polish / scene 的 `vdn-types` 镜像副本（12 处）与手写 JSON；后处理级非有限检查与 Engine classify 规则合成一处；非有限值检查在 Pipeline 与送音端各一处（进程边界，有意）统一口径 · 出处 `docs/plan/011_schema-crosscheck.md` · Rust 骨架审查 `docs/plan/010_rust-skeleton-audit.md`

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

## 人耳（评测器）

- [ ] **下一轮盲听**：①多留背景（下限 −6 dB）的两组红线通过率最高、评分模型却最低——评分模型判不了，要人耳（018 exp011 results.md 第 4 节）②夹 4–6 道众人锚点题（D66）③016 4-9 第 3 题「覆盖 ≥ 8 个系统」一起排 · 出处 018 · 017 · 016

## 数据

- [ ] **exp008 第②段续跑**：FRCRN（179/348）· MossFormerGAN · URGENT-BSRNN · GMAP-SE 断点续跑再打分，回填 016 mvp 2.3；补充，不改结论 · 出处 016 现状页「转交」
- [ ] **LibriSpeech dev-clean + train-clean-100（约 6.6 GB）下载**：挂在待拍板 4-9「换素材池」与 D65 触发条件上

- [ ] **⏰ 触发：要开始训练、或要用真实原料合成时** → 先全量下载公开真实数据集（约 40 GB，解压峰值约 80 GB，不含 ECHI / CHiME-4），当场按说话人 / 场次切测试份与训练份，切法与哈希进仓；DNS · ACE · DR-VCTK 许可核清前只评测 · 出处 D63 · D64 · D65，清单见 `docs/research/014_real-datasets/decision.md` 第一节
- [ ] **DEMAND 署名**：按 D64，SA 与 CC BY 都可训练，场景小模型不必因许可重训；发布模型卡时写明 DEMAND 来源与许可（exp006 README 第 34 行写 CC BY 4.0，015-5 写 BY-SA，以 Zenodo 原页为准）· 出处 D63 · D64
- [ ] 016 benchsynth 交代补一条：合成原料可用公开集训练份，与测试份按说话人 / 场次隔离（D63）

## 仓库与工作区

- [ ] **清理**：删已合并的 worktree `../noise_canceling-prefs` · `../noise_canceling-benchsynth`（016，PR #47）· `../noise_canceling-ears`（017，PR #48）· `../noise_canceling-judge-loop`（018，PR #49）；删 `../noise_canceling-dfnhush`（015 已随 PR #40 合并，D61）；本地 main 同步到远端（落后 77）；推送 `chore/coord-20261031-b6`；`_tmp/` 8.3 GB 看哪些能删

## 引擎（engine）

- [ ] **019 推理引擎对比续做**（issue #45 · worktree `../noise_canceling-engine` · 2026-10-31 会话中断，只写了部分笔记）：不在当前大事内，以后单开一件大事（「轻快」那把尺子）

- [ ] **修 engine 停顿门控误开**（GTCRN 纯噪声停顿门控开 0.99；验收 = R17 由 xfail 转通过）· 出处 board 第四批「下一批建议 ②」
- [ ] **接入线**：p2 认人 / scene 路由接进 engine · 出处 board 第四批「下一批建议 ①」

## 待拍板清单

- [ ] **待拍板清单瘦身**：约 35 题按实体重新分组，过时的（BlackHole 相关）划掉 · 出处 `state/pending-decisions.md`
