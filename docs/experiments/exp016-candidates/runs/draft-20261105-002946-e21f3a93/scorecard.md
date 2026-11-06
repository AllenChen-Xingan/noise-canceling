# 场景成绩单

共 180 个 场景 × 档 × 设备 × 系统 组合、1440 段：过 0 · 不过 180 · 待测 0。感知项全部 L0（只作约束，不当奖励）。

## 最差的场景

- **canteen · stress · external-mic · dfn3-ll** · 不过：否决项不过 V-LAT V-SPLVL · 不过的项 V-LAT V-SPLVL V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA
- **canteen · stress · external-mic · dpdfnet2** · 不过：否决项不过 V-LAT V-SPLVL · 不过的项 V-LAT V-SPLVL V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA
- **canteen · stress · mba-builtin · dfn3-ll** · 不过：否决项不过 V-LAT V-SPLVL · 不过的项 V-LAT V-SPLVL V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA

## 每个场景

### home-quiet · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（home-quiet-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.256 dB | -0.698（home-quiet-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 1.801 dB | 4.499（home-quiet-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.028 比例 | 0.064（home-quiet-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.697 dB | 1.194（home-quiet-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.141 dB² | 4.129（home-quiet-c02-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 5.000 个/分钟 | 20.000（home-quiet-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.028 Δ | -0.000（home-quiet-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.764 dB | 5.520（home-quiet-c08-ext） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.996 × | 10.039（home-quiet-c01-ext） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -7.725 dB | -6.236（home-quiet-c03-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-room`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `fan`（pattern）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `hvac`（pattern）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-room`（bed）：差 +1.409（有 7 段 2.224 · 没有 3 段 0.814）
- V-ONSET · `fan`（pattern）：差 +1.337（有 7 段 2.202 · 没有 3 段 0.865）
- V-ONSET · `keyboard`（event）：差 +1.226（有 6 段 2.291 · 没有 4 段 1.065）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`（bed）：差 +0.016（有 7 段 0.033 · 没有 3 段 0.017）
- V-SIM-GATE · `fan`（pattern）：差 +0.013（有 7 段 0.032 · 没有 3 段 0.019）
- V-SIM-GATE · `hvac`（pattern）：差 +0.011（有 7 段 0.031 · 没有 3 段 0.020）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +1.010（有 6 段 1.545 · 没有 4 段 0.536）
- V-PUMPV · `bed-room`（bed）：差 +0.811（有 7 段 1.385 · 没有 3 段 0.574）
- V-PUMPV · `fan`（pattern）：差 +0.773（有 7 段 1.373 · 没有 3 段 0.600）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +3.207（有 3 段 4.240 · 没有 7 段 1.034）
- V-MUSIC · `fan`（pattern）：差 +0.991（有 7 段 2.293 · 没有 3 段 1.302）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（home-quiet-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.096 dB | -0.154（home-quiet-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 5 / 0 | 3.656 dB | 7.513（home-quiet-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.063 比例 | 0.172（home-quiet-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.711 dB | 1.613（home-quiet-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 3.215 dB² | 10.110（home-quiet-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 7 / 3 / 0 | 1.500 个/分钟 | 5.000（home-quiet-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.033 Δ | -0.001（home-quiet-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.081 dB | 8.367（home-quiet-c08-ext） | 6.000 | 不过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.759 × | 9.299（home-quiet-c01-ext） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -9.446 dB | -7.577（home-quiet-c07-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `mouse`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-room`（bed）：差 +2.940（有 7 段 4.538 · 没有 3 段 1.597）
- V-ONSET · `hvac`（pattern）：差 +1.296（有 7 段 4.044 · 没有 3 段 2.748）
- V-ONSET · `fan`（pattern）：差 +0.504（有 7 段 3.807 · 没有 3 段 3.303）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`（pattern）：差 +0.053（有 7 段 0.079 · 没有 3 段 0.026）
- V-SIM-GATE · `bed-room`（bed）：差 +0.045（有 7 段 0.077 · 没有 3 段 0.032）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`（bed）：差 +2.772（有 7 段 4.047 · 没有 3 段 1.275）
- V-PUMPV · `hvac`（pattern）：差 +1.982（有 7 段 3.810 · 没有 3 段 1.828）
- V-PUMPV · `keyboard`（event）：差 +0.207（有 6 段 3.298 · 没有 4 段 3.091）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +2.635（有 3 段 2.926 · 没有 7 段 0.291）
- V-TIMBRE · `keyboard`（event）：差 +1.679（有 6 段 1.753 · 没有 4 段 0.074）
- V-TIMBRE · `hvac`（pattern）：差 +0.879（有 7 段 1.345 · 没有 3 段 0.466）
- V-TIMBRE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +2.973（有 3 段 3.840 · 没有 7 段 0.868）
- V-MUSIC · `fan`（pattern）：差 +1.232（有 7 段 2.129 · 没有 3 段 0.897）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-quiet-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.165 dB | -0.248（home-quiet-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 2.084 dB | 4.833（home-quiet-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.039 比例 | 0.123（home-quiet-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.232 dB | 1.947（home-quiet-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.777 dB² | 3.964（home-quiet-c00-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 3.000 个/分钟 | 10.000（home-quiet-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.018 Δ | -0.002（home-quiet-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.178 dB | 8.400（home-quiet-c08-ext） | 6.000 | 不过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.575 × | 5.950（home-quiet-c01-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 4 / 0 | -6.670 dB | -4.794（home-quiet-c00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### home-quiet · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（home-quiet-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.326 dB | -1.149（home-quiet-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 9 / 1 / 0 | 1.390 dB | 4.394（home-quiet-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.027 比例 | 0.043（home-quiet-c07-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.879 dB | 0.386（home-quiet-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.162 dB² | 4.494（home-quiet-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 6.000 个/分钟 | 25.000（home-quiet-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.036 Δ | 0.008（home-quiet-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.337 dB | 1.242（home-quiet-c08-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.938 × | 7.238（home-quiet-c01-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.115 dB | -4.122（home-quiet-c08-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-room`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `fan`（pattern）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `hvac`（pattern）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `keyboard`（event）：差 +1.007（有 6 段 1.793 · 没有 4 段 0.786）
- V-ONSET · `bed-room`（bed）：差 +0.560（有 7 段 1.558 · 没有 3 段 0.998）
- V-ONSET · `fan`（pattern）：差 +0.373（有 7 段 1.502 · 没有 3 段 1.129）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`（bed）：差 +0.016（有 7 段 0.032 · 没有 3 段 0.016）
- V-SIM-GATE · `fan`（pattern）：差 +0.009（有 7 段 0.030 · 没有 3 段 0.021）
- V-SIM-GATE · `hvac`（pattern）：差 +0.008（有 7 段 0.030 · 没有 3 段 0.021）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +1.249（有 6 段 1.662 · 没有 4 段 0.413）
- V-PUMPV · `fan`（pattern）：差 +0.617（有 7 段 1.347 · 没有 3 段 0.730）
- V-PUMPV · `bed-room`（bed）：差 +0.565（有 7 段 1.332 · 没有 3 段 0.767）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +1.880（有 3 段 3.254 · 没有 7 段 1.374）
- V-MUSIC · `fan`（pattern）：差 +0.398（有 7 段 2.057 · 没有 3 段 1.659）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `mouse`（event）：差 +1.308（有 3 段 -7.199 · 没有 7 段 -8.507）
- V-NR · `keyboard`（event）：差 +0.480（有 6 段 -7.923 · 没有 4 段 -8.402）
- V-NR · `bed-room`（bed）：差 +0.404（有 7 段 -7.993 · 没有 3 段 -8.397）
- V-NR · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（home-quiet-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.083 dB | -0.177（home-quiet-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 5 / 0 | 3.023 dB | 5.876（home-quiet-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.055 比例 | 0.110（home-quiet-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.837 dB | 0.870（home-quiet-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 2.730 dB² | 6.868（home-quiet-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 4.000 个/分钟 | 20.000（home-quiet-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.045 Δ | 0.009（home-quiet-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.714 dB | 4.970（home-quiet-c08-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.969 × | 8.382（home-quiet-c01-mba） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -9.375 dB | -8.038（home-quiet-c03-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `mouse`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-room`（bed）：差 +0.944（有 7 段 3.307 · 没有 3 段 2.363）
- V-ONSET · `hvac`（pattern）：差 +0.255（有 7 段 3.100 · 没有 3 段 2.845）
- V-ONSET · `mouse`（event）：差 +0.044（有 3 段 3.055 · 没有 7 段 3.010）
- V-ONSET · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`（pattern）：差 +0.028（有 7 段 0.063 · 没有 3 段 0.036）
- V-SIM-GATE · `bed-room`（bed）：差 +0.014（有 7 段 0.059 · 没有 3 段 0.045）
- V-SIM-GATE · `mouse`（event）：差 +0.013（有 3 段 0.065 · 没有 7 段 0.051）
- V-SIM-GATE · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`（bed）：差 +1.756（有 7 段 3.257 · 没有 3 段 1.500）
- V-PUMPV · `hvac`（pattern）：差 +0.346（有 7 段 2.834 · 没有 3 段 2.487）
- V-PUMPV · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`（bed）：差 +2.225（有 3 段 3.526 · 没有 7 段 1.301）
- V-MUSIC · `fan`（pattern）：差 +0.583（有 7 段 2.144 · 没有 3 段 1.560）
- V-MUSIC · `fridge-hum`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### home-quiet · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-quiet-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-quiet-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.168 dB | -0.269（home-quiet-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 2.063 dB | 5.024（home-quiet-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.044 比例 | 0.104（home-quiet-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.448 dB | 1.625（home-quiet-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 2.024 dB² | 4.810（home-quiet-c00-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 8.500 个/分钟 | 25.000（home-quiet-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.024 Δ | 0.004（home-quiet-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.789 dB | 4.660（home-quiet-c08-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.868 × | 6.936（home-quiet-c01-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -7.831 dB | -5.735（home-quiet-c05-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### home-quiet · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（home-quiet-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.497 dB | -1.010（home-quiet-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 2.949 dB | 7.185（home-quiet-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.063 比例 | 0.095（home-quiet-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.387 dB | -1.752（home-quiet-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 5.312 dB² | 9.556（home-quiet-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 25.000 个/分钟 | 65.000（home-quiet-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.049 Δ | 0.021（home-quiet-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.755 dB | 5.802（home-quiet-s02-ext） | 6.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.167 × | 1.971（home-quiet-s01-ext） | 2.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -9.094 dB | -7.411（home-quiet-s00-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fan`（pattern）：差 +2.684（有 3 段 4.291 · 没有 3 段 1.607）
- V-ONSET · `fridge-hum`（pattern）：差 +2.227（有 3 段 4.063 · 没有 3 段 1.835）
- V-ONSET · `bed-ace`（bed）：差 +1.974（有 3 段 3.936 · 没有 3 段 1.962）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.041（有 3 段 0.083 · 没有 3 段 0.042）
- V-SIM-GATE · `fan`（pattern）：差 +0.012（有 3 段 0.069 · 没有 3 段 0.057）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.004（有 3 段 0.064 · 没有 3 段 0.061）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +5.992（有 3 段 8.308 · 没有 3 段 2.317）
- V-PUMPV · `bed-ace`（bed）：差 +2.234（有 3 段 6.430 · 没有 3 段 4.195）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（home-quiet-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.284 dB | -0.492（home-quiet-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.596 dB | 7.602（home-quiet-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.080 比例 | 0.125（home-quiet-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.326 dB | -1.779（home-quiet-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 6.083 dB² | 12.732（home-quiet-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 12.500 个/分钟 | 20.000（home-quiet-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.054 Δ | 0.028（home-quiet-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.723 dB | 5.510（home-quiet-s02-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.183 × | 2.102（home-quiet-s01-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.888 dB | -7.781（home-quiet-s03-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fridge-hum`（pattern）：差 +2.600（有 3 段 4.896 · 没有 3 段 2.296）
- V-ONSET · `fan`（pattern）：差 +2.548（有 3 段 4.870 · 没有 3 段 2.322）
- V-ONSET · `bed-ace`（bed）：差 +1.308（有 3 段 4.250 · 没有 3 段 2.942）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.064（有 3 段 0.112 · 没有 3 段 0.048）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.011（有 3 段 0.085 · 没有 3 段 0.075）
- V-SIM-GATE · `fan`（pattern）：差 +0.002（有 3 段 0.081 · 没有 3 段 0.079）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +7.443（有 3 段 9.804 · 没有 3 段 2.361）
- V-PUMPV · `bed-ace`（bed）：差 +1.957（有 3 段 7.061 · 没有 3 段 5.104）
- V-PUMPV · `fan`（pattern）：差 +0.734（有 3 段 6.450 · 没有 3 段 5.716）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `fan`（pattern）：差 +0.461（有 3 段 1.414 · 没有 3 段 0.953）
- V-MUSIC · `fridge-hum`（pattern）：差 +0.322（有 3 段 1.345 · 没有 3 段 1.022）
- V-MUSIC · `bed-ace`（bed）：差 +0.298（有 3 段 1.332 · 没有 3 段 1.035）
- V-MUSIC · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-quiet-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.740 dB | -1.343（home-quiet-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 1 / 0 | 2.070 dB | 6.027（home-quiet-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.051 比例 | 0.085（home-quiet-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.322 dB | -1.289（home-quiet-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 4.344 dB² | 8.046（home-quiet-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 15.833 个/分钟 | 30.000（home-quiet-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.033 Δ | 0.014（home-quiet-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.665 dB | 5.163（home-quiet-s02-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.201 × | 2.324（home-quiet-s01-ext） | 2.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -6.546 dB | -4.057（home-quiet-s03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### home-quiet · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（home-quiet-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.599 dB | -0.946（home-quiet-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.054 dB | 7.816（home-quiet-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.075 比例 | 0.121（home-quiet-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.984 dB | -1.835（home-quiet-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 6.710 dB² | 14.589（home-quiet-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 30.000 个/分钟 | 45.000（home-quiet-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.058 Δ | 0.048（home-quiet-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.619 dB | 6.531（home-quiet-s02-mba） | 6.000 | 不过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.184 × | 2.198（home-quiet-s01-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.612 dB | -7.696（home-quiet-s03-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fan`（pattern）：差 +3.010（有 3 段 5.559 · 没有 3 段 2.548）
- V-ONSET · `fridge-hum`（pattern）：差 +1.507（有 3 段 4.807 · 没有 3 段 3.300）
- V-ONSET · `bed-ace`（bed）：差 +1.281（有 3 段 4.694 · 没有 3 段 3.413）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.038（有 3 段 0.095 · 没有 3 段 0.056）
- V-SIM-GATE · `fan`（pattern）：差 +0.025（有 3 段 0.088 · 没有 3 段 0.063）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.006（有 3 段 0.078 · 没有 3 段 0.072）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +7.395（有 3 段 10.408 · 没有 3 段 3.013）
- V-PUMPV · `bed-room`（bed）：差 +1.791（有 3 段 7.606 · 没有 3 段 5.814）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +2.157（有 3 段 2.698 · 没有 3 段 0.541）
- V-TIMBRE · `fridge-hum`（pattern）：差 +1.392（有 3 段 2.316 · 没有 3 段 0.923）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `fan`（pattern）：差 +0.489（有 3 段 1.428 · 没有 3 段 0.939）
- V-MUSIC · `fridge-hum`（pattern）：差 +0.379（有 3 段 1.373 · 没有 3 段 0.994）
- V-MUSIC · `bed-ace`（bed）：差 +0.336（有 3 段 1.352 · 没有 3 段 1.016）
- V-MUSIC · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（home-quiet-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.385 dB | -0.654（home-quiet-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 4.217 dB | 8.208（home-quiet-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.080 比例 | 0.133（home-quiet-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.871 dB | -1.820（home-quiet-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 7.720 dB² | 18.918（home-quiet-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 18.333 个/分钟 | 25.000（home-quiet-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.063 Δ | 0.052（home-quiet-s05-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.525 dB | 6.276（home-quiet-s02-mba） | 6.000 | 不过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.187 × | 2.139（home-quiet-s01-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.606 dB | -7.933（home-quiet-s03-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `fridge-hum`（pattern）：差 +3.048（有 3 段 5.741 · 没有 3 段 2.693）
- V-ONSET · `fan`（pattern）：差 +2.266（有 3 段 5.350 · 没有 3 段 3.084）
- V-ONSET · `bed-ace`（bed）：差 +0.758（有 3 段 4.596 · 没有 3 段 3.838）
- V-ONSET · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `fridge-hum`（pattern）：差 +0.059（有 3 段 0.109 · 没有 3 段 0.050）
- V-SIM-GATE · `fan`（pattern）：差 +0.017（有 3 段 0.088 · 没有 3 段 0.071）
- V-SIM-GATE · `bed-room`（bed）：差 +0.003（有 3 段 0.081 · 没有 3 段 0.078）
- V-SIM-GATE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `fridge-hum`（pattern）：差 +9.697（有 3 段 12.568 · 没有 3 段 2.871）
- V-PUMPV · `bed-room`（bed）：差 +2.447（有 3 段 8.943 · 没有 3 段 6.496）
- V-PUMPV · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`（bed）：差 +1.714（有 3 段 2.382 · 没有 3 段 0.667）
- V-TIMBRE · `fridge-hum`（pattern）：差 +1.697（有 3 段 2.373 · 没有 3 段 0.676）
- V-TIMBRE · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `fan`（pattern）：差 +0.491（有 3 段 1.432 · 没有 3 段 0.942）
- V-MUSIC · `fridge-hum`（pattern）：差 +0.336（有 3 段 1.355 · 没有 3 段 1.019）
- V-MUSIC · `bed-ace`（bed）：差 +0.296（有 3 段 1.335 · 没有 3 段 1.039）
- V-MUSIC · `door-knock`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-MUSIC · `keyboard`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### home-quiet · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-quiet-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-quiet-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.818 dB | -1.349（home-quiet-s05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.745 dB | 8.561（home-quiet-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.073 比例 | 0.128（home-quiet-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.874 dB | -1.400（home-quiet-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.666 dB² | 13.402（home-quiet-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 19.167 个/分钟 | 40.000（home-quiet-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.037 Δ | 0.026（home-quiet-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.778 dB | 6.420（home-quiet-s02-mba） | 6.000 | 不过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.243 × | 2.217（home-quiet-s01-mba） | 2.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -8.059 dB | -5.877（home-quiet-s03-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### home-family · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（home-family-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 0 / 5 | -1.302 dB | -2.174（home-family-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 1.704 dB | 4.199（home-family-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.028 比例 | 0.143（home-family-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.603 dB | -1.041（home-family-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 5.407 dB² | 28.543（home-family-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 60.500 个/分钟 | 120.000（home-family-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.034 Δ | 0.011（home-family-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 1.175 dB | 2.440（home-family-c07-ext） | 6.000 | 过 |
| V-NR |  | 2 / 0 / 8 | -8.433 dB | -7.084（home-family-c05-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -4.019 dB | -1.592（home-family-c02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 8 / 1 | 0.645 比例 | 0.810（home-family-c07-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 8 | 6.740 dB | 7.146（home-family-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`（event）：差 +0.090（有 3 段 1.767 · 没有 7 段 1.677）
- V-ONSET · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `doorbell`（event）：差 +0.045（有 3 段 0.059 · 没有 7 段 0.014）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `doorbell`（event）：差 +7.268（有 3 段 10.494 · 没有 7 段 3.226）
- V-PUMPV · `dishes`（event）：差 +0.558（有 3 段 5.797 · 没有 7 段 5.239）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（home-family-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 0 / 5 | -0.072 dB | -0.111（home-family-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 2.227 dB | 6.850（home-family-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.028 比例 | 0.116（home-family-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.749 dB | -1.772（home-family-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 7.295 dB² | 44.817（home-family-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 50.000 个/分钟 | 95.000（home-family-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.051 Δ | 0.019（home-family-c08-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 0.716 dB | 1.332（home-family-c07-ext） | 6.000 | 过 |
| V-NR |  | 2 / 0 / 8 | -8.928 dB | -8.075（home-family-c05-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -2.834 dB | -0.671（home-family-c02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.711 比例 | 0.885（home-family-c01-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 8 | 6.152 dB | 6.272（home-family-c03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `dishes`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `doorbell`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`（event）：差 +0.591（有 3 段 2.641 · 没有 7 段 2.050）
- V-ONSET · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `doorbell`（event）：差 +0.031（有 3 段 0.050 · 没有 7 段 0.019）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `doorbell`（event）：差 +11.961（有 3 段 15.667 · 没有 7 段 3.706）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `dishes`（event）：差 +0.092（有 3 段 0.773 · 没有 6 段 0.681）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-family-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 5 | -0.554 dB | -2.257（home-family-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

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

### home-family · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（home-family-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 4 / 0 / 6 | -0.676 dB | -1.159（home-family-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 1.558 dB | 5.094（home-family-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.021 比例 | 0.045（home-family-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.781 dB | -2.292（home-family-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.046 dB² | 4.545（home-family-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 45.000 个/分钟 | 80.000（home-family-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.045 Δ | 0.023（home-family-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 6 | 2.764 dB | 6.729（home-family-c07-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 0 / 7 | -7.617 dB | -6.462（home-family-c05-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -4.761 dB | -2.143（home-family-c02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 8 / 1 | 0.527 比例 | 0.802（home-family-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 4 / 0 / 6 | 4.127 dB | 6.578（home-family-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `doorbell`（event）：差 +0.005（有 3 段 0.024 · 没有 7 段 0.019）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（home-family-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 4 / 0 / 6 | -0.080 dB | -0.093（home-family-c05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 9 / 1 / 0 | 1.578 dB | 5.368（home-family-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 8 / 2 / 0 | 0.021 比例 | 0.047（home-family-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.840 dB | -2.513（home-family-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 2.192 dB² | 3.622（home-family-c00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 54.500 个/分钟 | 110.000（home-family-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.061 Δ | 0.043（home-family-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 6 | 2.498 dB | 5.881（home-family-c07-mba） | 6.000 | 过 |
| V-NR |  | 3 / 0 / 7 | -9.287 dB | -7.542（home-family-c05-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -2.905 dB | -0.719（home-family-c02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 9 / 1 | 0.696 比例 | 0.858（home-family-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 4 / 0 / 6 | 3.805 dB | 6.083（home-family-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `dishes`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `doorbell`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `baby-cry`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `dishes`（event）：差 +0.055（有 3 段 0.733 · 没有 6 段 0.678）
- V-SIM-FA · `baby-cry`：样本不够（有 0 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）

### home-family · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（home-family-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（home-family-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 0 / 6 | -0.523 dB | -1.594（home-family-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### home-family · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（home-family-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -1.670 dB | -2.741（home-family-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 1.867 dB | 6.367（home-family-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.052 比例 | 0.113（home-family-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.966 dB | -5.072（home-family-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 16.093 dB² | 26.084（home-family-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 50.000 个/分钟 | 75.000（home-family-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.019 Δ | -0.015（home-family-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 2 / 2 | 4.620 dB | 7.573（home-family-s02-ext） | 6.000 | 不过 |
| V-NR |  | 2 / 0 / 4 | -9.877 dB | -9.720（home-family-s03-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 5 / 1 | -4.547 dB | -1.441（home-family-s04-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 0.635 比例 | 0.957（home-family-s02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 2.999 dB | 5.700（home-family-s03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.438（有 3 段 2.086 · 没有 3 段 1.648）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.053（有 3 段 0.078 · 没有 3 段 0.026）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +14.631（有 3 段 23.408 · 没有 3 段 8.777）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（home-family-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -0.870 dB | -1.770（home-family-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 1.180 dB | 3.632（home-family-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.054 比例 | 0.119（home-family-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.837 dB | -5.398（home-family-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.676 dB² | 25.687（home-family-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 43.333 个/分钟 | 65.000（home-family-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.027 Δ | -0.015（home-family-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 2 | 3.460 dB | 7.256（home-family-s03-ext） | 6.000 | 不过 |
| V-NR |  | 2 / 0 / 4 | -10.039 dB | -10.038（home-family-s02-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 5 / 1 | -2.873 dB | -0.682（home-family-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 0.807 比例 | 0.957（home-family-s02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 3.288 dB | 5.488（home-family-s03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.243（有 3 段 1.801 · 没有 3 段 0.558）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.047（有 3 段 0.078 · 没有 3 段 0.031）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +6.269（有 3 段 15.810 · 没有 3 段 9.541）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（home-family-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -1.465 dB | -2.013（home-family-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

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

### home-family · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（home-family-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 3 / 1 / 2 | -2.155 dB | -4.310（home-family-s04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.547 dB | 7.368（home-family-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.031 比例 | 0.066（home-family-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.838 dB | -8.311（home-family-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 17.728 dB² | 63.820（home-family-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 52.500 个/分钟 | 100.000（home-family-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.022 Δ | -0.012（home-family-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 1 / 2 | 3.323 dB | 8.324（home-family-s02-mba） | 6.000 | 不过 |
| V-NR |  | 2 / 0 / 4 | -9.455 dB | -8.948（home-family-s03-mba） | -6.000 | 过 |
| V-INTF |  | 1 / 4 / 1 | -4.811 dB | -1.641（home-family-s04-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 4 / 1 | 0.607 比例 | 0.960（home-family-s04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 3.390 dB | 5.936（home-family-s03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.868（有 3 段 4.481 · 没有 3 段 2.613）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.032（有 3 段 0.047 · 没有 3 段 0.015）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`（event）：差 +13.555（有 3 段 24.506 · 没有 3 段 10.951）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `baby-cry`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-kitchen`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）

### home-family · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（home-family-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（home-family-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 3 / 1 / 2 | -1.527 dB | -4.843（home-family-s04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.646 dB | 9.627（home-family-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.054 比例 | 0.084（home-family-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.672 dB | -8.408（home-family-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 10.087 dB² | 18.633（home-family-s05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 45.833 个/分钟 | 105.000（home-family-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.024 Δ | -0.027（home-family-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 0 / 2 | 2.407 dB | 5.961（home-family-s02-mba） | 6.000 | 过 |
| V-NR |  | 2 / 0 / 4 | -9.640 dB | -9.587（home-family-s02-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 5 / 1 | -3.046 dB | -0.890（home-family-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 5 / 1 | 0.764 比例 | 0.967（home-family-s02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 2 / 0 / 4 | 3.814 dB | 5.639（home-family-s03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `baby-cry`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `bed-kitchen`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +3.924（有 3 段 5.608 · 没有 3 段 1.684）
- V-ONSET · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.008（有 3 段 0.058 · 没有 3 段 0.050）
- V-SIM-GATE · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +7.277（有 3 段 13.725 · 没有 3 段 6.448）
- V-PUMPV · `baby-cry`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-kitchen`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### open-office · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（open-office-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.900 dB | -1.734（open-office-c08-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 1 / 2 | 1.605 dB | 5.248（open-office-c05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 7 / 3 / 0 | 0.025 比例 | 0.071（open-office-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.744 dB | 0.190（open-office-c01-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 4.806 dB² | 31.520（open-office-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 21.500 个/分钟 | 50.000（open-office-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.029 Δ | 0.000（open-office-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 2.075 dB | 10.937（open-office-c08-ext） | 6.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -8.042 dB | -1.591（open-office-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -3.017 dB | -1.918（open-office-c03-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 6 | 0.721 比例 | 1.000（open-office-c07-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.292 dB | 8.032（open-office-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-LAT · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `mouse`（event）：差 +1.983（有 3 段 2.844 · 没有 5 段 0.861）
- V-ONSET · `notify`（event）：差 +1.871（有 3 段 2.774 · 没有 5 段 0.903）
- V-ONSET · `bed-ace`（bed）：差 +0.890（有 5 段 1.939 · 没有 3 段 1.049）
- V-ONSET · `hvac`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`（event）：差 +0.027（有 4 段 0.041 · 没有 6 段 0.014）
- V-SIM-GATE · `mouse`（event）：差 +0.016（有 4 段 0.034 · 没有 6 段 0.018）
- V-SIM-GATE · `notify`（event）：差 +0.010（有 3 段 0.031 · 没有 7 段 0.022）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +8.312（有 4 段 9.793 · 没有 6 段 1.481）
- V-PUMPV · `bed-office`（bed）：差 +6.513（有 4 段 8.714 · 没有 6 段 2.201）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +4.012（有 4 段 4.482 · 没有 6 段 0.470）
- V-TIMBRE · `notify`（event）：差 +3.050（有 3 段 4.210 · 没有 7 段 1.160）
- V-TIMBRE · `bed-ace`（bed）：差 +2.891（有 6 段 3.231 · 没有 4 段 0.340）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-office`（bed）：差 +3.118（有 4 段 -6.171 · 没有 6 段 -9.290）
- V-NR · `keyboard`（event）：差 +0.791（有 4 段 -7.568 · 没有 6 段 -8.359）
- V-NR · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-office`：样本不够（有 2 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（open-office-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.232 dB | -0.813（open-office-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 4 / 2 | 2.808 dB | 6.403（open-office-c05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.050 比例 | 0.175（open-office-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -2.542 dB | 0.156（open-office-c01-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 5.316 dB² | 26.742（open-office-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 8 / 0 | 15.000 个/分钟 | 30.000（open-office-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.043 Δ | 0.000（open-office-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.816 dB | 8.259（open-office-c08-ext） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -9.778 dB | -6.617（open-office-c08-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 5 / 5 | -3.712 dB | -2.153（open-office-c00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 6 | 0.613 比例 | 1.000（open-office-c07-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 9 / 1 / 0 | 2.691 dB | 13.564（open-office-c01-ext） | 10.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `notify`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-LAT · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `mouse`（event）：差 +4.120（有 3 段 5.382 · 没有 5 段 1.263）
- V-ONSET · `bed-ace`（bed）：差 +1.934（有 5 段 3.533 · 没有 3 段 1.599）
- V-ONSET · `notify`（event）：差 +1.281（有 3 段 3.608 · 没有 5 段 2.327）
- V-ONSET · `hvac`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `keyboard`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`（event）：差 +0.047（有 4 段 0.078 · 没有 6 段 0.031）
- V-SIM-GATE · `bed-office`（bed）：差 +0.023（有 4 段 0.064 · 没有 6 段 0.041）
- V-SIM-GATE · `mouse`（event）：差 +0.013（有 4 段 0.058 · 没有 6 段 0.045）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +6.654（有 4 段 9.308 · 没有 6 段 2.654）
- V-PUMPV · `bed-office`（bed）：差 +4.769（有 4 段 8.177 · 没有 6 段 3.408）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +3.135（有 4 段 3.697 · 没有 6 段 0.562）
- V-TIMBRE · `notify`（event）：差 +3.102（有 3 段 3.987 · 没有 7 段 0.885）
- V-TIMBRE · `bed-ace`（bed）：差 +1.963（有 6 段 2.601 · 没有 4 段 0.638）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-office`：样本不够（有 2 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-BGSTAB · `bed-office`（bed）：差 +1.345（有 4 段 3.498 · 没有 6 段 2.153）
- V-BGSTAB · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-BGSTAB · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### open-office · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（open-office-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.571 dB | -1.418（open-office-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### open-office · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（open-office-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.787 dB | -1.880（open-office-c08-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 3 / 1 | 2.384 dB | 6.321（open-office-c05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.034 比例 | 0.097（open-office-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.139 dB | -0.064（open-office-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.596 dB² | 5.409（open-office-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 23.500 个/分钟 | 60.000（open-office-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.041 Δ | 0.011（open-office-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.647 dB | 10.569（open-office-c08-mba） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -9.862 dB | -6.777（open-office-c08-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 6 / 4 | -4.608 dB | -1.048（open-office-c03-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 6 | 0.449 比例 | 0.700（open-office-c07-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.960 dB | 7.536（open-office-c01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-LAT · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `mouse`（event）：差 +2.544（有 3 段 4.080 · 没有 6 段 1.536）
- V-ONSET · `keyboard`（event）：差 +1.673（有 3 段 3.499 · 没有 6 段 1.827）
- V-ONSET · `notify`（event）：差 +0.965（有 3 段 3.027 · 没有 6 段 2.063）
- V-ONSET · `hvac`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-ONSET · `phone-ring`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `mouse`（event）：差 +0.022（有 4 段 0.047 · 没有 6 段 0.025）
- V-SIM-GATE · `keyboard`（event）：差 +0.020（有 4 段 0.046 · 没有 6 段 0.026）
- V-SIM-GATE · `notify`（event）：差 +0.009（有 3 段 0.040 · 没有 7 段 0.031）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `mouse`（event）：差 +1.051（有 4 段 3.227 · 没有 6 段 2.176）
- V-PUMPV · `keyboard`（event）：差 +0.827（有 4 段 3.092 · 没有 6 段 2.266）
- V-PUMPV · `notify`（event）：差 +0.158（有 3 段 2.707 · 没有 7 段 2.549）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `mouse`（event）：差 +3.306（有 4 段 3.631 · 没有 6 段 0.325）
- V-TIMBRE · `notify`（event）：差 +3.223（有 3 段 3.903 · 没有 7 段 0.681）
- V-TIMBRE · `bed-ace`（bed）：差 +2.177（有 6 段 2.518 · 没有 4 段 0.341）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-office`（bed）：差 +0.157（有 3 段 -4.529 · 没有 3 段 -4.687）
- V-INTF · `hvac`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-INTF · `keyboard`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（open-office-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.159 dB | -0.482（open-office-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 5 / 1 | 3.166 dB | 6.544（open-office-c07-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.041 比例 | 0.105（open-office-c07-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.073 dB | -0.136（open-office-c01-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 3.492 dB² | 7.919（open-office-c07-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 17.500 个/分钟 | 40.000（open-office-c09-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.057 Δ | 0.025（open-office-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.850 dB | 9.993（open-office-c08-mba） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.319 dB | -6.355（open-office-c08-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 6 / 4 | -4.202 dB | -0.983（open-office-c03-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 6 | 0.395 比例 | 0.757（open-office-c02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.698 dB | 6.487（open-office-c09-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `notify`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-LAT · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `mouse`（event）：差 +3.186（有 3 段 5.290 · 没有 6 段 2.104）
- V-ONSET · `bed-ace`（bed）：差 +0.987（有 5 段 3.604 · 没有 4 段 2.617）
- V-ONSET · `notify`（event）：差 +0.376（有 3 段 3.416 · 没有 6 段 3.040）
- V-ONSET · `hvac`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-ONSET · `phone-ring`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `mouse`（event）：差 +0.034（有 4 段 0.061 · 没有 6 段 0.027）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.017（有 6 段 0.048 · 没有 4 段 0.031）
- V-SIM-GATE · `keyboard`（event）：差 +0.011（有 4 段 0.048 · 没有 6 段 0.037）
- V-SIM-GATE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `mouse`（event）：差 +2.237（有 4 段 4.835 · 没有 6 段 2.597）
- V-PUMPV · `keyboard`（event）：差 +1.223（有 4 段 4.226 · 没有 6 段 3.003）
- V-PUMPV · `bed-ace`（bed）：差 +0.849（有 6 段 3.832 · 没有 4 段 2.983）
- V-PUMPV · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `notify`（event）：差 +3.281（有 3 段 4.146 · 没有 7 段 0.866）
- V-TIMBRE · `mouse`（event）：差 +2.914（有 4 段 3.598 · 没有 6 段 0.685）
- V-TIMBRE · `bed-ace`（bed）：差 +1.674（有 6 段 2.520 · 没有 4 段 0.846）
- V-TIMBRE · `hvac`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `phone-ring`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-office`（bed）：差 +0.395（有 3 段 -4.004 · 没有 3 段 -4.399）
- V-INTF · `hvac`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-INTF · `keyboard`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-office`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### open-office · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（open-office-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（open-office-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.721 dB | -1.715（open-office-c08-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

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

