# 大总管看板 · noise-canceling-99

> 本大总管的账本（e3 的在 `state/coord.md`，不要改它）。大总管检出：`../noise_canceling-coord99`，分支 `chore/coord-20261102-b9`。手下交代在主目录 `_tmp/20261102-b9/`。决策号：记前先看主检出与各分支最大号，并向 e3 发「占用 Dxx」（2026-11-02 主检出最大 D78；e3 占用 D79「推理以能效核为准」→ 我方下一个从 D80 起）。

## 当前大事 · 大总管 noise-canceling-99（2026-11-02 起，与 e3 的推理引擎并行）

**处理图 + schema 交叉校验**（D72 · `docs/plan/011_schema-crosscheck.md` · CLAUDE.md 第五节第 5 条）。人原话「我们现在开始做 schema 以及图的事情吧」。批次 `_tmp/20261102-b9/`，tmux 工作区 `vdn-schema`。
第一阶段两条线并行；第二阶段「处理图（含会话记录记图版本 · 前端只算一次 D71 · 参数描述导出 schema · 模型接口）」等第一阶段合并后派。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| single | 清镜像副本 · 参数只写一处 · #56 · `feat/types-single-source` · `../noise_canceling-single` | vdn-types（参数描述）· vdn-speaker · vdn-polish · vdn-scene | 完成，PR #60 已合并（#56 关），窗口已关；worktree 保留待清 |
| schema | schema 导出 + 交叉校验 · #57 · `feat/schema-crosscheck` · `../noise_canceling-schema` | schema 机制 · 共享内存头 · 配置 · 评测任务清单 | 完成，PR #63 已合并（#57 关，0604143），窗口已关；worktree 保留待清 |
| graph | 第二阶段 · 处理图描述 + 与 Pipeline 交叉校验 + 会话记录记图版本 · #64 · `feat/processing-graph` · `../noise_canceling-graph` | 处理图描述 · Pipeline · 会话记录 · 修音链频谱级归属 | 已派 |
| iface | 第二阶段 · 参数描述导出 schema · 模型接口三方一致 · FE-M 名字只留一个 · #65 · `feat/param-model-schema` · `../noise_canceling-iface` | 参数描述导出 · 场景参数表 / 我的配置文本 · 模型接口 | 已报到（noise-canceling-iface-d6）|

## 进度板

