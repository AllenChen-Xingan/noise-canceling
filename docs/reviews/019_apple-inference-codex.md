# 评审 019 · 推理后端总方案与判据（codex · gpt-6-astra · read-only）

> 对象：`docs/research/019_apple-inference/decision.md` · `standard.md`，依据含 `docs/experiments/exp010-inference-bench/apple-rs/` 的 `src/{main,alloc,bnns,coreml,rt}.rs` 与 `tools/{power_parse.py,impact.sh,rt-matrix.sh}`。**最新一轮：第 6 轮（2026-11-02）· 总评「修改」· 待第 7 轮**。
> 累计处理：P1×13、P2×22 逐条回源码核实，属实 35、驳回 0、无法核实 0；引用抽查「不符」2 条（已改）。
> 送 codex 之前已做同模型 3 视角只读自审一轮（数字核对 · 结论强度 · 测量口径），其发现已先改完；那一轮的 P2 留在 `decision.md` §7。

## 第 1 轮（2026-11-02）· 总评「修改」

原文：`_tmp/codex-019/decision-codex-1.md`（不入库）。类型 research，effort medium，耗时 77 秒（`result: P0×0 P1×5 P2×2 P3×0`）。日志头：`model: gpt-6-astra` · `sandbox: read-only` · `reasoning effort: medium`。行号指评审时的版本。
本轮 P0/P1：5 条 · 停止线：未满足 · 留作待办的 P2：无（本轮两条 P2 都已改）

- **P1｜standard.md:121 · impact.sh:35 影响测试可能把降噪退出后的负载算进「开」组**（第 1 轮 #1）
  - 核实：属实。上一版把降噪封顶 1500 帧（24 s），而负载 8000 遍在 4 线程下可能更久；负载跑完后没检查降噪是否还活着。
  - 处理：降噪改为跑到被 kill（`--frames 100000`），孤儿风险改由 bench 自己检测父进程（`main.rs`：每 64 帧查 `getppid() == 1` 就退出）；负载跑完后 `kill -0` 检查降噪仍在，不在就记 `covered:false`，该对作废并在汇总里列出。
- **P1｜standard.md:55 · main.rs:303–322,372 BE-DEADLINE 漏掉唤醒迟到**（第 1 轮 #2）
  - 核实：属实。旧版只数推理调用耗时 > 3 ms，睡过头的帧不会被数到。
  - 处理：bench 按节奏跑时每帧记「做完的时刻 − 该到的时刻」，新字段 `late_frames_over_3ms` / `late_max_ms`；`standard.md` 的 `BE-DEADLINE` 改用它判，并注明 19:00 前的 build 只有调用耗时、不能判这一条；`decision.md` 挡线表同步改为「未测」。
- **P1｜decision.md:15 · power_parse.py:41,35 v2 没做到「只平均计时窗内样本」**（第 1 轮 #3）
  - 核实：属实。窗口两头各**放宽** 1 s，会混入预热和事后对拍；缺功率项记成 0。
  - 处理：改为向内收（样本时间戳是秒级、标在样本结束处，只收 `u − 1 ≥ lo` 且 `u ≤ hi`）；按样本时长加权；缺项的样本整段作废；窗内样本合计不足 80% 墙钟的段判无效。旧数据（没有计时窗字段）重算结果与原表一致。
- **P1｜standard.md:76 · main.rs:339–370 cpu_over_wall 分子分母不在同一窗口**（第 1 轮 #4）
  - 核实：属实。上一版墙钟在循环末停，CPU 时间和分配计数却在对拍之后才读；分配按三位小数打印可能把稀少分配舍成 0。
  - 处理：循环一结束立刻读 CPU 时间、Rust 与全进程分配计数；新增整数字段 `rust_allocs_total` / `process_allocs_total`，判 `BE-RT-ALLOC` 用整数；`standard.md` 注明 19:00 前的 build 的 `cpu_over_wall` 含对拍、略偏高。
