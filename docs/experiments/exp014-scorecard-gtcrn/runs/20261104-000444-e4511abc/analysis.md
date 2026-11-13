# exp014 分析表（run `20261104-000444-e4511abc`，由 analyze.py 生成，勿手改）

## 1. 每个场景：gtcrn 不过的项，以及原声直通是否也不过

| 场景 | 档 | 设备 | 判定 | 不过的项 | 其中原声直通也不过 |
|---|---|---|---|---|---|
| bus-metro-cabin | common | external-mic | fail | V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| bus-metro-cabin | common | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-MUSIC V-NR V-INTF | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| bus-metro-cabin | stress | external-mic | fail | V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| bus-metro-cabin | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| cafe | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| cafe | common | mba-builtin | fail | V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| cafe | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| cafe | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| canteen | common | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| canteen | common | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| canteen | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| canteen | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| car-passenger | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-MUSIC V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-INTF |
| car-passenger | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-MUSIC V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-INTF |
| car-passenger | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-MUSIC V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF |
| car-passenger | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| cowork | common | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| cowork | common | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| cowork | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| cowork | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| home-family | common | external-mic | fail | V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| home-family | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA |
| home-family | stress | external-mic | fail | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| home-family | stress | mba-builtin | fail | V-EAT-ASR V-ONSET V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA | V-ONSET V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| home-quiet | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC V-NR | V-SIM-GATE V-TIMBRE V-NR |
| home-quiet | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-MUSIC V-NR | V-SIM-GATE V-NR |
| home-quiet | stress | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-MUSIC V-NR | V-SIM-GATE V-PUMPV V-NR |
| home-quiet | stress | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC V-NR | V-SIM-GATE V-PUMPV V-TIMBRE V-NR |
| hotel-room | common | external-mic | fail | V-EAT-ASR V-SIM-GATE V-PUMPV V-NR | V-SIM-GATE V-PUMPV V-NR |
| hotel-room | common | mba-builtin | fail | V-EAT-ASR V-SIM-GATE V-PUMPV V-MUSIC V-NR | V-SIM-GATE V-PUMPV V-NR |
| hotel-room | stress | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NR | V-SIM-GATE V-PUMPV V-NR |
| hotel-room | stress | mba-builtin | fail | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR | V-SIM-GATE V-PUMPV V-TIMBRE V-NR |
| library-classroom | common | external-mic | fail | V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-MUSIC V-NR | V-SIM-GATE V-NR |
| library-classroom | common | mba-builtin | fail | V-EAT-ASR V-SIM-GATE V-PUMPV V-MUSIC V-NR | V-SIM-GATE V-NR |
| library-classroom | stress | external-mic | fail | V-SIM-GATE V-PUMPV V-TIMBRE V-NR | V-SIM-GATE V-TIMBRE V-NR |
| library-classroom | stress | mba-builtin | fail | V-ONSET V-SIM-GATE V-PUMPV V-MUSIC | V-SIM-GATE V-PUMPV |
| meeting-room | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NR | V-SIM-GATE V-PUMPV V-NR |
| meeting-room | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ | V-SIM-GATE V-PUMPV |
| meeting-room | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-NR | V-SIM-GATE V-TIMBRE V-NR |
| meeting-room | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR | V-SIM-GATE V-TIMBRE V-NR |
| open-office | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| open-office | common | mba-builtin | fail | V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA | V-SIM-GATE V-NR V-INTF V-SIM-FA |
| open-office | stress | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA |
| open-office | stress | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-INTF V-SIM-FA | V-SIM-GATE V-PUMPV V-TIMBRE V-INTF V-SIM-FA |
| phone-booth | common | external-mic | fail | V-LAT V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-MUSIC V-NR | V-LAT V-SIM-GATE V-NR |
| phone-booth | common | mba-builtin | fail | V-LAT V-EAT-ASR V-SIM-GATE V-PUMPV V-MUSIC V-NR | V-LAT V-SIM-GATE V-NR |
| phone-booth | stress | external-mic | fail | V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-MUSIC V-NR | V-SIM-GATE V-TIMBRE V-NR |
| phone-booth | stress | mba-builtin | fail | V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC | V-SIM-GATE V-TIMBRE |
| rail-air-cabin | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| rail-air-cabin | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-INTF |
| rail-air-cabin | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| rail-air-cabin | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-INTF | V-SIM-GATE V-PUMPV V-TIMBRE V-INTF |
| street-outdoor | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-MUSIC V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF |
| street-outdoor | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-MUSIC V-NR V-INTF | V-SIM-GATE V-PUMPV V-NR V-INTF |
| street-outdoor | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| street-outdoor | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-MUSIC V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| transit-hub | common | external-mic | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-MUSIC V-NR V-INTF | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| transit-hub | common | mba-builtin | fail | V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-MUSIC V-NR V-INTF | V-SIM-GATE V-PUMPV V-NR V-INTF |
| transit-hub | stress | external-mic | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF | V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |
| transit-hub | stress | mba-builtin | fail | V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-NR V-INTF | V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF |

## 2. 每项在多少个组合里不过（组合 = 场景 × 档 × 设备）

每格：不过 / 不知道 / 过（共 60 个组合）。「不知道」= 该组合里每段都判不了（如没有旁人说话）

| 项 | passthrough | ref | gtcrn |
|---|---|---|---|
| V-NAN | 0 / 0 / 60 | 0 / 0 / 60 | 0 / 0 / 60 |
| V-LAT | 2 / 0 / 58 | 2 / 0 / 58 | 2 / 0 / 58 |
| V-SPLVL | 0 / 0 / 60 | 0 / 0 / 60 | 22 / 0 / 38 |
| V-EAT-ASR | 0 / 0 / 60 | 39 / 0 / 21 | 51 / 0 / 9 |
| V-ONSET | 21 / 0 / 39 | 40 / 0 / 20 | 47 / 0 / 13 |
| V-SIM-GATE | 60 / 0 / 0 | 56 / 0 / 4 | 59 / 0 / 1 |
| V-WEAK | 0 / 0 / 60 | 0 / 0 / 60 | 0 / 0 / 60 |
| V-PUMPV | 48 / 0 / 12 | 58 / 0 / 2 | 60 / 0 / 0 |
| V-CLICK（只报不判） | 0 / 0 / 60 | 59 / 0 / 1 | 59 / 0 / 1 |
| V-ESTOI | 0 / 0 / 60 | 0 / 0 / 60 | 0 / 0 / 60 |
| V-TIMBRE | 46 / 0 / 14 | 45 / 0 / 15 | 34 / 0 / 26 |
| V-NRMOS-SCOREQ | 0 / 0 / 60 | 36 / 0 / 24 | 30 / 0 / 30 |
| V-NRMOS-DISTILL | 0 / 0 / 60 | 9 / 0 / 51 | 0 / 0 / 60 |
| V-MUSIC | 0 / 24 / 36 | 20 / 24 / 16 | 24 / 24 / 12 |
| V-NR | 60 / 0 / 0 | 60 / 0 / 0 | 52 / 0 / 8 |
| V-INTF | 40 / 20 / 0 | 40 / 20 / 0 | 40 / 20 / 0 |
| V-SIM-FA | 18 / 40 / 2 | 20 / 40 / 0 | 20 / 40 / 0 |
| V-BGSTAB | 0 / 16 / 44 | 0 / 16 / 44 | 0 / 16 / 44 |

## 3. gtcrn：模型 − 原声直通（同一批段配对）

| 项 | 好的方向 | 配对段数 | 均值差（模型 − 原声直通） |
|---|---|---|---|
| V-NAN | 越小越好 | 480 | +0.00 |
| V-LAT | 越小越好 | 480 | +16.00 |
| V-SPLVL | 越大越好 | 465 | -1.47 |
| V-EAT-ASR | 越小越好 | 480 | +0.60 |
| V-ONSET | 越小越好 | 432 | +1.48 |
| V-SIM-GATE | 越小越好 | 480 | -0.37 |
| V-WEAK | 越小越好 | 480 | +4.08 |
| V-PUMPV | 越小越好 | 480 | +2.36 |
| V-ESTOI | 越大越好 | 480 | +0.02 |
| V-TIMBRE | 越小越好 | 462 | -0.69 |
| V-NRMOS-SCOREQ | 越大越好 | 480 | +0.14 |
| V-NRMOS-DISTILL | 越大越好 | 480 | +0.21 |
| V-MUSIC | 越小越好 | 288 | +0.56 |
| V-NR | 越小越好 | 457 | -6.58 |
| V-INTF | 越小越好 | 178 | -4.05 |
| V-SIM-FA | 越小越好 | 79 | +0.42 |
| V-BGSTAB | 越小越好 | 329 | +2.40 |

### 3b. 每个场景 × 档的配对差值（gtcrn − passthrough，两种设备合并，括号里是段数）

