# 现状页 · scenesuite（批次 20261103-b11）

- **状态**：已收尾（2026-11-03），等大总管验收
- **手上**：exp013 场景测试集：15 个开会地点的混合声音片段 + MacBook Air 内置麦模拟 + 逐段声音标注
- **交代**：主目录 `_tmp/20261103-b11/scenesuite.md`
- **做到哪**：正式版 build `20261103230756-17f5`（480 段）已出，68 条测试全过，codex 4 轮已达停止线，场景任务文件 60 条指向它
- **下一步**：大总管验收；合并前大总管在本分支 merge main，并在 `src/schema/src/eval.rs` 的 DATA_FILES 加 `scenes.json`

## 交付

| 交付 | 位置 |
|---|---|
| 方案 · 实现偏差 · 许可说明 | `docs/experiments/exp013-scene-suite/README.md`（第九节是实现与方案的偏差）|
| 混响修法方案 | `docs/experiments/exp013-scene-suite/reverb-plan.md` |
| 片段清单格式 v1.2（正本是 scorecard 的 Rust 类型导出的 schema）| `docs/experiments/exp013-scene-suite/clip-manifest.md` |
| 设备链模块 + 参数正本（与 022 文档交叉校验）| `device_chain.py` · `device_chain.json` |
| 地点组法 · 声音类别 · 素材登记（正本）| `scenes.json` · `sounds.json` · `materials.json` |
| 生成器 | `build.py` · `compose.py` · `tracks.py` · `synth_sounds.py` · `pools.py` · `rirs.py` · `rooms.py` · `labels.py` · `manifest.py` · `dsp.py` |
| 检查器 | `leak.py`（防泄漏 + 测试份）· `reverb_check.py`（同房间 / T60）· `build.level_check`（电平容差）· `tasks.coverage_errors` |
| 测试（68 条）| `docs/experiments/exp013-scene-suite/tests/`；跑法：`<主目录>/_tmp/exp002-venv/bin/python -m unittest discover -s docs/experiments/exp013-scene-suite/tests` |
| 正式 build 的清单 / 哈希 / 排除表 / 参数快照 | `docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/` |
| 场景任务文件（60 条，graders 留空由成绩单标准定）| `src/eval/suite/scenes.json` |
| codex 评审记录 | `docs/reviews/013_scene-suite-codex.md`（索引已加）|
| 音频（仓外）| 主目录 `_tmp/exp013/20261103230756-17f5/`（3.3 GB）|

正式版的数字【实测，build 17f5 的 manifest】：480 段 = 15 个地点 × 16 段 × 2 个设备；同房间 37.5%（ACE 148 段 + 车内 32 段）；SNR 99.4% 段与名义值差 ≤ 1 dB、最大 1.08 dB；SIR 98.6% ≤ 1 dB、最大 1.31 dB；对 exp006 训练素材交集为 0；混响检查 0 段不过。

## 改了哪些文件

只写了交代里列的位置：`docs/experiments/exp013-scene-suite/`、`src/eval/suite/scenes.json`、主目录 `_tmp/exp013/`、本页；另按共同规矩第 9 条写了 `docs/reviews/013_scene-suite-codex.md` 与 `docs/reviews/README.md` 一行索引。下载素材放 `~/vdn-data/`（esc50 · demand 三个新环境 · rir）。

## 发现

- 本机可用的英文说话人只有 20 人（LibriSpeech test-clean 奇数位），都是朗读；中文只有 2 人有许可记录
- 本集与 exp006 的 test / seq 同池：**exp006 场景分类器不能在本集上评**。降噪模型不受影响
- 两种电平算法（gated / P.56）对稀疏人声本来就差最多约 1.5 dB，所以核对分两层（README 第九节第 2 条）
- 自写的 P.56 曾取错门限、合成舱噪曾有亚音频漂移、直达声起点曾抓到反射，都已修并有回归测试
- 评审脚本会把以「P1 #」开头的 codex 输出误判为位图，第 2 轮以后要先在文件开头加一行标题（大总管已开 issue）

## 待人拍板

1. **会议室怎么判**（近处 2–3 人都要留，和其他 14 个地点「只留我」相反）
   - A. 照样出，干净参考 = 近处所有人 —— 代价：现有模型大概率成绩差，但差得有道理 ← 建议，已按 A 出
   - B. 本批不出会议室 —— 代价：缺一个地点
   - 不回默认 A

已定（不再问）：合成得可能不像的声音成绩单标「合成，待真实录音校」（大总管定 A）· 实测 RIR 下载档 A（人选）· T60 比对阈值 ×[0.8, 1.25] 暂定（大总管定）· DEMAND 没用过的环境按时段切测试份（大总管定 A，016 第 45 行已由大总管改）

## 转交（范围外，留给大总管）

- `src/schema/src/eval.rs` 的 DATA_FILES 加 `scenes.json`（大总管在本分支 merge main 后加）
- 已开 issue：清单加「设备链之后」的有效 T60 / DRR；评审脚本位图误判
- 成绩单按 `chain.room.paired`（人声与底噪是否同房间）分组比较（scorecard 已记转交）
- 五版 build 的音频共约 16 GB 在主目录 `_tmp/exp013/`（b22d · c025 · de56 · 72df · 17f5）；人试听过的是第一版 b22d。留删由大总管定

## 流水

- 读交代、共同规矩、手下协议 → 报到；读 021 / 022 / 016、exp006 / exp008、vdn-eval → 方案草稿与清单格式；scorecard 对齐清单 v1
- 大总管批方案与两批下载 → ESC-50 168 段（71 MB）+ DEMAND TCAR / TMETRO / PSTATION（376 MB）
- 写设备链（与 022 交叉校验）、生成器、检查器、测试；冒烟发现并修 P.56 门限、T60 量法、广播提示音、合成噪声亚音频、峰值超满幅
- 草稿目录里跑过一次完整规模冒烟（17 s，未申请名额，已向大总管补报）
- 第一版 b22d；三路自审 → 修 owner_dry、补 7 种声音、风噪、方位等；第二版 c025（任务加外接麦对照，60 条）
- 人试听第一版指出人声与底噪混响不一致 → 停 codex → 两路调研 → reverb-plan.md → 人选下载档 A（实测 RIR 约 685 MB）→「一段 = 一个房间」→ 第三版 de56（同房间 37.5%）
- codex 第 1 轮（P1×3 P2×2）+ 实测房间自审 → 修测试份切分、ESC 去尾、电平验收、覆盖核对、直达起点、截断 IR → 第四版 72df
- codex 第 2 轮（P1×1 P2×4）、第 3 轮（P2×5）、第 4 轮（P2×3）→ 第 3、4 轮连续无 P0/P1，达停止线
- 第五版（正式）17f5 → 名额归还；场景任务文件指向它；现状页已收尾
