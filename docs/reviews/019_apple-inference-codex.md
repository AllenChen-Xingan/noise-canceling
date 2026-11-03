# 评审 019 · 推理后端总方案与判据（codex · gpt-6-astra · read-only）

> 对象：`docs/research/019_apple-inference/decision.md` · `standard.md`，依据含 `docs/experiments/exp010-inference-bench/apple-rs/` 的 `src/{main,alloc,bnns,coreml,rt}.rs` 与 `tools/{power_parse.py,impact.sh,rt-matrix.sh}`。**最新一轮：第 3 轮（2026-11-02）· 总评「修改」· 待第 4 轮**。
> 累计处理：P1×10、P2×11 逐条回源码核实，属实 21、驳回 0、无法核实 0；引用抽查「不符」1 条（已改）。
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