- **P1｜decision.md:47 DFN3「不进实时链」的性能论证不成立**（第 1 轮 #5）
  - 核实：属实。D75 后 CPU% 不挡；0.61 ms 是 Python 调 ORT、没有这个模型的 Rust 差值校准；GTCRN 的 5–7 倍不能直接套到别的模型上。
  - 处理：改为「实时链暂缓（时间盒）」，写明要淘汰得补 Rust 实时测量（前提是先流式重导出，约 1 天）；离线盲听照做。
- **P2｜standard.md:113 选赢家的挡线漏掉 BE-COLD、BE-OS 和 BE-POWER「不高于 ORT」**（第 1 轮 #6）
  - 核实：属实；rt-matrix 也没有 ORT，重跑仍判不了「不高于 ORT」。
  - 处理：§6.2 挡线集合改为「总表里所有挡上用户电脑 / 挡合并 / 挡默认后端的」并逐个列出；`rt-matrix.sh` 加同场 `ort rt` 设置（第 10 个）；`decision.md` 挡线表补 `BE-COLD`（未测）、`BE-OS`（过）、「不高于 ORT」（未判）。
- **P2｜decision.md:82 把忙等提速直接归因为降频，证据不足**（第 1 轮 #7）
  - 核实：属实。忙等同时改变核迁移、缓存热度与睡眠唤醒；单次、噪声内的功率差也不足以证明「提速要花电」。
  - 处理：两处都标【假设】，写明要补核驻留与频率观测、交替重复；结论改为「这一批证据推翻不了 D74」。

### 引用抽查

- codex 列的「不符」：`standard.md:70` 说 `open_ms` 含编译，而 `main.rs:273` 明确减去了 `compile_ms` → 已改：`open_ms` 不含编译，`BE-COLD` 分「首次启动」「已有缓存」两个数。
- codex 列的「核实属实」：4 条（ORT 会话设置、3 ms 预算含唤醒抖动、fbank 25 / 10 ms、BNNS 缓存换指针）。
- 无法核实：`decision.md` 引的 0.65 / 0.66 ms（结果 summary 不在许可清单）、D 编号原文（决策文件不在许可清单）→ 第 2 轮把 `apple/results/*/summary.md` 和 `docs/plan/001_decisions-20261029-b1.md` 加进依据。

### 是否再审

有 P1 → 改完开第 2 轮，提示词附本记录。

## 第 2 轮（2026-11-02）· 总评「修改」

原文：`_tmp/codex-019/decision-codex-2.md`（不入库）。effort medium，耗时 102 秒（`result: P0×0 P1×4 P2×4 P3×0`）。日志头：`model: gpt-6-astra` · `sandbox: read-only`。本轮依据加了两份结果 summary 与决策原文摘录（`_tmp/codex-019/decisions-excerpt.md`，摘自主检出 `docs/plan/001_decisions-20261029-b1.md`）。
第 1 轮的输出文件开头补了一行 Markdown 标题再作 `--previous`：原文以「P1」开头，`file` 把它认成 PBM 位图，脚本当非纯文本拒发；原文另存 `decision-codex-1.orig.md`，内容未改。
上一轮处理的验证：#1、#3 codex 认为修得不完整（见本轮 #2、#1），其余未再提。
本轮 P0/P1：4 条 · 停止线：未满足

- **P1｜power_parse.py:43 窗口筛选仍不保守**（第 2 轮 #1）
  - 核实：属实。时间戳是秒级向下取整，真实结束 ∈ [u, u+1)；上一版没用样本时长 e，也没给结束端留 1 s。
  - 处理：只收「最早开始 u − e ≥ lo 且最晚结束 u + 1 ≤ hi」的样本。
- **P1｜impact.sh:36「全程覆盖」仍未证实**（第 2 轮 #2）
  - 核实：属实。`sleep 4` 后只查存活，模型可能还在加载或预热。
  - 处理：bench 加 `--ready-file`，进入计时窗（加载、预热都过）时才写就绪文件；impact 等文件出现才起负载（最多 30 s，超时停下），结束时仍查存活。已用 30 帧小跑验证文件会写出。
