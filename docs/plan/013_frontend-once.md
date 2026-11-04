# 013 · 每帧只算一次频谱：降噪 · 场景 · 修音共用（#94，D71）

> 状态：大总管 2026-11-04 点头（修音槽不做，见第 4 节）。依据：读源码盘点（`state/handoff/fecore.md`「盘点」），未构建、未实测。
> 范围（第一段，本线）：场景特征与修音借降噪模型的谱。认人不借降噪的谱（大总管 2026-11-04 定）。
> 人改定（D82，2026-11-04）：仓里所有频谱计算只在一个模块实现，认人保留自己的分帧、各自切法。第一段立好这个模块；认人 fbank、评测器、离线例子收进来是第二段，见第 11 节。

## 1. 现在的毛病

- 512 / 256 / 257 这组参数手抄在 4 处：GTCRN、场景特征、两个离线例子
- 场景特征自己又建一套 STFT（窗、缓冲、FFT 计划），和 GTCRN 的分析逐步相同（读源码核对，见现状页）
- 原借谱接口（借幅度谱的那个方法）没人实现，也没人调用；它给的是幅度，场景要的是功率（幅度再平方不逐位相同）

## 2. STFT 参数正本

放 `vdn-types`（零依赖，`MODEL_RATE` 也在这里）新文件 `frontend.rs`：

```rust
pub enum Window { SqrtHannPeriodic }
pub struct StftSpec { pub n_fft: usize, pub hop: usize, pub sample_rate: u32, pub window: Window }
impl StftSpec { pub const fn bins(&self) -> usize { self.n_fft / 2 + 1 } }
pub const FRONTEND: StftSpec = StftSpec { n_fft: 512, hop: 256, sample_rate: MODEL_RATE, window: Window::SqrtHannPeriodic };
```

- `vdn-dsp` 加 `StreamingStft::from_spec(&StftSpec)`，窗种类 → 窗函数只在这一处对应（dsp 新增依赖 vdn-types，后者零依赖）
- GTCRN、场景、两个离线例子删掉本地常量，改引用 `FRONTEND`
- 比对：场景编译期断言 `FRONTEND` 的点数、帧移、采样率、窗种类都等于权重训练时的值；GTCRN 已有测试把 `mix` 形状与频点数比对，改为与 `FRONTEND.bins()` 比
- 能扩展：`StftSpec` 是通用的口径描述，`FRONTEND` 只是其中一份实例，512 / 256 只写在这份实例里。第二段要的「窗长 ≠ FFT 点数（补零）」「更多窗种类」都是给它加字段、加枚举值，现有实例取默认值，结果不变，见第 11 节

## 3. 借谱接口（正本归 fecore）

```rust
pub trait SpectrumTap: Model {
    fn spectrum_spec(&self) -> StftSpec;             // 谱的口径，吃谱方每帧与自己要的口径比对
    fn input_power(&self) -> Option<&[f32]>;          // 本帧带噪输入功率谱 re²+im²，长度 bins；推理前取；还没处理过帧为 None
}
```

- 换掉原来借幅度谱的方法（无人用）。给功率不给幅度：场景只用功率，逐位可复现
- 不给输出谱（认人不接）。不给复数谱：`vdn-types` 零依赖，没有复数类型

## 4. 功率谱只算在一处；GTCRN 借出

不新建结构（原稿的 `SpectrumFrontend` 删掉：没有修音槽，它只剩一个 Vec，不值一个类型）。

- `vdn_dsp::stft::power_into(spec, out)`：功率 `re²+im²` 的唯一写法。GTCRN 借出与场景自算都调它，逐位相同靠构造保证
- `StreamingStft` 加 `feed(hop)`（只移位、不做 FFT）与 `forward_count()`（测试用：正向 FFT 次数）
- GTCRN：`analyze` 之后、推理之前 `power_into` 到自己的 `power` 缓冲，实现 `SpectrumTap`
- 「一个模块」落在哪：参数正本在 `vdn-types::frontend`，实现（窗表 `window::for_spec`、流式 `StreamingStft`、功率 `power_into`）在 `vdn-dsp`。第二段在 `vdn-dsp` 加按整段分帧的入口，两个入口共用同一段「加窗 → 补零 → 正向 FFT → 功率」，见第 11 节
- **修音频域级槽：不做**。原因：把修音接到模型里属于接线，不在本大事（大总管 2026-11-04 定，`SpectralStage` 留在 polish）；槽与 engine 接线留给 backlog 那件事

