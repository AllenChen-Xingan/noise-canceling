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

## 第 2 轮

（待跑）
