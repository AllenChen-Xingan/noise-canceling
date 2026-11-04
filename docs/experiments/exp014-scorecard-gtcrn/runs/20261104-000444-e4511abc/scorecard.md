# 场景成绩单

共 180 个 场景 × 档 × 设备 × 系统 组合、1440 段：过 0 · 不过 180 · 待测 0。感知项全部 L0（只作约束，不当奖励）。

## 最差的场景

- **cafe · stress · external-mic · gtcrn** · 不过：否决项不过 V-SPLVL V-EAT-ASR · 不过的项 V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA
- **cafe · stress · mba-builtin · gtcrn** · 不过：否决项不过 V-SPLVL V-EAT-ASR · 不过的项 V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA
- **canteen · common · external-mic · gtcrn** · 不过：否决项不过 V-SPLVL V-EAT-ASR · 不过的项 V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA

## 每个场景

### home-quiet · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-quiet-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.165 dB | -0.248（home-quiet-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.217 百分点 | 2.174（home-quiet-c03-ext） | 1.000 | 不过 |
| V-ONSET |  | 8 / 2 / 0 | 2.084 dB | 4.833（home-quiet-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.039 比例 | 0.123（home-quiet-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.232 dB | 1.947（home-quiet-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.777 dB² | 3.964（home-quiet-c00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 3.000 个/分钟 | 10.000（home-quiet-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.018 Δ | -0.002（home-quiet-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.178 dB | 8.400（home-quiet-c08-ext） | 6.000 | 不过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.575 × | 5.950（home-quiet-c01-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 4 / 0 | -6.670 dB | -4.794（home-quiet-c00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.104 · Distill-MOS Δ +0.199 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.104 | -0.053（home-quiet-c08-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.199 | +0.148（home-quiet-c07-ext） |

送人耳：
- home-quiet-c08-ext：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.17
- home-quiet-c09-ext：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.26

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +0.725（有 3 段 0.725 · 没有 7 段 0.000）
- V-EAT-ASR · `keyboard`（event）：差 +0.362（有 6 段 0.362 · 没有 4 段 0.000）
- V-EAT-ASR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `keyboard`（event）：差 +1.420（有 6 段 2.652 · 没有 4 段 1.232）
- V-ONSET · `bed-room`（bed）：差 +0.909（有 7 段 2.357 · 没有 3 段 1.448）
- V-ONSET · `fan`（pattern）：差 +0.773（有 7 段 2.316 · 没有 3 段 1.543）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`（pattern）：差 +0.025（有 7 段 0.047 · 没有 3 段 0.022）
- V-SIM-GATE · `keyboard`（event）：差 +0.017（有 6 段 0.046 · 没有 4 段 0.029）
- V-SIM-GATE · `bed-room`（bed）：差 +0.017（有 7 段 0.044 · 没有 3 段 0.028）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +1.587（有 6 段 2.411 · 没有 4 段 0.824）
- V-PUMPV · `mouse`（event）：差 +0.127（有 3 段 1.866 · 没有 7 段 1.738）
- V-PUMPV · `hvac`（pattern）：差 +0.043（有 7 段 1.789 · 没有 3 段 1.747）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +3.179（有 3 段 3.403 · 没有 7 段 0.224）
- V-TIMBRE · `keyboard`（event）：差 +1.848（有 6 段 1.917 · 没有 4 段 0.069）
- V-TIMBRE · `hvac`（pattern）：差 +1.079（有 7 段 1.501 · 没有 3 段 0.422）
- V-TIMBRE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +1.692（有 3 段 2.760 · 没有 7 段 1.068）
- V-MUSIC · `fan`（pattern）：差 +0.583（有 7 段 1.750 · 没有 3 段 1.167）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +1.492（有 3 段 -5.626 · 没有 7 段 -7.117）
- V-NR · `keyboard`（event）：差 +0.394（有 6 段 -6.512 · 没有 4 段 -6.906）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（home-quiet-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.001（home-quiet-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-quiet-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.001 dB | 0.502（home-quiet-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.074 比例 | 0.112（home-quiet-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.897 dB | -0.215（home-quiet-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.073 dB² | 0.295（home-quiet-c08-ext） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（home-quiet-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（home-quiet-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.738 dB | 12.114（home-quiet-c08-ext） | 6.000 | 不过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（home-quiet-c00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（home-quiet-c00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（home-quiet-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（home-quiet-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `hvac`（pattern）：差 +0.018（有 7 段 0.079 · 没有 3 段 0.061）
- V-SIM-GATE · `fan`（pattern）：差 +0.014（有 7 段 0.078 · 没有 3 段 0.064）
- V-SIM-GATE · `bed-room`（bed）：差 +0.003（有 7 段 0.075 · 没有 3 段 0.072）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +4.101（有 3 段 4.609 · 没有 7 段 0.508）
- V-TIMBRE · `keyboard`（event）：差 +2.842（有 6 段 2.875 · 没有 4 段 0.033）
- V-TIMBRE · `hvac`（pattern）：差 +0.856（有 7 段 1.995 · 没有 3 段 1.139）
- V-TIMBRE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-quiet-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.046 dB | -0.075（home-quiet-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-quiet-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 1.022 dB | 3.111（home-quiet-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.018 比例 | 0.059（home-quiet-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.869 dB | 1.357（home-quiet-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 0.635 dB² | 2.412（home-quiet-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 5.500 个/分钟 | 30.000（home-quiet-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.011 Δ | -0.003（home-quiet-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.764 dB | 11.971（home-quiet-c08-ext） | 6.000 | 不过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.391 × | 4.094（home-quiet-c01-ext） | 2.000 | 不过 |
| V-NR |  | 4 / 6 / 0 | -3.522 dB | -0.341（home-quiet-c03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.111 · Distill-MOS Δ +0.131 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.111 | -0.054（home-quiet-c09-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.131 | +0.009（home-quiet-c00-ext） |

送人耳：
- home-quiet-c08-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.08
- home-quiet-c09-ext：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.18

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-room`（bed）：差 +1.027（有 7 段 1.330 · 没有 3 段 0.302）
- V-ONSET · `fan`（pattern）：差 +0.937（有 7 段 1.303 · 没有 3 段 0.365）
- V-ONSET · `keyboard`（event）：差 +0.556（有 6 段 1.244 · 没有 4 段 0.688）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`（bed）：差 +0.013（有 7 段 0.022 · 没有 3 段 0.008）
- V-SIM-GATE · `fan`（pattern）：差 +0.009（有 7 段 0.020 · 没有 3 段 0.011）
- V-SIM-GATE · `hvac`（pattern）：差 +0.009（有 7 段 0.020 · 没有 3 段 0.011）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`（bed）：差 +0.713（有 7 段 0.849 · 没有 3 段 0.136）
- V-PUMPV · `fan`（pattern）：差 +0.703（有 7 段 0.846 · 没有 3 段 0.143）
- V-PUMPV · `keyboard`（event）：差 +0.545（有 6 段 0.853 · 没有 4 段 0.309）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +3.997（有 3 段 4.562 · 没有 7 段 0.565）
- V-TIMBRE · `keyboard`（event）：差 +2.828（有 6 段 2.896 · 没有 4 段 0.067）
- V-TIMBRE · `hvac`（pattern）：差 +0.803（有 7 段 2.005 · 没有 3 段 1.202）
- V-TIMBRE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +0.914（有 3 段 2.031 · 没有 7 段 1.116）
- V-MUSIC · `fan`（pattern）：差 +0.465（有 7 段 1.530 · 没有 3 段 1.065）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `keyboard`（event）：差 +4.474（有 6 段 -1.733 · 没有 4 段 -6.207）
- V-NR · `mouse`（event）：差 +2.732（有 3 段 -1.610 · 没有 7 段 -4.342）
- V-NR · `bed-ace`（bed）：差 +1.164（有 3 段 -2.708 · 没有 7 段 -3.871）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-quiet-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.168 dB | -0.269（home-quiet-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.455 百分点 | 4.545（home-quiet-c05-mba） | 1.000 | 不过 |
| V-ONSET |  | 7 / 3 / 0 | 2.063 dB | 5.024（home-quiet-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.044 比例 | 0.104（home-quiet-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.448 dB | 1.625（home-quiet-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 2.024 dB² | 4.810（home-quiet-c00-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 8.500 个/分钟 | 25.000（home-quiet-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.024 Δ | 0.004（home-quiet-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.789 dB | 4.660（home-quiet-c08-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.868 × | 6.936（home-quiet-c01-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -7.831 dB | -5.735（home-quiet-c05-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.143 · Distill-MOS Δ +0.239 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.143 | -0.091（home-quiet-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.239 | +0.142（home-quiet-c02-mba） |

送人耳：
- home-quiet-c08-mba：一个说好一个说差：SCOREQ Δ -0.09 · Distill-MOS Δ +0.17

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `keyboard`（event）：差 +0.758（有 6 段 0.758 · 没有 4 段 0.000）
- V-EAT-ASR · `bed-room`（bed）：差 +0.649（有 7 段 0.649 · 没有 3 段 0.000）
- V-EAT-ASR · `fan`（pattern）：差 +0.649（有 7 段 0.649 · 没有 3 段 0.000）
- V-EAT-ASR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `keyboard`（event）：差 +1.687（有 6 段 2.738 · 没有 4 段 1.051）
- V-ONSET · `mouse`（event）：差 +0.664（有 3 段 2.527 · 没有 7 段 1.864）
- V-ONSET · `bed-room`（bed）：差 +0.305（有 7 段 2.154 · 没有 3 段 1.849）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `mouse`（event）：差 +0.030（有 3 段 0.065 · 没有 7 段 0.035）
- V-SIM-GATE · `hvac`（pattern）：差 +0.026（有 7 段 0.052 · 没有 3 段 0.026）
- V-SIM-GATE · `keyboard`（event）：差 +0.013（有 6 段 0.049 · 没有 4 段 0.036）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +1.969（有 6 段 2.812 · 没有 4 段 0.843）
- V-PUMPV · `mouse`（event）：差 +0.282（有 3 段 2.221 · 没有 7 段 1.940）
- V-PUMPV · `bed-ace`（bed）：差 +0.232（有 3 段 2.186 · 没有 7 段 1.955）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +1.816（有 3 段 3.139 · 没有 7 段 1.323）
- V-MUSIC · `fan`（pattern）：差 +0.416（有 7 段 1.992 · 没有 3 段 1.577）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `keyboard`（event）：差 +1.720（有 6 段 -7.143 · 没有 4 段 -8.863）
- V-NR · `bed-ace`（bed）：差 +0.905（有 3 段 -7.197 · 没有 7 段 -8.102）
- V-NR · `mouse`（event）：差 +0.107（有 3 段 -7.756 · 没有 7 段 -7.863）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（home-quiet-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.008（home-quiet-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-quiet-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.027 dB | 0.067（home-quiet-c03-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.076 比例 | 0.129（home-quiet-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.289 dB | -0.670（home-quiet-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.059 dB² | 0.145（home-quiet-c02-mba） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（home-quiet-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（home-quiet-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.420 dB | 8.769（home-quiet-c08-mba） | 6.000 | 不过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（home-quiet-c00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（home-quiet-c00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（home-quiet-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（home-quiet-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`（bed）：差 +0.018（有 3 段 0.089 · 没有 7 段 0.070）
- V-SIM-GATE · `hvac`（pattern）：差 +0.015（有 7 段 0.080 · 没有 3 段 0.065）
- V-SIM-GATE · `fan`（pattern）：差 +0.001（有 7 段 0.076 · 没有 3 段 0.075）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +3.144（有 3 段 3.621 · 没有 7 段 0.477）
- V-TIMBRE · `keyboard`（event）：差 +2.311（有 6 段 2.345 · 没有 4 段 0.034）
- V-TIMBRE · `hvac`（pattern）：差 +1.181（有 7 段 1.775 · 没有 3 段 0.594）
- V-TIMBRE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-quiet-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.066 dB | -0.101（home-quiet-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | 0.327 百分点 | 4.545（home-quiet-c05-mba） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 0.962 dB | 1.970（home-quiet-c02-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.016 比例 | 0.041（home-quiet-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.868 dB | 1.606（home-quiet-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 0.587 dB² | 1.970（home-quiet-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 8.500 个/分钟 | 30.000（home-quiet-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.016 Δ | -0.002（home-quiet-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.500 dB | 8.626（home-quiet-c08-mba） | 6.000 | 不过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.664 × | 4.684（home-quiet-c01-mba） | 2.000 | 不过 |
| V-NR |  | 3 / 7 / 0 | -3.540 dB | -0.425（home-quiet-c05-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.161 · Distill-MOS Δ +0.191 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.161 | -0.006（home-quiet-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.191 | +0.083（home-quiet-c00-mba） |

送人耳：
- home-quiet-c08-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.11

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +0.568（有 3 段 0.725 · 没有 7 段 0.157）
- V-EAT-ASR · `keyboard`（event）：差 +0.545（有 6 段 0.545 · 没有 4 段 0.000）
- V-EAT-ASR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`（bed）：差 +0.009（有 7 段 0.019 · 没有 3 段 0.010）
- V-SIM-GATE · `hvac`（pattern）：差 +0.007（有 7 段 0.018 · 没有 3 段 0.012）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`（bed）：差 +0.329（有 7 段 0.685 · 没有 3 段 0.356）
- V-PUMPV · `fan`（pattern）：差 +0.217（有 7 段 0.652 · 没有 3 段 0.435）
- V-PUMPV · `keyboard`（event）：差 +0.201（有 6 段 0.667 · 没有 4 段 0.466）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +3.157（有 3 段 3.710 · 没有 7 段 0.553）
- V-TIMBRE · `keyboard`（event）：差 +2.378（有 6 段 2.451 · 没有 4 段 0.073）
- V-TIMBRE · `hvac`（pattern）：差 +1.169（有 7 段 1.851 · 没有 3 段 0.681）
- V-TIMBRE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +0.794（有 3 段 2.220 · 没有 7 段 1.426）
- V-MUSIC · `fan`（pattern）：差 +0.078（有 7 段 1.687 · 没有 3 段 1.609）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `keyboard`（event）：差 +5.971（有 6 段 -1.152 · 没有 4 段 -7.123）
- V-NR · `mouse`（event）：差 +2.465（有 3 段 -1.815 · 没有 7 段 -4.280）
- V-NR · `bed-ace`（bed）：差 +0.624（有 3 段 -3.104 · 没有 7 段 -3.727）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-quiet-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.740 dB | -1.343（home-quiet-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.538 百分点 | 3.226（home-quiet-s00-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 2.070 dB | 6.027（home-quiet-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.051 比例 | 0.085（home-quiet-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.322 dB | -1.289（home-quiet-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 4.344 dB² | 8.046（home-quiet-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 15.833 个/分钟 | 30.000（home-quiet-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.033 Δ | 0.014（home-quiet-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.665 dB | 5.163（home-quiet-s02-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.201 × | 2.324（home-quiet-s01-ext） | 2.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -6.546 dB | -4.057（home-quiet-s03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.179 · Distill-MOS Δ +0.205 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.179 | -0.065（home-quiet-s02-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.205 | +0.115（home-quiet-s05-ext） |

送人耳：
- home-quiet-s02-ext：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.18

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-room`（bed）：差 +1.075（有 3 段 1.075 · 没有 3 段 0.000）
- V-EAT-ASR · `fan`（pattern）：差 +1.075（有 3 段 1.075 · 没有 3 段 0.000）
- V-EAT-ASR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fridge-hum`（pattern）：差 +2.560（有 3 段 3.350 · 没有 3 段 0.790）
- V-ONSET · `fan`（pattern）：差 +1.620（有 3 段 2.880 · 没有 3 段 1.260）
- V-ONSET · `bed-ace`（bed）：差 +1.584（有 3 段 2.863 · 没有 3 段 1.278）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.041（有 3 段 0.072 · 没有 3 段 0.030）
- V-SIM-GATE · `fan`（pattern）：差 +0.008（有 3 段 0.055 · 没有 3 段 0.047）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.002（有 3 段 0.052 · 没有 3 段 0.050）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +4.274（有 3 段 6.481 · 没有 3 段 2.207）
- V-PUMPV · `fan`（pattern）：差 +0.502（有 3 段 4.595 · 没有 3 段 4.093）
- V-PUMPV · `bed-ace`（bed）：差 +0.349（有 3 段 4.519 · 没有 3 段 4.169）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +0.560（有 3 段 1.481 · 没有 3 段 0.921）
- V-MUSIC · `fridge-hum`（pattern）：差 +0.471（有 3 段 1.436 · 没有 3 段 0.965）
- V-MUSIC · `fan`（pattern）：差 +0.394（有 3 段 1.398 · 没有 3 段 1.004）
- V-MUSIC · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `hvac`（pattern）：差 +2.559（有 3 段 -5.266 · 没有 3 段 -7.825）
- V-NR · `bed-ace`（bed）：差 +0.404（有 3 段 -6.344 · 没有 3 段 -6.748）
- V-NR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（home-quiet-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（home-quiet-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（home-quiet-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | -0.253 dB | 0.307（home-quiet-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.250 比例 | 0.583（home-quiet-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -8.115 dB | -3.569（home-quiet-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 2 / 0 | 1.264 dB² | 3.004（home-quiet-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（home-quiet-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（home-quiet-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.809 dB | 9.256（home-quiet-s02-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（home-quiet-s00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（home-quiet-s00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（home-quiet-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（home-quiet-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `fan`（pattern）：差 +0.236（有 3 段 0.368 · 没有 3 段 0.132）
- V-SIM-GATE · `bed-room`（bed）：差 +0.078（有 3 段 0.289 · 没有 3 段 0.211）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +1.570（有 3 段 2.050 · 没有 3 段 0.479）
- V-PUMPV · `bed-room`（bed）：差 +0.394（有 3 段 1.461 · 没有 3 段 1.067）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +2.255（有 3 段 3.936 · 没有 3 段 1.682）
- V-TIMBRE · `fridge-hum`（pattern）：差 +1.114（有 3 段 3.366 · 没有 3 段 2.252）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-quiet-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.257 dB | -0.462（home-quiet-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -0.538 百分点 | 0.000（home-quiet-s01-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 1.327 dB | 4.277（home-quiet-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.030 比例 | 0.075（home-quiet-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.576 dB | -1.017（home-quiet-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 3.583 dB² | 8.012（home-quiet-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 28.333 个/分钟 | 35.000（home-quiet-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.020 Δ | -0.002（home-quiet-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.947 dB | 9.436（home-quiet-s02-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.171 × | 1.985（home-quiet-s01-ext） | 2.000 | 过 |
| V-NR |  | 1 / 5 / 0 | -2.409 dB | -0.922（home-quiet-s03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.144 · Distill-MOS Δ +0.109 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.144 | -0.042（home-quiet-s02-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.109 | +0.017（home-quiet-s03-ext） |

送人耳：
- home-quiet-s02-ext：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.04

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `fan`（pattern）：差 +1.619（有 3 段 2.137 · 没有 3 段 0.518）
- V-ONSET · `bed-ace`（bed）：差 +1.369（有 3 段 2.012 · 没有 3 段 0.643）
- V-ONSET · `fridge-hum`（pattern）：差 +1.244（有 3 段 1.949 · 没有 3 段 0.706）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fan`（pattern）：差 +0.045（有 3 段 0.052 · 没有 3 段 0.007）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.005（有 3 段 0.032 · 没有 3 段 0.027）
- V-SIM-GATE · `bed-room`（bed）：差 +0.003（有 3 段 0.031 · 没有 3 段 0.028）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +3.288（有 3 段 5.227 · 没有 3 段 1.940）
- V-PUMPV · `bed-room`（bed）：差 +1.667（有 3 段 4.417 · 没有 3 段 2.750）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +2.474（有 3 段 4.184 · 没有 3 段 1.710）
- V-TIMBRE · `fridge-hum`（pattern）：差 +0.935（有 3 段 3.415 · 没有 3 段 2.480）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `hvac`（pattern）：差 +1.685（有 3 段 -1.567 · 没有 3 段 -3.252）
- V-NR · `bed-room`（bed）：差 +1.426（有 3 段 -1.696 · 没有 3 段 -3.122）
- V-NR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-quiet-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.818 dB | -1.349（home-quiet-s05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 1.262 百分点 | 4.348（home-quiet-s02-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 3 / 0 | 3.745 dB | 8.561（home-quiet-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.073 比例 | 0.128（home-quiet-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.874 dB | -1.400（home-quiet-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.666 dB² | 13.402（home-quiet-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 19.167 个/分钟 | 40.000（home-quiet-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.037 Δ | 0.026（home-quiet-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.778 dB | 6.420（home-quiet-s02-mba） | 6.000 | 不过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.243 × | 2.217（home-quiet-s01-mba） | 2.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -8.059 dB | -5.877（home-quiet-s03-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.232 · Distill-MOS Δ +0.244 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.232 | -0.041（home-quiet-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.244 | +0.156（home-quiet-s05-mba） |

送人耳：
- home-quiet-s02-mba：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.23

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-room`（bed）：差 +2.525（有 3 段 2.525 · 没有 3 段 0.000）
- V-EAT-ASR · `fridge-hum`（pattern）：差 +0.374（有 3 段 1.449 · 没有 3 段 1.075）
- V-EAT-ASR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fan`（pattern）：差 +2.879（有 3 段 5.185 · 没有 3 段 2.306）
- V-ONSET · `fridge-hum`（pattern）：差 +2.810（有 3 段 5.150 · 没有 3 段 2.340）
- V-ONSET · `bed-ace`（bed）：差 +1.171（有 3 段 4.331 · 没有 3 段 3.160）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fan`（pattern）：差 +0.038（有 3 段 0.092 · 没有 3 段 0.054）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.036（有 3 段 0.091 · 没有 3 段 0.055）
- V-SIM-GATE · `bed-room`（bed）：差 +0.004（有 3 段 0.075 · 没有 3 段 0.071）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +6.305（有 3 段 9.818 · 没有 3 段 3.513）
- V-PUMPV · `bed-room`（bed）：差 +1.702（有 3 段 7.517 · 没有 3 段 5.815）
- V-PUMPV · `fan`（pattern）：差 +0.468（有 3 段 6.900 · 没有 3 段 6.432）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +2.513（有 3 段 3.034 · 没有 3 段 0.522）
- V-TIMBRE · `fridge-hum`（pattern）：差 +1.040（有 3 段 2.298 · 没有 3 段 1.258）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `fridge-hum`（pattern）：差 +0.596（有 3 段 1.541 · 没有 3 段 0.945）
- V-MUSIC · `bed-ace`（bed）：差 +0.552（有 3 段 1.519 · 没有 3 段 0.967）
- V-MUSIC · `fan`（pattern）：差 +0.288（有 3 段 1.387 · 没有 3 段 1.099）
- V-MUSIC · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `hvac`（pattern）：差 +1.185（有 3 段 -7.466 · 没有 3 段 -8.651）
- V-NR · `bed-ace`（bed）：差 +0.054（有 3 段 -8.032 · 没有 3 段 -8.086）
- V-NR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（home-quiet-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（home-quiet-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（home-quiet-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.022 dB | 0.865（home-quiet-s01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.290 比例 | 0.584（home-quiet-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.172 dB | -4.430（home-quiet-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 3 / 3 / 0 | 2.560 dB² | 10.469（home-quiet-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（home-quiet-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（home-quiet-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.444 dB | 9.820（home-quiet-s02-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（home-quiet-s00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（home-quiet-s00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（home-quiet-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（home-quiet-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `fan`（pattern）：差 +0.242（有 3 段 0.411 · 没有 3 段 0.169）
- V-SIM-GATE · `bed-room`（bed）：差 +0.031（有 3 段 0.305 · 没有 3 段 0.274）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +3.593（有 3 段 4.356 · 没有 3 段 0.763）
- V-PUMPV · `bed-room`（bed）：差 +3.347（有 3 段 4.233 · 没有 3 段 0.886）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +2.603（有 3 段 3.745 · 没有 3 段 1.142）
- V-TIMBRE · `fridge-hum`（pattern）：差 +2.157（有 3 段 3.522 · 没有 3 段 1.365）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-quiet-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.334 dB | -0.490（home-quiet-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.538 百分点 | 3.226（home-quiet-s00-mba） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 1.888 dB | 5.822（home-quiet-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.040 比例 | 0.131（home-quiet-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -5.060 dB | -0.893（home-quiet-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.102 dB² | 15.291（home-quiet-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 38.333 个/分钟 | 75.000（home-quiet-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.023 Δ | 0.016（home-quiet-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.507 dB | 9.988（home-quiet-s02-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.140 × | 1.937（home-quiet-s01-mba） | 2.000 | 过 |
| V-NR |  | 1 / 5 / 0 | -2.591 dB | -1.364（home-quiet-s02-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.225 · Distill-MOS Δ +0.158 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.225 | +0.040（home-quiet-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.158 | +0.088（home-quiet-s03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-room`（bed）：差 +1.075（有 3 段 1.075 · 没有 3 段 0.000）
- V-EAT-ASR · `fan`（pattern）：差 +1.075（有 3 段 1.075 · 没有 3 段 0.000）
- V-EAT-ASR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fridge-hum`（pattern）：差 +1.849（有 3 段 2.813 · 没有 3 段 0.963）
- V-ONSET · `fan`（pattern）：差 +1.785（有 3 段 2.781 · 没有 3 段 0.996）
- V-ONSET · `bed-ace`（bed）：差 +1.444（有 3 段 2.610 · 没有 3 段 1.166）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fan`（pattern）：差 +0.050（有 3 段 0.065 · 没有 3 段 0.015）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.026（有 3 段 0.053 · 没有 3 段 0.026）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.025（有 3 段 0.052 · 没有 3 段 0.027）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +5.914（有 3 段 8.059 · 没有 3 段 2.145）
- V-PUMPV · `bed-room`（bed）：差 +3.440（有 3 段 6.822 · 没有 3 段 3.382）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +2.645（有 3 段 3.829 · 没有 3 段 1.184）
- V-TIMBRE · `fridge-hum`（pattern）：差 +2.114（有 3 段 3.564 · 没有 3 段 1.450）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`（bed）：差 +1.821（有 3 段 -1.681 · 没有 3 段 -3.502）
- V-NR · `hvac`（pattern）：差 +1.752（有 3 段 -1.716 · 没有 3 段 -3.467）
- V-NR · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-family · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-family-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 5 | -0.554 dB | -2.257（home-family-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-family-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 1.036 dB | 2.124（home-family-c08-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.018 比例 | 0.041（home-family-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.105 dB | -1.872（home-family-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 7.886 dB² | 46.462（home-family-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 54.000 个/分钟 | 110.000（home-family-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.016 Δ | -0.004（home-family-c08-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 0.905 dB | 2.688（home-family-c07-ext） | 6.000 | 过 |
| V-NR |  | 0 / 2 / 8 | -2.035 dB | -0.936（home-family-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -1.330 dB | -0.360（home-family-c03-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.955 比例 | 1.000（home-family-c03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 8 | 2.822 dB | 4.892（home-family-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.157 · Distill-MOS Δ +0.204 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.157 | +0.091（home-family-c03-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.204 | +0.152（home-family-c07-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `doorbell`（event）：差 +12.360（有 3 段 16.538 · 没有 7 段 4.178）
- V-PUMPV · `dishes`（event）：差 +1.847（有 3 段 9.179 · 没有 7 段 7.332）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（home-family-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 5 | -0.001 dB | -0.003（home-family-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-family-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.260 dB | 1.265（home-family-c04-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.138 比例 | 0.226（home-family-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -6.842 dB | -4.154（home-family-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 7 / 3 / 0 | 4.976 dB² | 25.166（home-family-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（home-family-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（home-family-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 1.303 dB | 2.250（home-family-c05-ext） | 6.000 | 过 |
| V-NR |  | 0 / 2 / 8 | 0.000 dB | 0.000（home-family-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | 0.000 dB | 0.000（home-family-c00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.727 比例 | 0.960（home-family-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 8 | 0.405 dB | 0.576（home-family-c03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（home-family-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（home-family-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `doorbell`（event）：差 +5.220（有 3 段 8.630 · 没有 7 段 3.410）
- V-PUMPV · `dishes`（event）：差 +4.011（有 3 段 7.784 · 没有 7 段 3.773）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-family-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 5 | -0.010 dB | -0.025（home-family-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-family-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 1.057 dB | 2.250（home-family-c06-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.031 比例 | 0.090（home-family-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.180 dB | -2.089（home-family-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 4.011 dB² | 25.072（home-family-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 36.500 个/分钟 | 75.000（home-family-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | -0.000 Δ | -0.004（home-family-c08-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 0.824 dB | 2.061（home-family-c05-ext） | 6.000 | 过 |
| V-NR |  | 0 / 2 / 8 | -2.653 dB | -1.048（home-family-c05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -0.856 dB | -0.306（home-family-c04-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.935 比例 | 1.000（home-family-c03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 8 | 1.382 dB | 2.044（home-family-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.083 · Distill-MOS Δ +0.130 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.083 | +0.042（home-family-c07-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.130 | +0.086（home-family-c01-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `doorbell`（event）：差 +0.003（有 3 段 0.033 · 没有 7 段 0.031）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `doorbell`（event）：差 +7.032（有 3 段 8.934 · 没有 7 段 1.902）
- V-PUMPV · `dishes`（event）：差 +0.178（有 3 段 4.136 · 没有 7 段 3.958）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `dishes`（event）：差 +0.390（有 3 段 -0.596 · 没有 6 段 -0.986）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-family-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 6 | -0.523 dB | -1.594（home-family-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 4 / 0 | 2.068 百分点 | 8.000（home-family-c07-mba） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.017 dB | 3.116（home-family-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.010 比例 | 0.025（home-family-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.258 dB | -2.700（home-family-c08-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.347 dB² | 2.453（home-family-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 62.500 个/分钟 | 180.000（home-family-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.023 Δ | 0.013（home-family-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 2.218 dB | 5.008（home-family-c07-mba） | 6.000 | 过 |
| V-NR |  | 1 / 2 / 7 | -6.047 dB | -4.646（home-family-c05-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -1.935 dB | -1.120（home-family-c08-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.884 比例 | 0.972（home-family-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 4 / 0 / 6 | 3.371 dB | 4.796（home-family-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.206 · Distill-MOS Δ +0.188 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.206 | +0.065（home-family-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.188 | +0.161（home-family-c09-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `dishes`（event）：差 +2.925（有 3 段 4.116 · 没有 7 段 1.190）
- V-EAT-ASR · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `doorbell`（event）：差 +0.004（有 3 段 0.013 · 没有 7 段 0.009）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `doorbell`（event）：差 +0.424（有 3 段 1.644 · 没有 7 段 1.219）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `dishes`（event）：差 +0.268（有 3 段 -1.756 · 没有 6 段 -2.024）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `dishes`（event）：差 +0.005（有 3 段 0.888 · 没有 6 段 0.883）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（home-family-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 6 | -0.002 dB | -0.005（home-family-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（home-family-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.159 dB | 1.209（home-family-c04-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.218 比例 | 0.324（home-family-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.605 dB | -6.137（home-family-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 0.330 dB² | 1.333（home-family-c07-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（home-family-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（home-family-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 6 | 2.532 dB | 6.202（home-family-c07-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 3 / 7 | 0.000 dB | 0.000（home-family-c03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | 0.000 dB | 0.000（home-family-c00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 8 / 1 | 0.527 比例 | 0.896（home-family-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 4 / 0 / 6 | 0.190 dB | 0.333（home-family-c09-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（home-family-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（home-family-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`（event）：差 +0.390（有 3 段 0.603 · 没有 7 段 0.213）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `dishes`（event）：差 +0.158（有 3 段 0.632 · 没有 6 段 0.474）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-family-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 6 | -0.063 dB | -0.089（home-family-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.476 百分点 | 0.000（home-family-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 0.874 dB | 3.518（home-family-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.023 比例 | 0.088（home-family-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.929 dB | -3.140（home-family-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 0.981 dB² | 2.009（home-family-c07-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 47.000 个/分钟 | 110.000（home-family-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.005 Δ | -0.001（home-family-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 2.079 dB | 4.150（home-family-c07-mba） | 6.000 | 过 |
| V-NR |  | 1 / 2 / 7 | -2.506 dB | -0.583（home-family-c05-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -0.868 dB | -0.318（home-family-c07-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.894 比例 | 1.000（home-family-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 4 / 0 / 6 | 1.368 dB | 3.972（home-family-c03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.179 · Distill-MOS Δ +0.135 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.179 | +0.043（home-family-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.135 | +0.096（home-family-c05-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `doorbell`（event）：差 +0.012（有 3 段 0.031 · 没有 7 段 0.019）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`（event）：差 +0.491（有 3 段 1.325 · 没有 7 段 0.834）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `dishes`（event）：差 +0.418（有 3 段 -0.590 · 没有 6 段 -1.008）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `dishes`（event）：差 +0.073（有 3 段 0.942 · 没有 6 段 0.869）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-family-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -1.465 dB | -2.013（home-family-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -4.146 百分点 | 0.000（home-family-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.778 dB | 3.136（home-family-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.064 比例 | 0.178（home-family-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.239 dB | -4.867（home-family-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 16.252 dB² | 27.639（home-family-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 41.667 个/分钟 | 65.000（home-family-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.013 Δ | -0.005（home-family-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 2 / 2 | 4.216 dB | 6.398（home-family-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 2 / 4 | -3.847 dB | -2.882（home-family-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -2.069 dB | -0.616（home-family-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 1.000 比例 | 1.000（home-family-s00-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 1.519 dB | 2.803（home-family-s03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.230 · Distill-MOS Δ +0.224 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.230 | +0.119（home-family-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.224 | +0.124（home-family-s01-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `dishes`（event）：差 +1.200（有 3 段 1.378 · 没有 3 段 0.178）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.051（有 3 段 0.089 · 没有 3 段 0.038）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +9.158（有 3 段 20.831 · 没有 3 段 11.672）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（home-family-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | 0.000 dB | 0.000（home-family-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（home-family-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.222 dB | 4.190（home-family-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.492 比例 | 0.768（home-family-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -16.191 dB | -9.207（home-family-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.477 dB² | 17.402（home-family-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（home-family-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（home-family-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 2 / 2 | 5.533 dB | 10.563（home-family-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 2 / 4 | 0.000 dB | 0.000（home-family-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | 0.000 dB | 0.000（home-family-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 4 / 1 | 0.516 比例 | 0.841（home-family-s00-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 0.071 dB | 0.093（home-family-s02-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（home-family-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（home-family-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `dishes`（event）：差 +1.963（有 3 段 1.204 · 没有 3 段 -0.759）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.246（有 3 段 0.615 · 没有 3 段 0.369）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.562（有 3 段 9.258 · 没有 3 段 7.696）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-family-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -0.235 dB | -0.715（home-family-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -1.765 百分点 | 0.000（home-family-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 1.318 dB | 3.758（home-family-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.184 比例 | 0.492（home-family-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.736 dB | -7.251（home-family-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.685 dB² | 30.028（home-family-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 28.333 个/分钟 | 50.000（home-family-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | -0.004 Δ | -0.011（home-family-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 2 | 4.539 dB | 10.198（home-family-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 2 / 4 | -2.167 dB | -1.054（home-family-s03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -0.753 dB | -0.171（home-family-s04-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 0.934 比例 | 1.000（home-family-s01-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 0.877 dB | 0.972（home-family-s03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.047 · Distill-MOS Δ +0.128 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.047 | -0.010（home-family-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.128 | +0.058（home-family-s01-ext） |

送人耳：
- home-family-s05-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.18

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `dishes`（event）：差 +0.467（有 3 段 1.551 · 没有 3 段 1.084）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.149（有 3 段 0.258 · 没有 3 段 0.109）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +4.222（有 3 段 17.796 · 没有 3 段 13.573）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-family-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 2 | -1.654 dB | -2.673（home-family-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.616 百分点 | 7.143（home-family-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 2.604 dB | 6.091（home-family-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 0 / 0 | 0.025 比例 | 0.036（home-family-s05-mba） | 0.050 | 过 |
| V-WEAK |  | 6 / 0 / 0 | -14.091 dB | -8.241（home-family-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.292 dB² | 34.793（home-family-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 43.333 个/分钟 | 100.000（home-family-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.011 Δ | -0.006（home-family-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 2 / 2 | 5.346 dB | 7.549（home-family-s02-mba） | 6.000 | 不过 |
| V-NR |  | 1 / 1 / 4 | -6.895 dB | -5.499（home-family-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -2.428 dB | -1.244（home-family-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 0.981 比例 | 1.000（home-family-s01-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 0.765 dB | 1.178（home-family-s03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.125 · Distill-MOS Δ +0.204 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.125 | +0.057（home-family-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.204 | +0.140（home-family-s01-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +1.232（有 3 段 1.232 · 没有 3 段 0.000）
- V-EAT-ASR · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.605（有 3 段 2.907 · 没有 3 段 2.302）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +5.238（有 3 段 15.911 · 没有 3 段 10.673）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（home-family-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 2 | -0.010 dB | -0.041（home-family-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（home-family-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.970 dB | 3.670（home-family-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.609 比例 | 0.861（home-family-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -18.540 dB | -13.254（home-family-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 4.137 dB² | 9.356（home-family-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（home-family-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（home-family-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 2 | 4.932 dB | 9.488（home-family-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 2 / 4 | 0.000 dB | 0.000（home-family-s02-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | 0.000 dB | 0.000（home-family-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 4 / 1 | 0.413 比例 | 0.696（home-family-s02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 0.033 dB | 0.037（home-family-s02-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（home-family-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（home-family-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +0.023（有 3 段 0.981 · 没有 3 段 0.959）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.120（有 3 段 0.669 · 没有 3 段 0.550）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +2.490（有 3 段 5.382 · 没有 3 段 2.892）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-family-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 2 | -0.489 dB | -0.909（home-family-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.595 百分点 | 3.571（home-family-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.627 dB | 6.081（home-family-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.149 比例 | 0.374（home-family-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.145 dB | -9.677（home-family-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.084 dB² | 36.104（home-family-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 38.333 个/分钟 | 60.000（home-family-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | -0.001 Δ | -0.010（home-family-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 2 | 4.304 dB | 8.642（home-family-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 2 / 4 | -2.100 dB | -1.406（home-family-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -0.749 dB | -0.210（home-family-s04-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 0.917 比例 | 0.991（home-family-s01-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 0.691 dB | 0.908（home-family-s03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.097 · Distill-MOS Δ +0.176 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.097 | +0.022（home-family-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.176 | +0.089（home-family-s03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +1.190（有 3 段 1.190 · 没有 3 段 0.000）
- V-EAT-ASR · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`（event）：差 +0.057（有 3 段 2.656 · 没有 3 段 2.598）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.100（有 3 段 0.199 · 没有 3 段 0.099）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +7.865（有 3 段 19.017 · 没有 3 段 11.152）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `baby-cry`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-kitchen`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### open-office · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（open-office-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.571 dB | -1.418（open-office-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.400 百分点 | 4.000（open-office-c00-ext） | 1.000 | 不过 |
| V-ONSET |  | 7 / 1 / 2 | 0.947 dB | 3.543（open-office-c05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.013 比例 | 0.067（open-office-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.749 dB | 0.944（open-office-c01-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 2.281 dB² | 9.939（open-office-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 16.500 个/分钟 | 40.000（open-office-c03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.019 Δ | -0.002（open-office-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 2.510 dB | 9.699（open-office-c08-ext） | 6.000 | 不过 |
| V-NR |  | 3 / 7 / 0 | -4.460 dB | -0.925（open-office-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -2.783 dB | -0.664（open-office-c06-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 6 | 0.843 比例 | 1.000（open-office-c07-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.896 dB | 5.615（open-office-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.088 · Distill-MOS Δ +0.174 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.088 | +0.004（open-office-c06-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.174 | +0.086（open-office-c06-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +0.667（有 6 段 0.667 · 没有 4 段 0.000）
- V-EAT-ASR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `notify`（event）：差 +1.080（有 3 段 1.622 · 没有 5 段 0.542）
- V-ONSET · `mouse`（event）：差 +0.993（有 3 段 1.568 · 没有 5 段 0.575）
- V-ONSET · `bed-ace`（bed）：差 +0.527（有 5 段 1.145 · 没有 3 段 0.618）
- V-ONSET · `hvac`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-SIM-GATE · `mouse`（event）：差 +0.023（有 4 段 0.027 · 没有 6 段 0.004）
- V-SIM-GATE · `notify`（event）：差 +0.021（有 3 段 0.028 · 没有 7 段 0.007）
- V-SIM-GATE · `keyboard`（event）：差 +0.017（有 4 段 0.023 · 没有 6 段 0.007）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +3.098（有 4 段 4.140 · 没有 6 段 1.042）
- V-PUMPV · `bed-office`（bed）：差 +1.162（有 4 段 2.979 · 没有 6 段 1.816）
- V-PUMPV · `notify`（event）：差 +0.237（有 3 段 2.447 · 没有 7 段 2.210）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `notify`（event）：差 +5.629（有 3 段 6.450 · 没有 7 段 0.821）
- V-TIMBRE · `mouse`（event）：差 +5.412（有 4 段 5.757 · 没有 6 段 0.345）
- V-TIMBRE · `bed-ace`（bed）：差 +3.938（有 6 段 4.085 · 没有 4 段 0.147）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-office`（bed）：差 +3.572（有 4 段 -2.317 · 没有 6 段 -5.889）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-office`：样本不够（有 2 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（open-office-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.007 dB | -0.033（open-office-c08-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（open-office-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 8 / 0 / 2 | -0.241 dB | 0.087（open-office-c02-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.107 比例 | 0.338（open-office-c07-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.469 dB | -0.030（open-office-c01-ext） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 1.468 dB² | 11.750（open-office-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（open-office-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（open-office-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.450 dB | 9.520（open-office-c08-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（open-office-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | 0.000 dB | 0.000（open-office-c00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 6 | 0.616 比例 | 1.000（open-office-c03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.847 dB | 4.273（open-office-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（open-office-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（open-office-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `mouse`（event）：差 +0.150（有 4 段 0.197 · 没有 6 段 0.047）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.145（有 6 段 0.165 · 没有 4 段 0.020）
- V-SIM-GATE · `notify`（event）：差 +0.041（有 3 段 0.136 · 没有 7 段 0.094）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +2.912（有 4 段 3.215 · 没有 6 段 0.303）
- V-PUMPV · `bed-office`（bed）：差 +2.709（有 4 段 3.093 · 没有 6 段 0.384）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `notify`（event）：差 +6.085（有 3 段 7.709 · 没有 7 段 1.624）
- V-TIMBRE · `bed-ace`（bed）：差 +5.670（有 6 段 5.717 · 没有 4 段 0.048）
- V-TIMBRE · `mouse`（event）：差 +5.040（有 4 段 6.474 · 没有 6 段 1.433）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-office`：样本不够（有 2 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（open-office-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.083 dB | -0.151（open-office-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（open-office-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 8 / 0 / 2 | 0.393 dB | 1.537（open-office-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 10 / 0 / 0 | 0.013 比例 | 0.044（open-office-c07-ext） | 0.050 | 过 |
| V-WEAK |  | 10 / 0 / 0 | -3.888 dB | 0.224（open-office-c01-ext） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 2.478 dB² | 19.049（open-office-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 9.500 个/分钟 | 25.000（open-office-c03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.004 Δ | -0.001（open-office-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.282 dB | 9.697（open-office-c08-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 9 / 0 | -2.639 dB | -0.600（open-office-c08-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -1.141 dB | -0.422（open-office-c00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 6 | 0.880 比例 | 1.000（open-office-c03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.583 dB | 5.894（open-office-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 1 / 1 | SCOREQ Δ +0.045 · Distill-MOS Δ +0.067 | | Δ ≥ 0 | 不过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.045 | -0.030（open-office-c09-ext） |
| V-NRMOS-DISTILL | 9 / 1 / 0 | +0.067 | -0.008（open-office-c09-ext） |

