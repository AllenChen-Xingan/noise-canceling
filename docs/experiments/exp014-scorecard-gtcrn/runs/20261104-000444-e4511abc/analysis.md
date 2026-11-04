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

| 项 | passthrough 不过组合数 | ref 不过组合数 | gtcrn 不过组合数 |
|---|---|---|---|
| V-NAN | 0 / 60 | 0 / 60 | 0 / 60 |
| V-LAT | 2 / 60 | 2 / 60 | 2 / 60 |
| V-SPLVL | 0 / 60 | 0 / 60 | 22 / 60 |
| V-EAT-ASR | 0 / 60 | 39 / 60 | 51 / 60 |
| V-ONSET | 21 / 60 | 40 / 60 | 47 / 60 |
| V-SIM-GATE | 60 / 60 | 56 / 60 | 59 / 60 |
| V-WEAK | 0 / 60 | 0 / 60 | 0 / 60 |
| V-PUMPV | 48 / 60 | 58 / 60 | 60 / 60 |
| V-CLICK（只报不判） | 0 / 60 | 59 / 60 | 59 / 60 |
| V-ESTOI | 0 / 60 | 0 / 60 | 0 / 60 |
| V-TIMBRE | 46 / 60 | 45 / 60 | 34 / 60 |
| V-NRMOS-SCOREQ | 0 / 60 | 36 / 60 | 30 / 60 |
| V-NRMOS-DISTILL | 0 / 60 | 9 / 60 | 0 / 60 |
| V-MUSIC | 0 / 60 | 20 / 60 | 24 / 60 |
| V-NR | 60 / 60 | 60 / 60 | 52 / 60 |
| V-INTF | 40 / 60 | 40 / 60 | 40 / 60 |
| V-SIM-FA | 18 / 60 | 20 / 60 | 20 / 60 |
| V-BGSTAB | 0 / 60 | 0 / 60 | 0 / 60 |

## 3. gtcrn：模型 − 原声直通（同一批段配对）

| 项 | 好的方向 | 配对段数 | 均值差（模型 − 原声直通） |
|---|---|---|---|
| V-NAN | ? | 480 | +0.00 |
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

## 4. Air 内置麦 vs 外接麦（gtcrn，全部场景合并）

| 项 | Air 均值 | 外接均值 | 差（Air − 外接） | Air 不过段 | 外接不过段 |
|---|---|---|---|---|---|
| V-NAN | +0.00 | +0.00 | +0.00 | 0 / 240 | 0 / 240 |
| V-LAT | +43.67 | +43.67 | +0.00 | 1 / 240 | 1 / 240 |
| V-SPLVL | -1.58 | -1.38 | -0.20 | 33 / 240 | 24 / 240 |
| V-EAT-ASR | +0.81 | +0.39 | +0.41 | 53 / 240 | 45 / 240 |
| V-ONSET | +1.95 | +1.69 | +0.26 | 50 / 240 | 38 / 240 |
| V-SIM-GATE | +0.04 | +0.04 | -0.00 | 109 / 240 | 84 / 240 |
| V-WEAK | -6.91 | -5.99 | -0.92 | 0 / 240 | 0 / 240 |
| V-PUMPV | +6.24 | +5.35 | +0.89 | 213 / 240 | 197 / 240 |
| V-ESTOI | +0.02 | +0.02 | +0.00 | 0 / 240 | 0 / 240 |
| V-TIMBRE | +2.50 | +2.48 | +0.02 | 32 / 240 | 33 / 240 |
| V-NRMOS-SCOREQ | +0.15 | +0.13 | +0.02 | 27 / 240 | 24 / 240 |
| V-NRMOS-DISTILL | +0.22 | +0.20 | +0.02 | 0 / 240 | 0 / 240 |
| V-MUSIC | +1.60 | +1.51 | +0.09 | 35 / 240 | 22 / 240 |
| V-NR | -7.26 | -5.90 | -1.36 | 68 / 240 | 119 / 240 |
| V-INTF | -4.31 | -3.78 | -0.53 | 89 / 240 | 88 / 240 |
| V-SIM-FA | +0.83 | +0.91 | -0.08 | 38 / 240 | 40 / 240 |
| V-BGSTAB | +2.85 | +2.28 | +0.57 | 0 / 240 | 0 / 240 |

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
| V-SPLVL | -1.25 | -1.62 | +0.37 | 17/180 | 40/300 |
| V-EAT-ASR | +0.29 | +0.79 | -0.49 | 28/180 | 70/300 |
| V-ONSET | +1.80 | +1.83 | -0.03 | 27/180 | 61/300 |
| V-SIM-GATE | +0.04 | +0.04 | +0.00 | 81/180 | 112/300 |
| V-WEAK | -4.50 | -7.62 | +3.12 | 0/180 | 0/300 |
| V-PUMPV | +4.09 | +6.82 | -2.74 | 148/180 | 262/300 |
| V-ESTOI | +0.02 | +0.02 | -0.00 | 0/180 | 0/300 |
| V-TIMBRE | +1.81 | +2.92 | -1.10 | 18/180 | 47/300 |
| V-NRMOS-SCOREQ | +0.16 | +0.13 | +0.03 | 9/180 | 42/300 |
| V-NRMOS-DISTILL | +0.22 | +0.20 | +0.02 | 0/180 | 0/300 |
| V-MUSIC | +1.75 | +1.42 | +0.34 | 33/180 | 24/300 |
| V-NR | -7.04 | -6.28 | -0.76 | 57/180 | 130/300 |
| V-INTF | -4.65 | -3.84 | -0.81 | 46/180 | 131/300 |
| V-SIM-FA | +0.89 | +0.86 | +0.03 | 20/180 | 58/300 |
| V-BGSTAB | +2.80 | +2.46 | +0.34 | 0/180 | 0/300 |

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
