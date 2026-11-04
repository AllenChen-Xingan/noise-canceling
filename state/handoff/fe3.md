# 现状页 · fe3：评测器 4 处改调统一频谱模块，评分逐位不变（#104）

- 分支 `feat/frontend-eval`（worktree `../noise_canceling-fe3`），基于 main `f2782cf4`
- 管的实体：评测器的频谱计算（4 处）· 频谱前端（只在必要时小扩展）· 成绩单对拍（验收用）
- 状态：进行中

## 记录

- 2026-11-04 报到；读完交代、共同规矩、013 第 11–12 节、评测器 4 处源码、`vdn_dsp::fft` / `window` / `stft`
- 2026-11-04 方案已发大总管，等点头（见下「方案」）
- 2026-11-04 大总管点头；另准多写 `fft_site_tests.rs` · `window_tests.rs`
- 2026-11-04 代码与测试写完（未编译）：4 处 · 新窗值 · R-FFT 白名单缩到 1 条 · `eval/tests/spectrum_golden.rs`（新）· `eval/tests/common/platform.rs`（复制，逐字比对）· 013 第 13 节
- 2026-11-04 一次性跑的脚本在草稿目录 `run-all.sh`；等名额

## 方案（读源码，未构建）

| 位置 | 改成 | 窗 | 精度 | 为什么逐位不变 |
|---|---|---|---|---|
| `speech.rs` `band_levels` | `FrameFft<f64>`：`win_len` = 0.125·sr，`n_fft` = 下一个 2 的幂，`hop` = 窗长一半；整段先转 f64 一次 | `HannPeriodic` | f64 | 窗公式运算顺序与原式相同；`(x as f64)·w` 与 `w·(x as f64)` 乘法交换不变；帧数口径 `s + win ≤ n` 等于 `snip_edges` |
| `estoi.rs` `hann_inner` · `tob` | 窗表改 `window::table::<f64>(HannNumpyInner, 256)`（静音帧去除与重叠相加也用它）；FFT 改 `FrameFft<f64>` 256 / 512 / 128；帧号仍按原 `0..len−256` 步长 128 自己数 | `HannNumpyInner` | f64 | 原式 i = 1..n 与 `coef` 的 i+1 同序 |
| `system.rs` 参考降噪器 | `StreamingStft::from_spec(&FRONTEND)`；`N_FFT` / `HOP` 改为取自 `FRONTEND` | `SqrtHannPeriodic` | f32 | 同一张窗表 |
| `analysis.rs` 声谱图 | 新加窗值 `SqrtHannPeriodicSquared`（先按 f64 算 sqrt-Hann、转样本类型、再在样本类型里平方）；`StreamingStft::new(n, hop, window::table(..))` | 新值 | f32 | 与现写法同序；`HannPeriodic` 先 f64 后转，末位不同，不能代用 |

## 改了哪些文件

- `src/types/src/frontend.rs`（+1 窗值）· `src/dsp/src/window.rs` · `src/dsp/src/window_tests.rs`
- `src/eval/src/verifiers/speech.rs` · `estoi.rs` · `src/eval/src/system.rs` · `src/eval/src/analysis.rs`
- `src/eval/tests/spectrum_golden.rs`（新）· `src/eval/tests/common/platform.rs`（新，场景那份的逐字复制）
- `src/xtask/src/fft_site.rs` · `fft_site_tests.rs`
- `docs/plan/013_frontend-once.md`（第 13 节）

## 待人拍板

（暂无）

## 转交（评分算法疑点，给评测器大总管）

（暂无）