## 5. 场景特征

- `push_hop(x)`：自算（兜底，FE-M 用），内部改用 `StreamingStft::from_spec`，删掉自建的窗、缓冲、FFT 计划
- 新增 `push_hop_with_power(x, spec, power)`：吃现成功率谱；`x` 仍要（削波是时域算的）。`spec` 是借出方的口径（`spectrum_spec()`），不等于 `FRONTEND` 或长度不对就退回自算（自审 P2，`docs/reviews/013_frontend-once-selfreview.md`）
- 两个入口只差「功率从哪来」：各一行填 `self.pow`，之后同一段 `finish_frame` 算 35 维要用的帧量
- 借谱时也把 `x` 推进自算缓冲（`StreamingStft::feed`），换到 FE-M 时第一帧不吃旧数据
- 不整齐长度：旧代码按 `min(len, hop)` 移位、不补零。为保持逐位不变，`StreamingStft` 加 `feed_partial` / `analyze_partial`，场景自算走它们；借谱入口遇到 `x` 不是整 hop 就退回自算（codex 第 1 轮 P1 #1：「短输入改了数值行为」）。有对拍：`scene/tests/golden.rs::ragged_hops_match_golden`
- 逆 FFT（功率谱 → 自相关求谐波度）省不掉，留着
- `SceneAnalyzer` 同样加 `push_hop_with_power`

## 6. 拿不到谱的模型（FE-M）

不实现 `SpectrumTap`；场景走 `push_hop` 自算一次；修音只跑时域部分（现状不变）。

## 7. 处理图描述

不加新节点种类。场景特征不在音频路径上，本来就不进图。修音频域级已有种类 `POLISH_SPECTRAL`（挂点 Spectral）。以后模型里装修音槽时，要与 `ProcessingGraph::spectral` 比对——那是接线那件事（backlog）的活。本线不改图。

## 8. 测试

1. **GTCRN 对拍**：固定约 3 s 输入（截到整 hop，每个点都真过模型；codex 第 1 轮 P2 #4），输出 f32 逐位的哈希写进 `src/models/tests/fixtures/`。顺序：先在**没改过**的 `gtcrn.rs` 上生成哈希（名额放开后），再改 GTCRN。没模型文件时同 `smoke.rs` 跳过（设 `VDN_REQUIRE_MODELS` 不许跳）
2. 场景：借谱 vs 自算，35 维最大绝对差 ≤ 1e-5（预期 0）；场景判断逐帧相同。谱源用 `StreamingStft` + `power_into`（与 GTCRN 同一段代码），CI 上能跑。「改前的场景实现」的参照是两份基准：整 hop 一份、不整齐长度一份，都在改前的提交上生成。原稿说「把旧实现抄进测试」，没有这样做，因为那是一份 300 多行的副本。基准带平台标签：别的平台上跳过；CI 是 macos-15（arm），和生成基准的平台相同，所以 CI 上真比（codex 第 1 轮 P2 #3）
3. 计数：一个 `StreamingStft` 当模型前端 + 场景借谱 + 测试里手动接修音，每帧正向 FFT 总数恰好 1（前端 `forward_count` 增 1、场景自算 0）。真 GTCRN 版在 `models/tests/tap.rs::gtcrn_plus_scene_borrowing_does_no_extra_fft`：场景 0 次，判断与自算逐位相同。GTCRN 内部那 1 次由构造保证，没有对外计数
4. 修音（测试里手动接）：dry 用前端借出的那份 vs 另起一个 `StreamingStft` 自算的 dry，`PolishChain` 输出逐位相同
5. 0 分配：`assert_no_alloc` 包住 `power_into`、`feed`、场景借谱入口、GTCRN 借谱

