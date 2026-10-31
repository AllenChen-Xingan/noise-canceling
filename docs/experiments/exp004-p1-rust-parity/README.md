# exp004 · P1 离线原型：Rust（`vdn`）对拍 Python · 每帧耗时 · 三件保命

> 意图：技术方案 `docs/plan/003_tech-design.md` 第四节 P1——Rust 命令行「WAV → GTCRN 逐帧流式降噪 + 三件保命 → WAV」，和 exp003 的 Python 结果逐值对拍。
> 2026-10-30 手下 p1。数字全部本机实测（Apple M5 · macOS 27.0.1 · rustc 1.99.1 · ort 2.0.0-rc.15 / ONNX Runtime 1.31 · Python onnxruntime 1.31.1）。

## 一句话结论
- **对拍**：三件保命关掉时，**GTCRN 的 Rust 输出和 Python 最大差 4.5e-7**（12 段里最差那段，对拍 SNR ≥ 124 dB），SI-SDR 差 ≤ 2e-6 dB；**FastEnhancer-M 逐样本完全相同**（差 0）
- **每帧耗时**（单线程，口径同 exp001）：GTCRN p50 / p99.9 / max = **0.71 / 1.20 / 1.32 ms**（16 ms 一帧，最坏占 8%）；FE-M **0.69 / 1.02 / 1.20 ms**（10 ms 一帧，最坏占 12%）。同一时段的 Python：GTCRN 0.73 / 1.24 / 1.45，FE-M 0.69 / 1.21 / 1.30。**两边耗时几乎都在 ONNX Runtime 里，Rust 省下的只是外面 numpy 那一点**。三件保命全开只多约 0.002 ms（p50）
- **三件保命**：故障注入 4 类（NaN · 后端报错 · 幅度爆掉 · 卡 40 ms）全部接住，输出无 NaN，故障点附近的最大相邻样本差 ≤ 原声本身；衰减下限把「停顿里最多压多少」从 −28.8 dB（GTCRN）/ −63.4 dB（FE-M）收到 −11.6 / −12.0 dB；**快开慢关在这批素材上几乎一直开着**（96–99% 的帧判为有声，原因见下），默认的 −6 dB 让 SI-SDR 降约 1 dB——默认值待人定

## 怎么跑

```sh
cargo build --release
./target/release/vdn denoise in.wav out.wav                       # 默认 GTCRN + 三件保命
./target/release/vdn denoise in.wav out.wav --model fastenhancer-m --floor-db -12
./target/release/vdn denoise in.wav out.wav --no-safety           # 输出 = 模型原样输出（对拍用）
./target/release/vdn bench --model gtcrn --input mix.wav          # 每帧耗时 + 每帧 Rust 侧分配次数
```

只收 16 kHz 单声道 WAV，输出 32-bit float。模型文件在仓外：`--model-path`，或环境变量 `VDN_GTCRN_ONNX` / `VDN_FASTENHANCER_ONNX`，或 `~/vdn-data/models/{gtcrn/gtcrn_simple.onnx, fastenhancer-dns-v1.0.0/fastenhancer_m.onnx}`，或当前目录下的 `resources/…`。**GTCRN 目前只在主目录 `resources/gtcrn/stream/onnx_models/`**，在 worktree 里跑要给 `--model-path`（待拍板 #3）。

复现本实验（项目根下，`PY=<主目录>/_tmp/exp002-venv/bin/python`）：`$PY docs/experiments/exp004-p1-rust-parity/exp004.py {src|py|rs|compare|bench}`。音频全写主目录 `_tmp/20261030-exp004/`，不进仓。

## crate 划分（`src/<crate>/`，依赖单向 dsp ← engine ← models ← cli）

| crate | 管什么 | 依赖 | 为什么这样切 |
|---|---|---|---|
| `vdn-dsp`（`src/dsp`） | STFT/iSTFT、延迟线、快开慢关包络、窗 | realfft | 纯数学，不认识模型、不碰文件；以后声纹 fbank、音色链都从这里取积木 |
| `vdn-engine`（`src/engine`） | `trait Model`、`Engine<M>`（逐帧调度 + 三件保命）、故障注入、离线驱动 `offline::run` | vdn-dsp | 不链接 ONNX Runtime：保命逻辑用假模型就能测；离线和实时走同一个 `process`（`.42cog/cog.md`「eval 和实时用同一份处理代码」） |
| `vdn-models`（`src/models`） | GTCRN、FastEnhancer-M 适配器，`AnyModel` 运行期选择 | ort、vdn-engine、vdn-dsp | 重的 C++ 依赖只出现在这一个 crate；换后端（tract / CoreML）只动这里 |
| `vdn-cli`（`src/cli`，可执行 `vdn`） | `denoise`、`bench` | 以上 + hound、clap、serde_json、stats_alloc | 文件 I/O 和参数解析只在最外层 |

