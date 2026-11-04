# 现状页 · fecore（每帧只算一次频谱，#94）

- 分支 `feat/frontend-once` · 大总管 `noise-canceling-a2` · 批次 20261104-b13
- 管的实体：频谱前端 · 降噪模型适配（GTCRN / FE-M）· 场景特征 · 修音频谱级

## 进度（当场记）

- 2026-11-04 报到；读完交代与 common
- 2026-11-04 盘点完（读源码，未构建），见下表
- 2026-11-04 大总管与 fespk 先后告知：认人不接共享频谱 → 借谱接口不提供「输出谱」，只服务场景特征与修音
- 2026-11-04 写设计稿 `docs/plan/013_frontend-once.md`，发大总管待点头
- 2026-11-04 大总管点头；修音频域级槽不做（`SpectralStage` 留在 polish）；dsp → types 依赖边核对允许（规约 §2.2 图里本来就有，check-deps 只查禁用清单）
- 2026-11-04 写完代码与测试（**未编译、未跑**，等名额）：
  - 正本 `src/types/src/frontend.rs`（`FRONTEND`）；`SpectrumTap` 改为 `spectrum_spec()` + `input_power()`
  - `vdn-dsp`：`StreamingStft::from_spec / feed / forward_count`、`power_into`、`window::for_spec`
  - GTCRN 实现 `SpectrumTap`；场景加 `push_hop_with_power`，自算入口改用 `StreamingStft`；离线例子改引用正本
  - 测试：`scene/tests/{golden,frontend}.rs` · `models/tests/{golden,tap}.rs` · `polish/tests/borrowed_spectrum.rs` · dsp / scene / models 的 no_alloc 各加一条
- 2026-11-04 大总管授权改 `specs/rust/architecture.md:225` 的借谱接口说明 → 已改为 `input_power`；全仓旧接口名 0 处
- 2026-11-04 名额 ①：基准已生成（实测）。场景 `45107641c255052a`，24 次判断；GTCRN `72e947a3fb17c048`，48000 点，权重 `780e68e301ab1513`；平台 aarch64-macos。原样 `features.rs` 的 sha256 与 `git show 990e7843:` 一致。名额已归还，② 已申请
- 2026-11-04 名额 ②：五个 crate（types · dsp · scene · polish · models，带真模型）`cargo test` 全绿，0 失败（实测）。GTCRN 对拍、场景对拍都不是跳过，而是真比了，逐位相同；借谱 vs 自算 35 维最大差 0（阈值 1e-5），24 次判断全同。修了一处：`src/dsp/Cargo.toml` 把 `vdn-types` 从 dev 依赖挪进普通依赖。名额已归还，③ 已申请
- 基准怎么生成：草稿目录里的 `bless.sh` 把 `990e7843`（本分支改动前）原样导出，只放两份对拍测试，在那里生成基准再拷回 `tests/fixtures/`。所以基准不受本分支任何改动影响

## 盘点：每处 FFT / STFT（读源码，未实测）

| 位置 | 参数 | 窗 | 缩放 | 吃什么 | 备注 |
|---|---|---|---|---|---|
| `src/dsp/src/stft.rs:28` `StreamingStft` | 由调用方给 | 由调用方给 | 分析不归一；合成 ×1/n 再乘窗 | — | 唯一的流式 STFT 实现 |
| `src/models/src/gtcrn.rs:19-21,49` | 512 / 256 / 257 点，本地常量 | `sqrt_hann_periodic(512)` | 同上 | 带噪输入；`infer` 后频谱被**改写成模型输出谱** | 不实现 `SpectrumTap` |
| `src/models/src/fastenhancer.rs:18` | `N_FFT=512` 只用来算延迟；hop 160 | 图内 | 图内 | 波形进波形出 | 拿不到谱 |
| `src/scene/src/features.rs:43-44,134-159,209-218` | 512，hop 由构造给（只许 256），本地常量 | `sqrt_hann_periodic(512)` | 不归一；功率 = `re²+im²+1e-12` | 带噪输入 | 另有一次**逆** FFT 求自相关（功率谱 → 自相关），这次省不掉 |
| `src/polish/src/spectral.rs` | `new(n_fft, hop, sr)` 由调用方给 | — | — | 现成的 dry / wet 复数谱 | **库里不做 FFT**；只有离线例子做 |
| `src/polish/examples/polish_offline.rs:17-18,98-99` | 512 / 256，本地常量 | `sqrt_hann_periodic` | 同 StreamingStft | dry 波形、wet 波形各做一次 | wet 是「降噪后波形再分析」，不等于模型输出谱 |
| `src/scene/examples/scene_offline.rs:16` | HOP=256 本地常量 | — | — | — | 只用 hop |
| `src/speaker/src/fbank.rs:73,121` | 帧长 400 → 512，hop 160 | hamming / povey | Kaldi 口径 | 降噪后 | 归 fespk；口径与上面不同，本线不碰 |

**场景特征与 GTCRN 口径核对（读源码）**：窗同为 `sqrt_hann_periodic(512)`；输入缓冲同样全零起步、每次左移一个 hop 再填尾；加窗同为 `x * w`；正向 FFT 同为 `realfft` 512 点、不归一。结论：同一段输入下，场景自算的复数谱与 GTCRN 分析出的复数谱**逐位应相同**（推断，待测试证实）。差别只有一处：GTCRN 推理后把频谱原地改写成输出谱，借谱必须在推理**之前**取。

## 待人拍板

（暂无）

## 转交

（暂无）
