# exp016 配对分析（run `20261105-005916-4ccf0638+20261105-115707-fcf7cfb0`，由 analyze.py 生成，勿手改）

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

## dpdfnet2 vs gtcrn

### 每个评分项（全部 480 段）

| 评分项 | 方向 | 单位 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|---|
| V-BGSTAB | 越小越好 | dB | 329 | -1.39 | [-1.75, -1.06] | 候选差 |
| V-CLICK | 越小越好 | 个/分钟 | 480 | +0.27 | [-1.12, +1.62] | 分不出 |
| V-EAT-ASR | 越小越好 | 百分点 | 480 | +0.48 | [-0.15, +1.07] | 分不出 |
| V-ESTOI | 越大越好 | Δ | 480 | +0.02 | [+0.02, +0.03] | 候选好 |
| V-INTF | 越小越好 | dB | 178 | +1.11 | [+0.75, +1.47] | 候选好 |
| V-LEAK | 越小越好 | % | 100 | +6.08 | [+0.01, +12.54] | 候选好 |
| V-MUSIC | 越小越好 | × | 288 | -0.17 | [-0.36, +0.02] | 分不出 |
| V-NR | 越小越好 | dB | 457 | +3.11 | [+2.88, +3.34] | 候选好 |
| V-NRMOS-DISTILL | 越大越好 | Δ MOS | 480 | +0.12 | [+0.11, +0.13] | 候选好 |
| V-NRMOS-SCOREQ | 越大越好 | Δ MOS | 480 | +0.11 | [+0.10, +0.12] | 候选好 |
| V-ONSET | 越小越好 | dB | 432 | -1.45 | [-1.65, -1.25] | 候选差 |
| V-PSE-OVL | 越大越好 | dB | 242 | +1.37 | [+1.15, +1.61] | 候选好 |
| V-PUMPV | 越小越好 | dB² | 480 | -1.55 | [-1.95, -1.18] | 候选差 |
| V-SIM-FA | 越小越好 | 比例 | 79 | +0.25 | [+0.18, +0.33] | 候选好 |
| V-SIM-GATE | 越小越好 | 比例 | 480 | -0.04 | [-0.05, -0.03] | 候选差 |
| V-SPLVL | 越大越好 | dB | 465 | +0.75 | [+0.58, +0.90] | 候选好 |
| V-SWALLOW | 越小越好 | ms | 432 | -24.83 | [-46.72, -8.06] | 候选差 |
| V-TIMBRE | 越小越好 | dB | 480 | -0.07 | [-0.13, -0.02] | 候选差 |
| V-TSOS | 越小越好 | % | 480 | -1.52 | [-2.93, -0.19] | 候选差 |
| V-VACUUM | 越大越好 | dB | 457 | -3.11 | [-3.34, -2.89] | 候选差 |
| V-WEAK | 越小越好 | dB | 480 | +0.73 | [+0.63, +0.83] | 候选好 |

### 场景 × 评分项：判断不是「分不出」的格子

