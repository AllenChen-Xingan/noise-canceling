# 现状页 · 手下 rs-entry（批次 20261102-b8-rust）

> 大总管：`noise-canceling-47`。交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-entry.md`。
> 管的实体：`vdn-control` · `vdn-capture` · `vdn-denoised` · `vdn-cli`（可执行 `vdn`）

## 手上
- 自审（三个对抗视角的子代理在跑）

## 判据结果（2026-11-02，本机 debug 构建，机器判；降噪块长 96，定块长 Pipeline）

| # | 判据 | 命令 | 结果 |
|---|---|---|---|
| V1 | 直通逐样本对齐 | `vdn chain-test --mode passthrough` | **过**：相关 1.000000（线 ≥ 0.999）· 实测延迟 2208 = L_total · 虚拟麦欠载 0 |
| V2 | 降噪死掉仍出声 | `vdn chain-test --kill-denoiser-at 2` | **过**：110.0 ms 切原声（线 ≤ 150）· 最长静音 0 ms；GTCRN 下同样 110.0 ms |
| V3 | 真模型出声 | `vdn chain-test --mode denoise --model-path <gtcrn_simple.onnx>` | **过**：均方根 0.1151（线 > 0.01）· 非有限 0 · 长度 240000 / 240000 |
| V4 | 两个真进程 | 见下 | **过**：相关 1.000000 · 实测延迟 2208 · 读出欠载 0 · 实时提升成功 · 结束后 unlink 成功 |
| V5 | 真麦克风（人判） | 见「V5 人工步骤」 | 待人做 |

- **延迟分项**（`with_block` 合入后）：
  - 直通：L_total = 2208 样本（46.00 ms）= Pipeline 768（16.00 ms）+ 环余量 1440（30.00 ms）
  - GTCRN：L_total = 2976 样本（62.00 ms）= Pipeline 1536（32.00 ms）+ 环余量 1440（30.00 ms）
  - 合入前（块长 480、`Pipeline::new`）：直通 2303（47.98 ms）· GTCRN 3071（63.98 ms）；V1–V4 当时也全过
- **加测 lead = 1 块（480，不作过线条件，只计数）**：
  - 降噪块长 96：直通 3 次「处理后欠载」1728 / 0 / 0 样本；GTCRN 3 次 480 / 480 / 1920 样本（缺的用延迟原声补，不是静音；虚拟麦欠载都是 0）。GTCRN 时 L_total = 2016 样本（42.00 ms）
  - 降噪块长 480（合入前）：直通、GTCRN 各 3 次全 0
  - 推断（未核实）：GTCRN 第一次推理慢（预热），10 ms 余量不够；debug 构建、降噪线程空闲时睡 0.5 ms 也有影响
- 坏块长：`--denoise-block 100` 启动即报错「块长 100 不整除 hop 768」【测】
- V3 用的权重：`../noise_canceling/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`（sha256 前缀 `b4718df6228e7bdf`；本工作区 `resources/` 里没有）
- V4 命令：
  ```
  ./target/debug/vdn-denoised --passthrough --bus /vdn-v4b --seconds 9 &
  ./target/debug/vdn-capture --source sine --sink file _tmp/20261102-rs-entry-v4/out2.wav --bus /vdn-v4b --seconds 5
  ./target/debug/vdn chain-check _tmp/20261102-rs-entry-v4/out2.wav --l-total 2208
  ```

## V5 人工步骤（给大总管转人）

⚠️ 这一步会开麦克风、从扬声器出声。**请戴耳机**：用外放时麦克风会收到扬声器的声音，可能啸叫。

1. 在仓库根目录跑 `./target/debug/vdn devices`（只列设备，不开麦、不弹框），确认默认输入、输出设备是你想用的
2. 戴上耳机，跑 `./target/debug/vdn run --passthrough --seconds 5`
3. 第一次跑时 macOS 会弹「终端想使用麦克风」，请点「允许」（点了之后要重跑第 2 步）
4. 5 秒内对着麦克风说话

**算过**：耳机里听到自己的声音，比嘴上晚一点点（链路内约 48 ms，另加设备缓冲），没有断续、没有爆音。
**算不过**：没声音、断断续续、有咔哒声或啸叫。结束时终端会打印「输出欠载 N 样本」，请把那一行发回来。

## 改了哪些文件
- 新：`src/control/**` · `src/capture/**` · `src/denoised/**` · `src/cli/src/{chain,live,verify}.rs`
- 改：`src/cli/src/main.rs`（新子命令 devices · run · chain-test · chain-check）· `src/cli/Cargo.toml`（依赖；forbid 去掉 cpal，留注释）
- 根 `Cargo.toml`：members 加三项；workspace 依赖加 `vdn-control` · `vdn-capture` · `vdn-denoised` · `serde` · `ctrlc`

## 新 crate 实体行（收尾时由大总管写进 `.42cog/cog.md`）
- `vdn-control`：配置文件与控制消息 · 人写配置文件、bin 读 · 是 `vdn-capture` / `vdn-denoised` / `vdn` 的输入
- `vdn-capture`：送音进程（lib `Forwarder` + bin）· 麦克风回调驱动 · 输入原声，输出 Out 环给虚拟麦克风；读 Processed 环与心跳
- `vdn-denoised`：降噪进程（lib `Worker` + bin）· 建总线 · 输入 Raw 环，输出 Processed 环与心跳；依赖 `Pipeline`

## 设计要点
- 对齐账只用 `L_total`（总线头里有）：Processed 第 j 个样本在输出时刻 `A(j − L_total) + L_total` 播出。迟到的丢、缺的用延迟原声补、Raw 环满的缺口单独记——降噪慢、卡、死、同进程恢复都逐样本对齐（单测覆盖）
- 心跳按音频样本计时（4800 样本 = 100 ms），回调里不读系统时钟
- `vdn-denoised` 不依赖 `vdn-capture`（后者带 cpal）；`Worker` 直接用 `vdn_shm::Bus`，单测用 `Bus::heap`

## 发现（本批不查，大总管记进 backlog，压延迟那批用 release 构建 + 预热核实）
- **现象**：环余量 = 1 块（480 样本，10 ms）且降噪块长 96 时，送音端「处理后欠载」（处理后的声音没按时到，用延迟原声补上；不是静音）
- **次数**：6 次里 4 次——直通 3 次：1728 / 0 / 0 样本；GTCRN 3 次：480 / 480 / 1920 样本。虚拟麦欠载 6 次都是 0
- **对照**：降噪块长 480 时同样 6 次全 0；默认环余量 1440 时全 0【测】
- **推断，未核实**：GTCRN 第一次推理慢（预热）吃掉 10 ms 余量；debug 构建、降噪线程没活时睡 0.5 ms 轮询也有影响
- 默认环余量保持 1440（大总管定）

## 已知缺口
- 环余量 30 ms，预算 3 ms：GTCRN 时 L_total 64 ms > 45 ms 预算（D32）。待实时线程 + 小块后压（大总管定）
- 降噪进程**换了个进程**重启：新流和旧对齐账对不上，靠「积压超限 → 重新对账」兜底，不保证逐样本对齐；新进程重建同名总线时送音进程要重开总线（本批不做自动重开）
- 降噪线程没活时睡 0.5 ms 轮询（B 档；后续换跨进程唤醒）
- `--sink hal`：本批没有插件读 Out，环满后计溢出

## 依赖许可
- `serde` 1（MIT OR Apache-2.0）· `ctrlc` 3（MIT OR Apache-2.0）——与 GPL-3.0 兼容（子代理复核中）

## 待人拍板
- 无

## 流水
- 2026-11-02 报到；三个零件手下的 worktree 还没有新代码
- 2026-11-02 收到三家签名；lead 定为参数、默认 1440（大总管）
- 2026-11-02 `Forwarder` 第一版「单个净账」在 Raw 溢出 + 处理后缺失同时发生时错位（单测红）→ 改成按输出时刻逐样本记账 + 缺口表，转绿
- 2026-11-02 大总管合入三家零件（65e62a9）；接线：capture 的 shm/hal 适配、两个 bin、vdn devices / run / chain-test / chain-check
- 2026-11-02 check-deps 报 vdn-denoised 闭包有 cpal（经 vdn-capture）→ Worker 改直接用 Bus，capture 改 dev 依赖，复跑通过
- 2026-11-02 V1–V4 过；lead = 1 块加测 6 次欠载 0；Ctrl-C 退出后总线已删【测】
- 2026-11-02 大总管合入 `Pipeline::with_block`（fd26a1d）；`Config.denoise_block` 默认 96，vdn-denoised 与 chain-test 用定块长，`vdn run` 仍用 `Pipeline::new`；V1–V4 重跑全过
- 2026-11-02 4 个 crate 27 个单测过（含 1.2 s 版 V1 / V2 真共享内存测试），clippy 0 警告，fmt 过
