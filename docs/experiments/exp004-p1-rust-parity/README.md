# exp004 · P1 离线原型：Rust（`vdn`）对拍 Python · 每帧耗时 · 三件保命

> 意图：技术方案 `docs/plan/003_tech-design.md` 第四节 P1——Rust 命令行「WAV → GTCRN 逐帧流式降噪 + 三件保命 → WAV」，和 exp003 的 Python 结果逐值对拍。
> 2026-10-30 手下 p1。数字全部本机实测（Apple M5 · macOS 27.0.1 · rustc 1.99.1 · ort 2.0.0-rc.15 / ONNX Runtime 1.31 · Python onnxruntime 1.31.1）。经三个对抗视角自审后改过一版（见文末「自审改了什么」）。

## 一句话结论
- **对拍**：三件保命关掉时，**GTCRN 的 Rust 输出和 Python 最大差 4.5e-7**（12 段里最差那段；对拍 SNR ≥ 124 dB），SI-SDR 差 ≤ 2e-6 dB；**FastEnhancer-M 12 段全部逐样本相同**（差 0）
- **每帧耗时**（单线程，Rust 与 Python 交替跑 3 轮）：GTCRN p50 **0.70–0.71 ms**、max **0.89–0.93 ms**（16 ms 一帧，最坏占 6%）；FE-M p50 **0.67–0.68 ms**、max **0.78–0.93 ms**（10 ms 一帧，最坏占 9%）。Python 同口径 GTCRN p50 0.71–0.72 / max 1.02–1.04，FE-M p50 0.68–0.69 / max 0.78–1.49。**两边的时间几乎都花在 ONNX Runtime 里，Rust 与 Python 持平**
- **三件保命**：
  - 故障注入 4 类（NaN、后端报错、幅度爆掉、卡 40 ms）全部接住，输出没有 NaN；48 个接缝（切到原声、切回模型）都没有比两路信号本身更尖的跳变
  - 衰减下限把「停顿里最多压多少」从 −28.8 dB（GTCRN）/ −63.4 dB（FE-M）收到 −11.6 / −12.0 dB
  - **快开慢关的人声判据不合格**：只看能量，停顿里也判「有声」，94–96% 的帧是开着的。默认的 −6 dB 等于全程多混一份原声，SI-SDR 比只开下限再降约 1.1 dB。建议 P1 先关掉它（待拍板 #1）

## 怎么跑

```sh
cargo build --release
./target/release/vdn denoise in.wav out.wav --model-path <gtcrn_simple.onnx>   # GTCRN + 三件保命
./target/release/vdn denoise in.wav out.wav --model fastenhancer-m --floor-db -12
./target/release/vdn denoise in.wav out.wav --no-safety           # 输出 = 模型原样输出（对拍用）
./target/release/vdn bench --model gtcrn --input mix.wav          # 每帧耗时 + 每帧 Rust 侧分配次数
```

- 只收 16 kHz 单声道 WAV，输出 32-bit float
- 模型文件都在仓外，按顺序查找：`--model-path` → 环境变量 `VDN_GTCRN_ONNX` / `VDN_FASTENHANCER_ONNX` → `~/vdn-data/models/{gtcrn/gtcrn_simple.onnx, fastenhancer-dns-v1.0.0/fastenhancer_m.onnx}` → 当前目录下的 `resources/…`
- FE-M 已经在 `~/vdn-data`，不带参数就能跑。**GTCRN 目前只在主目录的 `resources/gtcrn/stream/onnx_models/`**，在 worktree 里要给 `--model-path`（待拍板 #3）
- 本机验收测试：设上两个模型的环境变量，再加 `VDN_REQUIRE_MODELS=1`，然后 `cargo test --workspace`。找不到模型就判失败，不会静默跳过。CI 上没有权重，模型测试打印一行后跳过

复现本实验：在项目根下设 `PY=<主目录>/_tmp/exp002-venv/bin/python`，再跑 `$PY docs/experiments/exp004-p1-rust-parity/exp004.py {src|py|rs|compare|bench}`。音频全写在主目录 `_tmp/20261030-exp004/`，不进仓。

