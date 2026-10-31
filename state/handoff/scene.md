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