送人耳：
- open-office-c03-ext：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.05

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-PUMPV · `keyboard`（event）：差 +5.058（有 4 段 5.513 · 没有 6 段 0.455）
- V-PUMPV · `bed-office`（bed）：差 +4.193（有 4 段 4.994 · 没有 6 段 0.801）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `notify`（event）：差 +7.123（有 3 段 8.268 · 没有 7 段 1.145）
- V-TIMBRE · `bed-ace`（bed）：差 +5.368（有 6 段 5.429 · 没有 4 段 0.061）
- V-TIMBRE · `mouse`（event）：差 +4.049（有 4 段 5.712 · 没有 6 段 1.662）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +2.829（有 6 段 -1.507 · 没有 4 段 -4.336）
- V-NR · `notify`（event）：差 +2.669（有 3 段 -0.771 · 没有 7 段 -3.439）
- V-NR · `keyboard`（event）：差 +2.197（有 4 段 -1.320 · 没有 6 段 -3.518）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-office`：样本不够（有 2 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `bed-office`（bed）：差 +0.033（有 4 段 0.025 · 没有 6 段 0.058）
- V-NRMOS-SCOREQ · `keyboard`（event）：差 +0.010（有 4 段 0.039 · 没有 6 段 0.049）
- V-NRMOS-SCOREQ · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `bed-office`（bed）：差 +0.034（有 4 段 0.047 · 没有 6 段 0.081）
- V-NRMOS-DISTILL · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### open-office · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（open-office-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.721 dB | -1.715（open-office-c08-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.667 百分点 | 0.000（open-office-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 8 / 1 / 1 | 1.599 dB | 5.283（open-office-c05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.019 比例 | 0.066（open-office-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.231 dB | 0.253（open-office-c01-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.077 dB² | 6.920（open-office-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 26.000 个/分钟 | 50.000（open-office-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.027 Δ | 0.018（open-office-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.968 dB | 3.951（open-office-c08-mba） | 6.000 | 过 |
| V-NR |  | 8 / 2 / 0 | -7.341 dB | -4.327（open-office-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 6 / 4 | -3.353 dB | -1.086（open-office-c03-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 6 | 0.736 比例 | 0.919（open-office-c02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.219 dB | 7.934（open-office-c08-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.120 · Distill-MOS Δ +0.207 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.120 | +0.021（open-office-c03-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.207 | +0.143（open-office-c08-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `mouse`（event）：差 +1.616（有 3 段 2.676 · 没有 6 段 1.060）
- V-ONSET · `keyboard`（event）：差 +1.418（有 3 段 2.544 · 没有 6 段 1.126）
- V-ONSET · `notify`（event）：差 +1.315（有 3 段 2.476 · 没有 6 段 1.161）
- V-ONSET · `hvac`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-ONSET · `phone-ring`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `mouse`（event）：差 +0.015（有 4 段 0.028 · 没有 6 段 0.013）
- V-SIM-GATE · `keyboard`（event）：差 +0.011（有 4 段 0.025 · 没有 6 段 0.014）
- V-SIM-GATE · `notify`（event）：差 +0.009（有 3 段 0.025 · 没有 7 段 0.016）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `notify`（event）：差 +1.903（有 3 段 3.409 · 没有 7 段 1.506）
- V-PUMPV · `mouse`（event）：差 +1.628（有 4 段 3.054 · 没有 6 段 1.425）
- V-PUMPV · `keyboard`（event）：差 +1.591（有 4 段 3.031 · 没有 6 段 1.440）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +0.310（有 6 段 -7.217 · 没有 4 段 -7.527）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-office`（bed）：差 +0.445（有 3 段 -3.131 · 没有 3 段 -3.576）
- V-INTF · `hvac`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-INTF · `keyboard`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（open-office-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.007 dB | -0.030（open-office-c08-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（open-office-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 9 / 0 / 1 | -0.107 dB | 0.352（open-office-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.108 比例 | 0.228（open-office-c06-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -6.574 dB | -2.023（open-office-c01-mba） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.359 dB² | 0.718（open-office-c04-mba） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（open-office-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（open-office-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.337 dB | 9.595（open-office-c08-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（open-office-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 6 / 4 | 0.000 dB | 0.000（open-office-c00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 6 | 0.313 比例 | 0.500（open-office-c03-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.609 dB | 2.183（open-office-c00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（open-office-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（open-office-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `mouse`（event）：差 +0.060（有 4 段 0.144 · 没有 6 段 0.085）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.026（有 6 段 0.119 · 没有 4 段 0.093）
- V-SIM-GATE · `keyboard`（event）：差 +0.023（有 4 段 0.123 · 没有 6 段 0.099）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `notify`（event）：差 +5.887（有 3 段 7.458 · 没有 7 段 1.571）
- V-TIMBRE · `bed-ace`（bed）：差 +5.229（有 6 段 5.428 · 没有 4 段 0.199）
- V-TIMBRE · `mouse`（event）：差 +5.142（有 4 段 6.422 · 没有 6 段 1.280）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-INTF · `keyboard`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（open-office-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.105 dB | -0.169（open-office-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.667 百分点 | 6.667（open-office-c03-mba） | 1.000 | 不过 |
| V-ONSET |  | 9 / 0 / 1 | 0.903 dB | 2.207（open-office-c05-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 10 / 0 / 0 | 0.010 比例 | 0.033（open-office-c05-mba） | 0.050 | 过 |
| V-WEAK |  | 10 / 0 / 0 | -4.024 dB | 0.422（open-office-c01-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 0.997 dB² | 2.199（open-office-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 23.000 个/分钟 | 55.000（open-office-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.010 Δ | -0.001（open-office-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.041 dB | 9.741（open-office-c08-mba） | 6.000 | 不过 |
| V-NR |  | 1 / 9 / 0 | -3.014 dB | -0.564（open-office-c08-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 6 / 4 | -1.507 dB | -0.452（open-office-c00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 6 | 0.756 比例 | 0.978（open-office-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.528 dB | 3.762（open-office-c00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.097 · Distill-MOS Δ +0.136 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.097 | +0.005（open-office-c09-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.136 | +0.064（open-office-c08-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-office`（bed）：差 +1.667（有 4 段 1.667 · 没有 6 段 0.000）
- V-EAT-ASR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +0.884（有 4 段 1.527 · 没有 6 段 0.644）
- V-PUMPV · `mouse`（event）：差 +0.347（有 4 段 1.206 · 没有 6 段 0.859）
- V-PUMPV · `notify`（event）：差 +0.247（有 3 段 1.170 · 没有 7 段 0.923）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `notify`（event）：差 +7.105（有 3 段 8.015 · 没有 7 段 0.910）
- V-TIMBRE · `bed-ace`（bed）：差 +4.628（有 6 段 4.893 · 没有 4 段 0.264）
- V-TIMBRE · `mouse`（event）：差 +3.736（有 4 段 5.283 · 没有 6 段 1.547）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +4.077（有 6 段 -1.383 · 没有 4 段 -5.460）
- V-NR · `notify`（event）：差 +3.151（有 3 段 -0.809 · 没有 7 段 -3.959）
- V-NR · `mouse`（event）：差 +2.854（有 4 段 -1.302 · 没有 6 段 -4.156）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-ace`（bed）：差 +0.547（有 3 段 -1.234 · 没有 3 段 -1.780）
- V-INTF · `hvac`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-INTF · `keyboard`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（open-office-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.547 dB | -2.210（open-office-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | -3.362 百分点 | 3.226（open-office-s04-ext） | 1.000 | 不过 |
| V-ONSET |  | 2 / 2 / 2 | 1.850 dB | 3.812（open-office-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.021 比例 | 0.044（open-office-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.127 dB | -2.153（open-office-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 7.335 dB² | 15.964（open-office-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 40.000 个/分钟 | 55.000（open-office-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.019 Δ | 0.009（open-office-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 1 / 5 / 0 | 7.237 dB | 14.346（open-office-s02-ext） | 6.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -6.202 dB | -5.838（open-office-s03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -2.550 dB | -1.701（open-office-s02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.939 比例 | 1.000（open-office-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.980 dB | 6.112（open-office-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.108 · Distill-MOS Δ +0.231 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.108 | +0.059（open-office-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.231 | +0.103（open-office-s05-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `phone-ring`（event）：差 +5.975（有 3 段 -0.374 · 没有 3 段 -6.349）
- V-EAT-ASR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `phone-ring`（event）：差 +0.171（有 3 段 -6.116 · 没有 3 段 -6.287）
- V-NR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）

### open-office · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（open-office-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（open-office-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（open-office-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 4 / 0 / 2 | 0.740 dB | 2.407（open-office-s02-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.395 比例 | 0.560（open-office-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.758 dB | -4.421（open-office-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 6.116 dB² | 17.661（open-office-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（open-office-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（open-office-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 1 / 5 / 0 | 8.036 dB | 16.415（open-office-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（open-office-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（open-office-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.701 比例 | 0.885（open-office-s05-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.129 dB | 0.485（open-office-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（open-office-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（open-office-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `phone-ring`（event）：差 +0.129（有 3 段 0.459 · 没有 3 段 0.330）
- V-SIM-GATE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`（event）：差 +0.093（有 3 段 8.082 · 没有 3 段 7.990）
- V-TIMBRE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）

### open-office · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（open-office-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.301 dB | -0.531（open-office-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | -0.356 百分点 | 3.333（open-office-s01-ext） | 1.000 | 不过 |
| V-ONSET |  | 3 / 1 / 2 | 1.757 dB | 4.034（open-office-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.030 比例 | 0.079（open-office-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.772 dB | -2.561（open-office-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 10.392 dB² | 37.747（open-office-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 29.167 个/分钟 | 45.000（open-office-s03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.004（open-office-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 1 / 5 / 0 | 8.654 dB | 16.649（open-office-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.845 dB | -1.272（open-office-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -0.907 dB | -0.755（open-office-s02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.977 比例 | 1.000（open-office-s00-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.655 dB | 2.785（open-office-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.080 · Distill-MOS Δ +0.145 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.080 | +0.009（open-office-s03-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.145 | +0.045（open-office-s01-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.025（有 3 段 0.042 · 没有 3 段 0.017）
- V-SIM-GATE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `phone-ring`（event）：差 +2.248（有 3 段 -1.721 · 没有 3 段 -3.969）
- V-NR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）

### open-office · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（open-office-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.889 dB | -2.447（open-office-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 8.211 百分点 | 33.333（open-office-s02-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 1 / 2 | 2.721 dB | 4.021（open-office-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.032 比例 | 0.107（open-office-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.943 dB | -3.482（open-office-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.824 dB² | 11.791（open-office-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 32.500 个/分钟 | 40.000（open-office-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.019 Δ | 0.008（open-office-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.823 dB | 11.069（open-office-s02-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -8.009 dB | -6.780（open-office-s02-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -2.607 dB | -1.979（open-office-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.914 比例 | 1.000（open-office-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.832 dB | 5.991（open-office-s01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.179 · Distill-MOS Δ +0.270 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.179 | +0.075（open-office-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.270 | +0.181（open-office-s05-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.029（有 3 段 0.046 · 没有 3 段 0.018）
- V-SIM-GATE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）

### open-office · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（open-office-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（open-office-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（open-office-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 4 / 0 / 2 | 0.687 dB | 1.702（open-office-s02-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.479 比例 | 0.574（open-office-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -14.277 dB | -7.543（open-office-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.466 dB² | 12.724（open-office-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（open-office-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（open-office-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.233 dB | 12.863（open-office-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（open-office-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（open-office-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.483 比例 | 0.724（open-office-s02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.060 dB | 0.183（open-office-s01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（open-office-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（open-office-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `phone-ring`（event）：差 +0.024（有 3 段 0.491 · 没有 3 段 0.467）
- V-SIM-GATE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）

### open-office · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（open-office-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.591 dB | -1.013（open-office-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 5.775 百分点 | 33.333（open-office-s02-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 1 / 2 | 2.590 dB | 4.123（open-office-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.031 比例 | 0.058（open-office-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.497 dB | -3.546（open-office-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.799 dB² | 12.128（open-office-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 33.333 个/分钟 | 40.000（open-office-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.004（open-office-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.803 dB | 13.165（open-office-s02-mba） | 6.000 | 不过 |
| V-NR |  | 2 / 4 / 0 | -3.587 dB | -1.458（open-office-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -1.209 dB | -0.913（open-office-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.926 比例 | 1.000（open-office-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.342 dB | 4.022（open-office-s01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.163 · Distill-MOS Δ +0.235 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.163 | +0.067（open-office-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.235 | +0.139（open-office-s04-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.016（有 3 段 0.039 · 没有 3 段 0.023）
- V-SIM-GATE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`（event）：差 +1.162（有 3 段 7.380 · 没有 3 段 6.218）
- V-PUMPV · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `phone-ring`（event）：差 +3.217（有 3 段 -1.979 · 没有 3 段 -5.196）
- V-NR · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）

### phone-booth · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 43.667 ms | 43.667（phone-booth-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.186 dB | -0.385（phone-booth-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.312 百分点 | 3.125（phone-booth-c06-ext） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 0.761 dB | 3.001（phone-booth-c01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.021 比例 | 0.033（phone-booth-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.363 dB | 1.887（phone-booth-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.064 dB² | 2.780（phone-booth-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.002 Δ | -0.004（phone-booth-c05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.171 dB | 0.882（phone-booth-c00-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.719 × | 5.761（phone-booth-c07-ext） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.350 dB | -3.080（phone-booth-c03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.171 · Distill-MOS Δ +0.176 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.171 | +0.047（phone-booth-c04-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.176 | +0.070（phone-booth-c03-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `muffled-talker`（speech）：差 +0.521（有 6 段 0.521 · 没有 4 段 0.000）
- V-EAT-ASR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 27.667 ms | 27.667（phone-booth-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.007（phone-booth-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.016 dB | 0.144（phone-booth-c09-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.075 比例 | 0.131（phone-booth-c01-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.827 dB | -0.073（phone-booth-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.007 dB² | 0.021（phone-booth-c00-ext） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（phone-booth-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.009 dB | 0.020（phone-booth-c03-ext） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（phone-booth-c00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（phone-booth-c00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（phone-booth-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（phone-booth-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `muffled-talker`（speech）：差 +0.006（有 6 段 0.077 · 没有 4 段 0.071）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 43.667 ms | 43.667（phone-booth-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.038 dB | -0.092（phone-booth-c01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.747 dB | 1.506（phone-booth-c06-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.022 比例 | 0.048（phone-booth-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.007 dB | 1.527（phone-booth-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 0.339 dB² | 1.013（phone-booth-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.004 Δ | -0.003（phone-booth-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.073 dB | 0.282（phone-booth-c03-ext） | 6.000 | 过 |
| V-MUSIC |  | 2 / 8 / 0 | 3.247 × | 6.059（phone-booth-c00-ext） | 2.000 | 不过 |
| V-NR |  | 7 / 3 / 0 | -6.972 dB | -2.795（phone-booth-c03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.204 · Distill-MOS Δ +0.102 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.204 | +0.052（phone-booth-c02-ext） |
| V-NRMOS-DISTILL | 9 / 1 / 0 | +0.102 | -0.025（phone-booth-c04-ext） |

送人耳：
- phone-booth-c04-ext：一个说好一个说差：SCOREQ Δ +0.06 · Distill-MOS Δ -0.03

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `muffled-talker`（speech）：差 +0.173（有 6 段 0.408 · 没有 4 段 0.235）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `muffled-talker`（speech）：差 +0.673（有 6 段 3.516 · 没有 4 段 2.843）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `muffled-talker`（speech）：差 +0.471（有 6 段 -6.784 · 没有 4 段 -7.255）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 43.667 ms | 43.667（phone-booth-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.196 dB | -0.669（phone-booth-c01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.500 百分点 | 5.000（phone-booth-c00-mba） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 0.588 dB | 2.493（phone-booth-c01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.021 比例 | 0.030（phone-booth-c01-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.252 dB | 1.946（phone-booth-c05-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.009 dB² | 2.705（phone-booth-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 2.500 个/分钟 | 10.000（phone-booth-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.007 Δ | -0.003（phone-booth-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.219 dB | 0.938（phone-booth-c00-mba） | 6.000 | 过 |
| V-MUSIC |  | 4 / 6 / 0 | 2.931 × | 6.693（phone-booth-c06-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -9.483 dB | -5.296（phone-booth-c04-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.240 · Distill-MOS Δ +0.249 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.240 | +0.119（phone-booth-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.249 | +0.114（phone-booth-c03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `muffled-talker`（speech）：差 +0.833（有 6 段 0.833 · 没有 4 段 0.000）
- V-EAT-ASR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 27.667 ms | 27.667（phone-booth-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.007（phone-booth-c02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.033 dB | 0.078（phone-booth-c00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.067 比例 | 0.127（phone-booth-c01-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -1.042 dB | -0.146（phone-booth-c05-mba） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.007 dB² | 0.039（phone-booth-c00-mba） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（phone-booth-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.008 dB | 0.013（phone-booth-c02-mba） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（phone-booth-c00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（phone-booth-c00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（phone-booth-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（phone-booth-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 43.667 ms | 43.667（phone-booth-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.043 dB | -0.098（phone-booth-c01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.500 百分点 | 5.000（phone-booth-c00-mba） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 0.700 dB | 1.709（phone-booth-c01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.024 比例 | 0.062（phone-booth-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.049 dB | 1.715（phone-booth-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.301 dB² | 0.825（phone-booth-c09-mba） | 1.000 | 过 |
| V-CLICK |  | 7 / 3 / 0 | 2.500 个/分钟 | 10.000（phone-booth-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.009 Δ | -0.002（phone-booth-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.081 dB | 0.357（phone-booth-c03-mba） | 6.000 | 过 |
| V-MUSIC |  | 3 / 7 / 0 | 2.863 × | 5.396（phone-booth-c06-mba） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.593 dB | -5.259（phone-booth-c05-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.293 · Distill-MOS Δ +0.208 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.293 | +0.155（phone-booth-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.208 | +0.080（phone-booth-c03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `muffled-talker`（speech）：差 +0.833（有 6 段 0.833 · 没有 4 段 0.000）
- V-EAT-ASR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `muffled-talker`（speech）：差 +0.003（有 6 段 0.025 · 没有 4 段 0.022）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `muffled-talker`（speech）：差 +0.398（有 6 段 3.022 · 没有 4 段 2.624）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `muffled-talker`（speech）：差 +0.435（有 6 段 -7.419 · 没有 4 段 -7.854）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（phone-booth-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.482 dB | -1.414（phone-booth-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | -0.216 百分点 | 3.704（phone-booth-s00-ext） | 1.000 | 不过 |
| V-ONSET |  | 6 / 0 / 0 | 0.925 dB | 2.335（phone-booth-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.022 比例 | 0.044（phone-booth-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.825 dB | 0.302（phone-booth-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 1.560 dB² | 2.625（phone-booth-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 3.333 个/分钟 | 5.000（phone-booth-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.028 Δ | 0.016（phone-booth-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.108 dB | 8.965（phone-booth-s04-ext） | 6.000 | 不过 |
| V-MUSIC |  | 4 / 2 / 0 | 1.594 × | 2.361（phone-booth-s00-ext） | 2.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -7.082 dB | -5.613（phone-booth-s02-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.173 · Distill-MOS Δ +0.186 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.173 | +0.044（phone-booth-s04-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.186 | +0.132（phone-booth-s03-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（phone-booth-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.003 dB | -0.014（phone-booth-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.064 dB | 0.363（phone-booth-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.172 比例 | 0.267（phone-booth-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -5.681 dB | -2.697（phone-booth-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 0 / 0 | 0.150 dB² | 0.300（phone-booth-s00-ext） | 1.000 | 过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（phone-booth-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.435 dB | 8.309（phone-booth-s04-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（phone-booth-s00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（phone-booth-s00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（phone-booth-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（phone-booth-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（phone-booth-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.075 dB | -0.106（phone-booth-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.617 百分点 | 3.704（phone-booth-s00-ext） | 1.000 | 不过 |
| V-ONSET |  | 6 / 0 / 0 | 1.116 dB | 2.321（phone-booth-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 0 / 0 | 0.012 比例 | 0.022（phone-booth-s00-ext） | 0.050 | 过 |
| V-WEAK |  | 6 / 0 / 0 | -1.906 dB | 0.366（phone-booth-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 2 / 0 | 0.761 dB² | 1.285（phone-booth-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 5.000 个/分钟 | 15.000（phone-booth-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.022 Δ | 0.009（phone-booth-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.743 dB | 8.509（phone-booth-s04-ext） | 6.000 | 不过 |
| V-MUSIC |  | 2 / 4 / 0 | 2.646 × | 4.234（phone-booth-s01-ext） | 2.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -5.021 dB | -0.092（phone-booth-s04-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.208 · Distill-MOS Δ +0.144 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.208 | +0.075（phone-booth-s04-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.144 | +0.095（phone-booth-s03-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（phone-booth-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.384 dB | -0.811（phone-booth-s04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 1.040 dB | 2.115（phone-booth-s04-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.028 比例 | 0.050（phone-booth-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.897 dB | 0.294（phone-booth-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 1.707 dB² | 2.894（phone-booth-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 10.000 个/分钟 | 20.000（phone-booth-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.016（phone-booth-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.036 dB | 9.182（phone-booth-s04-mba） | 6.000 | 不过 |
| V-MUSIC |  | 1 / 5 / 0 | 2.125 × | 2.677（phone-booth-s03-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -8.751 dB | -6.308（phone-booth-s04-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.133 · Distill-MOS Δ +0.226 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.133 | -0.050（phone-booth-s05-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.226 | +0.180（phone-booth-s01-mba） |

送人耳：
- phone-booth-s05-mba：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.24

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（phone-booth-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.005 dB | -0.017（phone-booth-s04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.005 dB | 0.167（phone-booth-s04-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.180 比例 | 0.310（phone-booth-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -6.006 dB | -3.150（phone-booth-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 0 / 0 | 0.301 dB² | 0.854（phone-booth-s02-mba） | 1.000 | 过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（phone-booth-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.454 dB | 8.356（phone-booth-s04-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（phone-booth-s00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（phone-booth-s00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（phone-booth-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（phone-booth-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（phone-booth-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.098 dB | -0.122（phone-booth-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（phone-booth-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 1.034 dB | 1.602（phone-booth-s00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.015 比例 | 0.030（phone-booth-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.088 dB | 0.238（phone-booth-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 2 / 0 | 0.874 dB² | 1.406（phone-booth-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 15.833 个/分钟 | 30.000（phone-booth-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.027 Δ | 0.015（phone-booth-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.743 dB | 8.705（phone-booth-s04-mba） | 6.000 | 不过 |
| V-MUSIC |  | 1 / 5 / 0 | 2.728 × | 5.478（phone-booth-s01-mba） | 2.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -5.993 dB | -0.135（phone-booth-s04-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.159 · Distill-MOS Δ +0.195 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.159 | -0.029（phone-booth-s05-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.195 | +0.115（phone-booth-s04-mba） |

