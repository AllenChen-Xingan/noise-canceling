# 现状页 · fastpath（研究 020 / 实验 exp012 · issue #58 · 分支 exp/012-fastpath）

管的实体：推理后端 · 模型（GTCRN）· 评测器（帧耗时 · 逐值一致 · 耗电）

## 合并前必做（大总管定 2026-11-02）
- [ ] applebe 的 019 先合进 main；之后本分支 `git merge main`
- [ ] 在 `docs/experiments/exp010-inference-bench/apple-rs/tests/fields.rs` 的 `APPLE_ONLY` 里删掉 `win_start_unix` / `win_end_unix` / `wall_s` 三条，跑该测试转绿
- [ ] **所有代码文件 ≤ 600 行**（人定，随 #62 写进 CLAUDE.md 第五节第 7 条，不设基线、不留豁免）：拆 `bench/src/gtcrn_native.rs`（1926）· `bench/src/main.rs`（1280）· `bench/src/gtcrn_native/v2.rs`（605）。按职责拆，输出逐位不变（单元测试 + 对拍证明）；`apple-rs/tests/fields.rs` 读 `bench/src/main.rs` 的输出格式串当真相源——拆后若 bench 的 println 移到别的文件，那边的路径一并改（或告诉 applebe）

## 进度
- [x] 报到；调研 workflow 已起（3 调研 + 3 怀疑派）
- [x] bench 补计时窗字段 `win_start_unix` / `win_end_unix` / `wall_s` + 测试 `bench_emits_window_fields`（已编译、测过）
- [x] 逐层打点 + 计算量统计（gtcrn_native.rs，`infer_probed` + `Cost`；交叉校验测试 4 条）
- [x] 本机峰值微基准（hw-peak）· 逐层计时（gtcrn-layers）—— 计时中 `results/20261102-peak1/`
- [x] 调研 workflow 完成（6 代理）：`docs/research/020_fastpath/notes/prior-art-raw.json`
- [x] 等价改写清单 `docs/research/020_fastpath/rewrites.md`
- [x] native-v2 / native-v2-fast 代码（`bench/src/gtcrn_native/v2.rs`，未编译，等计时结束）
- [x] dsp-bench（前后处理计时 + 重采样点积对照，未编译）
- [x] 第一段计时 `results/20261102-peak1/`（负载 5.7 开跑，两项微基准作废，已注明）
- [x] v2 / v2-fast：BE-PARITY 过（134.99 / 132.72 dB）· 每帧 0 分配 · 单元测试 27 个
- [x] 第二段第一次 `results/20261102-v2/`：中途拔电 + 系统睡眠 42 分钟，只作电池对照（README 已写）；已加睡眠检测与插电检查
- [ ] **等人插电** → 大总管建锁 → `run_v2.sh 20261102-v2ac`（约 32 分钟）
- [ ] 下界表（bound_table.py）
- [ ] 耗电：`sudo bash docs/experiments/exp012-fastpath/tools/power_v2.sh 20261102-pw1`（人跑，约 13 分钟）
- [ ] decision.md · codex 评审 · PR

## 经验
- 长计时一律 `caffeinate -dims` 且查插电：拔电后系统空闲睡眠，单调时钟停走，进程像「卡在 sleep」；电池模式性能核约降半频
- [ ] 结论文档 · codex 评审

## 转交
（暂无）