### open-office · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（open-office-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.852 dB | -1.404（open-office-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 2 / 2 | 1.673 dB | 3.494（open-office-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.032 比例 | 0.065（open-office-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.325 dB | -2.247（open-office-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.305 dB² | 23.800（open-office-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 46.667 个/分钟 | 75.000（open-office-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.027 Δ | 0.016（open-office-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.821 dB | 14.166（open-office-s02-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.249 dB | -7.351（open-office-s02-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -2.241 dB | -1.504（open-office-s03-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.886 比例 | 0.921（open-office-s02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.753 dB | 6.307（open-office-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.021（有 3 段 0.043 · 没有 3 段 0.022）
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

### open-office · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（open-office-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.527 dB | -1.308（open-office-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 1 / 2 | 1.088 dB | 3.074（open-office-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.035 比例 | 0.108（open-office-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -8.787 dB | -2.554（open-office-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.942 dB² | 17.810（open-office-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 35.833 个/分钟 | 65.000（open-office-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.035 Δ | 0.017（open-office-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 6.116 dB | 14.494（open-office-s02-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.070 dB | -7.954（open-office-s04-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -2.106 dB | -1.108（open-office-s03-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.881 比例 | 0.940（open-office-s00-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.240 dB | 6.677（open-office-s01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.025（有 3 段 0.048 · 没有 3 段 0.023）
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

### open-office · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（open-office-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.547 dB | -2.210（open-office-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### open-office · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（open-office-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.955 dB | -1.511（open-office-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 2 / 2 | 3.537 dB | 6.625（open-office-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.030 比例 | 0.062（open-office-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.177 dB | -3.773（open-office-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.065 dB² | 26.341（open-office-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 45.000 个/分钟 | 80.000（open-office-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.035 Δ | 0.024（open-office-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.401 dB | 10.503（open-office-s02-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.445 dB | -6.762（open-office-s02-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -2.339 dB | -1.577（open-office-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.875 比例 | 0.925（open-office-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 4.115 dB | 7.208（open-office-s03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.022（有 3 段 0.042 · 没有 3 段 0.019）
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

### open-office · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（open-office-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.588 dB | -0.882（open-office-s04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 1 / 2 | 3.148 dB | 5.039（open-office-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.046 比例 | 0.115（open-office-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.697 dB | -3.523（open-office-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.858 dB² | 26.064（open-office-s02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 41.667 个/分钟 | 55.000（open-office-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.042 Δ | 0.029（open-office-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.291 dB | 10.322（open-office-s02-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.788 dB | -7.267（open-office-s02-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -2.058 dB | -1.287（open-office-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.874 比例 | 0.925（open-office-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.447 dB | 6.315（open-office-s02-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `hvac`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `phone-ring`（event）：差 +0.040（有 3 段 0.066 · 没有 3 段 0.026）
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

### open-office · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（open-office-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（open-office-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.889 dB | -2.447（open-office-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### phone-booth · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（phone-booth-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.597 dB | -2.049（phone-booth-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 10 / 0 / 0 | 0.728 dB | 2.904（phone-booth-c01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.025 比例 | 0.062（phone-booth-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.220 dB | 2.160（phone-booth-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 8 / 2 / 0 | 1.028 dB² | 3.114（phone-booth-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 8 / 2 / 0 | 1.500 个/分钟 | 10.000（phone-booth-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | -0.001 Δ | -0.009（phone-booth-c05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.314 dB | 1.134（phone-booth-c03-ext） | 6.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 2.936 × | 8.575（phone-booth-c07-ext） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.947 dB | -3.771（phone-booth-c04-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `muffled-talker`（speech）：差 +0.004（有 6 段 0.026 · 没有 4 段 0.023）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（phone-booth-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -0.412 dB | -3.182（phone-booth-c01-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 2.193 dB | 7.039（phone-booth-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.111 比例 | 0.444（phone-booth-c01-ext） | 0.050 | 不过 |
| V-WEAK |  | 8 / 2 / 0 | 1.466 dB | 6.560（phone-booth-c01-ext） | 4.000 | 不过 |
| V-PUMPV |  | 5 / 5 / 0 | 5.447 dB² | 26.690（phone-booth-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.006 Δ | -0.005（phone-booth-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.342 dB | 1.922（phone-booth-c01-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.475 × | 8.278（phone-booth-c07-ext） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.153 dB | -6.168（phone-booth-c04-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-WEAK · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-WEAK · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 43.667 ms | 43.667（phone-booth-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.186 dB | -0.385（phone-booth-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 9 / 1 / 0 | 0.761 dB | 3.001（phone-booth-c01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.021 比例 | 0.033（phone-booth-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.363 dB | 1.887（phone-booth-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.064 dB² | 2.780（phone-booth-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 10 / 0 / 0 | 0.000 个/分钟 | 0.000（phone-booth-c00-ext） | 0.000 | 过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.002 Δ | -0.004（phone-booth-c05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.171 dB | 0.882（phone-booth-c00-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.719 × | 5.761（phone-booth-c07-ext） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.350 dB | -3.080（phone-booth-c03-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
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

### phone-booth · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（phone-booth-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.741 dB | -2.182（phone-booth-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 9 / 1 / 0 | 0.454 dB | 3.083（phone-booth-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.023 比例 | 0.051（phone-booth-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.214 dB | 2.130（phone-booth-c05-mba） | 4.000 | 过 |
| V-PUMPV |  | 7 / 3 / 0 | 0.903 dB² | 2.789（phone-booth-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 3.000 个/分钟 | 10.000（phone-booth-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.007 Δ | -0.003（phone-booth-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.332 dB | 1.265（phone-booth-c03-mba） | 6.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.964 × | 10.614（phone-booth-c07-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.786 dB | -4.298（phone-booth-c04-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `muffled-talker`（speech）：差 +0.002（有 6 段 0.024 · 没有 4 段 0.022）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（phone-booth-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.337 dB | -2.066（phone-booth-c02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 1.545 dB | 5.503（phone-booth-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.043 比例 | 0.067（phone-booth-c07-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.470 dB | 0.983（phone-booth-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 2.141 dB² | 6.053（phone-booth-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 8 / 2 / 0 | 1.000 个/分钟 | 5.000（phone-booth-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.016 Δ | 0.000（phone-booth-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.452 dB | 2.737（phone-booth-c02-mba） | 6.000 | 过 |
| V-MUSIC |  | 4 / 6 / 0 | 3.497 × | 12.843（phone-booth-c07-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -9.968 dB | -5.816（phone-booth-c04-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
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

### phone-booth · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（phone-booth-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 9 / 1 / 0 | 43.667 ms | 43.667（phone-booth-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.196 dB | -0.669（phone-booth-c01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 10 / 0 / 0 | 0.588 dB | 2.493（phone-booth-c01-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.021 比例 | 0.030（phone-booth-c01-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 1.252 dB | 1.946（phone-booth-c05-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.009 dB² | 2.705（phone-booth-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 2.500 个/分钟 | 10.000（phone-booth-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.007 Δ | -0.003（phone-booth-c08-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.219 dB | 0.938（phone-booth-c00-mba） | 6.000 | 过 |
| V-MUSIC |  | 4 / 6 / 0 | 2.931 × | 6.693（phone-booth-c06-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -9.483 dB | -5.296（phone-booth-c04-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### phone-booth · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（phone-booth-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.842 dB | -1.797（phone-booth-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 1.597 dB | 4.003（phone-booth-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.063 比例 | 0.221（phone-booth-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.538 dB | 0.355（phone-booth-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 3.807 dB² | 11.492（phone-booth-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 14.167 个/分钟 | 30.000（phone-booth-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.031 Δ | 0.019（phone-booth-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 0.551 dB | 1.067（phone-booth-s04-ext） | 6.000 | 过 |
| V-MUSIC |  | 2 / 4 / 0 | 2.701 × | 4.137（phone-booth-s02-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.044 dB | -9.274（phone-booth-s02-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（phone-booth-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.255 dB | -6.775（phone-booth-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.179 dB | 7.632（phone-booth-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.189 比例 | 0.812（phone-booth-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 5 / 1 / 0 | -0.540 dB | 5.506（phone-booth-s00-ext） | 4.000 | 不过 |
| V-PUMPV |  | 1 / 5 / 0 | 4.855 dB² | 14.581（phone-booth-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 3 / 3 / 0 | 2.500 个/分钟 | 5.000（phone-booth-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.033 Δ | -0.003（phone-booth-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 0.460 dB | 1.085（phone-booth-s04-ext） | 6.000 | 过 |
| V-MUSIC |  | 3 / 3 / 0 | 2.362 × | 5.546（phone-booth-s03-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -11.482 dB | -10.833（phone-booth-s02-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-WEAK · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-WEAK · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（phone-booth-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.482 dB | -1.414（phone-booth-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 0 / 0 | 0.925 dB | 2.335（phone-booth-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.022 比例 | 0.044（phone-booth-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.825 dB | 0.302（phone-booth-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 1.560 dB² | 2.625（phone-booth-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 3.333 个/分钟 | 5.000（phone-booth-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.028 Δ | 0.016（phone-booth-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.108 dB | 8.965（phone-booth-s04-ext） | 6.000 | 不过 |
| V-MUSIC |  | 4 / 2 / 0 | 1.594 × | 2.361（phone-booth-s00-ext） | 2.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -7.082 dB | -5.613（phone-booth-s02-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

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

