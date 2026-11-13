# exp016 配对分析（run `20261105-005916-4ccf0638+20261105-103000-4cf04ac1`，由 analyze.py 生成，勿手改）

差值 = 候选相对 gtcrn，**正 = 候选更好**；区间 = 成对片段自助重抽样 95%（2000 次）。按 025 待拍板 G-1 默认推进，待人确认。

## 锚点：两次 run 的 passthrough 逐段对照（已过：判的项逐段差 ≤ 1e-06）

| 评分项 | 判不判 | 段数 | 最大绝对差 | 平均差（后 − 前） |
|---|---|---|---|---|
| V-BGSTAB | 判 | 329 | 0 | +0.0000 |
| V-CLICK | 判 | 480 | 0 | +0.0000 |
| V-EAT-ASR | 只报 | 480 | 0 | +0.0000 |
| V-ESTOI | 判 | 480 | 0 | +0.0000 |
| V-INTF | 判 | 178 | 0 | +0.0000 |
| V-LAT | 判 | 480 | 0 | +0.0000 |
| V-LEAK | 判 | 100 | 0 | +0.0000 |
| V-MUSIC | 判 | 288 | 0 | +0.0000 |
| V-NAN | 判 | 480 | 0 | +0.0000 |
| V-NR | 判 | 457 | 0 | +0.0000 |
| V-NRMOS-DISTILL | 判 | 480 | 0 | +0.0000 |
| V-NRMOS-SCOREQ | 判 | 480 | 0 | +0.0000 |
| V-ONSET | 判 | 432 | 0 | +0.0000 |
| V-PSE-OVL | 判 | 242 | 0 | +0.0000 |
| V-PUMPV | 判 | 480 | 0 | +0.0000 |
| V-SIM-FA | 判 | 79 | 0 | +0.0000 |
| V-SIM-GATE | 判 | 480 | 0 | +0.0000 |
| V-SPLVL | 判 | 465 | 0 | +0.0000 |
| V-SWALLOW | 判 | 432 | 0 | +0.0000 |
| V-TIMBRE | 判 | 480 | 0 | +0.0000 |
| V-TSOS | 判 | 480 | 0 | +0.0000 |
| V-VACUUM | 判 | 457 | 0 | +0.0000 |
| V-WEAK | 判 | 480 | 0 | +0.0000 |

只报的项（V-EAT-ASR）在直通上量不出跨 run 识别差，照不照常比看 `asr_drift.py` 的实测结果（见 findings）。

## dfn3-ll vs gtcrn

### 每个评分项（全部 480 段）

| 评分项 | 方向 | 单位 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|---|
| V-BGSTAB | 越小越好 | dB | 329 | -0.74 | [-1.02, -0.47] | 候选差 |
| V-CLICK | 越小越好 | 个/分钟 | 480 | -6.15 | [-7.66, -4.72] | 候选差 |
| V-EAT-ASR | 越小越好 | 百分点 | 480 | +0.13 | [-0.57, +0.77] | 分不出 |
| V-ESTOI | 越大越好 | Δ | 480 | +0.01 | [+0.01, +0.02] | 候选好 |
| V-INTF | 越小越好 | dB | 178 | +0.66 | [+0.34, +0.99] | 候选好 |
| V-LEAK | 越小越好 | % | 100 | +2.89 | [-3.23, +8.86] | 分不出 |
| V-MUSIC | 越小越好 | × | 288 | -0.14 | [-0.29, +0.03] | 分不出 |
| V-NR | 越小越好 | dB | 457 | +2.16 | [+1.93, +2.38] | 候选好 |
| V-NRMOS-DISTILL | 越大越好 | Δ MOS | 480 | +0.06 | [+0.05, +0.07] | 候选好 |
| V-NRMOS-SCOREQ | 越大越好 | Δ MOS | 480 | +0.05 | [+0.04, +0.06] | 候选好 |
| V-ONSET | 越小越好 | dB | 432 | -0.93 | [-1.10, -0.76] | 候选差 |
| V-PSE-OVL | 越大越好 | dB | 242 | +0.70 | [+0.54, +0.87] | 候选好 |
| V-PUMPV | 越小越好 | dB² | 480 | +0.44 | [+0.27, +0.62] | 候选好 |
| V-SIM-FA | 越小越好 | 比例 | 79 | +0.21 | [+0.13, +0.29] | 候选好 |
| V-SIM-GATE | 越小越好 | 比例 | 480 | -0.01 | [-0.02, -0.00] | 候选差 |
| V-SPLVL | 越大越好 | dB | 465 | +0.38 | [+0.26, +0.49] | 候选好 |
| V-SWALLOW | 越小越好 | ms | 432 | +1.52 | [-1.53, +4.53] | 分不出 |
| V-TIMBRE | 越小越好 | dB | 480 | -0.13 | [-0.18, -0.07] | 候选差 |
| V-TSOS | 越小越好 | % | 480 | +0.60 | [-0.14, +1.49] | 分不出 |
| V-VACUUM | 越大越好 | dB | 457 | -2.16 | [-2.41, -1.92] | 候选差 |
| V-WEAK | 越小越好 | dB | 480 | +0.27 | [+0.17, +0.36] | 候选好 |