注意：离线例子 `polish_offline` 的 wet 是「降噪后波形再分析」，和模型输出谱本来就不同，所以不拿它对拍；它只改为引用正本，输出不变。

## 9. 复杂度账（D71 第 2 条）

- 加：`StftSpec`（约 30 行）· `power_into` / `feed` / 计数（约 20 行）· 场景借谱入口（约 15 行）· dsp → types 一条依赖边（规约 §2.2 的图里本来就有；check-deps 只查禁用清单，dsp 的清单不含 types）
- 减：场景自建 STFT（约 25 行）· 4 处本地常量 · 没人用的借幅度谱方法
- 净增约 40 行（估计）、0 个新结构。每帧省一次 512 点正向 FFT（实测约 0.4 µs，见第 10 节）

## 10. 实测结果（2026-11-04）

**对拍**（`cargo test`，五个 crate 全绿；`xtask ci` 6 项全绿）：

| 判据（测试名） | 结果 | 阈值 | 过线 |
|---|---|---|---|
| GTCRN 改前改后输出（`models/tests/golden.rs`，约 3 s，截到整 hop 共 47872 点） | 逐位相同 | 逐位 | 是 |
| 引擎最终输出（同文件，48 kHz `Pipeline`，默认安全参数，块长轮换，144000 点） | 逐位相同 | 逐位 | 是 |
| 场景改前改后判断（`scene/tests/golden.rs`，12 s，整 hop，24 次判断） | 逐位相同 | 逐位 | 是 |
| 场景不整齐长度（同文件，含不足一个 hop、空、超过一个 hop，31 次判断） | 逐位相同 | 逐位 | 是 |
| 场景借谱 vs 自算（`scene/tests/frontend.rs`） | 35 维最大差 0，24 次判断全同 | ≤ 1e-5，判断全同 | 是 |
| 组合路径每帧正向 FFT 次数（同上） | 每帧 1 次 | 1 次 | 是 |
| 修音吃借来的谱 vs 自算（`polish/tests/borrowed_spectrum.rs`） | 逐位相同 | 逐位 | 是 |
| 真 GTCRN 借出的功率谱 vs 自算（`models/tests/tap.rs`） | 逐位相同 | 逐位 | 是 |

改前基准在 `990e7843`（本分支改动前）的原样源码上生成。

**每帧耗时**（verifier：`src/models/examples/frontend_timing.rs`）：

- 条件：Apple M5，插电（AC Power，跑前跑后都查了），`caffeinate -dims`，release。
- 跑法：默认 QoS 主线程，连续跑，不是实时线程，所以不会被系统降级。
- 样本：同一份源码，两种走法交替先后，每种 6 轮 × 1875 帧 = 11250 帧。
- 墙钟 10 s，与计算量吻合，中途没有睡眠。

| 走法 | 模型 p50 µs | 场景 p50 µs | 场景 p99 µs | 合计 p50 µs | 合计 p99 µs |
|---|---|---|---|---|---|
| 场景自算（等于改前） | 391.6 | 2.42 | 3.17 | 394.1 | 424.5 |
| 场景借谱（013） | 392.0 | 2.00 | 2.67 | 394.1 | 421.1 |

结论（实测，单次会话）：
- 场景每帧省约 0.4 µs（p50 降 17%）。
- 每帧合计没有可测出的变化，0.4 µs 只占一帧 16 ms 预算的约 0.003%。
- 本改动的价值在口径只有一份正本、每帧只做 1 次正向 FFT，不在提速。

## 11. 第二段：认人与评测器收进来（D82，另派手下，基于本分支）

人定：所有频谱计算放在同一处代码，各自切法。认人保留自己的 400 / 160 分帧和汉明 / povey 窗，不共用降噪的结果，声纹输出逐位不变。下面是第一段留下的接口怎么容纳它（读源码判断，未实现）。

**要收进来的几处**（读源码）：