送人耳：
- phone-booth-s05-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.19

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### meeting-room · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（meeting-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.026 dB | -2.039（meeting-room-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 7 / 3 / 0 | 0.893 百分点 | 9.375（meeting-room-c04-ext） | 1.000 | 不过 |
| V-ONSET |  | 2 / 1 / 7 | 2.029 dB | 3.510（meeting-room-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.028 比例 | 0.068（meeting-room-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.484 dB | 2.922（meeting-room-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.363 dB² | 4.214（meeting-room-c06-ext） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 6.500 个/分钟 | 15.000（meeting-room-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.005 Δ | -0.011（meeting-room-c02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.606 dB | 2.565（meeting-room-c00-ext） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -7.188 dB | -4.909（meeting-room-c05-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.011 dB | 7.167（meeting-room-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.101 · Distill-MOS Δ +0.182 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.101 | +0.008（meeting-room-c08-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.182 | +0.088（meeting-room-c08-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `chair`（event）：差 +2.962（有 5 段 2.374 · 没有 5 段 -0.588）
- V-EAT-ASR · `bed-ace`（bed）：差 +0.795（有 3 段 1.449 · 没有 7 段 0.655）
- V-EAT-ASR · `door`（event）：差 +0.055（有 4 段 0.926 · 没有 6 段 0.871）
- V-EAT-ASR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.023（有 3 段 0.045 · 没有 7 段 0.021）
- V-SIM-GATE · `door`（event）：差 +0.013（有 4 段 0.036 · 没有 6 段 0.023）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +2.129（有 4 段 3.641 · 没有 6 段 1.512）
- V-PUMPV · `bed-ace`（bed）：差 +0.992（有 3 段 3.058 · 没有 7 段 2.066）
- V-PUMPV · `laugh`（event）：差 +0.651（有 6 段 2.624 · 没有 4 段 1.973）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `door`（event）：差 +1.245（有 4 段 -6.441 · 没有 6 段 -7.686）
- V-NR · `bed-room`（bed）：差 +0.694（有 7 段 -6.980 · 没有 3 段 -7.674）
- V-NR · `laugh`（event）：差 +0.459（有 6 段 -7.004 · 没有 4 段 -7.463）
- V-NR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（meeting-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.001（meeting-room-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（meeting-room-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 3 / 0 / 7 | -0.218 dB | -0.095（meeting-room-c09-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.171 比例 | 0.386（meeting-room-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.619 dB | -0.079（meeting-room-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 0.347 dB² | 1.232（meeting-room-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（meeting-room-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（meeting-room-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.346 dB | 7.250（meeting-room-c04-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（meeting-room-c00-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.917 dB | 3.254（meeting-room-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（meeting-room-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（meeting-room-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`（bed）：差 +0.088（有 3 段 0.233 · 没有 7 段 0.145）
- V-SIM-GATE · `door`（event）：差 +0.058（有 4 段 0.206 · 没有 6 段 0.148）
- V-SIM-GATE · `chair`（event）：差 +0.045（有 5 段 0.193 · 没有 5 段 0.149）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +0.761（有 4 段 0.803 · 没有 6 段 0.042）
- V-PUMPV · `bed-ace`（bed）：差 +0.112（有 3 段 0.425 · 没有 7 段 0.313）
- V-PUMPV · `laugh`（event）：差 +0.063（有 6 段 0.372 · 没有 4 段 0.309）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`（event）：差 +1.568（有 6 段 1.973 · 没有 4 段 0.405）
- V-TIMBRE · `chair`（event）：差 +0.973（有 5 段 1.832 · 没有 5 段 0.860）
- V-TIMBRE · `bed-ace`（bed）：差 +0.412（有 3 段 1.634 · 没有 7 段 1.222）
- V-TIMBRE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（meeting-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.165 dB | -0.304（meeting-room-c06-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | -1.195 百分点 | 6.250（meeting-room-c04-ext） | 1.000 | 不过 |
| V-ONSET |  | 3 / 0 / 7 | 0.506 dB | 0.827（meeting-room-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.049 比例 | 0.172（meeting-room-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.641 dB | 1.336（meeting-room-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.071 dB² | 3.362（meeting-room-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 12.000 个/分钟 | 50.000（meeting-room-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.002 Δ | -0.010（meeting-room-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.346 dB | 8.345（meeting-room-c04-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 9 / 0 | -3.629 dB | -0.989（meeting-room-c07-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.460 dB | 6.066（meeting-room-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 4 | SCOREQ Δ +0.042 · Distill-MOS Δ +0.052 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.042 | -0.017（meeting-room-c05-ext） |
| V-NRMOS-DISTILL | 9 / 1 / 0 | +0.052 | -0.031（meeting-room-c02-ext） |

送人耳：
- meeting-room-c02-ext：一个说好一个说差：SCOREQ Δ +0.07 · Distill-MOS Δ -0.03
- meeting-room-c03-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.06
- meeting-room-c05-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.01
- meeting-room-c08-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.01

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +3.778（有 3 段 1.449 · 没有 7 段 -2.328）
- V-EAT-ASR · `door`（event）：差 +1.992（有 4 段 0.000 · 没有 6 段 -1.992）
- V-EAT-ASR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.051（有 3 段 0.085 · 没有 7 段 0.034）
- V-SIM-GATE · `door`（event）：差 +0.025（有 4 段 0.064 · 没有 6 段 0.039）
- V-SIM-GATE · `laugh`（event）：差 +0.022（有 6 段 0.058 · 没有 4 段 0.036）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +1.369（有 4 段 1.893 · 没有 6 段 0.523）
- V-PUMPV · `bed-ace`（bed）：差 +0.331（有 3 段 1.303 · 没有 7 段 0.972）
- V-PUMPV · `laugh`（event）：差 +0.231（有 6 段 1.163 · 没有 4 段 0.933）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`（event）：差 +1.608（有 6 段 1.989 · 没有 4 段 0.382）
- V-TIMBRE · `chair`（event）：差 +1.286（有 5 段 1.989 · 没有 5 段 0.703）
- V-TIMBRE · `bed-room`（bed）：差 +0.046（有 7 段 1.360 · 没有 3 段 1.314）
- V-TIMBRE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `chair`（event）：差 +1.601（有 5 段 -2.829 · 没有 5 段 -4.430）
- V-NR · `door`（event）：差 +0.991（有 4 段 -3.035 · 没有 6 段 -4.026）
- V-NR · `bed-ace`（bed）：差 +0.329（有 3 段 -3.399 · 没有 7 段 -3.728）
- V-NR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（meeting-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.050 dB | -1.940（meeting-room-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | 2.020 百分点 | 11.111（meeting-room-c07-mba） | 1.000 | 不过 |
| V-ONSET |  | 1 / 2 / 7 | 2.599 dB | 4.097（meeting-room-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.037 比例 | 0.077（meeting-room-c06-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.437 dB | 2.471（meeting-room-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 3.035 dB² | 6.662（meeting-room-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 4.500 个/分钟 | 15.000（meeting-room-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.010 Δ | -0.004（meeting-room-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.564 dB | 2.672（meeting-room-c00-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -9.935 dB | -7.731（meeting-room-c06-mba） | -6.000 | 过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.258 dB | 8.279（meeting-room-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.101 · Distill-MOS Δ +0.218 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.101 | -0.047（meeting-room-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.218 | +0.102（meeting-room-c08-mba） |

送人耳：
- meeting-room-c08-mba：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.10

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-room`（bed）：差 +2.886（有 7 段 2.886 · 没有 3 段 0.000）
- V-EAT-ASR · `door`（event）：差 +1.263（有 4 段 2.778 · 没有 6 段 1.515）
- V-EAT-ASR · `chair`（event）：差 +0.404（有 5 段 2.222 · 没有 5 段 1.818）
- V-EAT-ASR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.039（有 4 段 0.061 · 没有 6 段 0.022）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.013（有 3 段 0.046 · 没有 7 段 0.034）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +3.356（有 4 段 5.048 · 没有 6 段 1.693）
- V-PUMPV · `bed-ace`（bed）：差 +1.837（有 3 段 4.321 · 没有 7 段 2.484）
- V-PUMPV · `laugh`（event）：差 +1.450（有 6 段 3.615 · 没有 4 段 2.165）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（meeting-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.000（meeting-room-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（meeting-room-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 3 / 0 / 7 | -0.217 dB | -0.028（meeting-room-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.184 比例 | 0.360（meeting-room-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.090 dB | -0.320（meeting-room-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 0.619 dB² | 3.735（meeting-room-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（meeting-room-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（meeting-room-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.755 dB | 2.803（meeting-room-c06-mba） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（meeting-room-c00-mba） | -6.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.570 dB | 1.531（meeting-room-c09-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（meeting-room-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（meeting-room-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`（bed）：差 +0.094（有 3 段 0.250 · 没有 7 段 0.155）
- V-SIM-GATE · `door`（event）：差 +0.050（有 4 段 0.214 · 没有 6 段 0.164）
- V-SIM-GATE · `chair`（event）：差 +0.037（有 5 段 0.202 · 没有 5 段 0.165）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +1.460（有 4 段 1.495 · 没有 6 段 0.035）
- V-PUMPV · `bed-ace`（bed）：差 +0.927（有 3 段 1.268 · 没有 7 段 0.341）
- V-PUMPV · `laugh`（event）：差 +0.467（有 6 段 0.806 · 没有 4 段 0.339）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（meeting-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.250 dB | -0.404（meeting-room-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 7 / 3 / 0 | 4.811 百分点 | 30.769（meeting-room-c04-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 0 / 7 | 0.870 dB | 1.345（meeting-room-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.038 比例 | 0.105（meeting-room-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.409 dB | 1.693（meeting-room-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.247 dB² | 3.130（meeting-room-c07-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 8 / 0 | 5.500 个/分钟 | 15.000（meeting-room-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.007 Δ | -0.006（meeting-room-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.614 dB | 2.276（meeting-room-c04-mba） | 6.000 | 过 |
| V-NR |  | 4 / 6 / 0 | -4.599 dB | -1.008（meeting-room-c07-mba） | -6.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.569 dB | 4.663（meeting-room-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.055 · Distill-MOS Δ +0.115 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.055 | -0.078（meeting-room-c05-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.115 | +0.045（meeting-room-c05-mba） |

送人耳：
- meeting-room-c05-mba：一个说好一个说差：SCOREQ Δ -0.08 · Distill-MOS Δ +0.05
- meeting-room-c08-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.10

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-room`（bed）：差 +6.873（有 7 段 6.873 · 没有 3 段 0.000）
- V-EAT-ASR · `chair`（event）：差 +4.167（有 5 段 6.895 · 没有 5 段 2.727）
- V-EAT-ASR · `laugh`（event）：差 +0.793（有 6 段 5.128 · 没有 4 段 4.335）
- V-EAT-ASR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`（event）：差 +0.024（有 6 段 0.048 · 没有 4 段 0.024）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.007（有 3 段 0.043 · 没有 7 段 0.036）
- V-SIM-GATE · `chair`（event）：差 +0.003（有 5 段 0.040 · 没有 5 段 0.036）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +1.256（有 4 段 2.001 · 没有 6 段 0.745）
- V-PUMPV · `bed-room`（bed）：差 +0.269（有 7 段 1.328 · 没有 3 段 1.059）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `chair`（event）：差 +3.485（有 5 段 -2.856 · 没有 5 段 -6.341）
- V-NR · `bed-ace`（bed）：差 +2.174（有 3 段 -3.077 · 没有 7 段 -5.251）
- V-NR · `door`（event）：差 +1.307（有 4 段 -3.815 · 没有 6 段 -5.122）
- V-NR · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（meeting-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.956 dB | -3.448（meeting-room-s01-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | -1.774 百分点 | 3.333（meeting-room-s05-ext） | 1.000 | 不过 |
| V-ONSET |  | 1 / 0 / 5 | 2.160 dB | 2.160（meeting-room-s00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.022 比例 | 0.038（meeting-room-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 1.191 dB | 3.406（meeting-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 3 / 3 / 0 | 1.347 dB² | 2.587（meeting-room-s03-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 5.833 个/分钟 | 10.000（meeting-room-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.001 Δ | -0.006（meeting-room-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 4.838 dB | 10.358（meeting-room-s05-ext） | 6.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -6.405 dB | -3.887（meeting-room-s03-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.652 dB | 3.406（meeting-room-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.067 · Distill-MOS Δ +0.209 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.067 | +0.002（meeting-room-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.209 | +0.152（meeting-room-s01-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `chair`（event）：差 +4.144（有 3 段 0.298 · 没有 3 段 -3.846）
- V-EAT-ASR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +3.685（有 3 段 6.680 · 没有 3 段 2.996）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（meeting-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.002 dB | -0.006（meeting-room-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（meeting-room-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 1 / 0 / 5 | 0.173 dB | 0.173（meeting-room-s00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.406 比例 | 0.509（meeting-room-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.274 dB | -0.347（meeting-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 0 / 0 | 0.195 dB² | 0.655（meeting-room-s00-ext） | 1.000 | 过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（meeting-room-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（meeting-room-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.134 dB | 9.172（meeting-room-s05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（meeting-room-s00-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.103 dB | 0.261（meeting-room-s05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（meeting-room-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（meeting-room-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +1.942（有 3 段 7.105 · 没有 3 段 5.162）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（meeting-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.353 dB | -0.485（meeting-room-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 2.661 百分点 | 12.500（meeting-room-s03-ext） | 1.000 | 不过 |
| V-ONSET |  | 1 / 0 / 5 | 0.231 dB | 0.231（meeting-room-s00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.048 比例 | 0.081（meeting-room-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 0.360 dB | 2.097（meeting-room-s05-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 1.088 dB² | 1.845（meeting-room-s03-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 7.500 个/分钟 | 20.000（meeting-room-s03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.002 Δ | -0.006（meeting-room-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.411 dB | 9.277（meeting-room-s05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -1.620 dB | -0.435（meeting-room-s04-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.250 dB | 0.976（meeting-room-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.081 · Distill-MOS Δ +0.083 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.081 | +0.017（meeting-room-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.083 | +0.054（meeting-room-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +1.376（有 3 段 7.099 · 没有 3 段 5.723）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `chair`（event）：差 +1.689（有 3 段 -0.776 · 没有 3 段 -2.465）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（meeting-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.184 dB | -3.770（meeting-room-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 3.205 百分点 | 19.231（meeting-room-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 1 / 0 / 5 | 0.810 dB | 0.810（meeting-room-s00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.029 比例 | 0.064（meeting-room-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 0.582 dB | 3.472（meeting-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 1.941 dB² | 3.180（meeting-room-s03-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 14.167 个/分钟 | 25.000（meeting-room-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.008 Δ | 0.000（meeting-room-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 3.707 dB | 7.961（meeting-room-s04-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -6.537 dB | -4.308（meeting-room-s03-mba） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.989 dB | 5.629（meeting-room-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.126 · Distill-MOS Δ +0.230 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.126 | -0.006（meeting-room-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.230 | +0.164（meeting-room-s01-mba） |

送人耳：
- meeting-room-s00-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.24

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `chair`（event）：差 +0.009（有 3 段 0.034 · 没有 3 段 0.025）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `chair`（event）：差 +0.639（有 3 段 2.261 · 没有 3 段 1.621）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +2.483（有 3 段 4.949 · 没有 3 段 2.466）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（meeting-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.002 dB | -0.010（meeting-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（meeting-room-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 1 / 0 / 5 | 0.310 dB | 0.310（meeting-room-s00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.487 比例 | 0.568（meeting-room-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -3.746 dB | -0.955（meeting-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 0 / 0 | 0.174 dB² | 0.420（meeting-room-s03-mba） | 1.000 | 过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（meeting-room-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（meeting-room-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 5.945 dB | 8.846（meeting-room-s05-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（meeting-room-s00-mba） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.052 dB | 0.142（meeting-room-s05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（meeting-room-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（meeting-room-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +2.552（有 3 段 7.221 · 没有 3 段 4.669）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（meeting-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.596 dB | -0.818（meeting-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 9.900 百分点 | 46.154（meeting-room-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 1 / 0 / 5 | 1.600 dB | 1.600（meeting-room-s00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.046 比例 | 0.077（meeting-room-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -0.075 dB | 2.125（meeting-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 1.538 dB² | 2.128（meeting-room-s05-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 10.000 个/分钟 | 30.000（meeting-room-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.015 Δ | 0.008（meeting-room-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.175 dB | 9.110（meeting-room-s05-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -1.733 dB | -0.720（meeting-room-s00-mba） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.852 dB | 4.842（meeting-room-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.152 · Distill-MOS Δ +0.175 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.152 | +0.011（meeting-room-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.175 | +0.115（meeting-room-s03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `chair`（event）：差 +0.004（有 3 段 0.048 · 没有 3 段 0.044）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `chair`（event）：差 +0.166（有 3 段 1.621 · 没有 3 段 1.455）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +1.976（有 3 段 7.163 · 没有 3 段 5.187）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `chair`（event）：差 +1.468（有 3 段 -0.999 · 没有 3 段 -2.467）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### cowork · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cowork-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -1.315 dB | -4.766（cowork-c00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.230 百分点 | 9.524（cowork-c00-ext） | 1.000 | 不过 |
| V-ONSET |  | 6 / 1 / 3 | 1.263 dB | 5.763（cowork-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.031 比例 | 0.144（cowork-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.025 dB | -0.873（cowork-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.176 dB² | 7.447（cowork-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 26.000 个/分钟 | 45.000（cowork-c07-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.005（cowork-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.884 dB | 3.476（cowork-c00-ext） | 6.000 | 过 |
| V-NR |  | 1 / 9 / 0 | -4.172 dB | -1.328（cowork-c05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -3.853 dB | -2.691（cowork-c06-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.849 比例 | 1.000（cowork-c02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.987 dB | 5.653（cowork-c06-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.153 · Distill-MOS Δ +0.203 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.153 | +0.027（cowork-c06-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.203 | +0.063（cowork-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`（bed）：差 +0.522（有 7 段 -1.471 · 没有 3 段 -0.949）
- V-SPLVL · `talker`（speech）：差 +0.232（有 5 段 -1.430 · 没有 5 段 -1.199）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `talker`（speech）：差 +2.460（有 5 段 1.460 · 没有 5 段 -1.000）
- V-EAT-ASR · `bed-ace`（bed）：差 +0.329（有 7 段 0.329 · 没有 3 段 0.000）
- V-EAT-ASR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.021（有 7 段 0.037 · 没有 3 段 0.016）
- V-SIM-GATE · `talker`（speech）：差 +0.015（有 5 段 0.038 · 没有 5 段 0.023）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.477（有 5 段 2.914 · 没有 5 段 1.437）
- V-PUMPV · `bed-ace`（bed）：差 +1.220（有 7 段 2.542 · 没有 3 段 1.322）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `coffee-machine`（pattern）：差 +1.009（有 5 段 -3.667 · 没有 5 段 -4.677）
- V-NR · `footsteps`（event）：差 +0.399（有 5 段 -3.972 · 没有 5 段 -4.372）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（cowork-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.003（cowork-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（cowork-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 1 / 3 | 0.617 dB | 5.397（cowork-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.304 比例 | 0.533（cowork-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.727 dB | -3.147（cowork-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 1.115 dB² | 6.119（cowork-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（cowork-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（cowork-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.333 dB | 4.326（cowork-c00-ext） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（cowork-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（cowork-c00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.486 比例 | 0.958（cowork-c02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.323 dB | 1.348（cowork-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（cowork-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（cowork-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +1.353（有 4 段 1.197 · 没有 3 段 -0.157）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.095（有 7 段 0.332 · 没有 3 段 0.237）
- V-SIM-GATE · `footsteps`（event）：差 +0.081（有 5 段 0.344 · 没有 5 段 0.263）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +1.017（有 7 段 1.420 · 没有 3 段 0.403）
- V-PUMPV · `footsteps`（event）：差 +0.865（有 5 段 1.548 · 没有 5 段 0.682）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cowork-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.162 dB | -0.335（cowork-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.722 百分点 | 0.000（cowork-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 1 / 3 | 1.553 dB | 7.421（cowork-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.041 比例 | 0.117（cowork-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.360 dB | -2.063（cowork-c01-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.969 dB² | 7.626（cowork-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 28.000 个/分钟 | 55.000（cowork-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.011 Δ | -0.002（cowork-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.191 dB | 4.253（cowork-c00-ext） | 6.000 | 过 |
| V-NR |  | 1 / 9 / 0 | -3.156 dB | -1.125（cowork-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -1.267 dB | -0.496（cowork-c02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.917 比例 | 1.000（cowork-c02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.486 dB | 3.936（cowork-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 1 / 0 | SCOREQ Δ +0.062 · Distill-MOS Δ +0.099 | | Δ ≥ 0 | 不过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.062 | -0.015（cowork-c05-ext） |
| V-NRMOS-DISTILL | 9 / 1 / 0 | +0.099 | -0.002（cowork-c05-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +1.259（有 4 段 2.092 · 没有 3 段 0.833）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.017（有 5 段 0.050 · 没有 5 段 0.033）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +1.535（有 7 段 2.429 · 没有 3 段 0.894）
- V-PUMPV · `footsteps`（event）：差 +0.855（有 5 段 2.396 · 没有 5 段 1.541）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +0.603（有 5 段 -2.855 · 没有 5 段 -3.458）
- V-NR · `coffee-machine`（pattern）：差 +0.347（有 5 段 -2.983 · 没有 5 段 -3.330）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `talker`（speech）：差 +0.024（有 5 段 0.049 · 没有 5 段 0.074）
- V-NRMOS-SCOREQ · `coffee-machine`（pattern）：差 +0.021（有 5 段 0.051 · 没有 5 段 0.072）
- V-NRMOS-SCOREQ · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `coffee-machine`（pattern）：差 +0.055（有 5 段 0.071 · 没有 5 段 0.126）
- V-NRMOS-DISTILL · `talker`（speech）：差 +0.001（有 5 段 0.098 · 没有 5 段 0.099）
- V-NRMOS-DISTILL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### cowork · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cowork-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -1.470 dB | -3.943（cowork-c00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -0.744 百分点 | 4.545（cowork-c06-mba） | 1.000 | 不过 |
| V-ONSET |  | 6 / 1 / 3 | 1.484 dB | 3.520（cowork-c07-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.037 比例 | 0.104（cowork-c01-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.556 dB | -2.267（cowork-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.575 dB² | 4.145（cowork-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 29.500 个/分钟 | 60.000（cowork-c07-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.030 Δ | 0.007（cowork-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.065 dB | 1.885（cowork-c03-mba） | 6.000 | 过 |
| V-NR |  | 4 / 6 / 0 | -5.632 dB | -3.391（cowork-c08-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -5.201 dB | -3.591（cowork-c06-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.546 比例 | 0.754（cowork-c09-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.635 dB | 7.570（cowork-c00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.204 · Distill-MOS Δ +0.233 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.204 | +0.046（cowork-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.233 | +0.104（cowork-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`（bed）：差 +0.472（有 7 段 -1.612 · 没有 3 段 -1.140）
- V-SPLVL · `talker`（speech）：差 +0.247（有 5 段 -1.594 · 没有 5 段 -1.346）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `coffee-machine`（pattern）：差 +0.599（有 5 段 -0.444 · 没有 5 段 -1.043）
- V-EAT-ASR · `talker`（speech）：差 +0.512（有 5 段 -0.488 · 没有 5 段 -1.000）
- V-EAT-ASR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.026（有 7 段 0.045 · 没有 3 段 0.019）
- V-SIM-GATE · `footsteps`（event）：差 +0.019（有 5 段 0.046 · 没有 5 段 0.028）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.134（有 5 段 3.142 · 没有 5 段 2.008）
- V-PUMPV · `bed-ace`（bed）：差 +0.263（有 7 段 2.654 · 没有 3 段 2.391）
- V-PUMPV · `talker`（speech）：差 +0.084（有 5 段 2.618 · 没有 5 段 2.533）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +2.071（有 5 段 -4.597 · 没有 5 段 -6.668）
- V-NR · `bed-ace`（bed）：差 +0.564（有 7 段 -5.463 · 没有 3 段 -6.027）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（cowork-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.013（cowork-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（cowork-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 7 / 0 / 3 | 0.044 dB | 2.006（cowork-c06-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.328 比例 | 0.519（cowork-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.902 dB | -6.820（cowork-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 7 / 3 / 0 | 1.064 dB² | 6.240（cowork-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（cowork-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（cowork-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.188 dB | 3.906（cowork-c05-mba） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（cowork-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（cowork-c00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.486 比例 | 0.979（cowork-c02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.218 dB | 0.713（cowork-c07-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（cowork-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（cowork-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`（bed）：差 +0.065（有 7 段 0.348 · 没有 3 段 0.283）
- V-SIM-GATE · `talker`（speech）：差 +0.045（有 5 段 0.351 · 没有 5 段 0.306）
- V-SIM-GATE · `footsteps`（event）：差 +0.016（有 5 段 0.336 · 没有 5 段 0.320）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.177（有 5 段 1.653 · 没有 5 段 0.476）
- V-PUMPV · `coffee-machine`（pattern）：差 +0.822（有 5 段 1.476 · 没有 5 段 0.653）
- V-PUMPV · `bed-ace`（bed）：差 +0.491（有 7 段 1.212 · 没有 3 段 0.721）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cowork-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.294 dB | -0.698（cowork-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.500 百分点 | 0.000（cowork-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 1 / 3 | 1.171 dB | 3.459（cowork-c06-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.029 比例 | 0.092（cowork-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -6.551 dB | -2.875（cowork-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.327 dB² | 7.844（cowork-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 29.500 个/分钟 | 55.000（cowork-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.017 Δ | 0.007（cowork-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.031 dB | 2.940（cowork-c05-mba） | 6.000 | 过 |
| V-NR |  | 1 / 9 / 0 | -3.098 dB | -1.113（cowork-c03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -1.583 dB | -0.708（cowork-c02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.907 比例 | 1.000（cowork-c02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.621 dB | 3.346（cowork-c08-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.162 · Distill-MOS Δ +0.160 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.162 | +0.033（cowork-c03-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.160 | +0.098（cowork-c03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.003（有 5 段 0.030 · 没有 5 段 0.027）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.401（有 5 段 3.028 · 没有 5 段 1.627）
- V-PUMPV · `coffee-machine`（pattern）：差 +0.985（有 5 段 2.820 · 没有 5 段 1.835）
- V-PUMPV · `bed-ace`（bed）：差 +0.650（有 7 段 2.522 · 没有 3 段 1.872）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +1.211（有 5 段 -2.493 · 没有 5 段 -3.704）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cowork-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.356 dB | -5.620（cowork-s03-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.347 百分点 | 2.083（cowork-s02-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 1.402 dB | 3.793（cowork-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.052 比例 | 0.113（cowork-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -8.897 dB | -2.241（cowork-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.270 dB² | 42.743（cowork-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 22.500 个/分钟 | 40.000（cowork-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.018 Δ | 0.003（cowork-s05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.173 dB | 4.084（cowork-s05-ext） | 6.000 | 过 |
| V-NR |  | 0 / 6 / 0 | -4.305 dB | -2.711（cowork-s05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -4.319 dB | -3.364（cowork-s05-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 1.000 比例 | 1.000（cowork-s01-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.683 dB | 2.915（cowork-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.059 · Distill-MOS Δ +0.146 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.059 | -0.073（cowork-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.146 | +0.082（cowork-s03-ext） |

送人耳：
- cowork-s01-ext：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.17
- cowork-s03-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.08

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `babble`（speech）：差 +1.472（有 3 段 -4.092 · 没有 3 段 -2.620）
- V-SPLVL · `bed-ace`（bed）：差 +1.173（有 3 段 -3.942 · 没有 3 段 -2.770）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +1.666（有 3 段 2.235 · 没有 3 段 0.569）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.042（有 3 段 0.073 · 没有 3 段 0.031）
- V-SIM-GATE · `babble`（speech）：差 +0.008（有 3 段 0.056 · 没有 3 段 0.048）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +8.879（有 3 段 16.709 · 没有 3 段 7.831）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +1.333（有 3 段 -3.639 · 没有 3 段 -4.972）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（cowork-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.002 dB | -0.013（cowork-s03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（cowork-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.900 dB | 2.248（cowork-s05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.484 比例 | 0.774（cowork-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.130 dB | -5.506（cowork-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.965 dB² | 13.562（cowork-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（cowork-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（cowork-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 3.328 dB | 4.794（cowork-s05-ext） | 6.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（cowork-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（cowork-s01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 2 | 0.367 比例 | 0.750（cowork-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.134 dB | 0.617（cowork-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（cowork-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（cowork-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`（bed）：差 +0.227（有 3 段 0.597 · 没有 3 段 0.370）
- V-SIM-GATE · `babble`（speech）：差 +0.220（有 3 段 0.594 · 没有 3 段 0.374）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cowork-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.319 dB | -0.696（cowork-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 3.138 百分点 | 13.043（cowork-s04-ext） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.159 dB | 4.312（cowork-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.111 比例 | 0.196（cowork-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.610 dB | -2.277（cowork-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 19.148 dB² | 60.776（cowork-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 29.167 个/分钟 | 35.000（cowork-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.006 Δ | -0.002（cowork-s05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 3.407 dB | 4.962（cowork-s05-ext） | 6.000 | 过 |
| V-NR |  | 0 / 6 / 0 | -1.632 dB | -1.014（cowork-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -0.891 dB | -0.458（cowork-s04-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.773 比例 | 1.000（cowork-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.649 dB | 1.538（cowork-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 3 / 0 / 3 | SCOREQ Δ -0.007 · Distill-MOS Δ +0.117 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 3 / 3 / 0 | -0.007 | -0.039（cowork-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.117 | +0.071（cowork-s02-ext） |

送人耳：
- cowork-s01-ext：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.15
- cowork-s03-ext：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.13
- cowork-s04-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.09

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +2.419（有 3 段 4.348 · 没有 3 段 1.929）
- V-EAT-ASR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +2.114（有 3 段 3.215 · 没有 3 段 1.102）
- V-ONSET · `bed-ace`（bed）：差 +0.276（有 3 段 2.297 · 没有 3 段 2.021）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`（speech）：差 +0.075（有 3 段 0.148 · 没有 3 段 0.073）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.003（有 3 段 0.112 · 没有 3 段 0.109）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +20.605（有 3 段 29.450 · 没有 3 段 8.845）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +0.496（有 3 段 -1.384 · 没有 3 段 -1.881）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cowork-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.412 dB | -5.734（cowork-s03-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 2.654 百分点 | 7.407（cowork-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 1.588 dB | 4.880（cowork-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.033 比例 | 0.085（cowork-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.633 dB | -3.408（cowork-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.202 dB² | 25.109（cowork-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 24.167 个/分钟 | 35.000（cowork-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.021 Δ | 0.004（cowork-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.583 dB | 2.230（cowork-s01-mba） | 6.000 | 过 |
| V-NR |  | 1 / 5 / 0 | -5.474 dB | -3.798（cowork-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -4.552 dB | -3.922（cowork-s01-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.979 比例 | 1.000（cowork-s01-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.769 dB | 5.833（cowork-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.061 · Distill-MOS Δ +0.169 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.061 | -0.068（cowork-s03-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.169 | +0.080（cowork-s03-mba） |

送人耳：
- cowork-s01-mba：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.14
- cowork-s03-mba：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.08

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `babble`（speech）：差 +1.955（有 3 段 -4.390 · 没有 3 段 -2.435）
- V-SPLVL · `bed-ace`（bed）：差 +0.947（有 3 段 -3.886 · 没有 3 段 -2.939）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +1.538（有 3 段 2.358 · 没有 3 段 0.819）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`（speech）：差 +0.029（有 3 段 0.048 · 没有 3 段 0.019）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.017（有 3 段 0.042 · 没有 3 段 0.025）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +5.623（有 3 段 11.013 · 没有 3 段 5.390）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +1.685（有 3 段 -4.632 · 没有 3 段 -6.316）
- V-NR · `bed-ace`（bed）：差 +0.875（有 3 段 -5.037 · 没有 3 段 -5.911）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（cowork-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.032 dB | -0.174（cowork-s05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（cowork-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 1.034 dB | 3.395（cowork-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.532 比例 | 0.849（cowork-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -14.275 dB | -7.187（cowork-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 7.050 dB² | 21.022（cowork-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（cowork-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（cowork-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.982 dB | 5.204（cowork-s01-mba） | 6.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（cowork-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（cowork-s01-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 2 | 0.339 比例 | 0.750（cowork-s03-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.094 dB | 0.440（cowork-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（cowork-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（cowork-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `babble`（speech）：差 +1.195（有 3 段 1.632 · 没有 3 段 0.437）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`（speech）：差 +0.256（有 3 段 0.660 · 没有 3 段 0.404）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.027（有 3 段 0.546 · 没有 3 段 0.519）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +3.963（有 3 段 9.031 · 没有 3 段 5.068）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cowork-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.448 dB | -1.037（cowork-s05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 1.994 百分点 | 4.348（cowork-s04-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.434 dB | 6.431（cowork-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.093 比例 | 0.213（cowork-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.890 dB | -3.212（cowork-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 8.046 dB² | 16.222（cowork-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 37.500 个/分钟 | 55.000（cowork-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.011 Δ | -0.002（cowork-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 3.086 dB | 5.896（cowork-s01-mba） | 6.000 | 过 |
| V-NR |  | 0 / 6 / 0 | -1.773 dB | -0.904（cowork-s01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -1.205 dB | -0.418（cowork-s04-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.770 比例 | 1.000（cowork-s03-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.657 dB | 2.057（cowork-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.054 · Distill-MOS Δ +0.179 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.054 | -0.002（cowork-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.179 | +0.059（cowork-s04-mba） |