### phone-booth · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（phone-booth-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.235 dB | -2.357（phone-booth-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 1.287 dB | 3.159（phone-booth-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.043 比例 | 0.087（phone-booth-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.800 dB | -0.219（phone-booth-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 3 / 3 / 0 | 1.782 dB² | 3.788（phone-booth-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 10.833 个/分钟 | 25.000（phone-booth-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.038 Δ | 0.019（phone-booth-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 0.662 dB | 1.143（phone-booth-s02-mba） | 6.000 | 过 |
| V-MUSIC |  | 3 / 3 / 0 | 2.705 × | 4.608（phone-booth-s02-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.284 dB | -7.557（phone-booth-s02-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（phone-booth-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.004 dB | -5.545（phone-booth-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 2.511 dB | 5.666（phone-booth-s04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.152 比例 | 0.658（phone-booth-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.256 dB | 3.216（phone-booth-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 3.927 dB² | 9.840（phone-booth-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 3 / 0 | 6.667 个/分钟 | 25.000（phone-booth-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.041 Δ | -0.008（phone-booth-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 0.482 dB | 1.350（phone-booth-s00-mba） | 6.000 | 过 |
| V-MUSIC |  | 3 / 3 / 0 | 1.682 × | 2.419（phone-booth-s03-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -11.404 dB | -10.893（phone-booth-s02-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### phone-booth · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（phone-booth-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（phone-booth-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.384 dB | -0.811（phone-booth-s04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 0 / 0 | 1.040 dB | 2.115（phone-booth-s04-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.028 比例 | 0.050（phone-booth-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.897 dB | 0.294（phone-booth-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 1.707 dB² | 2.894（phone-booth-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 10.000 个/分钟 | 20.000（phone-booth-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.016（phone-booth-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.036 dB | 9.182（phone-booth-s04-mba） | 6.000 | 不过 |
| V-MUSIC |  | 1 / 5 / 0 | 2.125 × | 2.677（phone-booth-s03-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -8.751 dB | -6.308（phone-booth-s04-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### meeting-room · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（meeting-room-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.524 dB | -2.228（meeting-room-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 0 / 7 | 0.870 dB | 2.635（meeting-room-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.045 比例 | 0.111（meeting-room-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.679 dB | 2.956（meeting-room-c04-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.875 dB² | 4.878（meeting-room-c06-ext） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 7.000 个/分钟 | 15.000（meeting-room-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.001 Δ | -0.018（meeting-room-c02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.549 dB | 0.903（meeting-room-c02-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -8.601 dB | -6.373（meeting-room-c08-ext） | -6.000 | 过 |
| V-BGSTAB |  | 9 / 1 / 0 | 2.806 dB | 11.329（meeting-room-c05-ext） | 10.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-room`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.023（有 4 段 0.059 · 没有 6 段 0.035）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.003（有 3 段 0.047 · 没有 7 段 0.044）
- V-SIM-GATE · `laugh`（event）：差 +0.000（有 6 段 0.045 · 没有 4 段 0.044）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +2.242（有 4 段 4.220 · 没有 6 段 1.978）
- V-PUMPV · `bed-ace`（bed）：差 +0.961（有 3 段 3.548 · 没有 7 段 2.587）
- V-PUMPV · `laugh`（event）：差 +0.634（有 6 段 3.129 · 没有 4 段 2.494）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-BGSTAB · `bed-room`（bed）：差 +0.675（有 7 段 3.008 · 没有 3 段 2.333）
- V-BGSTAB · `door`（event）：差 +0.161（有 4 段 2.903 · 没有 6 段 2.741）
- V-BGSTAB · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-BGSTAB · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（meeting-room-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.629 dB | -2.851（meeting-room-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 2 / 7 | 2.887 dB | 5.490（meeting-room-c05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.090 比例 | 0.183（meeting-room-c02-ext） | 0.050 | 不过 |
| V-WEAK |  | 9 / 1 / 0 | 1.481 dB | 4.159（meeting-room-c02-ext） | 4.000 | 不过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.031 dB² | 8.372（meeting-room-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 7 / 3 / 0 | 3.000 个/分钟 | 15.000（meeting-room-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.001 Δ | -0.027（meeting-room-c02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.817 dB | 1.814（meeting-room-c00-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -11.715 dB | -10.722（meeting-room-c09-ext） | -6.000 | 过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.885 dB | 7.563（meeting-room-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.070（有 3 段 0.139 · 没有 7 段 0.069）
- V-SIM-GATE · `door`（event）：差 +0.041（有 4 段 0.114 · 没有 6 段 0.073）
- V-SIM-GATE · `chair`（event）：差 +0.022（有 5 段 0.100 · 没有 5 段 0.079）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-WEAK · `chair`（event）：差 +1.561（有 5 段 2.262 · 没有 5 段 0.701）
- V-WEAK · `bed-room`（bed）：差 +0.175（有 7 段 1.534 · 没有 3 段 1.359）
- V-WEAK · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-WEAK · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +2.694（有 3 段 6.917 · 没有 7 段 4.223）
- V-PUMPV · `door`（event）：差 +2.328（有 4 段 6.428 · 没有 6 段 4.100）
- V-PUMPV · `laugh`（event）：差 +0.898（有 6 段 5.390 · 没有 4 段 4.493）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（meeting-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.026 dB | -2.039（meeting-room-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 1 / 7 | 2.029 dB | 3.510（meeting-room-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.028 比例 | 0.068（meeting-room-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.484 dB | 2.922（meeting-room-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.363 dB² | 4.214（meeting-room-c06-ext） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 6.500 个/分钟 | 15.000（meeting-room-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.005 Δ | -0.011（meeting-room-c02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.606 dB | 2.565（meeting-room-c00-ext） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -7.188 dB | -4.909（meeting-room-c05-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.011 dB | 7.167（meeting-room-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### meeting-room · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（meeting-room-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.516 dB | -2.762（meeting-room-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 1 / 7 | 2.646 dB | 3.140（meeting-room-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.049 比例 | 0.120（meeting-room-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.570 dB | 3.115（meeting-room-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 3.570 dB² | 7.482（meeting-room-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 9.000 个/分钟 | 25.000（meeting-room-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.009 Δ | -0.013（meeting-room-c02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.704 dB | 1.477（meeting-room-c04-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.476 dB | -8.282（meeting-room-c04-mba） | -6.000 | 过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.335 dB | 9.208（meeting-room-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-room`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `chair`（event）：差 +0.008（有 5 段 0.053 · 没有 5 段 0.045）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.004（有 3 段 0.052 · 没有 7 段 0.048）
- V-SIM-GATE · `door`（event）：差 +0.000（有 4 段 0.049 · 没有 6 段 0.049）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +2.908（有 4 段 5.315 · 没有 6 段 2.407）
- V-PUMPV · `bed-ace`（bed）：差 +1.956（有 3 段 4.939 · 没有 7 段 2.984）
- V-PUMPV · `laugh`（event）：差 +0.749（有 6 段 3.870 · 没有 4 段 3.121）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（meeting-room-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.493 dB | -2.688（meeting-room-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 3 / 7 | 3.955 dB | 4.450（meeting-room-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.083 比例 | 0.195（meeting-room-c02-mba） | 0.050 | 不过 |
| V-WEAK |  | 9 / 1 / 0 | 1.100 dB | 4.200（meeting-room-c02-mba） | 4.000 | 不过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.817 dB² | 7.528（meeting-room-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 6.000 个/分钟 | 25.000（meeting-room-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.009 Δ | -0.022（meeting-room-c02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.684 dB | 1.393（meeting-room-c02-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -11.810 dB | -10.848（meeting-room-c09-mba） | -6.000 | 过 |
| V-BGSTAB |  | 10 / 0 / 0 | 1.530 dB | 6.357（meeting-room-c08-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.083（有 3 段 0.141 · 没有 7 段 0.058）
- V-SIM-GATE · `door`（event）：差 +0.029（有 4 段 0.100 · 没有 6 段 0.071）
- V-SIM-GATE · `chair`（event）：差 +0.014（有 5 段 0.090 · 没有 5 段 0.076）
- V-SIM-GATE · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-WEAK · `chair`（event）：差 +1.898（有 5 段 2.049 · 没有 5 段 0.152）
- V-WEAK · `bed-ace`（bed）：差 +0.240（有 3 段 1.269 · 没有 7 段 1.028）
- V-WEAK · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-WEAK · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +2.294（有 3 段 6.423 · 没有 7 段 4.129）
- V-PUMPV · `door`（event）：差 +1.470（有 4 段 5.699 · 没有 6 段 4.229）
- V-PUMPV · `cough`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `hvac`：样本不够（有 9 段 · 没有 1 段，每组要 ≥ 3）

### meeting-room · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（meeting-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（meeting-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.050 dB | -1.940（meeting-room-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 2 / 7 | 2.599 dB | 4.097（meeting-room-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.037 比例 | 0.077（meeting-room-c06-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.437 dB | 2.471（meeting-room-c04-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 3.035 dB² | 6.662（meeting-room-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 4.500 个/分钟 | 15.000（meeting-room-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.010 Δ | -0.004（meeting-room-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.564 dB | 2.672（meeting-room-c00-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -9.935 dB | -7.731（meeting-room-c06-mba） | -6.000 | 过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.258 dB | 8.279（meeting-room-c05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### meeting-room · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（meeting-room-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.855 dB | -3.432（meeting-room-s01-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 1 / 5 | 4.302 dB | 4.302（meeting-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.040 比例 | 0.081（meeting-room-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 1.176 dB | 3.826（meeting-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 5 / 0 | 2.017 dB² | 2.881（meeting-room-s03-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 8.333 个/分钟 | 15.000（meeting-room-s03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | -0.003 Δ | -0.010（meeting-room-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.176 dB | 10.479（meeting-room-s05-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.163 dB | -8.385（meeting-room-s05-ext） | -6.000 | 过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.162 dB | 0.481（meeting-room-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +3.384（有 3 段 4.868 · 没有 3 段 1.484）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（meeting-room-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.105 dB | -3.111（meeting-room-s01-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 1 / 5 | 4.923 dB | 4.923（meeting-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.071 比例 | 0.108（meeting-room-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 5 / 1 / 0 | 2.007 dB | 4.066（meeting-room-s01-ext） | 4.000 | 不过 |
| V-PUMPV |  | 0 / 6 / 0 | 4.614 dB² | 6.791（meeting-room-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 4.167 个/分钟 | 10.000（meeting-room-s05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | -0.006 Δ | -0.016（meeting-room-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.209 dB | 7.493（meeting-room-s04-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -11.414 dB | -8.562（meeting-room-s03-ext） | -6.000 | 过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.057 dB | 5.887（meeting-room-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `chair`（event）：差 +0.054（有 3 段 0.098 · 没有 3 段 0.044）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-WEAK · `chair`（event）：差 +2.259（有 3 段 3.137 · 没有 3 段 0.878）
- V-WEAK · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-WEAK · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `chair`（event）：差 +2.387（有 3 段 5.807 · 没有 3 段 3.420）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +2.018（有 3 段 4.219 · 没有 3 段 2.200）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（meeting-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.956 dB | -3.448（meeting-room-s01-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 0 / 5 | 2.160 dB | 2.160（meeting-room-s00-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.022 比例 | 0.038（meeting-room-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 1.191 dB | 3.406（meeting-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 3 / 3 / 0 | 1.347 dB² | 2.587（meeting-room-s03-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 5.833 个/分钟 | 10.000（meeting-room-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.001 Δ | -0.006（meeting-room-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 4.838 dB | 10.358（meeting-room-s05-ext） | 6.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -6.405 dB | -3.887（meeting-room-s03-ext） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.652 dB | 3.406（meeting-room-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `chair`（event）：差 +3.685（有 3 段 6.680 · 没有 3 段 2.996）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（meeting-room-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.967 dB | -3.628（meeting-room-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 1 / 5 | 5.356 dB | 5.356（meeting-room-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.064 比例 | 0.098（meeting-room-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 0.461 dB | 3.405（meeting-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 3.331 dB² | 4.656（meeting-room-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 10.833 个/分钟 | 25.000（meeting-room-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.008 Δ | -0.000（meeting-room-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.411 dB | 2.146（meeting-room-s03-mba） | 6.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -11.219 dB | -8.609（meeting-room-s03-mba） | -6.000 | 过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.034 dB | 5.950（meeting-room-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `chair`（event）：差 +0.014（有 3 段 0.071 · 没有 3 段 0.057）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `chair`（event）：差 +1.990（有 3 段 4.326 · 没有 3 段 2.336）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（meeting-room-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.053 dB | -3.207（meeting-room-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 1 / 5 | 5.930 dB | 5.930（meeting-room-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.100 比例 | 0.216（meeting-room-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 1.081 dB | 3.672（meeting-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.814 dB² | 10.532（meeting-room-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 10.000 个/分钟 | 20.000（meeting-room-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.008 Δ | -0.001（meeting-room-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.828 dB | 2.968（meeting-room-s04-mba） | 6.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -11.909 dB | -11.750（meeting-room-s01-mba） | -6.000 | 过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.032 dB | 5.944（meeting-room-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `chair`（event）：差 +0.074（有 3 段 0.136 · 没有 3 段 0.063）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `chair`（event）：差 +2.694（有 3 段 7.161 · 没有 3 段 4.467）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### meeting-room · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（meeting-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（meeting-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.184 dB | -3.770（meeting-room-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 0 / 5 | 0.810 dB | 0.810（meeting-room-s00-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.029 比例 | 0.064（meeting-room-s03-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | 0.582 dB | 3.472（meeting-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 2 / 4 / 0 | 1.941 dB² | 3.180（meeting-room-s03-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 14.167 个/分钟 | 25.000（meeting-room-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.008 Δ | 0.000（meeting-room-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 3.707 dB | 7.961（meeting-room-s04-mba） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -6.537 dB | -4.308（meeting-room-s03-mba） | -6.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.989 dB | 5.629（meeting-room-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
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

### cowork · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（cowork-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.472 dB | -1.374（cowork-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 1 / 3 | 1.776 dB | 6.486（cowork-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.020 比例 | 0.041（cowork-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.185 dB | -1.135（cowork-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.552 dB² | 8.484（cowork-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 30.000 个/分钟 | 50.000（cowork-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.048 Δ | 0.018（cowork-c05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.782 dB | 2.934（cowork-c00-ext） | 6.000 | 过 |
| V-NR |  | 8 / 2 / 0 | -8.203 dB | -4.400（cowork-c00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -5.544 dB | -4.454（cowork-c06-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.523 比例 | 0.646（cowork-c02-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.251 dB | 9.392（cowork-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.452（有 4 段 2.399 · 没有 3 段 0.947）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.006（有 5 段 0.023 · 没有 5 段 0.017）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.002（有 7 段 0.020 · 没有 3 段 0.019）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.829（有 5 段 3.466 · 没有 5 段 1.637）
- V-PUMPV · `bed-ace`（bed）：差 +1.175（有 7 段 2.904 · 没有 3 段 1.729）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +0.815（有 7 段 -7.958 · 没有 3 段 -8.773）
- V-NR · `footsteps`（event）：差 +0.566（有 5 段 -7.920 · 没有 5 段 -8.486）
- V-NR · `talker`（speech）：差 +0.554（有 5 段 -7.926 · 没有 5 段 -8.480）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（cowork-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.276 dB | -0.572（cowork-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 2 / 3 | 2.319 dB | 6.365（cowork-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.030 比例 | 0.093（cowork-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -3.942 dB | -1.167（cowork-c05-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 3.263 dB² | 11.423（cowork-c01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 22.500 个/分钟 | 40.000（cowork-c07-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.060 Δ | 0.022（cowork-c05-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.724 dB | 2.251（cowork-c03-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -8.954 dB | -6.192（cowork-c03-ext） | -6.000 | 过 |
| V-INTF |  | 1 / 3 / 6 | -6.326 dB | -2.178（cowork-c06-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.371 比例 | 0.614（cowork-c09-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.938 dB | 7.333（cowork-c05-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.375（有 4 段 2.479 · 没有 3 段 2.105）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.025（有 5 段 0.043 · 没有 5 段 0.017）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.019（有 7 段 0.036 · 没有 3 段 0.016）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +3.058（有 5 段 4.792 · 没有 5 段 1.734）
- V-PUMPV · `bed-ace`（bed）：差 +1.970（有 7 段 3.854 · 没有 3 段 1.885）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cowork-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -1.315 dB | -4.766（cowork-c00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`（bed）：差 +0.522（有 7 段 -1.471 · 没有 3 段 -0.949）
- V-SPLVL · `talker`（speech）：差 +0.232（有 5 段 -1.430 · 没有 5 段 -1.199）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
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

### cowork · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（cowork-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.545 dB | -1.519（cowork-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 1 / 3 | 1.680 dB | 3.663（cowork-c06-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.024 比例 | 0.082（cowork-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.815 dB | -2.844（cowork-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 2.717 dB² | 5.687（cowork-c08-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 37.500 个/分钟 | 65.000（cowork-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.061 Δ | 0.043（cowork-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.827 dB | 2.871（cowork-c05-mba） | 6.000 | 过 |
| V-NR |  | 6 / 4 / 0 | -7.489 dB | -5.023（cowork-c08-mba） | -6.000 | 不过 |
| V-INTF |  | 1 / 3 / 6 | -7.276 dB | -5.518（cowork-c02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.370 比例 | 0.830（cowork-c02-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.184 dB | 7.491（cowork-c08-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.199（有 4 段 1.765 · 没有 3 段 1.566）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.006（有 7 段 0.026 · 没有 3 段 0.020）
- V-SIM-GATE · `footsteps`（event）：差 +0.005（有 5 段 0.027 · 没有 5 段 0.021）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.667（有 5 段 3.550 · 没有 5 段 1.883）
- V-PUMPV · `bed-ace`（bed）：差 +0.478（有 7 段 2.860 · 没有 3 段 2.382）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +2.062（有 5 段 -6.458 · 没有 5 段 -8.520）
- V-NR · `bed-ace`（bed）：差 +1.519（有 7 段 -7.034 · 没有 3 段 -8.552）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（cowork-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.262 dB | -0.497（cowork-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 2 / 3 | 2.211 dB | 4.175（cowork-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.025 比例 | 0.067（cowork-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.601 dB | -2.335（cowork-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 3.204 dB² | 7.354（cowork-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 25.500 个/分钟 | 65.000（cowork-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.073 Δ | 0.051（cowork-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.730 dB | 2.244（cowork-c03-mba） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -8.436 dB | -5.289（cowork-c00-mba） | -6.000 | 不过 |
| V-INTF |  | 2 / 2 / 6 | -7.824 dB | -3.167（cowork-c06-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 2 / 1 / 7 | 0.153 比例 | 0.460（cowork-c06-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.048 dB | 8.810（cowork-c07-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.040（有 4 段 2.229 · 没有 3 段 2.188）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 6 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.012（有 7 段 0.028 · 没有 3 段 0.017）
- V-SIM-GATE · `footsteps`（event）：差 +0.003（有 5 段 0.026 · 没有 5 段 0.023）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.976（有 5 段 3.692 · 没有 5 段 2.715）
- V-PUMPV · `bed-ace`（bed）：差 +0.684（有 7 段 3.409 · 没有 3 段 2.724）
- V-PUMPV · `footsteps`（event）：差 +0.633（有 5 段 3.520 · 没有 5 段 2.887）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +0.421（有 5 段 -8.225 · 没有 5 段 -8.646）
- V-NR · `talker`（speech）：差 +0.219（有 5 段 -8.326 · 没有 5 段 -8.545）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 0 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cowork · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cowork-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cowork-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -1.470 dB | -3.943（cowork-c00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-ace`（bed）：差 +0.472（有 7 段 -1.612 · 没有 3 段 -1.140）
- V-SPLVL · `talker`（speech）：差 +0.247（有 5 段 -1.594 · 没有 5 段 -1.346）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
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

### cowork · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（cowork-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.316 dB | -1.954（cowork-s05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.068 dB | 6.208（cowork-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.045 比例 | 0.069（cowork-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -8.730 dB | -2.097（cowork-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 12.065 dB² | 17.754（cowork-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 34.167 个/分钟 | 60.000（cowork-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.037 Δ | 0.019（cowork-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.825 dB | 3.669（cowork-s03-ext） | 6.000 | 过 |
| V-NR |  | 5 / 1 / 0 | -7.879 dB | -5.638（cowork-s05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -4.111 dB | -2.980（cowork-s01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.940 比例 | 1.000（cowork-s01-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.877 dB | 6.209（cowork-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +2.011（有 3 段 5.073 · 没有 3 段 3.062）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`（speech）：差 +0.000（有 3 段 0.045 · 没有 3 段 0.045）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +2.875（有 3 段 -6.442 · 没有 3 段 -9.316）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（cowork-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.723 dB | -1.292（cowork-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 4.311 dB | 6.835（cowork-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.045 比例 | 0.062（cowork-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -8.595 dB | -2.659（cowork-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.580 dB² | 17.926（cowork-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 28.333 个/分钟 | 50.000（cowork-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.054 Δ | 0.030（cowork-s03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.400 dB | 3.339（cowork-s04-ext） | 6.000 | 过 |
| V-NR |  | 3 / 3 / 0 | -7.462 dB | -5.490（cowork-s03-ext） | -6.000 | 不过 |
| V-INTF |  | 1 / 3 / 2 | -6.247 dB | -4.157（cowork-s05-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 2 | 0.746 比例 | 1.000（cowork-s04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.990 dB | 6.631（cowork-s00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +1.880（有 3 段 5.251 · 没有 3 段 3.371）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`（speech）：差 +0.007（有 3 段 0.049 · 没有 3 段 0.042）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +2.872（有 3 段 -6.026 · 没有 3 段 -8.898）
- V-NR · `bed-ace`（bed）：差 +1.271（有 3 段 -6.826 · 没有 3 段 -8.098）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cowork-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.356 dB | -5.620（cowork-s03-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `babble`（speech）：差 +1.472（有 3 段 -4.092 · 没有 3 段 -2.620）
- V-SPLVL · `bed-ace`（bed）：差 +1.173（有 3 段 -3.942 · 没有 3 段 -2.770）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
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

### cowork · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（cowork-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.640 dB | -2.538（cowork-s05-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 4.152 dB | 7.311（cowork-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.041 比例 | 0.066（cowork-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.537 dB | -3.439（cowork-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.511 dB² | 15.389（cowork-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 29.167 个/分钟 | 50.000（cowork-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.043 Δ | 0.011（cowork-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.851 dB | 4.613（cowork-s01-mba） | 6.000 | 过 |
| V-NR |  | 5 / 1 / 0 | -6.922 dB | -5.741（cowork-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -4.091 dB | -3.083（cowork-s01-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 4 / 2 | 0.844 比例 | 1.000（cowork-s01-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.101 dB | 5.986（cowork-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +1.770（有 3 段 5.037 · 没有 3 段 3.267）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.004（有 3 段 0.043 · 没有 3 段 0.039）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +3.194（有 3 段 10.107 · 没有 3 段 6.914）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +0.707（有 3 段 -6.568 · 没有 3 段 -7.276）
- V-NR · `bed-ace`（bed）：差 +0.612（有 3 段 -6.616 · 没有 3 段 -7.228）
- V-NR · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 3 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-ace`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### cowork · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cowork-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（cowork-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.767 dB | -1.121（cowork-s04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.483 dB | 9.155（cowork-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.060 比例 | 0.097（cowork-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.338 dB | -3.645（cowork-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.411 dB² | 20.160（cowork-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 34.167 个/分钟 | 45.000（cowork-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.058 Δ | 0.028（cowork-s03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 0.820 dB | 1.956（cowork-s01-mba） | 6.000 | 过 |
| V-NR |  | 4 / 2 / 0 | -7.454 dB | -5.346（cowork-s03-mba） | -6.000 | 不过 |
| V-INTF |  | 1 / 3 / 2 | -6.878 dB | -5.184（cowork-s01-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 3 / 2 | 0.507 比例 | 1.000（cowork-s04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.763 dB | 6.279（cowork-s00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-LAT · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `babble`（speech）：差 +2.296（有 3 段 5.631 · 没有 3 段 3.335）
- V-ONSET · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`（speech）：差 +0.004（有 3 段 0.062 · 没有 3 段 0.058）
- V-SIM-GATE · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +3.689（有 3 段 13.255 · 没有 3 段 9.567）
- V-PUMPV · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +2.445（有 3 段 -6.232 · 没有 3 段 -8.677）
- V-NR · `babble`（speech）：差 +1.662（有 3 段 -6.623 · 没有 3 段 -8.285）
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
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `babble`（speech）：差 +1.955（有 3 段 -4.390 · 没有 3 段 -2.435）
- V-SPLVL · `bed-ace`（bed）：差 +0.947（有 3 段 -3.886 · 没有 3 段 -2.939）
- V-SPLVL · `bed-cafeteria`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SPLVL · `bed-office`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
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

