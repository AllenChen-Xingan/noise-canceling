# 以后再办（积压清单）

> 大总管一次只做好一件大事，每个手下只管 2–4 个实体（CLAUDE.md 第五节第 4 条）。不属于当前大事的事按**实体**分组记在这里；换入某个实体时把它下面的事一起取走，打勾写去向，**不删**。
> 要人拍板的题不放这里，放 `state/pending-decisions.md`。

## 处理链的统一表示（下一件大事，D66 · 011）

- [ ] **处理图 + schema 交叉校验**：处理链表示成「节点带类型 · 参数 · 版本」的图，会话记录同一套表示；Rust 类型为源头导出 schema、CI 比对；清掉 speaker / polish / scene 的 `vdn-types` 镜像副本（12 处）与手写 JSON；后处理级非有限检查与 Engine classify 规则合成一处；非有限值检查在 Pipeline 与送音端各一处（进程边界，有意）统一口径 · 出处 `docs/plan/011_schema-crosscheck.md` · Rust 骨架审查 `docs/plan/010_rust-skeleton-audit.md`

## Rust 骨架的遗留（b8-rust 批次）

- [ ] 空闲时推理块约 3 ms（标定 0.7 ms）：线程停在能效核低频；QoS user-interactive 小实验未完则续做；压 lead 到 144（3 ms）· 出处 `docs/experiments/exp-v7-latency/`
- [ ] 共享内存权限 0600 与 coreaudiod 下的虚拟麦克风能否打开（装驱动那批实测；补 009 待拍板 3）
- [ ] 虚拟麦克风真驱动接口 + 安装（D05，要人输密码、断音几秒）；构建部署目标 14.4（D67）
- [ ] 麦克风流无缝重接（回调里换设备，不断声）
- [ ] `vdn-eval` / `vdn-speaker` / `vdn-speaker-lab` 改走推理后端，不再直接依赖 ort
- [ ] ContentId 用 FNV → 规约原写 sha256
- [ ] RTSan 检查（规约 G8）
- [ ] Ring 拆 Producer / Consumer 句柄（类型挡住「一端一线程」）

## 虚拟麦克风（HAL 插件）

- [ ] **dev-4 实时管线 + Rust 虚拟麦克风**：第一步是 D05 空插件加载与崩溃实验（D54 · D55）；要人输密码、断音几秒 · 出处 `docs/plan/001_decisions-20261029-b1.md` D54 / D55

## 模型

- [ ] **查 Silero VAD 能否纯 Rust、实时线程零分配地跑，能就替换 earshot**（D59，发布阻断）· 出处 `docs/plan/009_model-map.md` 第二节

## 数据

- [ ] **⏰ 触发：要开始训练、或要用真实原料合成时** → 先全量下载公开真实数据集（约 40 GB，解压峰值约 80 GB，不含 ECHI / CHiME-4），当场按说话人 / 场次切测试份与训练份，切法与哈希进仓；DNS · ACE · DR-VCTK 许可核清前只评测 · 出处 D63 · D64 · D65，清单见 `docs/research/014_real-datasets/decision.md` 第一节
- [ ] **DEMAND 署名**：按 D64，SA 与 CC BY 都可训练，场景小模型不必因许可重训；发布模型卡时写明 DEMAND 来源与许可（exp006 README 第 34 行写 CC BY 4.0，015-5 写 BY-SA，以 Zenodo 原页为准）· 出处 D63 · D64
- [ ] 016 benchsynth 交代补一条：合成原料可用公开集训练份，与测试份按说话人 / 场次隔离（D63）

## 仓库与工作区

- [ ] **清理**：删已合并的 worktree `../noise_canceling-prefs`；删 `../noise_canceling-dfnhush`（015 已随 PR #40 合并，D61）；本地 main 同步到远端（落后 77）；推送 `chore/coord-20261031-b6`；`_tmp/` 8.3 GB 看哪些能删

## 引擎（engine）

- [ ] **019 推理引擎对比续做**（issue #45 · worktree `../noise_canceling-engine` · 2026-10-31 会话中断，只写了部分笔记）：不在当前大事内，以后单开一件大事（「轻快」那把尺子）

- [ ] **修 engine 停顿门控误开**（GTCRN 纯噪声停顿门控开 0.99；验收 = R17 由 xfail 转通过）· 出处 board 第四批「下一批建议 ②」
- [ ] **接入线**：p2 认人 / scene 路由接进 engine · 出处 board 第四批「下一批建议 ①」

## 待拍板清单

- [ ] **待拍板清单瘦身**：约 35 题按实体重新分组，过时的（BlackHole 相关）划掉 · 出处 `state/pending-decisions.md`