送人耳：
- cowork-s01-mba：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.14

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +1.210（有 3 段 2.599 · 没有 3 段 1.389）
- V-EAT-ASR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +2.277（有 3 段 3.572 · 没有 3 段 1.295）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.045（有 3 段 0.115 · 没有 3 段 0.070）
- V-SIM-GATE · `babble`（speech）：差 +0.022（有 3 段 0.103 · 没有 3 段 0.082）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +4.309（有 3 段 10.201 · 没有 3 段 5.892）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +0.854（有 3 段 -1.346 · 没有 3 段 -2.200）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cafe-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.405 dB | -2.841（cafe-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 1.905 百分点 | 19.048（cafe-c05-ext） | 1.000 | 不过 |
| V-ONSET |  | 8 / 1 / 1 | 1.878 dB | 6.868（cafe-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.021 比例 | 0.052（cafe-c02-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.166 dB | -2.984（cafe-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.278 dB² | 6.312（cafe-c02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 34.000 个/分钟 | 55.000（cafe-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.038 Δ | 0.022（cafe-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 2.828 dB | 8.446（cafe-c08-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 9 / 0 | -5.989 dB | -4.384（cafe-c05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -4.816 dB | -2.998（cafe-c08-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.767 比例 | 1.000（cafe-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.876 dB | 5.913（cafe-c06-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.223 · Distill-MOS Δ +0.259 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.223 | +0.079（cafe-c03-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.259 | +0.189（cafe-c02-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +3.175（有 6 段 3.175 · 没有 4 段 0.000）
- V-EAT-ASR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`（pattern）：差 +1.388（有 4 段 2.649 · 没有 5 段 1.261）
- V-ONSET · `talker`（speech）：差 +1.344（有 6 段 2.326 · 没有 3 段 0.982）
- V-ONSET · `music`（pattern）：差 +0.503（有 6 段 2.046 · 没有 3 段 1.543）
- V-ONSET · `bed-restaurant`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.006（有 4 段 0.024 · 没有 6 段 0.019）
- V-SIM-GATE · `talker`（speech）：差 +0.002（有 6 段 0.022 · 没有 4 段 0.020）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.614（有 6 段 4.523 · 没有 4 段 3.909）
- V-PUMPV · `dishes`（event）：差 +0.137（有 4 段 4.360 · 没有 6 段 4.223）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `music`（pattern）：差 +1.691（有 6 段 3.504 · 没有 4 段 1.813）
- V-TIMBRE · `dishes`（event）：差 +1.573（有 4 段 3.772 · 没有 6 段 2.199）
- V-TIMBRE · `talker`（speech）：差 +0.685（有 6 段 3.102 · 没有 4 段 2.417）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `dishes`（event）：差 +0.623（有 4 段 -5.615 · 没有 6 段 -6.238）
- V-NR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（cafe-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（cafe-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（cafe-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 8 / 1 / 1 | 0.248 dB | 3.069（cafe-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.392 比例 | 0.664（cafe-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -12.757 dB | -7.393（cafe-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.029 dB² | 3.622（cafe-c06-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（cafe-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（cafe-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 4.520 dB | 9.706（cafe-c05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（cafe-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（cafe-c02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 2 / 1 / 7 | 0.198 比例 | 0.595（cafe-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.058 dB | 0.106（cafe-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（cafe-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（cafe-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `coffee-machine`（pattern）：差 +1.051（有 4 段 0.831 · 没有 5 段 -0.219）
- V-ONSET · `music`（pattern）：差 +1.008（有 6 段 0.584 · 没有 3 段 -0.424）
- V-ONSET · `talker`（speech）：差 +0.945（有 6 段 0.563 · 没有 3 段 -0.382）
- V-ONSET · `bed-restaurant`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.131（有 4 段 0.470 · 没有 6 段 0.339）
- V-SIM-GATE · `music`（pattern）：差 +0.076（有 6 段 0.422 · 没有 4 段 0.346）
- V-SIM-GATE · `dishes`（event）：差 +0.031（有 4 段 0.410 · 没有 6 段 0.379）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.523（有 6 段 2.238 · 没有 4 段 1.715）
- V-PUMPV · `dishes`（event）：差 +0.109（有 4 段 2.094 · 没有 6 段 1.985）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `music`（pattern）：差 +1.031（有 6 段 4.933 · 没有 4 段 3.901）
- V-TIMBRE · `dishes`（event）：差 +0.636（有 4 段 4.902 · 没有 6 段 4.266）
- V-TIMBRE · `talker`（speech）：差 +0.135（有 6 段 4.575 · 没有 4 段 4.439）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cafe-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.335 dB | -0.675（cafe-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 7 / 3 / 0 | 2.971 百分点 | 19.048（cafe-c05-ext） | 1.000 | 不过 |
| V-ONSET |  | 8 / 1 / 1 | 1.654 dB | 6.077（cafe-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.088 比例 | 0.211（cafe-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.158 dB | -3.704（cafe-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.032 dB² | 16.312（cafe-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 39.000 个/分钟 | 90.000（cafe-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.017 Δ | 0.004（cafe-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 4.892 dB | 10.058（cafe-c05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -2.836 dB | -1.541（cafe-c01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -2.286 dB | -1.186（cafe-c08-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.778 比例 | 1.000（cafe-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.999 dB | 2.858（cafe-c06-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.173 · Distill-MOS Δ +0.199 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.173 | +0.035（cafe-c02-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.199 | +0.121（cafe-c04-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +3.286（有 6 段 4.286 · 没有 4 段 1.000）
- V-EAT-ASR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.041（有 6 段 2.334 · 没有 3 段 0.293）
- V-ONSET · `coffee-machine`（pattern）：差 +1.269（有 4 段 2.359 · 没有 5 段 1.090）
- V-ONSET · `music`（pattern）：差 +1.132（有 6 段 2.031 · 没有 3 段 0.899）
- V-ONSET · `bed-restaurant`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.091（有 4 段 0.143 · 没有 6 段 0.051）
- V-SIM-GATE · `music`（pattern）：差 +0.069（有 6 段 0.115 · 没有 4 段 0.047）
- V-SIM-GATE · `dishes`（event）：差 +0.020（有 4 段 0.100 · 没有 6 段 0.080）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +2.964（有 6 段 7.217 · 没有 4 段 4.254）
- V-PUMPV · `dishes`（event）：差 +0.256（有 4 段 6.186 · 没有 6 段 5.929）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`（event）：差 +1.264（有 4 段 5.651 · 没有 6 段 4.387）
- V-TIMBRE · `music`（pattern）：差 +0.761（有 6 段 5.197 · 没有 4 段 4.436）
- V-TIMBRE · `talker`（speech）：差 +0.549（有 6 段 5.112 · 没有 4 段 4.563）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `coffee-machine`（pattern）：差 +0.918（有 4 段 -2.285 · 没有 6 段 -3.203）
- V-NR · `music`（pattern）：差 +0.716（有 6 段 -2.549 · 没有 4 段 -3.265）
- V-NR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cafe-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.543 dB | -2.762（cafe-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.476 百分点 | 0.000（cafe-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 1.584 dB | 5.615（cafe-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.020 比例 | 0.048（cafe-c02-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.976 dB | -3.720（cafe-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 3.892 dB² | 12.134（cafe-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 36.500 个/分钟 | 80.000（cafe-c09-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.037 Δ | 0.013（cafe-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.579 dB | 3.363（cafe-c05-mba） | 6.000 | 过 |
| V-NR |  | 8 / 2 / 0 | -6.866 dB | -4.296（cafe-c05-mba） | -6.000 | 不过 |
| V-INTF |  | 1 / 3 / 6 | -5.751 dB | -2.774（cafe-c08-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.470 比例 | 1.000（cafe-c04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.726 dB | 7.204（cafe-c06-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.243 · Distill-MOS Δ +0.251 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.243 | +0.026（cafe-c03-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.251 | +0.180（cafe-c07-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +0.569（有 6 段 1.811 · 没有 4 段 1.243）
- V-ONSET · `coffee-machine`（pattern）：差 +0.559（有 4 段 1.919 · 没有 6 段 1.360）
- V-ONSET · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.008（有 4 段 0.025 · 没有 6 段 0.017）
- V-SIM-GATE · `talker`（speech）：差 +0.004（有 6 段 0.022 · 没有 4 段 0.018）
- V-SIM-GATE · `music`（pattern）：差 +0.003（有 6 段 0.021 · 没有 4 段 0.018）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +3.217（有 6 段 5.179 · 没有 4 段 1.962）
- V-PUMPV · `coffee-machine`（pattern）：差 +2.059（有 4 段 5.127 · 没有 6 段 3.068）
- V-PUMPV · `music`（pattern）：差 +1.526（有 6 段 4.502 · 没有 4 段 2.976）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（cafe-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（cafe-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（cafe-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.085 dB | 1.288（cafe-c02-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.381 比例 | 0.605（cafe-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -12.643 dB | -8.064（cafe-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.995 dB² | 9.437（cafe-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（cafe-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（cafe-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.605 dB | 8.633（cafe-c04-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（cafe-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（cafe-c02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 2 / 1 / 7 | 0.180 比例 | 0.541（cafe-c04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.059 dB | 0.154（cafe-c08-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（cafe-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（cafe-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.092（有 4 段 0.436 · 没有 6 段 0.344）
- V-SIM-GATE · `music`（pattern）：差 +0.070（有 6 段 0.409 · 没有 4 段 0.339）
- V-SIM-GATE · `dishes`（event）：差 +0.052（有 4 段 0.412 · 没有 6 段 0.360）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +2.617（有 6 段 3.042 · 没有 4 段 0.425）
- V-PUMPV · `music`（pattern）：差 +2.017（有 6 段 2.802 · 没有 4 段 0.785）
- V-PUMPV · `coffee-machine`（pattern）：差 +1.913（有 4 段 3.143 · 没有 6 段 1.230）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `music`（pattern）：差 +1.948（有 6 段 4.384 · 没有 4 段 2.436）
- V-TIMBRE · `talker`（speech）：差 +1.161（有 6 段 4.069 · 没有 4 段 2.909）
- V-TIMBRE · `dishes`（event）：差 +0.001（有 4 段 3.606 · 没有 6 段 3.604）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cafe-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.351 dB | -0.648（cafe-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | 0.288 百分点 | 12.000（cafe-c09-mba） | 1.000 | 不过 |
| V-ONSET |  | 8 / 2 / 0 | 1.678 dB | 4.159（cafe-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.079 比例 | 0.186（cafe-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.961 dB | -4.378（cafe-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.067 dB² | 10.452（cafe-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 31.000 个/分钟 | 60.000（cafe-c09-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.018 Δ | 0.002（cafe-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.515 dB | 8.885（cafe-c08-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -2.910 dB | -1.730（cafe-c01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -2.338 dB | -1.261（cafe-c08-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.991 比例 | 1.000（cafe-c02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.207 dB | 3.355（cafe-c01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.210 · Distill-MOS Δ +0.195 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.210 | +0.022（cafe-c03-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.195 | +0.106（cafe-c04-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `dishes`（event）：差 +5.909（有 4 段 3.833 · 没有 6 段 -2.076）
- V-EAT-ASR · `music`（pattern）：差 +2.464（有 6 段 1.274 · 没有 4 段 -1.190）
- V-EAT-ASR · `coffee-machine`（pattern）：差 +1.315（有 4 段 1.077 · 没有 6 段 -0.238）
- V-EAT-ASR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.431（有 6 段 1.851 · 没有 4 段 1.420）
- V-ONSET · `coffee-machine`（pattern）：差 +0.124（有 4 段 1.753 · 没有 6 段 1.629）
- V-ONSET · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.089（有 4 段 0.132 · 没有 6 段 0.043）
- V-SIM-GATE · `music`（pattern）：差 +0.063（有 6 段 0.104 · 没有 4 段 0.041）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +3.126（有 6 段 5.318 · 没有 4 段 2.191）
- V-PUMPV · `music`（pattern）：差 +2.189（有 6 段 4.943 · 没有 4 段 2.754）
- V-PUMPV · `coffee-machine`（pattern）：差 +1.711（有 4 段 5.094 · 没有 6 段 3.383）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `music`（pattern）：差 +1.546（有 6 段 4.133 · 没有 4 段 2.588）
- V-TIMBRE · `talker`（speech）：差 +1.115（有 6 段 3.961 · 没有 4 段 2.846）
- V-TIMBRE · `dishes`（event）：差 +0.989（有 4 段 4.108 · 没有 6 段 3.119）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `coffee-machine`（pattern）：差 +0.999（有 4 段 -2.311 · 没有 6 段 -3.310）
- V-NR · `music`（pattern）：差 +0.611（有 6 段 -2.666 · 没有 4 段 -3.277）
- V-NR · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cafe-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.335 dB | -3.720（cafe-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 3.379 百分点 | 13.953（cafe-s02-ext） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.835 dB | 7.106（cafe-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.047 比例 | 0.118（cafe-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.888 dB | -10.279（cafe-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.055 dB² | 21.829（cafe-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 60.833 个/分钟 | 95.000（cafe-s05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.017 Δ | 0.002（cafe-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 4.418 dB | 7.577（cafe-s00-ext） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -5.924 dB | -4.874（cafe-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -3.262 dB | -2.423（cafe-s01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.881 比例 | 1.000（cafe-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.485 dB | 5.026（cafe-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.052 · Distill-MOS Δ +0.183 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.052 | -0.070（cafe-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.183 | +0.124（cafe-s00-ext） |

送人耳：
- cafe-s01-ext：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.19

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +2.975（有 3 段 4.323 · 没有 3 段 1.348）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.005（有 3 段 0.049 · 没有 3 段 0.044）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cafe · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（cafe-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（cafe-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（cafe-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.637 dB | 4.138（cafe-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.670 比例 | 0.825（cafe-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -17.858 dB | -14.713（cafe-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.828 dB² | 21.693（cafe-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（cafe-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（cafe-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.461 dB | 9.703（cafe-s04-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（cafe-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | 0.000 dB | 0.000（cafe-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.514 比例 | 0.792（cafe-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.047 dB | 0.224（cafe-s05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（cafe-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（cafe-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `music`（pattern）：差 +0.944（有 3 段 1.109 · 没有 3 段 0.165）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.016（有 3 段 0.678 · 没有 3 段 0.662）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cafe · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cafe-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.443 dB | -0.664（cafe-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 1 / 5 / 0 | 2.912 百分点 | 8.000（cafe-s05-ext） | 1.000 | 不过 |
| V-ONSET |  | 3 / 3 / 0 | 3.635 dB | 7.697（cafe-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.157 比例 | 0.312（cafe-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.978 dB | -11.246（cafe-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.117 dB² | 31.124（cafe-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 60.000 个/分钟 | 90.000（cafe-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.010 Δ | -0.007（cafe-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.449 dB | 9.843（cafe-s04-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.954 dB | -2.232（cafe-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -1.026 dB | -0.403（cafe-s03-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.950 比例 | 1.000（cafe-s04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.554 dB | 2.790（cafe-s03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.093 · Distill-MOS Δ +0.198 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.093 | -0.110（cafe-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.198 | +0.137（cafe-s05-ext） |

送人耳：
- cafe-s01-ext：一个说好一个说差：SCOREQ Δ -0.11 · Distill-MOS Δ +0.19

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `music`（pattern）：差 +3.939（有 3 段 4.882 · 没有 3 段 0.942）
- V-EAT-ASR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +1.228（有 3 段 4.249 · 没有 3 段 3.021）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.175（有 3 段 0.244 · 没有 3 段 0.070）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cafe · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cafe-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.269 dB | -3.503（cafe-s02-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | -0.392 百分点 | 4.762（cafe-s00-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.097 dB | 4.282（cafe-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.033 比例 | 0.059（cafe-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.294 dB | -9.798（cafe-s05-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.363 dB² | 34.490（cafe-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 40.833 个/分钟 | 75.000（cafe-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.020 Δ | 0.005（cafe-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 3.579 dB | 8.291（cafe-s00-mba） | 6.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -6.245 dB | -4.878（cafe-s02-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -3.333 dB | -2.437（cafe-s03-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.953 比例 | 1.000（cafe-s03-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.504 dB | 5.674（cafe-s04-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.041 · Distill-MOS Δ +0.172 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.041 | -0.039（cafe-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.172 | +0.131（cafe-s00-mba） |

送人耳：
- cafe-s01-mba：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.19
- cafe-s02-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.15

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +1.981（有 3 段 3.088 · 没有 3 段 1.107）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.013（有 3 段 0.040 · 没有 3 段 0.027）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cafe · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（cafe-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（cafe-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（cafe-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.073 dB | 1.752（cafe-s05-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.666 比例 | 0.812（cafe-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -17.387 dB | -14.165（cafe-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.429 dB² | 24.921（cafe-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（cafe-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（cafe-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.315 dB | 9.635（cafe-s00-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（cafe-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | 0.000 dB | 0.000（cafe-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.598 比例 | 0.667（cafe-s04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.021 dB | 0.058（cafe-s04-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（cafe-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（cafe-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `music`（pattern）：差 +0.013（有 3 段 0.672 · 没有 3 段 0.659）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cafe · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cafe-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.432 dB | -0.746（cafe-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 1 / 5 / 0 | 3.882 百分点 | 8.000（cafe-s05-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.166 dB | 3.543（cafe-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.131 比例 | 0.298（cafe-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.385 dB | -10.641（cafe-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.135 dB² | 30.580（cafe-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 48.333 个/分钟 | 70.000（cafe-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.007 Δ | -0.008（cafe-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.216 dB | 9.839（cafe-s00-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -3.276 dB | -2.068（cafe-s01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -1.187 dB | -0.383（cafe-s03-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.973 比例 | 1.000（cafe-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.395 dB | 2.781（cafe-s04-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.079 · Distill-MOS Δ +0.194 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.079 | -0.035（cafe-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.194 | +0.139（cafe-s05-mba） |

送人耳：
- cafe-s01-mba：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.18
- cafe-s02-mba：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.25

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +0.721（有 3 段 2.527 · 没有 3 段 1.806）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.160（有 3 段 0.212 · 没有 3 段 0.051）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `music`（pattern）：差 +2.549（有 3 段 16.410 · 没有 3 段 13.861）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `music`（pattern）：差 +0.427（有 3 段 -3.063 · 没有 3 段 -3.490）
- V-NR · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### canteen · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（canteen-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 8 / 2 / 0 | -2.554 dB | -4.168（canteen-c09-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -2.225 百分点 | 3.333（canteen-c00-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 3 / 2 | 2.505 dB | 4.758（canteen-c03-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.029 比例 | 0.069（canteen-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -10.528 dB | -5.980（canteen-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 8.370 dB² | 44.124（canteen-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 27.500 个/分钟 | 55.000（canteen-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.024 Δ | 0.007（canteen-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.761 dB | 18.211（canteen-c09-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 9 / 0 | -5.105 dB | -3.626（canteen-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -4.060 dB | -2.952（canteen-c07-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.786 比例 | 0.885（canteen-c09-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.087 dB | 5.282（canteen-c04-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 0 / 3 | SCOREQ Δ +0.085 · Distill-MOS Δ +0.197 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.085 | -0.044（canteen-c06-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.197 | +0.047（canteen-c03-ext） |

送人耳：
- canteen-c02-ext：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.29
- canteen-c03-ext：两个评分模型都说不比原声差，但 Δ 相差 0.34 ≥ 0.3
- canteen-c05-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.17
- canteen-c06-ext：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.18

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `babble`（speech）：差 +0.428（有 7 段 -2.683 · 没有 3 段 -2.255）
- V-SPLVL · `bed-cafeteria`（bed）：差 +0.257（有 3 段 -2.734 · 没有 7 段 -2.477）
- V-SPLVL · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cafeteria`（bed）：差 +4.766（有 3 段 1.111 · 没有 7 段 -3.655）
- V-EAT-ASR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +2.393（有 5 段 3.403 · 没有 3 段 1.009）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.020（有 3 段 0.044 · 没有 7 段 0.023）
- V-SIM-GATE · `babble`（speech）：差 +0.015（有 7 段 0.034 · 没有 3 段 0.019）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +4.465（有 7 段 9.709 · 没有 3 段 5.244）
- V-PUMPV · `bed-restaurant`（bed）：差 +4.206（有 7 段 9.631 · 没有 3 段 5.426）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +3.948（有 3 段 6.525 · 没有 7 段 2.577）
- V-TIMBRE · `talker`（speech）：差 +3.746（有 6 段 5.260 · 没有 4 段 1.514）
- V-TIMBRE · `babble`（speech）：差 +0.549（有 7 段 3.926 · 没有 3 段 3.377）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.209（有 3 段 -4.259 · 没有 7 段 -5.468）
- V-NR · `babble`（speech）：差 +0.378（有 7 段 -4.992 · 没有 3 段 -5.370）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（canteen-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（canteen-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（canteen-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 7 / 1 / 2 | 1.162 dB | 3.686（canteen-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.672 比例 | 0.789（canteen-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -15.394 dB | -10.049（canteen-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 5.626 dB² | 31.784（canteen-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（canteen-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（canteen-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 5.281 dB | 18.119（canteen-c09-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（canteen-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（canteen-c01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.174 比例 | 0.217（canteen-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.048 dB | 0.139（canteen-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（canteen-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（canteen-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-restaurant`（bed）：差 +1.836（有 5 段 1.851 · 没有 3 段 0.015）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.069（有 7 段 0.692 · 没有 3 段 0.623）
- V-SIM-GATE · `babble`（speech）：差 +0.065（有 7 段 0.691 · 没有 3 段 0.626）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +3.832（有 7 段 6.776 · 没有 3 段 2.944）
- V-PUMPV · `bed-restaurant`（bed）：差 +2.355（有 7 段 6.333 · 没有 3 段 3.978）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +3.843（有 6 段 6.818 · 没有 4 段 2.975）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +3.422（有 3 段 7.677 · 没有 7 段 4.254）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（canteen-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.567 dB | -1.009（canteen-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 4 / 0 | 3.830 百分点 | 22.222（canteen-c04-ext） | 1.000 | 不过 |
| V-ONSET |  | 4 / 4 / 2 | 3.067 dB | 4.693（canteen-c01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.096 比例 | 0.187（canteen-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -11.595 dB | -6.948（canteen-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 9.380 dB² | 44.272（canteen-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 33.500 个/分钟 | 65.000（canteen-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.019 Δ | 0.010（canteen-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 5.616 dB | 18.655（canteen-c09-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -2.761 dB | -1.346（canteen-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -1.828 dB | -1.052（canteen-c01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.748 比例 | 0.984（canteen-c09-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.136 dB | 2.280（canteen-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 0 / 3 | SCOREQ Δ +0.087 · Distill-MOS Δ +0.211 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.087 | -0.031（canteen-c06-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.211 | +0.114（canteen-c00-ext） |

送人耳：
- canteen-c00-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.11
- canteen-c02-ext：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.28
- canteen-c06-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.22

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-cafeteria`（bed）：差 +6.093（有 3 段 8.095 · 没有 7 段 2.002）
- V-EAT-ASR · `talker`（speech）：差 +3.900（有 6 段 5.390 · 没有 4 段 1.490）
- V-EAT-ASR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +1.565（有 5 段 3.654 · 没有 3 段 2.089）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.044（有 6 段 0.113 · 没有 4 段 0.070）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.007（有 7 段 0.098 · 没有 3 段 0.091）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +4.467（有 7 段 10.720 · 没有 3 段 6.253）
- V-PUMPV · `bed-restaurant`（bed）：差 +3.145（有 7 段 10.323 · 没有 3 段 7.179）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +4.051（有 6 段 7.237 · 没有 4 段 3.186）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +3.922（有 3 段 8.362 · 没有 7 段 4.440）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.316（有 3 段 -1.839 · 没有 7 段 -3.155）
- V-NR · `babble`（speech）：差 +0.874（有 7 段 -2.498 · 没有 3 段 -3.372）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（canteen-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 7 / 3 / 0 | -2.812 dB | -5.559（canteen-c06-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 6 / 4 / 0 | 2.841 百分点 | 14.815（canteen-c04-mba） | 1.000 | 不过 |
| V-ONSET |  | 8 / 1 / 1 | 1.668 dB | 4.803（canteen-c03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.028 比例 | 0.087（canteen-c06-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -10.568 dB | -6.403（canteen-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 7.009 dB² | 35.904（canteen-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 25.500 个/分钟 | 35.000（canteen-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.022 Δ | 0.006（canteen-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.831 dB | 17.168（canteen-c09-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 7 / 0 | -5.663 dB | -4.363（canteen-c07-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -4.286 dB | -3.064（canteen-c07-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.696 比例 | 0.868（canteen-c09-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.346 dB | 5.454（canteen-c04-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 0 / 3 | SCOREQ Δ +0.058 · Distill-MOS Δ +0.182 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.058 | -0.102（canteen-c04-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.182 | +0.032（canteen-c06-mba） |

送人耳：
- canteen-c02-mba：两个评分模型都说不比原声差，但 Δ 相差 0.32 ≥ 0.3
- canteen-c04-mba：一个说好一个说差：SCOREQ Δ -0.10 · Distill-MOS Δ +0.26
- canteen-c05-mba：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.18
- canteen-c06-mba：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.03

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `talker`（speech）：差 +0.388（有 6 段 -2.967 · 没有 4 段 -2.579）
- V-SPLVL · `bed-restaurant`（bed）：差 +0.241（有 7 段 -2.884 · 没有 3 段 -2.643）
- V-SPLVL · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cafeteria`（bed）：差 +5.136（有 3 段 6.436 · 没有 7 段 1.300）
- V-EAT-ASR · `talker`（speech）：差 +2.949（有 6 段 4.020 · 没有 4 段 1.071）
- V-EAT-ASR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +1.924（有 6 段 2.309 · 没有 3 段 0.386）
- V-ONSET · `babble`（speech）：差 +0.514（有 6 段 1.840 · 没有 3 段 1.325）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.009（有 7 段 0.031 · 没有 3 段 0.022）
- V-SIM-GATE · `talker`（speech）：差 +0.007（有 6 段 0.031 · 没有 4 段 0.024）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +5.088（有 7 段 8.535 · 没有 3 段 3.448）
- V-PUMPV · `bed-restaurant`（bed）：差 +3.431（有 7 段 8.038 · 没有 3 段 4.607）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +3.702（有 6 段 5.312 · 没有 4 段 1.610）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +3.549（有 3 段 6.316 · 没有 7 段 2.766）
- V-TIMBRE · `babble`（speech）：差 +0.131（有 7 段 3.870 · 没有 3 段 3.740）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.444（有 3 段 -4.652 · 没有 7 段 -6.096）
- V-NR · `babble`（speech）：差 +0.794（有 7 段 -5.425 · 没有 3 段 -6.219）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（canteen-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（canteen-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（canteen-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 9 / 0 / 1 | 0.209 dB | 2.578（canteen-c03-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.635 比例 | 0.761（canteen-c02-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -15.591 dB | -10.958（canteen-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 3.732 dB² | 15.924（canteen-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（canteen-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（canteen-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 4.824 dB | 17.191（canteen-c09-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（canteen-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（canteen-c01-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.168 比例 | 0.213（canteen-c04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.044 dB | 0.096（canteen-c01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（canteen-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（canteen-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `babble`（speech）：差 +0.085（有 7 段 0.660 · 没有 3 段 0.575）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.070（有 7 段 0.655 · 没有 3 段 0.586）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +2.413（有 7 段 4.456 · 没有 3 段 2.043）
- V-PUMPV · `bed-restaurant`（bed）：差 +0.429（有 7 段 3.861 · 没有 3 段 3.432）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +4.524（有 3 段 7.991 · 没有 7 段 3.467）
- V-TIMBRE · `talker`（speech）：差 +2.739（有 6 段 5.919 · 没有 4 段 3.180）
- V-TIMBRE · `babble`（speech）：差 +0.401（有 7 段 4.944 · 没有 3 段 4.543）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（canteen-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.549 dB | -0.936（canteen-c02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 6 / 0 | 7.844 百分点 | 37.037（canteen-c04-mba） | 1.000 | 不过 |
| V-ONSET |  | 7 / 2 / 1 | 2.065 dB | 4.383（canteen-c03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.090 比例 | 0.149（canteen-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -11.799 dB | -7.709（canteen-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.041 dB² | 13.551（canteen-c00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 31.000 个/分钟 | 45.000（canteen-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.016 Δ | 0.005（canteen-c06-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 5.321 dB | 17.767（canteen-c09-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -2.731 dB | -1.354（canteen-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -1.761 dB | -0.940（canteen-c07-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.729 比例 | 0.926（canteen-c09-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.626 dB | 3.130（canteen-c01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.084 · Distill-MOS Δ +0.203 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.084 | -0.073（canteen-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.203 | +0.087（canteen-c00-mba） |

送人耳：
- canteen-c04-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.27
- canteen-c05-mba：两个评分模型都说不比原声差，但 Δ 相差 0.32 ≥ 0.3
- canteen-c06-mba：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.13

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +7.518（有 6 段 10.851 · 没有 4 段 3.333）
- V-EAT-ASR · `bed-cafeteria`（bed）：差 +1.740（有 3 段 9.062 · 没有 7 段 7.322）
- V-EAT-ASR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +1.284（有 6 段 2.493 · 没有 3 段 1.209）
- V-ONSET · `babble`（speech）：差 +0.575（有 6 段 2.257 · 没有 3 段 1.682）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.032（有 6 段 0.103 · 没有 4 段 0.070）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.004（有 3 段 0.093 · 没有 7 段 0.088）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +2.478（有 7 段 6.785 · 没有 3 段 4.307）
- V-PUMPV · `bed-cafeteria`（bed）：差 +1.912（有 3 段 7.380 · 没有 7 段 5.468）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +4.814（有 3 段 8.691 · 没有 7 段 3.877）
- V-TIMBRE · `talker`（speech）：差 +3.397（有 6 段 6.680 · 没有 4 段 3.283）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.405（有 3 段 -1.747 · 没有 7 段 -3.153）
- V-NR · `babble`（speech）：差 +0.888（有 7 段 -2.465 · 没有 3 段 -3.352）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（canteen-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.386 dB | -4.243（canteen-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 1.724 百分点 | 22.727（canteen-s04-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 1.589 dB | 5.598（canteen-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.061 比例 | 0.133（canteen-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.679 dB | -2.010（canteen-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.188 dB² | 26.777（canteen-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 50.833 个/分钟 | 90.000（canteen-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.008 Δ | 0.001（canteen-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.226 dB | 7.010（canteen-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -4.624 dB | -4.114（canteen-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.830 dB | -3.090（canteen-s02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 2 / 4 | 0.913 比例 | 1.000（canteen-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.043 dB | 2.449（canteen-s02-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.007 · Distill-MOS Δ +0.171 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.007 | -0.156（canteen-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.171 | +0.046（canteen-s04-ext） |

送人耳：
- canteen-s00-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.11
- canteen-s05-ext：一个说好一个说差：SCOREQ Δ -0.16 · Distill-MOS Δ +0.24

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`（bed）：差 +0.541（有 3 段 -3.657 · 没有 3 段 -3.116）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-restaurant`（bed）：差 +14.735（有 3 段 9.091 · 没有 3 段 -5.644）
- V-EAT-ASR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +0.692（有 3 段 1.935 · 没有 3 段 1.243）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.070（有 3 段 0.096 · 没有 3 段 0.026）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +14.141（有 3 段 22.258 · 没有 3 段 8.118）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +2.706（有 3 段 4.579 · 没有 3 段 1.873）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +0.244（有 3 段 -4.502 · 没有 3 段 -4.746）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（canteen-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（canteen-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（canteen-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.246 dB | 4.182（canteen-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.830 比例 | 1.000（canteen-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -18.744 dB | -6.661（canteen-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 10.675 dB² | 17.320（canteen-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（canteen-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（canteen-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.805 dB | 8.488（canteen-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（canteen-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | 0.000 dB | 0.000（canteen-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 2 / 0 / 4 | 0.000 比例 | 0.000（canteen-s00-ext） | 0.100 | 过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.030 dB | 0.049（canteen-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（canteen-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（canteen-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-restaurant`（bed）：差 +0.087（有 3 段 0.290 · 没有 3 段 0.203）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.044（有 3 段 0.852 · 没有 3 段 0.808）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +7.100（有 3 段 14.225 · 没有 3 段 7.125）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +1.953（有 3 段 4.782 · 没有 3 段 2.828）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（canteen-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.783 dB | -1.450（canteen-s03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 2 / 4 / 0 | 1.947 百分点 | 13.636（canteen-s04-ext） | 1.000 | 不过 |
| V-ONSET |  | 2 / 4 / 0 | 2.906 dB | 7.001（canteen-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.165 比例 | 0.278（canteen-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -14.960 dB | -3.726（canteen-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.032 dB² | 21.288（canteen-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 45.833 个/分钟 | 65.000（canteen-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.005 Δ | -0.001（canteen-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 4.241 dB | 8.262（canteen-s02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -1.909 dB | -1.337（canteen-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -1.699 dB | -1.502（canteen-s02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 2 / 4 | 0.783 比例 | 1.000（canteen-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.812 dB | 1.375（canteen-s02-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 3 / 0 / 3 | SCOREQ Δ +0.004 · Distill-MOS Δ +0.193 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 3 / 3 / 0 | +0.004 | -0.159（canteen-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.193 | +0.109（canteen-s03-ext） |

送人耳：
- canteen-s00-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.18
- canteen-s03-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.11
- canteen-s05-ext：一个说好一个说差：SCOREQ Δ -0.16 · Distill-MOS Δ +0.20

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-restaurant`（bed）：差 +11.257（有 3 段 7.576 · 没有 3 段 -3.682）
- V-EAT-ASR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +0.916（有 3 段 3.364 · 没有 3 段 2.448）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.021（有 3 段 0.176 · 没有 3 段 0.155）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +10.523（有 3 段 20.294 · 没有 3 段 9.771）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +0.671（有 3 段 4.576 · 没有 3 段 3.905）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +0.913（有 3 段 -1.453 · 没有 3 段 -2.366）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（canteen-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 1 / 5 / 0 | -3.265 dB | -3.830（canteen-s04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | -1.701 百分点 | 12.500（canteen-s05-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 1.500 dB | 6.401（canteen-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.036 比例 | 0.066（canteen-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.529 dB | -3.623（canteen-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 31.140 dB² | 131.596（canteen-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 39.167 个/分钟 | 60.000（canteen-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.009 Δ | 0.004（canteen-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.387 dB | 6.043（canteen-s02-mba） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -5.050 dB | -4.267（canteen-s02-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -4.159 dB | -3.251（canteen-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 1 / 5 | 0.931 比例 | 0.931（canteen-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.426 dB | 2.624（canteen-s04-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 3 / 0 / 3 | SCOREQ Δ -0.002 · Distill-MOS Δ +0.153 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 3 / 3 / 0 | -0.002 | -0.078（canteen-s04-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.153 | +0.050（canteen-s04-mba） |