### cafe · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（cafe-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.623 dB | -0.977（cafe-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 3 / 1 | 2.995 dB | 7.774（cafe-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.041 比例 | 0.080（cafe-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.078 dB | -3.120（cafe-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.173 dB² | 9.381（cafe-c06-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 43.500 个/分钟 | 75.000（cafe-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.070 Δ | 0.043（cafe-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.909 dB | 6.661（cafe-c05-ext） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.132 dB | -7.809（cafe-c01-ext） | -6.000 | 过 |
| V-INTF |  | 1 / 3 / 6 | -6.130 dB | -2.780（cafe-c08-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.332 比例 | 0.865（cafe-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.421 dB | 6.396（cafe-c06-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`（pattern）：差 +1.932（有 4 段 4.068 · 没有 5 段 2.136）
- V-ONSET · `talker`（speech）：差 +1.798（有 6 段 3.594 · 没有 3 段 1.796）
- V-ONSET · `music`（pattern）：差 +0.267（有 6 段 3.084 · 没有 3 段 2.817）
- V-ONSET · `bed-restaurant`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.005（有 6 段 0.043 · 没有 4 段 0.038）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.003（有 4 段 0.043 · 没有 6 段 0.040）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.311（有 6 段 6.297 · 没有 4 段 5.986）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +0.123（有 6 段 1.958 · 没有 4 段 1.836）
- V-TIMBRE · `coffee-machine`（pattern）：差 +0.115（有 4 段 1.978 · 没有 6 段 1.863）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（cafe-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.281 dB | -0.806（cafe-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 4 / 1 | 2.945 dB | 7.230（cafe-c02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.054 比例 | 0.159（cafe-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.837 dB | -2.696（cafe-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.901 dB² | 11.304（cafe-c06-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 37.000 个/分钟 | 50.000（cafe-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.079 Δ | 0.056（cafe-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.656 dB | 6.170（cafe-c05-ext） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.808 dB | -8.920（cafe-c01-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -5.142 dB | -3.357（cafe-c08-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.573 比例 | 0.892（cafe-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.739 dB | 8.668（cafe-c01-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`（pattern）：差 +0.720（有 4 段 3.345 · 没有 5 段 2.625）
- V-ONSET · `talker`（speech）：差 +0.289（有 6 段 3.042 · 没有 3 段 2.753）
- V-ONSET · `bed-restaurant`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.016（有 6 段 0.061 · 没有 4 段 0.044）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.084（有 6 段 6.934 · 没有 4 段 6.850）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `coffee-machine`（pattern）：差 +0.019（有 4 段 1.668 · 没有 6 段 1.649）
- V-TIMBRE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（cafe-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.405 dB | -2.841（cafe-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### cafe · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（cafe-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.766 dB | -1.547（cafe-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 2.826 dB | 7.020（cafe-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.045 比例 | 0.092（cafe-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.878 dB | -3.645（cafe-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.005 dB² | 14.243（cafe-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 40.500 个/分钟 | 65.000（cafe-c09-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.071 Δ | 0.044（cafe-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.283 dB | 4.641（cafe-c05-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -9.538 dB | -7.765（cafe-c04-mba） | -6.000 | 过 |
| V-INTF |  | 1 / 3 / 6 | -7.044 dB | -2.556（cafe-c08-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.142 比例 | 0.297（cafe-c04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.862 dB | 7.212（cafe-c09-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`（pattern）：差 +0.973（有 4 段 3.410 · 没有 6 段 2.437）
- V-ONSET · `talker`（speech）：差 +0.635（有 6 段 3.080 · 没有 4 段 2.446）
- V-ONSET · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.009（有 6 段 0.049 · 没有 4 段 0.040）
- V-SIM-GATE · `coffee-machine`（pattern）：差 +0.002（有 4 段 0.047 · 没有 6 段 0.045）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`（pattern）：差 +3.917（有 4 段 8.355 · 没有 6 段 4.438）
- V-PUMPV · `talker`（speech）：差 +3.868（有 6 段 7.552 · 没有 4 段 3.684）
- V-PUMPV · `music`（pattern）：差 +1.430（有 6 段 6.577 · 没有 4 段 5.147）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### cafe · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（cafe-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（cafe-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.282 dB | -0.748（cafe-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 2.525 dB | 5.937（cafe-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.051 比例 | 0.162（cafe-c05-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.686 dB | -3.397（cafe-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.153 dB² | 14.153（cafe-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 36.000 个/分钟 | 55.000（cafe-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.084 Δ | 0.065（cafe-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.123 dB | 4.249（cafe-c05-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.438 dB | -8.378（cafe-c01-mba） | -6.000 | 过 |
| V-INTF |  | 2 / 2 / 6 | -7.104 dB | -2.429（cafe-c08-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 2 / 1 / 7 | 0.234 比例 | 0.703（cafe-c04-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.608 dB | 8.574（cafe-c01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.407（有 6 段 2.688 · 没有 4 段 2.281）
- V-ONSET · `dishes`（event）：差 +0.245（有 4 段 2.672 · 没有 6 段 2.427）
- V-ONSET · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.024（有 6 段 0.060 · 没有 4 段 0.037）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +2.978（有 6 段 7.344 · 没有 4 段 4.366）
- V-PUMPV · `coffee-machine`（pattern）：差 +0.748（有 4 段 6.602 · 没有 6 段 5.854）
- V-PUMPV · `bed-restaurant`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
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
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

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

### cafe · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（cafe-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.543 dB | -2.075（cafe-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.530 dB | 10.359（cafe-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.079 比例 | 0.147（cafe-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.610 dB | -10.010（cafe-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.004 dB² | 27.902（cafe-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 85.000 个/分钟 | 125.000（cafe-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.031 Δ | -0.001（cafe-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.033 dB | 6.442（cafe-s00-ext） | 6.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -8.774 dB | -5.294（cafe-s02-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -3.172 dB | -1.837（cafe-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.912 比例 | 1.000（cafe-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.560 dB | 4.237（cafe-s03-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +4.073（有 3 段 6.566 · 没有 3 段 2.493）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.059（有 3 段 0.109 · 没有 3 段 0.050）
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

### cafe · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（cafe-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.268 dB | -3.136（cafe-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.853 dB | 11.587（cafe-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.112 比例 | 0.253（cafe-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.172 dB | -9.989（cafe-s05-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.452 dB² | 35.876（cafe-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 74.167 个/分钟 | 120.000（cafe-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.036 Δ | -0.009（cafe-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.774 dB | 5.691（cafe-s04-ext） | 6.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -8.945 dB | -6.285（cafe-s02-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 5 / 1 | -2.991 dB | -1.701（cafe-s00-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.896 比例 | 0.952（cafe-s00-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.998 dB | 6.014（cafe-s04-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +5.687（有 3 段 7.696 · 没有 3 段 2.010）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.003（有 3 段 0.114 · 没有 3 段 0.111）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-restaurant`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `coffee-machine`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-FA · `bed-restaurant`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `coffee-machine`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### cafe · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（cafe-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.335 dB | -3.720（cafe-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
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

### cafe · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（cafe-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.428 dB | -2.116（cafe-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 3.941 dB | 7.605（cafe-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.072 比例 | 0.155（cafe-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.127 dB | -9.477（cafe-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.330 dB² | 27.789（cafe-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 44.167 个/分钟 | 75.000（cafe-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.034 Δ | 0.004（cafe-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.051 dB | 7.504（cafe-s00-mba） | 6.000 | 不过 |
| V-NR |  | 5 / 1 / 0 | -8.510 dB | -5.315（cafe-s02-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -2.936 dB | -1.661（cafe-s03-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.942 比例 | 1.000（cafe-s03-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.248 dB | 4.687（cafe-s04-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +2.531（有 3 段 5.207 · 没有 3 段 2.675）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.086（有 3 段 0.115 · 没有 3 段 0.029）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `music`（pattern）：差 +3.271（有 3 段 16.966 · 没有 3 段 13.694）
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

### cafe · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（cafe-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（cafe-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.435 dB | -4.593（cafe-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 4.317 dB | 9.789（cafe-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.117 比例 | 0.272（cafe-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.666 dB | -9.325（cafe-s05-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 17.178 dB² | 34.072（cafe-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 48.333 个/分钟 | 110.000（cafe-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.043 Δ | -0.002（cafe-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.725 dB | 5.976（cafe-s00-mba） | 6.000 | 过 |
| V-NR |  | 5 / 1 / 0 | -8.671 dB | -5.881（cafe-s02-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -3.289 dB | -2.109（cafe-s00-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 3 | 0.920 比例 | 1.000（cafe-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.179 dB | 6.039（cafe-s01-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `music`（pattern）：差 +4.901（有 3 段 6.767 · 没有 3 段 1.866）
- V-ONSET · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `music`（pattern）：差 +0.008（有 3 段 0.121 · 没有 3 段 0.113）
- V-SIM-GATE · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `music`（pattern）：差 +0.654（有 3 段 17.505 · 没有 3 段 16.851）
- V-PUMPV · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
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
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `coffee-machine`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
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

### canteen · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（canteen-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.456 dB | -2.982（canteen-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 5 / 2 | 4.174 dB | 6.802（canteen-c01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.039 比例 | 0.103（canteen-c00-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -10.413 dB | -5.952（canteen-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 8.855 dB² | 36.861（canteen-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 36.500 个/分钟 | 75.000（canteen-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.048 Δ | 0.016（canteen-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 4.062 dB | 16.454（canteen-c09-ext） | 6.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.641 dB | -5.256（canteen-c05-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -4.302 dB | -2.511（canteen-c01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.589 比例 | 0.817（canteen-c04-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.767 dB | 6.833（canteen-c08-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`（speech）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `bed-restaurant`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +2.561（有 5 段 5.135 · 没有 3 段 2.574）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.016（有 3 段 0.051 · 没有 7 段 0.034）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +2.920（有 7 段 9.732 · 没有 3 段 6.811）
- V-PUMPV · `babble`（speech）：差 +2.793（有 7 段 9.693 · 没有 3 段 6.900）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +3.254（有 6 段 5.364 · 没有 4 段 2.110）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +2.690（有 3 段 5.945 · 没有 7 段 3.256）
- V-TIMBRE · `babble`（speech）：差 +0.420（有 7 段 4.189 · 没有 3 段 3.768）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +2.171（有 7 段 -7.990 · 没有 3 段 -10.161）
- V-NR · `bed-cafeteria`（bed）：差 +1.177（有 3 段 -7.818 · 没有 7 段 -8.994）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（canteen-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.073 dB | -2.451（canteen-c09-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 8 / 2 | 5.173 dB | 8.482（canteen-c03-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.088 比例 | 0.234（canteen-c04-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.821 dB | -5.497（canteen-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 14.266 dB² | 68.795（canteen-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 28.500 个/分钟 | 90.000（canteen-c05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.055 Δ | 0.015（canteen-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 3.957 dB | 16.425（canteen-c09-ext） | 6.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -9.334 dB | -5.696（canteen-c05-ext） | -6.000 | 不过 |
| V-INTF |  | 1 / 3 / 6 | -6.536 dB | -1.942（canteen-c01-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 1 / 2 / 7 | 0.348 比例 | 0.554（canteen-c01-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.712 dB | 9.008（canteen-c00-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cafeteria`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +1.522（有 5 段 5.744 · 没有 3 段 4.222）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.037（有 7 段 0.099 · 没有 3 段 0.062）
- V-SIM-GATE · `talker`（speech）：差 +0.024（有 6 段 0.098 · 没有 4 段 0.074）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +8.277（有 7 段 16.749 · 没有 3 段 8.472）
- V-PUMPV · `babble`（speech）：差 +4.155（有 7 段 15.513 · 没有 3 段 11.358）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +3.332（有 6 段 5.289 · 没有 4 段 1.958）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +2.793（有 3 段 5.912 · 没有 7 段 3.119）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +2.157（有 7 段 -8.687 · 没有 3 段 -10.845）
- V-NR · `bed-cafeteria`（bed）：差 +1.724（有 3 段 -8.127 · 没有 7 段 -9.852）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（canteen-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 8 / 2 / 0 | -2.554 dB | -4.168（canteen-c09-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `babble`（speech）：差 +0.428（有 7 段 -2.683 · 没有 3 段 -2.255）
- V-SPLVL · `bed-cafeteria`（bed）：差 +0.257（有 3 段 -2.734 · 没有 7 段 -2.477）
- V-SPLVL · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
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

### canteen · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（canteen-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.446 dB | -2.418（canteen-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 6 / 1 | 3.605 dB | 7.487（canteen-c03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.051 比例 | 0.130（canteen-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -10.502 dB | -6.576（canteen-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 7.420 dB² | 22.241（canteen-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 36.000 个/分钟 | 60.000（canteen-c05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.044 Δ | 0.014（canteen-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.748 dB | 15.704（canteen-c09-mba） | 6.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.713 dB | -5.244（canteen-c05-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -4.271 dB | -3.467（canteen-c07-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.667 比例 | 0.824（canteen-c09-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 2.778 dB | 6.450（canteen-c03-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`（speech）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `bed-restaurant`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +2.135（有 6 段 4.317 · 没有 3 段 2.181）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cafeteria`（bed）：差 +0.010（有 3 段 0.058 · 没有 7 段 0.048）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +1.592（有 7 段 7.898 · 没有 3 段 6.306）
- V-PUMPV · `bed-restaurant`（bed）：差 +1.279（有 7 段 7.804 · 没有 3 段 6.525）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +3.198（有 6 段 5.027 · 没有 4 段 1.829）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +2.817（有 3 段 5.720 · 没有 7 段 2.903）
- V-TIMBRE · `babble`（speech）：差 +0.304（有 7 段 3.839 · 没有 3 段 3.535）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `babble`（speech）：差 +1.848（有 7 段 -8.159 · 没有 3 段 -10.007）
- V-NR · `bed-cafeteria`（bed）：差 +0.947（有 3 段 -8.051 · 没有 7 段 -8.998）
- V-NR · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）

### canteen · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（canteen-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（canteen-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.960 dB | -1.848（canteen-c09-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 7 / 1 | 4.569 dB | 8.107（canteen-c03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.088 比例 | 0.194（canteen-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.970 dB | -5.971（canteen-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 10.286 dB² | 32.338（canteen-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 27.500 个/分钟 | 60.000（canteen-c07-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.051 Δ | 0.022（canteen-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.439 dB | 15.139（canteen-c09-mba） | 6.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.956 dB | -5.838（canteen-c05-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -6.722 dB | -5.209（canteen-c07-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 3 / 7 | 0.254 比例 | 0.456（canteen-c09-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.897 dB | 9.750（canteen-c00-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cafeteria`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +1.498（有 6 段 5.069 · 没有 3 段 3.571）
- V-ONSET · `dishes`：样本不够（有 7 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `laugh`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.026（有 7 段 0.096 · 没有 3 段 0.070）
- V-SIM-GATE · `talker`（speech）：差 +0.019（有 6 段 0.096 · 没有 4 段 0.077）
- V-SIM-GATE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `babble`（speech）：差 +1.626（有 7 段 10.774 · 没有 3 段 9.147）
- V-PUMPV · `bed-restaurant`（bed）：差 +1.479（有 7 段 10.729 · 没有 3 段 9.250）
- V-PUMPV · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `talker`（speech）：差 +3.235（有 6 段 4.733 · 没有 4 段 1.497）
- V-TIMBRE · `bed-cafeteria`（bed）：差 +2.897（有 3 段 5.466 · 没有 7 段 2.570）
- V-TIMBRE · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.854（有 3 段 -7.659 · 没有 7 段 -9.512）
- V-NR · `babble`（speech）：差 +1.665（有 7 段 -8.457 · 没有 3 段 -10.122）
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
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `talker`（speech）：差 +0.388（有 6 段 -2.967 · 没有 4 段 -2.579）
- V-SPLVL · `bed-restaurant`（bed）：差 +0.241（有 7 段 -2.884 · 没有 3 段 -2.643）
- V-SPLVL · `dishes`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `laugh`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
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

