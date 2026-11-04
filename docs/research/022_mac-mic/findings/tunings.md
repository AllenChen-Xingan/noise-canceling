# 本机系统调音文件：内置麦标准模式的处理链（macmic 亲手核，2026-11-03）

> 用途：回答「第三方经普通采集（HAL）拿到的内置麦信号，系统已经做了什么」。这是本线证据最硬的一份。
> 做法：只读本机 `/System/Library/Audio/Tunings/`（45 MB，系统自带，纯文本 `.dspg` + plist）和 `ioreg`。没开麦、没录音、没改任何设置。
> ⚠️ 限定（codex 第 1 轮 #1）：本机运行时用的是哪一份调音（AIDnn）**未查到**；下文以 AID40 为例的处理链是「候选」，运行时选的图、`micSense` 与波束器运行时是否自适应，都要第 `selftest.md` 第 2 步确认。
> 来源标记：【读系统文件】= 本机 macOS 自带文件的原文；【推断】= 我据此推出的。样本：1 台机（本机 Mac17,3）+ 系统里 34 份机型调音。
> 起因：sim 路怀疑派指出这个目录（`findings/sim-skeptic.md` 末尾第 1 条），我逐条复核并补全。

## 一、本机身份

| 项 | 值 | 出处 |
|---|---|---|
| 机型 | MacBook Air，Apple M5，`Mac17,3`，板号 `J813`，型号 A3449 | `system_profiler SPHardwareDataType`；`ioreg -rd1 -c IOPlatformExpertDevice`【读系统文件】 |
| 内核给 coreaudiod 的内置麦 | 设备名「Digital Mic」，`MaxInputChannelCount = 3`；流格式 4 声道 16 位、48 kHz（第 4 路用途未知） | `ioreg -l` 中 `audio-hp-mic` 节点【读系统文件】 |
| 低功耗麦（语音唤醒用） | `lp-mic-device`：3 声道、16 kHz、24 位 | 同上【读系统文件】 |
| 本机用的是哪一份调音（AIDnn） | **未查到**。`lsof` / `vmmap` 看不到 coreaudiod（要管理员权限），系统文件里也没有机型→编号的对照 | — |

**结论 1**：硬件到系统是 3 路，到 app 是 1 路。3 合 1 发生在 coreaudiod 的处理图里（下节），不在硬件里。【读系统文件】这与 Triforce 作者「Apple 在用户态做波束成形」的说法一致（`findings/apple.md` 第五节 #6）。

## 二、标准模式的处理图（以 AID40 为例）

`AID40/DSP/DSPDatabase.plist` 里「Built-in Microphone」各模式对应的图：

| 模式 | 用的图 | 说明 |
|---|---|---|
| General · Telephony · Measurement · Movie | `builtin_microphone_general.dspg` | 3 进 1 出。**推断普通 HAL 采集走这条**【推断：General 是默认模式名；运行时选图未见证据】。Measurement 也指向它 |
| Voice | `builtin_microphone_voice.dspg` | 3 进 1 出，只有增益、求和、均衡。**推断**：开 VPIO 时系统用它，降噪由 VPIO 自己做 |
| VoiceRaw | `builtin_microphone_voice_raw.dspg` | 3 进 3 出，只有增益 |
| Dictation（听写） | `builtin_microphone_dictation.dspg` | 多麦对齐 → 去混响（`drev`，注释 MCLP）→ 多通道维纳滤波（`dnos`，注释 PMWF）→ 波束 → 神经降噪（`nnsp`） |
| Diagnostic | `builtin_microphone_raw.dspg` | 3 进 3 出，什么都不做 |

`builtin_microphone_general.dspg` 原文的处理顺序（AID40，2164 字节）：

```
in(3 路, 48 kHz)
 → dcBlocker (aufx dccb)          直流隔离
 → 每路增益 dbgn_Mic1..3
 → micSense (aufx sens)           原文注释「; Noise Reduction」
 → preDSPGain
 → FFT（块长 480 = 10 ms）
 → BF (aufx bmfd)                 固定波束成形，3 进 1 出
 → 逆 FFT
 → ToneMeister (aufx tmst)        原文注释「; EQ」
 → userGain（系统设置里的输入音量）
 → out(1 路)
```