## crate 划分（`src/<crate>/`，依赖单向 dsp ← engine ← models ← cli）

| crate | 管什么 | 依赖 | 为什么这样切 |
|---|---|---|---|
| `vdn-dsp`（`src/dsp`） | STFT/iSTFT、延迟线、快开慢关包络、窗 | realfft | 纯数学，不认识模型、不碰文件；以后声纹 fbank、音色链都从这里取积木 |
| `vdn-engine`（`src/engine`） | `trait Model`、`Engine<M>`（逐帧调度 + 三件保命）、故障注入、离线驱动 `offline::run` | vdn-dsp | 不链接 ONNX Runtime：保命逻辑用假模型就能测；离线和实时走同一个 `process`（`.42cog/cog.md`「eval 和实时用同一份处理代码」） |
| `vdn-models`（`src/models`） | GTCRN、FastEnhancer-M 适配器，`AnyModel` 运行期选择 | ort、vdn-engine、vdn-dsp | 重的 C++ 依赖只出现在这一个 crate；换后端（tract / CoreML）只动这里 |
| `vdn-cli`（`src/cli`，可执行 `vdn`） | `denoise`、`bench` | 以上 + hound、clap、serde_json、stats_alloc | 文件 I/O 和参数解析只在最外层 |

rustarch 初审（`p1-review-v1`）同意这个划分，并定了以下几条，都已照改：
- `eval` 现在不拆；**一出现指标或报告就拆出 `vdn-eval`，engine 永远不依赖 hound / serde**
- `Model: Send`
- 运行期切模型用手写的 `enum AnyModel`，不用 `Box<dyn>`
- `process` 里长度不符时，release 下送原声，不 panic

rustarch 的终版规约（`specs/rust/architecture.md`）出来后再对一遍。

**模型接口**（`src/engine/src/model.rs`）：
- 一律「一个 hop 进、一个 hop 出」的波形接口，声明 `hop()`、`latency_samples()`、`reset()`
- 频域模型（GTCRN）在适配器里自己做 STFT
- 出错返回不带 `String` 的 `ModelError`，出错路径也不分配
- 以后要加的东西都不改这个 trait（rustarch 初审），注释里已经写明：
  - 实时回调块长 ≠ hop 时，在外面加一层定长分块
  - 模型切换由外层持有两套模型、补齐延迟后交叉淡化
  - 声纹门控做成独立的处理级

## 对拍（`results/compare.json`）

素材和参考的来源：
- 素材：用 exp003 的 `make_clips.py` 原样重生成 12 段，只改了输出位置。重生成的 `clips.json` 与 exp003 提交的**逐字段一致**；每个 WAV 的 sha256 在 `results/src_manifest.json`
- Python 参考：直接 import exp003 `enhance.py` 的 `Gtcrn` + `stream()` 与 `FastEnhancer.run()`，一行没改

| 模型 | 最大绝对误差（12 段最差） | 对拍 SNR（最差 / 中位） | SI-SDR 差（最大） | 均值 SI-SDR Python / Rust |
|---|---|---|---|---|
| GTCRN | 4.5e-7 | 124.4 / 128.8 dB | 1.9e-6 dB | 3.678 / 3.678 dB |
| FastEnhancer-M | **0**（12/12 段逐样本相同） | 无穷大 | 0 | 4.902 / 4.902 dB |

- GTCRN 那 1e-7 级的差，推断来自 FFT：numpy 在 float64 里算，Rust 的 realfft 用 float32。窗函数照 numpy 的做法，先用 float64 算再转 float32。对拍按容差判，不按逐位相等判
- **对齐口径**：
  - Python `stream()` 开头补 n_fft 个零，所以第一次推理是全零帧
  - Rust 的输入缓冲本来就是零，要对齐就在开头多喂一个 hop（256 个样本）的零（`--lead-in 256`）
  - 不加 lead-in 时，第一帧少了一次全零推理，开头会有小差异，**没量**
  - FE-M 两边开头都不补零
- 延迟：GTCRN 512/256，输出晚 256 个样本（16 ms）；FE-M 晚 352 个样本（22 ms）。Rust 按声明的延迟平移后，和 Python 逐样本对齐（上表即证）

