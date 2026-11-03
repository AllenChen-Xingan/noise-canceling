# 现状页 · fastpath（研究 020 / 实验 exp012 · issue #58 · 分支 exp/012-fastpath）

管的实体：推理后端 · 模型（GTCRN）· 评测器（帧耗时 · 逐值一致 · 耗电）

## 合并前必做（大总管定 2026-11-02）
- [x] applebe 的 019 先合进 main；之后本分支 `git merge main`（85b0015e）
- [x] `apple-rs/tests/fields.rs` 的 `APPLE_ONLY` 删掉 `win_start_unix` / `win_end_unix` / `wall_s` 三条，`cargo test --test fields` 2 passed（93571f9f）
- [x] **所有代码文件 ≤ 600 行**：gtcrn_native.rs 1926 → 445（拆 weights / cost / kern / ops / block / dpgrnn / simd / tests）· main.rs 1280 → 485（拆 alloc / nets / util / parity）· v2.rs 605 → 532；最大 553（v3.rs，之后加了测试）。证据：单元测试全过（现 36 个）；11 个手写后端对 ORT 的最大差与 SNR 拆前拆后一字不差
- [x] codex 换谱系评审：3 轮，第 2、3 轮连续无 P0/P1，满足停止线（`docs/reviews/020_fastpath-codex.md`）；剩 3 件 P2 待开 issue（decision.md「后续」）
- 推送、开 PR（Closes #58）、合并由大总管做

## 结论（详见 `docs/research/020_fastpath/decision.md`）
- 默认候选 **native-v4i**（大总管定）：16 ms 节奏、电池 3 轮每帧 0.073–0.092 ms · CPU 0.5–0.7 % · BE-PARITY 132.58 dB · 0 分配
- 原写法离性能核下界 15.7 倍 → v4i 2.5 倍；节奏下 v4i 是能效核下界（42.9 µs）的 1.7–2.1 倍
- 撤掉第 14 条（倒数代替除法，实测更慢）

## 未测（待产品接入时测）
耗电（`exp012/tools/power_v2.sh`，sudo，人跑）· 插电节奏组 · 预取整帧（无结论）· 能效核节奏下逐层 · v4i 10 分钟长跑对拍 · BE-IMPACT

## 转交
- 「接起来」：产品接入 v4i；`src/dsp/src/resample.rs` 的 `dot` 改多累加器（节奏下约 60 → 14 µs/帧）
- 所有测速的人：rt 线程连续跑会被降级（L1 读 105 → 约 30 GB/s），连续跑峰值用 `--qos ui`；长计时 `caffeinate -dims` + 查电源

## 经验
- 长计时一律 `caffeinate -dims` 且查电源：拔电后系统空闲睡眠，单调时钟停走，进程像「卡在 sleep」；电池模式性能核约降半频
- shell 里变量后紧跟中文全角字符要写 `${v}`：在 caffeinate 启动的环境里 bash 会把多字节字符吃进变量名
- macOS 的 `seq 2 1` 会倒数，循环轮转用 `for ((i = 1; i < r; i++))`