rustarch 初审（`p1-review-v1`）同意这个划分，并定了：`eval` 现在不拆，**一出现指标 / 报告就拆 `vdn-eval`，engine 永不依赖 hound / serde**；`Model: Send`；运行期切模型用手写 `enum AnyModel`（不用 `Box<dyn>`）；`process` 里长度不符 release 下送原声不 panic。均已照改。rustarch 终版规约（`specs/rust/architecture.md`）出来后再对一遍。

**模型接口**（`src/engine/src/model.rs`）：一律「一个 hop 进、一个 hop 出」的波形接口，声明 `hop()`、`latency_samples()`、`reset()`；频域模型（GTCRN）在适配器里自己做 STFT。出错返回不带 `String` 的 `ModelError`，出错路径也不分配。

## 对拍（`results/compare.json`）

素材：用 exp003 的 `make_clips.py` 原样重生成 12 段（只改输出位置），重生成的 `clips.json` 与 exp003 提交的**逐字段一致**；每个 WAV 的 sha256 在 `results/src_manifest.json`。Python 参考 = 直接 import exp003 `enhance.py` 的 `Gtcrn` + `stream()` 与 `FastEnhancer.run()`，不改一行。

| 模型 | 最大绝对误差（12 段最差） | 对拍 SNR（最差 / 中位） | SI-SDR 差（最大） | 均值 SI-SDR Python / Rust |
|---|---|---|---|---|
| GTCRN | 4.5e-7 | 124.4 / 128.8 dB | 1.9e-6 dB | 3.678 / 3.678 dB |
| FastEnhancer-M | **0** | 319.9 / 325.8 dB（浮点打印极限，实际逐样本相等） | 0 | 4.902 / 4.902 dB |

- GTCRN 的 1e-7 级差来自 STFT：numpy 用 float64 做 FFT，Rust 用 realfft 的 float32；窗函数按 numpy 的做法先 float64 算再转 float32，所以窗是逐值相同的
- **对齐口径**：Python `stream()` 开头补 n_fft 个零，第一次推理是全零帧；Rust 输入缓冲本来就是零，要对齐就开头多喂一个 hop（256）的零（`--lead-in 256`）。不加 lead-in 时第一帧少一次全零推理，GTCRN 缓存起点略不同，开头约几十毫秒有小差异（没量，推断）。FE-M 两边都不补
- 延迟：GTCRN 512/256 → 输出晚 256 样本（16 ms）；FE-M 晚 352 样本（22 ms）。Rust 按声明的延迟平移后与 Python 逐样本对齐（上表即证）

## 每帧耗时（`results/bench.json`）

输入 `resources/gtcrn/test_wavs/mix.wav`，热身 200 帧后计 6000 帧，单线程（ORT intra / inter = 1、不自旋）。Rust 计的是模型适配器整次 `process`（GTCRN 含 Rust 的 STFT/iSTFT；FE-M 的 STFT 在图里）；Python 计 `sess.run`（GTCRN 不含 numpy 的 STFT，同 exp001）。

| 模型 | 周期 | Rust p50 / p99 / p99.9 / max（ms） | Python p50 / p99 / p99.9 / max（ms） | Rust 最坏占周期 | 每帧 Rust 侧分配 |
|---|---|---|---|---|---|
| GTCRN | 16 ms | 0.71 / 0.92 / 1.20 / 1.32 | 0.73 / 1.02 / 1.24 / 1.45 | 8.3% | **0 次** |
| FastEnhancer-M | 10 ms | 0.69 / 0.79 / 1.02 / 1.20 | 0.69 / 1.01 / 1.21 / 1.30 | 12.0% | 7 次 |
| 三件保命全开（GTCRN / FE-M 整个 `Engine::process`） | | 0.71 / 1.08 / 1.24 / 1.31 · 0.69 / 1.03 / 1.26 / 1.39 | | | |

- 本次 GTCRN 比 exp001 慢（exp001 Python p50 0.40 ms、max 0.57 ms）：**同一时段 Python 也一样慢**，判断是本机同时跑着别的会话（推断，没控负载）。Rust 与 Python 只按同时段比
- 「每帧分配」用计数分配器（stats_alloc）量 Rust 侧：GTCRN 有 4 个输入、4 个输出，落在 ort 的栈上小数组里，加上预分配输出，**Rust 侧零分配**——`src/models/tests/no_alloc.rs` 用 assert_no_alloc 断言了这一点（反例自测：在断言里分配 64 字节，进程立刻中止）；FE-M 有 7 个输入、7 个输出，超过 ort 的栈上容量（输入 6、输出 4，ort `src/util/stack.rs:13-15`），每帧固定分配 7 次。ORT C++ 内部的分配不走 Rust 分配器，两种都量不到