| 场景 | 评分项 | 段数 | 平均 | 95% 区间 | 判断 |
|---|---|---|---|---|---|
| bus-metro-cabin | V-BGSTAB | 32 | -1.64 dB | [-2.61, -0.68] | 候选差 |
| bus-metro-cabin | V-ESTOI | 32 | +0.02 Δ | [+0.01, +0.02] | 候选好 |
| bus-metro-cabin | V-INTF | 19 | +2.97 dB | [+1.79, +4.13] | 候选好 |
| bus-metro-cabin | V-NR | 32 | +3.53 dB | [+2.33, +4.70] | 候选好 |
| bus-metro-cabin | V-NRMOS-DISTILL | 32 | +0.06 Δ MOS | [+0.02, +0.10] | 候选好 |
| bus-metro-cabin | V-NRMOS-SCOREQ | 32 | +0.06 Δ MOS | [+0.02, +0.10] | 候选好 |
| bus-metro-cabin | V-ONSET | 29 | -1.14 dB | [-1.77, -0.48] | 候选差 |
| bus-metro-cabin | V-PSE-OVL | 28 | +1.37 dB | [+0.69, +2.01] | 候选好 |
| bus-metro-cabin | V-PUMPV | 32 | -2.71 dB² | [-4.69, -0.98] | 候选差 |
| bus-metro-cabin | V-SIM-GATE | 32 | -0.07 比例 | [-0.17, -0.01] | 候选差 |
| bus-metro-cabin | V-TSOS | 32 | -4.95 % | [-9.84, -1.59] | 候选差 |
| bus-metro-cabin | V-VACUUM | 32 | -3.53 dB | [-4.72, -2.35] | 候选差 |
| bus-metro-cabin | V-WEAK | 32 | +0.78 dB | [+0.40, +1.11] | 候选好 |
| cafe | V-BGSTAB | 32 | -1.26 dB | [-1.98, -0.62] | 候选差 |
| cafe | V-ESTOI | 32 | +0.04 Δ | [+0.03, +0.04] | 候选好 |
| cafe | V-NR | 32 | +3.64 dB | [+2.85, +4.43] | 候选好 |
| cafe | V-NRMOS-DISTILL | 32 | +0.15 Δ MOS | [+0.12, +0.19] | 候选好 |
| cafe | V-NRMOS-SCOREQ | 32 | +0.18 Δ MOS | [+0.13, +0.23] | 候选好 |
| cafe | V-ONSET | 31 | -1.43 dB | [-2.19, -0.74] | 候选差 |
| cafe | V-PSE-OVL | 24 | +1.14 dB | [+0.40, +1.74] | 候选好 |
| cafe | V-PUMPV | 32 | -1.10 dB² | [-1.89, -0.41] | 候选差 |
| cafe | V-SIM-GATE | 32 | -0.05 比例 | [-0.07, -0.03] | 候选差 |
| cafe | V-SPLVL | 32 | +1.10 dB | [+0.77, +1.42] | 候选好 |
| cafe | V-VACUUM | 32 | -3.64 dB | [-4.37, -2.87] | 候选差 |
| cafe | V-WEAK | 32 | +0.57 dB | [+0.40, +0.75] | 候选好 |
| canteen | V-BGSTAB | 32 | -3.81 dB | [-4.73, -2.79] | 候选差 |
| canteen | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| canteen | V-INTF | 14 | +2.04 dB | [+0.67, +3.49] | 候选好 |
| canteen | V-NR | 32 | +3.58 dB | [+2.71, +4.44] | 候选好 |
| canteen | V-NRMOS-DISTILL | 32 | +0.17 Δ MOS | [+0.13, +0.21] | 候选好 |
| canteen | V-NRMOS-SCOREQ | 32 | +0.10 Δ MOS | [+0.06, +0.14] | 候选好 |
| canteen | V-ONSET | 29 | -3.13 dB | [-3.95, -2.38] | 候选差 |
| canteen | V-PSE-OVL | 22 | +1.41 dB | [+0.50, +2.27] | 候选好 |
| canteen | V-SIM-FA | 9 | +0.48 比例 | [+0.27, +0.66] | 候选好 |
| canteen | V-SIM-GATE | 32 | -0.06 比例 | [-0.10, -0.03] | 候选差 |
| canteen | V-SPLVL | 32 | +1.54 dB | [+0.91, +2.10] | 候选好 |
| canteen | V-VACUUM | 32 | -3.58 dB | [-4.40, -2.69] | 候选差 |
| canteen | V-WEAK | 32 | +1.19 dB | [+0.95, +1.41] | 候选好 |
| car-passenger | V-BGSTAB | 31 | -1.38 dB | [-2.26, -0.51] | 候选差 |
| car-passenger | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.03] | 候选好 |
| car-passenger | V-NR | 32 | +2.30 dB | [+1.51, +3.10] | 候选好 |
| car-passenger | V-NRMOS-DISTILL | 32 | +0.17 Δ MOS | [+0.13, +0.21] | 候选好 |
| car-passenger | V-NRMOS-SCOREQ | 32 | +0.13 Δ MOS | [+0.09, +0.17] | 候选好 |
| car-passenger | V-ONSET | 32 | -0.97 dB | [-1.55, -0.41] | 候选差 |
| car-passenger | V-PSE-OVL | 26 | +1.22 dB | [+0.72, +1.73] | 候选好 |
| car-passenger | V-PUMPV | 32 | -2.19 dB² | [-3.67, -1.07] | 候选差 |
| car-passenger | V-SIM-GATE | 32 | -0.01 比例 | [-0.02, -0.00] | 候选差 |
| car-passenger | V-SPLVL | 32 | +1.35 dB | [+1.04, +1.70] | 候选好 |
| car-passenger | V-TSOS | 32 | +1.61 % | [+0.52, +2.81] | 候选好 |
| car-passenger | V-VACUUM | 32 | -2.30 dB | [-3.14, -1.50] | 候选差 |
| car-passenger | V-WEAK | 32 | +0.42 dB | [+0.25, +0.57] | 候选好 |
| cowork | V-BGSTAB | 32 | -1.46 dB | [-2.40, -0.51] | 候选差 |
| cowork | V-ESTOI | 32 | +0.04 Δ | [+0.03, +0.04] | 候选好 |
| cowork | V-INTF | 16 | +2.34 dB | [+1.04, +3.67] | 候选好 |
| cowork | V-NR | 32 | +3.33 dB | [+2.52, +4.16] | 候选好 |
| cowork | V-NRMOS-DISTILL | 32 | +0.17 Δ MOS | [+0.14, +0.21] | 候选好 |
| cowork | V-NRMOS-SCOREQ | 32 | +0.15 Δ MOS | [+0.10, +0.19] | 候选好 |
| cowork | V-ONSET | 26 | -1.82 dB | [-2.59, -1.03] | 候选差 |
| cowork | V-PSE-OVL | 20 | +2.67 dB | [+1.92, +3.55] | 候选好 |
| cowork | V-PUMPV | 32 | -1.64 dB² | [-3.22, -0.18] | 候选差 |
| cowork | V-SIM-FA | 14 | +0.39 比例 | [+0.14, +0.64] | 候选好 |
| cowork | V-SPLVL | 32 | +1.69 dB | [+1.03, +2.39] | 候选好 |
| cowork | V-TSOS | 32 | +6.73 % | [+0.31, +15.94] | 候选好 |
| cowork | V-VACUUM | 32 | -3.33 dB | [-4.19, -2.51] | 候选差 |
| cowork | V-WEAK | 32 | +0.90 dB | [+0.66, +1.14] | 候选好 |
| home-family | V-BGSTAB | 10 | -1.75 dB | [-2.93, -0.29] | 候选差 |
| home-family | V-ESTOI | 32 | +0.03 Δ | [+0.02, +0.04] | 候选好 |
| home-family | V-INTF | 28 | +1.05 dB | [+0.39, +1.81] | 候选好 |
| home-family | V-NR | 9 | +4.60 dB | [+4.17, +5.10] | 候选好 |
| home-family | V-NRMOS-DISTILL | 32 | +0.09 Δ MOS | [+0.06, +0.12] | 候选好 |
| home-family | V-NRMOS-SCOREQ | 32 | +0.08 Δ MOS | [+0.04, +0.12] | 候选好 |
| home-family | V-ONSET | 32 | -0.82 dB | [-1.45, -0.19] | 候选差 |
| home-family | V-PSE-OVL | 28 | +1.40 dB | [+0.61, +2.14] | 候选好 |
| home-family | V-PUMPV | 32 | -2.28 dB² | [-4.36, -0.36] | 候选差 |
| home-family | V-SIM-FA | 28 | +0.21 比例 | [+0.14, +0.29] | 候选好 |
| home-family | V-TIMBRE | 32 | -0.06 dB | [-0.10, -0.02] | 候选差 |
| home-family | V-TSOS | 32 | -1.74 % | [-3.49, -0.25] | 候选差 |
| home-family | V-VACUUM | 9 | -4.60 dB | [-5.10, -4.17] | 候选差 |
| home-family | V-WEAK | 32 | +0.83 dB | [+0.47, +1.19] | 候选好 |
| home-quiet | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.02] | 候选好 |
| home-quiet | V-NR | 32 | +2.27 dB | [+1.62, +3.01] | 候选好 |
| home-quiet | V-NRMOS-DISTILL | 32 | +0.08 Δ MOS | [+0.06, +0.10] | 候选好 |
| home-quiet | V-NRMOS-SCOREQ | 32 | +0.10 Δ MOS | [+0.07, +0.12] | 候选好 |
| home-quiet | V-ONSET | 32 | -1.17 dB | [-1.80, -0.59] | 候选差 |
| home-quiet | V-PUMPV | 32 | -1.96 dB² | [-3.51, -0.63] | 候选差 |
| home-quiet | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| home-quiet | V-SPLVL | 32 | +0.21 dB | [+0.10, +0.36] | 候选好 |
| home-quiet | V-TIMBRE | 32 | -0.13 dB | [-0.24, -0.04] | 候选差 |
| home-quiet | V-VACUUM | 32 | -2.27 dB | [-3.01, -1.58] | 候选差 |
| home-quiet | V-WEAK | 32 | +0.73 dB | [+0.44, +1.03] | 候选好 |
| hotel-room | V-CLICK | 32 | +3.91 个/分钟 | [+1.56, +6.56] | 候选好 |
| hotel-room | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| hotel-room | V-NR | 32 | +2.92 dB | [+2.10, +3.89] | 候选好 |
| hotel-room | V-NRMOS-DISTILL | 32 | +0.09 Δ MOS | [+0.05, +0.12] | 候选好 |
| hotel-room | V-NRMOS-SCOREQ | 32 | +0.11 Δ MOS | [+0.09, +0.13] | 候选好 |
| hotel-room | V-ONSET | 32 | -0.76 dB | [-1.21, -0.33] | 候选差 |
| hotel-room | V-PUMPV | 32 | -1.19 dB² | [-2.24, -0.26] | 候选差 |
| hotel-room | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| hotel-room | V-SPLVL | 32 | +0.22 dB | [+0.10, +0.37] | 候选好 |
| hotel-room | V-SWALLOW | 32 | -23.80 ms | [-44.85, -5.85] | 候选差 |
| hotel-room | V-TIMBRE | 32 | -0.06 dB | [-0.08, -0.04] | 候选差 |
| hotel-room | V-TSOS | 32 | +2.09 % | [+0.84, +3.53] | 候选好 |
| hotel-room | V-VACUUM | 32 | -2.92 dB | [-3.87, -1.99] | 候选差 |
| hotel-room | V-WEAK | 32 | +0.92 dB | [+0.66, +1.19] | 候选好 |
| library-classroom | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| library-classroom | V-NR | 32 | +2.73 dB | [+2.12, +3.35] | 候选好 |
| library-classroom | V-NRMOS-DISTILL | 32 | +0.08 Δ MOS | [+0.04, +0.12] | 候选好 |
| library-classroom | V-NRMOS-SCOREQ | 32 | +0.11 Δ MOS | [+0.07, +0.16] | 候选好 |
| library-classroom | V-ONSET | 28 | -1.12 dB | [-1.72, -0.60] | 候选差 |
| library-classroom | V-SIM-GATE | 32 | -0.02 比例 | [-0.03, -0.01] | 候选差 |
| library-classroom | V-SPLVL | 32 | +0.31 dB | [+0.15, +0.52] | 候选好 |
| library-classroom | V-TSOS | 32 | +5.47 % | [+3.42, +7.54] | 候选好 |
| library-classroom | V-VACUUM | 32 | -2.73 dB | [-3.31, -2.11] | 候选差 |
| library-classroom | V-WEAK | 32 | +1.21 dB | [+1.04, +1.39] | 候选好 |
| meeting-room | V-NR | 32 | +3.95 dB | [+3.23, +4.65] | 候选好 |
| meeting-room | V-NRMOS-DISTILL | 32 | +0.09 Δ MOS | [+0.06, +0.12] | 候选好 |
| meeting-room | V-NRMOS-SCOREQ | 32 | +0.15 Δ MOS | [+0.11, +0.19] | 候选好 |
| meeting-room | V-ONSET | 8 | -1.82 dB | [-3.22, -0.82] | 候选差 |
| meeting-room | V-PUMPV | 32 | -2.48 dB² | [-3.41, -1.67] | 候选差 |
| meeting-room | V-SIM-GATE | 32 | -0.06 比例 | [-0.08, -0.04] | 候选差 |
| meeting-room | V-TSOS | 32 | -18.42 % | [-26.18, -10.10] | 候选差 |
| meeting-room | V-VACUUM | 32 | -3.95 dB | [-4.60, -3.23] | 候选差 |
| meeting-room | V-WEAK | 32 | -0.79 dB | [-1.10, -0.46] | 候选差 |
| open-office | V-ESTOI | 32 | +0.02 Δ | [+0.02, +0.03] | 候选好 |
| open-office | V-NR | 32 | +3.65 dB | [+2.63, +4.61] | 候选好 |
| open-office | V-NRMOS-DISTILL | 32 | +0.10 Δ MOS | [+0.07, +0.13] | 候选好 |
| open-office | V-NRMOS-SCOREQ | 32 | +0.12 Δ MOS | [+0.08, +0.16] | 候选好 |
| open-office | V-ONSET | 25 | -1.11 dB | [-2.09, -0.23] | 候选差 |
| open-office | V-PSE-OVL | 30 | +1.11 dB | [+0.38, +1.74] | 候选好 |
| open-office | V-PUMPV | 32 | -1.21 dB² | [-2.48, -0.13] | 候选差 |
| open-office | V-SIM-FA | 16 | +0.17 比例 | [+0.05, +0.33] | 候选好 |
| open-office | V-SIM-GATE | 32 | -0.02 比例 | [-0.04, -0.01] | 候选差 |
| open-office | V-SPLVL | 32 | +0.72 dB | [+0.42, +1.01] | 候选好 |
| open-office | V-SWALLOW | 25 | -32.45 ms | [-62.93, -7.55] | 候选差 |
| open-office | V-VACUUM | 32 | -3.65 dB | [-4.64, -2.72] | 候选差 |
| open-office | V-WEAK | 32 | +1.09 dB | [+0.90, +1.31] | 候选好 |
| phone-booth | V-ESTOI | 32 | +0.01 Δ | [+0.00, +0.01] | 候选好 |
| phone-booth | V-NR | 32 | +2.35 dB | [+1.62, +3.15] | 候选好 |
| phone-booth | V-NRMOS-SCOREQ | 32 | +0.07 Δ MOS | [+0.03, +0.10] | 候选好 |
| phone-booth | V-ONSET | 32 | -1.44 dB | [-2.37, -0.63] | 候选差 |
| phone-booth | V-PUMPV | 32 | -2.93 dB² | [-6.06, -0.46] | 候选差 |
| phone-booth | V-SIM-GATE | 32 | -0.09 比例 | [-0.19, -0.03] | 候选差 |
| phone-booth | V-SWALLOW | 32 | -219.10 ms | [-491.30, -19.00] | 候选差 |
| phone-booth | V-VACUUM | 32 | -2.35 dB | [-3.12, -1.63] | 候选差 |
| rail-air-cabin | V-ESTOI | 32 | +0.01 Δ | [+0.01, +0.02] | 候选好 |
| rail-air-cabin | V-MUSIC | 32 | -0.58 × | [-1.21, -0.04] | 候选差 |
| rail-air-cabin | V-NR | 32 | +3.58 dB | [+3.12, +4.00] | 候选好 |
| rail-air-cabin | V-NRMOS-DISTILL | 32 | +0.13 Δ MOS | [+0.08, +0.17] | 候选好 |
| rail-air-cabin | V-NRMOS-SCOREQ | 32 | +0.05 Δ MOS | [+0.03, +0.08] | 候选好 |
| rail-air-cabin | V-ONSET | 32 | -2.51 dB | [-3.26, -1.78] | 候选差 |
| rail-air-cabin | V-PSE-OVL | 30 | +0.65 dB | [+0.20, +1.04] | 候选好 |
| rail-air-cabin | V-PUMPV | 32 | -1.82 dB² | [-3.48, -0.29] | 候选差 |
| rail-air-cabin | V-SIM-GATE | 32 | -0.08 比例 | [-0.16, -0.03] | 候选差 |
| rail-air-cabin | V-SPLVL | 32 | +1.01 dB | [+0.07, +1.65] | 候选好 |
| rail-air-cabin | V-TSOS | 32 | -3.77 % | [-9.42, -0.52] | 候选差 |
| rail-air-cabin | V-VACUUM | 32 | -3.58 dB | [-4.03, -3.09] | 候选差 |
| rail-air-cabin | V-WEAK | 32 | +0.88 dB | [+0.42, +1.25] | 候选好 |
| street-outdoor | V-BGSTAB | 32 | -1.66 dB | [-2.65, -0.79] | 候选差 |
| street-outdoor | V-ESTOI | 32 | +0.03 Δ | [+0.03, +0.04] | 候选好 |
| street-outdoor | V-INTF | 8 | +1.40 dB | [+0.62, +2.48] | 候选好 |
| street-outdoor | V-NR | 32 | +3.04 dB | [+2.50, +3.60] | 候选好 |
| street-outdoor | V-NRMOS-DISTILL | 32 | +0.19 Δ MOS | [+0.17, +0.21] | 候选好 |
| street-outdoor | V-NRMOS-SCOREQ | 32 | +0.12 Δ MOS | [+0.10, +0.15] | 候选好 |
| street-outdoor | V-ONSET | 32 | -1.41 dB | [-2.12, -0.73] | 候选差 |
| street-outdoor | V-PSE-OVL | 10 | +1.15 dB | [+0.99, +1.34] | 候选好 |
| street-outdoor | V-SIM-GATE | 32 | -0.02 比例 | [-0.04, -0.01] | 候选差 |
| street-outdoor | V-SPLVL | 32 | +1.11 dB | [+0.73, +1.51] | 候选好 |
| street-outdoor | V-TSOS | 32 | -0.89 % | [-1.45, -0.42] | 候选差 |
| street-outdoor | V-VACUUM | 32 | -3.04 dB | [-3.63, -2.49] | 候选差 |
| street-outdoor | V-WEAK | 32 | +0.58 dB | [+0.41, +0.77] | 候选好 |
| transit-hub | V-BGSTAB | 32 | -2.20 dB | [-3.46, -0.97] | 候选差 |
| transit-hub | V-ESTOI | 32 | +0.03 Δ | [+0.03, +0.04] | 候选好 |
| transit-hub | V-INTF | 14 | +1.53 dB | [+0.91, +2.13] | 候选好 |
| transit-hub | V-NR | 32 | +2.21 dB | [+1.64, +2.83] | 候选好 |
| transit-hub | V-NRMOS-DISTILL | 32 | +0.15 Δ MOS | [+0.12, +0.18] | 候选好 |
| transit-hub | V-NRMOS-SCOREQ | 32 | +0.13 Δ MOS | [+0.07, +0.19] | 候选好 |
| transit-hub | V-ONSET | 32 | -1.46 dB | [-2.07, -0.85] | 候选差 |
| transit-hub | V-PSE-OVL | 24 | +1.90 dB | [+1.28, +2.44] | 候选好 |
| transit-hub | V-SIM-GATE | 32 | -0.03 比例 | [-0.06, -0.00] | 候选差 |
| transit-hub | V-SPLVL | 32 | +1.40 dB | [+0.99, +1.79] | 候选好 |
| transit-hub | V-TIMBRE | 32 | -0.11 dB | [-0.14, -0.08] | 候选差 |
| transit-hub | V-VACUUM | 32 | -2.21 dB | [-2.82, -1.68] | 候选差 |
| transit-hub | V-WEAK | 32 | +0.95 dB | [+0.73, +1.16] | 候选好 |

