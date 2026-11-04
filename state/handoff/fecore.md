# 现状页 · fecore（每帧只算一次频谱，#94）· 已收尾

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
- 2026-11-04 名额 ③：`xtask ci` 全绿，fmt · clippy · test · tidy · check-deps · cargo-deny 6 项都过（实测）。中途 clippy 红一类：测试里 `chunks_exact` 要换成 `as_chunks`，3 处，一次修好。另加了计时例子 `src/models/examples/frontend_timing.rs`，models 因此加 dev 依赖 vdn-scene。名额已归还，④ 已申请
- 2026-11-04 名额 ④：每帧耗时实测，结果在 013 第 10 节。场景段中位 2.42 → 2.00 µs；每帧合计中位 394.1 → 394.1 µs，差别在噪声内
- 2026-11-04 多视角自审：P2×2 已修（借谱入口加口径比对，编译期断言补上窗和采样率），P3 部分已修；范围外的转交见下。记录在 `docs/reviews/013_frontend-once-selfreview.md`
- 2026-11-04 人改定 D82（所有频谱计算放同一处代码，各自切法）：本线范围不变；013 第 2、4 节补了说明，第 11 节写了「第二段」接口
- 2026-11-04 codex 3 轮，第 2、3 轮连续无 P0/P1，**已达停止线**。第 1 轮 P1：短输入的移位语义，已修，加了不整齐长度的对拍。P2 共 7 条全修，没有留 issue。记录在 `docs/reviews/013_frontend-once-codex.md`
- 2026-11-04 大总管要求补引擎最终输出的逐位对拍 → 已加（48 kHz `Pipeline`，144000 点）。四份基准都在 `990e7843` 上重生成，改后逐位相同；五个 crate 的测试和 `xtask ci` 全绿
- 基准怎么生成：草稿目录里的 `bless.sh` 把 `990e7843`（本分支改动前）原样导出，只放对拍测试（场景 `golden.rs` + `common/mod.rs`、模型 `golden.rs`），在那里生成基准再拷回 `tests/fixtures/`。所以基准不受本分支任何改动影响

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

## 收尾清单

**交付物**：
- 设计与实测：`docs/plan/013_frontend-once.md`（第 10 节是对拍和耗时表，第 11 节是第二段接口）
- 评审：`docs/reviews/013_frontend-once-selfreview.md` · `docs/reviews/013_frontend-once-codex.md`
- 计时工具：`src/models/examples/frontend_timing.rs`

**改了哪些文件**（相对 `990e7843`，36 个）：
- 正本：`src/types/src/{frontend.rs,lib.rs,model.rs,frame.rs}`
- dsp：`src/dsp/{Cargo.toml,src/lib.rs,src/stft.rs,src/window.rs,tests/no_alloc.rs}`
- 模型：`src/models/{Cargo.toml,src/gtcrn.rs,examples/frontend_timing.rs,tests/golden.rs,tests/tap.rs,tests/no_alloc.rs,tests/fixtures/*}`
- 场景：`src/scene/{src/features.rs,src/lib.rs,examples/scene_offline.rs,tests/*}`
- 修音：`src/polish/{src/lib.rs,src/spectral.rs,examples/polish_offline.rs,tests/borrowed_spectrum.rs}`
- 其他：`specs/rust/architecture.md:225`（已授权）、`Cargo.lock`（新依赖边带出）、本页、上面三份文档

## 待人拍板

（暂无）

## 转交

给大总管：
1. 索引 `docs/reviews/README.md` 加一行 013 frontend-once（该文件不在我可写范围）。
2. 第二段（认人 fbank、评测器 FFT、离线例子收进同一模块）：接口与验收见 013 第 11 节。要点：`StftSpec` 加 `win_len`，`Window` 加枚举值，加每帧预处理钩子，实现对 f32 / f64 都通用，每处做逐位对拍。
3. `src/eval/src/system.rs` 的参考降噪器手写 512 / 256：要么改引用 `FRONTEND`，要么写明不跟（归第二段或 eval 线）。
4. `src/models/Cargo.toml` 的 dev 依赖 `vdn-scene` 用的是 path 写法：根 `Cargo.toml` 的 `[workspace.dependencies]` 没有 vdn-scene，改它不在我可写范围。
5. 自审视角顺带提出的范围外老问题（不是本线引入的），建议记进待办：
   - P1：`src/README.md` 还是模板。
   - P1：模块说明里内部编号多、没有白话。
   - P1：A / B / C 档没有定义。
   - P2：「档」一词有三个意思；L0–L3 没有逐级说明；verifier 代号没有中文；`types/src/lib.rs` 表头写「三块」、表里四行，帧长 10 / 16 ms 不一致。
6. 基准生成脚本在我的草稿目录（不入库）。第二段如要重生成，做法照抄：`git archive <改前提交>` 导出到临时目录，只放对拍测试，用单独的 target 目录，`VDN_BLESS=1` 跑完把 `tests/fixtures/*` 拷回。