## 三件保命（`src/engine/src/engine.rs`）

| 件 | 做法 | 默认（P1 初值，待标定） |
|---|---|---|
| **出事送原声** | 模型报错 / 输出含 NaN·Inf / 幅度 > 4 / 超过截止时间 → 这一帧输出**按模型延迟对齐**的原声。前三类（状态可能坏了）还会复位模型，再多送 100 ms 原声。切到原声时坏输出不能用来交叉淡化，改用「接缝偏移衰减」（上一个样本的差值在 5 ms 内衰减掉，波形连续）；切回模型用 5 ms 线性交叉淡化 | 开；截止时间离线默认不设 |
| **衰减下限** | `输出 = (1 − m)·降噪 + m·原声`，停顿时 `m = 10^(floor_db/20)`：模型把噪音压到零时，背景也只降 `−floor_db` dB（同 libDF 的 atten_lim 做法） | −12 dB |
| **快开慢关** | 有声时 `m` 在约 2 ms 内升到 `10^(open_db/20)`；人声停了先保持 150 ms，再以 300 ms 时间常数回到下限。有声的判据只看能量：降噪输出这一帧比「底噪跟踪值」高 9 dB | open −6 dB · hangover 150 ms · release 300 ms |

### 实测效果（12 段，`results/compare.json.summary`）

| 模式 | GTCRN 均值 SI-SDR | FE-M 均值 SI-SDR | 停顿压低中位数（GTCRN / FE-M，9 段） | 停顿压得最狠的一段 |
|---|---|---|---|---|
| 保命全关（= Python） | 3.68 dB | 4.90 dB | −4.8 / −6.4 dB | −28.8 / −63.4 dB（c10 钢琴） |
| 只留衰减下限 −12 dB | 3.37 | 4.75 | −3.5 / −5.2 | −11.6 / −12.0 |
| 三件全开（默认） | 2.63 | 3.58 | −2.3 / −3.5 | −6.6 / −11.0 |

- **衰减下限按设计生效**：最狠那段被收在 −12 dB，没有压成真空；代价是 SI-SDR 降 0.2–0.3 dB（混回的原声本来就算误差）
- **快开慢关几乎一直开着**：判为有声（含 hangover）的帧，GTCRN 99%、FE-M 97%。素材是邻桌说话、赌场这类：目标停顿时别人在说话，模型把别人的声音也留下了，能量判据当然判「有声」。连纯噪音段（c09 杯碟、c10 钢琴），词与词之间的短停顿里模型也留了不少声音（GTCRN 停顿帧「降噪后 / 原声」能量比中位数 −9.5 dB），再加 150 ms 保持，就很难收回去。**这是 P1 不认人的必然结果**：分清「你」和「别人」是 P2 声纹的事（003 §2.3：放开只看能量，收要声纹确认）。现在默认 −6 dB 的效果约等于「全程混 50% 原声」，SI-SDR 比只留下限再降约 1 dB
- 吃字率（ASR）没测：要回答「快开慢关有没有护住词尾」，得跑 exp003 的 Qwen3-ASR 打分或人耳听，本批没做（待拍板 #1）

### 故障注入（c05 · c06 · c10，第 100 / 200 / 300 / 400 帧依次注入 NaN · 报错 · 幅度爆掉 · 卡 40 ms，截止时间 = 半个周期）

- 4 类全部计数正确（每类 1 次、复位 3 次——卡顿不复位），输出全部有限
- 送原声的帧：GTCRN 25 帧（4 × 1 帧 + 3 × 复位后 7 帧）、FE-M 34 帧
- **无爆音**：故障点前 1 帧到后 3 帧内的最大相邻样本差，24 个点全部 ≤ 原声在同一窗口里的最大相邻样本差（`compare.json.summary.<模型>.fault.<段>.local_step`）
- 代价：4 次故障让整段 SI-SDR 变化 −0.4 ~ +0.5 dB（复位后模型重新收敛；FE-M c05 反而略升，因为那段模型本身比原声差）

