# 现状页 · 手下 rs-entry（批次 20261102-b8-rust）

> 大总管：`noise-canceling-47`。交代：`../noise_canceling-skeleton/_tmp/20261102-b8-rust/rs-entry.md`。
> 管的实体：`vdn-control` · `vdn-capture` · `vdn-denoised` · `vdn-cli`（可执行 `vdn`）

## 手上
- 等零件合入（rs-io 已合进骨架分支；rs-core、rs-shm 未齐），然后接线、跑 V1–V4

## 做到哪
- `vdn-control`：`Config`（含 `lead_frames`，默认 1440）· `Cmd` · 校验，6 个单测过【测】
- `vdn-capture` lib：`Forwarder`（对 `Push`/`Pop`/`Heartbeat` 泛型）+ 假环，8 个单测过【测】
  - 对齐账只用 `L_total`（总线头里有）：Processed 第 j 个样本在输出时刻 `A(j − L_total) + L_total` 播出；
    迟到的丢、缺的用延迟原声补、Raw 环满的缺口单独记——降噪进程慢、卡、死、同进程恢复都逐样本对齐（单测覆盖）
  - 心跳按音频样本计时（4800 样本 = 100 ms 不动 → 切原声，10 ms 淡化），回调里不读系统时钟
- `vdn-denoised` lib：`Worker`（prime 推 lead 个零 · step 一块一心跳），3 个单测过【测】；bin 暂为「未实现」占位
- `vdn` 的判据纯函数（`src/cli/src/verify.rs`：相关 · 找延迟 · 最长静音），5 个单测过【测】
- clippy（三个新 crate，`--all-targets`）0 警告【测】

## 下一步
- 合入后：capture 里写 shm / hal 适配；两个 bin 接线；`vdn devices` · `run` · `chain-test`
- cli 的 `metadata.vdn.forbid` 去掉 `cpal`（大总管定）

## 新 crate 实体行（收尾时由大总管写进 `.42cog/cog.md`）
- `vdn-control`：配置文件与控制消息 · 人写配置文件、bin 读 · 是 `vdn-capture` / `vdn-denoised` / `vdn` 的输入
- `vdn-capture`：送音进程（lib `Forwarder` + bin）· 麦克风回调驱动 · 输入原声，输出给虚拟麦克风的 Out 环；读 Processed 环与心跳
- `vdn-denoised`：降噪进程（lib `Worker` + bin）· 建总线 · 输入 Raw 环，输出 Processed 环与心跳；依赖 `Pipeline`

## 已知缺口
- 环余量 30 ms（lead 1440 样本），延迟预算 3 ms：GTCRN 时 L_total = 1631 + 1440 = 3071 样本（64 ms）> 45 ms 预算（D32）。待实时线程 + 小块后压（大总管定，2026-11-02）
- 降噪进程**换了个进程**重启时，新进程的流和旧对齐账对不上：靠「积压超限 → 重新对账」兜底（不保证逐样本对齐）；若新进程重建了同名总线，送音进程要重开总线（本批 bin 不做自动重开）

## 依赖许可
- `serde` 1（MIT OR Apache-2.0，Cargo.lock 已有）——与 GPL-3.0 兼容

## 流水
- 2026-11-02 报到；三个零件手下的 worktree 还没有新代码【测：git status 干净】
- 2026-11-02 收到三家签名（rs-core Pipeline · rs-shm Bus · rs-io io-macos/hal）；lead 定为参数、默认 1440（大总管）
- 2026-11-02 `Forwarder` 第一版用「单个净账」对齐，raw 溢出 + 处理后缺失同时发生时错位（单测 `raw_overflow_keeps_alignment` 红）→ 改成按输出时刻逐样本记账 + 缺口表，转绿
- 2026-11-02 control / capture / denoised / cli-verify 共 22 个单测过；clippy 0 警告
