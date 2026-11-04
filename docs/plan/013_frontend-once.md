# 013 · 每帧只算一次频谱：降噪 · 场景 · 修音共用（#94，D71）

> 状态：大总管 2026-11-04 点头（修音槽不做，见第 4 节）。依据：读源码盘点（`state/handoff/fecore.md`「盘点」），未构建、未实测。
> 范围：场景特征与修音借降噪模型的谱。认人不接（大总管 2026-11-04 定，fespk 估最多省约 5%）。

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
- 比对：场景 `const _: () = assert!(FRONTEND.hop == 256)`（权重按它训练）；GTCRN 已有测试把 `mix` 形状与频点数比对，改为与 `FRONTEND.bins()` 比

## 3. 借谱接口（正本归 fecore）

```rust
pub trait SpectrumTap: Model {
    fn spectrum_spec(&self) -> StftSpec;             // 谱的口径，调用方构造期比对
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
- **修音频域级槽：不做**。原因：把修音接到模型里属于接线，不在本大事（大总管 2026-11-04 定，`SpectralStage` 留在 polish）；槽与 engine 接线留给 backlog 那件事

## 5. 场景特征

- `push_hop(x)`：自算（兜底，FE-M 用），内部改用 `StreamingStft::from_spec`，删掉自建的窗、缓冲、FFT 计划
- 新增 `push_hop_with_power(x, power)`：吃现成功率谱；`x` 仍要（削波是时域算的）
- 两个入口只差「功率从哪来」：各一行填 `self.pow`，之后同一段 `finish_frame` 算 35 维要用的帧量
- 借谱时也把 `x` 推进自算缓冲（`StreamingStft::feed`），换到 FE-M 时第一帧不吃旧数据
- 只差一处：`x` 不足一个 hop 时，旧代码只移 `len` 个样本，`StreamingStft` 移一整个 hop 并补零。实时路径每次都是整 hop，不受影响；测试只喂整 hop
- 逆 FFT（功率谱 → 自相关求谐波度）省不掉，留着
- `SceneAnalyzer` 同样加 `push_hop_with_power`

## 6. 拿不到谱的模型（FE-M）

不实现 `SpectrumTap`；场景走 `push_hop` 自算一次；修音只跑时域部分（现状不变）。

## 7. 处理图描述

不加新节点种类。场景特征不在音频路径上，本来就不进图。修音频域级已有种类 `POLISH_SPECTRAL`（挂点 Spectral）。以后模型里装修音槽时，要与 `ProcessingGraph::spectral` 比对——那是接线那件事（backlog）的活。本线不改图。

## 8. 测试

1. **GTCRN 对拍**：固定 2 s 输入，输出 f32 逐位的哈希写进 `src/models/tests/fixtures/`。顺序：先在**没改过**的 `gtcrn.rs` 上生成哈希（名额放开后），再改 GTCRN。没模型文件时同 `smoke.rs` 跳过（设 `VDN_REQUIRE_MODELS` 不许跳）
2. 场景：借谱 vs 自算，35 维最大绝对差 ≤ 1e-5（预期 0）；场景判断逐帧相同。谱源用 `StreamingStft` + `power_into`（与 GTCRN 同一段代码），CI 上能跑；另有一条对「改前的场景实现」的对拍：旧 `push_hop` 的计算原样留在测试里当参照（只在测试里，不进库）
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
| GTCRN 改前改后输出（`models/tests/golden.rs`，3 s，48000 点） | 逐位相同 | 逐位 | 是 |
| 场景改前改后判断（`scene/tests/golden.rs`，12 s，24 次判断） | 逐位相同 | 逐位 | 是 |
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