| 位置 | 窗长 / FFT 点数 / 帧移 | 窗 | 精度 | 分帧方式 |
|---|---|---|---|---|
| `src/speaker/src/fbank.rs` | 400 / 补零到 512 / 160 | 汉明、povey、汉宁（对称，分母 n−1） | f32 | 整段，`snip_edges` |
| `src/eval/src/verifiers/speech.rs` | 0.125 s / 补零到 2 的幂 / 窗长一半 | 周期 Hann | f64 | 整段 |
| `src/eval/src/verifiers/estoi.rs` | 256 / 512 | numpy `hanning(n+2)[1:-1]` | f64 | 整段 |
| `src/eval/src/system.rs` 参考降噪器 | 512 / 512 / 256 | 周期 sqrt-Hann | f32 | 流式（已用 `StreamingStft`，只差改引用正本或写明不跟） |

**接口怎么长（都是加东西，不改现有的）**：

1. `StftSpec` 加 `win_len`（窗长，≤ `n_fft`，尾部补零到 `n_fft`）。`FRONTEND` 取 `win_len = n_fft`，降噪、场景、修音的结果不变。
2. `Window` 加枚举值：汉明对称、povey、汉宁对称、numpy 内缩汉宁、周期 Hann。窗表仍只写在 `vdn_dsp::window::for_spec` 一处。每种窗按各处现有写法实现（例如认人是先用 f64 算再转 f32），保证逐位不变。
3. 每帧预处理钩子：分帧之后、加窗之前，调用方可以改这一帧（闭包或 trait）。Kaldi 的去直流、预加重、×32768 留在认人自己的 crate 里，统一模块不认识它们。
4. f32 / f64 都通用：评测器现在用 f64 FFT，改成 f32 分数会变。所以实现对样本类型泛型（realfft 两种都支持）。
5. 按整段分帧的入口（暂名 `FrameFft`）：给整段信号和帧号，取出那一帧，做钩子 → 加窗 → 补零 → 正向 FFT，可选再出功率。流式的 `StreamingStft` 与它共用这一段。

**验收**（第二段的测试）：

- 每处改前改后逐位对拍：声纹 fbank 输出、评测器分数、参考降噪器输出。做法同第一段：在改前的提交上生成基准。
- `FRONTEND` 加了 `win_len` 以后，第一段的 GTCRN 和场景对拍仍逐位相同。
- 全仓 grep：除 `vdn-dsp` 外没有 `plan_fft_forward`（只管正向 FFT 的创建；场景求自相关的逆 FFT 不在此列，第 5 节说它留着。要收进来就另加验收；codex 第 2 轮 P2 #3）。可以写成一条 tidy 规则，把「只在一处」变成机器检查。

**估计改动量**：types 约 +30 行，dsp 约 +100 行；认人 fbank 约 −30 行，评测器 3 处约 −40 行；每处配一条逐位对拍测试。

## 12. 第二段 · 实现（#102，手下 fe2，2026-11-04）

> 范围：统一模块扩展 + 认人 fbank 改调它 + 「只一处」机器检查。评测器不在本 PR（下一 PR，见本节末清单）。
> 依据：读源码实现；测试结果见下表（名额放开后实测填写）。

**正本与实现**

- `vdn_types::frontend`：`StftSpec` 加 `win_len`（`FRONTEND` 取 512，编译期断言 `win_len == n_fft`）与 `is_valid()`；`Window` 加 5 个值：`HannPeriodic` · `HannNumpyInner` · `KaldiHamming` · `KaldiHanning` · `KaldiPovey`
- `vdn_dsp::window::coef`：窗公式全仓唯一一处。每种照抄原调用方的运算顺序（Kaldi 是 `(2π/(n−1))·i`，周期窗是 `2π·i/n`），先 f64 算再转
- `vdn_dsp::fft`（新文件）：
  - `Sample`：f32 / f64 两行 impl，样本类型泛型，不手抄两份
  - `FftCore<T>`（crate 内部）：「取一帧 → 钩子 → 加窗 → 尾部补零 → 正向 FFT → 计数」只写在 `transform` 一处
  - `FrameFft<T>`：按整段分帧入口，第 k 帧取 `x[k·hop .. k·hop + win_len]`；`frame_with(x, k, 钩子)`，越界返回 `None`；`num_frames` 按 `snip_edges`
  - `power_into<T>`：功率公式唯一一处，改为泛型，f32 调用方不变