| 场景 | 档 | V-NR | V-INTF | V-EAT-ASR | V-SPLVL | V-ONSET | V-PUMPV | V-NRMOS-SCOREQ | V-NRMOS-DISTILL |
|---|---|---|---|---|---|---|---|---|---|
| bus-metro-cabin | common | -4.02 (20) | -3.76 (10) | +1.33 (20) | -1.82 (20) | +1.08 (20) | +0.89 (20) | +0.13 (20) | +0.21 (20) |
| bus-metro-cabin | stress | -4.42 (12) | -4.45 (9) | +1.87 (12) | -2.69 (11) | +1.30 (9) | +0.30 (12) | +0.07 (12) | +0.17 (12) |
| cafe | common | -6.43 (20) | -5.28 (8) | +0.71 (20) | -1.47 (20) | +1.56 (19) | +2.07 (20) | +0.23 (20) | +0.26 (20) |
| cafe | stress | -6.08 (12) | -3.30 (10) | +1.49 (12) | -2.30 (12) | +2.11 (12) | +3.08 (12) | +0.05 (12) | +0.18 (12) |
| canteen | common | -5.38 (20) | -4.17 (8) | +0.31 (20) | -2.68 (20) | +1.40 (17) | +3.01 (20) | +0.07 (20) | +0.19 (20) |
| canteen | stress | -4.84 (12) | -3.99 (6) | +0.01 (12) | -3.33 (12) | +1.38 (12) | +7.90 (12) | +0.00 (12) | +0.16 (12) |
| car-passenger | common | -8.19 (20) | -7.18 (8) | -2.73 (20) | -1.38 (20) | +2.05 (20) | +3.35 (20) | +0.21 (20) | +0.27 (20) |
| car-passenger | stress | -8.15 (12) | -5.41 (8) | +2.31 (12) | -2.81 (12) | +2.71 (12) | +3.99 (12) | +0.07 (12) | +0.21 (12) |
| cowork | common | -4.90 (20) | -4.53 (8) | -0.26 (20) | -1.39 (20) | +1.04 (14) | +1.29 (20) | +0.18 (20) | +0.22 (20) |
| cowork | stress | -4.89 (12) | -4.44 (8) | +1.50 (12) | -3.37 (12) | +0.53 (12) | +3.23 (12) | +0.06 (12) | +0.16 (12) |
| home-family | common | -4.44 (5) | -1.63 (18) | +1.03 (20) | -0.54 (9) | +0.98 (20) | +1.96 (20) | +0.18 (20) | +0.20 (20) |
| home-family | stress | -5.37 (4) | -2.25 (10) | -1.77 (12) | -1.54 (9) | +1.09 (12) | +8.46 (12) | +0.18 (12) | +0.21 (12) |
| home-quiet | common | -7.25 (20) | — | +0.34 (20) | -0.17 (20) | +2.09 (20) | +1.83 (20) | +0.12 (20) | +0.22 (20) |
| home-quiet | stress | -7.30 (12) | — | +0.90 (12) | -0.78 (12) | +3.02 (12) | +3.59 (12) | +0.21 (12) | +0.22 (12) |
| hotel-room | common | -6.98 (20) | — | +0.60 (20) | -0.17 (20) | +1.05 (20) | +1.14 (20) | +0.14 (20) | +0.20 (20) |
| hotel-room | stress | -6.77 (12) | — | +1.03 (12) | -0.66 (12) | +2.15 (12) | +5.91 (12) | +0.19 (12) | +0.22 (12) |
| library-classroom | common | -7.49 (20) | — | +0.01 (20) | -0.25 (20) | +0.80 (16) | +1.56 (20) | +0.21 (20) | +0.24 (20) |
| library-classroom | stress | -7.23 (12) | — | -1.68 (12) | -0.77 (12) | +1.76 (12) | +1.95 (12) | +0.16 (12) | +0.19 (12) |
| meeting-room | common | -8.56 (20) | — | +1.46 (20) | -1.04 (20) | +2.53 (6) | +2.22 (20) | +0.10 (20) | +0.20 (20) |
| meeting-room | stress | -6.47 (12) | — | +0.72 (12) | -2.07 (12) | +1.24 (2) | +1.46 (12) | +0.10 (12) | +0.22 (12) |
| open-office | common | -5.90 (20) | -3.09 (11) | -0.13 (20) | -0.64 (20) | +1.46 (17) | +1.27 (20) | +0.10 (20) | +0.19 (20) |
| open-office | stress | -7.11 (12) | -2.58 (8) | +2.42 (12) | -1.72 (12) | +1.57 (8) | +1.29 (12) | +0.14 (12) | +0.25 (12) |
| phone-booth | common | -8.42 (20) | — | +0.41 (20) | -0.19 (20) | +0.65 (20) | +1.03 (20) | +0.21 (20) | +0.21 (20) |
| phone-booth | stress | -7.92 (12) | — | -0.11 (12) | -0.43 (12) | +0.95 (12) | +1.41 (12) | +0.15 (12) | +0.21 (12) |
| rail-air-cabin | common | -7.56 (20) | -4.58 (18) | +3.20 (20) | -2.10 (20) | +1.71 (20) | +1.92 (20) | +0.09 (20) | +0.14 (20) |
| rail-air-cabin | stress | -6.81 (12) | -5.50 (8) | +1.03 (12) | -3.33 (12) | +1.79 (12) | +5.05 (12) | +0.03 (12) | +0.08 (12) |
| street-outdoor | common | -7.53 (20) | -4.79 (6) | +0.06 (20) | -0.93 (20) | +1.27 (20) | +1.42 (20) | +0.23 (20) | +0.24 (20) |
| street-outdoor | stress | -6.00 (12) | -5.64 (2) | -1.68 (12) | -2.24 (12) | +1.87 (12) | -1.39 (12) | +0.12 (12) | +0.22 (12) |
| transit-hub | common | -6.10 (20) | -5.20 (8) | +0.09 (20) | -1.72 (20) | +1.43 (20) | +2.02 (20) | +0.23 (20) | +0.29 (20) |
| transit-hub | stress | -5.05 (12) | -3.82 (6) | +5.28 (12) | -2.70 (12) | +0.97 (12) | +3.21 (12) | +0.06 (12) | +0.23 (12) |