### canteen · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（canteen-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.704 dB | -4.586（canteen-s02-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.054 dB | 7.746（canteen-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.076 比例 | 0.182（canteen-s00-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.525 dB | -2.157（canteen-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.217 dB² | 25.888（canteen-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 63.333 个/分钟 | 110.000（canteen-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.018 Δ | -0.001（canteen-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.741 dB | 7.777（canteen-s02-ext） | 6.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -7.072 dB | -5.537（canteen-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.436 dB | -2.340（canteen-s02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 2 / 4 | 1.000 比例 | 1.000（canteen-s00-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.863 dB | 5.235（canteen-s02-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `bed-restaurant`（bed）：差 +1.917（有 3 段 -3.662 · 没有 3 段 -1.745）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-cafeteria`（bed）：差 +0.272（有 3 段 3.190 · 没有 3 段 2.918）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.078（有 3 段 0.115 · 没有 3 段 0.036）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +11.074（有 3 段 20.754 · 没有 3 段 9.680）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +2.557（有 3 段 5.020 · 没有 3 段 2.463）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.383（有 3 段 -6.380 · 没有 3 段 -7.763）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（canteen-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.248 dB | -5.725（canteen-s02-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.598 dB | 8.927（canteen-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.113 比例 | 0.342（canteen-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.864 dB | -1.772（canteen-s03-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 19.872 dB² | 37.702（canteen-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 66.667 个/分钟 | 100.000（canteen-s02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.025 Δ | -0.003（canteen-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.266 dB | 6.918（canteen-s02-ext） | 6.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -8.122 dB | -4.894（canteen-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -4.919 dB | -2.322（canteen-s02-ext） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 2 / 4 | 0.502 比例 | 0.700（canteen-s03-ext） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 4.662 dB | 6.641（canteen-s04-ext） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `bed-restaurant`（bed）：差 +1.591（有 3 段 -3.044 · 没有 3 段 -1.452）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-restaurant`（bed）：差 +1.613（有 3 段 5.404 · 没有 3 段 3.791）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.128（有 3 段 0.177 · 没有 3 段 0.049）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +16.133（有 3 段 27.938 · 没有 3 段 11.806）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +1.448（有 3 段 3.990 · 没有 3 段 2.541）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +3.091（有 3 段 -6.576 · 没有 3 段 -9.668）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 2 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 1 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（canteen-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.386 dB | -4.243（canteen-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`（bed）：差 +0.541（有 3 段 -3.657 · 没有 3 段 -3.116）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### canteen · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（canteen-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.594 dB | -3.703（canteen-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 2.935 dB | 8.322（canteen-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.066 比例 | 0.147（canteen-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.450 dB | -3.724（canteen-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 16.394 dB² | 31.194（canteen-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 50.833 个/分钟 | 80.000（canteen-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.019 Δ | 0.001（canteen-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.112 dB | 6.560（canteen-s02-mba） | 6.000 | 不过 |
| V-NR |  | 4 / 2 / 0 | -7.617 dB | -5.609（canteen-s05-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.962 dB | -2.653（canteen-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 1 / 5 | 1.000 比例 | 1.000（canteen-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.292 dB | 4.946（canteen-s02-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `bed-restaurant`（bed）：差 +1.480（有 3 段 -3.334 · 没有 3 段 -1.854）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-cafeteria`（bed）：差 +1.431（有 3 段 3.651 · 没有 3 段 2.220）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.095（有 3 段 0.113 · 没有 3 段 0.018）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +5.452（有 3 段 19.119 · 没有 3 段 13.668）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `bed-restaurant`（bed）：差 +3.037（有 3 段 4.631 · 没有 3 段 1.594）
- V-TIMBRE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +1.804（有 3 段 -6.715 · 没有 3 段 -8.520）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（canteen-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.751 dB | -3.745（canteen-s02-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 5.715 dB | 11.510（canteen-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.118 比例 | 0.248（canteen-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.947 dB | -3.536（canteen-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 25.352 dB² | 49.305（canteen-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 49.167 个/分钟 | 75.000（canteen-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.029 Δ | -0.001（canteen-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.256 dB | 4.453（canteen-s02-mba） | 6.000 | 过 |
| V-NR |  | 5 / 1 / 0 | -8.123 dB | -5.082（canteen-s01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -6.023 dB | -2.543（canteen-s02-mba） | -10.000 | 不过 |
| V-SIM-FA |  | 0 / 1 / 5 | 0.103 比例 | 0.103（canteen-s00-mba） | 0.100 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 6.169 dB | 6.694（canteen-s05-mba） | 10.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `bed-restaurant`（bed）：差 +0.869（有 3 段 -2.186 · 没有 3 段 -1.316）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `bed-cafeteria`（bed）：差 +1.120（有 3 段 6.275 · 没有 3 段 5.155）
- V-ONSET · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-restaurant`（bed）：差 +0.132（有 3 段 0.184 · 没有 3 段 0.052）
- V-SIM-GATE · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `bed-restaurant`（bed）：差 +4.780（有 3 段 27.742 · 没有 3 段 22.962）
- V-PUMPV · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `bed-cafeteria`（bed）：差 +3.363（有 3 段 -6.442 · 没有 3 段 -9.804）
- V-NR · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-cafeteria`：样本不够（有 1 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-FA · `babble`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-FA · `bed-cafeteria`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）

### canteen · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（canteen-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（canteen-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 1 / 5 / 0 | -3.265 dB | -3.830（canteen-s04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-restaurant`（bed）：差 +0.102（有 3 段 -3.316 · 没有 3 段 -3.214）
- V-SPLVL · `babble`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `dishes`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### transit-hub · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（transit-hub-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.125 dB | -2.951（transit-hub-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 6 / 0 | 3.053 dB | 5.631（transit-hub-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.040 比例 | 0.078（transit-hub-c01-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.977 dB | -3.599（transit-hub-c06-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 3.683 dB² | 9.447（transit-hub-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 44.500 个/分钟 | 70.000（transit-hub-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.054 Δ | 0.025（transit-hub-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.843 dB | 2.109（transit-hub-c03-ext） | 6.000 | 过 |
| V-NR |  | 8 / 2 / 0 | -8.215 dB | -5.194（transit-hub-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -6.518 dB | -5.712（transit-hub-c01-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.013 dB | 9.706（transit-hub-c09-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.091 × | 1.824（transit-hub-c04-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-station`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-ONSET · `bed-station`（bed）：差 +1.594（有 7 段 3.531 · 没有 3 段 1.938）
- V-ONSET · `footsteps`（event）：差 +1.390（有 4 段 3.888 · 没有 6 段 2.497）
- V-ONSET · `babble`（speech）：差 +0.428（有 4 段 3.310 · 没有 6 段 2.882）
- V-SIM-GATE · `bed-station`（bed）：差 +0.028（有 7 段 0.048 · 没有 3 段 0.020）
- V-SIM-GATE · `footsteps`（event）：差 +0.019（有 4 段 0.051 · 没有 6 段 0.032）
- V-SIM-GATE · `babble`（speech）：差 +0.003（有 4 段 0.041 · 没有 6 段 0.039）
- V-PUMPV · `footsteps`（event）：差 +3.034（有 4 段 5.503 · 没有 6 段 2.469）
- V-PUMPV · `bed-station`（bed）：差 +2.112（有 7 段 4.316 · 没有 3 段 2.205）
- V-PUMPV · `babble`（speech）：差 +1.660（有 4 段 4.679 · 没有 6 段 3.019）
- V-NR · `babble`（speech）：差 +3.048（有 4 段 -6.387 · 没有 6 段 -9.435）
- V-NR · `announce`（speech）：差 +1.194（有 3 段 -7.380 · 没有 7 段 -8.574）
- V-NR · `bed-ace`（bed）：差 +1.194（有 3 段 -7.380 · 没有 7 段 -8.574）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### transit-hub · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（transit-hub-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.392 dB | -0.897（transit-hub-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 3.146 dB | 6.484（transit-hub-c03-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.048 比例 | 0.110（transit-hub-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.738 dB | -3.178（transit-hub-c06-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.069 dB² | 9.246（transit-hub-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 38.000 个/分钟 | 65.000（transit-hub-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.067 Δ | 0.030（transit-hub-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.383 dB | 8.815（transit-hub-c03-ext） | 6.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.650 dB | -5.259（transit-hub-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -7.433 dB | -6.281（transit-hub-c01-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.507 dB | 6.297（transit-hub-c09-ext） | 10.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.087 × | 1.498（transit-hub-c00-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-ace`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `pa-chime`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-ONSET · `bed-station`（bed）：差 +1.466（有 7 段 3.586 · 没有 3 段 2.120）
- V-ONSET · `footsteps`（event）：差 +1.272（有 4 段 3.909 · 没有 6 段 2.637）
- V-ONSET · `babble`（speech）：差 +1.106（有 4 段 3.809 · 没有 6 段 2.704）
- V-SIM-GATE · `bed-station`（bed）：差 +0.043（有 7 段 0.061 · 没有 3 段 0.018）
- V-SIM-GATE · `footsteps`（event）：差 +0.027（有 4 段 0.064 · 没有 6 段 0.037）
- V-SIM-GATE · `babble`（speech）：差 +0.019（有 4 段 0.059 · 没有 6 段 0.041）
- V-PUMPV · `bed-station`（bed）：差 +2.828（有 7 段 4.917 · 没有 3 段 2.089）
- V-PUMPV · `footsteps`（event）：差 +2.593（有 4 段 5.625 · 没有 6 段 3.031）
- V-PUMPV · `babble`（speech）：差 +1.859（有 4 段 5.184 · 没有 6 段 3.325）
- V-TIMBRE · `babble`（speech）：差 +2.659（有 4 段 2.979 · 没有 6 段 0.320）
- V-TIMBRE · `footsteps`（event）：差 +1.855（有 4 段 2.497 · 没有 6 段 0.641）
- V-TIMBRE · `bed-station`（bed）：差 +1.530（有 7 段 1.842 · 没有 3 段 0.312）
- V-NR · `babble`（speech）：差 +4.085（有 4 段 -6.199 · 没有 6 段 -10.284）
- V-NR · `footsteps`（event）：差 +0.322（有 4 段 -8.457 · 没有 6 段 -8.779）
- V-NR · `announce`（speech）：差 +0.094（有 3 段 -8.584 · 没有 7 段 -8.678）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### transit-hub · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（transit-hub-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.598 dB | -2.608（transit-hub-c06-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### transit-hub · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（transit-hub-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -1.229 dB | -3.068（transit-hub-c04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 6 / 0 | 3.475 dB | 6.164（transit-hub-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.035 比例 | 0.062（transit-hub-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.506 dB | -4.635（transit-hub-c06-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.925 dB² | 16.068（transit-hub-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 48.500 个/分钟 | 90.000（transit-hub-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.057 Δ | 0.033（transit-hub-c04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.790 dB | 2.519（transit-hub-c04-mba） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -8.078 dB | -5.924（transit-hub-c08-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -6.204 dB | -5.246（transit-hub-c06-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 3.663 dB | 6.081（transit-hub-c02-mba） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.046 × | 2.773（transit-hub-c02-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-station`（bed）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-SPLVL · `bed-station`（bed）：差 +0.784（有 7 段 -1.464 · 没有 3 段 -0.681）
- V-ONSET · `bed-station`（bed）：差 +1.222（有 7 段 3.842 · 没有 3 段 2.620）
- V-ONSET · `footsteps`（event）：差 +1.181（有 4 段 4.184 · 没有 6 段 3.003）
- V-SIM-GATE · `footsteps`（event）：差 +0.019（有 4 段 0.046 · 没有 6 段 0.028）
- V-SIM-GATE · `bed-station`（bed）：差 +0.012（有 7 段 0.039 · 没有 3 段 0.027）
- V-PUMPV · `footsteps`（event）：差 +7.237（有 4 段 10.267 · 没有 6 段 3.030）
- V-PUMPV · `announce`（speech）：差 +1.481（有 3 段 6.961 · 没有 7 段 5.481）
- V-PUMPV · `bed-ace`（bed）：差 +1.481（有 3 段 6.961 · 没有 7 段 5.481）
- V-NR · `announce`（speech）：差 +1.205（有 3 段 -7.235 · 没有 7 段 -8.440）
- V-NR · `bed-ace`（bed）：差 +1.205（有 3 段 -7.235 · 没有 7 段 -8.440）
- V-NR · `pa-chime`（event）：差 +1.205（有 3 段 -7.235 · 没有 7 段 -8.440）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.585（有 4 段 1.397 · 没有 6 段 0.812）
- V-MUSIC · `announce`（speech）：差 +0.041（有 3 段 1.075 · 没有 7 段 1.034）
- V-MUSIC · `bed-ace`（bed）：差 +0.041（有 3 段 1.075 · 没有 7 段 1.034）

### transit-hub · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（transit-hub-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.448 dB | -0.882（transit-hub-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 6 / 0 | 3.586 dB | 5.717（transit-hub-c02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.040 比例 | 0.075（transit-hub-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.321 dB | -4.231（transit-hub-c06-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 6.076 dB² | 13.328（transit-hub-c09-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 31.000 个/分钟 | 70.000（transit-hub-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.070 Δ | 0.041（transit-hub-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.744 dB | 3.487（transit-hub-c03-mba） | 6.000 | 过 |
| V-NR |  | 8 / 2 / 0 | -8.458 dB | -5.317（transit-hub-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 6 | -6.860 dB | -6.511（transit-hub-c01-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.454 dB | 6.683（transit-hub-c03-mba） | 10.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.330 × | 2.991（transit-hub-c04-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-ace`（bed）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `pa-chime`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-ONSET · `bed-station`（bed）：差 +0.848（有 7 段 3.840 · 没有 3 段 2.992）
- V-ONSET · `babble`（speech）：差 +0.625（有 4 段 3.961 · 没有 6 段 3.336）
- V-ONSET · `footsteps`（event）：差 +0.533（有 4 段 3.905 · 没有 6 段 3.372）
- V-SIM-GATE · `bed-station`（bed）：差 +0.023（有 7 段 0.047 · 没有 3 段 0.024）
- V-SIM-GATE · `footsteps`（event）：差 +0.010（有 4 段 0.046 · 没有 6 段 0.037）
- V-SIM-GATE · `babble`（speech）：差 +0.002（有 4 段 0.042 · 没有 6 段 0.040）
- V-PUMPV · `footsteps`（event）：差 +4.671（有 4 段 8.878 · 没有 6 段 4.208）
- V-PUMPV · `babble`（speech）：差 +0.702（有 4 段 6.497 · 没有 6 段 5.795）
- V-PUMPV · `talker`（speech）：差 +0.493（有 4 段 6.372 · 没有 6 段 5.879）
- V-NR · `babble`（speech）：差 +3.192（有 4 段 -6.543 · 没有 6 段 -9.735）
- V-NR · `announce`（speech）：差 +1.002（有 3 段 -7.757 · 没有 7 段 -8.759）
- V-NR · `bed-ace`（bed）：差 +1.002（有 3 段 -7.757 · 没有 7 段 -8.759）
- V-INTF · `announce`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +0.269（有 3 段 1.518 · 没有 7 段 1.249）
- V-MUSIC · `bed-ace`（bed）：差 +0.269（有 3 段 1.518 · 没有 7 段 1.249）
- V-MUSIC · `pa-chime`（event）：差 +0.269（有 3 段 1.518 · 没有 7 段 1.249）

### transit-hub · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（transit-hub-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（transit-hub-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.850 dB | -2.813（transit-hub-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### transit-hub · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（transit-hub-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.459 dB | -2.210（transit-hub-s01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 2.401 dB | 4.354（transit-hub-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 2 / 0 | 0.037 比例 | 0.075（transit-hub-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.760 dB | -7.965（transit-hub-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.742 dB² | 22.289（transit-hub-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 74.167 个/分钟 | 115.000（transit-hub-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.038 Δ | 0.031（transit-hub-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.185 dB | 6.566（transit-hub-s00-ext） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -6.611 dB | -4.698（transit-hub-s01-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.139 dB | -2.776（transit-hub-s02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.713 dB | 3.313（transit-hub-s04-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.297 × | 1.693（transit-hub-s01-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.034（有 3 段 0.054 · 没有 3 段 0.020）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +6.639（有 3 段 13.062 · 没有 3 段 6.423）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（transit-hub-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.176 dB | -3.283（transit-hub-s01-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 2.652 dB | 4.714（transit-hub-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.073 比例 | 0.265（transit-hub-s01-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.221 dB | -7.521（transit-hub-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.750 dB² | 17.571（transit-hub-s02-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 36.667 个/分钟 | 60.000（transit-hub-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.049 Δ | 0.024（transit-hub-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 3.658 dB | 7.094（transit-hub-s00-ext） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -6.972 dB | -4.971（transit-hub-s00-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -5.059 dB | -4.442（transit-hub-s03-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 4.877 dB | 6.697（transit-hub-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.336 × | 1.657（transit-hub-s04-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `footsteps`（event）：差 +0.085（有 3 段 2.695 · 没有 3 段 2.610）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +1.251（有 3 段 10.375 · 没有 3 段 9.125）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（transit-hub-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.732 dB | -3.743（transit-hub-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
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

### transit-hub · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（transit-hub-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.503 dB | -2.249（transit-hub-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.106 dB | 6.238（transit-hub-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.031 比例 | 0.065（transit-hub-s05-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.050 dB | -7.844（transit-hub-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.264 dB² | 11.116（transit-hub-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 73.333 个/分钟 | 100.000（transit-hub-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.034 Δ | 0.024（transit-hub-s05-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.561 dB | 4.076（transit-hub-s02-mba） | 6.000 | 过 |
| V-NR |  | 4 / 2 / 0 | -6.896 dB | -4.840（transit-hub-s01-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -3.349 dB | -3.149（transit-hub-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.158 dB | 4.098（transit-hub-s05-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.421 × | 1.772（transit-hub-s01-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.023（有 3 段 0.042 · 没有 3 段 0.019）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（transit-hub-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.086 dB | -2.499（transit-hub-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 3.120 dB | 4.612（transit-hub-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.088 比例 | 0.212（transit-hub-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.606 dB | -7.483（transit-hub-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.266 dB² | 16.527（transit-hub-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 44.167 个/分钟 | 80.000（transit-hub-s04-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.046 Δ | 0.029（transit-hub-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.403 dB | 3.700（transit-hub-s00-mba） | 6.000 | 过 |
| V-NR |  | 4 / 2 / 0 | -6.739 dB | -4.921（transit-hub-s00-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 3 | -4.523 dB | -2.706（transit-hub-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 4.835 dB | 7.216（transit-hub-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.542 × | 2.199（transit-hub-s04-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）

### transit-hub · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（transit-hub-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（transit-hub-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.675 dB | -3.463（transit-hub-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
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

### car-passenger · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（car-passenger-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.640 dB | -0.984（car-passenger-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 3.288 dB | 8.309（car-passenger-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.028 比例 | 0.056（car-passenger-c02-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.477 dB | -5.310（car-passenger-c08-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.319 dB² | 9.446（car-passenger-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 35.000 个/分钟 | 50.000（car-passenger-c04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.054 Δ | 0.023（car-passenger-c07-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.875 dB | 3.654（car-passenger-c07-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.138 dB | -7.752（car-passenger-c06-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -5.381 dB | -3.826（car-passenger-c04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.740 dB | 6.403（car-passenger-c06-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.890 × | 3.846（car-passenger-c09-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `talker`（speech）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.262（有 7 段 3.966 · 没有 3 段 1.704）
- V-ONSET · `nav-voice`（speech）：差 +0.843（有 3 段 3.878 · 没有 7 段 3.035）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.182（有 7 段 5.674 · 没有 3 段 4.492）
- V-PUMPV · `nav-voice`（speech）：差 +0.311（有 3 段 5.537 · 没有 7 段 5.226）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.216（有 7 段 1.955 · 没有 3 段 1.739）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（car-passenger-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.290 dB | -0.464（car-passenger-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 3.582 dB | 10.803（car-passenger-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.045 比例 | 0.111（car-passenger-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.144 dB | -5.066（car-passenger-c08-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 6.435 dB² | 15.255（car-passenger-c09-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 25.500 个/分钟 | 45.000（car-passenger-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.059 Δ | 0.034（car-passenger-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.900 dB | 3.191（car-passenger-c07-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.761 dB | -9.929（car-passenger-c08-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -7.126 dB | -4.357（car-passenger-c04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.826 dB | 6.044（car-passenger-c05-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.826 × | 3.108（car-passenger-c09-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `nav-voice`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.766（有 7 段 4.412 · 没有 3 段 1.646）
- V-ONSET · `nav-voice`（speech）：差 +0.010（有 3 段 3.589 · 没有 7 段 3.580）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.015（有 7 段 0.050 · 没有 3 段 0.035）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.003（有 3 段 0.047 · 没有 7 段 0.044）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +2.506（有 7 段 7.187 · 没有 3 段 4.680）
- V-PUMPV · `nav-voice`（speech）：差 +1.492（有 3 段 7.479 · 没有 7 段 5.987）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.384（有 7 段 1.941 · 没有 3 段 1.557）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（car-passenger-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.342 dB | -2.247（car-passenger-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### car-passenger · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（car-passenger-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.569 dB | -0.681（car-passenger-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 3.066 dB | 8.981（car-passenger-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.032 比例 | 0.086（car-passenger-c04-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.488 dB | -6.421（car-passenger-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 4.803 dB² | 9.130（car-passenger-c01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 35.500 个/分钟 | 65.000（car-passenger-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.053 Δ | 0.028（car-passenger-c07-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.975 dB | 3.447（car-passenger-c07-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.282 dB | -8.982（car-passenger-c06-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -5.383 dB | -4.014（car-passenger-c04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.669 dB | 6.377（car-passenger-c03-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.177 × | 5.228（car-passenger-c09-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `talker`（speech）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.890（有 7 段 3.633 · 没有 3 段 1.743）
- V-ONSET · `nav-voice`（speech）：差 +0.002（有 3 段 3.067 · 没有 7 段 3.065）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.013（有 3 段 0.041 · 没有 7 段 0.028）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +3.029（有 3 段 6.923 · 没有 7 段 3.894）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.645（有 7 段 2.371 · 没有 3 段 1.726）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（car-passenger-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.257 dB | -0.362（car-passenger-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 3.249 dB | 11.090（car-passenger-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.035 比例 | 0.087（car-passenger-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.188 dB | -5.821（car-passenger-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.453 dB² | 12.187（car-passenger-c09-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 25.500 个/分钟 | 50.000（car-passenger-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.059 Δ | 0.038（car-passenger-c06-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 1.069 dB | 3.307（car-passenger-c07-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.678 dB | -9.868（car-passenger-c03-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 6 | -6.880 dB | -4.060（car-passenger-c04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.876 dB | 6.046（car-passenger-c03-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 5 / 0 | 2.129 × | 4.047（car-passenger-c09-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `nav-voice`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.658（有 7 段 4.046 · 没有 3 段 1.388）
- V-ONSET · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.004（有 7 段 0.036 · 没有 3 段 0.032）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +2.418（有 3 段 7.146 · 没有 7 段 4.728）
- V-PUMPV · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 1 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.902（有 7 段 2.399 · 没有 3 段 1.497）
- V-MUSIC · `bed-car-mcv`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### car-passenger · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（car-passenger-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（car-passenger-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.421 dB | -2.868（car-passenger-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### car-passenger · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（car-passenger-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.430 dB | -1.781（car-passenger-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 6 / 0 | 6.268 dB | 11.385（car-passenger-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.082 比例 | 0.123（car-passenger-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.934 dB | -14.475（car-passenger-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 16.460 dB² | 44.907（car-passenger-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 52.500 个/分钟 | 105.000（car-passenger-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.030 Δ | 0.017（car-passenger-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.297 dB | 7.645（car-passenger-s05-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.455 dB | -7.827（car-passenger-s05-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -5.392 dB | -4.136（car-passenger-s02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 2.929 dB | 5.661（car-passenger-s04-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 0.922 × | 1.686（car-passenger-s01-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `nav-voice`（speech）：差 +3.904（有 3 段 8.220 · 没有 3 段 4.316）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.000（有 3 段 0.082 · 没有 3 段 0.081）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +11.211（有 3 段 22.066 · 没有 3 段 10.855）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（car-passenger-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.117 dB | -1.508（car-passenger-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 6 / 0 | 6.036 dB | 8.489（car-passenger-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.110 比例 | 0.176（car-passenger-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.575 dB | -14.153（car-passenger-s04-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.654 dB² | 23.326（car-passenger-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 45.833 个/分钟 | 70.000（car-passenger-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.028 Δ | 0.007（car-passenger-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.031 dB | 3.479（car-passenger-s03-ext） | 6.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -10.107 dB | -8.728（car-passenger-s02-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -5.702 dB | -4.869（car-passenger-s02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 5.635 dB | 6.497（car-passenger-s05-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.155 × | 2.411（car-passenger-s05-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `nav-voice`（speech）：差 +3.084（有 3 段 7.578 · 没有 3 段 4.494）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.022（有 3 段 0.121 · 没有 3 段 0.099）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `nav-voice`（speech）：差 +0.630（有 3 段 15.969 · 没有 3 段 15.339）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（car-passenger-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -2.790 dB | -3.714（car-passenger-s00-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
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

### car-passenger · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（car-passenger-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.365 dB | -1.969（car-passenger-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 5.215 dB | 6.295（car-passenger-s02-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.067 比例 | 0.142（car-passenger-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.277 dB | -13.533（car-passenger-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.648 dB² | 19.282（car-passenger-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 61.667 个/分钟 | 120.000（car-passenger-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.019（car-passenger-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.161 dB | 6.076（car-passenger-s05-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.238 dB | -8.228（car-passenger-s05-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -5.363 dB | -4.415（car-passenger-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 5 / 0 / 1 | 3.881 dB | 6.032（car-passenger-s05-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.064 × | 1.784（car-passenger-s01-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `nav-voice`（speech）：差 +1.839（有 3 段 6.135 · 没有 3 段 4.296）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（car-passenger-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.017 dB | -1.488（car-passenger-s03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 6 / 0 | 5.590 dB | 7.021（car-passenger-s03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.094 比例 | 0.166（car-passenger-s04-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -14.945 dB | -13.166（car-passenger-s04-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.506 dB² | 17.771（car-passenger-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 43.333 个/分钟 | 70.000（car-passenger-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.031 Δ | 0.008（car-passenger-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 2.194 dB | 4.032（car-passenger-s03-mba） | 6.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -10.042 dB | -8.319（car-passenger-s02-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -5.713 dB | -4.978（car-passenger-s02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 5 / 0 / 1 | 6.008 dB | 7.696（car-passenger-s05-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.168 × | 2.187（car-passenger-s05-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `nav-voice`（speech）：差 +1.893（有 3 段 6.537 · 没有 3 段 4.644）
- V-ONSET · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `nav-voice`（speech）：差 +0.031（有 3 段 0.110 · 没有 3 段 0.078）
- V-SIM-GATE · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `bed-car-mcv`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `car-horn`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）

### car-passenger · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（car-passenger-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（car-passenger-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -2.867 dB | -4.176（car-passenger-s00-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-car-mcv`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `car-horn`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
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

