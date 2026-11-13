# 030 · 来源

只列本线用到的；路径相对仓库根。标记：【仓内】读了源码或文档 ·【实测】本线跑出来的 ·【推断】。

## 一、混回比例的出处

| # | 来源 | 说了什么 | 级别 |
|---|---|---|---|
| S1 | `src/types/src/desc/mod.rs` 的 `FLOOR_DB` · `OPEN_DB` · `DETECT_DB` · `RELEASE_S` · `HANGOVER_S` · `ATTACK_S` | 参数正本：停顿下限 −12 dB、说话时 −6 dB、门控灵敏度 9 dB、慢关 0.3 s、保持 0.15 s、快开 2 ms | 【仓内】 |
| S2 | `src/engine/src/engine.rs` 第 1–10 行、`mix_out` | 输出 = 降噪 + m ×（原声 − 降噪）；有声时 m = 10^(open_db/20)，停顿时回到 10^(floor_db/20) | 【仓内】 |
| S3 | `src/dsp/src/envelope.rs` 第 82–107 行 | 有声判定只看模型输出的能量，与跟踪的底噪比；低于绝对门限的帧不参与跟踪 | 【仓内】 |
| S4 | `docs/plan/003_tech-design.md` 第 36 行 | 「压低也有下限（初值 −12 dB，不压成真空）」：−12 的最早出处；原文是认人门控的下限 | 【仓内】 |
| S5 | `docs/research/006_sota-landscape/decision.md` 第 3 条 | 「背景保留」旋钮 + 快开慢关 + 出事送原声，默认偏保守；默认值要由吃字率 + 人耳 AB 标定 | 【仓内】 |
| S6 | 提交 `35d3b6e3`（2026-10-30，`src/types/src/params.rs`） | −12 / −6 首次写进代码；提交与相邻文档都没写 −6 的依据 | 【仓内】 |
| S7 | `docs/research/018_ondevice-judge/decision.md` 第 143 行 | 下限 −6 dB 的两组红线通过率最高，两个评分模型却给最低分；作者在咖啡馆偏好留一点背景（低置信）。这个取舍评分模型判不了，要人耳 | 【仓内】 |

结论：−12 dB 来自技术设计稿的初值，−6 dB 没有查到出处；006 说的「用吃字率 + 人耳标定」至今没做【仓内 · 本线检索】。

## 二、场景判别与路由（已定与已做）

| # | 来源 | 内容 | 级别 |
|---|---|---|---|
| S8 | `docs/plan/001_decisions-20261029-b1.md` D40 | 场景路由：不同场景自动换一组参数，慢慢过渡，场景内学偏好 | 【仓内】 |
| S9 | 同上 D41 | 三层：用户层少数基本场景 → 内部细场景（自动、滞回、平滑、无感）→ 场景内自适应；细场景逻辑用模拟环境开发 | 【仓内】 |
| S10 | 同上 D52 | 用户电脑上改为「训练过的调节模型」直接输出怎么调；D40 / D41 的场景路由降为调节模型的输入或兜底 | 【仓内】 |
| S11 | `src/scene/`（`classify.rs` · `features.rs` · `route.rs` · `lib.rs`） | 已实现：35 维便宜特征、2 s 缓冲、约 0.5 s 判一次；7 类噪音（安静 · 空调 · 人声嘈杂 · 邻座说话 · 杯碟 · 交通 · 音乐）；一层 24 个隐元，约 800 次乘加；滞回门（连续确认、最短停留）；路由表只切模型与修音预设 | 【仓内】 |
| S12 | `docs/research/018_ondevice-judge/notes/r3-sound-to-params.md` 第 48–49 行 | 调节模型形态：35 维特征 → 2 层 MLP（约 7 千参数）→ strength · floor_db · open_db 等；先只回归 strength / floor_db / detect_db | 【仓内】 |
| S13 | `docs/research/021_meeting-scenes/decision.md` | 15 个开会地点；公开资料里没有按开会地点分的带会议语音的分类集 | 【仓内】 |

## 三、本线实测

| # | 文件 | 内容 |
|---|---|---|
| E1 | `docs/experiments/exp021-gtcrn-aux/runs/20261105-parity-dfn3/` | 预混 + 关保命 = 原路径，21 项逐段差 0（DFN3，480 段） |
| E2 | `docs/experiments/exp021-gtcrn-aux/runs/20261105-parity-gtcrn/` | 内置 GTCRN 与外部入口差在预热段；外部入口降噪量偏 −0.108 dB [−0.133, −0.083] |
| E3 | `docs/experiments/exp021-gtcrn-aux/runs/20261105-sweep-dfn3/` | DFN3 九档扫描 |
