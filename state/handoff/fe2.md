# 现状页 · fe2：统一频谱模块支持 Kaldi 分帧，认人 fbank 收进来（#102）

- 分支 `feat/frontend-speaker`（worktree `../noise_canceling-fe2`），基于 main `d23bfe66`
- 管的实体：频谱前端（`vdn_types::frontend` · `vdn-dsp`）· 认人前端（`src/speaker/src/fbank.rs`）· 离线例子
- 状态：进行中

## 记录

- 2026-11-04 报到；读完交代、013 计划、第一段实现、fbank、评测器三处
- 2026-11-04 盘点完成（读源码 + grep，未构建），见下表；设计要点已发大总管等点头

## 盘点：全仓手写频谱 / 窗（读源码，grep `plan_fft|RealFftPlanner|hann|hamming|povey|cos\(`）

| 位置 | 内容 | 参数 | 精度 | 归属 |
|---|---|---|---|---|
| `src/speaker/src/fbank.rs:74-84` | Kaldi 窗（汉明 / 汉宁 / povey，对称，分母 n−1），先 f64 算再转 | 窗长 400 | f32 | **本 PR** |
| `src/speaker/src/fbank.rs:121` | 自建正向 FFT 计划，补零 400 → 512，帧移 160，整段分帧 | 512 | f32 | **本 PR** |
| `src/eval/src/verifiers/speech.rs:31-34` | 周期 Hann + 正向 FFT，补零到 2 的幂（2000 → 2048） | 0.125 s / 半窗帧移 | f64 | 评测器下一 PR |
| `src/eval/src/verifiers/estoi.rs:106-110,181-182` | numpy `hanning(n+2)[1:-1]` + 正向 FFT | 256 / 512 | f64 | 评测器下一 PR |
| `src/eval/src/system.rs:47` | 参考降噪器手写 512 / 256 / sqrt-Hann 建 `StreamingStft` | 512 / 256 | f32 | 评测器下一 PR（改 `from_spec`） |
| `src/eval/src/analysis.rs:163` | 声谱图：sqrt-Hann 在 f32 里平方当窗，手写 n_fft / hop | 调用方给 | f32 | 评测器下一 PR（交代里的 3 处之外，多出这 1 处） |
| `src/scene/src/features.rs:146` | 逆 FFT：功率谱 → 自相关求谐波度 | 512 | f32 | 白名单（013 第 5 节：留着） |
| `src/dsp/src/resample.rs:41` | Blackman 窗 | — | f64 | 不是频谱（重采样滤波器设计） |
| `src/eval/src/synth.rs:159,162` | 升余弦淡入淡出 | — | f64 | 不是频谱（包络） |
| 离线例子 `polish_offline` · `scene_offline` · `frontend_timing` | 已在第一段改为引用 `FRONTEND` / `StreamingStft::from_spec` | — | — | 无需再改 |

## 待人拍板

（暂无）

## 转交

（收尾时填）
