# 015 · DFN 实时版 · Hush 与竞争说话人 · 合成数据（给辅助模型的 RL 训练）

> 2026-10-31 开线。人原话：「dfn 的实时版也看下 / 然后就是合成数据以及之类的东西 / 我们如果要做特定的辅助模型，可以专门拿来用强化学习 / 以及各种合成的东西 / 另外还有hush / 这种就是竞争性说话人的东西，是否也可以看下」
> 起因：读到一篇降噪选工具的文章（自然度档位 · 舒适底噪 · 不叠加降噪；DeepFilterNet 两段式保谐波；WHAM! 真实噪声；WHAMR! 加混响），对照决策记录后发现三处缺口（见本线 decision.md 第〇节）。

## 三路

| 路 | 问题 | 笔记 |
|---|---|---|
| A | DeepFilterNet 一系的实时版（DFN2/3 `_ll`、libDF/tract、LADSPA、DPDFNet、Hush）能不能进 ≤45 ms 管线；权重许可；`atten_lim` 等自然度旋钮 | `notes/A-dfn-realtime.md` |
| B | 合成数据配方：给 D47 辅助小模型和 013 RL 环境用的混合 / RIR / 噪声 / 竞争说话人生成方法与许可 | `notes/B-synth-data.md` |
| C | Hush 深读 + 竞争说话人（background speaker suppression）方向的其他开源工作；Hush 的训练招能否搬到我们 | `notes/C-hush-competing.md` |

结论与待拍板：`decision.md`。
