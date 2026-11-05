# exp017 · 音色项三种量法在已知劣化上的表现

> 2026-11-05 · 手下 timbre · 调研 026 · issue #119。只评不改：没动 `src/eval/` 的评分代码。

## 假设

对干净人声做已知程度的劣化，好的音色量应满足：
- 按程度分得开；
- 同样大小的变化放在不同频率读数相近；
- 不受整体音量和残留噪声影响。

现行代码和标准 N4 两种量法谁更接近这些，还是都不满足？

## 做法

- **素材**：exp013 正式 build `20261103230756-17f5` 的 `owner.wav`（干净参考）。每个场景每种设备取第一段，15 × 2 = 30 段，每段 12 s、16 kHz。哈希在 `runs/<run>/inputs.json`。
- **模拟的是「干净人声单独过一遍」那一路**：输出 = 劣化（参考），没有停顿掩码，不扣噪声（同 `runner::Prepared::clean_view`）。另算一列 `*_noisy_path`：按主讲人掩码外扩 30 ms 求停顿帧、扣残留噪声，作带噪那一路的旁证。
- **劣化**（`run.py` 的 `DEGS`，共 66 种）：
  - 只改音量 −10 / +6 dB；
  - 低通 4 / 2 / 1 kHz（8 阶，零相位）；
  - 3 kHz 高搁架 ±3 / ±6 dB；250 Hz 低搁架 ±6 dB；
  - 窄带：2–4 kHz 压 6 / 12 dB，0.5–1 kHz、1–2 kHz 压 12 dB，2–4 kHz 抬 6 dB；
  - 整体倾斜 −1 / −2 dB/倍频程；
  - 白噪声 / 粉噪声，信噪比 30 / 20 / 10 dB；
  - T12 阈值点：两倍频程带（中心 150 / 600 / 4 800 Hz）±3.5 / ±7.3 dB，600 Hz −12.5 dB；
  - 滤波叠加增益：低通 4 / 2 kHz 后再 ±20 dB（查电平不变性）；
  - 定标组：倍频程带（中心 250 / 1k / 4k Hz）±3 / ±6 dB，两倍频程带（中心 150 / 600 / 4 800 Hz）±4 / ±6 dB，中心频率同 T12–T14。
- **三种量法**（`measures.py`）：
  - `code_*`：照 `perceptual.rs` `v_timbre`（main 90ed08f2）移植；
  - `doc_*`：照 N4 原文；
  - `erb_*`：调研 026 推荐的候选。
  另有 `third_*`（1/3 倍频程）作参照。

## 怎么复现

```sh
# 主目录下；venv 用主目录 _tmp/exp002-venv（numpy · scipy · soundfile · pandas）
python -I docs/experiments/exp017-timbre-measure/run.py _tmp/exp013/20261103230756-17f5 _tmp/026/<run-id>
python -I docs/experiments/exp017-timbre-measure/summarize.py _tmp/026/<run-id>/readings.csv
# 旁证：GTCRN 真实输出（exp014 v2 run 给识别服务存的 16 位副本）
python -I docs/experiments/exp017-timbre-measure/realout.py _tmp/exp013/20261103230756-17f5 _tmp/scorecard/20261105-005916-4ccf0638/qa/asr-wav <out.csv>
```

## 结果

- 正式一轮：`runs/20261105-094833/`，30 段 × 66 种，`summary.md` 是中位数与 95 分位表。比 `094442` 只多一列 `erb_std_sym`（两边对称扣停顿能量），其余列逐字相同。
- 过程轮（留作过程）：`091900`、`092549`、`092828`、`093858`、`094442`。`092828` 及以前的 ERB 有两处错：频带只到约 7.2 kHz；先截下限再去均值，滤波后改音量会改读数。都由 codex 第 1 轮查出，已改。
- 旁证 `realout-20261105` 已用改后的 ERB 重算。
- 旁证：`runs/realout-20261105/gtcrn-noisy.csv`，480 段。
- 判据与解读见 `docs/research/026_timbre-measure/decision.md` ②。

## 结论的边界

- 劣化都是线性滤波或加噪。降噪器的非线性损伤（逐帧忽深忽浅的压制）没有模拟。
- 只有 exp013 一个说话人集合，16 kHz。
- 代码版是 Python 移植。与 Rust 的逐位对拍在 `src/eval/tests/timbre_parity.rs`，要名额编译，**尚未跑**。在那之前，「代码读数」这一列只算推断。
- 音频在主目录 `_tmp/026/<run-id>/`，不进仓；仓里只有读数与哈希。
