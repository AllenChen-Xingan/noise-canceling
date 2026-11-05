# exp016 配对分析（run `20261105-002946-e21f3a93-dirty`，由 analyze.py 生成，勿手改）

差值 = 候选相对 gtcrn，**正 = 候选更好**；区间 = 成对片段自助重抽样 95%（2000 次）。按 025 待拍板 G-1 默认推进，待人确认。

## dpdfnet2 vs gtcrn

### 每个评分项（全部 480 段）

| 评分项 | 方向 | 单位 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|---|
| V-BGSTAB | 越小越好 | dB | 329 | -1.44 | [-1.79, -1.11] | 候选差 |
| V-CLICK | 越小越好 | 个/分钟 | 480 | +0.27 | [-1.12, +1.62] | 分不出 |
| V-ESTOI | 越大越好 | Δ | 480 | +0.02 | [+0.02, +0.03] | 候选好 |
| V-INTF | 越小越好 | dB | 178 | +1.11 | [+0.75, +1.46] | 候选好 |
| V-MUSIC | 越小越好 | × | 288 | -0.17 | [-0.37, +0.02] | 分不出 |
| V-NR | 越小越好 | dB | 457 | +3.11 | [+2.88, +3.33] | 候选好 |
| V-ONSET | 越小越好 | dB | 432 | -1.45 | [-1.65, -1.25] | 候选差 |
| V-PUMPV | 越小越好 | dB² | 480 | -1.77 | [-2.28, -1.19] | 候选差 |
| V-SIM-FA | 越小越好 | 比例 | 79 | +0.25 | [+0.17, +0.34] | 候选好 |
| V-SIM-GATE | 越小越好 | 比例 | 480 | -0.04 | [-0.05, -0.03] | 候选差 |
| V-SPLVL | 越大越好 | dB | 465 | +0.75 | [+0.58, +0.91] | 候选好 |
| V-TIMBRE | 越小越好 | dB | 462 | +0.52 | [+0.35, +0.71] | 候选好 |
| V-WEAK | 越小越好 | dB | 480 | -0.24 | [-0.34, -0.14] | 候选差 |

### 场景 × 评分项：判断不是「分不出」的格子