## 实时安全
- `process` 路径不分配、不加锁、不做 I/O：`vdn-dsp` 与 `vdn-engine`（假模型，含故障路径）有 assert_no_alloc 测试；GTCRN 真模型路径也有（Rust 侧）；FE-M 每帧 7 次 Rust 分配是 ort 的栈上容量问题，已登记
- 选型（rusteco `answers-p1.md`）：assert_no_alloc 常开在测试；rtsan-standalone（libc 层，连 ORT 的 C++ 分配也能抓）留作以后 CI 的一个可选 job，本批没接
- **flush-to-zero 没开**：`no_denormals` 是 `unsafe`，而且会让数值和 Python 不同；P1 先不开，P3 在实时线程入口加
- 截止时间检查用 `Instant::now()`（macOS 走 mach_absolute_time，可在实时线程用）。离线下「超时」是事后判定：模型已经算完，只是不用它的输出；**真实时下没法打断一次推理**，P3/P4 要靠双进程 + 环形缓冲兜底（003 §1）

## 依赖许可（与 GPL-3.0 兼容性）

手查 `cargo metadata`（本机没装 cargo-deny；rusteco `answers-p1.md` §5 有 deny.toml 白名单模板，以 rustarch 终版为准，下一批接进 CI）：

| 依赖 | 许可 | 用在 |
|---|---|---|
| ort 2.0.0-rc.15 / ort-sys | MIT OR Apache-2.0 | 推理 |
| ONNX Runtime 1.31（ort 构建时下载的 pyke 预编译库，静态链接） | MIT（微软） | 推理 |
| realfft · rustfft · clap · serde_json · num-complex 等 | MIT OR Apache-2.0 | |
| hound | Apache-2.0 | WAV 读写 |
| stats_alloc | MIT | bench 分配计数 |
| assert_no_alloc（只在测试） | BSD-2-Clause | 不分配断言 |
| 构建期（ort-sys 下载用）：openssl · ureq · webpki-root-certs 等 | Apache-2.0 / MIT / ISC / CDLA-Permissive-2.0 | 只在构建时下载 ORT，不进成品 |

全部是宽松许可，与 GPL-3.0 兼容；没有 GPL-2.0-only。**模型权重另算**：GTCRN 仓库 MIT；FastEnhancer 代码 MIT，release 权重的许可待核（003 §5，发布前必须定）。`download-binaries` 构建时联网下载 ORT，复现与 CI 要钉版本与哈希（rustarch 初审），本批没钉。

## 已知局限
1. **快开慢关的判据只看能量**，有别人说话时一直开着；参数没用吃字率或人耳标定（待拍板 #1）
2. **ORT 内部分配没量**：C++ 侧分配要 rtsan 才抓得到；FE-M Rust 侧每帧 7 次分配未消除
3. **离线超时 ≠ 实时超时**：离线是事后判定，见上
4. **没开 flush-to-zero**
5. ort 构建时联网下载 ORT 预编译库，**没钉哈希**；本机没装 cargo-deny，许可证是手查的
6. 只支持 16 kHz 单声道，不重采样；`offline::run` 一次把整段读进内存
7. 计时时本机有别的会话在跑，绝对值偏高；Rust 与 Python 只按同时段比
8. 盲听没做：003 第四节说 P1 由人用 exp003 素材再听一版「Rust 版不比 Python 版差」。保命关闭时两边逐值相同，**再听一遍没有意义**；值得听的是「保命开 vs 关」（待拍板 #2）

## 待人拍板
1. **快开慢关的默认值**（现在 open −6 dB）。选项：A 保持 −6 dB（最保守，宁可漏噪音也不吃字；代价 SI-SDR −1 dB、说话时背景大约是原声的一半）· B open = 下限 −12 dB（P1 实际上关掉快开慢关，等 P2 声纹来判断「是不是你」再开）· C 折中 −9 dB。**建议 B**：P1 的判据分不清你和邻桌，A 的代价全花在放进别人的声音上了。**默认 A**（交代原文要求快开慢关）
2. **要不要出一版「保命开 / 关」盲听页**（exp003 页面框架，12 段 × {保命关, 只留下限, 全开}）。建议做，下一批；默认不做
3. **GTCRN 权重要不要复制一份到 `~/vdn-data/models/gtcrn/`**（和 FE-M 放一起，`vdn` 默认就能找到，worktree 里也能跑）。建议复制（535 KB，MIT，仓里留哈希）；默认不复制，用 `--model-path`

## 文件
| 文件 | 是什么 |
|---|---|
| `exp004.py` | 五步脚本（src · py · rs · compare · bench），只 import exp003 的脚本 |
| `results/src_manifest.json` | 重生成素材的 sha256，以及「clips.json 与 exp003 一致」 |
| `results/compare.json` | 每段 × 每模型 × 4 种模式：对拍误差、SI-SDR、停顿压低、保命计数、故障点局部跳变 |
| `results/bench.json` | Rust 与 Python 每帧耗时、每帧 Rust 分配次数 |