## 每帧耗时（`results/bench.json`）

- 输入：`resources/gtcrn/test_wavs/mix.wav`
- 每轮热身 200 帧后计 6000 帧；单线程（ORT intra / inter = 1、不自旋）
- **Rust 与 Python 同一模型挨着跑，交替 3 轮**；没控机器负载
- **口径不完全相同**：Rust 计的是适配器整次 `process`，GTCRN 这一项含 Rust 的 STFT/iSTFT；Python 只计 `sess.run`（同 exp001），GTCRN 不含 numpy 的 STFT。所以对 GTCRN，这个比法偏向 Python。FE-M 的 STFT 在图里，两边口径相同

| 模型 | 周期 | Rust p50 / p99.9 / max（ms，3 轮区间） | Python p50 / p99.9 / max（ms，3 轮区间） | 三件保命全开（整个 `Engine::process`）p50 / p99.9 / max | 每帧 Rust 侧分配 |
|---|---|---|---|---|---|
| GTCRN | 16 ms | 0.70–0.71 / 0.74–0.77 / 0.89–0.93 | 0.71–0.72 / 0.83–0.87 / 1.02–1.04 | 0.70–0.72 / 0.73–0.77 / 0.84–0.89 | **0 次** |
| FastEnhancer-M | 10 ms | 0.67–0.68 / 0.71–0.74 / 0.78–0.93 | 0.68–0.69 / 0.71–0.91 / 0.78–1.49 | 0.66–0.68 / 0.69–0.79 / 0.76–0.97 | 7 次 |

- 最坏一帧占周期：GTCRN 0.93 / 16 = 5.8%；FE-M 三件保命全开 0.97 / 10 = 9.7%。都远低于「最坏一帧 ≤ 周期一半」
- 三件保命本身的开销落在轮与轮之间的波动里，量不出来
- 比 exp001 慢（exp001 的 Python GTCRN p50 是 0.40 ms）。同一时段的 Python 也一样慢，推断是本机同时跑着别的会话；第一版自审前那次跑的尾部更差（max 1.3–1.45 ms），说明负载影响尾部。**绝对值不能当最坏情况用**，空载复测留给 P3
- 「每帧分配」用计数分配器（stats_alloc）量 Rust 侧：
  - GTCRN 有 4 个输入、4 个输出，落在 ort 的栈上小数组里，再加上预分配输出，**Rust 侧零分配**。`src/models/tests/no_alloc.rs` 用 assert_no_alloc 断言了这一点。反例自测：在断言里分配 64 字节，进程立刻中止
  - FE-M 有 7 个输入、7 个输出，超过 ort 的栈上容量（输入 6、输出 4，ort `src/util/stack.rs:13-15`），每帧固定分配 7 次
  - ORT C++ 内部的分配不走 Rust 分配器，两个模型都量不到

## 三件保命（`src/engine/src/engine.rs`）

| 件 | 做法 | 默认（P1 初值，待标定） |
|---|---|---|
| **出事送原声** | 触发条件：模型报错、输出含 NaN 或 Inf、输出幅度 > 4、超过截止时间。触发后这一帧输出**按模型延迟对齐**的原声。前三类说明状态可能坏了，还会复位模型，再多送 100 ms 原声。切到原声时坏输出不能拿来做交叉淡化，改用「接缝偏移衰减」：把上一个样本的差值在 5 ms 内衰减掉，波形保持连续。切回模型用 5 ms 线性交叉淡化 | 开；离线默认不设截止时间 |
| **衰减下限** | `输出 = (1 − m)·降噪 + m·原声`，停顿时 `m = 10^(floor_db/20)`。模型把噪音压到零时，背景也只降 `−floor_db` dB（同 libDF 的 atten_lim 做法） | −12 dB |
| **快开慢关** | 有声时，`m` 在约 2 ms 内升到 `10^(open_db/20)`；人声停了先保持 150 ms，再按 300 ms 时间常数回到下限。有声的判据只看能量：降噪输出这一帧比「底噪跟踪值」高 9 dB 就算有声。底噪跟踪值往下立刻跟，往上每秒最多涨 6 dB | open −6 dB、hangover 150 ms、release 300 ms |