| 场景 | 评分项 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|
| bus-metro-cabin | V-BGSTAB | 32 | -1.66 dB | [-2.63, -0.69] | 候选差 |
| bus-metro-cabin | V-ESTOI | 32 | +0.02 Δ | [+0.01, +0.02] | 候选好 |
| bus-metro-cabin | V-INTF | 19 | +2.97 dB | [+1.84, +4.07] | 候选好 |
| bus-metro-cabin | V-NR | 32 | +3.53 dB | [+2.31, +4.83] | 候选好 |
| bus-metro-cabin | V-ONSET | 29 | -1.14 dB | [-1.79, -0.48] | 候选差 |
| bus-metro-cabin | V-PUMPV | 32 | -2.62 dB² | [-4.42, -0.52] | 候选差 |
| bus-metro-cabin | V-SIM-GATE | 32 | -0.07 比例 | [-0.17, -0.01] | 候选差 |
| bus-metro-cabin | V-TIMBRE | 30 | +1.37 dB | [+0.61, +2.26] | 候选好 |
| bus-metro-cabin | V-WEAK | 32 | -0.91 dB | [-1.55, -0.42] | 候选差 |
| cafe | V-BGSTAB | 32 | -1.27 dB | [-1.94, -0.65] | 候选差 |
| cafe | V-ESTOI | 32 | +0.04 Δ | [+0.03, +0.04] | 候选好 |
| cafe | V-NR | 32 | +3.64 dB | [+2.81, +4.40] | 候选好 |
| cafe | V-ONSET | 31 | -1.43 dB | [-2.17, -0.77] | 候选差 |
| cafe | V-PUMPV | 32 | -3.44 dB² | [-4.58, -2.44] | 候选差 |
| cafe | V-SIM-GATE | 32 | -0.05 比例 | [-0.07, -0.03] | 候选差 |
| cafe | V-SPLVL | 32 | +1.10 dB | [+0.78, +1.41] | 候选好 |
| cafe | V-TIMBRE | 32 | +0.98 dB | [+0.31, +1.77] | 候选好 |
| cafe | V-WEAK | 32 | -0.45 dB | [-0.62, -0.27] | 候选差 |
| canteen | V-BGSTAB | 32 | -3.81 dB | [-4.72, -2.84] | 候选差 |
| canteen | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| canteen | V-INTF | 14 | +2.04 dB | [+0.79, +3.45] | 候选好 |
| canteen | V-NR | 32 | +3.58 dB | [+2.73, +4.41] | 候选好 |
| canteen | V-ONSET | 29 | -3.13 dB | [-3.95, -2.33] | 候选差 |
| canteen | V-SIM-FA | 9 | +0.48 比例 | [+0.27, +0.66] | 候选好 |
| canteen | V-SIM-GATE | 32 | -0.06 比例 | [-0.10, -0.03] | 候选差 |
| canteen | V-SPLVL | 32 | +1.54 dB | [+0.88, +2.10] | 候选好 |
| canteen | V-WEAK | 32 | -0.67 dB | [-0.94, -0.44] | 候选差 |
| car-passenger | V-BGSTAB | 31 | -1.39 dB | [-2.28, -0.49] | 候选差 |
| car-passenger | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| car-passenger | V-NR | 32 | +2.30 dB | [+1.55, +3.09] | 候选好 |
| car-passenger | V-ONSET | 32 | -0.97 dB | [-1.55, -0.40] | 候选差 |
| car-passenger | V-SIM-GATE | 32 | -0.01 比例 | [-0.02, -0.00] | 候选差 |
| car-passenger | V-SPLVL | 32 | +1.35 dB | [+1.04, +1.67] | 候选好 |
| car-passenger | V-TIMBRE | 32 | +0.59 dB | [+0.20, +1.02] | 候选好 |
| cowork | V-BGSTAB | 32 | -1.54 dB | [-2.53, -0.57] | 候选差 |
| cowork | V-ESTOI | 32 | +0.04 Δ | [+0.03, +0.04] | 候选好 |
| cowork | V-INTF | 16 | +2.34 dB | [+1.05, +3.61] | 候选好 |
| cowork | V-NR | 32 | +3.33 dB | [+2.48, +4.17] | 候选好 |
| cowork | V-ONSET | 26 | -1.82 dB | [-2.52, -1.03] | 候选差 |
| cowork | V-SIM-FA | 14 | +0.39 比例 | [+0.14, +0.66] | 候选好 |
| cowork | V-SPLVL | 32 | +1.69 dB | [+1.07, +2.37] | 候选好 |
| cowork | V-TIMBRE | 32 | +0.44 dB | [+0.09, +0.79] | 候选好 |
| home-family | V-BGSTAB | 10 | -1.80 dB | [-2.99, -0.34] | 候选差 |
| home-family | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.04] | 候选好 |
| home-family | V-INTF | 28 | +1.05 dB | [+0.39, +1.83] | 候选好 |
| home-family | V-NR | 9 | +4.60 dB | [+4.14, +5.10] | 候选好 |
| home-family | V-ONSET | 32 | -0.82 dB | [-1.46, -0.19] | 候选差 |
| home-family | V-SIM-FA | 28 | +0.21 比例 | [+0.14, +0.29] | 候选好 |
| home-family | V-TIMBRE | 16 | +0.90 dB | [+0.18, +1.77] | 候选好 |
| home-family | V-WEAK | 32 | -0.40 dB | [-0.63, -0.16] | 候选差 |
| home-quiet | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.02] | 候选好 |
| home-quiet | V-NR | 32 | +2.27 dB | [+1.57, +2.99] | 候选好 |
| home-quiet | V-ONSET | 32 | -1.17 dB | [-1.73, -0.62] | 候选差 |
| home-quiet | V-PUMPV | 32 | -1.19 dB² | [-2.12, -0.28] | 候选差 |
| home-quiet | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| home-quiet | V-SPLVL | 32 | +0.21 dB | [+0.11, +0.35] | 候选好 |
| home-quiet | V-WEAK | 32 | +0.27 dB | [+0.08, +0.47] | 候选好 |
| hotel-room | V-CLICK | 32 | +3.91 个/分钟 | [+1.41, +6.72] | 候选好 |
| hotel-room | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| hotel-room | V-NR | 32 | +2.92 dB | [+2.06, +3.85] | 候选好 |
| hotel-room | V-ONSET | 32 | -0.76 dB | [-1.19, -0.33] | 候选差 |
| hotel-room | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| hotel-room | V-SPLVL | 32 | +0.22 dB | [+0.10, +0.38] | 候选好 |
| hotel-room | V-WEAK | 32 | +0.54 dB | [+0.35, +0.71] | 候选好 |
| library-classroom | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| library-classroom | V-NR | 32 | +2.73 dB | [+2.12, +3.31] | 候选好 |
| library-classroom | V-ONSET | 28 | -1.12 dB | [-1.71, -0.55] | 候选差 |
| library-classroom | V-PUMPV | 32 | -0.70 dB² | [-1.40, -0.03] | 候选差 |
| library-classroom | V-SIM-GATE | 32 | -0.02 比例 | [-0.04, -0.01] | 候选差 |
| library-classroom | V-SPLVL | 32 | +0.31 dB | [+0.15, +0.51] | 候选好 |
| library-classroom | V-TIMBRE | 32 | +0.39 dB | [+0.04, +0.85] | 候选好 |
| library-classroom | V-WEAK | 32 | +0.49 dB | [+0.26, +0.73] | 候选好 |
| meeting-room | V-NR | 32 | +3.95 dB | [+3.23, +4.66] | 候选好 |
| meeting-room | V-ONSET | 8 | -1.82 dB | [-3.22, -0.82] | 候选差 |
| meeting-room | V-PUMPV | 32 | -2.73 dB² | [-3.59, -1.98] | 候选差 |
| meeting-room | V-SIM-GATE | 32 | -0.06 比例 | [-0.08, -0.04] | 候选差 |
| meeting-room | V-WEAK | 32 | -0.77 dB | [-0.99, -0.56] | 候选差 |
| open-office | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| open-office | V-NR | 32 | +3.65 dB | [+2.70, +4.61] | 候选好 |
| open-office | V-ONSET | 25 | -1.11 dB | [-2.12, -0.30] | 候选差 |
| open-office | V-PUMPV | 32 | -2.82 dB² | [-4.68, -1.27] | 候选差 |
| open-office | V-SIM-FA | 16 | +0.17 比例 | [+0.04, +0.33] | 候选好 |
| open-office | V-SIM-GATE | 32 | -0.02 比例 | [-0.04, -0.01] | 候选差 |
| open-office | V-SPLVL | 32 | +0.72 dB | [+0.42, +1.02] | 候选好 |
| open-office | V-WEAK | 32 | -0.22 dB | [-0.44, -0.01] | 候选差 |
| phone-booth | V-NR | 32 | +2.35 dB | [+1.61, +3.19] | 候选好 |
| phone-booth | V-ONSET | 32 | -1.44 dB | [-2.42, -0.63] | 候选差 |
| phone-booth | V-PUMPV | 32 | -2.76 dB² | [-5.18, -0.62] | 候选差 |
| phone-booth | V-SIM-GATE | 32 | -0.09 比例 | [-0.18, -0.03] | 候选差 |
| rail-air-cabin | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| rail-air-cabin | V-MUSIC | 32 | -0.58 × | [-1.19, -0.01] | 候选差 |
| rail-air-cabin | V-NR | 32 | +3.58 dB | [+3.08, +4.04] | 候选好 |
| rail-air-cabin | V-ONSET | 32 | -2.51 dB | [-3.24, -1.73] | 候选差 |
| rail-air-cabin | V-PUMPV | 32 | -3.58 dB² | [-4.43, -2.71] | 候选差 |
| rail-air-cabin | V-SIM-GATE | 32 | -0.08 比例 | [-0.16, -0.03] | 候选差 |
| rail-air-cabin | V-SPLVL | 32 | +1.01 dB | [+0.10, +1.64] | 候选好 |
| rail-air-cabin | V-WEAK | 32 | -0.49 dB | [-0.99, -0.19] | 候选差 |
| street-outdoor | V-BGSTAB | 32 | -1.67 dB | [-2.63, -0.80] | 候选差 |
| street-outdoor | V-ESTOI | 32 | +0.03 Δ | [+0.03, +0.04] | 候选好 |
| street-outdoor | V-INTF | 8 | +1.40 dB | [+0.62, +2.48] | 候选好 |
| street-outdoor | V-NR | 32 | +3.04 dB | [+2.44, +3.61] | 候选好 |
| street-outdoor | V-ONSET | 32 | -1.41 dB | [-2.15, -0.70] | 候选差 |
| street-outdoor | V-PUMPV | 32 | -2.49 dB² | [-4.00, -1.20] | 候选差 |
| street-outdoor | V-SIM-GATE | 32 | -0.02 比例 | [-0.04, -0.01] | 候选差 |
| street-outdoor | V-SPLVL | 32 | +1.11 dB | [+0.70, +1.47] | 候选好 |
| street-outdoor | V-TIMBRE | 32 | +0.99 dB | [+0.25, +1.82] | 候选好 |
| street-outdoor | V-WEAK | 32 | -0.22 dB | [-0.35, -0.10] | 候选差 |
| transit-hub | V-BGSTAB | 32 | -2.21 dB | [-3.44, -0.95] | 候选差 |
| transit-hub | V-ESTOI | 32 | +0.03 Δ | [+0.03, +0.04] | 候选好 |
| transit-hub | V-INTF | 14 | +1.53 dB | [+0.88, +2.13] | 候选好 |
| transit-hub | V-NR | 32 | +2.21 dB | [+1.69, +2.80] | 候选好 |
| transit-hub | V-ONSET | 32 | -1.46 dB | [-2.03, -0.84] | 候选差 |
| transit-hub | V-PUMPV | 32 | -1.57 dB² | [-3.02, -0.00] | 候选差 |
| transit-hub | V-SIM-GATE | 32 | -0.03 比例 | [-0.06, -0.00] | 候选差 |
| transit-hub | V-SPLVL | 32 | +1.40 dB | [+0.99, +1.79] | 候选好 |
| transit-hub | V-TIMBRE | 32 | +0.86 dB | [+0.28, +1.43] | 候选好 |
| transit-hub | V-WEAK | 32 | -0.44 dB | [-0.64, -0.25] | 候选差 |