### bus-metro-cabin · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（bus-metro-cabin-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.460 dB | -2.756（bus-metro-cabin-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 1.532 dB | 3.490（bus-metro-cabin-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.044 比例 | 0.118（bus-metro-cabin-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.208 dB | -0.442（bus-metro-cabin-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 2.838 dB² | 6.911（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 8 / 0 | 19.000 个/分钟 | 40.000（bus-metro-cabin-c06-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.031 Δ | 0.007（bus-metro-cabin-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 2.503 dB | 8.196（bus-metro-cabin-c05-ext） | 6.000 | 不过 |
| V-NR |  | 7 / 3 / 0 | -8.080 dB | -3.044（bus-metro-cabin-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -5.166 dB | -4.191（bus-metro-cabin-c04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.832 dB | 7.429（bus-metro-cabin-c09-ext） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.327 × | 2.368（bus-metro-cabin-c07-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `door-pneumatic`（event）：差 +1.407（有 3 段 2.517 · 没有 7 段 1.110）
- V-ONSET · `brake`（event）：差 +1.101（有 3 段 2.303 · 没有 7 段 1.202）
- V-ONSET · `bed-metro`（bed）：差 +1.063（有 6 段 1.957 · 没有 4 段 0.895）
- V-ONSET · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.045（有 3 段 0.076 · 没有 7 段 0.031）
- V-SIM-GATE · `bed-metro`（bed）：差 +0.036（有 6 段 0.059 · 没有 4 段 0.023）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `door-pneumatic`（event）：差 +3.912（有 3 段 5.577 · 没有 7 段 1.664）
- V-PUMPV · `bed-metro`（bed）：差 +2.495（有 6 段 3.836 · 没有 4 段 1.341）
- V-PUMPV · `talker`（speech）：差 +0.960（有 3 段 3.510 · 没有 7 段 2.550）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `door-pneumatic`（event）：差 +2.938（有 3 段 4.559 · 没有 7 段 1.621）
- V-TIMBRE · `bed-metro`（bed）：差 +2.770（有 6 段 3.611 · 没有 4 段 0.841）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `talker`（speech）：差 +0.123（有 3 段 -7.995 · 没有 7 段 -8.117）
- V-NR · `bed-bus`（bed）：差 +0.011（有 4 段 -8.074 · 没有 6 段 -8.085）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-bus`（bed）：差 +0.537（有 4 段 1.649 · 没有 6 段 1.113）
- V-MUSIC · `brake`（event）：差 +0.346（有 3 段 1.570 · 没有 7 段 1.223）
- V-MUSIC · `door-chime`（event）：差 +0.305（有 3 段 1.541 · 没有 7 段 1.236）
- V-MUSIC · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### bus-metro-cabin · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（bus-metro-cabin-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.360 dB | -1.435（bus-metro-cabin-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 2.091 dB | 4.246（bus-metro-cabin-c09-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.042 比例 | 0.093（bus-metro-cabin-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -4.260 dB | -0.995（bus-metro-cabin-c07-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 3.619 dB² | 7.489（bus-metro-cabin-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 14.500 个/分钟 | 30.000（bus-metro-cabin-c00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.051 Δ | 0.029（bus-metro-cabin-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 2.157 dB | 8.132（bus-metro-cabin-c05-ext） | 6.000 | 不过 |
| V-NR |  | 7 / 3 / 0 | -8.918 dB | -3.153（bus-metro-cabin-c06-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -6.793 dB | -5.366（bus-metro-cabin-c04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.942 dB | 6.411（bus-metro-cabin-c03-ext） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.450 × | 3.365（bus-metro-cabin-c07-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `brake`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `door-chime`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `door-pneumatic`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `door-chime`（event）：差 +1.453（有 3 段 3.109 · 没有 7 段 1.655）
- V-ONSET · `door-pneumatic`（event）：差 +1.364（有 3 段 3.046 · 没有 7 段 1.682）
- V-ONSET · `talker`（speech）：差 +1.169（有 3 段 2.910 · 没有 7 段 1.740）
- V-ONSET · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.022（有 3 段 0.058 · 没有 7 段 0.036）
- V-SIM-GATE · `bed-metro`（bed）：差 +0.016（有 6 段 0.048 · 没有 4 段 0.033）
- V-SIM-GATE · `talker`（speech）：差 +0.009（有 3 段 0.048 · 没有 7 段 0.040）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `door-pneumatic`（event）：差 +4.772（有 3 段 6.960 · 没有 7 段 2.187）
- V-PUMPV · `bed-metro`（bed）：差 +3.182（有 6 段 4.892 · 没有 4 段 1.710）
- V-PUMPV · `talker`（speech）：差 +2.225（有 3 段 5.176 · 没有 7 段 2.952）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `bed-metro`（bed）：差 +2.774（有 6 段 3.266 · 没有 4 段 0.492）
- V-TIMBRE · `door-pneumatic`（event）：差 +2.568（有 3 段 3.955 · 没有 7 段 1.386）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-metro`（bed）：差 +1.714（有 6 段 -8.233 · 没有 4 段 -9.947）
- V-NR · `door-pneumatic`（event）：差 +1.328（有 3 段 -7.988 · 没有 7 段 -9.317）
- V-NR · `talker`（speech）：差 +0.480（有 3 段 -8.583 · 没有 7 段 -9.062）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-bus`（bed）：差 +0.779（有 4 段 1.917 · 没有 6 段 1.138）
- V-MUSIC · `brake`（event）：差 +0.700（有 3 段 1.940 · 没有 7 段 1.240）
- V-MUSIC · `door-chime`（event）：差 +0.609（有 3 段 1.876 · 没有 7 段 1.267）
- V-MUSIC · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### bus-metro-cabin · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.419 dB | -2.510（bus-metro-cabin-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### bus-metro-cabin · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（bus-metro-cabin-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -1.910 dB | -3.090（bus-metro-cabin-c03-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 2.109 dB | 5.219（bus-metro-cabin-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.046 比例 | 0.110（bus-metro-cabin-c02-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -8.222 dB | -1.904（bus-metro-cabin-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 7.284 dB² | 33.398（bus-metro-cabin-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 27.500 个/分钟 | 45.000（bus-metro-cabin-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.032 Δ | 0.014（bus-metro-cabin-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 3.475 dB | 7.127（bus-metro-cabin-c06-mba） | 6.000 | 不过 |
| V-NR |  | 7 / 3 / 0 | -7.504 dB | -1.371（bus-metro-cabin-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -5.136 dB | -2.802（bus-metro-cabin-c03-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.103 dB | 6.120（bus-metro-cabin-c01-mba） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.268 × | 2.114（bus-metro-cabin-c02-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SPLVL · `talker`（speech）：差 +0.536（有 3 段 -2.286 · 没有 7 段 -1.749）
- V-SPLVL · `door-chime`（event）：差 +0.519（有 3 段 -2.274 · 没有 7 段 -1.754）
- V-SPLVL · `door-pneumatic`（event）：差 +0.302（有 3 段 -2.122 · 没有 7 段 -1.820）
- V-SPLVL · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `door-pneumatic`（event）：差 +1.479（有 3 段 3.145 · 没有 7 段 1.666）
- V-ONSET · `bed-metro`（bed）：差 +0.462（有 6 段 2.294 · 没有 4 段 1.832）
- V-ONSET · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.044（有 3 段 0.076 · 没有 7 段 0.032）
- V-SIM-GATE · `bed-metro`（bed）：差 +0.032（有 6 段 0.058 · 没有 4 段 0.027）
- V-SIM-GATE · `talker`（speech）：差 +0.002（有 3 段 0.047 · 没有 7 段 0.045）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +10.438（有 3 段 14.590 · 没有 7 段 4.152）
- V-PUMPV · `door-pneumatic`（event）：差 +10.071（有 3 段 14.333 · 没有 7 段 4.263）
- V-PUMPV · `bed-metro`（bed）：差 +5.236（有 6 段 9.378 · 没有 4 段 4.142）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `bed-metro`（bed）：差 +1.784（有 6 段 4.189 · 没有 4 段 2.405）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-metro`（bed）：差 +0.356（有 6 段 -7.362 · 没有 4 段 -7.718）
- V-NR · `talker`（speech）：差 +0.037（有 3 段 -7.479 · 没有 7 段 -7.515）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.489（有 3 段 1.610 · 没有 7 段 1.121）
- V-MUSIC · `door-pneumatic`（event）：差 +0.152（有 3 段 1.374 · 没有 7 段 1.222）
- V-MUSIC · `bed-bus`（bed）：差 +0.082（有 4 段 1.317 · 没有 6 段 1.235）
- V-MUSIC · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### bus-metro-cabin · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（bus-metro-cabin-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.858 dB | -2.278（bus-metro-cabin-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 2.314 dB | 5.525（bus-metro-cabin-c09-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.068 比例 | 0.280（bus-metro-cabin-c06-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -7.823 dB | -1.521（bus-metro-cabin-c07-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 8.146 dB² | 23.485（bus-metro-cabin-c02-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 17.500 个/分钟 | 35.000（bus-metro-cabin-c00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.047 Δ | 0.029（bus-metro-cabin-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 2.984 dB | 7.604（bus-metro-cabin-c02-mba） | 6.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.930 dB | -1.121（bus-metro-cabin-c06-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 5 | -7.454 dB | -5.747（bus-metro-cabin-c04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.231 dB | 6.755（bus-metro-cabin-c08-mba） | 10.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.363 × | 2.454（bus-metro-cabin-c00-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `brake`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `door-chime`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `door-pneumatic`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-ONSET · `door-pneumatic`（event）：差 +0.986（有 3 段 3.004 · 没有 7 段 2.018）
- V-ONSET · `door-chime`（event）：差 +0.779（有 3 段 2.859 · 没有 7 段 2.080）
- V-ONSET · `bed-bus`（bed）：差 +0.755（有 4 段 2.767 · 没有 6 段 2.012）
- V-ONSET · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-metro`（bed）：差 +0.061（有 6 段 0.093 · 没有 4 段 0.032）
- V-SIM-GATE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +6.486（有 3 段 12.687 · 没有 7 段 6.201）
- V-PUMPV · `door-pneumatic`（event）：差 +5.281（有 3 段 11.843 · 没有 7 段 6.562）
- V-PUMPV · `bed-metro`（bed）：差 +4.927（有 6 段 10.117 · 没有 4 段 5.190）
- V-PUMPV · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-TIMBRE · `bed-metro`（bed）：差 +3.114（有 6 段 4.230 · 没有 4 段 1.115）
- V-TIMBRE · `door-pneumatic`（event）：差 +1.738（有 3 段 4.200 · 没有 7 段 2.462）
- V-TIMBRE · `talker`（speech）：差 +0.400（有 3 段 3.264 · 没有 7 段 2.864）
- V-TIMBRE · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-NR · `bed-metro`（bed）：差 +2.263（有 6 段 -7.025 · 没有 4 段 -9.288）
- V-NR · `door-pneumatic`（event）：差 +1.077（有 3 段 -7.176 · 没有 7 段 -8.253）
- V-NR · `talker`（speech）：差 +0.285（有 3 段 -7.730 · 没有 7 段 -8.015）
- V-NR · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 1 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `talker`（speech）：差 +0.206（有 3 段 1.507 · 没有 7 段 1.301）
- V-MUSIC · `bed-metro`（bed）：差 +0.093（有 6 段 1.400 · 没有 4 段 1.307）
- V-MUSIC · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）

### bus-metro-cabin · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 9 / 1 / 0 | -2.225 dB | -3.092（bus-metro-cabin-c08-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `talker`（speech）：差 +0.758（有 3 段 -2.755 · 没有 7 段 -1.997）
- V-SPLVL · `door-chime`（event）：差 +0.545（有 3 段 -2.606 · 没有 7 段 -2.061）
- V-SPLVL · `bed-bus`（bed）：差 +0.293（有 4 段 -2.401 · 没有 6 段 -2.107）
- V-SPLVL · `announce`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 1 段 · 没有 9 段，每组要 ≥ 3）
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

### bus-metro-cabin · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（bus-metro-cabin-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -2.024 dB | -2.156（bus-metro-cabin-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 3 / 2 | 3.027 dB | 4.065（bus-metro-cabin-s02-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.090 比例 | 0.369（bus-metro-cabin-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -12.123 dB | -7.021（bus-metro-cabin-s02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 13.314 dB² | 28.444（bus-metro-cabin-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 26.667 个/分钟 | 50.000（bus-metro-cabin-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.012 Δ | 0.003（bus-metro-cabin-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 2 / 1 | 7.080 dB | 13.073（bus-metro-cabin-s04-ext） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -7.073 dB | -4.890（bus-metro-cabin-s04-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -5.565 dB | -4.188（bus-metro-cabin-s04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.074 dB | 0.299（bus-metro-cabin-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.109 × | 1.606（bus-metro-cabin-s03-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.098（有 3 段 0.139 · 没有 3 段 0.041）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door-chime`（event）：差 +3.555（有 3 段 15.092 · 没有 3 段 11.537）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `brake`（event）：差 +0.953（有 3 段 -6.596 · 没有 3 段 -7.549）
- V-NR · `door-chime`（event）：差 +0.893（有 3 段 -6.626 · 没有 3 段 -7.519）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（bus-metro-cabin-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 4 / 1 / 1 | -2.792 dB | -8.752（bus-metro-cabin-s02-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 2 / 2 | 2.747 dB | 4.956（bus-metro-cabin-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.184 比例 | 0.410（bus-metro-cabin-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -11.065 dB | -6.601（bus-metro-cabin-s02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 17.789 dB² | 34.453（bus-metro-cabin-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 17.500 个/分钟 | 35.000（bus-metro-cabin-s04-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.019 Δ | 0.008（bus-metro-cabin-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 2 / 1 | 6.268 dB | 11.869（bus-metro-cabin-s04-ext） | 6.000 | 不过 |
| V-NR |  | 3 / 3 / 0 | -6.701 dB | -3.800（bus-metro-cabin-s04-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 4 / 2 | -6.369 dB | -3.469（bus-metro-cabin-s04-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.556 dB | 3.115（bus-metro-cabin-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 0.924 × | 1.037（bus-metro-cabin-s04-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door-chime`（event）：差 +5.132（有 3 段 20.356 · 没有 3 段 15.223）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `brake`（event）：差 +0.910（有 3 段 -6.246 · 没有 3 段 -7.156）
- V-NR · `door-chime`（event）：差 +0.340（有 3 段 -6.531 · 没有 3 段 -6.871）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 4 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 2 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 0 / 1 | -2.263 dB | -2.774（bus-metro-cabin-s05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### bus-metro-cabin · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（bus-metro-cabin-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.691 dB | -4.269（bus-metro-cabin-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 4 / 1 | 6.373 dB | 18.660（bus-metro-cabin-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.072 比例 | 0.307（bus-metro-cabin-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.513 dB | -10.672（bus-metro-cabin-s03-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 17.323 dB² | 35.625（bus-metro-cabin-s03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 23.333 个/分钟 | 30.000（bus-metro-cabin-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.010 Δ | -0.001（bus-metro-cabin-s04-mba） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 3 / 1 | 10.163 dB | 19.505（bus-metro-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 2 / 4 / 0 | -7.064 dB | -4.132（bus-metro-cabin-s04-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 5 / 1 | -4.880 dB | -3.790（bus-metro-cabin-s04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.208 dB | 1.108（bus-metro-cabin-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.114 × | 1.486（bus-metro-cabin-s00-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `door-chime`（event）：差 +0.715（有 3 段 -3.049 · 没有 3 段 -2.334）
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.100（有 3 段 0.121 · 没有 3 段 0.022）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `brake`（event）：差 +4.603（有 3 段 19.625 · 没有 3 段 15.022）
- V-PUMPV · `door-pneumatic`（event）：差 +4.294（有 3 段 19.471 · 没有 3 段 15.176）
- V-PUMPV · `door-chime`（event）：差 +3.543（有 3 段 19.095 · 没有 3 段 15.552）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `brake`（event）：差 +1.041（有 3 段 -6.544 · 没有 3 段 -7.585）
- V-NR · `door-chime`（event）：差 +0.395（有 3 段 -6.867 · 没有 3 段 -7.262）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（bus-metro-cabin-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -3.197 dB | -11.481（bus-metro-cabin-s02-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 3 / 1 | 5.056 dB | 14.341（bus-metro-cabin-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 5 / 0 | 0.337 比例 | 1.000（bus-metro-cabin-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -13.763 dB | -7.326（bus-metro-cabin-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 15.225 dB² | 26.114（bus-metro-cabin-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 25.833 个/分钟 | 40.000（bus-metro-cabin-s03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.000（bus-metro-cabin-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 2 / 3 / 1 | 8.851 dB | 16.696（bus-metro-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 2 / 4 / 0 | -6.296 dB | -2.407（bus-metro-cabin-s04-mba） | -6.000 | 不过 |
| V-INTF |  | 1 / 4 / 1 | -7.457 dB | -2.510（bus-metro-cabin-s04-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 0.578 dB | 3.294（bus-metro-cabin-s03-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.152 × | 2.053（bus-metro-cabin-s00-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `door-pneumatic`（event）：差 +3.649（有 3 段 -5.021 · 没有 3 段 -1.372）
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door-pneumatic`（event）：差 +0.142（有 3 段 0.408 · 没有 3 段 0.266）
- V-SIM-GATE · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `babble`：样本不够（有 4 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `brake`（event）：差 +1.927（有 3 段 -5.332 · 没有 3 段 -7.259）
- V-NR · `door-chime`（event）：差 +0.292（有 3 段 -6.150 · 没有 3 段 -6.442）
- V-NR · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-NR · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-INTF · `announce`：样本不够（有 5 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `babble`：样本不够（有 3 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）

### bus-metro-cabin · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（bus-metro-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（bus-metro-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.044 dB | -3.961（bus-metro-cabin-s01-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `door-chime`（event）：差 +0.826（有 3 段 -3.457 · 没有 3 段 -2.631）
- V-SPLVL · `door-pneumatic`（event）：差 +0.268（有 3 段 -3.178 · 没有 3 段 -2.910）
- V-SPLVL · `announce`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `babble`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
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

### rail-air-cabin · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（rail-air-cabin-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.700 dB | -2.300（rail-air-cabin-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 3.416 dB | 9.389（rail-air-cabin-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.055 比例 | 0.201（rail-air-cabin-c09-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -11.978 dB | -3.781（rail-air-cabin-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 5.242 dB² | 12.228（rail-air-cabin-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 23.500 个/分钟 | 40.000（rail-air-cabin-c03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.029 Δ | 0.019（rail-air-cabin-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 7 / 3 / 0 | 4.185 dB | 7.894（rail-air-cabin-c07-ext） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -9.599 dB | -7.108（rail-air-cabin-c09-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -5.098 dB | -3.433（rail-air-cabin-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.450 dB | 6.637（rail-air-cabin-c05-ext） | 10.000 | 过 |
| V-MUSIC |  | 4 / 6 / 0 | 2.146 × | 3.851（rail-air-cabin-c06-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +2.329（有 3 段 5.047 · 没有 7 段 2.717）
- V-ONSET · `pa-chime`（event）：差 +2.329（有 3 段 5.047 · 没有 7 段 2.717）
- V-ONSET · `trolley`（event）：差 +0.416（有 3 段 3.707 · 没有 7 段 3.291）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `announce`（speech）：差 +2.469（有 3 段 6.970 · 没有 7 段 4.501）
- V-PUMPV · `pa-chime`（event）：差 +2.469（有 3 段 6.970 · 没有 7 段 4.501）
- V-PUMPV · `trolley`（event）：差 +0.850（有 3 段 5.836 · 没有 7 段 4.987）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `announce`（speech）：差 +2.466（有 3 段 5.911 · 没有 7 段 3.446）
- V-TIMBRE · `pa-chime`（event）：差 +2.466（有 3 段 5.911 · 没有 7 段 3.446）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `trolley`（event）：差 +0.158（有 3 段 -4.993 · 没有 6 段 -5.151）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +1.049（有 3 段 2.880 · 没有 7 段 1.832）
- V-MUSIC · `pa-chime`（event）：差 +1.049（有 3 段 2.880 · 没有 7 段 1.832）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（rail-air-cabin-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.709 dB | -1.960（rail-air-cabin-c05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 6 / 0 | 4.412 dB | 11.227（rail-air-cabin-c04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.076 比例 | 0.263（rail-air-cabin-c05-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -11.739 dB | -3.343（rail-air-cabin-c02-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 7.654 dB² | 18.291（rail-air-cabin-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 25.000 个/分钟 | 35.000（rail-air-cabin-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.039 Δ | 0.019（rail-air-cabin-c00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 4.204 dB | 10.419（rail-air-cabin-c09-ext） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.937 dB | -10.265（rail-air-cabin-c06-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -4.977 dB | -2.568（rail-air-cabin-c09-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.029 dB | 6.099（rail-air-cabin-c06-ext） | 10.000 | 过 |
| V-MUSIC |  | 4 / 6 / 0 | 2.407 × | 5.668（rail-air-cabin-c02-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `pa-chime`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `trolley`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +3.289（有 3 段 6.714 · 没有 7 段 3.425）
- V-ONSET · `pa-chime`（event）：差 +3.289（有 3 段 6.714 · 没有 7 段 3.425）
- V-ONSET · `trolley`（event）：差 +0.941（有 3 段 5.070 · 没有 7 段 4.129）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `trolley`（event）：差 +0.065（有 3 段 0.122 · 没有 7 段 0.057）
- V-SIM-GATE · `announce`（speech）：差 +0.002（有 3 段 0.078 · 没有 7 段 0.076）
- V-SIM-GATE · `pa-chime`（event）：差 +0.002（有 3 段 0.078 · 没有 7 段 0.076）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `announce`（speech）：差 +5.975（有 3 段 11.836 · 没有 7 段 5.862）
- V-PUMPV · `pa-chime`（event）：差 +5.975（有 3 段 11.836 · 没有 7 段 5.862）
- V-PUMPV · `trolley`（event）：差 +3.435（有 3 段 10.058 · 没有 7 段 6.624）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `announce`（speech）：差 +1.044（有 3 段 4.935 · 没有 7 段 3.891）
- V-TIMBRE · `pa-chime`（event）：差 +1.044（有 3 段 4.935 · 没有 7 段 3.891）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +0.096（有 3 段 2.474 · 没有 7 段 2.378）
- V-MUSIC · `pa-chime`（event）：差 +0.096（有 3 段 2.474 · 没有 7 段 2.378）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.935 dB | -2.927（rail-air-cabin-c04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### rail-air-cabin · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（rail-air-cabin-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.826 dB | -2.382（rail-air-cabin-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 7 / 0 | 4.314 dB | 10.256（rail-air-cabin-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.069 比例 | 0.197（rail-air-cabin-c09-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -13.707 dB | -4.503（rail-air-cabin-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 10.051 dB² | 35.599（rail-air-cabin-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 26.500 个/分钟 | 40.000（rail-air-cabin-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.030 Δ | 0.018（rail-air-cabin-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 4 / 0 | 5.629 dB | 11.834（rail-air-cabin-c09-mba） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.493 dB | -9.687（rail-air-cabin-c03-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -5.016 dB | -3.867（rail-air-cabin-c09-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.645 dB | 6.648（rail-air-cabin-c06-mba） | 10.000 | 过 |
| V-MUSIC |  | 2 / 8 / 0 | 2.578 × | 4.186（rail-air-cabin-c08-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +1.994（有 3 段 5.710 · 没有 7 段 3.716）
- V-ONSET · `pa-chime`（event）：差 +1.994（有 3 段 5.710 · 没有 7 段 3.716）
- V-ONSET · `trolley`（event）：差 +0.451（有 3 段 4.630 · 没有 7 段 4.179）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.007（有 3 段 0.074 · 没有 7 段 0.067）
- V-SIM-GATE · `pa-chime`（event）：差 +0.007（有 3 段 0.074 · 没有 7 段 0.067）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `trolley`（event）：差 +11.567（有 3 段 18.148 · 没有 7 段 6.580）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `trolley`（event）：差 +1.247（有 3 段 6.502 · 没有 7 段 5.254）
- V-TIMBRE · `announce`（speech）：差 +0.019（有 3 段 5.642 · 没有 7 段 5.623）
- V-TIMBRE · `pa-chime`（event）：差 +0.019（有 3 段 5.642 · 没有 7 段 5.623）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +0.909（有 3 段 3.215 · 没有 7 段 2.305）
- V-MUSIC · `pa-chime`（event）：差 +0.909（有 3 段 3.215 · 没有 7 段 2.305）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（rail-air-cabin-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.646 dB | -1.778（rail-air-cabin-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 7 / 0 | 4.926 dB | 12.493（rail-air-cabin-c04-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.069 比例 | 0.140（rail-air-cabin-c00-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -13.604 dB | -4.340（rail-air-cabin-c02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 10 / 0 | 11.101 dB² | 24.011（rail-air-cabin-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 21.500 个/分钟 | 45.000（rail-air-cabin-c02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.039 Δ | 0.020（rail-air-cabin-c00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 8 / 2 / 0 | 4.293 dB | 10.817（rail-air-cabin-c09-mba） | 6.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.968 dB | -10.351（rail-air-cabin-c06-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 9 / 1 | -4.985 dB | -2.369（rail-air-cabin-c09-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.387 dB | 6.108（rail-air-cabin-c06-mba） | 10.000 | 过 |
| V-MUSIC |  | 4 / 6 / 0 | 2.391 × | 4.253（rail-air-cabin-c08-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `pa-chime`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `trolley`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `announce`（speech）：差 +2.931（有 3 段 6.978 · 没有 7 段 4.047）
- V-ONSET · `pa-chime`（event）：差 +2.931（有 3 段 6.978 · 没有 7 段 4.047）
- V-ONSET · `trolley`（event）：差 +0.403（有 3 段 5.209 · 没有 7 段 4.805）
- V-ONSET · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `announce`（speech）：差 +0.028（有 3 段 0.089 · 没有 7 段 0.060）
- V-SIM-GATE · `pa-chime`（event）：差 +0.028（有 3 段 0.089 · 没有 7 段 0.060）
- V-SIM-GATE · `trolley`（event）：差 +0.007（有 3 段 0.074 · 没有 7 段 0.067）
- V-SIM-GATE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `trolley`（event）：差 +8.107（有 3 段 16.775 · 没有 7 段 8.669）
- V-PUMPV · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `announce`（speech）：差 +0.824（有 3 段 4.870 · 没有 7 段 4.046）
- V-TIMBRE · `pa-chime`（event）：差 +0.824（有 3 段 4.870 · 没有 7 段 4.046）
- V-TIMBRE · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `trolley`（event）：差 +0.100（有 3 段 -4.918 · 没有 6 段 -5.018）
- V-INTF · `bed-cabin`：样本不够（有 9 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `cough`：样本不够（有 2 段 · 没有 7 段，每组要 ≥ 3）
- V-MUSIC · `announce`（speech）：差 +0.220（有 3 段 2.545 · 没有 7 段 2.325）
- V-MUSIC · `pa-chime`（event）：差 +0.220（有 3 段 2.545 · 没有 7 段 2.325）
- V-MUSIC · `bed-cabin`：样本不够（有 10 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `cough`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### rail-air-cabin · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -2.261 dB | -2.818（rail-air-cabin-c01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### rail-air-cabin · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（rail-air-cabin-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -2.532 dB | -5.530（rail-air-cabin-s02-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 5.432 dB | 8.105（rail-air-cabin-s03-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.133 比例 | 0.404（rail-air-cabin-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.050 dB | -6.034（rail-air-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 19.180 dB² | 31.030（rail-air-cabin-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 30.833 个/分钟 | 65.000（rail-air-cabin-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.003（rail-air-cabin-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 6.838 dB | 17.085（rail-air-cabin-s01-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.471 dB | -7.391（rail-air-cabin-s00-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -4.855 dB | -3.085（rail-air-cabin-s01-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.009 dB | 4.358（rail-air-cabin-s01-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.127 × | 1.879（rail-air-cabin-s01-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### rail-air-cabin · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（rail-air-cabin-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -2.852 dB | -6.013（rail-air-cabin-s02-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 5.800 dB | 7.691（rail-air-cabin-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.190 比例 | 0.420（rail-air-cabin-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -14.638 dB | -5.345（rail-air-cabin-s00-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 20.917 dB² | 33.437（rail-air-cabin-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 24.167 个/分钟 | 35.000（rail-air-cabin-s03-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.013 Δ | 0.002（rail-air-cabin-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 6.367 dB | 16.245（rail-air-cabin-s01-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.967 dB | -8.319（rail-air-cabin-s04-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -4.892 dB | -1.526（rail-air-cabin-s01-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.554 dB | 6.004（rail-air-cabin-s00-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.053 × | 1.394（rail-air-cabin-s01-ext） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### rail-air-cabin · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 2 / 4 / 0 | -3.319 dB | -4.139（rail-air-cabin-s03-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### rail-air-cabin · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（rail-air-cabin-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -3.087 dB | -7.790（rail-air-cabin-s02-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.050 dB | 7.066（rail-air-cabin-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.188 比例 | 0.689（rail-air-cabin-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.941 dB | -9.725（rail-air-cabin-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 18.526 dB² | 31.079（rail-air-cabin-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 28.333 个/分钟 | 70.000（rail-air-cabin-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.013 Δ | -0.003（rail-air-cabin-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 3 / 3 / 0 | 6.250 dB | 14.821（rail-air-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.759 dB | -8.651（rail-air-cabin-s00-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -4.703 dB | -3.455（rail-air-cabin-s01-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.470 dB | 4.643（rail-air-cabin-s01-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.315 × | 1.866（rail-air-cabin-s01-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### rail-air-cabin · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（rail-air-cabin-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 3 / 3 / 0 | -3.153 dB | -8.362（rail-air-cabin-s02-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 4.944 dB | 7.515（rail-air-cabin-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.276 比例 | 0.913（rail-air-cabin-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -15.356 dB | -8.226（rail-air-cabin-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 20.354 dB² | 38.938（rail-air-cabin-s01-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 30.833 个/分钟 | 55.000（rail-air-cabin-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.014 Δ | 0.001（rail-air-cabin-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 4 / 2 / 0 | 5.835 dB | 13.961（rail-air-cabin-s01-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.444 dB | -7.718（rail-air-cabin-s04-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 4 / 2 | -4.738 dB | -1.573（rail-air-cabin-s01-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 1.590 dB | 5.998（rail-air-cabin-s00-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.450 × | 1.949（rail-air-cabin-s03-mba） | 2.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### rail-air-cabin · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（rail-air-cabin-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（rail-air-cabin-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 0 / 6 / 0 | -3.349 dB | -4.009（rail-air-cabin-s03-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `announce`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SPLVL · `bed-cabin`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
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