### 3c. 按场景合并字数的吃字率（过关线 ≤ 1 个百分点，是按段判的；这里合并只作参考）

| 场景 | 档 | 参考字数 | passthrough 合并吃字（百分点） | ref 合并吃字（百分点） | gtcrn 合并吃字（百分点） |
|---|---|---|---|---|---|
| bus-metro-cabin | common | 567 | +0.00 | +2.82 | +1.23 |
| bus-metro-cabin | stress | 399 | +0.00 | +2.01 | +1.25 |
| cafe | common | 594 | +0.00 | +1.35 | +0.51 |
| cafe | stress | 338 | +0.00 | +3.25 | +1.78 |
| canteen | common | 520 | +0.00 | +5.77 | +0.19 |
| canteen | stress | 289 | +0.00 | +0.35 | -0.35 |
| car-passenger | common | 586 | +0.00 | -1.54 | -2.56 |
| car-passenger | stress | 322 | +0.00 | -9.32 | +2.17 |
| cowork | common | 545 | +0.00 | -0.55 | -0.37 |
| cowork | stress | 362 | +0.00 | +2.49 | +1.66 |
| home-family | common | 555 | +0.00 | -0.18 | +0.90 |
| home-family | stress | 325 | +0.00 | -0.62 | -1.85 |
| home-quiet | common | 534 | +0.00 | +0.19 | +0.37 |
| home-quiet | stress | 340 | +0.00 | +0.00 | +0.88 |
| hotel-room | common | 550 | +0.00 | -0.73 | +0.36 |
| hotel-room | stress | 330 | +0.00 | +0.00 | +0.91 |
| library-classroom | common | 512 | +0.00 | +0.20 | +0.00 |
| library-classroom | stress | 360 | +0.00 | -1.11 | -2.50 |
| meeting-room | common | 582 | +0.00 | +1.20 | +1.20 |
| meeting-room | stress | 333 | +0.00 | +6.01 | +0.60 |
| open-office | common | 549 | +0.00 | +0.55 | -0.36 |
| open-office | stress | 300 | +0.00 | +2.33 | +2.67 |
| phone-booth | common | 552 | +0.00 | +0.18 | +0.36 |
| phone-booth | stress | 305 | +0.00 | +0.33 | -0.00 |
| rail-air-cabin | common | 517 | +0.00 | +1.74 | +2.90 |
| rail-air-cabin | stress | 404 | +0.00 | +2.23 | +0.74 |
| street-outdoor | common | 490 | +0.00 | -0.20 | +0.00 |
| street-outdoor | stress | 418 | +0.00 | +2.15 | -1.67 |
| transit-hub | common | 520 | +0.00 | -1.35 | +0.19 |
| transit-hub | stress | 350 | +0.00 | +6.29 | +5.43 |

### 3d. 按实测信噪比分档：不过段 / 判得了的段