### GTCRN 落后最多的前 5 处（场景 × 声音 × 评分项，配对单位 ≥ 5，按标准化效应排；未做多重比较校正）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| transit-hub | bed-ace | V-ESTOI | 10（5） | +0.05 Δ | [+0.04, +0.05] | 2.75 |
| bus-metro-cabin | bed-bus | V-NR | 10（5） | +5.18 dB | [+3.15, +7.22] | 2.46 |
| cafe | music | V-ESTOI | 18（9） | +0.04 Δ | [+0.04, +0.05] | 2.37 |
| street-outdoor | wind | V-ESTOI | 10（5） | +0.04 Δ | [+0.04, +0.05] | 2.33 |
| cowork | talker | V-ESTOI | 20（10） | +0.04 Δ | [+0.03, +0.05] | 2.27 |

### 反过来：候选落后 GTCRN 最多的前 5 处（同口径）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| canteen | dishes | V-ONSET | 26（13） | -3.31 dB | [-4.08, -2.66] | -1.97 |
| canteen | bed-cafeteria | V-ONSET | 12（6） | -3.28 dB | [-3.85, -2.68] | -1.95 |
| canteen | babble | V-ONSET | 24（12） | -3.10 dB | [-4.10, -2.22] | -1.84 |
| canteen | talker | V-ONSET | 21（11） | -3.03 dB | [-4.12, -2.06] | -1.80 |
| canteen | bed-restaurant | V-ONSET | 17（9） | -3.03 dB | [-4.41, -1.90] | -1.80 |

