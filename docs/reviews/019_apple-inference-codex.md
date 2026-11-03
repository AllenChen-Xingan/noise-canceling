# 评审 019 · 推理后端总方案与判据（codex · gpt-6-astra · read-only）

> 对象：`docs/research/019_apple-inference/decision.md` · `standard.md`，依据含 `docs/experiments/exp010-inference-bench/apple-rs/` 的 `src/{main,alloc,bnns,coreml,rt}.rs` 与 `tools/{power_parse.py,impact.sh,rt-matrix.sh}`。**最新一轮：第 1 轮（2026-11-02）· 总评「修改」· 待第 2 轮**。
> 累计处理：P1×5、P2×2 逐条回源码核实，属实 7、驳回 0、无法核实 0；引用抽查「不符」1 条（已改）。
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

（每轮结束都回页首更新「最新一轮」那一行。）
