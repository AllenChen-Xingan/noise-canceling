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
3. 计数：一个 `StreamingStft` 当模型前端 + 场景借谱 + 测试里手动接修音，每帧正向 FFT 总数恰好 1（前端 `forward_count` 增 1、场景自算 0）；有模型文件时对真 GTCRN 再测一次
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
- 全仓 grep：除 `vdn-dsp` 外没有 `plan_fft_forward` / `RealFftPlanner`。可以写成一条 tidy 规则，把「只在一处」变成机器检查。

**估计改动量**：types 约 +30 行，dsp 约 +100 行；认人 fbank 约 −30 行，评测器 3 处约 −40 行；每处配一条逐位对拍测试。