### 场景 × 评分项：判断不是「分不出」的格子

| 场景 | 评分项 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|
| bus-metro-cabin | V-BGSTAB | 32 | -1.09 dB | [-1.84, -0.35] | 候选差 |
| bus-metro-cabin | V-CLICK | 32 | -3.59 个/分钟 | [-5.94, -1.25] | 候选差 |
| bus-metro-cabin | V-ESTOI | 32 | +0.00 Δ | [+0.00, +0.01] | 候选好 |
| bus-metro-cabin | V-INTF | 19 | +1.08 dB | [+0.51, +1.78] | 候选好 |
| bus-metro-cabin | V-NR | 32 | +3.35 dB | [+2.39, +4.42] | 候选好 |
| bus-metro-cabin | V-ONSET | 29 | -1.14 dB | [-1.62, -0.70] | 候选差 |
| bus-metro-cabin | V-PSE-OVL | 28 | +0.80 dB | [+0.39, +1.22] | 候选好 |
| bus-metro-cabin | V-TIMBRE | 32 | -0.59 dB | [-0.77, -0.43] | 候选差 |
| bus-metro-cabin | V-TSOS | 32 | -1.94 % | [-3.90, -0.60] | 候选差 |
| bus-metro-cabin | V-VACUUM | 32 | -3.35 dB | [-4.36, -2.34] | 候选差 |
| bus-metro-cabin | V-WEAK | 32 | -0.84 dB | [-1.17, -0.48] | 候选差 |
| cafe | V-CLICK | 32 | -9.38 个/分钟 | [-15.00, -3.44] | 候选差 |
| cafe | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| cafe | V-NR | 32 | +3.09 dB | [+2.34, +3.83] | 候选好 |
| cafe | V-NRMOS-DISTILL | 32 | +0.10 Δ MOS | [+0.08, +0.12] | 候选好 |
| cafe | V-NRMOS-SCOREQ | 32 | +0.13 Δ MOS | [+0.09, +0.17] | 候选好 |
| cafe | V-ONSET | 31 | -1.41 dB | [-1.94, -0.95] | 候选差 |
| cafe | V-PSE-OVL | 24 | +0.56 dB | [+0.06, +0.99] | 候选好 |
| cafe | V-PUMPV | 32 | +1.16 dB² | [+0.20, +2.24] | 候选好 |
| cafe | V-SIM-GATE | 32 | -0.03 比例 | [-0.04, -0.01] | 候选差 |
| cafe | V-SPLVL | 32 | +0.79 dB | [+0.53, +1.07] | 候选好 |
| cafe | V-VACUUM | 32 | -3.09 dB | [-3.79, -2.35] | 候选差 |
| cafe | V-WEAK | 32 | +0.46 dB | [+0.14, +0.77] | 候选好 |
| canteen | V-CLICK | 32 | -10.62 个/分钟 | [-17.50, -4.53] | 候选差 |
| canteen | V-ESTOI | 32 | +0.02 Δ | [+0.01, +0.02] | 候选好 |
| canteen | V-NR | 32 | +3.00 dB | [+2.23, +3.75] | 候选好 |
| canteen | V-NRMOS-DISTILL | 32 | +0.12 Δ MOS | [+0.09, +0.14] | 候选好 |
| canteen | V-NRMOS-SCOREQ | 32 | +0.04 Δ MOS | [+0.01, +0.07] | 候选好 |
| canteen | V-ONSET | 29 | -1.66 dB | [-2.07, -1.28] | 候选差 |
| canteen | V-SPLVL | 32 | +1.02 dB | [+0.56, +1.47] | 候选好 |
| canteen | V-VACUUM | 32 | -3.00 dB | [-3.73, -2.25] | 候选差 |
| canteen | V-WEAK | 32 | +0.42 dB | [+0.18, +0.68] | 候选好 |
| car-passenger | V-CLICK | 32 | -12.50 个/分钟 | [-20.16, -6.25] | 候选差 |
| car-passenger | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| car-passenger | V-INTF | 16 | -0.92 dB | [-1.88, -0.17] | 候选差 |
| car-passenger | V-NR | 32 | +1.71 dB | [+1.08, +2.46] | 候选好 |
| car-passenger | V-NRMOS-DISTILL | 32 | +0.07 Δ MOS | [+0.05, +0.10] | 候选好 |
| car-passenger | V-NRMOS-SCOREQ | 32 | +0.08 Δ MOS | [+0.05, +0.10] | 候选好 |
| car-passenger | V-ONSET | 32 | -0.79 dB | [-1.32, -0.26] | 候选差 |
| car-passenger | V-PSE-OVL | 26 | +0.65 dB | [+0.28, +1.02] | 候选好 |
| car-passenger | V-PUMPV | 32 | +0.60 dB² | [+0.24, +1.06] | 候选好 |
| car-passenger | V-SPLVL | 32 | +1.02 dB | [+0.72, +1.33] | 候选好 |
| car-passenger | V-TSOS | 32 | +2.01 % | [+0.57, +3.60] | 候选好 |
| car-passenger | V-VACUUM | 32 | -1.71 dB | [-2.49, -1.08] | 候选差 |
| car-passenger | V-WEAK | 32 | +0.58 dB | [+0.42, +0.74] | 候选好 |
| cowork | V-BGSTAB | 32 | -0.80 dB | [-1.53, -0.13] | 候选差 |
| cowork | V-CLICK | 32 | -6.88 个/分钟 | [-12.19, -1.88] | 候选差 |
| cowork | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| cowork | V-NR | 32 | +2.78 dB | [+2.18, +3.41] | 候选好 |
| cowork | V-NRMOS-DISTILL | 32 | +0.10 Δ MOS | [+0.08, +0.13] | 候选好 |
| cowork | V-NRMOS-SCOREQ | 32 | +0.08 Δ MOS | [+0.04, +0.13] | 候选好 |
| cowork | V-ONSET | 26 | -1.40 dB | [-2.12, -0.61] | 候选差 |
| cowork | V-PSE-OVL | 20 | +1.72 dB | [+1.10, +2.44] | 候选好 |
| cowork | V-SIM-FA | 14 | +0.16 比例 | [+0.01, +0.35] | 候选好 |
| cowork | V-SPLVL | 32 | +1.27 dB | [+0.72, +1.88] | 候选好 |
| cowork | V-VACUUM | 32 | -2.78 dB | [-3.41, -2.17] | 候选差 |
| cowork | V-WEAK | 32 | +0.61 dB | [+0.24, +1.11] | 候选好 |
| home-family | V-BGSTAB | 10 | -1.88 dB | [-3.23, -0.19] | 候选差 |
| home-family | V-ESTOI | 32 | +0.02 Δ | [+0.01, +0.02] | 候选好 |
| home-family | V-INTF | 28 | +2.64 dB | [+1.62, +3.71] | 候选好 |
| home-family | V-NR | 9 | +3.85 dB | [+3.26, +4.31] | 候选好 |
| home-family | V-NRMOS-DISTILL | 32 | +0.05 Δ MOS | [+0.02, +0.08] | 候选好 |
| home-family | V-ONSET | 32 | -0.76 dB | [-1.36, -0.20] | 候选差 |
| home-family | V-PSE-OVL | 28 | +1.10 dB | [+0.67, +1.49] | 候选好 |
| home-family | V-PUMPV | 32 | +0.35 dB² | [+0.02, +0.75] | 候选好 |
| home-family | V-SIM-FA | 28 | +0.35 比例 | [+0.22, +0.49] | 候选好 |
| home-family | V-TIMBRE | 32 | -0.09 dB | [-0.18, -0.01] | 候选差 |
| home-family | V-VACUUM | 9 | -3.85 dB | [-4.31, -3.26] | 候选差 |
| home-family | V-WEAK | 32 | +0.49 dB | [+0.19, +0.73] | 候选好 |
| home-quiet | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| home-quiet | V-NR | 32 | +1.19 dB | [+0.29, +2.06] | 候选好 |
| home-quiet | V-NRMOS-DISTILL | 32 | +0.03 Δ MOS | [+0.01, +0.06] | 候选好 |
| home-quiet | V-NRMOS-SCOREQ | 32 | +0.04 Δ MOS | [+0.02, +0.07] | 候选好 |
| home-quiet | V-PUMPV | 32 | +0.84 dB² | [+0.28, +1.50] | 候选好 |
| home-quiet | V-SWALLOW | 32 | +19.75 ms | [+3.05, +40.40] | 候选好 |
| home-quiet | V-TIMBRE | 32 | -0.12 dB | [-0.24, -0.02] | 候选差 |
| home-quiet | V-TSOS | 32 | +2.63 % | [+0.96, +4.48] | 候选好 |
| home-quiet | V-VACUUM | 32 | -1.19 dB | [-2.09, -0.30] | 候选差 |
| home-quiet | V-WEAK | 32 | +0.49 dB | [+0.14, +0.84] | 候选好 |
| hotel-room | V-ESTOI | 32 | +0.01 Δ | [+0.00, +0.02] | 候选好 |
| hotel-room | V-NR | 32 | +1.72 dB | [+0.75, +2.76] | 候选好 |
| hotel-room | V-NRMOS-DISTILL | 32 | +0.04 Δ MOS | [+0.02, +0.06] | 候选好 |
| hotel-room | V-NRMOS-SCOREQ | 32 | +0.03 Δ MOS | [+0.01, +0.05] | 候选好 |
| hotel-room | V-SIM-GATE | 32 | -0.01 比例 | [-0.02, -0.00] | 候选差 |
| hotel-room | V-SWALLOW | 32 | +19.55 ms | [+6.45, +37.40] | 候选好 |
| hotel-room | V-TIMBRE | 32 | -0.18 dB | [-0.28, -0.09] | 候选差 |
| hotel-room | V-TSOS | 32 | +2.11 % | [+0.29, +4.00] | 候选好 |
| hotel-room | V-VACUUM | 32 | -1.72 dB | [-2.72, -0.74] | 候选差 |
| hotel-room | V-WEAK | 32 | +0.41 dB | [+0.19, +0.66] | 候选好 |
| library-classroom | V-EAT-ASR | 32 | -1.08 百分点 | [-2.12, -0.13] | 候选差 |
| library-classroom | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| library-classroom | V-NRMOS-SCOREQ | 32 | +0.04 Δ MOS | [+0.00, +0.08] | 候选好 |
| library-classroom | V-ONSET | 28 | -0.46 dB | [-0.78, -0.20] | 候选差 |
| library-classroom | V-PUMPV | 32 | +0.69 dB² | [+0.23, +1.41] | 候选好 |
| library-classroom | V-SWALLOW | 28 | +5.09 ms | [+0.80, +10.17] | 候选好 |
| library-classroom | V-TSOS | 32 | +6.14 % | [+4.00, +8.46] | 候选好 |
| library-classroom | V-WEAK | 32 | +0.98 dB | [+0.83, +1.13] | 候选好 |
| meeting-room | V-NR | 32 | +2.19 dB | [+1.02, +3.25] | 候选好 |
| meeting-room | V-NRMOS-DISTILL | 32 | +0.04 Δ MOS | [+0.03, +0.05] | 候选好 |
| meeting-room | V-NRMOS-SCOREQ | 32 | +0.08 Δ MOS | [+0.06, +0.09] | 候选好 |
| meeting-room | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| meeting-room | V-VACUUM | 32 | -2.19 dB | [-3.26, -1.05] | 候选差 |
| open-office | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| open-office | V-NR | 32 | +2.75 dB | [+2.00, +3.49] | 候选好 |
| open-office | V-NRMOS-DISTILL | 32 | +0.05 Δ MOS | [+0.02, +0.07] | 候选好 |
| open-office | V-NRMOS-SCOREQ | 32 | +0.05 Δ MOS | [+0.02, +0.08] | 候选好 |
| open-office | V-ONSET | 25 | -0.60 dB | [-1.01, -0.17] | 候选差 |
| open-office | V-SIM-FA | 16 | +0.13 比例 | [+0.04, +0.23] | 候选好 |
| open-office | V-SIM-GATE | 32 | -0.01 比例 | [-0.02, -0.00] | 候选差 |
| open-office | V-SWALLOW | 25 | -26.24 ms | [-60.03, -1.47] | 候选差 |
| open-office | V-TIMBRE | 32 | -0.15 dB | [-0.32, -0.03] | 候选差 |
| open-office | V-TSOS | 32 | -0.88 % | [-1.76, -0.04] | 候选差 |
| open-office | V-VACUUM | 32 | -2.75 dB | [-3.48, -2.06] | 候选差 |
| open-office | V-WEAK | 32 | +0.50 dB | [+0.19, +0.79] | 候选好 |
| phone-booth | V-CLICK | 32 | -2.81 个/分钟 | [-5.31, -0.62] | 候选差 |
| phone-booth | V-NR | 32 | +0.81 dB | [+0.12, +1.61] | 候选好 |
| phone-booth | V-NRMOS-DISTILL | 32 | +0.02 Δ MOS | [+0.00, +0.04] | 候选好 |
| phone-booth | V-NRMOS-SCOREQ | 32 | +0.02 Δ MOS | [+0.00, +0.04] | 候选好 |
| phone-booth | V-PUMPV | 32 | +0.82 dB² | [+0.16, +1.59] | 候选好 |
| phone-booth | V-SIM-GATE | 32 | -0.01 比例 | [-0.03, -0.00] | 候选差 |
| phone-booth | V-SPLVL | 32 | -0.53 dB | [-0.87, -0.19] | 候选差 |
| phone-booth | V-VACUUM | 32 | -0.81 dB | [-1.64, -0.14] | 候选差 |
| rail-air-cabin | V-ESTOI | 32 | +0.01 Δ | [+0.00, +0.01] | 候选好 |
| rail-air-cabin | V-MUSIC | 32 | -0.55 × | [-1.01, -0.16] | 候选差 |
| rail-air-cabin | V-NR | 32 | +2.98 dB | [+2.34, +3.62] | 候选好 |
| rail-air-cabin | V-NRMOS-DISTILL | 32 | +0.04 Δ MOS | [+0.00, +0.06] | 候选好 |
| rail-air-cabin | V-NRMOS-SCOREQ | 32 | +0.04 Δ MOS | [+0.02, +0.05] | 候选好 |
| rail-air-cabin | V-ONSET | 32 | -1.77 dB | [-2.41, -1.17] | 候选差 |
| rail-air-cabin | V-SIM-GATE | 32 | -0.05 比例 | [-0.11, -0.00] | 候选差 |
| rail-air-cabin | V-TIMBRE | 32 | -0.47 dB | [-0.80, -0.06] | 候选差 |
| rail-air-cabin | V-TSOS | 32 | -2.19 % | [-6.07, -0.14] | 候选差 |
| rail-air-cabin | V-VACUUM | 32 | -2.98 dB | [-3.67, -2.28] | 候选差 |
| rail-air-cabin | V-WEAK | 32 | -0.75 dB | [-1.02, -0.46] | 候选差 |
| street-outdoor | V-BGSTAB | 32 | -0.78 dB | [-1.49, -0.10] | 候选差 |
| street-outdoor | V-CLICK | 32 | -10.62 个/分钟 | [-14.69, -6.56] | 候选差 |
| street-outdoor | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| street-outdoor | V-NR | 32 | +2.26 dB | [+1.48, +3.03] | 候选好 |
| street-outdoor | V-NRMOS-DISTILL | 32 | +0.12 Δ MOS | [+0.10, +0.14] | 候选好 |
| street-outdoor | V-NRMOS-SCOREQ | 32 | +0.08 Δ MOS | [+0.06, +0.10] | 候选好 |
| street-outdoor | V-ONSET | 32 | -1.38 dB | [-2.08, -0.73] | 候选差 |
| street-outdoor | V-PSE-OVL | 10 | +0.59 dB | [+0.23, +0.98] | 候选好 |
| street-outdoor | V-PUMPV | 32 | +0.55 dB² | [+0.04, +1.32] | 候选好 |
| street-outdoor | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| street-outdoor | V-SPLVL | 32 | +0.70 dB | [+0.34, +1.09] | 候选好 |
| street-outdoor | V-TSOS | 32 | -1.05 % | [-1.75, -0.39] | 候选差 |
| street-outdoor | V-VACUUM | 32 | -2.26 dB | [-3.03, -1.51] | 候选差 |
| street-outdoor | V-WEAK | 32 | +0.38 dB | [+0.14, +0.63] | 候选好 |
| transit-hub | V-BGSTAB | 32 | -1.02 dB | [-1.72, -0.33] | 候选差 |
| transit-hub | V-CLICK | 32 | -17.34 个/分钟 | [-24.69, -10.00] | 候选差 |
| transit-hub | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| transit-hub | V-NR | 32 | +1.92 dB | [+1.54, +2.33] | 候选好 |
| transit-hub | V-NRMOS-DISTILL | 32 | +0.08 Δ MOS | [+0.05, +0.11] | 候选好 |
| transit-hub | V-NRMOS-SCOREQ | 32 | +0.07 Δ MOS | [+0.03, +0.12] | 候选好 |
| transit-hub | V-ONSET | 32 | -1.35 dB | [-1.91, -0.78] | 候选差 |
| transit-hub | V-PSE-OVL | 24 | +1.14 dB | [+0.75, +1.50] | 候选好 |
| transit-hub | V-SPLVL | 32 | +0.80 dB | [+0.28, +1.25] | 候选好 |
| transit-hub | V-TIMBRE | 32 | -0.23 dB | [-0.39, -0.10] | 候选差 |
| transit-hub | V-VACUUM | 32 | -1.92 dB | [-2.33, -1.53] | 候选差 |
| transit-hub | V-WEAK | 32 | +0.36 dB | [+0.06, +0.60] | 候选好 |

