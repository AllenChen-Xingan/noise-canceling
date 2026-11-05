# DPDFNet-2 的 87.7 ms 怎么拆、哪些能降

> 2026-11-05 · cands · 读源码 + 读官方仓库 issue + 已有实测，**未经 codex 审**；没跑新的模型（不占名额）
> 官方仓库本地副本：`resources/DPDFNet`，提交 `9bd9844`（远端 `ceva-ip/DPDFNet`）

## 一句话

87.7 ms 由三部分组成：
- 模型本身 60 ms，结构定死，换推理引擎去不掉。
- 评测框架给外部系统多记了 16 ms。以「整窗」口径、产品按模型自己的 hop 调度为前提，这 16 ms 可以去掉（推断，未验证）。
- 平台预算 11.7 ms，和 GTCRN 一样。

去掉 16 ms 后约 71.7 ms，仍超 45 ms 的线 26.7 ms。就算按维护者说的「约 40 ms」算，也要 51.7 ms，还超 6.7 ms。

只靠重写推理引擎、不重训，过不了线（推断，依据见下）。「2」不是前视帧数，是网络里双路 RNN 块的个数。官方没有前视更短的版本。

## 一、87.7 ms 的拆分（成绩单 run `20261105-115707-fcf7cfb0`，每段都一样）

| 部分 | 毫秒 | 出处 | 能不能降 |
|---|---|---|---|
| 模型算法延迟（声明值） | 60.0 | `run_dpdfnet2.py` 头注释：4 帧 × 160 + 窗长 320 = 960 样本 @16 kHz。V-LAT 记录的 `measured_algo_ms` 也是 60.0，但回放会按声明值在输出前补零（`external.rs:417`），所以它只证明「补齐无误」，**不独立证明结构下限**。结构下限靠第二节的源码推导 | 见第二节。结构定死，不重训降不了 |
| 外部回放的「攒满一个 hop」 | 16.0 | `src/engine/src/engine.rs` `latency_ledger` 的 hop-buffer 项；外部回放的 hop 固定取 `FRONTEND.hop` = 256 @16 kHz（`src/eval/src/system/external.rs:27`、`src/types/src/frontend.rs:65`） | **有条件能**（推断，未验证）。这是评测框架的记账口径，不是模型的。前提有两条：60 ms 按「整窗 + 前视」声明，已含攒一个 hop 的等待，和 GTCRN「hop 16 + 模型 16 = 窗 32」同一口径；产品里按模型自己的 hop（160）调度。两条都成立，这 16 ms 就是重复记账（#125） |
| 平台预算 | 11.67 | `src/types/src/latency.rs` `planned_platform`：重采样两跳 2 + 2、麦克风 IO 块 96 帧 2.0、跨进程环 3.0、HAL 输出 2.67（都 @48 kHz） | 这是预算，不是实测（`Basis::Budget`）。GTCRN 也是 16 kHz 模型，同样付这 11.67。和换推理引擎无关 |
| 合计 | 87.67 | `16 + 60 + 11.67`，与成绩单 V-LAT 值一致 | |

同一个 16 ms 也记在 DFN3 账上：41.4 = 声明 17.75 + 16 + 平台 7.67（DFN3 原生 48 kHz，不记重采样两跳）。GTCRN 是内置模型，没有这一项：43.67 = hop 16 + 模型 16 + 平台 11.67。

**推理引擎快慢不影响这张账**：算法延迟只看「输出要等多少输入」，和每帧算多快无关。每帧算力只要不超实时，就不进延迟账；它另由「最坏一帧耗时」管（V-FRAME，外部回放量不到）。

## 二、60 ms 里哪些是模型结构定死的

**结构（读源码）**：
- 两种前视都是 2 帧：`model/dpdfnet.py:383-384` 写 `df_lookahead=2`、`conv_lookahead=2`。
- 默认先掩码、后滤波：`mask_method='before_df'`（`model/dpdfnet.py:389`；`forward` 在 `:554-556`，先 `self.mask(spec, m)`，再对掩码后的谱做 `df_op`）。
- 后一级滤波（DF）要看掩码后的谱往后 2 帧，而掩码本身已经要看输入往后 2 帧。所以输出第 t 帧，最晚要等输入到第 t + 4 帧。
- 流式 ONNX 版照这个结构实现：掩码那级缓冲延 2 帧（`onnx_model/layers.py:377-383`，`delay_frames=2`），DF 系数再延 2 帧（`onnx_model/multiframe.py:171-175`）。官方离线推理脚本也按 4 帧对齐（`onnx_model/infer_dpdfnet_onnx.py:18` `ATTN_LIMIT_NOISY_FRAME_OFFSET = 4`，`:177` 注释「advances the output by ~4 hops」）。
- 所以 60 ms = 窗长 20 ms + 4 帧 × 10 ms。

