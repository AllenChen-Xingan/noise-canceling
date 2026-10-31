# 取舍卡 · 本项目 Rust 代码怎么组织

> 2026-10-30，手下 rustarch 起草。**一页纸**；细则在 `specs/rust/architecture.md`，理由在 `adr/001–005`，证据在 `notes/` 与 `appendix-probes/`。
> ⚠️ 末尾「待人拍板」**AI 不代填**：选项和建议是草稿，「拍了吗」一列留给人。
> 证据等级：【官】【源】【测】【传】【推】。

## 这次不可妥协的几条（`must-know.md`）

1. 离线评测与实时用同一份处理代码，核心不依赖平台
2. 实时路径不分配、不加锁、不阻塞、不 panic，**由机器证明**
3. 模型可替换：加一个模型 = 加一个适配器文件 + 一行注册

## 我选择什么

> **从 p1 骨架分阶段长出 crate 图（核心 = types · dsp · models · engine · speaker，平台与推理库只在叶子）；用 AST 与依赖图工具守规矩（一份 clippy 禁用清单 + cargo-deny + xtask check-deps / tidy + RTSan），不用宏生成代码；实时分三档，推理线程允许 ONNX Runtime 内部分配、靠截止时间与送原声兜底。**

## 我放弃什么

> **放弃「推理线程零分配」的承诺（ort 做不到，实测）；放弃一次建满 15 个 crate；放弃过程宏、build.rs 生成和 dylint（先不上，写明触发条件）；放弃 `Box<dyn Model>`，改手写 enum。**

## 依据来自哪一类

- [x] 自有积累：设计稿 §6 / §7 / §11；003 技术方案与管线取舍卡；D03 / D04 / D32 / D33；p1 骨架代码；rusteco 009
- [x] 他人积累：matklad 与 rust-analyzer 架构文档；Cargo / Clippy / cargo-deny 官方文档与源码；rtsan-standalone、assert_no_alloc、ort 等源码；nih-plug / firewheel / fundsp / tympan-aspl 的分层做法；ADC 演讲（Eckes 2024、Hobson 2025、Trevelyan & Apple 2024）与 Doumler 的锁文章
- [x] 自己跑的：本机探针 6 组（`appendix-probes/`）。**改变了一个取舍**：ort 跑 GTCRN 每帧约 1554 次 malloc、376 对 mutex【测】→ 从「推理线程零分配」改成三档（ADR 002）。另外 clippy 配置的三个探针改掉了 r1 / r4 的三处写法

## 拿两把尺子量一遍

### 一、复用分级

| 来源 | 权限 | 这次实际用到哪一步 |
|---|---|---|
| **自家的** | 可照搬 | p1 的 trait、Engine、三件保命原样保留，只加约束；设计稿 7 个 crate 逐个给了去留理由（ADR 001） |
| **外部的** | 只读借鉴 | 只借做法与结构（tidy 测试、实时模块 deny 头、回收队列、封闭 enum 消息），规约里没有成段的外部源码；§5.1 的禁用清单是按 clippy 配置格式自己写的 |

⚠️ 交付前已回参考区比对：规约与 ADR 只有转述和 `文件:行号`；`appendix-probes/` 的探针是子代理自写的小程序。

### 二、满意与最优

**停止线**：三条命门在首选方案下都有「怎么满足 + 哪种机器检查」，每个缺口都有去处（规约里有规则、§8 待测、或交人拍板）。到这里就停。没继续做：ast-grep 规则实写、cargo-deny 实跑、tract 实测——都在 §8，归后续实验。

## 支持它的最强依据

> 1. **ort 实测**：IoBinding、单线程、预热后，稳态每帧 malloc 1554 / free 1555 / mutex 376 对，1 帧 → 2 帧严格翻倍；而最坏一帧耗时约 1 ms、hop 16 ms【测】`appendix-probes/README.md` §1。分档不是让步，是按事实分配证明力度。
> 2. **cargo-deny 源码**：`wrappers` 只查被禁 crate 的直接依赖者【源】`resources/cargo-deny/src/bans.rs:616-650`——所以「依赖方向」必须两层查，这一条直接塑造了检查栈。
> 3. **rust-analyzer 的做法**：核心「never does any IO」，tidy 检查随 `cargo test` 跑，不另设 CI 步骤【官】【源】——证明「用规则守架构」在 20 万行级项目上可持续。

## 反对它的最强依据