### 实测效果（12 段，`results/compare.json.summary`）

| 模式 | GTCRN 均值 SI-SDR | FE-M 均值 SI-SDR | 「停顿」压低中位数（GTCRN / FE-M，9 段） | 压得最狠的一段 |
|---|---|---|---|---|
| 保命全关（= Python） | 3.68 dB | 4.90 dB | −4.8 / −6.4 dB | −28.8 / −63.4 dB（c10 钢琴） |
| 只开衰减下限 −12 dB | 3.37 | 4.75 | −3.5 / −5.2 | −11.6 / −12.0 |
| 三件全开（默认） | 2.64 | 3.59 | −2.3 / −3.5 | −7.1 / −11.2 |

注：「停顿」是按目标说话人的干净参考算的；很多段的停顿里有邻桌在说话，所以这一列不全是「压噪音」。

- **衰减下限按设计生效**：压得最狠的那段被收在 −12 dB，没有压成真空。代价是 SI-SDR 降 0.15 dB（FE-M）到 0.3 dB（GTCRN），因为混回去的原声本来就算误差
- **快开慢关几乎一直开着**：判为有声（含 hangover）的帧，GTCRN 96%、FE-M 95%；纯噪音的 c09 杯碟段上，GTCRN 是 99%、FE-M 88%。原因有两层：
  1. **bug，已修**：开头补的零、模型延迟里的零输出把底噪跟踪值钉在了 −120 dB，之后要十几秒才涨回来。自审抓到后改成「低于 −70 dBFS 的帧不参与跟踪」，并加了回归测试。修复前后的变化：有声帧比例 99.4% → 96.0%（GTCRN）、96.5% → 94.6%（FE-M）；三件全开时的 SI-SDR 2.63 → 2.64、3.58 → 3.59，几乎没变
  2. **主因，判据本身的问题**：底噪跟踪是「逐帧最小值，慢慢上涨」，会贴着最安静的那几帧走。模型输出里的残噪逐帧起伏 10–25 dB（杯碟瞬态、邻桌人声），所以多数停顿帧都比最小值高出 9 dB 门限，再加 150 ms 保持，就一直开着。拿 c09 GTCRN 复算：停顿帧比底噪跟踪值高出的量，中位数 14 dB；停顿帧的能量中位数 −32 dB，说话帧 −26 dB，两者大面积重叠，**只看能量分不开**。复算脚本没进仓，是照 Rust 逻辑写的 Python 模拟，跑在 `rs/raw` 输出上
  - 结论：要让快开慢关真正起作用，得换真正的人声检测（WebRTC VAD、Silero 一类），P2 再加声纹判断「是不是你」（003 §2.3）。现在默认的 −6 dB 约等于「全程多混一份原声」，比只开下限的 SI-SDR 再低约 1.1 dB
- 吃字率（ASR）没测。「快开慢关有没有护住词尾」要用 exp003 的 Qwen3-ASR 打分或人耳听来回答，本批没做

### 故障注入

设置：c05、c06、c10 三段，在第 100 / 200 / 300 / 400 帧依次注入 NaN、后端报错、幅度爆掉、卡 40 ms；截止时间设为半个周期。

- 4 类全部计数正确：每类 1 次，复位 3 次（卡顿不复位）；输出全部有限
- 送原声的帧数：GTCRN 25 帧（3 × (故障帧 + 复位后 7 帧) + 卡顿 1 帧）、FE-M 34 帧（3 × 11 + 1）
- **接缝没有咔声**：每次故障有两个接缝，一个是切到原声（故障那一帧开头），一个是切回模型（保持结束后交叉淡化的开头）。在每个接缝前后各 32 个样本内，量输出二阶差分的最大绝对值，和同一窗口里「无故障版」与「原声」两者中较大的那个比。2 个模型 × 3 段 × 4 类 × 2 个接缝 = 48 个点，比值最大 1.002（`compare.json.summary.<模型>.seam_ratio_max`）。也就是说，接缝处没有比两路信号本身更尖的跳变。这是波形上的判据，没做人耳听
- 代价：4 次故障让整段 SI-SDR 变化 −0.4 ~ +0.5 dB，原因是复位后模型要重新收敛。c05 上两个模型都略升，因为这一段模型输出本身比原声差