| 项 | 系统 | SNR < 0 dB | SNR 0–5 dB | SNR 5–10 dB | SNR 10–15 dB | SNR 15–20 dB | SNR ≥ 20 dB |
|---|---|---|---|---|---|---|---|
| V-PUMPV | passthrough | 91/95 | 99/144 | 35/108 | 4/53 | 2/39 | 0/41 |
| V-PUMPV | ref | 93/95 | 138/144 | 76/108 | 18/53 | 8/39 | 3/41 |
| V-PUMPV | gtcrn | 94/95 | 139/144 | 96/108 | 39/53 | 25/39 | 17/41 |
| V-TIMBRE | passthrough | 30/89 | 37/144 | 22/106 | 6/45 | 3/37 | 0/41 |
| V-TIMBRE | ref | 31/89 | 38/144 | 23/106 | 5/45 | 3/37 | 0/41 |
| V-TIMBRE | gtcrn | 28/89 | 25/144 | 11/106 | 1/45 | 0/37 | 0/41 |
| V-ONSET | passthrough | 12/92 | 8/135 | 4/89 | 0/42 | 0/35 | 0/39 |
| V-ONSET | ref | 51/92 | 28/135 | 12/89 | 1/42 | 0/35 | 1/39 |
| V-ONSET | gtcrn | 35/92 | 28/135 | 14/89 | 6/42 | 3/35 | 2/39 |
| V-SIM-GATE | passthrough | 95/95 | 144/144 | 103/108 | 46/53 | 33/39 | 39/41 |
| V-SIM-GATE | ref | 71/95 | 71/144 | 39/108 | 10/53 | 5/39 | 14/41 |
| V-SIM-GATE | gtcrn | 56/95 | 45/144 | 36/108 | 22/53 | 12/39 | 22/41 |
| V-SPLVL | passthrough | 0/91 | 0/144 | 0/107 | 0/45 | 0/37 | 0/41 |
| V-SPLVL | ref | 0/91 | 0/144 | 0/107 | 0/45 | 0/37 | 0/41 |
| V-SPLVL | gtcrn | 42/91 | 11/144 | 4/107 | 0/45 | 0/37 | 0/41 |
| V-EAT-ASR | passthrough | 0/95 | 0/144 | 0/108 | 0/53 | 0/39 | 0/41 |
| V-EAT-ASR | ref | 39/95 | 44/144 | 12/108 | 4/53 | 4/39 | 1/41 |
| V-EAT-ASR | gtcrn | 33/95 | 39/144 | 13/108 | 7/53 | 4/39 | 2/41 |
| V-NR | passthrough | 89/89 | 142/142 | 105/105 | 43/43 | 37/37 | 41/41 |
| V-NR | ref | 81/89 | 122/142 | 88/105 | 41/43 | 25/37 | 11/41 |
| V-NR | gtcrn | 55/89 | 60/142 | 43/105 | 19/43 | 5/37 | 5/41 |

### 3d2. 按设备与混响长短：不过段 / 判得了的段

| 项 | 系统 | mba-builtin | external-mic | T60 < 0.3 s | T60 0.3–0.6 s | T60 0.6–1 s | T60 ≥ 1 s |
|---|---|---|---|---|---|---|---|
| V-PUMPV | passthrough | 117/240 | 114/240 | 50/82 | 127/280 | 39/78 | 15/40 |
| V-PUMPV | ref | 174/240 | 162/240 | 76/82 | 173/280 | 60/78 | 27/40 |
| V-PUMPV | gtcrn | 213/240 | 197/240 | 78/82 | 227/280 | 69/78 | 36/40 |
| V-TIMBRE | passthrough | 47/231 | 51/231 | 7/76 | 72/268 | 11/78 | 8/40 |
| V-TIMBRE | ref | 50/231 | 50/231 | 10/76 | 68/268 | 14/78 | 8/40 |
| V-TIMBRE | gtcrn | 32/231 | 33/231 | 8/76 | 49/268 | 5/78 | 3/40 |

### 3e. 门控漏开分布（V-SIM-GATE）

| 系统 | 段数 | 漏开比例 中位 | p90 | 字头漏开 中位 | p90 | 不过段 | 其中主比例超线 | 其中只因字头 |
|---|---|---|---|---|---|---|---|---|
| passthrough | 480 | 0.361 | 0.845 | 0.484 | 1.000 | 460 | 446 | 14 |
| ref | 480 | 0.036 | 0.136 | 0.056 | 0.250 | 210 | 186 | 24 |
| gtcrn | 480 | 0.025 | 0.077 | 0.133 | 0.361 | 193 | 108 | 85 |

### 3f. 拖后腿的声音跨场景汇总（gtcrn；成绩单第五节的分组结果，相关不是因果）

| 项 | 声音（拖累 > 0 的组合数 · 拖累中位数） |
|---|---|
| V-EAT-ASR | `talker` 9/51 · +2.46 · `bed-ace` 7/51 · +0.79 · `footsteps` 5/51 · +3.49 · `bed-room` 4/51 · +1.08 · `brake` 3/51 · +5.29 |
| V-SPLVL | `bed-ace` 4/22 · +0.52 · `talker` 4/22 · +0.25 · `babble` 3/22 · +1.47 · `bed-restaurant` 3/22 · +0.24 · `door-chime` 2/22 · +0.55 |
| V-ONSET | `talker` 6/47 · +0.92 · `keyboard` 4/47 · +1.42 · `nav-voice` 3/47 · +2.98 · `music` 3/47 · +1.98 · `bed-restaurant` 3/47 · +1.92 |
| V-PUMPV | `talker` 12/60 · +0.98 · `bed-ace` 9/60 · +0.99 · `footsteps` 8/60 · +1.13 · `bed-restaurant` 4/60 · +4.21 · `keyboard` 4/60 · +1.59 |
| V-NRMOS-SCOREQ | — |
| V-MUSIC | `talker` 5/24 · +0.20 · `bed-ace` 4/24 · +0.56 · `fan` 4/24 · +0.39 · `nav-voice` 2/24 · +0.98 · `fridge-hum` 2/24 · +0.47 |
| V-NR | `bed-ace` 11/52 · +0.72 · `babble` 6/52 · +1.33 · `bed-cafeteria` 4/52 · +0.59 · `brake` 4/52 · +0.49 · `footsteps` 4/52 · +0.36 |
| V-INTF | `bed-office` 1/40 · +0.44 · `dishes` 1/40 · +0.27 |