- **P1｜decision.md:55 用未校准的 Python 数字淘汰 MLX，违反 D68**（第 2 轮 #3）
  - 核实：属实。
  - 处理：MLX 实时推理改为「暂缓（时间盒）」，依据改为执行机制（每帧建图 + `eval` 分配、加锁、等后台线程，读源码）；删掉「换 Rust 补不回 15 倍」；§9 那两行改为「只说明 Python 调 MLX 在这台机上是这个数」。
- **P1｜standard.md:144 Metal 淘汰依据不成立**（第 2 轮 #4）
  - 核实：属实。0.1 ms 是异机数据，又和连续跑的 0.088 ms 比；`notes/mlx.md` 里还有融合核约 0.0225 ms / 步的反例。
  - 处理：Metal 手写核改为「时间盒暂缓、无收益未验证」，写明要在本机测逐帧融合路径；`decision.md` 同步，并把「Core ML 下神经引擎 / GPU 没用上」（实测）和「Metal 没测」分开写。
- **P2｜standard.md:30 1% / 2% 影响挡线标成人已定，来源不符**（第 2 轮 #5）
  - 核实：属实。决策摘录写的是「AI 理解，待人确认」，人原话只定了维度。
  - 处理：`standard.md` 总表、§6.3 与 `decision.md` 页首改为「维度已定 · D75；两个数沿用 D69 两档【AI 理解，待人确认】」。
- **P2｜decision.md:9 因果修正没同步全篇**（第 2 轮 #6）
  - 核实：属实。
  - 处理：`decision.md` 风险 1、`standard.md` §6、`apple/results/20261102-183322-rt/summary.md` 第 2、3 条都改为【假设】；「提速要花电」也改为假设；D74 的「停在能效核约 1.1 GHz」注明是另一条线的实测。
- **P2｜standard.md:61 整数判零没写进标准**（第 2 轮 #7）
  - 核实：属实。
  - 处理：`BE-RT-ALLOC` 改为用 `process_allocs_total == 0` 判，且必须是开了 `--count-malloc` 的那次；null 一律不算过。
- **P2｜main.rs:382 新字段让默认输出变成非法 JSON**（第 2 轮 #8）
  - 核实：属实。不按节奏时 `late_max` 是 NaN，直接打出 `NaN`。
  - 处理：加 `jnum()`，非有限值一律打 `null`；`parity_max_abs`、`parity_snr_db`、`parity_min_frame_snr_db` 也走它；不按节奏时 `late_frames_over_3ms` 为 null。两种模式各跑一次，`json.loads` 都能解析。

另：本轮之间大总管更正决策编号，「最低系统提到 macOS 15」由 D77 改为 **D78**（D77 是另一条「NoNoise-Mac 进评测」），两份文件已全部改号。

### 引用抽查

- codex 列的「不符」：无。
- codex 列的「核实属实」：6 条（ORT 会话设置、fbank、3 ms 余量、BNNS 换指针、手写 rt 的 0.65 / 0.66 ms、open_ms 已减去 compile_ms）。
- 无法核实：D74 原文（在另一位大总管的分支上，本次没有给）。

### 是否再审

有 P1 → 改完开第 3 轮。

## 第 3 轮（2026-11-02）· 总评「修改」

原文：`_tmp/codex-019/decision-codex-3.md`（不入库）。effort medium（`result: P0×0 P1×1 P2×5 P3×0`）。日志头 `model: gpt-6-astra` · `sandbox: read-only`。依据里的决策摘录重摘过，含 D74 原文与 D78 改号。
上一轮处理的验证：codex 核实属实 power_parse 的保守窗口（:45）、impact 的就绪文件与存活检查（:39–43）、非有限值打 null（main.rs:421–425）。
本轮 P0/P1：1 条 · 停止线：未满足

- **P1｜main.rs:291 BE-MEM 混进基准自己的输出留存**（第 3 轮 #1）
  - 核实：属实。实测：全量对拍时「预热后 − 加载前」= +21.5 MB，`--parity-frames 10` 时 +9.4 MB；差的 12 MB 就是留存输出缓冲。先分配再逐页写一遍也没能让它进基线。
  - 处理：缓冲改为在取基线前分配并逐页写非零值（保留）；`standard.md` 规定量 `BE-MEM` 必须单独跑一次、加 `--parity-frames 10`，并写明原因和这两个实测数。