### street-outdoor · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（street-outdoor-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.473 dB | -1.354（street-outdoor-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 2.134 dB | 7.343（street-outdoor-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.038 比例 | 0.090（street-outdoor-c07-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.263 dB | -0.980（street-outdoor-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 3.293 dB² | 10.713（street-outdoor-c07-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 29.000 个/分钟 | 60.000（street-outdoor-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.064 Δ | 0.041（street-outdoor-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.430 dB | 0.930（street-outdoor-c05-ext） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -9.247 dB | -5.557（street-outdoor-c03-ext） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | -5.157 dB | -2.455（street-outdoor-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 4.988 dB | 8.204（street-outdoor-c00-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 2.094 × | 8.046（street-outdoor-c00-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +2.246（有 3 段 3.706 · 没有 7 段 1.460）
- V-ONSET · `car-horn`（event）：差 +1.408（有 4 段 2.979 · 没有 6 段 1.570）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +0.746（有 3 段 -8.725 · 没有 7 段 -9.471）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（street-outdoor-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.136 dB | -0.465（street-outdoor-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 1.839 dB | 4.742（street-outdoor-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.036 比例 | 0.073（street-outdoor-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -5.339 dB | -1.833（street-outdoor-c00-ext） | 4.000 | 过 |
| V-PUMPV |  | 2 / 8 / 0 | 3.253 dB² | 10.731（street-outdoor-c07-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 9 / 0 | 21.000 个/分钟 | 50.000（street-outdoor-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.073 Δ | 0.053（street-outdoor-c03-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.385 dB | 1.162（street-outdoor-c03-ext） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.145 dB | -8.580（street-outdoor-c03-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 3 / 7 | -6.303 dB | -2.426（street-outdoor-c02-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.531 dB | 6.665（street-outdoor-c00-ext） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 3.278 × | 14.139（street-outdoor-c00-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `footsteps`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `talker`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.446（有 3 段 2.851 · 没有 7 段 1.405）
- V-ONSET · `car-horn`（event）：差 +0.832（有 4 段 2.338 · 没有 6 段 1.506）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.004（有 3 段 0.038 · 没有 7 段 0.035）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`（event）：差 +0.851（有 3 段 3.873 · 没有 7 段 3.022）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.591 dB | -2.419（street-outdoor-c03-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### street-outdoor · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（street-outdoor-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.762 dB | -1.851（street-outdoor-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 2.530 dB | 5.011（street-outdoor-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.045 比例 | 0.110（street-outdoor-c07-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.316 dB | -3.903（street-outdoor-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 5.185 dB² | 10.055（street-outdoor-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 38.500 个/分钟 | 60.000（street-outdoor-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.065 Δ | 0.027（street-outdoor-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.737 dB | 2.238（street-outdoor-c07-mba） | 6.000 | 过 |
| V-NR |  | 9 / 1 / 0 | -9.123 dB | -3.869（street-outdoor-c03-mba） | -6.000 | 不过 |
| V-INTF |  | 0 / 3 / 7 | -5.127 dB | -2.977（street-outdoor-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.081 dB | 8.222（street-outdoor-c00-mba） | 10.000 | 过 |
| V-MUSIC |  | 6 / 4 / 0 | 1.610 × | 3.308（street-outdoor-c07-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.578（有 3 段 2.935 · 没有 7 段 2.357）
- V-ONSET · `car-horn`（event）：差 +0.555（有 4 段 2.863 · 没有 6 段 2.308）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.013（有 3 段 0.055 · 没有 7 段 0.041）
- V-SIM-GATE · `car-horn`（event）：差 +0.007（有 4 段 0.050 · 没有 6 段 0.042）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.101（有 3 段 5.255 · 没有 7 段 5.155）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `car-horn`（event）：差 +0.102（有 4 段 -9.062 · 没有 6 段 -9.164）
- V-NR · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（street-outdoor-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.293 dB | -0.975（street-outdoor-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 4 / 0 | 2.682 dB | 5.692（street-outdoor-c03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 10 / 0 | 0.048 比例 | 0.149（street-outdoor-c07-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -9.286 dB | -4.374（street-outdoor-c00-mba） | 4.000 | 过 |
| V-PUMPV |  | 1 / 9 / 0 | 5.752 dB² | 13.331（street-outdoor-c07-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 10 / 0 | 31.500 个/分钟 | 45.000（street-outdoor-c06-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.072 Δ | 0.037（street-outdoor-c03-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.728 dB | 2.011（street-outdoor-c07-mba） | 6.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.066 dB | -6.454（street-outdoor-c03-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 3 / 7 | -6.562 dB | -3.661（street-outdoor-c02-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 10 / 0 / 0 | 5.928 dB | 9.492（street-outdoor-c00-mba） | 10.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 1.808 × | 4.869（street-outdoor-c00-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `footsteps`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `talker`（speech）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.002（有 3 段 0.050 · 没有 7 段 0.048）
- V-SIM-GATE · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 3 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 3 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`（event）：差 +0.168（有 3 段 1.925 · 没有 7 段 1.757）
- V-MUSIC · `bed-street`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### street-outdoor · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -1.276 dB | -2.846（street-outdoor-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### street-outdoor · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（street-outdoor-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.689 dB | -1.286（street-outdoor-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 5.126 dB | 8.799（street-outdoor-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.055 比例 | 0.073（street-outdoor-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -7.844 dB | -4.145（street-outdoor-s05-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 8.062 dB² | 17.311（street-outdoor-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 37.500 个/分钟 | 60.000（street-outdoor-s00-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.062 Δ | 0.033（street-outdoor-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.567 dB | 9.162（street-outdoor-s02-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.301 dB | -7.521（street-outdoor-s01-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 1 / 5 | -5.514 dB | -5.514（street-outdoor-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 4.182 dB | 6.384（street-outdoor-s03-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.364 × | 2.876（street-outdoor-s04-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-street`（bed）：差 +0.985（有 3 段 5.618 · 没有 3 段 4.633）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.008（有 3 段 0.059 · 没有 3 段 0.051）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`（bed）：差 +3.360（有 3 段 9.742 · 没有 3 段 6.382）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +2.300（有 3 段 4.718 · 没有 3 段 2.417）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`（bed）：差 +0.746（有 3 段 1.737 · 没有 3 段 0.991）
- V-MUSIC · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### street-outdoor · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（street-outdoor-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.349 dB | -0.712（street-outdoor-s04-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 4.899 dB | 6.631（street-outdoor-s04-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.059 比例 | 0.086（street-outdoor-s03-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -7.570 dB | -3.746（street-outdoor-s05-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 9.197 dB² | 21.722（street-outdoor-s01-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 27.500 个/分钟 | 45.000（street-outdoor-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.071 Δ | 0.040（street-outdoor-s01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.318 dB | 9.001（street-outdoor-s02-ext） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -10.267 dB | -9.134（street-outdoor-s01-ext） | -6.000 | 过 |
| V-INTF |  | 0 / 1 / 5 | -6.025 dB | -6.025（street-outdoor-s00-ext） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 5.093 dB | 7.281（street-outdoor-s04-ext） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.554 × | 3.695（street-outdoor-s04-ext） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-terrace`（bed）：差 +0.251（有 3 段 5.025 · 没有 3 段 4.774）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.004（有 3 段 0.061 · 没有 3 段 0.057）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-terrace`（bed）：差 +5.157（有 3 段 11.775 · 没有 3 段 6.618）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +2.863（有 3 段 4.750 · 没有 3 段 1.887）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`（bed）：差 +0.947（有 3 段 2.028 · 没有 3 段 1.081）
- V-MUSIC · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### street-outdoor · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 5 / 1 / 0 | -1.944 dB | -3.190（street-outdoor-s04-ext） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-terrace`（bed）：差 +0.666（有 3 段 -2.277 · 没有 3 段 -1.611）
- V-SPLVL · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
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

### street-outdoor · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（street-outdoor-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -1.094 dB | -1.384（street-outdoor-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 6 / 0 | 5.197 dB | 9.350（street-outdoor-s03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.075 比例 | 0.125（street-outdoor-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -10.169 dB | -7.079（street-outdoor-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 20.925 dB² | 57.351（street-outdoor-s03-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 55.000 个/分钟 | 80.000（street-outdoor-s05-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.049 Δ | 0.023（street-outdoor-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.216 dB | 10.348（street-outdoor-s02-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.225 dB | -8.160（street-outdoor-s03-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 1 / 5 | -6.172 dB | -6.172（street-outdoor-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 3.111 dB | 6.032（street-outdoor-s05-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.498 × | 3.133（street-outdoor-s04-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-street`（bed）：差 +0.651（有 3 段 5.523 · 没有 3 段 4.871）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.032（有 3 段 0.091 · 没有 3 段 0.059）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`（bed）：差 +12.883（有 3 段 27.367 · 没有 3 段 14.483）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +4.534（有 3 段 5.483 · 没有 3 段 0.949）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`（bed）：差 +0.861（有 3 段 1.929 · 没有 3 段 1.068）
- V-MUSIC · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### street-outdoor · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（street-outdoor-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.618 dB | -0.915（street-outdoor-s00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 0 / 6 / 0 | 5.863 dB | 8.668（street-outdoor-s03-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.097 比例 | 0.121（street-outdoor-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -9.889 dB | -6.887（street-outdoor-s02-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 14.251 dB² | 21.646（street-outdoor-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 41.667 个/分钟 | 85.000（street-outdoor-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.056 Δ | 0.034（street-outdoor-s01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 3.954 dB | 10.161（street-outdoor-s02-mba） | 6.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.385 dB | -8.394（street-outdoor-s01-mba） | -6.000 | 过 |
| V-INTF |  | 0 / 1 / 5 | -6.621 dB | -6.621（street-outdoor-s00-mba） | -10.000 | 不过 |
| V-BGSTAB |  | 6 / 0 / 0 | 4.708 dB | 6.917（street-outdoor-s04-mba） | 10.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.374 × | 3.273（street-outdoor-s04-mba） | 2.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-LAT · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-street`（bed）：差 +0.649（有 3 段 6.187 · 没有 3 段 5.538）
- V-ONSET · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-ONSET · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-street`（bed）：差 +0.001（有 3 段 0.098 · 没有 3 段 0.097）
- V-SIM-GATE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-street`（bed）：差 +0.247（有 3 段 14.375 · 没有 3 段 14.128）
- V-PUMPV · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-TIMBRE · `bed-street`（bed）：差 +3.145（有 3 段 5.527 · 没有 3 段 2.382）
- V-TIMBRE · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-TIMBRE · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-INTF · `bed-street`：样本不够（有 1 段 · 没有 0 段，每组要 ≥ 3）
- V-INTF · `bed-terrace`：样本不够（有 0 段 · 没有 1 段，每组要 ≥ 3）
- V-MUSIC · `bed-terrace`（bed）：差 +0.830（有 3 段 1.789 · 没有 3 段 0.959）
- V-MUSIC · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-MUSIC · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### street-outdoor · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（street-outdoor-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（street-outdoor-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 4 / 2 / 0 | -2.543 dB | -3.231（street-outdoor-s04-mba） | -3.000 | 不过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
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
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-SPLVL · `bed-terrace`（bed）：差 +0.184（有 3 段 -2.635 · 没有 3 段 -2.451）
- V-SPLVL · `car-horn`：样本不够（有 6 段 · 没有 0 段，每组要 ≥ 3）
- V-SPLVL · `footsteps`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
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

### hotel-room · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（hotel-room-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.306 dB | -0.494（hotel-room-c00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 10 / 0 / 0 | 0.883 dB | 2.457（hotel-room-c08-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.033 比例 | 0.109（hotel-room-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.167 dB | 1.759（hotel-room-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.039 dB² | 3.458（hotel-room-c08-ext） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 9.000 个/分钟 | 35.000（hotel-room-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.010 Δ | -0.006（hotel-room-c04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.219 dB | 0.551（hotel-room-c07-ext） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.359 × | 3.773（hotel-room-c03-ext） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -8.109 dB | -5.158（hotel-room-c06-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.049（有 3 段 0.067 · 没有 7 段 0.019）
- V-SIM-GATE · `footsteps`（event）：差 +0.029（有 5 段 0.048 · 没有 5 段 0.019）
- V-SIM-GATE · `bed-room`（bed）：差 +0.002（有 6 段 0.034 · 没有 4 段 0.032）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `footsteps`（event）：差 +0.750（有 5 段 1.414 · 没有 5 段 0.664）
- V-PUMPV · `door`（event）：差 +0.698（有 3 段 1.527 · 没有 7 段 0.829）
- V-PUMPV · `bed-ace`（bed）：差 +0.160（有 4 段 1.135 · 没有 6 段 0.975）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +0.511（有 6 段 1.563 · 没有 4 段 1.052）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-ace`（bed）：差 +1.751（有 4 段 -7.059 · 没有 6 段 -8.810）
- V-NR · `footsteps`（event）：差 +0.777（有 5 段 -7.721 · 没有 5 段 -8.498）
- V-NR · `door-knock`（event）：差 +0.166（有 4 段 -8.010 · 没有 6 段 -8.176）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（hotel-room-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.085 dB | -0.136（hotel-room-c08-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 3 / 0 | 1.565 dB | 4.452（hotel-room-c06-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.051 比例 | 0.119（hotel-room-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.354 dB | 0.784（hotel-room-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 2.057 dB² | 6.285（hotel-room-c05-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 4.500 个/分钟 | 20.000（hotel-room-c07-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.020 Δ | 0.003（hotel-room-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.152 dB | 0.225（hotel-room-c05-ext） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.323 × | 3.720（hotel-room-c03-ext） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.020 dB | -7.511（hotel-room-c02-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `door`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `door`（event）：差 +3.374（有 3 段 3.927 · 没有 7 段 0.553）
- V-ONSET · `footsteps`（event）：差 +0.829（有 5 段 1.980 · 没有 5 段 1.151）
- V-ONSET · `bed-ace`（bed）：差 +0.100（有 4 段 1.625 · 没有 6 段 1.525）
- V-ONSET · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.078（有 3 段 0.106 · 没有 7 段 0.028）
- V-SIM-GATE · `footsteps`（event）：差 +0.016（有 5 段 0.059 · 没有 5 段 0.043）
- V-SIM-GATE · `bed-room`（bed）：差 +0.003（有 6 段 0.052 · 没有 4 段 0.049）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +4.926（有 3 段 5.505 · 没有 7 段 0.579）
- V-PUMPV · `footsteps`（event）：差 +0.844（有 5 段 2.479 · 没有 5 段 1.635）
- V-PUMPV · `bed-ace`（bed）：差 +0.341（有 4 段 2.261 · 没有 6 段 1.921）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +0.392（有 6 段 1.480 · 没有 4 段 1.088）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（hotel-room-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.183 dB | -0.436（hotel-room-c07-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 10 / 0 / 0 | 0.992 dB | 2.291（hotel-room-c05-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.030 比例 | 0.089（hotel-room-c06-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.449 dB | 1.940（hotel-room-c03-ext） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.229 dB² | 2.712（hotel-room-c04-ext） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 7.500 个/分钟 | 30.000（hotel-room-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.007 Δ | 0.001（hotel-room-c01-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.133 dB | 0.386（hotel-room-c09-ext） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 1.214 × | 1.786（hotel-room-c03-ext） | 2.000 | 过 |
| V-NR |  | 7 / 3 / 0 | -6.303 dB | -3.378（hotel-room-c09-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### hotel-room · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（hotel-room-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.420 dB | -1.220（hotel-room-c00-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 10 / 0 / 0 | 1.196 dB | 2.457（hotel-room-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 1 / 9 / 0 | 0.041 比例 | 0.102（hotel-room-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.022 dB | 1.602（hotel-room-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 7 / 3 / 0 | 1.376 dB² | 3.903（hotel-room-c04-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 6.000 个/分钟 | 10.000（hotel-room-c03-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.021 Δ | 0.006（hotel-room-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.306 dB | 0.702（hotel-room-c07-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.550 × | 3.269（hotel-room-c03-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.706 dB | -4.179（hotel-room-c06-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.032（有 3 段 0.063 · 没有 7 段 0.031）
- V-SIM-GATE · `footsteps`（event）：差 +0.019（有 5 段 0.050 · 没有 5 段 0.031）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.005（有 4 段 0.043 · 没有 6 段 0.039）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`（bed）：差 +0.995（有 4 段 1.973 · 没有 6 段 0.978）
- V-PUMPV · `footsteps`（event）：差 +0.986（有 5 段 1.869 · 没有 5 段 0.883）
- V-PUMPV · `door`（event）：差 +0.473（有 3 段 1.707 · 没有 7 段 1.234）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +1.020（有 6 段 1.959 · 没有 4 段 0.938）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `footsteps`（event）：差 +2.010（有 5 段 -7.700 · 没有 5 段 -9.711）
- V-NR · `bed-ace`（bed）：差 +1.874（有 4 段 -7.581 · 没有 6 段 -9.455）
- V-NR · `door`（event）：差 +1.234（有 3 段 -7.841 · 没有 7 段 -9.076）
- V-NR · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（hotel-room-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.095 dB | -0.192（hotel-room-c06-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 2 / 0 | 1.519 dB | 4.171（hotel-room-c05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.056 比例 | 0.144（hotel-room-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.537 dB | 0.585（hotel-room-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 2.173 dB² | 7.043（hotel-room-c05-mba） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 5.000 个/分钟 | 15.000（hotel-room-c07-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.033 Δ | 0.007（hotel-room-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.165 dB | 0.414（hotel-room-c06-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.641 × | 3.996（hotel-room-c03-mba） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -9.893 dB | -7.643（hotel-room-c02-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `door`（event）：差 +0.000（有 3 段 87.667 · 没有 7 段 87.667）
- V-LAT · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `door`（event）：差 +2.302（有 3 段 3.131 · 没有 7 段 0.829）
- V-ONSET · `bed-room`（bed）：差 +0.233（有 6 段 1.613 · 没有 4 段 1.379）
- V-ONSET · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `door`（event）：差 +0.068（有 3 段 0.103 · 没有 7 段 0.035）
- V-SIM-GATE · `footsteps`（event）：差 +0.023（有 5 段 0.067 · 没有 5 段 0.044）
- V-SIM-GATE · `bed-ace`（bed）：差 +0.005（有 4 段 0.059 · 没有 6 段 0.054）
- V-SIM-GATE · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `door`（event）：差 +3.706（有 3 段 4.767 · 没有 7 段 1.061）
- V-PUMPV · `bed-ace`（bed）：差 +0.624（有 4 段 2.547 · 没有 6 段 1.924）
- V-PUMPV · `footsteps`（event）：差 +0.449（有 5 段 2.398 · 没有 5 段 1.949）
- V-PUMPV · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`（bed）：差 +0.758（有 6 段 1.944 · 没有 4 段 1.186）
- V-MUSIC · `hvac-cycle`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）

