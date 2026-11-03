# 大总管看板 · noise-canceling-99

> 本大总管的账本（e3 的在 `state/coord.md`，不要改它）。大总管检出：`../noise_canceling-coord99`，分支 `chore/coord-20261102-b9`。手下交代在主目录 `_tmp/20261102-b9/`。决策号：记前先看主检出与各分支最大号，并向 e3 发「占用 Dxx」（2026-11-02 主检出最大 D78）。

## 当前大事 · 大总管 noise-canceling-99（2026-11-02 起，与 e3 的推理引擎并行）

**处理图 + schema 交叉校验**（D72 · `docs/plan/011_schema-crosscheck.md` · CLAUDE.md 第五节第 5 条）。人原话「我们现在开始做 schema 以及图的事情吧」。批次 `_tmp/20261102-b9/`，tmux 工作区 `vdn-schema`。
第一阶段两条线并行；第二阶段「处理图（含会话记录记图版本 · 前端只算一次 D71 · 参数描述导出 schema · 模型接口）」等第一阶段合并后派。

| 手下 | 线 | 管的实体 | 状态 |
|---|---|---|---|
| single | 清镜像副本 · 参数只写一处 · #56 · `feat/types-single-source` · `../noise_canceling-single` | vdn-types（参数描述）· vdn-speaker · vdn-polish · vdn-scene | 已报到（noise-canceling-single-68）|
| schema | schema 导出 + 交叉校验 · #57 · `feat/schema-crosscheck` · `../noise_canceling-schema` | schema 机制 · 共享内存头 · 配置 · 评测任务清单 | 已报到（noise-canceling-schema-1d）|

## 进度板

- 2026-11-02 两人报到。single：三镜像共 341 行，floor_db 21 个文件，先比对分歧，按 polish → speaker → scene 删。schema：新建 tool 档 `vdn-schema`，导出到 `specs/schema/`，先做共享内存头
- 2026-11-02 应 e3 要求，账本从 `state/coord.md` 挪到本文件，git 写操作改在本检出做
- 2026-11-02 single：9 处分歧，8 处按正本；polish 的 SpectralStage 不是镜像而是扩展 trait → 大总管定 A（留 polish，第二阶段处理图再定是否进 vdn-types）；FE-M 名改 "fastenhancer_m"，要求先全仓查旧名 + 加「route 输出名能被 vdn-types 解析」测试
- 2026-11-02 ~19:52 e3 基准（约 30 min，heavy.lock），已通知两人停编译测试
- 2026-11-02 schema：机制定（`specs/schema/<实体>.schema.json` · `cargo run -p vdn-schema [-- --check]` · schemars 派生 / 核心 crate 常量表 + 穷举解构小宏 · boon 只作测试依赖）；共享内存头 · Config · 评测清单 4 份已做完，带正负例测试，暂停期间跑 codex。大总管要求核 cargo-deny 许可 · 宏 ≤30 行 · check-deps；机制已转 single
- 2026-11-02 single：三镜像已删；vdn-types `desc.rs` 参数描述表 39 条；5 个 crate 改从表取默认与范围；tidy 加 R-MIRROR · R-PARAM；锁期间改的未编译。大总管让它锁期间先跑 codex
- 2026-11-02 schema 编译测试结束（自报）：vdn-schema --check 一致 · xtask ci 6 步全过 · 交叉校验 18 条 · eval ci 回归 13/17 同前；schemars 改为 vdn-control 可选特性；codex 第 1 轮 P1×2 P2×3 已改（docs/reviews/011_schema-crosscheck-codex.md），第 2 轮在跑。等 single 跑完再通知 e3
- 2026-11-02 single 编译测试结束（自报）：xtask ci 全绿；新 tidy 规则首跑抓到 polish AGC 私有字段 floor_db 撞名 → 改 noise_floor_db；codex 第 1 轮 P1×1 P2×5 已改，第 2 轮在跑。已通知 e3「编译结束」
