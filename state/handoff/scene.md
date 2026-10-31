# 现状页 · scene（批次 20261031-b4）

- **状态**：进行中
- **手上**：场景识别与路由（`vdn-scene`）+ 修音链（`vdn-polish`）+ 模拟环境第一版（exp006）
- **交代**：主目录 `_tmp/20261031-b4/scene.md`、`common.md`
- **分支**：`feat/scene-polish`（worktree `../noise_canceling-scene`）

## 流水
| 时间 | 事 |
|---|---|
| 10-31 | 报到；读交代、005、D32–D41、Rust 规约 §1–§3、exp003 README / make_clips |
| 10-31 | 向大总管报计划 + 问根 Cargo.toml members；问 p2 VAD 接口对齐 |
| 10-31 | DEMAND 补下 PRESTO / STRAFFIC / TBUS / OOFFICE（16k，md5 记 exp006 README） |
| 10-31 | exp006 场景合成器 `sim.py` + 参数表 `recipes.json`；build 20261030214214-1164（560/280/16）→ 20261030214530-44df（1680/280/16，现用） |
| 10-31 | `vdn-polish`（频谱平滑 · 下限 + 舒适噪声 · 去发闷 · AGC + 限幅，零前瞻）与 `vdn-scene`（35 维特征 · MLP · 设备施密特 · 滞回 · 路由表）编译过、clippy 干净 |
| 10-31 | 名额内 release 构建 + 特征提取；算力约 0.07% 个核；已归还 |
| 10-31 | 训练 + 扫参：已生效准确率 大类 87% / 用户层 93%；换场景中位 3.7 s 生效；滞回默认值发 align；FrameFeatures 字段定稿建议发 align |
| 10-31 | 测试 10 个（含两处 assert_no_alloc）全过；限幅释放段超限 bug 由测试抓到并修；clippy / fmt 干净 |
| 10-31 | `.42cog/cog.md` 加 vdn-polish、vdn-scene 两行；exp006 README 初稿（场景部分完成） |
| 10-31 | `offline_ab.py`（4 版：基线 / 路由 / 修音链 / 都开）冒烟 3 段通过；**等名额**跑全量推理 + 打分 |
| 10-31 | 派子代理做 Opus BBWENet 带宽扩展可行性调研（只调研，不实现） |
| 10-31 | 自审①（实时 / Rust）：修 P0 静音开头卡死底噪、P1 平滑方向写反、P1 时域路径 AGC 失效、P1 手动「会议」不生效、P2 若干；加回归测试 |
| 10-31 | 自审②（交付合规）：FE-M 场景改只开 AGC（实时做不到零延迟的频谱节点不进离线版）；盲听页无映射、检验题中性化、听测目录分离、生成时自检；crate 根补 tidy lint，spectral.rs 去下标重写；unblind.py |
| 10-31 | 大总管转人问：FE-M 修音怎么办 → `notes/fem-polish.md`（官方有 spec2spec 版；Hann/hop160 Σw² 每 10 ms 起伏 0.14 dB）；BBWENet 可行性 → `notes/bwe-feasibility.md` |
| 10-31 | 名额内 4 版推理 + 打分（244 段，ASR 92 段）；AGC τ 0.6 s 反增起伏 → 离线比 0.6/2/4 s 选 4 s → 整批重跑；第一版分数留 `ab_scores_agc-tau0.6s.json` |
| 10-31 | 盲听页 `_tmp/20261031-exp006-listen/20261030214530-44df/pages/20261030234932-1a0b/index.html`；对照表已入 docs；作废 4 个 build 留痕 |
| 10-31 | 自审③（实验方法怀疑派）进行中 |
