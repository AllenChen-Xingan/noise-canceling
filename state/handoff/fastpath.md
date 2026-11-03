# 现状页 · fastpath（研究 020 / 实验 exp012 · issue #58 · 分支 exp/012-fastpath）

管的实体：推理后端 · 模型（GTCRN）· 评测器（帧耗时 · 逐值一致 · 耗电）

## 合并前必做（大总管定 2026-11-02）
- [ ] applebe 的 019 先合进 main；之后本分支 `git merge main`
- [ ] 在 `docs/experiments/exp010-inference-bench/apple-rs/tests/fields.rs` 的 `APPLE_ONLY` 里删掉 `win_start_unix` / `win_end_unix` / `wall_s` 三条，跑该测试转绿

## 进度
- [x] 报到；调研 workflow 已起（3 调研 + 3 怀疑派）
- [x] bench 补计时窗字段 `win_start_unix` / `win_end_unix` / `wall_s` + 测试 `bench_emits_window_fields`（已编译、测过）
- [x] 逐层打点 + 计算量统计（gtcrn_native.rs，`infer_probed` + `Cost`；交叉校验测试 4 条）
- [x] 本机峰值微基准（hw-peak）· 逐层计时（gtcrn-layers）—— 计时中 `results/20261102-peak1/`
- [x] 调研 workflow 完成（6 代理）：`docs/research/020_fastpath/notes/prior-art-raw.json`
- [x] 等价改写清单 `docs/research/020_fastpath/rewrites.md`
- [x] native-v2 / native-v2-fast 代码（`bench/src/gtcrn_native/v2.rs`，未编译，等计时结束）
- [x] dsp-bench（前后处理计时 + 重采样点积对照，未编译）
- [ ] 下界表
- [ ] 结论文档 · codex 评审

## 转交
（暂无）