- 2026-11-02 两人报到。single：三镜像共 341 行，floor_db 21 个文件，先比对分歧，按 polish → speaker → scene 删。schema：新建 tool 档 `vdn-schema`，导出到 `specs/schema/`，先做共享内存头
- 2026-11-02 应 e3 要求，账本从 `state/coord.md` 挪到本文件，git 写操作改在本检出做
- 2026-11-02 single：9 处分歧，8 处按正本；polish 的 SpectralStage 不是镜像而是扩展 trait → 大总管定 A（留 polish，第二阶段处理图再定是否进 vdn-types）；FE-M 名改 "fastenhancer_m"，要求先全仓查旧名 + 加「route 输出名能被 vdn-types 解析」测试
- 2026-11-02 ~19:52 e3 基准（约 30 min，heavy.lock），已通知两人停编译测试
- 2026-11-02 schema：机制定（`specs/schema/<实体>.schema.json` · `cargo run -p vdn-schema [-- --check]` · schemars 派生 / 核心 crate 常量表 + 穷举解构小宏 · boon 只作测试依赖）；共享内存头 · Config · 评测清单 4 份已做完，带正负例测试，暂停期间跑 codex。大总管要求核 cargo-deny 许可 · 宏 ≤30 行 · check-deps；机制已转 single
- 2026-11-02 single：三镜像已删；vdn-types `desc.rs` 参数描述表 39 条；5 个 crate 改从表取默认与范围；tidy 加 R-MIRROR · R-PARAM；锁期间改的未编译。大总管让它锁期间先跑 codex
- 2026-11-02 schema 编译测试结束（自报）：vdn-schema --check 一致 · xtask ci 6 步全过 · 交叉校验 18 条 · eval ci 回归 13/17 同前；schemars 改为 vdn-control 可选特性；codex 第 1 轮 P1×2 P2×3 已改（docs/reviews/011_schema-crosscheck-codex.md），第 2 轮在跑。等 single 跑完再通知 e3
- 2026-11-02 single 编译测试结束（自报）：xtask ci 全绿；新 tidy 规则首跑抓到 polish AGC 私有字段 floor_db 撞名 → 改 noise_floor_db；codex 第 1 轮 P1×1 P2×5 已改，第 2 轮在跑。已通知 e3「编译结束」
- 2026-11-02 ~20:5x e3 第二段基准（约 32 min）；两人停编译
- 2026-11-02 schema codex 第 2 轮 P1×1 P2×2 已改：布局版本对照 main 只增不改（ci.yml 加 fetch-depth: 0，合并冲突大总管处理）· 总线名收窄为可见 ASCII 无 /（行为变化，进 PR 正文）· 往返整数精确比较；denoised 非 ASCII 负例测试派给 schema 补
- 2026-11-02 e3 基准作废（拔电睡眠），锁删；两人复跑
- 2026-11-02 single 复跑（自报）：xtask ci 全绿，test 372 过 / 0 败 / 5 忽略，含 codex 第 2、3 轮修改；在写 PR 草稿
- 2026-11-02 single 报收尾：49 文件 +3191 −512；desc.rs 58 条；codex 3 轮（P1 1→0→0），14 条已改；忽略 5 条均旧有人工测试。大总管验收中：亲手重跑 xtask ci（后台）。发现：FE-M 两个名字（vdn-types `fastenhancer_m` vs 命令行 / 配置 / 评测 `fastenhancer-m`），改动前就有 → 第二阶段「模型接口」统一
- 2026-11-02 schema 复跑（自报）：xtask ci 全过 · vdn-schema --check 一致 · vdn-schema 21 条 · eval ci 同前（回归 13/17 · 能力 1/10 · 探针 5/5）；codex 第 3 轮在跑
- 2026-11-02 single 验收：大总管亲手 xtask ci 全绿（372 / 0 / 5，与自报一致）；合 origin/main（+152，冲突仅评审索引）后再跑全绿；PR #60 已开，等 CI
- 2026-11-02 PR #60 CI 过，已合并（b2902db），#56 关，远端分支已删；通知 single 收尾、schema 等大总管合 main
- 2026-11-02 schema codex 第 3 轮 P1×2 P2×2 属实待改（基线读不到不放行 · push 基线用 before · 版本历史前缀比对 · 整数上界）；大总管合 origin/main 进 schema 分支（d157caf，冲突仅评审索引）
- 2026-11-02 ~21:30 e3 电池组基准约 35 min，之后插电组约 32 min；schema 只改文件
- 2026-11-02 ~22:05 schema 在空档里跑完（自报）：重新导出（布局加 history · 整数上下界）· vdn-schema 24 条 · xtask ci 全过（含新 tidy 规则）· --check 一致；codex 第 4 轮在跑。已通知 e3 开插电组；大总管验收复跑排在插电组后
- 2026-11-02 e3 基准结束；人说「schema他们那个可以开始做」（经 e3 转）。大总管在 6b276e1 亲手验收：xtask ci 全绿（400 / 0 / 5）· vdn-schema --check 一致 · eval ci 通过。codex 第 4 轮 P2×2（测试写死单版本 · 基线读失败测试挡不住回退）→ 定：都改，再跑第 5 轮
- 2026-11-02 schema 改完第 4 轮两条 P2（2e41218e，自报）：vdn-schema 25 条 · 变异验证新测试能红 · xtask ci / --check / eval ci 全过；codex 第 5 轮在跑
- 2026-11-02 schema codex 第 5 轮无 P0/P1（4、5 连续干净，评审结束）；P2：基线读入口测试 → 补；「3.0 算不算整数」schema 宽 Rust 严 → 开 issue #61
- 2026-11-02 代码卫生线 #62（feat/code-hygiene，会话 8d / 手下 hygiene）与我方对齐：采样率收口 · 阈值比对归 #62；他们第二段（control · eval）等 #57 合并后动 → **#57 合并时发 8d「#57 已合并」**
- 2026-11-02 schema 收尾（402 / 0 / 5，codex 5 轮 16 条：15 改 1 转 #61；手下自记用过一次 git rm）。大总管在 d32b4e8 亲手 ci / --check / eval ci 全过；PR #63 已开，等远端 CI
- 2026-11-02 PR #63 合并（0604143），#57 关；已告 8d「#57 已合并」；schema 关窗。**第一阶段完成**
- 2026-11-02 第二阶段派出：graph（#64）· iface（#65）。发现 main 已有 StageChain + Pipeline::with_stages（Rust 骨架线），图只补描述数据与交叉校验。前端只算一次（D71）不在本阶段，留第三步
