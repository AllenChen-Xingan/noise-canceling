# 013 · exp013 场景测试集（生成器 + 设备链 + 实测房间）· codex 换谱系评审

> 对象：`docs/experiments/exp013-scene-suite/`（不含 `builds/`）+ `src/eval/suite/scenes.json`；依据：022 `simulation.md` · 016 `synth-standard.md` · 021 `catalog.md`
> 手下 scenesuite · issue #86 · 分支 `exp/013-scene-suite`
> 停止线：P0 / P1 清零，连续两轮无 P0 / P1 即停；剩余 P2 开 issue（codex-review 规则）

## 第 1 轮（2026-11-03 · codex gpt-6-astra · read-only · effort medium · 83 s）

结果：P0×0 P1×3 P2×2 · 总评「修改」。另有一次第三版之前的启动，大总管叫停、没出结果，不计轮次。

| # | 级 | 条目 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | 开放办公用 OOFFICE，而 016 第 45 行把它留作测试份（S10 不许测试份当合成原料） | 属实 | 没被 exp006 合成用过的 DEMAND 环境（OOFFICE、TMETRO、PSTATION、TCAR）按时段切：前一半留作测试份，后一半当裁判原料（016 S10①「无说话人的按录音场次 / 时段」）。`pools.TEST_SPLIT_FRAC`；`leak.py` 起点落在测试份即判不过，负对照已加。与 016 第 45 行「整个环境当测试份」不同，写进现状页请大总管定 |
| 2 | P1 | 方案说先截 ESC-50 原录音的尾巴再卷积，代码没做 | 属实 | `pools.trim_tail`：最后一个高于最大帧 −30 dB 的帧之后只留 60 ms，再淡出；片段内部的原录音混响去不掉，写明是限制。回归测试 `test_dsp.Trim` |
| 3 | P1 | 正式 build 不验收 P.56 容差，只有小 build 的测试查 | 属实 | `build.level_check`：全部记录 SNR / SIR 分开报，≥ 95% ≤ 1 dB 且全部 ≤ 1.5 dB，不过 exit 3；结果写进 manifest；`tasks.py` 读到不过就拒绝生成 |
| 4 | P2 | 截断 IR 的测试是对已过滤列表再断言，空检；T60 估计失败（0）会被当短混响接受 | 属实 | `Measured.valid` 加两条：估计失败时 DRR ≥ 15 dB 才算真短混响；IR 结尾 50 ms 比混响开头高于 −30 dB 视为截断。独立造 4 条 IR（合格 / 截断 / 不衰减 / 只一个反射）做正负对照 |
| 5 | P2 | `tasks.py` 不核对 build 覆盖了哪些组合 | 属实 | `tasks.coverage_errors`：地点 × 档 × 设备段数必须等于 `counts`，泄漏与电平验收都要过；冒烟 build 必须被拒（测试已加）|

抽查里「无法核实」三条（schema、build 结果、MIT 距离原文）不在允许读取范围，属实情：schema 在 scorecard 线；build 结果见 `builds/*/manifest.json`；MIT 1.5 m 在代码里已标【推断】。

同轮另有一路同谱系自审（实测房间接入），属实并已修的：直达起点阈值 0.5 → 0.2（16 / 270 条 MIT IR 抓晚 2–15 ms，峰前能量 > 1% 的 IR 判不可用）；直达增益撞限幅时清单记等效距离；`reverb_check` 按类别判「应当有房间」、车内加查本座位 IR 与车速；排除表按房间整体保留。未改的：ACE 的 Babble 噪声按噪声算（与本集 babble 的定义一致：多人重叠听不清）。

## 第 2 轮（2026-11-03 · codex gpt-6-astra · read-only · effort medium · 118 s）

结果：P0×0 P1×1 P2×4 · 总评「修改」。第 1 轮 5 条的修复，codex 抽查均「核实属实」。第一次启动时评审脚本把第 1 轮输出（开头是「P1 #1」）误判成 Netpbm 位图、没发出，在文件开头加一行标题后重跑；已报大总管修 skill。