## dfn3-ll vs gtcrn

### 每个评分项（全部 480 段）

| 评分项 | 方向 | 单位 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|---|
| V-BGSTAB | 越小越好 | dB | 329 | -0.76 | [-1.03, -0.48] | 候选差 |
| V-CLICK | 越小越好 | 个/分钟 | 480 | -6.15 | [-7.66, -4.72] | 候选差 |
| V-ESTOI | 越大越好 | Δ | 480 | +0.01 | [+0.01, +0.02] | 候选好 |
| V-INTF | 越小越好 | dB | 178 | +0.66 | [+0.34, +1.00] | 候选好 |
| V-MUSIC | 越小越好 | × | 288 | -0.14 | [-0.30, +0.03] | 分不出 |
| V-NR | 越小越好 | dB | 457 | +2.16 | [+1.93, +2.41] | 候选好 |
| V-ONSET | 越小越好 | dB | 432 | -0.93 | [-1.09, -0.76] | 候选差 |
| V-PUMPV | 越小越好 | dB² | 480 | -0.68 | [-1.26, +0.03] | 分不出 |
| V-SIM-FA | 越小越好 | 比例 | 79 | +0.21 | [+0.14, +0.29] | 候选好 |
| V-SIM-GATE | 越小越好 | 比例 | 480 | -0.01 | [-0.02, -0.00] | 候选差 |
| V-SPLVL | 越大越好 | dB | 465 | +0.38 | [+0.26, +0.49] | 候选好 |
| V-TIMBRE | 越小越好 | dB | 462 | +0.32 | [+0.13, +0.52] | 候选好 |
| V-WEAK | 越小越好 | dB | 480 | +0.00 | [-0.06, +0.06] | 分不出 |