### hotel-room · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（hotel-room-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（hotel-room-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.164 dB | -0.430（hotel-room-c07-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 10 / 0 / 0 | 1.078 dB | 2.671（hotel-room-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 2 / 8 / 0 | 0.037 比例 | 0.085（hotel-room-c08-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.142 dB | 1.574（hotel-room-c03-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.596 dB² | 5.170（hotel-room-c04-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 8.000 个/分钟 | 20.000（hotel-room-c07-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.014 Δ | 0.004（hotel-room-c01-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.183 dB | 0.531（hotel-room-c09-mba） | 6.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 1.745 × | 3.423（hotel-room-c03-mba） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -7.660 dB | -1.998（hotel-room-c09-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### hotel-room · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（hotel-room-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.466 dB | -1.287（hotel-room-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 3.137 dB | 4.201（hotel-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.055 比例 | 0.109（hotel-room-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.976 dB | -0.647（hotel-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 3.493 dB² | 6.015（hotel-room-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 19.167 个/分钟 | 30.000（hotel-room-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.049 Δ | 0.011（hotel-room-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.714 dB | 5.723（hotel-room-s02-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.837 × | 5.410（hotel-room-s03-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -8.599 dB | -6.317（hotel-room-s05-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `footsteps`（event）：差 +0.359（有 3 段 3.316 · 没有 3 段 2.957）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.022（有 3 段 0.066 · 没有 3 段 0.044）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（hotel-room-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.183 dB | -0.372（hotel-room-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.539 dB | 7.084（hotel-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.061 比例 | 0.098（hotel-room-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -3.123 dB | -0.801（hotel-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 4.583 dB² | 13.257（hotel-room-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 7.500 个/分钟 | 15.000（hotel-room-s05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.062 Δ | 0.034（hotel-room-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.458 dB | 5.072（hotel-room-s02-ext） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 2.416 × | 8.909（hotel-room-s03-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.718 dB | -6.908（hotel-room-s05-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.006（有 3 段 0.065 · 没有 3 段 0.058）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（hotel-room-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.551 dB | -0.984（hotel-room-s05-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 2.063 dB | 4.735（hotel-room-s00-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.028 比例 | 0.064（hotel-room-s02-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.891 dB | -0.205（hotel-room-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 2.898 dB² | 7.077（hotel-room-s00-ext） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 13.333 个/分钟 | 30.000（hotel-room-s05-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.018（hotel-room-s02-ext） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.738 dB | 3.881（hotel-room-s01-ext） | 6.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.099 × | 1.557（hotel-room-s03-ext） | 2.000 | 过 |
| V-NR |  | 2 / 4 / 0 | -5.743 dB | -5.031（hotel-room-s00-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### hotel-room · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（hotel-room-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.663 dB | -1.301（hotel-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 2 / 4 / 0 | 3.568 dB | 5.160（hotel-room-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.065 比例 | 0.098（hotel-room-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.137 dB | -0.623（hotel-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.414 dB² | 19.550（hotel-room-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 25.833 个/分钟 | 55.000（hotel-room-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.063 Δ | 0.035（hotel-room-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.256 dB | 4.275（hotel-room-s02-mba） | 6.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.361 × | 1.755（hotel-room-s02-mba） | 2.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -9.344 dB | -7.107（hotel-room-s05-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `footsteps`（event）：差 +0.644（有 3 段 3.890 · 没有 3 段 3.246）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.006（有 3 段 0.068 · 没有 3 段 0.062）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（hotel-room-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.371 dB | -1.197（hotel-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 3.477 dB | 6.163（hotel-room-s00-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 0 / 6 / 0 | 0.063 比例 | 0.108（hotel-room-s00-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.178 dB | -0.414（hotel-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 6.878 dB² | 27.916（hotel-room-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 17.500 个/分钟 | 30.000（hotel-room-s00-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.076 Δ | 0.058（hotel-room-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.184 dB | 3.890（hotel-room-s02-mba） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.597 × | 3.833（hotel-room-s03-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -9.494 dB | -6.873（hotel-room-s05-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-ONSET · `footsteps`（event）：差 +0.506（有 3 段 3.730 · 没有 3 段 3.224）
- V-ONSET · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-SIM-GATE · `footsteps`（event）：差 +0.003（有 3 段 0.064 · 没有 3 段 0.062）
- V-SIM-GATE · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 4 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 4 段，每组要 ≥ 3）

### hotel-room · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（hotel-room-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（hotel-room-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.908 dB | -2.673（hotel-room-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 2.603 dB | 4.666（hotel-room-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.048 比例 | 0.089（hotel-room-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -4.024 dB | 0.255（hotel-room-s01-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 11.432 dB² | 54.621（hotel-room-s00-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 22.500 个/分钟 | 45.000（hotel-room-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.039 Δ | 0.032（hotel-room-s02-mba） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.181 dB | 6.383（hotel-room-s01-mba） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.182 × | 1.611（hotel-room-s03-mba） | 2.000 | 过 |
| V-NR |  | 4 / 2 / 0 | -7.804 dB | -4.466（hotel-room-s00-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

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

### library-classroom · common · external-mic · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（library-classroom-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.352 dB | -1.125（library-classroom-c01-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 0 / 2 | 0.852 dB | 2.744（library-classroom-c08-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.037 比例 | 0.104（library-classroom-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.583 dB | 2.340（library-classroom-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.199 dB² | 3.454（library-classroom-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 10.000 个/分钟 | 25.000（library-classroom-c08-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.009 Δ | -0.017（library-classroom-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 9 / 1 / 0 | 1.619 dB | 9.487（library-classroom-c07-ext） | 6.000 | 不过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.201 × | 2.400（library-classroom-c05-ext） | 2.000 | 不过 |
| V-NR |  | 6 / 4 / 0 | -7.127 dB | -4.213（library-classroom-c07-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `page-turn`（event）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-SIM-GATE · `page-turn`（event）：差 +0.006（有 7 段 0.039 · 没有 3 段 0.033）
- V-SIM-GATE · `talker`（speech）：差 +0.005（有 5 段 0.040 · 没有 5 段 0.035）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.188（有 5 段 1.293 · 没有 5 段 1.105）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-TIMBRE · `page-turn`（event）：差 +1.991（有 7 段 2.216 · 没有 3 段 0.225）
- V-TIMBRE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `page-turn`（event）：差 +0.034（有 7 段 -7.117 · 没有 3 段 -7.151）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · external-mic · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（library-classroom-c00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.096 dB | -0.251（library-classroom-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 2 / 2 | 1.579 dB | 5.451（library-classroom-c08-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 7 / 0 | 0.057 比例 | 0.145（library-classroom-c08-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.648 dB | 1.972（library-classroom-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.457 dB² | 4.041（library-classroom-c02-ext） | 1.000 | 不过 |
| V-CLICK |  | 6 / 4 / 0 | 6.000 个/分钟 | 25.000（library-classroom-c02-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.018 Δ | -0.010（library-classroom-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.656 dB | 2.502（library-classroom-c07-ext） | 6.000 | 过 |
| V-MUSIC |  | 10 / 0 / 0 | 0.977 × | 1.128（library-classroom-c00-ext） | 2.000 | 过 |
| V-NR |  | 10 / 0 / 0 | -10.246 dB | -7.675（library-classroom-c07-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +1.213（有 4 段 2.185 · 没有 4 段 0.972）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.024（有 5 段 0.069 · 没有 5 段 0.045）
- V-SIM-GATE · `page-turn`（event）：差 +0.015（有 7 段 0.062 · 没有 3 段 0.046）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +0.863（有 5 段 1.888 · 没有 5 段 1.025）
- V-PUMPV · `page-turn`（event）：差 +0.808（有 7 段 1.699 · 没有 3 段 0.891）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · external-mic · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-ext） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（library-classroom-c00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.241 dB | -0.424（library-classroom-c02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 0 / 2 | 0.634 dB | 2.239（library-classroom-c06-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 6 / 4 / 0 | 0.035 比例 | 0.085（library-classroom-c03-ext） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | 0.050 dB | 2.835（library-classroom-c09-ext） | 4.000 | 过 |
| V-PUMPV |  | 6 / 4 / 0 | 1.516 dB² | 3.923（library-classroom-c03-ext） | 1.000 | 不过 |
| V-CLICK |  | 5 / 5 / 0 | 10.500 个/分钟 | 35.000（library-classroom-c01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.003 Δ | -0.017（library-classroom-c09-ext） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.945 dB | 3.243（library-classroom-c08-ext） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.408 × | 2.694（library-classroom-c09-ext） | 2.000 | 不过 |
| V-NR |  | 8 / 2 / 0 | -6.740 dB | -3.525（library-classroom-c02-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

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

### library-classroom · common · mba-builtin · dfn3-ll — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 45.417 ms | 45.417（library-classroom-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.312 dB | -0.866（library-classroom-c03-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 7 / 1 / 2 | 1.075 dB | 3.534（library-classroom-c06-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.042 比例 | 0.116（library-classroom-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -1.076 dB | 2.116（library-classroom-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.421 dB² | 4.978（library-classroom-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 3 / 7 / 0 | 14.000 个/分钟 | 50.000（library-classroom-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.027 Δ | -0.002（library-classroom-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.668 dB | 2.469（library-classroom-c08-mba） | 6.000 | 过 |
| V-MUSIC |  | 7 / 3 / 0 | 2.342 × | 10.253（library-classroom-c09-mba） | 2.000 | 不过 |
| V-NR |  | 7 / 3 / 0 | -7.430 dB | -3.881（library-classroom-c07-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `page-turn`（event）：差 +0.000（有 7 段 45.417 · 没有 3 段 45.417）
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.136（有 4 段 1.143 · 没有 4 段 1.008）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.022（有 5 段 0.053 · 没有 5 段 0.031）
- V-SIM-GATE · `page-turn`（event）：差 +0.022（有 7 段 0.048 · 没有 3 段 0.027）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.019（有 5 段 1.931 · 没有 5 段 0.912）
- V-PUMPV · `page-turn`（event）：差 +0.803（有 7 段 1.662 · 没有 3 段 0.859）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-NR · `page-turn`（event）：差 +0.901（有 7 段 -7.160 · 没有 3 段 -8.061）
- V-NR · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · mba-builtin · dpdfnet2 — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 10 / 0 | 87.667 ms | 87.667（library-classroom-c00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.098 dB | -0.228（library-classroom-c02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 2 / 2 | 1.531 dB | 5.123（library-classroom-c08-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 4 / 6 / 0 | 0.046 比例 | 0.103（library-classroom-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -1.194 dB | 1.544（library-classroom-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 5 / 5 / 0 | 1.561 dB² | 5.911（library-classroom-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 4 / 6 / 0 | 6.500 个/分钟 | 25.000（library-classroom-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.039 Δ | 0.004（library-classroom-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.228 dB | 0.559（library-classroom-c08-mba） | 6.000 | 过 |
| V-MUSIC |  | 9 / 1 / 0 | 1.979 × | 10.634（library-classroom-c05-mba） | 2.000 | 不过 |
| V-NR |  | 10 / 0 / 0 | -10.007 dB | -7.446（library-classroom-c02-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-ONSET · `talker`（speech）：差 +0.773（有 4 段 1.918 · 没有 4 段 1.145）
- V-ONSET · `bed-ace`：样本不够（有 6 段 · 没有 2 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 2 段 · 没有 6 段，每组要 ≥ 3）
- V-SIM-GATE · `talker`（speech）：差 +0.017（有 5 段 0.055 · 没有 5 段 0.037）
- V-SIM-GATE · `page-turn`（event）：差 +0.010（有 7 段 0.049 · 没有 3 段 0.039）
- V-SIM-GATE · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-PUMPV · `talker`（speech）：差 +1.536（有 5 段 2.330 · 没有 5 段 0.793）
- V-PUMPV · `page-turn`（event）：差 +1.382（有 7 段 1.976 · 没有 3 段 0.594）
- V-PUMPV · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）
- V-MUSIC · `bed-ace`：样本不够（有 8 段 · 没有 2 段，每组要 ≥ 3）
- V-MUSIC · `bed-room`：样本不够（有 2 段 · 没有 8 段，每组要 ≥ 3）

### library-classroom · common · mba-builtin · gtcrn — 不过（10 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 10 / 0 / 0 | 0.000 个 | 0.000（library-classroom-c00-mba） | 0.000 | 过 |
| V-LAT | 是 | 10 / 0 / 0 | 43.667 ms | 43.667（library-classroom-c00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 10 / 0 / 0 | -0.262 dB | -0.518（library-classroom-c04-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 10 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 8 / 0 / 2 | 0.834 dB | 2.480（library-classroom-c08-mba） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 5 / 0 | 0.034 比例 | 0.103（library-classroom-c03-mba） | 0.050 | 不过 |
| V-WEAK |  | 10 / 0 / 0 | -0.564 dB | 2.624（library-classroom-c09-mba） | 4.000 | 过 |
| V-PUMPV |  | 4 / 6 / 0 | 1.778 dB² | 5.930（library-classroom-c03-mba） | 1.000 | 不过 |
| V-CLICK |  | 2 / 8 / 0 | 10.000 个/分钟 | 30.000（library-classroom-c01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 10 / 0 / 0 | 0.016 Δ | -0.006（library-classroom-c09-mba） | -0.050 | 过 |
| V-TIMBRE |  | 10 / 0 / 0 | 0.243 dB | 0.730（library-classroom-c02-mba） | 6.000 | 过 |
| V-MUSIC |  | 8 / 2 / 0 | 1.404 × | 2.177（library-classroom-c05-mba） | 2.000 | 不过 |
| V-NR |  | 9 / 1 / 0 | -8.246 dB | -4.014（library-classroom-c02-mba） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 10 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 10 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 10 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
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

### library-classroom · stress · external-mic · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（library-classroom-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.170 dB | -0.320（library-classroom-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 2.601 dB | 4.907（library-classroom-s01-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.030 比例 | 0.064（library-classroom-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.761 dB | -0.171（library-classroom-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 2.463 dB² | 4.126（library-classroom-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 12.500 个/分钟 | 25.000（library-classroom-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.025 Δ | 0.007（library-classroom-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.211 dB | 9.738（library-classroom-s03-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.082 × | 1.475（library-classroom-s03-ext） | 2.000 | 过 |
| V-NR |  | 3 / 3 / 0 | -6.774 dB | -3.412（library-classroom-s04-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`（event）：差 +0.016（有 3 段 0.037 · 没有 3 段 0.022）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +4.087（有 3 段 4.255 · 没有 3 段 0.168）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-NR · `keyboard`（event）：差 +0.326（有 3 段 -6.611 · 没有 3 段 -6.937）
- V-NR · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-NR · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · external-mic · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（library-classroom-s00-ext） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.156 dB | -0.321（library-classroom-s02-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 1 / 5 / 0 | 3.726 dB | 5.422（library-classroom-s05-ext） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.060 比例 | 0.119（library-classroom-s05-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.385 dB | -0.021（library-classroom-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 4.245 dB² | 7.881（library-classroom-s05-ext） | 1.000 | 不过 |
| V-CLICK |  | 2 / 4 / 0 | 11.667 个/分钟 | 35.000（library-classroom-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.017（library-classroom-s00-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 1.924 dB | 8.287（library-classroom-s03-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 0.983 × | 1.059（library-classroom-s01-ext） | 2.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -9.906 dB | -7.949（library-classroom-s05-ext） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`（event）：差 +0.048（有 3 段 0.083 · 没有 3 段 0.036）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `keyboard`（event）：差 +1.155（有 3 段 4.822 · 没有 3 段 3.667）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-TIMBRE · `keyboard`（event）：差 +3.422（有 3 段 3.635 · 没有 3 段 0.213）
- V-TIMBRE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-TIMBRE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · external-mic · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-ext） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（library-classroom-s00-ext） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.546 dB | -0.964（library-classroom-s00-ext） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 6 / 0 / 0 | 1.594 dB | 2.685（library-classroom-s01-ext） | 3.000 | 过 |
| V-SIM-GATE |  | 5 / 1 / 0 | 0.030 比例 | 0.071（library-classroom-s04-ext） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -1.108 dB | 0.183（library-classroom-s01-ext） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 2.299 dB² | 4.316（library-classroom-s04-ext） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 11.667 个/分钟 | 20.000（library-classroom-s01-ext） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.016 Δ | 0.008（library-classroom-s04-ext） | -0.050 | 过 |
| V-TIMBRE |  | 5 / 1 / 0 | 2.311 dB | 9.804（library-classroom-s03-ext） | 6.000 | 不过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.243 × | 1.829（library-classroom-s03-ext） | 2.000 | 过 |
| V-NR |  | 3 / 3 / 0 | -6.150 dB | -3.707（library-classroom-s04-ext） | -6.000 | 不过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

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

### library-classroom · stress · mba-builtin · dfn3-ll — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 45.417 ms | 45.417（library-classroom-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.349 dB | -0.965（library-classroom-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 3 / 3 / 0 | 2.665 dB | 4.039（library-classroom-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.029 比例 | 0.043（library-classroom-s01-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -3.330 dB | -2.066（library-classroom-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 4.412 dB² | 12.990（library-classroom-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 24.167 个/分钟 | 50.000（library-classroom-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.057 Δ | 0.035（library-classroom-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.128 dB | 4.424（library-classroom-s03-mba） | 6.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 1.132 × | 1.905（library-classroom-s02-mba） | 2.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -8.756 dB | -6.261（library-classroom-s02-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `keyboard`（event）：差 +0.364（有 3 段 2.847 · 没有 3 段 2.483）
- V-ONSET · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · mba-builtin · dpdfnet2 — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 0 / 6 / 0 | 87.667 ms | 87.667（library-classroom-s00-mba） | 45.000 | 不过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.236 dB | -0.589（library-classroom-s02-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 4 / 2 / 0 | 3.059 dB | 5.551（library-classroom-s01-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 2 / 4 / 0 | 0.048 比例 | 0.067（library-classroom-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -3.015 dB | -1.612（library-classroom-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 5.964 dB² | 16.112（library-classroom-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 0 / 6 / 0 | 19.167 个/分钟 | 35.000（library-classroom-s01-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.064 Δ | 0.036（library-classroom-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 0.562 dB | 1.321（library-classroom-s03-mba） | 6.000 | 过 |
| V-MUSIC |  | 6 / 0 / 0 | 0.929 × | 1.018（library-classroom-s01-mba） | 2.000 | 过 |
| V-NR |  | 6 / 0 / 0 | -10.357 dB | -8.388（library-classroom-s05-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

拖后腿的声音（有这类声音的段比没有的段差多少）：
- V-LAT · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-LAT · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-ONSET · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-ONSET · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-SIM-GATE · `keyboard`（event）：差 +0.021（有 3 段 0.059 · 没有 3 段 0.037）
- V-SIM-GATE · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-SIM-GATE · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）
- V-PUMPV · `bed-ace`：样本不够（有 5 段 · 没有 1 段，每组要 ≥ 3）
- V-PUMPV · `bed-room`：样本不够（有 1 段 · 没有 5 段，每组要 ≥ 3）

### library-classroom · stress · mba-builtin · gtcrn — 不过（6 段）

| 项 | 否决 | 过 / 不过 / 待测（段） | 均值 | 最差（段） | 过线 | 判定 |
|---|---|---|---|---|---|---|
| V-NAN | 是 | 6 / 0 / 0 | 0.000 个 | 0.000（library-classroom-s00-mba） | 0.000 | 过 |
| V-LAT | 是 | 6 / 0 / 0 | 43.667 ms | 43.667（library-classroom-s00-mba） | 45.000 | 过 |
| V-SPLVL | 是 | 6 / 0 / 0 | -0.989 dB | -2.425（library-classroom-s01-mba） | -3.000 | 过 |
| V-EAT-ASR | 是 | 0 / 0 / 6 | — 百分点 | — | — | 待测 |
| V-ONSET |  | 5 / 1 / 0 | 2.135 dB | 4.006（library-classroom-s05-mba） | 3.000 | 不过 |
| V-SIM-GATE |  | 3 / 3 / 0 | 0.036 比例 | 0.053（library-classroom-s02-mba） | 0.050 | 不过 |
| V-WEAK |  | 6 / 0 / 0 | -2.867 dB | -1.301（library-classroom-s00-mba） | 4.000 | 过 |
| V-PUMPV |  | 0 / 6 / 0 | 3.729 dB² | 8.779（library-classroom-s04-mba） | 1.000 | 不过 |
| V-CLICK |  | 1 / 5 / 0 | 10.833 个/分钟 | 25.000（library-classroom-s02-mba） | 0.000 | 不过（只报不判） |
| V-ESTOI |  | 6 / 0 / 0 | 0.032 Δ | 0.020（library-classroom-s00-mba） | -0.050 | 过 |
| V-TIMBRE |  | 6 / 0 / 0 | 1.728 dB | 5.052（library-classroom-s03-mba） | 6.000 | 过 |
| V-MUSIC |  | 5 / 1 / 0 | 1.171 × | 2.036（library-classroom-s04-mba） | 2.000 | 不过 |
| V-NR |  | 6 / 0 / 0 | -8.311 dB | -6.784（library-classroom-s01-mba） | -6.000 | 过 |
| 听感（SCOREQ + Distill-MOS 都说不比原声差） | | 0 / 0 / 6 | SCOREQ Δ — · Distill-MOS Δ — | | Δ ≥ 0 | 待测 |

| 评分模型 | 过 / 不过 / 待测（段） | Δ 均值 | 最差（段） |
|---|---|---|---|
| V-NRMOS-SCOREQ | 0 / 0 / 6 | — | — |
| V-NRMOS-DISTILL | 0 / 0 / 6 | — | — |

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