**所有官方权重都一样**（读源码）：
- 16 kHz 版：导出脚本 `onnx_model/export_dpdfnet_to_onnx.py` 导入的是 `onnx_model/dpdfnet.py`（第 9 行）。它的默认值同样是 `df_lookahead=2`、`conv_lookahead=2`、`mask_method='before_df'`（`onnx_model/dpdfnet.py:547-553`）。`build_model`（`:91-105`）只按版本改 `dprnn_num_blocks`，这三个参数都不改。所以 baseline / 2 / 4 / 8 的前视相同。
- 48 kHz 版：另有导出脚本 `onnx_model/export_dpdfnet_48khz_hr_to_onnx.py`，导入 `onnx_model/dpdfnet_48khz_hr.py`（第 9 行）。它的默认值也是 2 / 2 / `before_df`（`:612-618`），导出脚本没传这几个参数（读源码）。帧长和 hop 不同，所以毫秒数要另算，本文不算。
- 以上是「读源码 + 导出配置」的结论。权重文件里的实际前视没有逐个实测，只有 dpdfnet2（16 kHz）有对齐实测（下文）。

**换推理引擎能去掉吗**：去不掉（推断）。
- 这 4 帧是权重训练时用的输入时序：输出第 t 帧真的用到了第 t + 4 帧的输入，不是多余的缓冲。
- 缩短等待，就得在缺最后几帧输入的情况下算，等于改了模型看到的输入，没训过。
- 要短于 4 帧，只能重训：零前视或前视更短的版本，或改成掩码和滤波并联（`mask_method='separate'`，两级各看 2 帧，合计 2 帧）。
- 官方没开源训练代码（issue #5、#10，维护者：训练代码属 Ceva 专有，暂无计划）。

**和维护者说法的出入**：
- 维护者在 issue #14（2026-04-16）说：「当前权重用 df_lookahead=2 和 conv_lookahead=2，整体延迟约 40 ms」。
- 这个数和上面的 4 帧结构对不上：他们自己的 ONNX 图和推理脚本都按 4 帧走。推断他是按「窗 20 + 前视 2 帧 20」算的，没把两级串联叠加进去。
- 已有实测（实测）：官方 ONNX 流式输出按 4 帧平移后，和输入的互相关滞后正好是 0（本次 run 的 `lag_after_align` = 0）。这证明「现成实现」晚 4 帧，**不单独证明结构非要 4 帧不可**。结构这一点靠上面的源码推导；因果实测是下面的严容差截半，还没做。
- 截半实测的前视是 532 样本（33 ms，容差 1e-4）。它只是下限：远处帧影响小，低于容差就测不到，所以分不出 40 ms 还是 60 ms。
- 要定论，可以用 1e-7 的容差再截半跑一段。这要跑降噪模型，需要名额。**建议，待定。**
- 就算按 40 ms 算，40 + 11.67 = 51.7 ms，也还超线。

## 三、「2」是什么：双路 RNN 块的个数，不是前视帧数

- README 的导出命令：`--dprnn-num-blocks 2` 对应 `dpdfnet2`，4 对应 `dpdfnet4`，8 对应 `dpdfnet8`（`README.md:237-257`）。
- `model/dpdfnet.py:47` `dprnn_num_blocks`；`baseline` 就是 0 块（`README.md:46` 参数表：baseline 2.31 M，dpdfnet2 2.49 M）。
- 本地有 6 份官方权重：baseline、dpdfnet2、dpdfnet4、dpdfnet8，另有 48 kHz 的 dpdfnet2 和 dpdfnet8（`resources/weights/dpdfnet/`）。前视都相同（第二节）。

**有没有前视 0 或 1 的版本**：没有。
- issue #7（2026-02-07）：有人问考没考虑 DFN3 低延迟版那种零前视，维护者答「No」。
- issue #14：维护者说零前视版本「possible」，但质量取舍不同，也没给发布计划。

**不重训能不能缩短前视**：不能。理由见第二节，属推断，未做实验。

## 四、要过 45 ms 线，模型本身最多能占多少

- 45 − 平台 11.67 = **33.3 ms**（16 kHz 模型；整窗加前视都算在模型里）。
- DPDFNet 的窗 20 ms + 每帧前视 10 ms，最多只能带 1 帧前视（30 ms）。现有权重是 4 帧。
- 零前视 + 窗 20 ms ≈ 31.7 ms，能过线，但要重训。