### GTCRN 落后最多的前 5 处（场景 × 声音 × 评分项，配对单位 ≥ 5，按标准化效应排；未做多重比较校正）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| transit-hub | bed-ace | V-ESTOI | 10（5） | +0.05 Δ | [+0.04, +0.05] | 2.75 |
| street-outdoor | wind | V-NRMOS-DISTILL | 10（5） | +0.22 Δ MOS | [+0.19, +0.26] | 2.48 |
| street-outdoor | bed-terrace | V-NRMOS-DISTILL | 10（5） | +0.22 Δ MOS | [+0.18, +0.26] | 2.46 |
| bus-metro-cabin | bed-bus | V-NR | 10（5） | +5.18 dB | [+3.20, +7.22] | 2.46 |
| cafe | music | V-ESTOI | 18（9） | +0.04 Δ | [+0.04, +0.05] | 2.37 |

### 反过来：候选落后 GTCRN 最多的前 5 处（同口径）

| 场景 | 声音 | 评分项 | 段数（单位） | 平均 | 95% 区间 | 标准化效应 |
|---|---|---|---|---|---|---|
| bus-metro-cabin | bed-bus | V-VACUUM | 10（5） | -5.18 dB | [-7.22, -3.20] | -2.46 |
| meeting-room | laugh | V-TSOS | 14（7） | -25.69 % | [-33.16, -17.34] | -2.26 |
| home-family | bed-kitchen | V-VACUUM | 9（5） | -4.60 dB | [-5.10, -4.14] | -2.18 |
| meeting-room | bed-ace | V-VACUUM | 16（8） | -4.53 dB | [-5.30, -3.77] | -2.15 |
| bus-metro-cabin | door-chime | V-VACUUM | 12（6） | -4.52 dB | [-6.19, -2.75] | -2.15 |