送人耳：
- canteen-s03-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.16
- canteen-s04-mba：一个说好一个说差：SCOREQ Δ -0.08 · Distill-MOS Δ +0.05
- canteen-s05-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.18

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`（bed）：差 +0.102（有 3 段 -3.316 · 没有 3 段 -3.214）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-restaurant`（bed）：差 +3.257（有 3 段 -0.072 · 没有 3 段 -3.329）
- V-EAT-ASR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-cafeteria`（bed）：差 +1.213（有 3 段 2.106 · 没有 3 段 0.893）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.021（有 3 段 0.047 · 没有 3 段 0.026）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +39.846（有 3 段 51.063 · 没有 3 段 11.217）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +2.664（有 3 段 4.719 · 没有 3 段 2.055）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +0.593（有 3 段 -4.754 · 没有 3 段 -5.347）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（canteen-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（canteen-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（canteen-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.084 dB | 4.987（canteen-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.812 比例 | 1.000（canteen-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -18.706 dB | -8.604（canteen-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 19.851 dB² | 45.660（canteen-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（canteen-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（canteen-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.348 dB | 7.796（canteen-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（canteen-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | 0.000 dB | 0.000（canteen-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 0 / 5 | 0.000 比例 | 0.000（canteen-s00-mba） | 0.100 | 过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.026 dB | 0.043（canteen-s02-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（canteen-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（canteen-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-cafeteria`（bed）：差 +1.674（有 3 段 0.921 · 没有 3 段 -0.753）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.018（有 3 段 0.821 · 没有 3 段 0.803）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +1.421（有 3 段 20.561 · 没有 3 段 19.140）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +1.346（有 3 段 4.021 · 没有 3 段 2.675）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（canteen-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.740 dB | -1.328（canteen-s04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | -1.903 百分点 | 12.500（canteen-s05-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 3 / 0 | 3.123 dB | 8.225（canteen-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.153 比例 | 0.216（canteen-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -14.856 dB | -5.291（canteen-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 17.337 dB² | 44.240（canteen-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 32.500 个/分钟 | 50.000（canteen-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.006 Δ | 0.003（canteen-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.793 dB | 8.084（canteen-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -1.833 dB | -1.149（canteen-s01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -1.975 dB | -1.770（canteen-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 1 / 5 | 0.655 比例 | 0.655（canteen-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.665 dB | 1.618（canteen-s02-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ -0.009 · Distill-MOS Δ +0.204 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | -0.009 | -0.088（canteen-s04-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.204 | +0.132（canteen-s04-mba） |

送人耳：
- canteen-s03-mba：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.14
- canteen-s04-mba：一个说好一个说差：SCOREQ Δ -0.09 · Distill-MOS Δ +0.13

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-cafeteria`（bed）：差 +14.520（有 3 段 5.357 · 没有 3 段 -9.163）
- V-EAT-ASR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-cafeteria`（bed）：差 +1.663（有 3 段 3.955 · 没有 3 段 2.292）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.041（有 3 段 0.173 · 没有 3 段 0.132）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +9.810（有 3 段 22.242 · 没有 3 段 12.432）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +0.446（有 3 段 4.016 · 没有 3 段 3.570）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.123（有 3 段 -1.272 · 没有 3 段 -2.394）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（transit-hub-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.598 dB | -2.608（transit-hub-c06-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 7 / 3 / 0 | -0.555 百分点 | 3.846（transit-hub-c07-ext） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.425 dB | 4.296（transit-hub-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.020 比例 | 0.039（transit-hub-c01-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.267 dB | -3.789（transit-hub-c06-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.457 dB² | 4.397（transit-hub-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 45.000 个/分钟 | 110.000（transit-hub-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.033 Δ | 0.018（transit-hub-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.808 dB | 6.296（transit-hub-c03-ext） | 6.000 | 不过 |
| V-NR |  | 4 / 6 / 0 | -5.436 dB | -3.635（transit-hub-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -4.834 dB | -4.516（transit-hub-c05-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.334 dB | 4.969（transit-hub-c00-ext） | 10.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.300 × | 2.675（transit-hub-c01-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.200 · Distill-MOS Δ +0.280 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.200 | -0.103（transit-hub-c03-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.280 | +0.178（transit-hub-c06-ext） |

送人耳：
- transit-hub-c03-ext：一个说好一个说差：SCOREQ Δ -0.10 · Distill-MOS Δ +0.39

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `babble`（speech）：差 +3.956（有 4 段 1.819 · 没有 6 段 -2.137）
- V-EAT-ASR · `footsteps`（event）：差 +3.956（有 4 段 1.819 · 没有 6 段 -2.137）
- V-EAT-ASR · `announce`（speech）：差 +0.792（有 3 段 0.000 · 没有 7 段 -0.792）
- V-ONSET · `bed-station`（bed）：差 +1.398（有 7 段 1.844 · 没有 3 段 0.446）
- V-SIM-GATE · `bed-station`（bed）：差 +0.012（有 7 段 0.023 · 没有 3 段 0.012）
- V-SIM-GATE · `talker`（speech）：差 +0.007（有 4 段 0.024 · 没有 6 段 0.017）
- V-PUMPV · `bed-station`（bed）：差 +0.977（有 7 段 2.750 · 没有 3 段 1.773）
- V-PUMPV · `footsteps`（event）：差 +0.663（有 4 段 2.855 · 没有 6 段 2.192）
- V-PUMPV · `talker`（speech）：差 +0.085（有 4 段 2.508 · 没有 6 段 2.423）
- V-TIMBRE · `bed-station`（bed）：差 +1.552（有 7 段 2.274 · 没有 3 段 0.722）
- V-TIMBRE · `babble`（speech）：差 +1.055（有 4 段 2.441 · 没有 6 段 1.386）
- V-TIMBRE · `footsteps`（event）：差 +0.276（有 4 段 1.974 · 没有 6 段 1.698）
- V-NR · `babble`（speech）：差 +1.573（有 4 段 -4.492 · 没有 6 段 -6.065）
- V-NR · `announce`（speech）：差 +0.105（有 3 段 -5.363 · 没有 7 段 -5.468）
- V-NR · `bed-ace`（bed）：差 +0.105（有 3 段 -5.363 · 没有 7 段 -5.468）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-station`（bed）：差 +0.455（有 7 段 1.436 · 没有 3 段 0.981）
- V-MUSIC · `talker`（speech）：差 +0.452（有 4 段 1.571 · 没有 6 段 1.119）
- V-MUSIC · `luggage`（event）：差 +0.029（有 5 段 1.314 · 没有 5 段 1.285）

### transit-hub · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（transit-hub-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.004 dB | -0.035（transit-hub-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（transit-hub-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.227 dB | 2.754（transit-hub-c04-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.507 比例 | 0.728（transit-hub-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -12.854 dB | -7.846（transit-hub-c06-ext） | 4.000 | 过 |
| V-PUMPV |  | 7 / 3 / 0 | 1.008 dB² | 2.489（transit-hub-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（transit-hub-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（transit-hub-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 2.258 dB | 8.660（transit-hub-c02-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（transit-hub-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（transit-hub-c01-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.124 dB | 0.286（transit-hub-c00-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（transit-hub-c00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（transit-hub-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（transit-hub-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-station`（bed）：差 +0.150（有 7 段 0.551 · 没有 3 段 0.402）
- V-SIM-GATE · `babble`（speech）：差 +0.111（有 4 段 0.573 · 没有 6 段 0.462）
- V-SIM-GATE · `talker`（speech）：差 +0.028（有 4 段 0.523 · 没有 6 段 0.495）
- V-PUMPV · `footsteps`（event）：差 +1.089（有 4 段 1.661 · 没有 6 段 0.572）
- V-PUMPV · `talker`（speech）：差 +0.215（有 4 段 1.137 · 没有 6 段 0.922）
- V-PUMPV · `babble`（speech）：差 +0.160（有 4 段 1.104 · 没有 6 段 0.944）
- V-TIMBRE · `bed-station`（bed）：差 +2.682（有 7 段 3.063 · 没有 3 段 0.381）
- V-TIMBRE · `talker`（speech）：差 +0.721（有 4 段 2.691 · 没有 6 段 1.970）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### transit-hub · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（transit-hub-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.380 dB | -1.048（transit-hub-c08-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.728 百分点 | 0.000（transit-hub-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 2.146 dB | 6.332（transit-hub-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.037 比例 | 0.087（transit-hub-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.848 dB | -5.140（transit-hub-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.991 dB² | 6.463（transit-hub-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 37.000 个/分钟 | 75.000（transit-hub-c03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.023 Δ | 0.004（transit-hub-c06-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 1.978 dB | 8.144（transit-hub-c04-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 9 / 0 | -3.659 dB | -0.490（transit-hub-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -2.608 dB | -0.452（transit-hub-c06-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.022 dB | 2.351（transit-hub-c04-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.187 × | 1.554（transit-hub-c02-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.139 · Distill-MOS Δ +0.233 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.139 | -0.095（transit-hub-c03-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.233 | +0.041（transit-hub-c06-ext） |

送人耳：
- transit-hub-c03-ext：一个说好一个说差：SCOREQ Δ -0.10 · Distill-MOS Δ +0.36

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-station`（bed）：差 +1.677（有 7 段 2.650 · 没有 3 段 0.972）
- V-SIM-GATE · `footsteps`（event）：差 +0.024（有 4 段 0.052 · 没有 6 段 0.028）
- V-SIM-GATE · `talker`（speech）：差 +0.023（有 4 段 0.051 · 没有 6 段 0.028）
- V-SIM-GATE · `luggage`（event）：差 +0.012（有 5 段 0.043 · 没有 5 段 0.031）
- V-PUMPV · `footsteps`（event）：差 +1.738（有 4 段 4.034 · 没有 6 段 2.296）
- V-PUMPV · `bed-station`（bed）：差 +0.910（有 7 段 3.264 · 没有 3 段 2.354）
- V-PUMPV · `babble`（speech）：差 +0.400（有 4 段 3.231 · 没有 6 段 2.831）
- V-TIMBRE · `bed-station`（bed）：差 +2.492（有 7 段 2.725 · 没有 3 段 0.233）
- V-TIMBRE · `babble`（speech）：差 +0.665（有 4 段 2.377 · 没有 6 段 1.712）
- V-TIMBRE · `footsteps`（event）：差 +0.199（有 4 段 2.097 · 没有 6 段 1.898）
- V-NR · `babble`（speech）：差 +2.104（有 4 段 -2.397 · 没有 6 段 -4.501）
- V-NR · `announce`（speech）：差 +2.048（有 3 段 -2.226 · 没有 7 段 -4.274）
- V-NR · `bed-ace`（bed）：差 +2.048（有 3 段 -2.226 · 没有 7 段 -4.274）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### transit-hub · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（transit-hub-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.850 dB | -2.813（transit-hub-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | 0.731 百分点 | 7.143（transit-hub-c09-mba） | 1.000 | 不过 |
| V-ONSET |  | 6 / 4 / 0 | 2.273 dB | 5.347（transit-hub-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.023 比例 | 0.055（transit-hub-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.612 dB | -4.661（transit-hub-c06-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.876 dB² | 12.551（transit-hub-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 29.500 个/分钟 | 45.000（transit-hub-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.032 Δ | 0.018（transit-hub-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.345 dB | 4.076（transit-hub-c02-mba） | 6.000 | 过 |
| V-NR |  | 7 / 3 / 0 | -6.757 dB | -4.292（transit-hub-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -5.562 dB | -5.162（transit-hub-c06-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.436 dB | 6.541（transit-hub-c05-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.627 × | 2.937（transit-hub-c01-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.267 · Distill-MOS Δ +0.302 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.267 | +0.030（transit-hub-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.302 | +0.220（transit-hub-c06-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `footsteps`（event）：差 +3.494（有 4 段 2.827 · 没有 6 段 -0.667）
- V-EAT-ASR · `announce`（speech）：差 +0.940（有 3 段 1.389 · 没有 7 段 0.449）
- V-EAT-ASR · `bed-ace`（bed）：差 +0.940（有 3 段 1.389 · 没有 7 段 0.449）
- V-ONSET · `bed-station`（bed）：差 +1.960（有 7 段 2.861 · 没有 3 段 0.900）
- V-SIM-GATE · `talker`（speech）：差 +0.008（有 4 段 0.028 · 没有 6 段 0.020）
- V-SIM-GATE · `bed-station`（bed）：差 +0.005（有 7 段 0.025 · 没有 3 段 0.020）
- V-PUMPV · `footsteps`（event）：差 +4.351（有 4 段 7.486 · 没有 6 段 3.135）
- V-PUMPV · `talker`（speech）：差 +1.283（有 4 段 5.646 · 没有 6 段 4.362）
- V-PUMPV · `announce`（speech）：差 +0.533（有 3 段 5.249 · 没有 7 段 4.716）
- V-NR · `babble`（speech）：差 +2.249（有 4 段 -5.408 · 没有 6 段 -7.657）
- V-NR · `announce`（speech）：差 +1.371（有 3 段 -5.797 · 没有 7 段 -7.168）
- V-NR · `bed-ace`（bed）：差 +1.371（有 3 段 -5.797 · 没有 7 段 -7.168）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.607（有 4 段 1.991 · 没有 6 段 1.384）
- V-MUSIC · `bed-station`（bed）：差 +0.199（有 7 段 1.686 · 没有 3 段 1.488）

### transit-hub · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（transit-hub-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.003 dB | -0.033（transit-hub-c02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（transit-hub-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.620 dB | 2.658（transit-hub-c04-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.591 比例 | 0.777（transit-hub-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -14.636 dB | -9.338（transit-hub-c06-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 2.279 dB² | 7.492（transit-hub-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（transit-hub-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（transit-hub-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.979 dB | 8.185（transit-hub-c02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（transit-hub-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（transit-hub-c01-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.110 dB | 0.288（transit-hub-c01-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（transit-hub-c00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（transit-hub-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（transit-hub-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-station`（bed）：差 +0.129（有 7 段 0.629 · 没有 3 段 0.500）
- V-SIM-GATE · `babble`（speech）：差 +0.084（有 4 段 0.641 · 没有 6 段 0.557）
- V-PUMPV · `footsteps`（event）：差 +2.913（有 4 段 4.027 · 没有 6 段 1.114）
- V-PUMPV · `announce`（speech）：差 +0.776（有 3 段 2.822 · 没有 7 段 2.047）
- V-PUMPV · `bed-ace`（bed）：差 +0.776（有 3 段 2.822 · 没有 7 段 2.047）
- V-TIMBRE · `bed-station`（bed）：差 +2.263（有 7 段 2.658 · 没有 3 段 0.395）
- V-TIMBRE · `talker`（speech）：差 +0.642（有 4 段 2.364 · 没有 6 段 1.722）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### transit-hub · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（transit-hub-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.581 dB | -1.331（transit-hub-c08-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -1.871 百分点 | 11.111（transit-hub-c04-mba） | 1.000 | 不过 |
| V-ONSET |  | 7 / 3 / 0 | 2.680 dB | 5.376（transit-hub-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.041 比例 | 0.087（transit-hub-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -10.159 dB | -5.401（transit-hub-c06-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.187 dB² | 9.982（transit-hub-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 34.000 个/分钟 | 60.000（transit-hub-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.026 Δ | 0.013（transit-hub-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 1.743 dB | 6.868（transit-hub-c04-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 7 / 0 | -4.473 dB | -1.829（transit-hub-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -3.070 dB | -1.329（transit-hub-c06-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.808 dB | 3.024（transit-hub-c01-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.246 × | 1.683（transit-hub-c02-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.214 · Distill-MOS Δ +0.277 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.214 | +0.065（transit-hub-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.277 | +0.141（transit-hub-c06-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `announce`（speech）：差 +2.673（有 3 段 0.000 · 没有 7 段 -2.673）
- V-EAT-ASR · `bed-ace`（bed）：差 +2.673（有 3 段 0.000 · 没有 7 段 -2.673）
- V-EAT-ASR · `pa-chime`（event）：差 +2.673（有 3 段 0.000 · 没有 7 段 -2.673）
- V-ONSET · `bed-station`（bed）：差 +1.536（有 7 段 3.140 · 没有 3 段 1.605）
- V-SIM-GATE · `talker`（speech）：差 +0.032（有 4 段 0.060 · 没有 6 段 0.029）
- V-SIM-GATE · `luggage`（event）：差 +0.024（有 5 段 0.053 · 没有 5 段 0.029）
- V-SIM-GATE · `announce`（speech）：差 +0.014（有 3 段 0.051 · 没有 7 段 0.037）
- V-PUMPV · `footsteps`（event）：差 +2.590（有 4 段 5.741 · 没有 6 段 3.151）
- V-PUMPV · `talker`（speech）：差 +0.962（有 4 段 4.764 · 没有 6 段 3.802）
- V-PUMPV · `announce`（speech）：差 +0.751（有 3 段 4.712 · 没有 7 段 3.961）
- V-TIMBRE · `bed-station`（bed）：差 +2.122（有 7 段 2.379 · 没有 3 段 0.258）
- V-TIMBRE · `babble`（speech）：差 +0.436（有 4 段 2.004 · 没有 6 段 1.569）
- V-TIMBRE · `footsteps`（event）：差 +0.024（有 4 段 1.757 · 没有 6 段 1.734）
- V-NR · `announce`（speech）：差 +2.163（有 3 段 -2.959 · 没有 7 段 -5.122）
- V-NR · `bed-ace`（bed）：差 +2.163（有 3 段 -2.959 · 没有 7 段 -5.122）
- V-NR · `pa-chime`（event）：差 +2.163（有 3 段 -2.959 · 没有 7 段 -5.122）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### transit-hub · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（transit-hub-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.732 dB | -3.743（transit-hub-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 6.700 百分点 | 21.875（transit-hub-s01-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 0.969 dB | 3.658（transit-hub-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.057 比例 | 0.153（transit-hub-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.776 dB | -8.212（transit-hub-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 10.232 dB² | 33.587（transit-hub-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 42.500 个/分钟 | 75.000（transit-hub-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.015 Δ | 0.007（transit-hub-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 4.976 dB | 9.942（transit-hub-s00-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -4.818 dB | -3.439（transit-hub-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.761 dB | -2.550（transit-hub-s02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.675 dB | 4.584（transit-hub-s02-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.151 × | 1.500（transit-hub-s02-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.073 · Distill-MOS Δ +0.230 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.073 | -0.029（transit-hub-s04-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.230 | +0.168（transit-hub-s05-ext） |

送人耳：
- transit-hub-s01-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.25
- transit-hub-s04-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.19

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +9.500（有 3 段 14.982 · 没有 3 段 5.482）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（transit-hub-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（transit-hub-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（transit-hub-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | -0.082 dB | 2.939（transit-hub-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.756 比例 | 0.925（transit-hub-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.569 dB | -12.503（transit-hub-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.366 dB² | 9.485（transit-hub-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（transit-hub-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（transit-hub-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 4.981 dB | 8.657（transit-hub-s00-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（transit-hub-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | 0.000 dB | 0.000（transit-hub-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.093 dB | 0.175（transit-hub-s04-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（transit-hub-s00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（transit-hub-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（transit-hub-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.612（有 3 段 6.172 · 没有 3 段 4.560）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（transit-hub-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.699 dB | -1.127（transit-hub-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 2 / 4 / 0 | 8.442 百分点 | 23.077（transit-hub-s00-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 2.236 dB | 7.751（transit-hub-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.102 比例 | 0.334（transit-hub-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.438 dB | -8.497（transit-hub-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 7.535 dB² | 11.340（transit-hub-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 35.000 个/分钟 | 55.000（transit-hub-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.005（transit-hub-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.124 dB | 9.716（transit-hub-s00-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.153 dB | -1.239（transit-hub-s04-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -1.959 dB | -1.549（transit-hub-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.209 dB | 3.286（transit-hub-s02-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.026 × | 1.246（transit-hub-s01-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.122 · Distill-MOS Δ +0.218 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.122 | -0.005（transit-hub-s04-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.218 | +0.176（transit-hub-s05-ext） |

送人耳：
- transit-hub-s04-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.21

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（transit-hub-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.675 dB | -3.463（transit-hub-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 3.855 百分点 | 18.750（transit-hub-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 5 / 1 / 0 | 2.066 dB | 6.195（transit-hub-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.029 比例 | 0.079（transit-hub-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.006 dB | -7.951（transit-hub-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 7.076 dB² | 14.658（transit-hub-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 43.333 个/分钟 | 65.000（transit-hub-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.015 Δ | 0.010（transit-hub-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.948 dB | 6.011（transit-hub-s00-mba） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -5.276 dB | -3.783（transit-hub-s01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.887 dB | -2.891（transit-hub-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.568 dB | 4.108（transit-hub-s04-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.272 × | 1.875（transit-hub-s02-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.053 · Distill-MOS Δ +0.226 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.053 | -0.062（transit-hub-s04-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.226 | +0.190（transit-hub-s05-mba） |

送人耳：
- transit-hub-s01-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.24
- transit-hub-s04-mba：一个说好一个说差：SCOREQ Δ -0.06 · Distill-MOS Δ +0.21

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（transit-hub-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（transit-hub-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（transit-hub-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 1.185 dB | 5.894（transit-hub-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.796 比例 | 0.913（transit-hub-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.984 dB | -12.424（transit-hub-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.524 dB² | 12.288（transit-hub-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（transit-hub-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（transit-hub-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 4.204 dB | 6.928（transit-hub-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（transit-hub-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | 0.000 dB | 0.000（transit-hub-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.055 dB | 0.150（transit-hub-s04-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（transit-hub-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（transit-hub-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（transit-hub-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（transit-hub-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.878 dB | -1.185（transit-hub-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 3.947 百分点 | 21.875（transit-hub-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.322 dB | 3.909（transit-hub-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.075 比例 | 0.249（transit-hub-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.660 dB | -8.169（transit-hub-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.838 dB² | 12.833（transit-hub-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 38.333 个/分钟 | 45.000（transit-hub-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.006（transit-hub-s05-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 5.386 dB | 7.309（transit-hub-s04-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.467 dB | -1.533（transit-hub-s04-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -2.157 dB | -1.599（transit-hub-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.799 dB | 1.505（transit-hub-s04-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.179 × | 1.567（transit-hub-s01-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.108 · Distill-MOS Δ +0.225 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.108 | -0.012（transit-hub-s04-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.225 | +0.201（transit-hub-s00-mba） |

送人耳：
- transit-hub-s01-mba：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.23
- transit-hub-s04-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.24

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### car-passenger · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（car-passenger-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.342 dB | -2.247（car-passenger-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -2.402 百分点 | 3.846（car-passenger-c07-ext） | 1.000 | 不过 |
| V-ONSET |  | 8 / 2 / 0 | 2.875 dB | 9.462（car-passenger-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.034 比例 | 0.119（car-passenger-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.137 dB | -5.074（car-passenger-c08-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.314 dB² | 13.467（car-passenger-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 26.500 个/分钟 | 60.000（car-passenger-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.009（car-passenger-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.051 dB | 4.133（car-passenger-c07-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -7.850 dB | -6.381（car-passenger-c00-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -6.641 dB | -4.513（car-passenger-c04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.504 dB | 5.976（car-passenger-c02-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.223 × | 6.087（car-passenger-c04-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.200 · Distill-MOS Δ +0.269 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.200 | -0.051（car-passenger-c06-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.269 | +0.122（car-passenger-c07-ext） |

送人耳：
- car-passenger-c01-ext：两个评分模型都说不比原声差，但 Δ 相差 0.31 ≥ 0.3
- car-passenger-c06-ext：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.25

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +5.387（有 7 段 -0.786 · 没有 3 段 -6.173）
- V-EAT-ASR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.791（有 7 段 3.712 · 没有 3 段 0.921）
- V-ONSET · `nav-voice`（speech）：差 +0.499（有 3 段 3.224 · 没有 7 段 2.725）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.000（有 7 段 0.034 · 没有 3 段 0.033）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.654（有 7 段 5.810 · 没有 3 段 4.156）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `nav-voice`（speech）：差 +1.280（有 3 段 3.119 · 没有 7 段 1.839）
- V-MUSIC · `talker`（speech）：差 +0.040（有 7 段 2.235 · 没有 3 段 2.195）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（car-passenger-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.002 dB | -0.018（car-passenger-c06-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（car-passenger-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 0.581 dB | 3.691（car-passenger-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.712 比例 | 0.885（car-passenger-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -14.306 dB | -9.902（car-passenger-c08-ext） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.820 dB² | 4.492（car-passenger-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（car-passenger-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（car-passenger-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.058 dB | 4.156（car-passenger-c07-ext） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（car-passenger-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（car-passenger-c00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.052 dB | 0.213（car-passenger-c08-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（car-passenger-c00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（car-passenger-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（car-passenger-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +0.900（有 7 段 0.851 · 没有 3 段 -0.049）
- V-ONSET · `nav-voice`（speech）：差 +0.827（有 3 段 1.160 · 没有 7 段 0.333）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.153（有 3 段 0.819 · 没有 7 段 0.666）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.921（有 7 段 2.096 · 没有 3 段 1.175）
- V-PUMPV · `nav-voice`（speech）：差 +0.664（有 3 段 2.284 · 没有 7 段 1.621）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）

### car-passenger · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（car-passenger-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.559 dB | -0.801（car-passenger-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -1.967 百分点 | 3.846（car-passenger-c02-ext） | 1.000 | 不过 |
| V-ONSET |  | 7 / 3 / 0 | 2.927 dB | 7.595（car-passenger-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.068 比例 | 0.131（car-passenger-c01-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.478 dB | -5.296（car-passenger-c08-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.528 dB² | 9.065（car-passenger-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 21.500 个/分钟 | 35.000（car-passenger-c01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.021 Δ | 0.009（car-passenger-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.624 dB | 2.185（car-passenger-c07-ext） | 6.000 | 过 |
| V-NR |  | 5 / 5 / 0 | -6.122 dB | -5.229（car-passenger-c08-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -5.052 dB | -3.635（car-passenger-c04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.473 dB | 3.826（car-passenger-c08-ext） | 10.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 1.932 × | 2.870（car-passenger-c05-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.188 · Distill-MOS Δ +0.233 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.188 | -0.032（car-passenger-c06-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.233 | +0.153（car-passenger-c07-ext） |

送人耳：
- car-passenger-c06-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.22

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +4.177（有 7 段 -0.714 · 没有 3 段 -4.891）
- V-EAT-ASR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.425（有 7 段 3.655 · 没有 3 段 1.230）
- V-ONSET · `nav-voice`（speech）：差 +0.922（有 3 段 3.573 · 没有 7 段 2.651）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.025（有 3 段 0.085 · 没有 7 段 0.060）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.404（有 7 段 4.949 · 没有 3 段 3.545）
- V-PUMPV · `nav-voice`（speech）：差 +1.105（有 3 段 5.301 · 没有 7 段 4.196）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `talker`（speech）：差 +0.359（有 7 段 -6.014 · 没有 3 段 -6.373）
- V-NR · `nav-voice`（speech）：差 +0.046（有 3 段 -6.090 · 没有 7 段 -6.136）
- V-NR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.115（有 7 段 1.967 · 没有 3 段 1.852）
- V-MUSIC · `nav-voice`（speech）：差 +0.111（有 3 段 2.010 · 没有 7 段 1.899）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（car-passenger-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.421 dB | -2.868（car-passenger-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -3.058 百分点 | 3.846（car-passenger-c02-mba） | 1.000 | 不过 |
| V-ONSET |  | 7 / 3 / 0 | 2.631 dB | 10.327（car-passenger-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.028 比例 | 0.099（car-passenger-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.119 dB | -5.713（car-passenger-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.258 dB² | 13.124（car-passenger-c09-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 24.000 个/分钟 | 50.000（car-passenger-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.023 Δ | 0.007（car-passenger-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.322 dB | 4.143（car-passenger-c07-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -8.526 dB | -6.384（car-passenger-c00-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -7.728 dB | -4.728（car-passenger-c04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.971 dB | 6.020（car-passenger-c02-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.920 × | 4.592（car-passenger-c04-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.224 · Distill-MOS Δ +0.265 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.224 | -0.145（car-passenger-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.265 | +0.136（car-passenger-c07-mba） |

送人耳：
- car-passenger-c06-mba：一个说好一个说差：SCOREQ Δ -0.14 · Distill-MOS Δ +0.21
- car-passenger-c07-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.14

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `nav-voice`（speech）：差 +2.605（有 3 段 -1.235 · 没有 7 段 -3.839）
- V-EAT-ASR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.437（有 7 段 3.362 · 没有 3 段 0.925）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.014（有 7 段 0.032 · 没有 3 段 0.018）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +1.712（有 3 段 6.456 · 没有 7 段 4.744）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `nav-voice`（speech）：差 +0.982（有 3 段 2.608 · 没有 7 段 1.626）
- V-MUSIC · `talker`（speech）：差 +0.171（有 7 段 1.972 · 没有 3 段 1.801）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（car-passenger-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.002（car-passenger-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（car-passenger-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 8 / 2 / 0 | 0.822 dB | 4.166（car-passenger-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.692 比例 | 0.845（car-passenger-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -14.252 dB | -10.961（car-passenger-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 2.049 dB² | 6.584（car-passenger-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（car-passenger-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（car-passenger-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.163 dB | 4.272（car-passenger-c07-mba） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（car-passenger-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | 0.000 dB | 0.000（car-passenger-c00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.049 dB | 0.230（car-passenger-c08-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（car-passenger-c00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（car-passenger-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（car-passenger-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +1.206（有 7 段 1.183 · 没有 3 段 -0.022）
- V-ONSET · `nav-voice`（speech）：差 +0.632（有 3 段 1.264 · 没有 7 段 0.632）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.118（有 3 段 0.775 · 没有 7 段 0.657）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +2.209（有 3 段 3.596 · 没有 7 段 1.386）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）

### car-passenger · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（car-passenger-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.508 dB | -0.732（car-passenger-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -1.640 百分点 | 3.846（car-passenger-c02-mba） | 1.000 | 不过 |
| V-ONSET |  | 7 / 3 / 0 | 3.116 dB | 9.020（car-passenger-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.057 比例 | 0.101（car-passenger-c01-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.475 dB | -6.263（car-passenger-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.590 dB² | 19.841（car-passenger-c09-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 23.000 个/分钟 | 40.000（car-passenger-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.018 Δ | 0.006（car-passenger-c06-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.711 dB | 2.233（car-passenger-c07-mba） | 6.000 | 过 |
| V-NR |  | 6 / 4 / 0 | -6.127 dB | -5.366（car-passenger-c08-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -5.126 dB | -3.728（car-passenger-c04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.623 dB | 3.836（car-passenger-c08-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.009 × | 2.976（car-passenger-c05-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.204 · Distill-MOS Δ +0.225 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.204 | -0.128（car-passenger-c06-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.225 | +0.139（car-passenger-c07-mba） |

送人耳：
- car-passenger-c06-mba：一个说好一个说差：SCOREQ Δ -0.13 · Distill-MOS Δ +0.19
- car-passenger-c07-mba：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.14

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `nav-voice`（speech）：差 +0.579（有 3 段 -1.235 · 没有 7 段 -1.813）
- V-EAT-ASR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.998（有 7 段 4.015 · 没有 3 段 1.017）
- V-ONSET · `nav-voice`（speech）：差 +1.025（有 3 段 3.834 · 没有 7 段 2.808）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.011（有 3 段 0.064 · 没有 7 段 0.053）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +1.279（有 3 段 6.485 · 没有 7 段 5.206）
- V-PUMPV · `talker`（speech）：差 +1.129（有 7 段 5.929 · 没有 3 段 4.799）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `talker`（speech）：差 +0.237（有 7 段 -6.056 · 没有 3 段 -6.292）
- V-NR · `nav-voice`（speech）：差 +0.154（有 3 段 -6.019 · 没有 7 段 -6.173）
- V-NR · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.096（有 7 段 2.038 · 没有 3 段 1.942）
- V-MUSIC · `nav-voice`（speech）：差 +0.032（有 3 段 2.031 · 没有 7 段 2.000）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（car-passenger-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -2.790 dB | -3.714（car-passenger-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 5.733 百分点 | 34.783（car-passenger-s02-ext） | 1.000 | 不过 |
| V-ONSET |  | 2 / 4 / 0 | 4.637 dB | 11.010（car-passenger-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.083 比例 | 0.157（car-passenger-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.726 dB | -14.254（car-passenger-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 18.252 dB² | 49.011（car-passenger-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 39.167 个/分钟 | 65.000（car-passenger-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.010（car-passenger-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.450 dB | 4.605（car-passenger-s01-ext） | 6.000 | 过 |
| V-NR |  | 5 / 1 / 0 | -8.376 dB | -5.822（car-passenger-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -5.427 dB | -4.905（car-passenger-s02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.486 dB | 5.584（car-passenger-s05-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.231 × | 2.108（car-passenger-s00-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.074 · Distill-MOS Δ +0.204 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.074 | +0.039（car-passenger-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.204 | +0.137（car-passenger-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `nav-voice`（speech）：差 +5.469（有 3 段 7.371 · 没有 3 段 1.902）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.051（有 3 段 0.109 · 没有 3 段 0.057）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +9.056（有 3 段 22.780 · 没有 3 段 13.724）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（car-passenger-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（car-passenger-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（car-passenger-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 4 / 2 / 0 | 1.909 dB | 6.078（car-passenger-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.981 比例 | 1.000（car-passenger-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -21.501 dB | -20.200（car-passenger-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.645 dB² | 29.423（car-passenger-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（car-passenger-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（car-passenger-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.929 dB | 6.796（car-passenger-s01-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（car-passenger-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（car-passenger-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.020 dB | 0.039（car-passenger-s02-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（car-passenger-s00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（car-passenger-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（car-passenger-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `nav-voice`（speech）：差 +3.877（有 3 段 3.848 · 没有 3 段 -0.030）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +4.766（有 3 段 15.028 · 没有 3 段 10.262）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `nav-voice`（speech）：差 +0.951（有 3 段 3.404 · 没有 3 段 2.453）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（car-passenger-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.242 dB | -1.904（car-passenger-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -10.881 百分点 | 0.000（car-passenger-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 2 / 4 / 0 | 4.813 dB | 8.956（car-passenger-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.144 比例 | 0.264（car-passenger-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -16.265 dB | -14.906（car-passenger-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 19.404 dB² | 59.291（car-passenger-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 50.833 个/分钟 | 75.000（car-passenger-s05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.015 Δ | 0.008（car-passenger-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 3.863 dB | 7.550（car-passenger-s05-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -5.441 dB | -4.162（car-passenger-s05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -5.260 dB | -3.544（car-passenger-s02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.773 dB | 3.615（car-passenger-s04-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.420 × | 2.421（car-passenger-s04-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.157 · Distill-MOS Δ +0.242 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.157 | +0.055（car-passenger-s03-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.242 | +0.138（car-passenger-s03-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `nav-voice`（speech）：差 +3.729（有 3 段 6.678 · 没有 3 段 2.948）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.003（有 3 段 0.146 · 没有 3 段 0.143）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +16.288（有 3 段 27.548 · 没有 3 段 11.260）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `nav-voice`（speech）：差 +1.107（有 3 段 4.416 · 没有 3 段 3.310）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `nav-voice`（speech）：差 +0.460（有 3 段 1.650 · 没有 3 段 1.190）
- V-MUSIC · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（car-passenger-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -2.867 dB | -4.176（car-passenger-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | -1.114 百分点 | 7.143（car-passenger-s04-mba） | 1.000 | 不过 |
| V-ONSET |  | 2 / 4 / 0 | 4.028 dB | 6.266（car-passenger-s04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.084 比例 | 0.165（car-passenger-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.007 dB | -13.298（car-passenger-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.502 dB² | 15.461（car-passenger-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 41.667 个/分钟 | 65.000（car-passenger-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.009（car-passenger-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 4.272 dB | 7.856（car-passenger-s03-mba） | 6.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -7.921 dB | -5.812（car-passenger-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -5.391 dB | -5.010（car-passenger-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 5 / 0 / 1 | 4.443 dB | 5.795（car-passenger-s02-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.283 × | 2.198（car-passenger-s05-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.060 · Distill-MOS Δ +0.213 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.060 | -0.035（car-passenger-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.213 | +0.117（car-passenger-s00-mba） |