### 场景 × 评分项：判断不是「分不出」的格子

| 场景 | 评分项 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|
| bus-metro-cabin | V-BGSTAB | 32 | -1.11 dB | [-1.87, -0.38] | 候选差 |
| bus-metro-cabin | V-CLICK | 32 | -3.59 个/分钟 | [-5.94, -1.25] | 候选差 |
| bus-metro-cabin | V-ESTOI | 32 | +0.00 Δ | [+0.00, +0.01] | 候选好 |
| bus-metro-cabin | V-INTF | 19 | +1.08 dB | [+0.49, +1.76] | 候选好 |
| bus-metro-cabin | V-NR | 32 | +3.35 dB | [+2.33, +4.41] | 候选好 |
| bus-metro-cabin | V-ONSET | 29 | -1.14 dB | [-1.64, -0.69] | 候选差 |
| bus-metro-cabin | V-PUMPV | 32 | -1.66 dB² | [-2.82, -0.67] | 候选差 |
| bus-metro-cabin | V-WEAK | 32 | -0.28 dB | [-0.59, -0.06] | 候选差 |
| cafe | V-CLICK | 32 | -9.38 个/分钟 | [-15.31, -3.59] | 候选差 |
| cafe | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| cafe | V-NR | 32 | +3.09 dB | [+2.32, +3.79] | 候选好 |
| cafe | V-ONSET | 31 | -1.41 dB | [-1.93, -0.97] | 候选差 |
| cafe | V-PUMPV | 32 | -2.36 dB² | [-3.30, -1.49] | 候选差 |
| cafe | V-SIM-GATE | 32 | -0.03 比例 | [-0.04, -0.02] | 候选差 |
| cafe | V-SPLVL | 32 | +0.79 dB | [+0.55, +1.07] | 候选好 |
| cafe | V-TIMBRE | 32 | +0.74 dB | [+0.17, +1.45] | 候选好 |
| cafe | V-WEAK | 32 | -0.14 dB | [-0.24, -0.05] | 候选差 |
| canteen | V-CLICK | 32 | -10.62 个/分钟 | [-17.19, -4.53] | 候选差 |
| canteen | V-ESTOI | 32 | +0.02 Δ | [+0.01, +0.02] | 候选好 |
| canteen | V-NR | 32 | +3.00 dB | [+2.24, +3.71] | 候选好 |
| canteen | V-ONSET | 29 | -1.66 dB | [-2.06, -1.28] | 候选差 |
| canteen | V-SPLVL | 32 | +1.02 dB | [+0.58, +1.50] | 候选好 |
| car-passenger | V-CLICK | 32 | -12.50 个/分钟 | [-19.84, -6.09] | 候选差 |
| car-passenger | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| car-passenger | V-INTF | 16 | -0.92 dB | [-1.80, -0.18] | 候选差 |
| car-passenger | V-NR | 32 | +1.71 dB | [+1.09, +2.46] | 候选好 |
| car-passenger | V-ONSET | 32 | -0.79 dB | [-1.36, -0.25] | 候选差 |
| car-passenger | V-SPLVL | 32 | +1.02 dB | [+0.74, +1.31] | 候选好 |
| car-passenger | V-WEAK | 32 | +0.31 dB | [+0.22, +0.41] | 候选好 |
| cowork | V-BGSTAB | 32 | -0.85 dB | [-1.64, -0.13] | 候选差 |
| cowork | V-CLICK | 32 | -6.88 个/分钟 | [-12.19, -1.72] | 候选差 |
| cowork | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| cowork | V-NR | 32 | +2.78 dB | [+2.16, +3.38] | 候选好 |
| cowork | V-ONSET | 26 | -1.40 dB | [-2.12, -0.62] | 候选差 |
| cowork | V-SIM-FA | 14 | +0.16 比例 | [+0.01, +0.35] | 候选好 |
| cowork | V-SPLVL | 32 | +1.27 dB | [+0.76, +1.84] | 候选好 |
| home-family | V-BGSTAB | 10 | -1.91 dB | [-3.25, -0.22] | 候选差 |
| home-family | V-ESTOI | 32 | +0.02 Δ | [+0.01, +0.02] | 候选好 |
| home-family | V-INTF | 28 | +2.64 dB | [+1.66, +3.77] | 候选好 |
| home-family | V-NR | 9 | +3.85 dB | [+3.26, +4.29] | 候选好 |
| home-family | V-ONSET | 32 | -0.76 dB | [-1.34, -0.20] | 候选差 |
| home-family | V-SIM-FA | 28 | +0.35 比例 | [+0.22, +0.49] | 候选好 |
| home-family | V-WEAK | 32 | -0.41 dB | [-0.59, -0.22] | 候选差 |
| home-quiet | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| home-quiet | V-NR | 32 | +1.19 dB | [+0.26, +2.10] | 候选好 |
| home-quiet | V-WEAK | 32 | +0.31 dB | [+0.05, +0.59] | 候选好 |
| hotel-room | V-ESTOI | 32 | +0.01 Δ | [+0.00, +0.02] | 候选好 |
| hotel-room | V-NR | 32 | +1.72 dB | [+0.71, +2.67] | 候选好 |
| hotel-room | V-SIM-GATE | 32 | -0.01 比例 | [-0.02, -0.00] | 候选差 |
| hotel-room | V-WEAK | 32 | +0.16 dB | [+0.00, +0.32] | 候选好 |
| library-classroom | V-CLICK | 32 | -3.75 个/分钟 | [-8.28, -0.16] | 候选差 |
| library-classroom | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| library-classroom | V-ONSET | 28 | -0.46 dB | [-0.79, -0.17] | 候选差 |
| library-classroom | V-WEAK | 32 | +0.57 dB | [+0.32, +0.81] | 候选好 |
| meeting-room | V-NR | 32 | +2.19 dB | [+1.07, +3.28] | 候选好 |
| meeting-room | V-PUMPV | 32 | -0.71 dB² | [-1.08, -0.36] | 候选差 |
| meeting-room | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| open-office | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| open-office | V-NR | 32 | +2.75 dB | [+2.02, +3.48] | 候选好 |
| open-office | V-ONSET | 25 | -0.60 dB | [-1.04, -0.18] | 候选差 |
| open-office | V-PUMPV | 32 | -2.49 dB² | [-4.62, -0.72] | 候选差 |
| open-office | V-SIM-FA | 16 | +0.13 比例 | [+0.04, +0.23] | 候选好 |
| open-office | V-SIM-GATE | 32 | -0.01 比例 | [-0.02, -0.00] | 候选差 |
| phone-booth | V-CLICK | 32 | -2.81 个/分钟 | [-5.31, -0.62] | 候选差 |
| phone-booth | V-NR | 32 | +0.81 dB | [+0.13, +1.65] | 候选好 |
| phone-booth | V-SIM-GATE | 32 | -0.01 比例 | [-0.03, -0.00] | 候选差 |
| phone-booth | V-SPLVL | 32 | -0.53 dB | [-0.87, -0.20] | 候选差 |
| rail-air-cabin | V-ESTOI | 32 | +0.01 Δ | [+0.00, +0.01] | 候选好 |
| rail-air-cabin | V-MUSIC | 32 | -0.55 × | [-0.98, -0.15] | 候选差 |
| rail-air-cabin | V-NR | 32 | +2.98 dB | [+2.29, +3.67] | 候选好 |
| rail-air-cabin | V-ONSET | 32 | -1.77 dB | [-2.42, -1.16] | 候选差 |
| rail-air-cabin | V-PUMPV | 32 | -1.83 dB² | [-3.32, -0.40] | 候选差 |
| rail-air-cabin | V-SIM-GATE | 32 | -0.05 比例 | [-0.11, -0.00] | 候选差 |
| rail-air-cabin | V-TIMBRE | 32 | -0.87 dB | [-1.75, -0.10] | 候选差 |
| street-outdoor | V-BGSTAB | 32 | -0.76 dB | [-1.55, -0.06] | 候选差 |
| street-outdoor | V-CLICK | 32 | -10.62 个/分钟 | [-14.53, -6.56] | 候选差 |
| street-outdoor | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| street-outdoor | V-NR | 32 | +2.26 dB | [+1.46, +3.01] | 候选好 |
| street-outdoor | V-ONSET | 32 | -1.38 dB | [-2.07, -0.70] | 候选差 |
| street-outdoor | V-PUMPV | 32 | -3.37 dB² | [-6.67, -1.01] | 候选差 |
| street-outdoor | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| street-outdoor | V-SPLVL | 32 | +0.70 dB | [+0.33, +1.05] | 候选好 |
| street-outdoor | V-TIMBRE | 32 | +1.07 dB | [+0.23, +2.05] | 候选好 |
| street-outdoor | V-WEAK | 32 | -0.13 dB | [-0.21, -0.05] | 候选差 |
| transit-hub | V-BGSTAB | 32 | -1.03 dB | [-1.76, -0.33] | 候选差 |
| transit-hub | V-CLICK | 32 | -17.34 个/分钟 | [-24.84, -9.69] | 候选差 |
| transit-hub | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| transit-hub | V-NR | 32 | +1.92 dB | [+1.54, +2.30] | 候选好 |
| transit-hub | V-ONSET | 32 | -1.35 dB | [-1.86, -0.76] | 候选差 |
| transit-hub | V-SPLVL | 32 | +0.80 dB | [+0.28, +1.25] | 候选好 |
| transit-hub | V-TIMBRE | 32 | +1.07 dB | [+0.56, +1.63] | 候选好 |

