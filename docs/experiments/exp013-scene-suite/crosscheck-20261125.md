# exp013 本机与云上 build 交叉核对（2026-11-25）

> 本机 build `20261124165427-6323`（macOS · arm64）对云上 build `20261125022300-aa9f`（cloudrun · RunPod CPU 机 · Linux）。
> 云上 build 的小文件原样放在 `builds/20261125022300-aa9f/`（取回目录与仓内逐文件 `diff -r` 相同）；音频只在云上网络盘 `/workspace/exp013-audio`。

## 结论

**两份 build 不是逐字节相同，但内容等价**。所以「逐字节一致」这条验收在跨平台时达不到。
原因（推断，有证据，没在两台机上逐步追到）：两台机器的浮点数学库不同（macOS arm64 与 Linux），同一个随机数推出来的参数在最后几位就不一样了。所以每段的音频样本都有微小差别。

## 证据（实测，`compare_builds.py` 与逐字段对比）

| 核对项 | 结果 |
|---|---|
| `compare_builds.py`（云上跑一次、本机复跑一次） | 不一致 1086 处，两边输出相同；退出码 1 |
| manifest：代码 · 参数 · 素材索引 · 依赖版本 · 基础种子 | 全部相同；素材索引 `a8de10ad…`，1994 个文件 |
| 泄漏检查 · 电平验收 · 各场景段数 · 排除表 `exclude.json` | 全部相同 |
| 逐段逐轨样本哈希 | 480 段里没有一段完全相同：owner 480 段 · owner_early 480 段不同，mix 177 段 · interf 279 段 · noise 142 段，分轨 1122 条 |
| 非哈希字段里的数值 | 最大差 4.5e-13（`sounds.src.params.lowpass_hz`）；`chain.direction_nodes_db` 差 3.6e-15。量级是浮点最后几位，不是逻辑差别 |
| 四舍五入后的实测值（SNR · SIR · 电平 · T60 等） | 相同 |

## 对后续的影响

- `vdn-eval` 读每段前会核 wav 文件的 sha256。云上评测若用仓里 `src/eval/suite/scenes.json`（指向本机 build 6323）去读云上生成的音频，每段都会因哈希对不上被记成「没跑成的段」
- 所以云上评测要么用云上 build `aa9f` 自己的清单，要么把本机 6323 的音频（约 3.7 GB）传上去。待大总管定
- 「样本差多少才算等价」要用音频才量得出：取几段云上 wav 回本机，量逐样本最大差与差信号电平。尚未量化

## 怎么复现

```
python docs/experiments/exp013-scene-suite/compare_builds.py \
  docs/experiments/exp013-scene-suite/builds/20261125022300-aa9f \
  docs/experiments/exp013-scene-suite/builds/20261124165427-6323
```
