# 现状页 · rubric（批次 20261030-b2）

状态：**验收通过（待重算）**——窗口不关。① 终版 scores.json 已收（sha e84730fa…），dryrun 与 decision 已重算；② 还等人的 AB 导出 JSON（大总管转来），填人耳列后报收尾

## 手上
007 降噪 rubric：评分标准 + 用它筛候选。交代 `_tmp/20261030-b2/rubric.md`（主目录）

## 交付
- `specs/rubric/denoise.md`：rubric 正文（否决 V1–V8 · 12 个机器维度四档锚点 · 6 条防讨好硬规则 · 人耳 5 项与单人听评流程 · 合成与裁决 · 正负例 · 脚本规格 · 缺陷词汇表 17 项）
- `docs/research/007_denoise-rubric/`：README · must-know（17 条）· sources（40 条外部 + 自家）· gap · **decision**（筛选表 + 待人拍板 4 题）· dryrun-exp003（临时机器分预演，**作者听完 AB 再看**）

## 结论一句
现在只能比「轻快」：GTCRN 22/25 领先，DPDFNet-2 16/25；清楚、自然要等作者听完 AB。D01 选 45 ms 只剩 GTCRN；选 75 ms 两者都入围，排名几乎全由耳朵决定。exp003 临时分预演：DPDFNet-2 在中文两段（c11、c12）疑似吃字 / 伤人声、停顿处可能真空，列为必听段。

## 待人拍板（详见 decision.md 末节）
1. 三把尺子权重（建议 清楚 40 · 自然 35 · 轻快 25）
2. 耳朵与机器怎么合（建议 0.6 × 耳朵 + 0.4 × 机器）
3. 淘汰线 + 「能用 / 勉强 / 不能用」一问留不留（与设计稿 §13「不打绝对分」有张力；建议留，≥ 1/3 段「不能用」淘汰）
4. 听评时长（三候选约 20 分钟；建议分两天各约 12–15 分钟，需 abtest 拆页）
另：D01 延迟上限仍是人的决定，decision.md 开头写了怎么用这份表定它

## 未了
- exp003 `scores.json`（sha256 2cf753a7…）是临时数：abtest 回「会变」，正在加 voxlock_given / voxlock_live 两列；旧三列除 c12 插入 / 替换外不变（我用的那份 c12 插入已是新值，删除数不受影响 → 预演结论不变）。abtest 发来新 sha256 后重算 `dryrun-exp003.md`
- 给 abtest 的减负建议：已采纳（吃字并进每个候选一行 + 「都没有吃字」按钮；标签折叠成「＋毛病」）
- 给 abtest 的可选减负建议（吃字并进每个候选一行、标签默认折叠）：由它定

## 流水
- 2026-10-30 读交代 + common；报到大总管（会话名：abtest=noise-canceling-exp003-2d · pse=noise-canceling-45 · sota=noise-canceling-sota-4f）
- 2026-10-30 读 intent · 001 决策 · 002 路线图 · 004 decision/must-know · exp001/002 · 002 decision · 005 decision · 设计稿 §3/§9/§13 · abtest 分支的 ab_page.html / build_page.py
- 2026-10-30 与 abtest 对齐：现有三问 + 两道陷阱原样采用；abtest 加 usable（ok/marginal/no）、tags（6 个键）、started_at/answered_at/plays/switches；VoxLock 本批进；score.py 已用 Qwen3-ASR 1.7B 且有 noisy 列。抄送大总管
- 2026-10-30 报 Workflow 计划，大总管批准，开跑（wf_158ffb40-3f9）
- 2026-10-30 abtest 定：每屏 A/B/C + 原声，「最清楚 / 最自然」选一，吃字多选，usable/tags 每候选各答，拉丁方轮换
- 2026-10-30 写 README · rubric 初稿 · gap；算单侧符号检验门槛
- 2026-10-30 调研完成（9 代理，怀疑派逐路复核）；据此改 rubric，写 must-know · sources · decision；给 abtest 两条页面建议（5 ms 淡入淡出、两问顺序随机）——abtest 已做，导出加 `qorder`
- 2026-10-30 三视角自审：怀疑派 3 P0（小样本放过吃字 / 几乎不处理也能入围 / 响度没对齐）+ 11 P1；实现派 5 P0（显著线表一格算错 / C2、C4 在 exp003 算不了 / 矛盾判不了 / V7 永不触发 / 骨架顺序与 `if H`）+ 约 15 P1；产品派 4 P0（时长低估 / 「GTCRN 领先」误导 / 默认值暗中生效 / D01 不在表里）。全部按条修改：rubric 整篇重写，decision 重写
- 2026-10-30 用 exp003 临时 scores.json 预演 rubric → `dryrun-exp003.md`（单独成文，防先入为主）；问 abtest 数是否终版；给 abtest 可选减负建议
- 2026-10-30 收尾
- 2026-10-30 abtest 交终版 scores.json（sha256 e84730fa…，含 voxlock_given / voxlock_live）+ voxlock_bench.json；重算 dryrun：VoxLock 两种都因 M5 单线程最坏一块超块长（最好 102%）被 V3 否决；DPDFNet-2 风险不变；GTCRN G3 降到 +0.91，只剩 c04 疑似。decision 的 VoxLock 行、时长（一屏 4 候选，约 25 分钟）同步改