## 实时安全
- **`process` 路径不分配、不加锁、不做 I/O**：
  - `vdn-dsp` 与 `vdn-engine` 有 assert_no_alloc 测试，后者用假模型，故障路径也在内
  - GTCRN 真模型路径也有同样的测试，只覆盖 Rust 侧
  - FE-M 每帧 7 次 Rust 分配来自 ort 的栈上容量，已登记
  - 这些不分配断言**只在 debug 下跑**：assert_no_alloc 的 `AllocDisabler` 只在 `debug_assertions` 下存在。`cargo test --release` 会跳过它们（dsp、models 的 `no_alloc.rs` 整个文件，engine `safety.rs` 里那一条），其余行为测试照跑
- 选型依据 rusteco 的 `answers-p1.md`：
  - assert_no_alloc 常开在测试里
  - rtsan-standalone 在 libc 层拦截，连 ORT 的 C++ 分配也能抓，留作以后 CI 的一个可选任务，本批没接
- **flush-to-zero 没开**：`no_denormals` 是 `unsafe`，而且会让数值和 Python 对不上。P1 先不开，P3 在实时线程入口加
- **截止时间**：
  - 检查用 `Instant::now()`，macOS 上走 mach_absolute_time，不分配，可以在实时线程里用
  - 离线下的「超时」是事后判定：模型已经算完，只是不用它的输出
  - **真实时下没法打断一次推理**，P3/P4 要靠双进程 + 环形缓冲兜底（003 §1）
- 模型出错时，GTCRN 适配器照样合成一帧（频谱清零），保证重叠相加缓冲不错位（自审 P2）

## 依赖许可（与 GPL-3.0 的兼容性）

许可证是用 `cargo metadata` 手查的，本机没装 cargo-deny。rusteco 的 `answers-p1.md` §5 有 deny.toml 白名单模板，以 rustarch 终版为准，下一批接进 CI。

| 依赖 | 许可 | 用在 |
|---|---|---|
| ort 2.0.0-rc.15 / ort-sys | MIT OR Apache-2.0 | 推理 |
| ONNX Runtime 1.31（ort 构建时下载的 pyke 预编译库，静态链接） | MIT（微软） | 推理 |
| realfft（MIT）、rustfft、clap、serde_json、num-complex 等 | MIT，或 MIT OR Apache-2.0 | |
| hound | Apache-2.0 | WAV 读写 |
| stats_alloc | MIT | bench 分配计数 |
| assert_no_alloc（只在测试里） | BSD-1-Clause | 不分配断言 |
| 构建期（ort-sys 下载 ORT 时用）：openssl、ureq、webpki-root-certs 等 | Apache-2.0 / MIT / ISC / CDLA-Permissive-2.0 | 只在构建时用，不进成品 |

- 全部是宽松许可，与 GPL-3.0 兼容，没有 GPL-2.0-only
- **模型权重另算**：GTCRN 仓库是 MIT；FastEnhancer 代码是 MIT，但 release 权重的许可待核（003 §5），发布前必须定
- `download-binaries` 在构建时联网下载 ORT，复现与 CI 要钉版本与哈希（rustarch 初审），本批没钉