## 4. Air 内置麦 vs 外接麦（gtcrn，全部场景合并）

不过段 = 不过 / 判得了的段（「不知道」不进分母）

| 项 | Air 均值 | 外接均值 | 差（Air − 外接） | Air 不过段 | 外接不过段 |
|---|---|---|---|---|---|
| V-NAN | +0.00 | +0.00 | +0.00 | 0/240 | 0/240 |
| V-LAT | +43.67 | +43.67 | +0.00 | 1/240 | 1/240 |
| V-SPLVL | -1.58 | -1.38 | -0.20 | 33/232 | 24/233 |
| V-EAT-ASR | +0.81 | +0.39 | +0.41 | 53/240 | 45/240 |
| V-ONSET | +1.95 | +1.69 | +0.26 | 50/218 | 38/214 |
| V-SIM-GATE | +0.04 | +0.04 | -0.00 | 109/240 | 84/240 |
| V-WEAK | -6.91 | -5.99 | -0.92 | 0/240 | 0/240 |
| V-PUMPV | +6.24 | +5.35 | +0.89 | 213/240 | 197/240 |
| V-ESTOI | +0.02 | +0.02 | +0.00 | 0/240 | 0/240 |
| V-TIMBRE | +2.50 | +2.48 | +0.02 | 32/231 | 33/231 |
| V-NRMOS-SCOREQ | +0.15 | +0.13 | +0.02 | 27/240 | 24/240 |
| V-NRMOS-DISTILL | +0.22 | +0.20 | +0.02 | 0/240 | 0/240 |
| V-MUSIC | +1.60 | +1.51 | +0.09 | 35/144 | 22/144 |
| V-NR | -7.26 | -5.90 | -1.36 | 68/229 | 119/228 |
| V-INTF | -4.31 | -3.78 | -0.53 | 89/90 | 88/88 |
| V-SIM-FA | +0.83 | +0.91 | -0.08 | 38/39 | 40/40 |
| V-BGSTAB | +2.85 | +2.28 | +0.57 | 0/165 | 0/164 |

### 4b. 按设备分开的配对差值（gtcrn − passthrough，扣掉设备链对原声本身的影响）

| 项 | Air：模型 − 原声（段数） | 外接：模型 − 原声（段数） | Air − 外接 |
|---|---|---|---|
| V-NAN | +0.00 (240) | +0.00 (240) | +0.00 |
| V-LAT | +16.00 (240) | +16.00 (240) | +0.00 |
| V-SPLVL | -1.57 (232) | -1.38 (233) | -0.20 |
| V-EAT-ASR | +0.81 (240) | +0.39 (240) | +0.41 |
| V-ONSET | +1.62 (218) | +1.33 (214) | +0.28 |
| V-SIM-GATE | -0.40 (240) | -0.34 (240) | -0.06 |
| V-WEAK | +4.29 (240) | +3.87 (240) | +0.42 |
| V-PUMPV | +2.74 (240) | +1.98 (240) | +0.76 |
| V-ESTOI | +0.02 (240) | +0.02 (240) | +0.00 |
| V-TIMBRE | -0.59 (231) | -0.79 (231) | +0.20 |
| V-NRMOS-SCOREQ | +0.15 (240) | +0.13 (240) | +0.02 |
| V-NRMOS-DISTILL | +0.22 (240) | +0.20 (240) | +0.02 |
| V-MUSIC | +0.60 (144) | +0.51 (144) | +0.09 |
| V-NR | -7.26 (229) | -5.90 (228) | -1.36 |
| V-INTF | -4.31 (90) | -3.78 (88) | -0.53 |
| V-SIM-FA | +0.43 (39) | +0.41 (40) | +0.02 |
| V-BGSTAB | +2.71 (165) | +2.08 (164) | +0.63 |

## 5. Air vs 外接，按场景（gtcrn）

