# 现状页 · fe3：评测器 4 处改调统一频谱模块，评分逐位不变（#104）

- 分支 `feat/frontend-eval`（worktree `../noise_canceling-fe3`），基于 main `f2782cf4`
- 管的实体：评测器的频谱计算（4 处）· 频谱前端（只在必要时小扩展）· 成绩单对拍（验收用）
- 状态：**已收尾**（2026-11-04）

## 记录

- 2026-11-04 报到；读完交代、共同规矩、013 第 11–12 节、评测器 4 处源码、`vdn_dsp::fft` / `window` / `stft`
- 2026-11-04 方案已发大总管，等点头（见下「方案」）
- 2026-11-04 大总管点头；另准多写 `fft_site_tests.rs` · `window_tests.rs`
- 2026-11-04 代码与测试写完（未编译）：4 处 · 新窗值 · R-FFT 白名单缩到 1 条 · `eval/tests/spectrum_golden.rs`（新）· `eval/tests/common/platform.rs`（复制，逐字比对）· 013 第 13 节
- 2026-11-04 一次性跑的脚本在草稿目录 `run-all.sh`；等名额
- 2026-11-04 名额到；大总管另准删 `src/eval/Cargo.toml` 的 realfft 依赖（Cargo.lock 随之变）
- 2026-11-04 第 1 次跑：函数级对拍改后逐位相同；成绩单（不带 `--qa`）去 meta 后逐字相同；xtask ci 红在 fmt + tidy（新测试文件的格式、采样率字面量）→ 已修，ESTOI 内部采样率改引用 `estoi::FS`（公开），不留第二份
- 2026-11-04 发现 `vdn-scene` golden 在 `--release` 下红、debug 下绿；改前源码 release 同样红、哈希相同 → 不是本 PR 造成，大总管开 #106
- 2026-11-04 第 2 次跑：xtask ci 全绿；带 `--qa` 成绩单去 meta 后逐字相同；识别缓存 80 条全同
- 2026-11-04 codex 第 1 轮：P0 0 · P1 0 · P2 3，全属实已改（补边界输入、断言 V-WEAK 进评分分支、写清基准生成步骤）；在旧源码上重生成基准，改后 release / debug 逐位相同，xtask ci 全绿；名额已归还
- 2026-11-04 codex 第 2 轮：P0 0 · P1 0 · P2 1 → 连续两轮无 P0/P1，达停止线。这条 P2 属实（截短片段取在开头静音段，没测到分帧边界），已改为从主讲人开口处截取并重生成基准，改后 release / debug 逐位相同；fmt · tidy · clippy（vdn-eval）通过
- 2026-11-04 评审记录 `docs/reviews/104_frontend-eval-codex.md`；PR 正文草稿 `_tmp/20261104-b13/pr-fe3.md`

## 方案（读源码；已实现，见第 13 节）

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
- `src/eval/src/verifiers/estoi.rs` 的 `FS` 改为公开（测试引用，不留第二份）
- `src/eval/tests/fixtures/spectrum_golden.txt`（新，基准）· `src/eval/Cargo.toml`（删 realfft）· `Cargo.lock`（随之变）
- `docs/plan/013_frontend-once.md`（第 13 节）

## 成绩单级对拍（实测，2026-11-04）

- 命令：`vdn-eval scorecard --suite <只含 S-cafe-common 的临时任务文件> --system passthrough,ref,gtcrn [--qa] --out <新目录>`；改前用 `f2782cf4` 导出源码的 release 二进制，改后用本分支
- 范围：cafe 常见档 Air 内置麦 10 段 × 3 个系统 = 30 段
- 去掉的字段：`scorecard.json` 的 `meta` 整块（run_id · commit · suite 路径 · systems · qa）
- 比了：其余全部（每场景每项 mean / worst / worst_clip / 过·不过·不知道段数 / 阈值 / 状态 / 拖后腿分析）+ `scorecard.md` 全文
- 结果：不带 `--qa` 逐字相同；带 `--qa`（吃字 · SCOREQ · Distill-MOS 都真出分）逐字相同
- 吃字逐段：成绩单只有场景汇总；改比识别缓存——两次各 80 条（40 个音频 × 2 次转写），键（含音频 sha256）与文字全同，逐段差 0、最大差 0
- 缓存：每次 `--qa` 用新建的空缓存，两次之间不共用；同一次里两次转写键也不同。未核实：识别服务端自己有没有缓存

## 待人拍板

（暂无）

## 转交（评分算法疑点，给评测器大总管）

- V-WEAK 在采样率低于约 9 kHz 时 panic：最高一带 4 kHz × 2^(1/6) ≈ 4.49 kHz 超过奈奎斯特，`speech.rs` 切片越界（实测：8192 Hz 时 bin 561 > 513）。改前改后一样，本 PR 不改；测试改用 16384 / 22050
- `vdn-scene` golden 在 `--release` 下红、debug 下绿，改前就有（不是评测器）→ #106