## 已知局限
1. **快开慢关的人声判据不合格**（见上），参数也没用吃字率或人耳标定
2. **ORT 内部的分配没量**：C++ 侧的分配要 rtsan 才抓得到；FE-M 在 Rust 侧每帧 7 次分配也没消除
3. **离线超时 ≠ 实时超时**（见上）
4. **没开 flush-to-zero**
5. ort 构建时联网下载 ORT 预编译库，**没钉哈希**；本机没装 cargo-deny，许可证是手查的
6. 只支持 16 kHz 单声道，不重采样；`offline::run` 一次把整段读进内存
7. **计时**：只有一个输入、3 轮，没控机器负载，尾部数字偏乐观或偏悲观都有可能；GTCRN 的两边口径不同（见上）
8. 「停顿压低」按目标说话人的停顿算，停顿里含邻桌人声，不等于压噪
9. 不加 lead-in 时 GTCRN 开头的差异没量
10. **盲听没做**：003 第四节说 P1 要由人用 exp003 素材再听一版「Rust 版不比 Python 版差」。保命关闭时两边逐值相同，再听一遍没有意义；值得听的是「保命开 vs 关」（待拍板 #2）
11. 接缝无咔声只有波形判据，没有人耳确认
12. 两个以后可做的优化（rusteco `answers-p1.md` 更新版），本批都没做：
    - 缓存现在每帧拷回输入；可以改成两套张量 + 两个 RunOptions 奇偶帧轮换，省掉这次拷贝
    - FastEnhancer 有 16 kHz 的 spec2spec 发布物（`fastenhancer_m.spec.onnx`），可以和 GTCRN 共用 Rust 的 STFT，也可能绕开每帧 7 次分配

## CI 建议（`.github/workflows/ci.yml` 不在 p1 的写入范围，交给大总管）
- 三条 cargo 命令加 `--locked`
- 加 `Swatinem/rust-cache`：ort-sys 每次冷编、联网下载 ORT，又慢又容易因为网络挂掉
- 现在 CI 上的模型测试是「跳过」不是「通过」：绿灯不代表测过模型。本机验收要用 `VDN_REQUIRE_MODELS=1`

## 待人拍板
1. **快开慢关 P1 的默认值**（现在 open −6 dB）
   - 选项 A：保持 −6 dB。最保守，宁可漏噪音也不吃字；代价是 SI-SDR 再降约 1.1 dB，说话时和大部分停顿里，背景约是原声的一半
   - 选项 B：open 设成和下限一样的 −12 dB。等于 P1 关掉快开慢关，等 P2 换上真正的人声检测和声纹再开
   - 选项 C：折中，−9 dB
   - **建议 B**：判据现在在停顿里也开着，A 的代价换不来「护住词尾」，只是多放进噪音
   - **默认 A**，因为交代原文要求做快开慢关
2. **要不要出一版「保命开 / 关」盲听页**：用 exp003 的页面框架，12 段 × {保命关、只开下限、全开}。建议做，放下一批；默认不做
3. **GTCRN 权重要不要复制一份到 `~/vdn-data/models/gtcrn/`**：和 FE-M 放一起，`vdn` 不带参数就能找到，worktree 里也能跑。建议复制（535 KB，MIT，仓里留哈希）；默认不复制，用 `--model-path`

## 自审改了什么
三个对抗视角并行审：代码正确性、实验结论怀疑派、架构与 CI。

**代码**：
- 底噪被钉在 −120 dB（P0，见上）
- GTCRN 出错时重叠相加缓冲会错位一个 hop
- FE-M 和 GTCRN 的输入输出没按名字校验配对
- hangover 用浮点计数，整除时会多开一帧 → 改成整数样本计数
- 保命全关时改成直接拷贝模型输出，保证逐位相同
- `session` 模块改成私有
- 模型测试加 `VDN_REQUIRE_MODELS`
- 删掉没用的 dev 依赖

**结论**：
- 改正「快开慢关一直开」的原因
- 「无爆音」改成两个接缝分别量、二阶差分对比
- 计时改成交替 3 轮，口径差异写明
- 删掉「Rust 省下 numpy 那一点」的说法：这次的比法根本量不到 numpy 那部分
- 「SNR 320 dB」改成「逐样本相同」：原数字是代码里防除零的下限钳出来的
- 三件保命的开销如实写成「量不出」
- 补了局限 7–11

## 文件
| 文件 | 是什么 |
|---|---|
| `exp004.py` | 五步脚本（src、py、rs、compare、bench），只 import exp003 的脚本 |
| `results/src_manifest.json` | 重生成素材的 sha256，以及「clips.json 与 exp003 一致」 |
| `results/compare.json` | 每段 × 每模型 × 4 种模式：对拍误差、SI-SDR、停顿压低、保命计数、故障接缝 |
| `results/bench.json` | Rust 与 Python 交替 3 轮的每帧耗时、每帧 Rust 分配次数 |