送人耳：
- car-passenger-s02-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.32

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `nav-voice`（speech）：差 +9.366（有 3 段 3.569 · 没有 3 段 -5.797）
- V-EAT-ASR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `nav-voice`（speech）：差 +2.976（有 3 段 5.517 · 没有 3 段 2.540）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.044（有 3 段 0.105 · 没有 3 段 0.062）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `nav-voice`（speech）：差 +0.996（有 3 段 4.770 · 没有 3 段 3.774）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（car-passenger-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.032 dB | -0.191（car-passenger-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（car-passenger-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 4 / 2 / 0 | 1.336 dB | 3.431（car-passenger-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.980 比例 | 1.000（car-passenger-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -20.757 dB | -19.209（car-passenger-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.129 dB² | 17.346（car-passenger-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（car-passenger-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（car-passenger-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.845 dB | 8.801（car-passenger-s01-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（car-passenger-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（car-passenger-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 5 / 0 / 1 | 0.020 dB | 0.035（car-passenger-s04-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（car-passenger-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（car-passenger-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（car-passenger-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `nav-voice`（speech）：差 +1.275（有 3 段 1.974 · 没有 3 段 0.699）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `nav-voice`（speech）：差 +2.148（有 3 段 3.919 · 没有 3 段 1.771）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（car-passenger-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.244 dB | -2.121（car-passenger-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -8.385 百分点 | 0.000（car-passenger-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 2 / 4 / 0 | 4.096 dB | 6.217（car-passenger-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.122 比例 | 0.239（car-passenger-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.620 dB | -14.045（car-passenger-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.085 dB² | 18.729（car-passenger-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 60.833 个/分钟 | 90.000（car-passenger-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.007（car-passenger-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 3.955 dB | 8.495（car-passenger-s01-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -5.484 dB | -4.181（car-passenger-s05-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -4.882 dB | -3.670（car-passenger-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 5 / 0 / 1 | 1.464 dB | 2.455（car-passenger-s02-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.170 × | 1.642（car-passenger-s04-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.122 · Distill-MOS Δ +0.241 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.122 | +0.046（car-passenger-s03-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.241 | +0.112（car-passenger-s03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `nav-voice`（speech）：差 +1.379（有 3 段 4.785 · 没有 3 段 3.406）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `nav-voice`（speech）：差 +2.256（有 3 段 5.083 · 没有 3 段 2.827）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.419 dB | -2.510（bus-metro-cabin-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 1.176 百分点 | 11.765（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 0.366 dB | 2.327（bus-metro-cabin-c09-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.026 比例 | 0.113（bus-metro-cabin-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.499 dB | -0.977（bus-metro-cabin-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 1.848 dB² | 4.241（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 17.500 个/分钟 | 45.000（bus-metro-cabin-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.007（bus-metro-cabin-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.014 dB | 8.919（bus-metro-cabin-c09-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -3.284 dB | -1.431（bus-metro-cabin-c01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -3.139 dB | -1.172（bus-metro-cabin-c07-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.862 dB | 7.069（bus-metro-cabin-c06-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.326 × | 1.852（bus-metro-cabin-c07-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.116 · Distill-MOS Δ +0.187 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.116 | +0.023（bus-metro-cabin-c01-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.187 | +0.079（bus-metro-cabin-c07-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `door-pneumatic`（event）：差 +3.922（有 3 段 3.922 · 没有 7 段 0.000）
- V-EAT-ASR · `bed-metro`（bed）：差 +1.961（有 6 段 1.961 · 没有 4 段 0.000）
- V-EAT-ASR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-bus`（bed）：差 +0.032（有 4 段 0.046 · 没有 6 段 0.014）
- V-SIM-GATE · `talker`（speech）：差 +0.021（有 3 段 0.041 · 没有 7 段 0.020）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `door-pneumatic`（event）：差 +1.664（有 3 段 3.013 · 没有 7 段 1.349）
- V-PUMPV · `bed-metro`（bed）：差 +1.174（有 6 段 2.318 · 没有 4 段 1.144）
- V-PUMPV · `talker`（speech）：差 +0.526（有 3 段 2.216 · 没有 7 段 1.690）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `door-pneumatic`（event）：差 +3.462（有 3 段 5.437 · 没有 7 段 1.975）
- V-TIMBRE · `bed-metro`（bed）：差 +2.333（有 6 段 3.947 · 没有 4 段 1.614）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-bus`（bed）：差 +0.826（有 4 段 -2.789 · 没有 6 段 -3.615）
- V-NR · `brake`（event）：差 +0.492（有 3 段 -2.940 · 没有 7 段 -3.432）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）

### bus-metro-cabin · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（bus-metro-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（bus-metro-cabin-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（bus-metro-cabin-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.485 dB | 0.884（bus-metro-cabin-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.230 比例 | 0.532（bus-metro-cabin-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.346 dB | -2.572（bus-metro-cabin-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.086 dB² | 3.712（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（bus-metro-cabin-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 4.838 dB | 9.559（bus-metro-cabin-c01-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（bus-metro-cabin-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | 0.000 dB | 0.000（bus-metro-cabin-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.309 dB | 1.362（bus-metro-cabin-c03-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（bus-metro-cabin-c00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `talker`（speech）：差 +0.126（有 3 段 0.319 · 没有 7 段 0.193）
- V-SIM-GATE · `bed-bus`（bed）：差 +0.011（有 4 段 0.237 · 没有 6 段 0.226）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `door-pneumatic`（event）：差 +1.097（有 3 段 1.854 · 没有 7 段 0.757）
- V-PUMPV · `bed-metro`（bed）：差 +0.444（有 6 段 1.263 · 没有 4 段 0.820）
- V-PUMPV · `talker`（speech）：差 +0.091（有 3 段 1.149 · 没有 7 段 1.059）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `door-pneumatic`（event）：差 +1.666（有 3 段 6.004 · 没有 7 段 4.338）
- V-TIMBRE · `bed-bus`（bed）：差 +1.088（有 4 段 5.490 · 没有 6 段 4.403）
- V-TIMBRE · `brake`（event）：差 +0.222（有 3 段 4.993 · 没有 7 段 4.771）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）

### bus-metro-cabin · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.115 dB | -0.252（bus-metro-cabin-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | 1.593 百分点 | 11.765（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 0.712 dB | 2.365（bus-metro-cabin-c09-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.020 比例 | 0.056（bus-metro-cabin-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.264 dB | -1.922（bus-metro-cabin-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 2.057 dB² | 6.346（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 8 / 0 | 12.500 个/分钟 | 35.000（bus-metro-cabin-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.014 Δ | -0.004（bus-metro-cabin-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 4.323 dB | 9.412（bus-metro-cabin-c03-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -2.866 dB | -1.233（bus-metro-cabin-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -1.700 dB | -0.491（bus-metro-cabin-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.874 dB | 2.879（bus-metro-cabin-c01-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.166 × | 1.877（bus-metro-cabin-c02-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 1 / 2 | SCOREQ Δ +0.094 · Distill-MOS Δ +0.118 | | Δ ≥ 0 | 不过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.094 | -0.031（bus-metro-cabin-c03-ext） |
| V-NRMOS-DISTILL | 8 / 2 / 0 | +0.118 | -0.026（bus-metro-cabin-c03-ext） |

送人耳：
- bus-metro-cabin-c01-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.05
- bus-metro-cabin-c07-ext：一个说好一个说差：SCOREQ Δ +0.02 · Distill-MOS Δ -0.01

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `door-pneumatic`（event）：差 +3.326（有 3 段 3.922 · 没有 7 段 0.595）
- V-EAT-ASR · `bed-metro`（bed）：差 +0.919（有 6 段 1.961 · 没有 4 段 1.042）
- V-EAT-ASR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-bus`（bed）：差 +0.017（有 4 段 0.030 · 没有 6 段 0.014）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.009（有 3 段 0.027 · 没有 7 段 0.018）
- V-SIM-GATE · `brake`（event）：差 +0.009（有 3 段 0.027 · 没有 7 段 0.018）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `door-pneumatic`（event）：差 +2.572（有 3 段 3.857 · 没有 7 段 1.285）
- V-PUMPV · `bed-metro`（bed）：差 +1.985（有 6 段 2.851 · 没有 4 段 0.866）
- V-PUMPV · `talker`（speech）：差 +0.854（有 3 段 2.655 · 没有 7 段 1.801）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `door-pneumatic`（event）：差 +2.525（有 3 段 6.090 · 没有 7 段 3.565）
- V-TIMBRE · `door-chime`（event）：差 +0.892（有 3 段 4.947 · 没有 7 段 4.055）
- V-TIMBRE · `bed-metro`（bed）：差 +0.623（有 6 段 4.572 · 没有 4 段 3.949）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-bus`（bed）：差 +0.826（有 4 段 -2.370 · 没有 6 段 -3.197）
- V-NR · `door-chime`（event）：差 +0.385（有 3 段 -2.597 · 没有 7 段 -2.982）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `bed-bus`（bed）：差 +0.165（有 4 段 -0.005 · 没有 6 段 0.160）
- V-NRMOS-SCOREQ · `door-chime`（event）：差 +0.080（有 3 段 0.038 · 没有 7 段 0.118）
- V-NRMOS-SCOREQ · `brake`（event）：差 +0.079（有 3 段 0.038 · 没有 7 段 0.118）
- V-NRMOS-SCOREQ · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `bed-bus`（bed）：差 +0.139（有 4 段 0.034 · 没有 6 段 0.173）
- V-NRMOS-DISTILL · `door-chime`（event）：差 +0.106（有 3 段 0.043 · 没有 7 段 0.150）
- V-NRMOS-DISTILL · `brake`（event）：差 +0.072（有 3 段 0.067 · 没有 7 段 0.139）
- V-NRMOS-DISTILL · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### bus-metro-cabin · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -2.225 dB | -3.092（bus-metro-cabin-c08-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 6 / 4 / 0 | 1.475 百分点 | 12.500（bus-metro-cabin-c04-mba） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.231 dB | 4.655（bus-metro-cabin-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.036 比例 | 0.073（bus-metro-cabin-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.298 dB | -1.883（bus-metro-cabin-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.841 dB² | 21.963（bus-metro-cabin-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 23.500 个/分钟 | 50.000（bus-metro-cabin-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.012（bus-metro-cabin-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 4.552 dB | 8.287（bus-metro-cabin-c05-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 7 / 0 | -4.764 dB | -1.269（bus-metro-cabin-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -4.380 dB | -3.327（bus-metro-cabin-c03-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.588 dB | 6.177（bus-metro-cabin-c09-mba） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.314 × | 2.225（bus-metro-cabin-c00-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.149 · Distill-MOS Δ +0.236 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.149 | +0.048（bus-metro-cabin-c04-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.236 | +0.137（bus-metro-cabin-c03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `talker`（speech）：差 +0.758（有 3 段 -2.755 · 没有 7 段 -1.997）
- V-SPLVL · `door-chime`（event）：差 +0.545（有 3 段 -2.606 · 没有 7 段 -2.061）
- V-SPLVL · `bed-bus`（bed）：差 +0.293（有 4 段 -2.401 · 没有 6 段 -2.107）
- V-SPLVL · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-EAT-ASR · `talker`（speech）：差 +9.339（有 3 段 8.013 · 没有 7 段 -1.327）
- V-EAT-ASR · `brake`（event）：差 +5.292（有 3 段 5.179 · 没有 7 段 -0.112）
- V-EAT-ASR · `door-chime`（event）：差 +5.292（有 3 段 5.179 · 没有 7 段 -0.112）
- V-EAT-ASR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `door-pneumatic`（event）：差 +2.035（有 3 段 2.655 · 没有 7 段 0.620）
- V-ONSET · `bed-metro`（bed）：差 +1.125（有 6 段 1.681 · 没有 4 段 0.556）
- V-ONSET · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.009（有 3 段 0.042 · 没有 7 段 0.033）
- V-SIM-GATE · `bed-metro`（bed）：差 +0.007（有 6 段 0.039 · 没有 4 段 0.031）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +6.279（有 3 段 10.237 · 没有 7 段 3.958）
- V-PUMPV · `door-pneumatic`（event）：差 +6.107（有 3 段 10.116 · 没有 7 段 4.009）
- V-PUMPV · `bed-metro`（bed）：差 +3.364（有 6 段 7.187 · 没有 4 段 3.823）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `door-pneumatic`（event）：差 +3.947（有 3 段 7.315 · 没有 7 段 3.368）
- V-TIMBRE · `bed-metro`（bed）：差 +3.371（有 6 段 5.901 · 没有 4 段 2.530）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `brake`（event）：差 +0.127（有 3 段 -4.675 · 没有 7 段 -4.802）
- V-NR · `bed-bus`（bed）：差 +0.103（有 4 段 -4.702 · 没有 6 段 -4.805）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.203（有 3 段 1.456 · 没有 7 段 1.253）
- V-MUSIC · `bed-metro`（bed）：差 +0.152（有 6 段 1.375 · 没有 4 段 1.223）
- V-MUSIC · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### bus-metro-cabin · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（bus-metro-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（bus-metro-cabin-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（bus-metro-cabin-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.086 dB | 1.592（bus-metro-cabin-c09-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.547 比例 | 0.771（bus-metro-cabin-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -13.147 dB | -5.228（bus-metro-cabin-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 3 / 7 / 0 | 4.821 dB² | 20.044（bus-metro-cabin-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（bus-metro-cabin-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 5.056 dB | 10.181（bus-metro-cabin-c06-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（bus-metro-cabin-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | 0.000 dB | 0.000（bus-metro-cabin-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.100 dB | 0.333（bus-metro-cabin-c03-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（bus-metro-cabin-c00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `talker`（speech）：差 +0.168（有 3 段 0.665 · 没有 7 段 0.497）
- V-SIM-GATE · `door-chime`（event）：差 +0.089（有 3 段 0.609 · 没有 7 段 0.520）
- V-SIM-GATE · `brake`（event）：差 +0.009（有 3 段 0.553 · 没有 7 段 0.544）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +6.028（有 3 段 9.041 · 没有 7 段 3.013）
- V-PUMPV · `door-pneumatic`（event）：差 +6.012（有 3 段 9.030 · 没有 7 段 3.018）
- V-PUMPV · `bed-metro`（bed）：差 +1.534（有 6 段 5.435 · 没有 4 段 3.901）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `bed-metro`（bed）：差 +1.948（有 6 段 5.835 · 没有 4 段 3.887）
- V-TIMBRE · `door-pneumatic`（event）：差 +1.845（有 3 段 6.347 · 没有 7 段 4.502）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）

### bus-metro-cabin · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.394 dB | -0.825（bus-metro-cabin-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 4 / 0 | 3.799 百分点 | 15.385（bus-metro-cabin-c08-mba） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.658 dB | 4.961（bus-metro-cabin-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.060 比例 | 0.166（bus-metro-cabin-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.112 dB | -2.670（bus-metro-cabin-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.053 dB² | 22.491（bus-metro-cabin-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 24.500 个/分钟 | 50.000（bus-metro-cabin-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.019 Δ | 0.006（bus-metro-cabin-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 5 / 0 | 5.310 dB | 8.927（bus-metro-cabin-c06-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | -2.947 dB | -0.876（bus-metro-cabin-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -2.389 dB | -1.750（bus-metro-cabin-c04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.802 dB | 3.771（bus-metro-cabin-c04-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.159 × | 1.880（bus-metro-cabin-c02-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.151 · Distill-MOS Δ +0.220 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.151 | +0.066（bus-metro-cabin-c05-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.220 | +0.131（bus-metro-cabin-c03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `brake`（event）：差 +7.001（有 3 段 8.700 · 没有 7 段 1.698）
- V-EAT-ASR · `talker`（speech）：差 +3.600（有 3 段 6.319 · 没有 7 段 2.719）
- V-EAT-ASR · `door-chime`（event）：差 +1.899（有 3 段 5.128 · 没有 7 段 3.229）
- V-EAT-ASR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `door-pneumatic`（event）：差 +1.861（有 3 段 2.961 · 没有 7 段 1.100）
- V-ONSET · `bed-metro`（bed）：差 +1.369（有 6 段 2.206 · 没有 4 段 0.837）
- V-ONSET · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.048（有 3 段 0.094 · 没有 7 段 0.046）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.005（有 3 段 0.064 · 没有 7 段 0.058）
- V-SIM-GATE · `bed-bus`（bed）：差 +0.004（有 4 段 0.062 · 没有 6 段 0.059）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +8.140（有 3 段 11.751 · 没有 7 段 3.611）
- V-PUMPV · `door-pneumatic`（event）：差 +6.654（有 3 段 10.710 · 没有 7 段 4.057）
- V-PUMPV · `bed-metro`（bed）：差 +3.952（有 6 段 7.633 · 没有 4 段 3.682）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `door-pneumatic`（event）：差 +3.237（有 3 段 7.576 · 没有 7 段 4.339）
- V-TIMBRE · `bed-metro`（bed）：差 +2.867（有 6 段 6.457 · 没有 4 段 3.590）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-bus`（bed）：差 +0.749（有 4 段 -2.497 · 没有 6 段 -3.246）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）

### bus-metro-cabin · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -2.263 dB | -2.774（bus-metro-cabin-s05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | -0.948 百分点 | 6.977（bus-metro-cabin-s05-ext） | 1.000 | 不过 |
| V-ONSET |  | 4 / 0 / 2 | 1.360 dB | 2.842（bus-metro-cabin-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.178 比例 | 0.363（bus-metro-cabin-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.720 dB | -9.408（bus-metro-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.461 dB² | 23.564（bus-metro-cabin-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 20.833 个/分钟 | 35.000（bus-metro-cabin-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.010 Δ | 0.002（bus-metro-cabin-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 2 / 1 | 6.845 dB | 13.610（bus-metro-cabin-s04-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -3.688 dB | -2.528（bus-metro-cabin-s03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -4.164 dB | -3.227（bus-metro-cabin-s04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.092 dB | 0.367（bus-metro-cabin-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.152 × | 1.585（bus-metro-cabin-s05-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.083 · Distill-MOS Δ +0.199 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.083 | -0.106（bus-metro-cabin-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.199 | +0.167（bus-metro-cabin-s03-ext） |

送人耳：
- bus-metro-cabin-s01-ext：一个说好一个说差：SCOREQ Δ -0.11 · Distill-MOS Δ +0.20
- bus-metro-cabin-s05-ext：一个说好一个说差：SCOREQ Δ -0.06 · Distill-MOS Δ +0.20

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `brake`（event）：差 +4.165（有 3 段 1.135 · 没有 3 段 -3.030）
- V-EAT-ASR · `door-chime`（event）：差 +4.165（有 3 段 1.135 · 没有 3 段 -3.030）
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-chime`（event）：差 +0.225（有 3 段 0.291 · 没有 3 段 0.066）
- V-SIM-GATE · `brake`（event）：差 +0.020（有 3 段 0.189 · 没有 3 段 0.168）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door-chime`（event）：差 +6.738（有 3 段 14.830 · 没有 3 段 8.092）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `door-chime`（event）：差 +1.642（有 3 段 -2.867 · 没有 3 段 -4.509）
- V-NR · `brake`（event）：差 +1.292（有 3 段 -3.042 · 没有 3 段 -4.334）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（bus-metro-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -0.000 dB | -0.001（bus-metro-cabin-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（bus-metro-cabin-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 4 / 0 / 2 | 0.455 dB | 1.924（bus-metro-cabin-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.561 比例 | 0.791（bus-metro-cabin-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -17.219 dB | -13.039（bus-metro-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.790 dB² | 45.572（bus-metro-cabin-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（bus-metro-cabin-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 3 / 1 | 7.503 dB | 15.117（bus-metro-cabin-s04-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（bus-metro-cabin-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（bus-metro-cabin-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.070 dB | 0.268（bus-metro-cabin-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（bus-metro-cabin-s00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `door-chime`（event）：差 +0.245（有 3 段 0.683 · 没有 3 段 0.438）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.045（有 3 段 0.583 · 没有 3 段 0.538）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door-chime`（event）：差 +15.172（有 3 段 23.376 · 没有 3 段 8.204）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -0.285 dB | -0.726（bus-metro-cabin-s05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 2 / 4 / 0 | 2.721 百分点 | 5.882（bus-metro-cabin-s01-ext） | 1.000 | 不过 |
| V-ONSET |  | 1 / 3 / 2 | 3.707 dB | 7.845（bus-metro-cabin-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.080 比例 | 0.163（bus-metro-cabin-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.675 dB | -9.640（bus-metro-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.481 dB² | 45.011（bus-metro-cabin-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 14.167 个/分钟 | 20.000（bus-metro-cabin-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.006 Δ | -0.005（bus-metro-cabin-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 3 / 1 | 7.484 dB | 13.551（bus-metro-cabin-s04-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.247 dB | -0.838（bus-metro-cabin-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -1.690 dB | -1.065（bus-metro-cabin-s04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.064 dB | 0.237（bus-metro-cabin-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.018 × | 1.091（bus-metro-cabin-s05-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 3 / 0 / 3 | SCOREQ Δ -0.015 · Distill-MOS Δ +0.176 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 3 / 3 / 0 | -0.015 | -0.199（bus-metro-cabin-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.176 | +0.094（bus-metro-cabin-s03-ext） |

送人耳：
- bus-metro-cabin-s01-ext：一个说好一个说差：SCOREQ Δ -0.16 · Distill-MOS Δ +0.21
- bus-metro-cabin-s03-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.09
- bus-metro-cabin-s05-ext：一个说好一个说差：SCOREQ Δ -0.20 · Distill-MOS Δ +0.20

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `door-chime`（event）：差 +2.411（有 3 段 3.926 · 没有 3 段 1.515）
- V-EAT-ASR · `door-pneumatic`（event）：差 +1.520（有 3 段 3.481 · 没有 3 段 1.961）
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-chime`（event）：差 +0.066（有 3 段 0.113 · 没有 3 段 0.047）
- V-SIM-GATE · `brake`（event）：差 +0.057（有 3 段 0.108 · 没有 3 段 0.052）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.023（有 3 段 0.091 · 没有 3 段 0.069）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door-chime`（event）：差 +11.075（有 3 段 21.019 · 没有 3 段 9.944）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `door-chime`（event）：差 +0.434（有 3 段 -2.030 · 没有 3 段 -2.465）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.044 dB | -3.961（bus-metro-cabin-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 4.682 百分点 | 25.000（bus-metro-cabin-s03-mba） | 1.000 | 不过 |
| V-ONSET |  | 1 / 4 / 1 | 5.169 dB | 13.190（bus-metro-cabin-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.033 比例 | 0.067（bus-metro-cabin-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.774 dB | -10.646（bus-metro-cabin-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 14.378 dB² | 26.391（bus-metro-cabin-s03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 19.167 个/分钟 | 25.000（bus-metro-cabin-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.008 Δ | -0.001（bus-metro-cabin-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 1 / 4 / 1 | 11.643 dB | 21.389（bus-metro-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -5.155 dB | -3.165（bus-metro-cabin-s04-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -4.676 dB | -2.760（bus-metro-cabin-s04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.059 dB | 0.218（bus-metro-cabin-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.154 × | 1.776（bus-metro-cabin-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.053 · Distill-MOS Δ +0.140 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.053 | +0.000（bus-metro-cabin-s03-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.140 | +0.076（bus-metro-cabin-s05-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `door-chime`（event）：差 +0.826（有 3 段 -3.457 · 没有 3 段 -2.631）
- V-SPLVL · `door-pneumatic`（event）：差 +0.268（有 3 段 -3.178 · 没有 3 段 -2.910）
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `brake`（event）：差 +5.752（有 3 段 7.558 · 没有 3 段 1.807）
- V-EAT-ASR · `door-chime`（event）：差 +5.752（有 3 段 7.558 · 没有 3 段 1.807）
- V-EAT-ASR · `door-pneumatic`（event）：差 +4.236（有 3 段 6.801 · 没有 3 段 2.564）
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `brake`（event）：差 +0.016（有 3 段 0.041 · 没有 3 段 0.026）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.005（有 3 段 0.036 · 没有 3 段 0.031）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `brake`（event）：差 +2.552（有 3 段 15.654 · 没有 3 段 13.102）
- V-PUMPV · `door-chime`（event）：差 +2.114（有 3 段 15.435 · 没有 3 段 13.321）
- V-PUMPV · `door-pneumatic`（event）：差 +0.064（有 3 段 14.410 · 没有 3 段 14.346）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `brake`（event）：差 +1.196（有 3 段 -4.557 · 没有 3 段 -5.753）
- V-NR · `door-chime`（event）：差 +0.740（有 3 段 -4.785 · 没有 3 段 -5.525）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（bus-metro-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.004 dB | -0.022（bus-metro-cabin-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（bus-metro-cabin-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 4 / 1 / 1 | 3.560 dB | 11.634（bus-metro-cabin-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.754 比例 | 0.931（bus-metro-cabin-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -21.024 dB | -15.484（bus-metro-cabin-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.449 dB² | 12.790（bus-metro-cabin-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（bus-metro-cabin-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 1 / 4 / 1 | 12.325 dB | 19.946（bus-metro-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（bus-metro-cabin-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | 0.000 dB | 0.000（bus-metro-cabin-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.030 dB | 0.056（bus-metro-cabin-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（bus-metro-cabin-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（bus-metro-cabin-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.142（有 3 段 0.825 · 没有 3 段 0.683）
- V-SIM-GATE · `door-chime`（event）：差 +0.141（有 3 段 0.824 · 没有 3 段 0.684）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.773 dB | -1.340（bus-metro-cabin-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 1.986 百分点 | 11.538（bus-metro-cabin-s00-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 2 / 1 | 5.172 dB | 14.906（bus-metro-cabin-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.083 比例 | 0.137（bus-metro-cabin-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -16.642 dB | -12.102（bus-metro-cabin-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.766 dB² | 23.808（bus-metro-cabin-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 20.000 个/分钟 | 30.000（bus-metro-cabin-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.011 Δ | -0.001（bus-metro-cabin-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 1 / 4 / 1 | 11.566 dB | 21.461（bus-metro-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -1.964 dB | -1.131（bus-metro-cabin-s02-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -2.287 dB | -0.894（bus-metro-cabin-s04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.078 dB | 0.361（bus-metro-cabin-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.198 × | 1.852（bus-metro-cabin-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.041 · Distill-MOS Δ +0.191 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.041 | -0.019（bus-metro-cabin-s03-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.191 | +0.131（bus-metro-cabin-s04-mba） |