- **P2｜decision.md:40 工作组「多耗电」仍写成确定代价**（第 3 轮 #2）
  - 处理：改为「那一次高 25–40 mW，单次观察、与噪声同量级，要等交替重复测」。
- **P2｜b2 summary.md:52 已撤回的 DFN3 淘汰论证仍在被引用的依据里**（第 3 轮 #3）
  - 处理：原句划掉，注明「推论已撤回」及原因，指向 decision 的「实时链暂缓」。
- **P2｜standard.md:55 截止挡线用的是过时的延迟账**（第 3 轮 #4）
  - 核实：属实。D74 原文：缓冲余量 lead 192 样本 = 4 ms。
  - 处理：`BE-DEADLINE` 写明产品余量是 4 ms，3 ms 是更严的技术线、不是产品截止时间；`decision.md` §6 同步，并注明「超 3 ms」与「超 4 ms（真会送原声）」那批没分开统计。
- **P2｜standard.md:70 BE-COLD 要的字段没实现**（第 3 轮 #5）
  - 处理：bench 加 `warmup_ms`（冷第 0 帧 + 全部预热）与 `first_warm_frame_ms`（预热后第 1 帧）；`standard.md` 写明用哪几个字段相加判首次启动与已有缓存两种情形。实测一次：BNNS 展开版 `compile_ms` 202 · `open_ms` 0.05 · `warmup_ms` 32 · 冷首帧 0.20 · 热首帧 0.16 ms。
- **P2｜standard.md:120 BE-IMPACT 有互不等价的几种判法**（第 3 轮 #6）
  - 处理：§6.3 改成唯一口径（= `impact.sh` v2：性能核个数个线程、8000 遍、15 对、成对比值中位数、噪声底 = 「关」的相对标准差），旧的两种口径明写作废。

### 引用抽查

- 「不符」：无。「核实属实」：7 条。
- 无法核实：rustbe 那边「每帧 0 次分配」的整数总数（正本在 rustbe worktree）→ `decision.md` 挡线表改为「转记，待对 rustbe 正本核整数」。

### 是否再审

有 P1 → 改完开第 4 轮。

## 第 4 轮（2026-11-02）· 总评「修改」

原文：`_tmp/codex-019/decision-codex-4.md`（不入库）。effort medium（`result: P0×0 P1×1 P2×5 P3×0`）。日志头 `model: gpt-6-astra` · `sandbox: read-only`。
上一轮处理的验证：codex 核实属实 7 条（内存基线前移与 `--parity-frames 10` 规定、工作组功耗改为单次观察、DFN3 推论已撤回、`warmup_ms` / `first_warm_frame_ms` 已输出、impact 成对比值与噪声底、power_parse 保守窗口、BNNS 换指针）。
本轮 P0/P1：1 条 · 停止线：未满足

- **P1｜decision.md:67 零分配被直接推成「可在推理线程直接调」**（第 4 轮 #1）
  - 核实：属实。`standard.md` §4 要分配、RTSan、跨进程三项都过才算；后两项没测。
  - 处理：改为「候选接法，未确认」，写明只有分配一项有数，零分配不能证明不加锁、不阻塞。
- **P2｜decision.md:27 整数判零没落到通过状态**（第 4 轮 #2）
  - 核实：属实。我读了 rustbe 正本 `results/20261102-blas1/malloc.jsonl`：native 是 `process_allocs_per_frame` 0.000、2000 帧、三位小数，只能说明总数 ≤ 1。
  - 处理：先改为「转记 0，待核整数」；随后 rustbe 指出整数字段早已加上，`results/20261102-verify1/cases.jsonl` 的 `bench_short_paced_rt`（rt、按节奏、300 帧）里 native `process_allocs_total` = 0、ORT 300 帧共 467 100 次。挡线表改为「过（整数 0，样本偏短）」，引 verify1、不引 blas1；第 2 节、§4 同步；副本 `_tmp/codex-019/rustbe-verify1-cases.jsonl` 供第 5 轮核对。顺带从同一份文件读到 native 常驻内存预热后只涨 1.09 MB，记进方案（单次）。
