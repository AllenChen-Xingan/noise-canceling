# 020 · 手写 GTCRN 引擎提速（研究 020 / exp012）· codex 换谱系评审

> 对象：`docs/research/020_fastpath/decision.md` · `rewrites.md` · `docs/experiments/exp012-fastpath/README.md` · `docs/experiments/exp010-inference-bench/bench/src/`（含 bound_table.py 作依据）
> 类型：research · effort high · 停止线：连续两轮无 P0/P1（codex-review skill）
> 提示词与原始输出：`_tmp/020-reviews/`（不入库）

## 第 1 轮（2026-11-02 22:21 · codex gpt-6-astra · read-only · high · 149 s）

结果：P0×0 P1×3 P2×2 P3×0 · 总评：修改

| # | 级 | 问题 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P1 | 下界只是按微基准中位数的估算，非线性延迟测的是标量 sig_fast（含额外乘加）而非 v4i 的向量核，不能说「任何写法都越不过」 | 属实 | decision.md 改称「按本机实测峰值估算的性能模型下界，不是数学证明」，列出三点局限；一句话结论同步改 |
| 2 | P1 | v2 / v4 后端仍读全局开关：TRA 的 sigmoid 读 FAST，DPGRNN 全连接读 BLAS；同进程先开 native-fast / native-blas 会改变 v4i 的激活算法、让它每帧分配 | 属实（bug） | `Kern` 加 `sig1` / `use_blas`：原写法照旧读开关，v2 / v3 / v4 固定 libm sigmoid、不走 BLAS；`Point::run_plain` 给 v2 退路用。测试：单元 `v2_v3_v4_ignore_global_switches`；端到端里 BLAS / FAST 置真后 v4i 610 帧与开关为假时逐位相同。11 个后端对拍数不变 |
| 3 | P1 | 「沿频率 GRU 再快只能改算法」漏了已调研的等价优化：输入投影整段提前算 | 属实 | decision.md 后续第 4 条改为 A 类候选「输入投影整段提前算」，写清做法与验证要求 |
| 4 | P2 | DSP 节奏对照固定原版先跑，醒来状态偏向后跑者，约 46 µs 不能全归因点积 | 属实 | `dsp.rs` 改成按帧交替先后；decision.md 第四节加 ⚠️ 注明旧数据的偏差、节奏组待复测 |
| 5 | P2 | 97 抽头误差界 7u 与 `dot_neon` 实际计算图不符，应按最长路径 γ₁₁ | 属实 | rewrites.md E1 改为 γ_d 的一般形式并写前提；#9 改为 γ₁₁ ≈ 6.6e-7·Σ\|hᵢxᵢ\| |
| 抽查「不符」 | — | 「与模型实际持有的参数个数比对」：param_count 不含转置副本 | 属实 | decision.md 改为「逻辑参数个数，每份计一次，转置副本不计」 |

采纳 6 / 提出 6。

## 第 2 轮（2026-11-02 22:28 · codex gpt-6-astra · read-only · high · 146 s）

结果：P0×0 P1×0 P2×5 P3×0 · 总评：修改 · 停止线：连续第 1 轮无 P0/P1。上一轮 #2 #3 #5 与参数计数改法，codex 抽查属实。

| # | 级 | 问题 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P2 | LayerNorm 只引 E1 + E2 不够：均值误差经 1/√(var+ε) 放大，近常量输入（528 个 0.1）两版差约 4e-3 | 属实 | rewrites.md #11 写清误差传播与病态区；注明原写法同样病态、整网只靠 BE-PARITY 证明真实输入不落在该区 |
| 2 | P2 | BN 折叠「相对误差 ≤ 2u」不成立，偏置有相消 | 属实 | rewrites.md 0a 分开写权重（约 3u 相对）与偏置（只有绝对界）的误差，引反例 |
| 3 | P2 | README 仍写「省约 46 µs/帧」，没同步上一轮 #4 | 属实 | README 同步标为有顺序偏差的旧观测、待交替对照复测 |
| 4 | P2 | bound_table.py 节奏模式仍用作废的 rt 连续跑常数换算非线性延迟；README 复现命令选 rt | 属实 | 脚本：节奏模式只认 `hwpeak_ui.jsonl`，连续跑模式拒绝 `--qos-peak rt`；README 复现命令改为 ui，重算结果与 decision.md 一致（v4i 整帧 2.5 倍） |
| 5 | P2 | 排除 SME 的依据扩大成了硬件事实；调研核实的是 M4 上 tract 的代价模型，切换开销本身未量化 | 属实 | rewrites.md「不做」与 decision.md 两处改为「按该模型推断，M5 未测」 |

采纳 5 / 提出 5。

工具注：第 2 轮第一次发送被隐私扫描拦下——上一轮输出以「P1 #1」开头，`file` 把它当成 PBM 位图，判「非纯文本」。在输出文件开头加一行 HTML 注释后识别为 text/plain。以「P1」「P2」开头的 codex 输出都会碰到，已报大总管。

## 第 3 轮（2026-11-02 22:37 · codex gpt-6-astra · read-only · high · 128 s）

结果：P0×0 P1×0 P2×3 P3×0 · 总评：修改 · **停止线：第 2、3 轮连续无 P0/P1 → 满足，停审**。上一轮 #3 #4 #5 codex 抽查属实。

| # | 级 | 问题 | 核实 | 处理 |
|---|---|---|---|---|
| 1 | P2 | LN：610 帧整网对拍不能证明真实输入、也不能证明中间层避开病态区 | 属实 | rewrites.md #11 改为「该样本端到端通过」，写明缺中间层证据；补测列为后续（issue） |
| 2 | P2 | 「整帧 36.2 µs」是打点区间求和，不是无打点整帧，又拿去和无打点的节奏耗时比 | 属实 | decision.md 表头改为「打点区间耗时之和」；后续第 1 条写明两者口径不同、倍率只是粗看；同路径复测列为后续（issue） |
| 3 | P2 | 降采样对照系数是 fl(fl(3h)/3)，不保证还原正本 h，对照差含系数舍入 | 属实 | decision.md 第四节与 rewrites.md #9 单列这项误差；改取系数列为后续（issue） |

采纳 3 / 提出 3。剩余要做的 3 件（不是文档措辞）记在 decision.md「后续」，请大总管开 issue。

**合计**：3 轮，采纳 14 / 提出 14（P1×3 P2×10 + 抽查不符 1）。