### GTCRN 落后最多的前 5 处（场景 × 声音 × 评分项，配对单位 ≥ 5，按标准化效应排；未做多重比较校正）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| street-outdoor | wind | V-ESTOI | 10（5） | +0.03 Δ | [+0.03, +0.04] | 2.34 |
| street-outdoor | footsteps | V-ESTOI | 10（5） | +0.03 Δ | [+0.03, +0.03] | 2.31 |
| transit-hub | bed-ace | V-ESTOI | 10（5） | +0.03 Δ | [+0.02, +0.04] | 2.18 |
| street-outdoor | car-horn | V-ESTOI | 20（10） | +0.03 Δ | [+0.02, +0.04] | 2.07 |
| cafe | music | V-ESTOI | 18（9） | +0.03 Δ | [+0.02, +0.04] | 1.99 |

### 反过来：候选落后 GTCRN 最多的前 5 处（同口径）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| transit-hub | announce | V-CLICK | 18（9） | -25.00 个/分钟 | [-32.22, -17.78] | -1.64 |
| transit-hub | pa-chime | V-CLICK | 18（9） | -25.00 个/分钟 | [-32.22, -17.50] | -1.64 |
| rail-air-cabin | announce | V-ONSET | 14（7） | -2.13 dB | [-3.18, -1.27] | -1.58 |
| rail-air-cabin | pa-chime | V-ONSET | 14（7） | -2.13 dB | [-3.27, -1.27] | -1.58 |
| transit-hub | footsteps | V-ONSET | 14（7） | -2.09 dB | [-2.58, -1.55] | -1.55 |