- **P2｜decision.md:100 · standard.md:52 延迟账仍写 3 ms 是当前余量**（第 4 轮 #3）
  - 处理：两处都改为「009 当时是 3.00 ms，D74 已改为 4 ms，以 4 ms 为准；3 ms 只作更严的技术线」。
- **P2｜standard.md:70「已有编译缓存」不能统一扣掉 compile_ms**（第 4 轮 #4）
  - 核实：属实。`bnns.rs` 每次打开都调 `BNNSGraphCompileFromFile`，没有读已编译图的分支。
  - 处理：BNNS 一律按 `compile_ms + open_ms + warmup_ms` 判；只有实测确实跳过的工作才能扣。
- **P2｜impact.sh:65 不足 15 个有效配对也能判通过**（第 4 轮 #5）
  - 处理：有效配对 < 15 时判「无效，补测」，不出过不过。
- **P2｜standard.md:176 Python 相对比较被提前放行**（第 4 轮 #6）
  - 处理：ORT 那一行改为「未获准」：判法要 p50 和 p99.9 两项都达标，p99.9 的差还没量。

### 引用抽查

- 「不符」：无。「核实属实」：7 条。
- 无法核实：rustbe 正本（上一轮）→ 本轮在 `_tmp/codex-019/rustbe-malloc.jsonl` 放了副本供第 5 轮核对。

### 是否再审

有 P1 → 改完开第 5 轮。

## 第 5 轮（2026-11-02）· 总评「修改」

原文：`_tmp/codex-019/decision-codex-5.md`（不入库）。effort medium（`result: P0×0 P1×1 P2×3 P3×0`）。日志头 `model: gpt-6-astra` · `sandbox: read-only`。依据加了 rustbe 的 `verify1/cases.jsonl` 与 `blas1/malloc.jsonl` 副本。
上一轮处理的验证：codex 核实属实 7 条（verify1 里 native 整数 0、撤回「零分配即可直接调」、BNNS 打开即编译已计入、impact 有效配对不足不判过、Python 比较已改「未获准」、延迟账 4 ms、power_parse 保守窗口）。
本轮 P0/P1：1 条 · 停止线：未满足

- **P1｜power_parse.py:69 缺计时窗的结果仍进耗电排名**（第 5 轮 #1）
  - 核实：属实。无窗口时退回整段平均（含加载、预热），却按计时帧数折算；rustbe 的 bench 输出确实没有窗口字段。
  - 处理：没有 `win_start_unix` 或 `wall_s` 的行标 `ranked: false`，只展示、不进 `summary`，另列 `unranked_no_window`；`rt-matrix.sh` 遇到缺窗口字段的输出会在日志里警告。rustbe 的 bench 要加这三个字段，记为转交（rustbe 会话已结束，交大总管）。
- **P2｜decision.md:40 BNNS 仍写「0 分配」「唯一能用」**（第 5 轮 #2）
  - 处理：改为「显示 0.000 / 帧，`BE-RT-ALLOC` 待整数验证」，「唯一能用」改为「候选」；`standard.md` §7.1 同步。
- **P2｜decision.md:91「本次所有基准都不含 STFT」不属实**（第 5 轮 #3）
  - 处理：限定为只对 GTCRN 成立；FE-M 现在是 wav2wav 端到端口径，换 spec2spec 后要按频谱口径重测。
- **P2｜standard.md:77 要求每个 CPU 数标「性能核」，与配置、证据冲突**（第 5 轮 #4）
  - 处理：默认测法改为 D74 的 `--qos rt`；记录实际策略；没观测核驻留的一律标「核类型未知」。