> **没有一个 Rust 音频项目同时做到「两层进程 + 推理库在实时线程 + 机器证明」。** 我们的检查栈是从不同项目拼起来的（rust-analyzer 的 tidy、tympan 的 deny 头、rtsan-standalone、cargo-deny），拼在一起有没有漏洞没人验证过；RTSan 只覆盖回放跑到的路径；cargo-deny ban 工作区成员**没实测**。另外按默认参数延迟约 56 ms，超 D32【推】rusteco 009 §一.3——规约给了调参目标，但能不能调进 45 ms 要 P3 才知道。

## 仍未补齐的缺口

见规约 §8：E-lat（延迟实测）、E-tract（tract 能否跑 GTCRN）、E-ort（关 arena 后分配能降多少）、E-deny（cargo-deny 实跑）、E-load（负载下 p99.9）、E-rate（音色链采样率）。

## 三个风险

1. **延迟超 45 ms**：默认参数约 56 ms。缓解：R13 记账 + P3 第一天脉冲实测 + 调参目标；实在不够交人拍板（#6）
2. **B 档豁免文件变成「什么都放行」**：缓解：豁免只许按 `libonnxruntime` 栈，改豁免要在 PR 里说明；升级 ort 必须重跑 G8
3. **规约比代码先行，p1 收尾时照不照改**：缓解：§2.3 只给 P1 列了 6 件小改，其余都挂在后续阶段的触发条件上

---

## ⚠️ 待人拍板

「默认」= 人不拍时 AI 实际怎么做、做到哪个里程碑为止。

| # | 题面 | 选项 | 代价 | 建议 | 默认（不拍时） | 拍了吗 |
|---|---|---|---|---|---|---|
| 1 | **「音频线程不分配、不加锁」改成分三档表述吗？**（`.42cog/intent.md` 作品区的措辞）推理线程里 ONNX Runtime 每帧约 1554 次内部分配，Rust 这边改不掉 | A 改成三档：IO 回调零容忍；推理线程我们的代码零分配、ORT 内部分配登记在案、靠截止时间 + 送原声兜底 / B 坚持推理线程零分配：换 tract 或自写推理，先做实验 | A：intent 措辞变软，多一个豁免文件要维护；B：卡住 P4，tract 能不能跑 GTCRN 还不知道 | A，同时继续做 E-ort / E-tract 缩小豁免 | 按 A 写规约与代码；intent.md 不改（只有人能改），到 P4 前再问 | ☐ |
| 2 | **crate 目录名要不要和包名一致？** 现在是 `src/dsp/` 里装 `vdn-dsp` | A 保持现状（短目录名 + `vdn-` 包名）/ B 目录也叫 `src/vdn-dsp/`（matklad 主张目录名 = crate 名） | A：目录与包名差一个前缀；B：p1 要改 4 个目录名和 workspace members | A | A | ☐ |
| 3 | **release 构建 panic 时直接退出进程吗？**（`panic = "abort"`） | A abort：推理进程退出，送音进程送原声，再重启 / B unwind：线程里可以接住 panic 再继续 | A：每次 panic = 一次重启 + 一段原声；B：接住后状态可能已坏，更难预测，二进制更大 | A | A（P1 离线 CLI 无所谓，P4 前生效） | ☐ |
| 4 | **两层进程做成两个程序还是一个？** | A 两个：`vdn-capture` 与 `vdn-denoised` / B 一个程序、用参数选角色 | A：多一个二进制；B：「送音进程不链推理库」就没法由机器证明（D03） | A | A（P4 才涉及） | ☐ |
| 5 | **HAL 插件（C）的目录放哪？** 这是新顶层目录（CLAUDE.md 铁律 10） | A 顶层 `hal-plugin/` / B 放 `src/hal-plugin/`（和 Rust crate 同级，但不进 workspace） | A：多一个顶层目录；B：`src/` 里混进非 Rust 代码，规约「`src/` 下都是 crate」不成立 | A | M3 前不建 | ☐ |
| 6 | **P3 实测延迟仍超 45 ms 时，先砍哪一级？** | A 换更低延迟的重采样器 + 缩小 IO 缓冲（可能多一点 CPU 和掉帧风险）/ B 改走「推理进程直接写 HAL 环」少一跳（003 的二期共享内存）/ C 放宽 D32 | A：功耗与稳定性；B：要先做自写 HAL 插件，提前 003 二期；C：违背已拍板的 D32 | 先 A，不够再 B | P3 实测出来前不动 | ☐ |
| 7 | **批准一次 tract 对照实验吗？** 要联网下载 `tract-onnx 0.23.9` 并占一次重活名额 | A 批准，P1 收尾后做 / B 只在 ort 出问题时再做（rusteco 009 的建议） | A：约半小时 + 下载；B：B 档能不能更干净一直不知道 | B | B | ☐ |