| 场景 | 档 | Air 判定 | Air 不过项数 | 外接判定 | 外接不过项数 | Air 多出的不过项 |
|---|---|---|---|---|---|---|
| bus-metro-cabin | common | fail | 9 | fail | 6 | V-SPLVL V-ONSET V-MUSIC |
| bus-metro-cabin | stress | fail | 8 | fail | 7 | V-SPLVL V-ONSET |
| cafe | common | fail | 6 | fail | 8 | — |
| cafe | stress | fail | 10 | fail | 10 | — |
| canteen | common | fail | 10 | fail | 10 | — |
| canteen | stress | fail | 10 | fail | 10 | — |
| car-passenger | common | fail | 7 | fail | 7 | — |
| car-passenger | stress | fail | 10 | fail | 8 | V-TIMBRE V-NRMOS-SCOREQ |
| cowork | common | fail | 8 | fail | 8 | — |
| cowork | stress | fail | 9 | fail | 9 | — |
| home-family | common | fail | 7 | fail | 5 | V-EAT-ASR V-ONSET |
| home-family | stress | fail | 7 | fail | 7 | V-EAT-ASR |
| home-quiet | common | fail | 7 | fail | 8 | — |
| home-quiet | stress | fail | 8 | fail | 7 | V-TIMBRE |
| hotel-room | common | fail | 5 | fail | 4 | V-MUSIC |
| hotel-room | stress | fail | 6 | fail | 5 | V-TIMBRE V-NRMOS-SCOREQ |
| library-classroom | common | fail | 5 | fail | 5 | V-EAT-ASR |
| library-classroom | stress | fail | 4 | fail | 4 | V-ONSET V-MUSIC |
| meeting-room | common | fail | 5 | fail | 5 | V-NRMOS-SCOREQ |
| meeting-room | stress | fail | 7 | fail | 6 | V-NRMOS-SCOREQ |
| open-office | common | fail | 6 | fail | 8 | — |
| open-office | stress | fail | 7 | fail | 8 | — |
| phone-booth | common | fail | 6 | fail | 7 | — |
| phone-booth | stress | fail | 5 | fail | 6 | V-NRMOS-SCOREQ |
| rail-air-cabin | common | fail | 8 | fail | 9 | — |
| rail-air-cabin | stress | fail | 8 | fail | 9 | — |
| street-outdoor | common | fail | 7 | fail | 8 | — |
| street-outdoor | stress | fail | 9 | fail | 9 | V-MUSIC |
| transit-hub | common | fail | 7 | fail | 9 | — |
| transit-hub | stress | fail | 9 | fail | 9 | — |

## 6. 人声与底噪同房间 vs 不同房间

同房间 180 段 · 不同房间 300 段（gtcrn，两种设备合计）

| 项 | 同房间均值 | 不同房间均值 | 差（同 − 不同） | 同房间不过比例 | 不同房间不过比例 |
|---|---|---|---|---|---|
| V-NAN | +0.00 | +0.00 | +0.00 | 0/180 | 0/300 |
| V-LAT | +43.67 | +43.67 | +0.00 | 2/180 | 0/300 |
| V-SPLVL | -1.25 | -1.62 | +0.37 | 17/180 | 40/285 |
| V-EAT-ASR | +0.29 | +0.79 | -0.49 | 28/180 | 70/300 |
| V-ONSET | +1.80 | +1.83 | -0.03 | 27/154 | 61/278 |
| V-SIM-GATE | +0.04 | +0.04 | +0.00 | 81/180 | 112/300 |
| V-WEAK | -4.50 | -7.62 | +3.12 | 0/180 | 0/300 |
| V-PUMPV | +4.09 | +6.82 | -2.74 | 148/180 | 262/300 |
| V-ESTOI | +0.02 | +0.02 | -0.00 | 0/180 | 0/300 |
| V-TIMBRE | +1.81 | +2.92 | -1.10 | 18/180 | 47/282 |
| V-NRMOS-SCOREQ | +0.16 | +0.13 | +0.03 | 9/180 | 42/300 |
| V-NRMOS-DISTILL | +0.22 | +0.20 | +0.02 | 0/180 | 0/300 |
| V-MUSIC | +1.75 | +1.42 | +0.34 | 33/120 | 24/168 |
| V-NR | -7.04 | -6.28 | -0.76 | 57/180 | 130/277 |
| V-INTF | -4.65 | -3.84 | -0.81 | 46/46 | 131/132 |
| V-SIM-FA | +0.89 | +0.86 | +0.03 | 20/20 | 58/59 |
| V-BGSTAB | +2.80 | +2.46 | +0.34 | 0/101 | 0/228 |

### 6b. 同一场景内比较（gtcrn − passthrough 配对差值；两组各 ≥ 3 段的场景）