送人耳：
- bus-metro-cabin-s03-mba：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.15

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `brake`（event）：差 +2.340（有 3 段 3.156 · 没有 3 段 0.816）
- V-EAT-ASR · `door-chime`（event）：差 +2.340（有 3 段 3.156 · 没有 3 段 0.816）
- V-EAT-ASR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-chime`（event）：差 +0.080（有 3 段 0.123 · 没有 3 段 0.043）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.035（有 3 段 0.101 · 没有 3 段 0.066）
- V-SIM-GATE · `brake`（event）：差 +0.021（有 3 段 0.094 · 没有 3 段 0.073）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `door-pneumatic`（event）：差 +0.364（有 3 段 -1.782 · 没有 3 段 -2.146）
- V-NR · `door-chime`（event）：差 +0.312（有 3 段 -1.808 · 没有 3 段 -2.120）
- V-NR · `brake`（event）：差 +0.023（有 3 段 -1.953 · 没有 3 段 -1.976）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）

### rail-air-cabin · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.935 dB | -2.927（rail-air-cabin-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 5 / 0 | 3.569 百分点 | 20.000（rail-air-cabin-c09-ext） | 1.000 | 不过 |
| V-ONSET |  | 6 / 4 / 0 | 2.266 dB | 6.715（rail-air-cabin-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.034 比例 | 0.082（rail-air-cabin-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -12.031 dB | -3.555（rail-air-cabin-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.011 dB² | 12.143（rail-air-cabin-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 24.500 个/分钟 | 40.000（rail-air-cabin-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.022 Δ | 0.014（rail-air-cabin-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 3.248 dB | 8.313（rail-air-cabin-c09-ext） | 6.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -6.609 dB | -5.513（rail-air-cabin-c02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -4.283 dB | -2.233（rail-air-cabin-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.110 dB | 5.726（rail-air-cabin-c09-ext） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.361 × | 3.481（rail-air-cabin-c03-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 0 / 3 | SCOREQ Δ +0.095 · Distill-MOS Δ +0.142 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.095 | -0.117（rail-air-cabin-c09-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.142 | +0.062（rail-air-cabin-c06-ext） |

送人耳：
- rail-air-cabin-c00-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.07
- rail-air-cabin-c06-ext：一个说好一个说差：SCOREQ Δ -0.10 · Distill-MOS Δ +0.06
- rail-air-cabin-c09-ext：一个说好一个说差：SCOREQ Δ -0.12 · Distill-MOS Δ +0.09

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `trolley`（event）：差 +6.226（有 3 段 7.928 · 没有 7 段 1.701）
- V-EAT-ASR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +1.761（有 3 段 3.498 · 没有 7 段 1.737）
- V-ONSET · `pa-chime`（event）：差 +1.761（有 3 段 3.498 · 没有 7 段 1.737）
- V-ONSET · `trolley`（event）：差 +0.638（有 3 段 2.712 · 没有 7 段 2.074）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.029（有 3 段 0.054 · 没有 7 段 0.025）
- V-SIM-GATE · `pa-chime`（event）：差 +0.029（有 3 段 0.054 · 没有 7 段 0.025）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `announce`（speech）：差 +4.070（有 3 段 7.860 · 没有 7 段 3.790）
- V-PUMPV · `pa-chime`（event）：差 +4.070（有 3 段 7.860 · 没有 7 段 3.790）
- V-PUMPV · `trolley`（event）：差 +1.610（有 3 段 6.138 · 没有 7 段 4.528）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `announce`（speech）：差 +0.648（有 3 段 3.702 · 没有 7 段 3.054）
- V-TIMBRE · `pa-chime`（event）：差 +0.648（有 3 段 3.702 · 没有 7 段 3.054）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `announce`（speech）：差 +0.411（有 3 段 -6.322 · 没有 7 段 -6.732）
- V-NR · `pa-chime`（event）：差 +0.411（有 3 段 -6.322 · 没有 7 段 -6.732）
- V-NR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `trolley`（event）：差 +0.933（有 3 段 2.014 · 没有 7 段 1.081）
- V-MUSIC · `announce`（speech）：差 +0.327（有 3 段 1.590 · 没有 7 段 1.263）
- V-MUSIC · `pa-chime`（event）：差 +0.327（有 3 段 1.590 · 没有 7 段 1.263）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（rail-air-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（rail-air-cabin-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（rail-air-cabin-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 0.706 dB | 4.718（rail-air-cabin-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.706 比例 | 0.899（rail-air-cabin-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -17.232 dB | -7.758（rail-air-cabin-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 3.384 dB² | 9.095（rail-air-cabin-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（rail-air-cabin-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.977 dB | 10.110（rail-air-cabin-c05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（rail-air-cabin-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | 0.000 dB | 0.000（rail-air-cabin-c00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.057 dB | 0.159（rail-air-cabin-c03-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（rail-air-cabin-c00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `announce`（speech）：差 +1.733（有 3 段 1.919 · 没有 7 段 0.186）
- V-ONSET · `pa-chime`（event）：差 +1.733（有 3 段 1.919 · 没有 7 段 0.186）
- V-ONSET · `trolley`（event）：差 +1.232（有 3 段 1.568 · 没有 7 段 0.336）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.232（有 3 段 0.869 · 没有 7 段 0.636）
- V-SIM-GATE · `pa-chime`（event）：差 +0.232（有 3 段 0.869 · 没有 7 段 0.636）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `announce`（speech）：差 +3.286（有 3 段 5.684 · 没有 7 段 2.399）
- V-PUMPV · `pa-chime`（event）：差 +3.286（有 3 段 5.684 · 没有 7 段 2.399）
- V-PUMPV · `trolley`（event）：差 +1.246（有 3 段 4.257 · 没有 7 段 3.010）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `trolley`（event）：差 +1.663（有 3 段 5.142 · 没有 7 段 3.478）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）

### rail-air-cabin · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.538 dB | -0.905（rail-air-cabin-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 5 / 5 / 0 | 0.093 百分点 | 10.345（rail-air-cabin-c07-ext） | 1.000 | 不过 |
| V-ONSET |  | 5 / 5 / 0 | 3.208 dB | 7.827（rail-air-cabin-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.071 比例 | 0.113（rail-air-cabin-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -12.362 dB | -3.765（rail-air-cabin-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.821 dB² | 13.730（rail-air-cabin-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 19.500 个/分钟 | 40.000（rail-air-cabin-c09-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.009（rail-air-cabin-c02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.488 dB | 9.985（rail-air-cabin-c05-ext） | 6.000 | 不过 |
| V-NR |  | 4 / 6 / 0 | -5.775 dB | -4.634（rail-air-cabin-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -3.300 dB | -1.342（rail-air-cabin-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.013 dB | 3.987（rail-air-cabin-c09-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.831 × | 3.318（rail-air-cabin-c06-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 0 / 3 | SCOREQ Δ +0.100 · Distill-MOS Δ +0.192 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.100 | -0.091（rail-air-cabin-c09-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.192 | +0.092（rail-air-cabin-c02-ext） |

送人耳：
- rail-air-cabin-c00-ext：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.13
- rail-air-cabin-c06-ext：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.12
- rail-air-cabin-c09-ext：一个说好一个说差：SCOREQ Δ -0.09 · Distill-MOS Δ +0.11

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `announce`（speech）：差 +9.130（有 3 段 6.484 · 没有 7 段 -2.646）
- V-EAT-ASR · `pa-chime`（event）：差 +9.130（有 3 段 6.484 · 没有 7 段 -2.646）
- V-EAT-ASR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +1.732（有 3 段 4.420 · 没有 7 段 2.688）
- V-ONSET · `pa-chime`（event）：差 +1.732（有 3 段 4.420 · 没有 7 段 2.688）
- V-ONSET · `trolley`（event）：差 +1.041（有 3 段 3.936 · 没有 7 段 2.895）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.045（有 3 段 0.103 · 没有 7 段 0.058）
- V-SIM-GATE · `pa-chime`（event）：差 +0.045（有 3 段 0.103 · 没有 7 段 0.058）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `announce`（speech）：差 +4.630（有 3 段 9.062 · 没有 7 段 4.432）
- V-PUMPV · `pa-chime`（event）：差 +4.630（有 3 段 9.062 · 没有 7 段 4.432）
- V-PUMPV · `trolley`（event）：差 +1.576（有 3 段 6.924 · 没有 7 段 5.348）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `trolley`（event）：差 +1.319（有 3 段 4.411 · 没有 7 段 3.092）
- V-TIMBRE · `announce`（speech）：差 +0.295（有 3 段 3.695 · 没有 7 段 3.400）
- V-TIMBRE · `pa-chime`（event）：差 +0.295（有 3 段 3.695 · 没有 7 段 3.400）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `trolley`（event）：差 +1.356（有 3 段 -4.826 · 没有 7 段 -6.182）
- V-NR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +1.324（有 3 段 2.757 · 没有 7 段 1.434）
- V-MUSIC · `pa-chime`（event）：差 +1.324（有 3 段 2.757 · 没有 7 段 1.434）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -2.261 dB | -2.818（rail-air-cabin-c01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 4 / 0 | 2.834 百分点 | 19.231（rail-air-cabin-c08-mba） | 1.000 | 不过 |
| V-ONSET |  | 5 / 5 / 0 | 2.863 dB | 8.314（rail-air-cabin-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.055 比例 | 0.147（rail-air-cabin-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -13.688 dB | -4.447（rail-air-cabin-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 8.469 dB² | 21.620（rail-air-cabin-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 22.500 个/分钟 | 40.000（rail-air-cabin-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.019 Δ | 0.014（rail-air-cabin-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 5.542 dB | 11.982（rail-air-cabin-c09-mba） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -8.515 dB | -7.463（rail-air-cabin-c02-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -4.867 dB | -2.896（rail-air-cabin-c09-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.106 dB | 5.850（rail-air-cabin-c09-mba） | 10.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 1.637 × | 2.318（rail-air-cabin-c07-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 7 / 0 / 3 | SCOREQ Δ +0.079 · Distill-MOS Δ +0.147 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 7 / 3 / 0 | +0.079 | -0.052（rail-air-cabin-c01-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.147 | +0.062（rail-air-cabin-c00-mba） |

送人耳：
- rail-air-cabin-c01-mba：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.20
- rail-air-cabin-c03-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.13
- rail-air-cabin-c08-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.13

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `trolley`（event）：差 +0.460（有 3 段 3.157 · 没有 7 段 2.696）
- V-EAT-ASR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +1.841（有 3 段 4.151 · 没有 7 段 2.310）
- V-ONSET · `pa-chime`（event）：差 +1.841（有 3 段 4.151 · 没有 7 段 2.310）
- V-ONSET · `trolley`（event）：差 +1.055（有 3 段 3.601 · 没有 7 段 2.546）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.015（有 3 段 0.065 · 没有 7 段 0.051）
- V-SIM-GATE · `pa-chime`（event）：差 +0.015（有 3 段 0.065 · 没有 7 段 0.051）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `trolley`（event）：差 +5.849（有 3 段 12.563 · 没有 7 段 6.714）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `trolley`（event）：差 +1.213（有 3 段 6.391 · 没有 7 段 5.178）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +0.699（有 3 段 2.127 · 没有 7 段 1.427）
- V-MUSIC · `pa-chime`（event）：差 +0.699（有 3 段 2.127 · 没有 7 段 1.427）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（rail-air-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | 0.000 dB | 0.000（rail-air-cabin-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（rail-air-cabin-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 1.002 dB | 5.245（rail-air-cabin-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.779 比例 | 0.901（rail-air-cabin-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -19.242 dB | -9.320（rail-air-cabin-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.265 dB² | 18.178（rail-air-cabin-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（rail-air-cabin-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.847 dB | 9.819（rail-air-cabin-c05-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（rail-air-cabin-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | 0.000 dB | 0.000（rail-air-cabin-c00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.060 dB | 0.148（rail-air-cabin-c03-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（rail-air-cabin-c00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `announce`（speech）：差 +1.619（有 3 段 2.135 · 没有 7 段 0.516）
- V-ONSET · `pa-chime`（event）：差 +1.619（有 3 段 2.135 · 没有 7 段 0.516）
- V-ONSET · `trolley`（event）：差 +0.643（有 3 段 1.452 · 没有 7 段 0.809）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.122（有 3 段 0.865 · 没有 7 段 0.742）
- V-SIM-GATE · `pa-chime`（event）：差 +0.122（有 3 段 0.865 · 没有 7 段 0.742）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `trolley`（event）：差 +5.519（有 3 段 10.128 · 没有 7 段 4.609）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `trolley`（event）：差 +2.489（有 3 段 5.589 · 没有 7 段 3.101）
- V-TIMBRE · `announce`（speech）：差 +0.071（有 3 段 3.897 · 没有 7 段 3.826）
- V-TIMBRE · `pa-chime`（event）：差 +0.071（有 3 段 3.897 · 没有 7 段 3.826）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）

### rail-air-cabin · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.736 dB | -1.128（rail-air-cabin-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 6 / 0 | 2.884 百分点 | 19.231（rail-air-cabin-c08-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 6 / 0 | 3.998 dB | 9.281（rail-air-cabin-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.098 比例 | 0.178（rail-air-cabin-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -14.139 dB | -4.675（rail-air-cabin-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 13.888 dB² | 54.959（rail-air-cabin-c07-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 22.000 个/分钟 | 30.000（rail-air-cabin-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.019（rail-air-cabin-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 4.020 dB | 10.413（rail-air-cabin-c09-mba） | 6.000 | 不过 |
| V-NR |  | 7 / 3 / 0 | -6.629 dB | -4.971（rail-air-cabin-c04-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 9 / 1 | -3.893 dB | -1.782（rail-air-cabin-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.691 dB | 4.765（rail-air-cabin-c07-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.850 × | 3.415（rail-air-cabin-c00-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 8 / 0 / 2 | SCOREQ Δ +0.084 · Distill-MOS Δ +0.190 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.084 | -0.032（rail-air-cabin-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.190 | +0.089（rail-air-cabin-c00-mba） |

送人耳：
- rail-air-cabin-c03-mba：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.22
- rail-air-cabin-c08-mba：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.21

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +1.439（有 3 段 5.005 · 没有 7 段 3.566）
- V-ONSET · `pa-chime`（event）：差 +1.439（有 3 段 5.005 · 没有 7 段 3.566）
- V-ONSET · `trolley`（event）：差 +0.922（有 3 段 4.643 · 没有 7 段 3.721）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.024（有 3 段 0.115 · 没有 7 段 0.091）
- V-SIM-GATE · `pa-chime`（event）：差 +0.024（有 3 段 0.115 · 没有 7 段 0.091）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `trolley`（event）：差 +2.497（有 3 段 15.636 · 没有 7 段 13.139）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `trolley`（event）：差 +1.179（有 3 段 4.846 · 没有 7 段 3.667）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `trolley`（event）：差 +1.437（有 3 段 -5.623 · 没有 7 段 -7.060）
- V-NR · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +1.161（有 3 段 2.663 · 没有 7 段 1.501）
- V-MUSIC · `pa-chime`（event）：差 +1.161（有 3 段 2.663 · 没有 7 段 1.501）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.319 dB | -4.139（rail-air-cabin-s03-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 1.355 百分点 | 8.000（rail-air-cabin-s00-ext） | 1.000 | 不过 |
| V-ONSET |  | 3 / 3 / 0 | 2.659 dB | 4.313（rail-air-cabin-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.064 比例 | 0.154（rail-air-cabin-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.330 dB | -5.781（rail-air-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.623 dB² | 22.092（rail-air-cabin-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 20.000 个/分钟 | 50.000（rail-air-cabin-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.011 Δ | 0.006（rail-air-cabin-s05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.410 dB | 12.027（rail-air-cabin-s03-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -6.069 dB | -5.722（rail-air-cabin-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -5.491 dB | -3.624（rail-air-cabin-s01-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.442 dB | 2.209（rail-air-cabin-s00-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.096 × | 1.267（rail-air-cabin-s00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.049 · Distill-MOS Δ +0.084 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.049 | -0.108（rail-air-cabin-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.084 | +0.008（rail-air-cabin-s03-ext） |

送人耳：
- rail-air-cabin-s05-ext：一个说好一个说差：SCOREQ Δ -0.11 · Distill-MOS Δ +0.04

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）

### rail-air-cabin · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（rail-air-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（rail-air-cabin-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（rail-air-cabin-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 1.076 dB | 3.261（rail-air-cabin-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.884 比例 | 1.000（rail-air-cabin-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -20.819 dB | -10.597（rail-air-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.673 dB² | 18.755（rail-air-cabin-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（rail-air-cabin-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.369 dB | 15.282（rail-air-cabin-s01-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（rail-air-cabin-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（rail-air-cabin-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.028 dB | 0.116（rail-air-cabin-s00-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（rail-air-cabin-s00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）

### rail-air-cabin · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.500 dB | -2.585（rail-air-cabin-s03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 2 / 4 / 0 | 5.209 百分点 | 16.000（rail-air-cabin-s00-ext） | 1.000 | 不过 |
| V-ONSET |  | 1 / 5 / 0 | 3.765 dB | 5.601（rail-air-cabin-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.150 比例 | 0.299（rail-air-cabin-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.574 dB | -5.947（rail-air-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 21.565 dB² | 44.780（rail-air-cabin-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 26.667 个/分钟 | 40.000（rail-air-cabin-s03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.016 Δ | 0.007（rail-air-cabin-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 4.918 dB | 11.659（rail-air-cabin-s03-ext） | 6.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -5.216 dB | -4.418（rail-air-cabin-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -3.609 dB | -2.172（rail-air-cabin-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.338 dB | 1.158（rail-air-cabin-s00-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.713 × | 2.511（rail-air-cabin-s00-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.096 · Distill-MOS Δ +0.137 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.096 | -0.079（rail-air-cabin-s05-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.137 | +0.054（rail-air-cabin-s03-ext） |

送人耳：
- rail-air-cabin-s05-ext：一个说好一个说差：SCOREQ Δ -0.08 · Distill-MOS Δ +0.09

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）

### rail-air-cabin · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 0 / 6 / 0 | -3.349 dB | -4.009（rail-air-cabin-s03-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 0.711 百分点 | 6.667（rail-air-cabin-s01-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 1.695 dB | 4.184（rail-air-cabin-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.076 比例 | 0.131（rail-air-cabin-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -16.641 dB | -11.520（rail-air-cabin-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 18.358 dB² | 49.640（rail-air-cabin-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 23.333 个/分钟 | 50.000（rail-air-cabin-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.011 Δ | 0.004（rail-air-cabin-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 4.730 dB | 10.604（rail-air-cabin-s03-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -7.560 dB | -6.089（rail-air-cabin-s03-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -5.508 dB | -3.574（rail-air-cabin-s01-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.251 dB | 4.208（rail-air-cabin-s01-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.305 × | 1.938（rail-air-cabin-s05-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 3 / 0 / 3 | SCOREQ Δ +0.001 · Distill-MOS Δ +0.066 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 3 / 3 / 0 | +0.001 | -0.073（rail-air-cabin-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.066 | +0.008（rail-air-cabin-s03-mba） |

送人耳：
- rail-air-cabin-s01-mba：一个说好一个说差：SCOREQ Δ -0.07 · Distill-MOS Δ +0.04
- rail-air-cabin-s02-mba：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.13
- rail-air-cabin-s05-mba：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.07

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）

### rail-air-cabin · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（rail-air-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（rail-air-cabin-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（rail-air-cabin-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | -0.301 dB | 1.746（rail-air-cabin-s00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.958 比例 | 1.000（rail-air-cabin-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -22.325 dB | -17.066（rail-air-cabin-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.197 dB² | 17.041（rail-air-cabin-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（rail-air-cabin-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 5.962 dB | 12.922（rail-air-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（rail-air-cabin-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | 0.000 dB | 0.000（rail-air-cabin-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.015 dB | 0.036（rail-air-cabin-s00-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（rail-air-cabin-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（rail-air-cabin-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）

### rail-air-cabin · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.725 dB | -2.750（rail-air-cabin-s03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 0.825 百分点 | 8.000（rail-air-cabin-s00-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 3 / 0 | 3.153 dB | 5.808（rail-air-cabin-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.210 比例 | 0.348（rail-air-cabin-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -16.815 dB | -11.577（rail-air-cabin-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 17.488 dB² | 29.700（rail-air-cabin-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 32.500 个/分钟 | 70.000（rail-air-cabin-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.005（rail-air-cabin-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.487 dB | 14.268（rail-air-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 2 / 4 / 0 | -6.230 dB | -4.868（rail-air-cabin-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -3.698 dB | -2.185（rail-air-cabin-s01-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.286 dB | 1.548（rail-air-cabin-s01-mba） | 10.000 | 过 |
| V-MUSIC |  | 2 / 4 / 0 | 2.107 × | 2.651（rail-air-cabin-s04-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 2 / 0 / 4 | SCOREQ Δ -0.016 · Distill-MOS Δ +0.123 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 2 / 4 / 0 | -0.016 | -0.115（rail-air-cabin-s05-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.123 | +0.048（rail-air-cabin-s03-mba） |

送人耳：
- rail-air-cabin-s01-mba：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.10
- rail-air-cabin-s02-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.17
- rail-air-cabin-s03-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.05
- rail-air-cabin-s05-mba：一个说好一个说差：SCOREQ Δ -0.12 · Distill-MOS Δ +0.11

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）

### street-outdoor · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.591 dB | -2.419（street-outdoor-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -0.680 百分点 | 4.000（street-outdoor-c05-ext） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.160 dB | 3.124（street-outdoor-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.016 比例 | 0.028（street-outdoor-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.366 dB | -1.499（street-outdoor-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.466 dB² | 7.593（street-outdoor-c07-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 20.000 个/分钟 | 30.000（street-outdoor-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.043 Δ | 0.023（street-outdoor-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.494 dB | 1.133（street-outdoor-c03-ext） | 6.000 | 过 |
| V-NR |  | 8 / 2 / 0 | -6.992 dB | -4.871（street-outdoor-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | -4.248 dB | -2.184（street-outdoor-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.278 dB | 6.267（street-outdoor-c00-ext） | 10.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 2.103 × | 6.155（street-outdoor-c06-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.207 · Distill-MOS Δ +0.237 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.207 | -0.029（street-outdoor-c02-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.237 | +0.163（street-outdoor-c02-ext） |

送人耳：
- street-outdoor-c02-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.16

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `car-horn`（event）：差 +0.988（有 4 段 -0.087 · 没有 6 段 -1.075）
- V-EAT-ASR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.917（有 3 段 1.802 · 没有 7 段 0.885）
- V-ONSET · `car-horn`（event）：差 +0.609（有 4 段 1.526 · 没有 6 段 0.916）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.011（有 3 段 0.023 · 没有 7 段 0.012）
- V-SIM-GATE · `car-horn`（event）：差 +0.008（有 4 段 0.020 · 没有 6 段 0.013）
- V-SIM-GATE · `footsteps`（event）：差 +0.003（有 3 段 0.018 · 没有 7 段 0.015）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `talker`（speech）：差 +0.048（有 3 段 -6.958 · 没有 7 段 -7.006）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`（event）：差 +1.156（有 3 段 2.912 · 没有 7 段 1.756）
- V-MUSIC · `car-horn`（event）：差 +0.452（有 4 段 2.374 · 没有 6 段 1.922）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（street-outdoor-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.007（street-outdoor-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（street-outdoor-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 9 / 1 / 0 | 0.331 dB | 4.810（street-outdoor-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.257 比例 | 0.449（street-outdoor-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.503 dB | -5.135（street-outdoor-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.720 dB² | 5.876（street-outdoor-c07-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（street-outdoor-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（street-outdoor-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.116 dB | 4.356（street-outdoor-c05-ext） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（street-outdoor-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | 0.000 dB | 0.000（street-outdoor-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.097 dB | 0.417（street-outdoor-c00-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（street-outdoor-c00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（street-outdoor-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（street-outdoor-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `talker`（speech）：差 +1.814（有 3 段 1.600 · 没有 7 段 -0.213）
- V-ONSET · `car-horn`（event）：差 +0.992（有 4 段 0.926 · 没有 6 段 -0.066）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`（event）：差 +0.156（有 4 段 0.351 · 没有 6 段 0.195）
- V-SIM-GATE · `talker`（speech）：差 +0.101（有 3 段 0.328 · 没有 7 段 0.227）
- V-SIM-GATE · `footsteps`（event）：差 +0.016（有 3 段 0.269 · 没有 7 段 0.253）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`（event）：差 +0.716（有 4 段 2.149 · 没有 6 段 1.433）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）

### street-outdoor · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.186 dB | -0.412（street-outdoor-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -0.035 百分点 | 4.000（street-outdoor-c05-ext） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.446 dB | 7.046（street-outdoor-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.020 比例 | 0.075（street-outdoor-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.739 dB | -1.548（street-outdoor-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 7.116 dB² | 49.639（street-outdoor-c07-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 23.500 个/分钟 | 45.000（street-outdoor-c01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.027 Δ | 0.022（street-outdoor-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.302 dB | 4.500（street-outdoor-c05-ext） | 6.000 | 过 |
| V-NR |  | 2 / 8 / 0 | -4.447 dB | -1.717（street-outdoor-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | -2.979 dB | -1.483（street-outdoor-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.381 dB | 3.174（street-outdoor-c02-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.706 × | 3.046（street-outdoor-c02-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.166 · Distill-MOS Δ +0.182 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.166 | -0.004（street-outdoor-c02-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.182 | +0.124（street-outdoor-c01-ext） |

送人耳：
- street-outdoor-c02-ext：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.15

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +1.954（有 3 段 1.333 · 没有 7 段 -0.621）
- V-EAT-ASR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.562（有 3 段 2.539 · 没有 7 段 0.977）
- V-ONSET · `car-horn`（event）：差 +1.050（有 4 段 2.075 · 没有 6 段 1.026）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.021（有 3 段 0.035 · 没有 7 段 0.014）
- V-SIM-GATE · `car-horn`（event）：差 +0.015（有 4 段 0.029 · 没有 6 段 0.014）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`（event）：差 +0.567（有 3 段 2.103 · 没有 7 段 1.536）
- V-MUSIC · `talker`（speech）：差 +0.138（有 3 段 1.802 · 没有 7 段 1.664）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.276 dB | -2.846（street-outdoor-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 8 / 2 / 0 | 0.810 百分点 | 4.762（street-outdoor-c07-mba） | 1.000 | 不过 |
| V-ONSET |  | 9 / 1 / 0 | 1.502 dB | 3.203（street-outdoor-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.020 比例 | 0.038（street-outdoor-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.453 dB | -4.346（street-outdoor-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.074 dB² | 7.622（street-outdoor-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 30.500 个/分钟 | 40.000（street-outdoor-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.038 Δ | 0.017（street-outdoor-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.967 dB | 2.718（street-outdoor-c07-mba） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -8.073 dB | -4.445（street-outdoor-c03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | -5.335 dB | -2.805（street-outdoor-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.526 dB | 8.251（street-outdoor-c00-mba） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.389 × | 2.284（street-outdoor-c00-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.249 · Distill-MOS Δ +0.249 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.249 | +0.046（street-outdoor-c03-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.249 | +0.143（street-outdoor-c07-mba） |

送人耳：
- street-outdoor-c04-mba：两个评分模型都说不比原声差，但 Δ 相差 0.32 ≥ 0.3

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `footsteps`（event）：差 +0.431（有 3 段 1.111 · 没有 7 段 0.680）
- V-EAT-ASR · `talker`（speech）：差 +0.431（有 3 段 1.111 · 没有 7 段 0.680）
- V-EAT-ASR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `car-horn`（event）：差 +0.026（有 4 段 1.518 · 没有 6 段 1.492）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.016（有 3 段 0.031 · 没有 7 段 0.016）
- V-SIM-GATE · `car-horn`（event）：差 +0.009（有 4 段 0.026 · 没有 6 段 0.017）
- V-SIM-GATE · `footsteps`（event）：差 +0.005（有 3 段 0.024 · 没有 7 段 0.019）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +0.490（有 3 段 4.417 · 没有 7 段 3.927）
- V-PUMPV · `talker`（speech）：差 +0.028（有 3 段 4.094 · 没有 7 段 4.066）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（street-outdoor-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.013（street-outdoor-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（street-outdoor-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.207 dB | 0.551（street-outdoor-c01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.470 比例 | 0.646（street-outdoor-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -14.306 dB | -8.999（street-outdoor-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.979 dB² | 5.911（street-outdoor-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（street-outdoor-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（street-outdoor-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.841 dB | 3.915（street-outdoor-c05-mba） | 6.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（street-outdoor-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | 0.000 dB | 0.000（street-outdoor-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 0.219 dB | 1.575（street-outdoor-c00-mba） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（street-outdoor-c00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（street-outdoor-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（street-outdoor-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `car-horn`（event）：差 +0.197（有 4 段 0.588 · 没有 6 段 0.391）
- V-SIM-GATE · `footsteps`（event）：差 +0.055（有 3 段 0.509 · 没有 7 段 0.454）
- V-SIM-GATE · `talker`（speech）：差 +0.025（有 3 段 0.488 · 没有 7 段 0.463）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）

### street-outdoor · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.415 dB | -0.933（street-outdoor-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | -0.543 百分点 | 3.333（street-outdoor-c02-mba） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 1.566 dB | 2.786（street-outdoor-c01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.036 比例 | 0.090（street-outdoor-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.932 dB | -4.678（street-outdoor-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.470 dB² | 10.213（street-outdoor-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 25.000 个/分钟 | 60.000（street-outdoor-c09-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.025 Δ | 0.016（street-outdoor-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.703 dB | 4.438（street-outdoor-c05-mba） | 6.000 | 过 |
| V-NR |  | 4 / 6 / 0 | -5.364 dB | -1.566（street-outdoor-c03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | -3.437 dB | -2.465（street-outdoor-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.445 dB | 4.137（street-outdoor-c02-mba） | 10.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 1.712 × | 3.207（street-outdoor-c07-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.245 · Distill-MOS Δ +0.216 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.245 | +0.109（street-outdoor-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.216 | +0.162（street-outdoor-c01-mba） |

送人耳：
- street-outdoor-c04-mba：两个评分模型都说不比原声差，但 Δ 相差 0.32 ≥ 0.3

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `footsteps`（event）：差 +0.458（有 3 段 -0.222 · 没有 7 段 -0.680）
- V-EAT-ASR · `talker`（speech）：差 +0.458（有 3 段 -0.222 · 没有 7 段 -0.680）
- V-EAT-ASR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`（event）：差 +0.016（有 4 段 0.046 · 没有 6 段 0.030）
- V-SIM-GATE · `talker`（speech）：差 +0.005（有 3 段 0.040 · 没有 7 段 0.035）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +0.442（有 3 段 4.779 · 没有 7 段 4.338）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`（event）：差 +0.274（有 3 段 1.904 · 没有 7 段 1.630）
- V-MUSIC · `talker`（speech）：差 +0.028（有 3 段 1.732 · 没有 7 段 1.704）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.944 dB | -3.190（street-outdoor-s04-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | 0.090 百分点 | 3.571（street-outdoor-s04-ext） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 3.939 dB | 12.340（street-outdoor-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.043 比例 | 0.071（street-outdoor-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -7.948 dB | -4.065（street-outdoor-s05-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.618 dB² | 10.538（street-outdoor-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 29.167 个/分钟 | 45.000（street-outdoor-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.030 Δ | 0.017（street-outdoor-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 5.909 dB | 8.721（street-outdoor-s05-ext） | 6.000 | 不过 |
| V-NR |  | 2 / 4 / 0 | -5.893 dB | -5.368（street-outdoor-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 1 / 5 | -5.418 dB | -5.418（street-outdoor-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.473 dB | 5.691（street-outdoor-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.207 × | 1.835（street-outdoor-s03-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.124 · Distill-MOS Δ +0.218 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.124 | -0.052（street-outdoor-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.218 | +0.164（street-outdoor-s04-ext） |

送人耳：
- street-outdoor-s01-ext：一个说好一个说差：SCOREQ Δ -0.05 · Distill-MOS Δ +0.17

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-terrace`（bed）：差 +0.666（有 3 段 -2.277 · 没有 3 段 -1.611）
- V-SPLVL · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-terrace`（bed）：差 +2.201（有 3 段 1.190 · 没有 3 段 -1.010）
- V-EAT-ASR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-street`（bed）：差 +3.585（有 3 段 5.731 · 没有 3 段 2.146）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.031（有 3 段 0.058 · 没有 3 段 0.027）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`（bed）：差 +1.545（有 3 段 6.391 · 没有 3 段 4.845）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-terrace`（bed）：差 +0.414（有 3 段 6.116 · 没有 3 段 5.702）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-terrace`（bed）：差 +0.733（有 3 段 -5.526 · 没有 3 段 -6.259）
- V-NR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### street-outdoor · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（street-outdoor-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.000 dB | -0.001（street-outdoor-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（street-outdoor-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 2.049 dB | 9.685（street-outdoor-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.494 比例 | 0.746（street-outdoor-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.686 dB | -8.554（street-outdoor-s05-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 8.250 dB² | 32.315（street-outdoor-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（street-outdoor-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（street-outdoor-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 4.750 dB | 8.230（street-outdoor-s05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（street-outdoor-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 1 / 5 | 0.000 dB | 0.000（street-outdoor-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.033 dB | 0.077（street-outdoor-s00-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（street-outdoor-s00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（street-outdoor-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（street-outdoor-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-street`（bed）：差 +3.086（有 3 段 3.592 · 没有 3 段 0.506）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`（bed）：差 +0.113（有 3 段 0.550 · 没有 3 段 0.437）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`（bed）：差 +8.877（有 3 段 12.688 · 没有 3 段 3.812）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-terrace`（bed）：差 +1.091（有 3 段 5.296 · 没有 3 段 4.205）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### street-outdoor · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.375 dB | -0.543（street-outdoor-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（street-outdoor-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 2 / 4 / 0 | 3.661 dB | 6.791（street-outdoor-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.033 比例 | 0.059（street-outdoor-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -8.768 dB | -5.054（street-outdoor-s02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.708 dB² | 30.118（street-outdoor-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 40.833 个/分钟 | 80.000（street-outdoor-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.018 Δ | 0.000（street-outdoor-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 5.247 dB | 8.845（street-outdoor-s05-ext） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.436 dB | -1.268（street-outdoor-s03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 1 / 5 | -2.105 dB | -2.105（street-outdoor-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.709 dB | 1.223（street-outdoor-s05-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.139 × | 1.753（street-outdoor-s04-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.057 · Distill-MOS Δ +0.173 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.057 | -0.011（street-outdoor-s01-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.173 | +0.114（street-outdoor-s00-ext） |

送人耳：
- street-outdoor-s01-ext：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.15

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-street`（bed）：差 +0.642（有 3 段 3.982 · 没有 3 段 3.340）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`（bed）：差 +0.009（有 3 段 0.038 · 没有 3 段 0.029）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`（bed）：差 +8.234（有 3 段 12.826 · 没有 3 段 4.591）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-terrace`（bed）：差 +1.809（有 3 段 6.151 · 没有 3 段 4.342）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-street`（bed）：差 +0.630（有 3 段 -2.121 · 没有 3 段 -2.751）
- V-NR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### street-outdoor · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.543 dB | -3.231（street-outdoor-s04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 5 / 1 / 0 | -3.452 百分点 | 9.375（street-outdoor-s05-mba） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.377 dB | 5.382（street-outdoor-s03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.061 比例 | 0.103（street-outdoor-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.370 dB | -7.165（street-outdoor-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.633 dB² | 14.195（street-outdoor-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 35.000 个/分钟 | 55.000（street-outdoor-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.021 Δ | 0.009（street-outdoor-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.087 dB | 8.753（street-outdoor-s02-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -6.098 dB | -5.663（street-outdoor-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 1 / 5 | -5.857 dB | -5.857（street-outdoor-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.861 dB | 6.085（street-outdoor-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.284 × | 2.179（street-outdoor-s03-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.117 · Distill-MOS Δ +0.222 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.117 | +0.006（street-outdoor-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.222 | +0.127（street-outdoor-s01-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-terrace`（bed）：差 +0.184（有 3 段 -2.635 · 没有 3 段 -2.451）
- V-SPLVL · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-terrace`（bed）：差 +5.217（有 3 段 -0.843 · 没有 3 段 -6.061）
- V-EAT-ASR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-street`（bed）：差 +0.269（有 3 段 2.512 · 没有 3 段 2.243）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.019（有 3 段 0.071 · 没有 3 段 0.052）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`（bed）：差 +1.266（有 3 段 9.266 · 没有 3 段 8.000）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +1.596（有 3 段 6.885 · 没有 3 段 5.289）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-terrace`（bed）：差 +0.608（有 3 段 -5.794 · 没有 3 段 -6.402）
- V-NR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-street`（bed）：差 +0.341（有 3 段 1.455 · 没有 3 段 1.114）
- V-MUSIC · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### street-outdoor · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（street-outdoor-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（street-outdoor-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（street-outdoor-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 0.525 dB | 3.591（street-outdoor-s03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.737 比例 | 0.830（street-outdoor-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.466 dB | -11.837（street-outdoor-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.783 dB² | 15.196（street-outdoor-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（street-outdoor-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（street-outdoor-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 5.193 dB | 8.320（street-outdoor-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（street-outdoor-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 1 / 5 | 0.000 dB | 0.000（street-outdoor-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.036 dB | 0.133（street-outdoor-s05-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（street-outdoor-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（street-outdoor-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（street-outdoor-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-street`（bed）：差 +0.347（有 3 段 0.698 · 没有 3 段 0.351）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.005（有 3 段 0.739 · 没有 3 段 0.734）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`（bed）：差 +3.263（有 3 段 10.414 · 没有 3 段 7.151）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +2.776（有 3 段 6.581 · 没有 3 段 3.805）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### street-outdoor · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.596 dB | -0.827（street-outdoor-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 3 / 3 / 0 | 4.503 百分点 | 14.706（street-outdoor-s03-mba） | 1.000 | 不过 |
| V-ONSET |  | 3 / 3 / 0 | 3.022 dB | 4.469（street-outdoor-s03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.051 比例 | 0.107（street-outdoor-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.237 dB | -7.521（street-outdoor-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 14.837 dB² | 26.807（street-outdoor-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 40.000 个/分钟 | 70.000（street-outdoor-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.013 Δ | -0.000（street-outdoor-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 5.822 dB | 8.711（street-outdoor-s02-mba） | 6.000 | 不过 |
| V-NR |  | 0 / 6 / 0 | -2.879 dB | -1.471（street-outdoor-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 1 / 5 | -2.663 dB | -2.663（street-outdoor-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.678 dB | 1.512（street-outdoor-s02-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.181 × | 1.719（street-outdoor-s04-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.129 · Distill-MOS Δ +0.212 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.129 | +0.043（street-outdoor-s04-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.212 | +0.137（street-outdoor-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-street`（bed）：差 +4.839（有 3 段 6.922 · 没有 3 段 2.083）
- V-EAT-ASR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-EAT-ASR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`（bed）：差 +0.199（有 3 段 3.122 · 没有 3 段 2.923）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.024（有 3 段 0.063 · 没有 3 段 0.039）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`（bed）：差 +0.201（有 3 段 14.938 · 没有 3 段 14.737）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +0.935（有 3 段 6.290 · 没有 3 段 5.354）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-street`（bed）：差 +1.162（有 3 段 -2.298 · 没有 3 段 -3.460）
- V-NR · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### hotel-room · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（hotel-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.183 dB | -0.436（hotel-room-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.625 百分点 | 6.250（hotel-room-c07-ext） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 0.992 dB | 2.291（hotel-room-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.030 比例 | 0.089（hotel-room-c06-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.449 dB | 1.940（hotel-room-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.229 dB² | 2.712（hotel-room-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 7.500 个/分钟 | 30.000（hotel-room-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.007 Δ | 0.001（hotel-room-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.133 dB | 0.386（hotel-room-c09-ext） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.214 × | 1.786（hotel-room-c03-ext） | 2.000 | 过 |
| V-NR |  | 7 / 3 / 0 | -6.303 dB | -3.378（hotel-room-c09-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.094 · Distill-MOS Δ +0.160 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.094 | +0.007（hotel-room-c08-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.160 | +0.072（hotel-room-c07-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`（bed）：差 +1.562（有 4 段 1.562 · 没有 6 段 0.000）
- V-EAT-ASR · `door-knock`（event）：差 +1.562（有 4 段 1.562 · 没有 6 段 0.000）
- V-EAT-ASR · `footsteps`（event）：差 +1.250（有 5 段 1.250 · 没有 5 段 0.000）
- V-EAT-ASR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.047（有 3 段 0.062 · 没有 7 段 0.016）
- V-SIM-GATE · `footsteps`（event）：差 +0.025（有 5 段 0.042 · 没有 5 段 0.017）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.006（有 4 段 0.033 · 没有 6 段 0.027）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +0.666（有 3 段 1.696 · 没有 7 段 1.029）
- V-PUMPV · `footsteps`（event）：差 +0.427（有 5 段 1.443 · 没有 5 段 1.016）
- V-PUMPV · `bed-ace`（bed）：差 +0.369（有 4 段 1.450 · 没有 6 段 1.082）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +1.425（有 4 段 -5.448 · 没有 6 段 -6.873）
- V-NR · `door-knock`（event）：差 +1.325（有 4 段 -5.508 · 没有 6 段 -6.833）
- V-NR · `footsteps`（event）：差 +0.202（有 5 段 -6.202 · 没有 5 段 -6.404）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（hotel-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.005（hotel-room-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | -0.035 dB | 0.249（hotel-room-c00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.053 比例 | 0.132（hotel-room-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -1.479 dB | -0.199（hotel-room-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 0.188 dB² | 1.007（hotel-room-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（hotel-room-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（hotel-room-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.306 dB | 0.859（hotel-room-c05-ext） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（hotel-room-c00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（hotel-room-c00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（hotel-room-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（hotel-room-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `door`（event）：差 +0.031（有 3 段 0.074 · 没有 7 段 0.044）
- V-SIM-GATE · `bed-room`（bed）：差 +0.027（有 6 段 0.064 · 没有 4 段 0.036）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +0.361（有 4 段 0.404 · 没有 6 段 0.044）
- V-PUMPV · `footsteps`（event）：差 +0.349（有 5 段 0.363 · 没有 5 段 0.014）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（hotel-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.028 dB | -0.070（hotel-room-c08-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.261 dB | 0.983（hotel-room-c00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 9 / 1 / 0 | 0.026 比例 | 0.082（hotel-room-c06-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.485 dB | 1.154（hotel-room-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 0.335 dB² | 1.036（hotel-room-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 5.500 个/分钟 | 35.000（hotel-room-c07-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.001 Δ | -0.004（hotel-room-c05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.294 dB | 0.861（hotel-room-c05-ext） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.312 × | 4.534（hotel-room-c03-ext） | 2.000 | 不过 |
| V-NR |  | 2 / 8 / 0 | -2.808 dB | -0.051（hotel-room-c09-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 5 | SCOREQ Δ +0.069 · Distill-MOS Δ +0.022 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.069 | +0.005（hotel-room-c07-ext） |
| V-NRMOS-DISTILL | 5 / 5 / 0 | +0.022 | -0.041（hotel-room-c07-ext） |