- `StreamingStft` 改为持有 `FftCore<f32>`，正向那段与 `FrameFft` 共用；逆变换与重叠相加不变。`from_spec` 要求 `win_len == n_fft`（流式合成不补零）
- 认人 `fbank.rs`：删掉自建 FFT 计划、窗公式和自有的 `WindowType`（第二处表示），改用 `vdn_types::Window` 与 `FrameFft` + `power_into`。去直流 · 预加重 · 输入缩放作为每帧钩子留在认人 crate；mel 滤波器组留在认人 crate（频谱之后的特征）。`vdn-speaker` 不再直接依赖 `realfft`
- 离线例子：`polish_offline` · `scene_offline` · `frontend_timing` 第一段已引用 `FRONTEND`，本段不用改

**「只一处」的机器检查**：tidy 新规则 R-FFT（`src/xtask/src/fft_site.rs`）。`src/` 下除 `src/dsp/src/` 外不许出现 FFT 计划标识符（`RealFftPlanner`、`FftPlanner*`、`plan_fft_*`）。白名单 3 条，各写原因；白名单只许缩短（文件不在了或已干净也报红）。边界：认不出手写窗公式与手写 DFT，靠评审守。

**对拍基准**：在 `d23bfe66` 的原样源码上生成（`git archive` 导出、单独 target 目录、`VDN_BLESS=1`）。改前的窗种类叫 `WindowType::{Povey, Hanning}`，生成时只把 `src/speaker/tests/golden.rs` 里 `old_names` 的两行换成旧名，其余逐字相同。

**测试**

| 判据（测试名） | 比什么 | 阈值 |
|---|---|---|
| `speaker/tests/golden.rs::fbank_matches_golden` | 3 种配置（汉明 ×32768 · povey 不放大 · 汉宁 256 不补零不预加重）fbank 输出哈希 vs 改前 | 逐位 |
| `speaker/tests/golden.rs::embedding_matches_golden`（`onnx`，有模型才跑） | 声纹向量哈希 vs 改前 | 逐位 |
| `dsp/src/window_tests.rs` | 6 种窗 vs 改前各处原公式（f32 / f64），6 种长度 | 逐位 |
| `dsp/src/fft_tests.rs::whole_matches_streaming_bitwise` | 整段入口 vs 流式入口 | 逐位 |
| `dsp/src/fft_tests.rs::zero_padding_is_correct` · `f64_path_matches_manual` | 补零 400 → 512；f64 2000 → 2048，vs 直接调 realfft | 逐位 |
| `dsp/tests/no_alloc.rs::frame_fft_does_not_allocate` | 整段入口 f32 / f64 每帧不分配 | 0 次分配 |
| `xtask` `fft_site_tests.rs` | R-FFT 负例红、正例不红、白名单只缩短 | — |
| 第一段 4 份对拍（GTCRN · 引擎 · 场景两份） | 加 `win_len` 后仍逐位 | 逐位 |

**交给评测器下一 PR 的清单**（本 PR 不碰 `src/eval/`）：

| 位置 | 改成 | 窗 |
|---|---|---|
| `src/eval/src/verifiers/speech.rs:31-34` | `FrameFft<f64>`，`win_len` 2000 · `n_fft` 2048 | `HannPeriodic` |
| `src/eval/src/verifiers/estoi.rs:106-110,181-182` | `FrameFft<f64>`，256 / 512；帧数口径是 `0..len−N` 步长 hop（比 snip_edges 少最后一帧），调用方自己算 | `HannNumpyInner` |
| `src/eval/src/system.rs:47` | `StreamingStft::from_spec(&FRONTEND)` 或写明不跟 | `SqrtHannPeriodic` |
| `src/eval/src/analysis.rs:163` | 声谱图把 sqrt-Hann 在 f32 里平方当窗（交代 3 处之外多出的 1 处）。与 `HannPeriodic` 不逐位相同，要么加窗种类、要么接受分数变化 | 待定 |

清完前两处后删 R-FFT 白名单对应两条（规则会提醒）。