| # | 级 | 条目 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | 同 build id 并发启动会互相覆盖音频：存在性检查与建目录分开 | 属实 | 开工前用 `mkdir(exist_ok=False)` 原子占用音频目录与清单目录；回归测试：同 id 再跑一次必须失败且不动已有音频 |
| 2 | P2 | 测试份切分代码写「待定」、README 写「已定」，016 第 45 行未改 | 属实 | 大总管已定按时段切；代码注释改为「已定」；016 第 45 行由大总管改（不在本线白名单，已转交）|
| 3 | P2 | 022 第 2 步「混响压力」分支只作用在点声源，扩散人声的尾巴仍被衰减 | 属实 | `Scene.diffuse`：开着该分支时直达做扩散衰减、尾巴不衰减；回归测试比 4 kHz 以上尾巴能量 |
| 4 | P2 | 复现指纹漏了 exp006 `recipes.json` 与实测 RIR、房间噪声、几何文件 | 属实 | manifest 的 `params_sha256` 加 exp006 配方；`pools.index_hash` 纳入 ACE / MIT / McVAMPIRE 全部文件 |
| 5 | P2 | `--allow-fallback` 入口实际不可用 | 属实 | 删掉回退：缺素材一律报错（同种子跨机器取材必须一致）；`sounds.json` 去掉 fallback；负对照测试 |

另：codex 指出 README 第九节第 10 条仍写「最大值的一半」，已改为 0.2。

## 第 3 轮（2026-11-03 · codex gpt-6-astra · read-only · effort medium）

结果：P0×0 P1×0 P2×5 · 连续第 1 轮无 P0 / P1。第 2 轮 5 条修复抽查均「核实属实」。

| # | 级 | 条目 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P2 | 016 第 45 行仍是「整环境当测试份」，只有转交 | 属实 | 016 由大总管改（已转交）；本线分区正本是 `pools.TEST_SPLIT_FRAC`，生成器与 `leak.py` 都读它 |
| 2 | P2 | README 仍写已删的 `--allow-fallback` 与「没有就合成」 | 属实 | 已删；第一节概览表按第三版改 |
| 3 | P2 | 方位衰减节点按声源重抽，022 规定每段抽一次 | 属实 | `draw_nodes` 在 Chain 初始化时抽一次，存进 `chain.direction_nodes_db`；回归测试：同段同角度两次结果相同 |
| 4 | P2 | 会议室共同目标可在 0.5 s 开口，破坏开头 1.5 s 留空 | 属实 | 所有要留的人都从 `lead_silence_s` 之后开口 |
| 5 | P2 | 没有正本 schema 时正式 build 仍可通过 | 属实 | 正式 build 找不到 schema 即停；`--dev` 只给冒烟，manifest 标 dev，`tasks.py` 拒用 |

## 第 4 轮（2026-11-03 · codex gpt-6-astra · read-only · effort medium）

结果：P0×0 P1×0 P2×3 · **本轮与上一轮都无 P0 / P1，满足停止线**。第 3 轮修复抽查均「核实属实」。

| # | 级 | 条目 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P2 | 016 第 45 行仍未改；`pools.py` 文件头还写「其余环境全段」 | 属实 | 文件头已改；016 第 45 行留给大总管（转交）——剩余项 |
| 2 | P2 | 清单的 T60 / DRR 是设备链之前的房间值，文档写成「设备链之后实测」；README 还留着模拟尾巴的解释 | 属实 | clip-manifest 写明是房间本身的值；README 第九节第 7 条改写。链后有效混响指标没做——剩余项 |
| 3 | P2 | 无 schema 的环境下，「同 build id 拒跑」测试第二次漏带 `--dev`，会因别的原因失败 | 属实 | 两次用同一模式 |

## 剩余（开 issue 留后处理）

1. 016 `synth-standard.md` 第 45 行改为指向 S10①「按时段切」（大总管改）
2. 清单加「设备链之后」的有效 T60 / DRR（对经过同一设备处理的直达与尾部计算），供成绩单按设备拆混响条件
3. 评审脚本把以「P1 #」开头的 codex 输出误判为 Netpbm 位图（已报大总管修 skill）

## 采纳 / 提出

4 轮共提出 15 条（P1×4 · P2×11），属实 15 条，已改 13 条，2 条列为剩余（第 4 轮 #1 的 016 部分、#2 的链后指标）。