### GTCRN 落后最多的前 5 处（场景 × 声音 × 评分项，配对单位 ≥ 5，按标准化效应排；未做多重比较校正）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| street-outdoor | wind | V-NRMOS-DISTILL | 10（5） | +0.16 Δ MOS | [+0.13, +0.19] | 2.40 |
| street-outdoor | wind | V-ESTOI | 10（5） | +0.03 Δ | [+0.03, +0.04] | 2.34 |
| street-outdoor | footsteps | V-ESTOI | 10（5） | +0.03 Δ | [+0.03, +0.03] | 2.31 |
| street-outdoor | bed-terrace | V-NRMOS-DISTILL | 10（5） | +0.15 Δ MOS | [+0.11, +0.19] | 2.26 |
| transit-hub | bed-ace | V-ESTOI | 10（5） | +0.03 Δ | [+0.02, +0.04] | 2.18 |

### 反过来：候选落后 GTCRN 最多的前 5 处（同口径）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| bus-metro-cabin | door-pneumatic | V-VACUUM | 12（6） | -4.21 dB | [-5.53, -2.88] | -1.91 |
| bus-metro-cabin | door-chime | V-TIMBRE | 12（6） | -0.82 dB | [-1.06, -0.54] | -1.78 |
| bus-metro-cabin | brake | V-VACUUM | 12（6） | -3.88 dB | [-5.54, -2.34] | -1.76 |
| home-family | bed-kitchen | V-VACUUM | 9（5） | -3.85 dB | [-4.32, -3.21] | -1.75 |
| bus-metro-cabin | bed-bus | V-VACUUM | 10（5） | -3.80 dB | [-5.75, -1.94] | -1.73 |

