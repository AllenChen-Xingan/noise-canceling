# 013 · 每帧只算一次频谱：降噪 · 场景 · 修音共用（#94，D71）

> 状态：草案，待大总管点头。依据：读源码盘点（`state/handoff/fecore.md`「盘点」），未构建、未实测。
> 范围：场景特征与修音借降噪模型的谱。认人不接（大总管 2026-11-04 定，fespk 估最多省约 5%）。

## 1. 现在的毛病

- 512 / 256 / 257 这组参数手抄在 4 处：GTCRN、场景特征、两个离线例子
- 场景特征自己又建一套 STFT（窗、缓冲、FFT 计划），和 GTCRN 的分析逐步相同（读源码核对，见现状页）
- 借谱接口 `SpectrumTap::last_magnitude` 没人实现，也没人调用；它给的是幅度，场景要的是功率（幅度再平方不逐位相同）

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

- 换掉 `last_magnitude`（无人用）。给功率不给幅度：场景只用功率，逐位可复现
- 不给输出谱（认人不接）。不给复数谱：`vdn-types` 零依赖，没有复数类型

## 4. 频谱前端：一个小结构，GTCRN 用它

`vdn_dsp::SpectrumFrontend` = `StreamingStft` + 带噪谱副本 `dry` + 功率 `power` + 帧计数：

- `analyze(hop)`：分析一次，复制 `dry`、算 `power`，返回可改写的谱（模型把输出谱写进去）
- `dry()` · `power()` · `synthesize(out)` · `forward_count()`（测试用：正向 FFT 次数）
- 为什么要复制 `dry`：GTCRN 推理后把谱原地改成输出谱，修音和场景都要推理前那份

GTCRN 持有它，实现 `SpectrumTap`；另加一个可选的频域级槽 `set_spectral(Box<dyn SpectralStage>)`，在「输出谱 → iSTFT」之间调用，`dry` 直接借前端那份。

⚠️ 这要把 `SpectralStage` trait 从 `vdn-polish` 挪到 `vdn-dsp`（`vdn-models` 看得见；polish 原样再导出）。graph 线 2026-11-02 定「trait 留在 polish」，理由是 types 零依赖没有复数类型——放 dsp 不碰这个理由。**建议，待你定**。不挪的话，修音只能在测试里手动接，GTCRN 不带槽。

## 5. 场景特征

- `push_hop(x)`：自算（兜底，FE-M 用），内部改用 `StreamingStft::from_spec`，删掉自建的窗、缓冲、FFT 计划
- 新增 `push_hop_with_power(x, power)`：吃现成功率谱；`x` 仍要（削波是时域算的）
- 两个入口只差「功率从哪来」：各一行填 `self.pow`，之后同一段 `finish_frame` 算 35 维要用的帧量
- 借谱时也把 `x` 推进自算缓冲（只移位、不做 FFT，`StreamingStft::feed`），换到 FE-M 时第一帧不吃旧数据
- 逆 FFT（功率谱 → 自相关求谐波度）省不掉，留着
- `SceneAnalyzer` 同样加 `push_hop_with_power`

## 6. 拿不到谱的模型（FE-M）

不实现 `SpectrumTap`；场景走 `push_hop` 自算一次；修音只跑时域部分（现状不变）。

## 7. 处理图描述

不加新节点种类。场景特征不在音频路径上，本来就不进图。修音频域级已有种类 `POLISH_SPECTRAL`（挂点 Spectral）。GTCRN 装槽时与 `ProcessingGraph::spectral` 比对——留给 engine 接线那件事（backlog），本线在 `set_spectral` 的文档里写明。

## 8. 测试

1. **GTCRN 对拍**：固定 2 s 输入，输出 f32 逐位的哈希写进 `src/models/tests/fixtures/`。顺序：先在**没改过**的 `gtcrn.rs` 上生成哈希（名额放开后），再改 GTCRN。没模型文件时同 `smoke.rs` 跳过（设 `VDN_REQUIRE_MODELS` 不许跳）
2. 场景：借谱 vs 自算，35 维最大绝对差 ≤ 1e-5（预期 0）；场景判断逐帧相同。用 `SpectrumFrontend` 当谱源，CI 上能跑
3. 计数：前端 + 场景借谱 + 修音槽，每帧 `forward_count` 增 1、场景自算次数 0
4. 修音：槽里跑出的谱 == 手动用同一对 dry / wet 调 `PolishChain` 的结果（逐位）
5. 0 分配：`assert_no_alloc` 包住前端、借谱入口、槽

注意：离线例子 `polish_offline` 的 wet 是「降噪后波形再分析」，和模型输出谱本来就不同，所以不拿它对拍；它只改为引用正本，输出不变。

## 9. 复杂度账（D71 第 2 条）

- 加：`StftSpec`（约 30 行）· `SpectrumFrontend`（约 80 行）· 场景借谱入口（约 15 行）· dsp → types 一条依赖边
- 减：场景自建 STFT（约 25 行）· 4 处本地常量 · 没人用的 `last_magnitude`
- 净增约 100 行（估计）、1 个新结构。每帧省一次 512 点正向 FFT（耗时待测）