本轮之间另做（大总管定 / 新规则）：
- 手写 + Accelerate：cblas / vDSP 路线写成「已不采用」（每帧 11 次分配，出处 `cblas_sgemm → APL_sgemm_QR → posix_memalign`），BNNS 路线待证实（大总管定）。
- `BE-RT-ALLOC` 改引 rustbe `results/20261102-alloc6k/malloc.jsonl`：native 连续 6000 帧 `process_allocs_total` = 0（ORT 9 342 000）。
- 字段交叉校验（CLAUDE.md 第五节第 5 条）：新增 `apple-rs/tests/fields.rs`，以 rustbe 的 `../bench/src/main.rs` 为唯一真相源比对字段，缺 / 多（未登记）即失败；用真实文件跑通过，往副本里塞假字段会失败。对齐时发现同名不同义：apple-rs 原来的 `late_*` 实为 rustbe 的 `release_*`，已改为 rustbe 的名字与含义，并补 `open_to_warm_ms`。
- 加测试（CLAUDE.md 第五节第 6 条）：`main.rs` 单元测试 4 条（JSON 非有限值、分位数口径、GTCRN 布局往返、形状一致）；`tests/test_power_parse.py` 4 条（计时窗向内收与基线相减、缺项作废、覆盖不足作废、无窗口不排名）。都已跑过、通过。

### 引用抽查

- 「不符」：无。「核实属实」：7 条。
- 无法核实：`MLComputePlan` 全落 CPU 的原始 `plan.jsonl` 不在许可清单 → 第 6 轮加进依据。

### 是否再审

有 P1 → 改完开第 6 轮。

## 第 6 轮（2026-11-02）· 总评「修改」

原文：`_tmp/codex-019/decision-codex-6.md`（不入库）。effort medium（`result: P0×0 P1×1 P2×3 P3×0`）。日志头 `model: gpt-6-astra` · `sandbox: read-only`。依据加了 `plan.jsonl`、rustbe `alloc6k` 副本、`tests/fields.rs`、`tests/test_power_parse.py`。
上一轮处理的验证：codex 核实属实 6 条（alloc6k 整数 0、手写 rt 的 0.65 / 0.66 ms、BNNS 每次打开都编译、`qos=rt` 走 time-constraint 且无核驻留观测、无窗口不进 summary、fbank 25 / 10 ms）。
本轮 P0/P1：1 条 · 停止线：未满足

- **P1｜power_parse.py:88 噪声底漏算 GPU、神经引擎**（第 6 轮 #1）
  - 核实：属实。排序指标是三项之和，噪声底只取 CPU。
  - 处理：噪声底改为各轮 CPU + GPU + 神经引擎空闲总功率的极差，输出写明口径；新增反例测试（两轮 CPU 相同、GPU 差 40 mW → 噪声底必须是 40），通过。
- **P2｜b2 summary.md:29,32,46 BNNS「0 分配」没同步**（第 6 轮 #2）
  - 处理：三处都改为「显示 0.000（三位小数，整数待验证）」。
- **P2｜standard.md:51 计时口径仍统一写「不含 STFT」**（第 6 轮 #3）
  - 处理：§3 按模型分开：GTCRN 推理口径（不含 STFT）· FE-M 现在端到端口径（含 STFT）· FE-M 将来 spec2spec 频谱口径（重测，不混比）。
- **P2｜fields.rs:77 读不到正本时直接通过**（第 6 轮 #4）
  - 处理：读不到就失败并说明原因；用不存在的路径跑一次，确认测试变红。

### 引用抽查

- 「不符」：`decision.md:53`「所有算子都落在 CPU」——`plan.jsonl` 实为 cpu 322、none 1785 → 已改为「被指派了设备的算子全部在 CPU；另有 1785 个没有设备归属（`deviceUsage` 返回空，多为常量类节点，采集代码 `apple/bench_coreml.swift:112–113`）」，decision / standard / b2 summary 三处同步。
- 「核实属实」：6 条。
- 无法核实：`none` 的含义（采集脚本不在许可清单）→ 第 7 轮加 `apple/bench_coreml.swift`。

本轮之间另做（大总管）：`decision.md` 第 2 节加「手写版耗电暂不进排名，待 bench 补计时窗字段（已转交）」。

### 是否再审

有 P1 → 改完开第 7 轮。
