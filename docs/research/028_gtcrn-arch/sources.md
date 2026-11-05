# 028 来源

> 核到哪一级：「全文」= 读了正文表格；「摘要」= 只读到摘要；「本仓」= 本仓库已有文件。

## 外部文献

| # | 来源 | 用在哪 | 核到 |
|---|---|---|---|
| S1 | Rong 等，GTCRN（ICASSP 2024）· 官方代码 `resources/gtcrn/`（主目录，只读） | 结构、参数量 23.67 K、33.0 MMACs/s（源码头注释）；TRA = 沿时间的 GRU 通道注意力，已在 GTCRN 里 | 本仓代码 |
| S2 | Rong 等，UL-UNAS（arXiv 2503.00340v2，2025）表 III | 在 GTCRN 式原型上加「因果时频注意力」：时间支 PESQ 1.99 → 2.09、ESTOI 74.08 → 75.44，+48 K 参数、+3.2 MMACs/s；频率支单独 1.99 → 2.00（「几乎没用」）；两支合计 2.15、+5.9 MMACs/s。频率支用双向 GRU，不是自注意力；时间支就是 GTCRN 的 TRA | 全文（HTML） |
| S3 | 同上，表 VI（DNS3 测试集，统一重训） | GTCRN 48 K / 34 MMACs/s · PESQ 2.10；UL-UNAS 171 K / 35 MMACs/s · PESQ 2.30 | 全文 |
| S4 | Wang 等，Adaptive Convolution for CNN-based Speech Enhancement Models（arXiv 2502.14224v2，IEEE TASLP 2025）表 III / V | GTCRN 把普通卷积换成逐帧动态卷积：23.67 K → 117.36 K 参数，33.83 → 40.75 MMACs/s（+20%），PESQ 2.130 → 2.292，ESTOI 0.727 → 0.747，SI-SNR 13.55 → 14.30（DNS5 测试集） | 全文（表格经抓取工具转述）：PESQ +0.16 与论文正文「约 0.16」一致；DNSMOS 几列的对齐是推断 |

检索范围：两次网络搜索（UL-UNAS / GTCRN 改进变体与注意力，2025），读了 S2、S4 全文 HTML；未系统检索，不能说「不存在」。

文献数字都是论文自己的测试集（DNS3 / DNS5，英文为主），**不能直接换算成我们成绩单上的分数**；只用来排先后。

## 本仓材料

| # | 文件 | 用在哪 |
|---|---|---|
| R1 | `src/models/src/lib.rs`（`ModelKind::declared`）· `src/types/src/latency.rs` · `src/models/tests/latency.rs` | GTCRN 延迟 = 攒一帧 256 点 + 算法延迟 256 点（各 16 ms）+ 平台预算；看未来帧最少加 1 帧 = 16 ms |
| R2 | `docs/research/020_fastpath/decision.md` | 手写 Rust 版 v4i：整帧 36.2 µs（性能核连续跑）、0.073–0.092 ms（16 ms 节奏）；逐层耗时，两层双路 RNN 合计 17.47 µs |
| R3 | `docs/research/019_apple-inference/standard.md` | 占用口径：§6.1 的 CPU 分摊（GTCRN 推理占一个核目标 0.50%、上限 1.00%）在 D75 后**只作参考**；现行挡线是对性能的影响（`BE-IMPACT`）与每帧耗电（`BE-POWER`）；§9：Python 测的数不进结论 |
| R4 | exp016 弱项 / 守住项（cands worktree，`docs/experiments/exp016-candidates/runs/weak-spots-20261105/weak_spots.md`，未合入 main） | 弱项 83 组、守住项 71 组 |
| R5 | exp018 单步训练实测（finetune worktree，`docs/experiments/exp018-gtcrn-finetune/runs/20261105-145149-step/`，未合入 main） | MPS 批量 32 一步 0.821 s（每墙钟小时约 156 小时音频）；CPU 批量 8 一步 0.368 s |
| R6 | 本调研的实测：`docs/experiments/exp019-gtcrn-arch/` | 每帧乘加、参数、缓存、torch 单帧耗时；热启动与因果性测试 |