| 场景 | 同房间段 | 不同房间段 | V-NR 同 / 不同 | V-PUMPV 同 / 不同 | V-TIMBRE 同 / 不同 | V-EAT-ASR 同 / 不同 | V-NRMOS-SCOREQ 同 / 不同 |
|---|---|---|---|---|---|---|---|
| cowork | 20 | 12 | -5.05 / -4.65 | +2.56 / +1.11 | -0.62 / -0.71 | -0.04 / +1.14 | +0.15 / +0.11 |
| home-quiet | 12 | 20 | -6.80 / -7.55 | +3.07 / +2.15 | -0.66 / -0.74 | +0.18 / +0.77 | +0.22 / +0.12 |
| hotel-room | 16 | 16 | -6.92 / -6.89 | +1.75 / +4.11 | -0.71 / -0.53 | +2.03 / -0.51 | +0.14 / +0.18 |
| library-classroom | 26 | 6 | -7.29 / -7.86 | +1.92 / +0.78 | -1.46 / -2.48 | -0.63 / -0.62 | +0.19 / +0.19 |
| meeting-room | 16 | 16 | -7.44 / -8.12 | +1.84 / +2.02 | -1.54 / -0.37 | +0.81 / +1.55 | +0.10 / +0.10 |
| open-office | 24 | 8 | -6.83 / -4.92 | +1.54 / +0.47 | -2.21 / +0.10 | +1.38 / -0.83 | +0.13 / +0.08 |
| phone-booth | 24 | 8 | -7.73 / -9.72 | +1.12 / +1.33 | +0.20 / +0.80 | +0.13 / +0.46 | +0.21 / +0.12 |
| transit-hub | 10 | 22 | -5.40 / -5.84 | +1.46 / +2.92 | +0.86 / -0.96 | +1.27 / +2.38 | +0.14 / +0.18 |

## 7. 两个评分模型分歧的段（gtcrn，共 55 段，列前 12）

| 段 | 场景 | 档 | SCOREQ Δ | Distill Δ | 相差 | 方向相反 |
|---|---|---|---|---|---|---|
| transit-hub-c03-ext | transit-hub | common | -0.10 | +0.39 | 0.49 | 是 |
| canteen-s05-ext | canteen | stress | -0.16 | +0.24 | 0.39 | 是 |
| canteen-c04-mba | canteen | common | -0.10 | +0.26 | 0.36 | 是 |
| car-passenger-c06-mba | car-passenger | common | -0.14 | +0.21 | 0.36 | 是 |
| car-passenger-s02-mba | car-passenger | stress | -0.03 | +0.32 | 0.36 | 是 |
| canteen-c02-ext | canteen | common | -0.04 | +0.29 | 0.33 | 是 |
| library-classroom-c03-ext | library-classroom | common | -0.14 | +0.19 | 0.33 | 是 |
| home-quiet-c09-ext | home-quiet | common | -0.05 | +0.26 | 0.31 | 是 |
| bus-metro-cabin-s01-ext | bus-metro-cabin | stress | -0.11 | +0.20 | 0.30 | 是 |
| car-passenger-c06-ext | car-passenger | common | -0.05 | +0.25 | 0.30 | 是 |
| phone-booth-s05-mba | phone-booth | stress | -0.05 | +0.24 | 0.29 | 是 |
| hotel-room-s02-mba | hotel-room | stress | -0.02 | +0.27 | 0.28 | 是 |

## 8. V-CLICK 检出最多的段（gtcrn，个/分钟）

- home-family-c00-mba：180
- home-family-c00-ext：110
- transit-hub-c08-ext：110
- home-family-c02-mba：105
- home-family-c08-mba：100

## 9. 不过项最多的段（gtcrn，前 10）

- cafe-s00-mba：8 项（V-SPLVL V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA）
- rail-air-cabin-s01-mba：8 项（V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NRMOS-SCOREQ V-INTF）
- cafe-s00-ext：8 项（V-SPLVL V-EAT-ASR V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA）
- canteen-s00-ext：8 项（V-SPLVL V-EAT-ASR V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA）
- cowork-s01-mba：7 项（V-SPLVL V-EAT-ASR V-PUMPV V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA）
- cowork-s03-mba：7 项（V-SPLVL V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-NR V-INTF V-SIM-FA）
- cowork-s05-mba：7 项（V-SPLVL V-ONSET V-SIM-GATE V-PUMPV V-NR V-INTF V-SIM-FA）
- canteen-c09-mba：7 项（V-SPLVL V-EAT-ASR V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA）
- canteen-s04-mba：7 项（V-SPLVL V-EAT-ASR V-ONSET V-SIM-GATE V-PUMPV V-NRMOS-SCOREQ V-NR）
- open-office-s03-ext：7 项（V-ONSET V-SIM-GATE V-PUMPV V-TIMBRE V-NR V-INTF V-SIM-FA）