- `micSense` 的参数（`builtin_microphone_general_aufx_sens.plist`）：3 路各一条 `NoiseProfile`，每条 481 点（0–24 kHz，每点 50 Hz）。【读系统文件】**推断**：这是按每支麦的固定底噪谱做的平稳降噪，主要去麦自噪声（嘶声），不是跟着环境变的降噪。
- 波束器参数（`builtin_microphone_general_aufx_bmfd.plist`）：1 个波束，`Azimuth 0`、`Elevation 90`，每支麦 481 点复数系数，外加一条 481 点 `EQSelection`。【读系统文件】文件里只有一组静态系数。**推断**波束是固定的；但静态初值不能排除运行时自适应更新，Triforce 作者说是自适应。两说并存，可信度低，待实测（如 `selftest.md` 第 5 步换角度看指向图是否随声场变）。
- 输入音量范围（`Config/AID40.plist`）：内置麦 `InputVolumeInitial 0`、`Min −12`、`Max 12`（推断单位 dB）。【读系统文件】所以系统设置里的输入音量是这条图末端的数字增益，±12 dB。

## 三、34 份机型调音的对比

用 `tools/read_tunings.py`（本目录，只读）解码各 AID 的 `general` 图与波束参数。下表把系数完全相同的 AID 归成一组。

| 组 | AID | 标准模式里额外的降噪 | 波束器对低频自噪声的放大（200 / 500 / 1000 Hz） | 推断是什么机器 |
|---|---|---|---|---|
| A | 8, 14, 24, 28, 30, 34, 39, 40 | 无 | +9.8 / +9.2 / +6.7 dB | **可能是无风扇的各代 Air**（依据只有两条：标准模式没有风扇降噪 `ocns`；8 份与 Air 的机型数目相近，M1 至 M5 的 13 / 15 寸共 9 款）【推断，可信度低】 |
| B | 9, 15, 29, 35, 41 | `internalNoiseSuppressor` + `ambientNoiseSuppressor`（`ocns`） | +9.8 / +9.2 / +6.4 dB | 有风扇的同代机型（如 MacBook Pro 13/14）【推断】 |
| C–I | 3, 18 · 13, 20, 37, 42 · 21, 27, 38, 43 · 5, 7 · 22, 23 · 4 · 31 | 部分有 `ocns` | −4.8 到 +5 dB 不等 | MacBook Pro 14/16、iMac、其他【推断】 |
| J | 44 | 2 麦，含 `nnet` | — | 2 麦机型【推断】 |

- 「对低频自噪声的放大」= 3 支麦系数的范数 ‖w‖（20·log10）。A 组在 200 Hz 是 +9.8 dB：波束器为了在低频取得方向性，把麦自噪声放大约 10 倍功率。【读系统文件 + 推断】这解释了为什么 `micSense` 放在波束器**之前**：先压每支麦的嘶声，再做波束。
- A 组对「三麦收到完全相同信号」的绝对增益 |Σw|：200 / 500 / 1000 Hz 为 −1.9 / −6.3 / −9.6 dB；相对等权求和的白噪声增益为 −16.4 / −20.3 / −21.0 dB（三个量的定义见 `tools/read_tunings.py` 的 `beam_stats`）。【读系统文件 + 计算】含义：正上方（与阵列平面垂直）来的声音会被压低几 dB，波束器属于超指向设计。**不能**把它当作扩散噪声的衰减量：扩散噪声在各麦之间并不同相【推断】。（第 1 轮把相对值 −20 dB 写成了「同相增益」，已改，codex #4）
- `EQSelection` 在 A 组是 1 kHz 以上约 −4.5 dB 的平缓下凹、100 Hz 处 −0.7 dB。它的确切作用（波束后的补偿还是选择权重）未查到，不直接当频响用。
- `ToneMeister` 均衡的参数在二进制 `.austrip` 里，**没解出来**。所以标准模式的最终频响仍要靠 DXOMARK 曲线（`findings/measure.md` 表 2）或实测。

## 四、对本项目的意义

1. 我们用 cpal 走 HAL，**很可能**拿到的是 `general` 图的输出：已做「每麦底噪谱降噪 + 波束成形 + 均衡 + 数字增益」，图里没有回声消除单元，A 组的图里也没有风扇 / 环境降噪单元。【读系统文件 + 推断】本机用哪组、运行时是否自适应，待 `selftest.md` 第 2 步确认。
2. 「第三方拿不到原始三路」不完全对：系统里有 `Diagnostic`（raw，3 出）和 `VoiceRaw` 图。普通 app 能否选到这些格式未查到，要在本机只读列格式（`selftest.md` 第 1 步）。
3. 模拟「像 MacBook Air 内置麦录的」时，波束器那一步可以**直接用苹果的系数**：给定三麦几何（Asahi 的 2 cm 等边三角形，可信度低），就能算出真实的指向图。没有几何就只能用 DXOMARK 指向数据。

## 复现

```
sysctl -n hw.model
ioreg -l -w400 | grep -n -E 'MaxInputChannelCount|"input stream description"'
cat /System/Library/Audio/Tunings/AID40/DSP/Graphs/builtin_microphone_general.dspg
python3 -I docs/research/022_mac-mic/tools/read_tunings.py
```