送人耳：
- hotel-room-c04-ext：一个说好一个说差：SCOREQ Δ +0.10 · Distill-MOS Δ -0.02
- hotel-room-c05-ext：一个说好一个说差：SCOREQ Δ +0.05 · Distill-MOS Δ -0.02
- hotel-room-c06-ext：一个说好一个说差：SCOREQ Δ +0.03 · Distill-MOS Δ -0.00
- hotel-room-c07-ext：一个说好一个说差：SCOREQ Δ +0.01 · Distill-MOS Δ -0.04
- hotel-room-c09-ext：一个说好一个说差：SCOREQ Δ +0.05 · Distill-MOS Δ -0.03

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `door`（event）：差 +0.030（有 3 段 0.047 · 没有 7 段 0.017）
- V-SIM-GATE · `footsteps`（event）：差 +0.013（有 5 段 0.032 · 没有 5 段 0.019）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.012（有 4 段 0.033 · 没有 6 段 0.021）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +0.439（有 5 段 0.555 · 没有 5 段 0.116）
- V-PUMPV · `bed-ace`（bed）：差 +0.228（有 4 段 0.472 · 没有 6 段 0.244）
- V-PUMPV · `door`（event）：差 +0.017（有 3 段 0.348 · 没有 7 段 0.330）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +0.511（有 6 段 1.517 · 没有 4 段 1.006）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `door-knock`（event）：差 +2.076（有 4 段 -1.563 · 没有 6 段 -3.638）
- V-NR · `bed-ace`（bed）：差 +2.028（有 4 段 -1.591 · 没有 6 段 -3.619）
- V-NR · `footsteps`（event）：差 +1.945（有 5 段 -1.835 · 没有 5 段 -3.781）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（hotel-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.164 dB | -0.430（hotel-room-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.573 百分点 | 13.889（hotel-room-c06-mba） | 1.000 | 不过 |
| V-ONSET |  | 10 / 0 / 0 | 1.078 dB | 2.671（hotel-room-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.037 比例 | 0.085（hotel-room-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.142 dB | 1.574（hotel-room-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.596 dB² | 5.170（hotel-room-c04-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 8.000 个/分钟 | 20.000（hotel-room-c07-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.014 Δ | 0.004（hotel-room-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.183 dB | 0.531（hotel-room-c09-mba） | 6.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 1.745 × | 3.423（hotel-room-c03-mba） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.660 dB | -1.998（hotel-room-c09-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.189 · Distill-MOS Δ +0.237 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.189 | +0.020（hotel-room-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.237 | +0.139（hotel-room-c07-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `door`（event）：差 +5.796（有 3 段 4.630 · 没有 7 段 -1.166）
- V-EAT-ASR · `bed-ace`（bed）：差 +4.833（有 4 段 3.472 · 没有 6 段 -1.361）
- V-EAT-ASR · `footsteps`（event）：差 +4.410（有 5 段 2.778 · 没有 5 段 -1.633）
- V-EAT-ASR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.046（有 3 段 0.069 · 没有 7 段 0.023）
- V-SIM-GATE · `footsteps`（event）：差 +0.018（有 5 段 0.046 · 没有 5 段 0.028）
- V-SIM-GATE · `bed-room`（bed）：差 +0.001（有 6 段 0.037 · 没有 4 段 0.036）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.279（有 5 段 2.235 · 没有 5 段 0.956）
- V-PUMPV · `bed-ace`（bed）：差 +1.238（有 4 段 2.339 · 没有 6 段 1.101）
- V-PUMPV · `door`（event）：差 +0.794（有 3 段 2.151 · 没有 7 段 1.358）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +0.970（有 6 段 2.133 · 没有 4 段 1.163）
- V-MUSIC · `door-knock`（event）：差 +0.089（有 4 段 1.798 · 没有 6 段 1.710）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `door-knock`（event）：差 +1.827（有 4 段 -6.564 · 没有 6 段 -8.391）
- V-NR · `door`（event）：差 +1.494（有 3 段 -6.614 · 没有 7 段 -8.108）
- V-NR · `bed-ace`（bed）：差 +0.721（有 4 段 -7.227 · 没有 6 段 -7.948）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（hotel-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.000 dB | -0.002（hotel-room-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.001 dB | 0.188（hotel-room-c02-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.056 比例 | 0.143（hotel-room-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.266 dB | -0.878（hotel-room-c01-mba） | 4.000 | 过 |
| V-PUMPV |  | 9 / 1 / 0 | 0.350 dB² | 2.565（hotel-room-c04-mba） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（hotel-room-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（hotel-room-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.359 dB | 1.310（hotel-room-c06-mba） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（hotel-room-c00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（hotel-room-c00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（hotel-room-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（hotel-room-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-room`（bed）：差 +0.023（有 6 段 0.065 · 没有 4 段 0.042）
- V-SIM-GATE · `door`（event）：差 +0.005（有 3 段 0.059 · 没有 7 段 0.054）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +0.798（有 4 段 0.829 · 没有 6 段 0.031）
- V-PUMPV · `footsteps`（event）：差 +0.668（有 5 段 0.684 · 没有 5 段 0.016）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（hotel-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.057 dB | -0.131（hotel-room-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | -0.816 百分点 | 0.000（hotel-room-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 10 / 0 / 0 | 0.560 dB | 1.171（hotel-room-c00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.023 比例 | 0.070（hotel-room-c06-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.249 dB | 1.453（hotel-room-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 0.677 dB² | 2.873（hotel-room-c04-mba） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 5.000 个/分钟 | 15.000（hotel-room-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.008 Δ | -0.001（hotel-room-c05-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.361 dB | 1.262（hotel-room-c06-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.399 × | 3.444（hotel-room-c00-mba） | 2.000 | 不过 |
| V-NR |  | 2 / 8 / 0 | -3.206 dB | -0.047（hotel-room-c09-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.183 · Distill-MOS Δ +0.162 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.183 | -0.014（hotel-room-c08-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.162 | +0.031（hotel-room-c07-mba） |

送人耳：
- hotel-room-c08-mba：一个说好一个说差：SCOREQ Δ -0.01 · Distill-MOS Δ +0.12

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `door`（event）：差 +0.018（有 3 段 0.036 · 没有 7 段 0.018）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.011（有 4 段 0.030 · 没有 6 段 0.019）
- V-SIM-GATE · `footsteps`（event）：差 +0.007（有 5 段 0.027 · 没有 5 段 0.020）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +0.865（有 4 段 1.197 · 没有 6 段 0.331）
- V-PUMPV · `footsteps`（event）：差 +0.860（有 5 段 1.107 · 没有 5 段 0.248）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +0.651（有 6 段 1.659 · 没有 4 段 1.008）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `door-knock`（event）：差 +2.114（有 4 段 -1.937 · 没有 6 段 -4.051）
- V-NR · `bed-ace`（bed）：差 +1.995（有 4 段 -2.009 · 没有 6 段 -4.003）
- V-NR · `footsteps`（event）：差 +1.963（有 5 段 -2.224 · 没有 5 段 -4.187）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（hotel-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.551 dB | -0.984（hotel-room-s05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 4 / 2 / 0 | 2.557 百分点 | 9.091（hotel-room-s03-ext） | 1.000 | 不过 |
| V-ONSET |  | 4 / 2 / 0 | 2.063 dB | 4.735（hotel-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.028 比例 | 0.064（hotel-room-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.891 dB | -0.205（hotel-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 2.898 dB² | 7.077（hotel-room-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 13.333 个/分钟 | 30.000（hotel-room-s05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.018（hotel-room-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.738 dB | 3.881（hotel-room-s01-ext） | 6.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.099 × | 1.557（hotel-room-s03-ext） | 2.000 | 过 |
| V-NR |  | 2 / 4 / 0 | -5.743 dB | -5.031（hotel-room-s00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.199 · Distill-MOS Δ +0.187 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.199 | +0.112（hotel-room-s02-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.187 | +0.137（hotel-room-s01-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `footsteps`（event）：差 +0.005（有 3 段 2.066 · 没有 3 段 2.061）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.023（有 3 段 0.039 · 没有 3 段 0.016）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（hotel-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（hotel-room-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.216 dB | 1.343（hotel-room-s00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.187 比例 | 0.238（hotel-room-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -6.131 dB | -3.102（hotel-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 5 / 1 / 0 | 0.627 dB² | 2.608（hotel-room-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（hotel-room-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（hotel-room-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.734 dB | 7.013（hotel-room-s01-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（hotel-room-s00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（hotel-room-s00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（hotel-room-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（hotel-room-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`（event）：差 +2.703（有 3 段 5.086 · 没有 3 段 2.382）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（hotel-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.099 dB | -0.160（hotel-room-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 1.332 dB | 3.023（hotel-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.029 比例 | 0.096（hotel-room-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -3.565 dB | -0.807（hotel-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 1.484 dB² | 4.001（hotel-room-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 13.333 个/分钟 | 30.000（hotel-room-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.015 Δ | 0.005（hotel-room-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.831 dB | 7.253（hotel-room-s01-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.078 × | 1.263（hotel-room-s04-ext） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | -1.252 dB | -0.442（hotel-room-s01-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 0 / 2 | SCOREQ Δ +0.106 · Distill-MOS Δ +0.044 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.106 | -0.040（hotel-room-s05-ext） |
| V-NRMOS-DISTILL | 5 / 1 / 0 | +0.044 | -0.006（hotel-room-s01-ext） |

送人耳：
- hotel-room-s00-ext：两个评分模型都说不比原声差，但 Δ 相差 0.32 ≥ 0.3
- hotel-room-s01-ext：一个说好一个说差：SCOREQ Δ +0.09 · Distill-MOS Δ -0.01
- hotel-room-s05-ext：一个说好一个说差：SCOREQ Δ -0.04 · Distill-MOS Δ +0.02

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`（event）：差 +2.660（有 3 段 5.161 · 没有 3 段 2.501）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +1.444（有 3 段 -0.530 · 没有 3 段 -1.974）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（hotel-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.908 dB | -2.673（hotel-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -0.490 百分点 | 0.000（hotel-room-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 4 / 2 / 0 | 2.603 dB | 4.666（hotel-room-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.048 比例 | 0.089（hotel-room-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.024 dB | 0.255（hotel-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.432 dB² | 54.621（hotel-room-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 22.500 个/分钟 | 45.000（hotel-room-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.039 Δ | 0.032（hotel-room-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.181 dB | 6.383（hotel-room-s01-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.182 × | 1.611（hotel-room-s03-mba） | 2.000 | 过 |
| V-NR |  | 4 / 2 / 0 | -7.804 dB | -4.466（hotel-room-s00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.185 · Distill-MOS Δ +0.248 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.185 | -0.015（hotel-room-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.248 | +0.175（hotel-room-s01-mba） |

送人耳：
- hotel-room-s02-mba：一个说好一个说差：SCOREQ Δ -0.02 · Distill-MOS Δ +0.27

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `footsteps`（event）：差 +0.901（有 3 段 3.054 · 没有 3 段 2.152）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.028（有 3 段 0.062 · 没有 3 段 0.034）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`（event）：差 +3.316（有 3 段 3.839 · 没有 3 段 0.523）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +0.364（有 3 段 -7.622 · 没有 3 段 -7.986）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（hotel-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.148 dB | -0.888（hotel-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.151 dB | 1.586（hotel-room-s05-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.220 比例 | 0.347（hotel-room-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -7.949 dB | -3.335（hotel-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 1 / 0 | 1.888 dB² | 9.594（hotel-room-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（hotel-room-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（hotel-room-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.897 dB | 7.061（hotel-room-s01-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（hotel-room-s00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（hotel-room-s00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（hotel-room-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（hotel-room-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`（event）：差 +2.254（有 3 段 4.024 · 没有 3 段 1.770）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（hotel-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.371 dB | -1.115（hotel-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（hotel-room-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 4 / 2 / 0 | 1.727 dB | 3.787（hotel-room-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.029 比例 | 0.088（hotel-room-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.589 dB | -0.249（hotel-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 2.524 dB² | 7.549（hotel-room-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 18.333 个/分钟 | 30.000（hotel-room-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.021 Δ | 0.007（hotel-room-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.262 dB | 7.427（hotel-room-s01-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.133 × | 1.494（hotel-room-s04-mba） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | -1.627 dB | -0.593（hotel-room-s01-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 5 / 0 / 1 | SCOREQ Δ +0.098 · Distill-MOS Δ +0.137 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 5 / 1 / 0 | +0.098 | -0.002（hotel-room-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.137 | +0.067（hotel-room-s01-mba） |

送人耳：
- hotel-room-s02-mba：一个说好一个说差：SCOREQ Δ -0.00 · Distill-MOS Δ +0.19

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`（event）：差 +2.088（有 3 段 4.306 · 没有 3 段 2.218）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +1.852（有 3 段 -0.701 · 没有 3 段 -2.553）
- V-NR · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### library-classroom · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（library-classroom-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.241 dB | -0.424（library-classroom-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 8 / 0 / 2 | 0.634 dB | 2.239（library-classroom-c06-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.035 比例 | 0.085（library-classroom-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.050 dB | 2.835（library-classroom-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.516 dB² | 3.923（library-classroom-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 10.500 个/分钟 | 35.000（library-classroom-c01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.003 Δ | -0.017（library-classroom-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.945 dB | 3.243（library-classroom-c08-ext） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.408 × | 2.694（library-classroom-c09-ext） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -6.740 dB | -3.525（library-classroom-c02-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 9 / 0 / 1 | SCOREQ Δ +0.143 · Distill-MOS Δ +0.223 | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 9 / 1 / 0 | +0.143 | -0.140（library-classroom-c03-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.223 | +0.111（library-classroom-c07-ext） |

送人耳：
- library-classroom-c03-ext：一个说好一个说差：SCOREQ Δ -0.14 · Distill-MOS Δ +0.19

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `page-turn`（event）：差 +0.108（有 7 段 1.548 · 没有 3 段 1.440）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `talker`（speech）：差 +0.914（有 5 段 -6.283 · 没有 5 段 -7.197）
- V-NR · `page-turn`（event）：差 +0.276（有 7 段 -6.657 · 没有 3 段 -6.933）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · external-mic · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（library-classroom-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.006（library-classroom-c08-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 8 / 0 / 2 | -0.103 dB | 0.205（library-classroom-c06-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.090 比例 | 0.220（library-classroom-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.637 dB | -0.005（library-classroom-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.062 dB² | 0.249（library-classroom-c03-ext） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（library-classroom-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（library-classroom-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 3.368 dB | 8.485（library-classroom-c07-ext） | 6.000 | 不过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（library-classroom-c00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（library-classroom-c00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（library-classroom-c00-ext） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（library-classroom-c00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `talker`（speech）：差 +0.019（有 5 段 0.100 · 没有 5 段 0.081）
- V-SIM-GATE · `page-turn`（event）：差 +0.015（有 7 段 0.095 · 没有 3 段 0.080）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `page-turn`（event）：差 +4.807（有 7 段 4.811 · 没有 3 段 0.003）
- V-TIMBRE · `talker`（speech）：差 +3.003（有 5 段 4.870 · 没有 5 段 1.867）
- V-TIMBRE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · external-mic · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（library-classroom-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.027 dB | -0.050（library-classroom-c01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-c00-ext） | 1.000 | 过 |
| V-ONSET |  | 8 / 0 / 2 | 0.093 dB | 0.483（library-classroom-c04-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.033 比例 | 0.077（library-classroom-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -1.309 dB | 1.535（library-classroom-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.284 dB² | 0.907（library-classroom-c09-ext） | 1.000 | 过 |
| V-CLICK |  | 6 / 4 / 0 | 5.500 个/分钟 | 20.000（library-classroom-c01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | -0.001 Δ | -0.006（library-classroom-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.262 dB | 8.502（library-classroom-c07-ext） | 6.000 | 不过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.216 × | 1.975（library-classroom-c05-ext） | 2.000 | 过 |
| V-NR |  | 2 / 8 / 0 | -3.059 dB | -0.516（library-classroom-c08-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 1 / 3 | SCOREQ Δ +0.064 · Distill-MOS Δ +0.052 | | Δ ≥ 0 | 不过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 8 / 2 / 0 | +0.064 | -0.153（library-classroom-c03-ext） |
| V-NRMOS-DISTILL | 7 / 3 / 0 | +0.052 | -0.056（library-classroom-c03-ext） |

送人耳：
- library-classroom-c00-ext：一个说好一个说差：SCOREQ Δ +0.00 · Distill-MOS Δ -0.02
- library-classroom-c04-ext：一个说好一个说差：SCOREQ Δ -0.03 · Distill-MOS Δ +0.08
- library-classroom-c06-ext：一个说好一个说差：SCOREQ Δ +0.00 · Distill-MOS Δ -0.03

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `page-turn`（event）：差 +4.634（有 7 段 4.652 · 没有 3 段 0.018）
- V-TIMBRE · `talker`（speech）：差 +2.766（有 5 段 4.645 · 没有 5 段 1.879）
- V-TIMBRE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `page-turn`（event）：差 +4.819（有 7 段 -1.613 · 没有 3 段 -6.432）
- V-NR · `talker`（speech）：差 +2.787（有 5 段 -1.665 · 没有 5 段 -4.452）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `talker`（speech）：差 +0.085（有 5 段 0.022 · 没有 5 段 0.106）
- V-NRMOS-SCOREQ · `page-turn`（event）：差 +0.083（有 7 段 0.039 · 没有 3 段 0.122）
- V-NRMOS-SCOREQ · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `page-turn`（event）：差 +0.057（有 7 段 0.035 · 没有 3 段 0.092）
- V-NRMOS-DISTILL · `talker`（speech）：差 +0.041（有 5 段 0.032 · 没有 5 段 0.073）
- V-NRMOS-DISTILL · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（library-classroom-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.262 dB | -0.518（library-classroom-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.014 百分点 | 3.846（library-classroom-c08-mba） | 1.000 | 不过 |
| V-ONSET |  | 8 / 0 / 2 | 0.834 dB | 2.480（library-classroom-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.034 比例 | 0.103（library-classroom-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.564 dB | 2.624（library-classroom-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.778 dB² | 5.930（library-classroom-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 8 / 0 | 10.000 个/分钟 | 30.000（library-classroom-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.016 Δ | -0.006（library-classroom-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.243 dB | 0.730（library-classroom-c02-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.404 × | 2.177（library-classroom-c05-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.246 dB | -4.014（library-classroom-c02-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.279 · Distill-MOS Δ +0.253 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.279 | +0.099（library-classroom-c02-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.253 | +0.123（library-classroom-c02-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +1.510（有 5 段 0.769 · 没有 5 段 -0.741）
- V-EAT-ASR · `page-turn`（event）：差 +0.020（有 7 段 0.020 · 没有 3 段 0.000）
- V-EAT-ASR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `page-turn`（event）：差 +0.009（有 7 段 0.037 · 没有 3 段 0.027）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `page-turn`（event）：差 +1.099（有 7 段 2.107 · 没有 3 段 1.009）
- V-PUMPV · `talker`（speech）：差 +0.982（有 5 段 2.269 · 没有 5 段 1.287）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `page-turn`（event）：差 +0.126（有 7 段 1.442 · 没有 3 段 1.316）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `page-turn`（event）：差 +1.660（有 7 段 -7.748 · 没有 3 段 -9.408）
- V-NR · `talker`（speech）：差 +0.458（有 5 段 -8.017 · 没有 5 段 -8.475）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · mba-builtin · passthrough — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 27.667 ms | 27.667（library-classroom-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.001 dB | -0.010（library-classroom-c08-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 10 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-c00-mba） | 1.000 | 过 |
| V-ONSET |  | 8 / 0 / 2 | -0.035 dB | 0.052（library-classroom-c03-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.091 比例 | 0.243（library-classroom-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.827 dB | -0.381（library-classroom-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 10 / 0 / 0 | 0.121 dB² | 0.814（library-classroom-c03-mba） | 1.000 | 过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（library-classroom-c00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.000 Δ | 0.000（library-classroom-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.571 dB | 9.038（library-classroom-c08-mba） | 6.000 | 不过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.000 × | 1.000（library-classroom-c00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 10 / 0 | 0.000 dB | 0.000（library-classroom-c00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.000 | +0.000（library-classroom-c00-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.000 | +0.000（library-classroom-c00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `page-turn`（event）：差 +0.040（有 7 段 0.103 · 没有 3 段 0.063）
- V-SIM-GATE · `talker`（speech）：差 +0.039（有 5 段 0.110 · 没有 5 段 0.071）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +2.847（有 5 段 2.994 · 没有 5 段 0.147）
- V-TIMBRE · `page-turn`（event）：差 +2.236（有 7 段 2.241 · 没有 3 段 0.006）
- V-TIMBRE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · mba-builtin · ref — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（library-classroom-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.062 dB | -0.139（library-classroom-c01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 9 / 1 / 0 | 0.399 百分点 | 7.692（library-classroom-c08-mba） | 1.000 | 不过 |
| V-ONSET |  | 8 / 0 / 2 | 0.329 dB | 0.747（library-classroom-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.023 比例 | 0.067（library-classroom-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -1.327 dB | 2.139（library-classroom-c05-mba） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 0.645 dB² | 1.741（library-classroom-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 12.000 个/分钟 | 50.000（library-classroom-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.011 Δ | 0.000（library-classroom-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.500 dB | 9.161（library-classroom-c08-mba） | 6.000 | 不过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.517 × | 3.068（library-classroom-c05-mba） | 2.000 | 不过 |
| V-NR |  | 2 / 8 / 0 | -3.267 dB | -0.779（library-classroom-c08-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 10 / 0 / 0 | SCOREQ Δ +0.250 · Distill-MOS Δ +0.178 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 10 / 0 / 0 | +0.250 | +0.063（library-classroom-c01-mba） |
| V-NRMOS-DISTILL | 10 / 0 / 0 | +0.178 | +0.041（library-classroom-c03-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-EAT-ASR · `talker`（speech）：差 +2.279（有 5 段 1.538 · 没有 5 段 -0.741）
- V-EAT-ASR · `page-turn`（event）：差 +0.570（有 7 段 0.570 · 没有 3 段 0.000）
- V-EAT-ASR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-EAT-ASR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.059（有 5 段 0.674 · 没有 5 段 0.615）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +2.652（有 5 段 2.826 · 没有 5 段 0.174）
- V-TIMBRE · `page-turn`（event）：差 +2.088（有 7 段 2.127 · 没有 3 段 0.039）
- V-TIMBRE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `page-turn`（event）：差 +4.808（有 7 段 -1.825 · 没有 3 段 -6.633）
- V-NR · `talker`（speech）：差 +2.852（有 5 段 -1.841 · 没有 5 段 -4.693）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（library-classroom-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.546 dB | -0.964（library-classroom-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -2.002 百分点 | 0.000（library-classroom-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 1.594 dB | 2.685（library-classroom-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.030 比例 | 0.071（library-classroom-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.108 dB | 0.183（library-classroom-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 2.299 dB² | 4.316（library-classroom-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 11.667 个/分钟 | 20.000（library-classroom-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.016 Δ | 0.008（library-classroom-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.311 dB | 9.804（library-classroom-s03-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.243 × | 1.829（library-classroom-s03-ext） | 2.000 | 过 |
| V-NR |  | 3 / 3 / 0 | -6.150 dB | -3.707（library-classroom-s04-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.121 · Distill-MOS Δ +0.164 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.121 | +0.033（library-classroom-s04-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.164 | +0.120（library-classroom-s01-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +3.678（有 3 段 4.150 · 没有 3 段 0.472）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · external-mic · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（library-classroom-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（library-classroom-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.165 dB | 0.908（library-classroom-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.167 比例 | 0.286（library-classroom-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.053 dB | -2.949（library-classroom-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 0 / 0 | 0.325 dB² | 0.953（library-classroom-s04-ext） | 1.000 | 过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（library-classroom-s00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（library-classroom-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.314 dB | 8.494（library-classroom-s03-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（library-classroom-s00-ext） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（library-classroom-s00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（library-classroom-s00-ext） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（library-classroom-s00-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +4.028（有 3 段 5.328 · 没有 3 段 1.300）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · external-mic · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（library-classroom-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.049 dB | -0.104（library-classroom-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -1.361 百分点 | 0.000（library-classroom-s00-ext） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.431 dB | 1.758（library-classroom-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 0 / 0 | 0.020 比例 | 0.037（library-classroom-s05-ext） | 0.050 | 过 |
| V-WEAK |  | 6 / 0 / 0 | -2.343 dB | -1.285（library-classroom-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 3 / 3 / 0 | 0.731 dB² | 1.771（library-classroom-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 8.333 个/分钟 | 20.000（library-classroom-s03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.009 Δ | -0.001（library-classroom-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.043 dB | 8.820（library-classroom-s03-ext） | 6.000 | 不过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.274 × | 2.303（library-classroom-s04-ext） | 2.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -3.303 dB | -1.206（library-classroom-s02-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 4 / 2 / 0 | SCOREQ Δ +0.057 · Distill-MOS Δ +0.028 | | Δ ≥ 0 | 不过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 4 / 2 / 0 | +0.057 | -0.040（library-classroom-s05-ext） |
| V-NRMOS-DISTILL | 4 / 2 / 0 | +0.028 | -0.028（library-classroom-s05-ext） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-PUMPV · `keyboard`（event）：差 +0.004（有 3 段 0.733 · 没有 3 段 0.729）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +4.834（有 3 段 5.460 · 没有 3 段 0.627）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`（event）：差 +4.136（有 3 段 -1.235 · 没有 3 段 -5.371）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `keyboard`（event）：差 +0.041（有 3 段 0.037 · 没有 3 段 0.078）
- V-NRMOS-SCOREQ · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NRMOS-SCOREQ · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NRMOS-DISTILL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（library-classroom-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.989 dB | -2.425（library-classroom-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | -1.361 百分点 | 0.000（library-classroom-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 5 / 1 / 0 | 2.135 dB | 4.006（library-classroom-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.036 比例 | 0.053（library-classroom-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.867 dB | -1.301（library-classroom-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 3.729 dB² | 8.779（library-classroom-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 10.833 个/分钟 | 25.000（library-classroom-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.020（library-classroom-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.728 dB | 5.052（library-classroom-s03-mba） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.171 × | 2.036（library-classroom-s04-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -8.311 dB | -6.784（library-classroom-s01-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.204 · Distill-MOS Δ +0.220 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.204 | +0.143（library-classroom-s01-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.220 | +0.149（library-classroom-s04-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-ONSET · `keyboard`（event）：差 +1.229（有 3 段 2.750 · 没有 3 段 1.521）
- V-ONSET · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`（event）：差 +0.005（有 3 段 0.039 · 没有 3 段 0.034）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · mba-builtin · passthrough — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 27.667 ms | 27.667（library-classroom-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | 0.000 dB | 0.000（library-classroom-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.047 dB | 0.692（library-classroom-s01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.219 比例 | 0.331（library-classroom-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -6.617 dB | -4.598（library-classroom-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 1 / 0 | 1.806 dB² | 8.882（library-classroom-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 0 / 0 | 0.000 个/分钟 | 0.000（library-classroom-s00-mba） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.000 Δ | 0.000（library-classroom-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 3.290 dB | 7.943（library-classroom-s03-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.000 × | 1.000（library-classroom-s00-mba） | 2.000 | 过 |
| V-NR |  | 0 / 6 / 0 | 0.000 dB | 0.000（library-classroom-s00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.000 · Distill-MOS Δ +0.000 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.000 | +0.000（library-classroom-s00-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.000 | +0.000（library-classroom-s00-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +1.451（有 3 段 4.015 · 没有 3 段 2.564）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · mba-builtin · ref — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（library-classroom-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.178 dB | -0.284（library-classroom-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 6 / 0 / 0 | 0.000 百分点 | 0.000（library-classroom-s00-mba） | 1.000 | 过 |
| V-ONSET |  | 6 / 0 / 0 | 0.896 dB | 1.758（library-classroom-s01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.021 比例 | 0.029（library-classroom-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -3.329 dB | -1.502（library-classroom-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 2.517 dB² | 9.355（library-classroom-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 25.000 个/分钟 | 40.000（library-classroom-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.023 Δ | 0.016（library-classroom-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 3.374 dB | 8.450（library-classroom-s03-mba） | 6.000 | 不过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.593 × | 4.379（library-classroom-s04-mba） | 2.000 | 不过 |
| V-NR |  | 1 / 5 / 0 | -3.606 dB | -1.291（library-classroom-s03-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 6 / 0 / 0 | SCOREQ Δ +0.185 · Distill-MOS Δ +0.154 | | Δ ≥ 0 | 过 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 6 / 0 / 0 | +0.185 | +0.089（library-classroom-s02-mba） |
| V-NRMOS-DISTILL | 6 / 0 / 0 | +0.154 | +0.032（library-classroom-s04-mba） |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `keyboard`（event）：差 +0.003（有 3 段 0.022 · 没有 3 段 0.019）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +1.749（有 3 段 4.248 · 没有 3 段 2.500）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`（event）：差 +3.358（有 3 段 -1.927 · 没有 3 段 -5.285）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

