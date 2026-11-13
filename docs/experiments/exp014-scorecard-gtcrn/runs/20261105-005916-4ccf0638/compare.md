# exp014 新规则重跑对照（v2，人定 D85）

旧 = `20261104-000444-e4511abc`（旧规则 v1，原表）；新 = `20261105-005916-4ccf0638`（新规则 v2，重跑音频）。每格是 60 个组合的 过 / 不过 / 待测。新增项旧表没有，记 0 / 0 / 0。

新 run 没跑成的段：0；段数：1440；元数据：`{"commit": "4ccf0638", "power": "ac", "qa": true, "rules": "v2", "run_id": "20261105-005916-4ccf0638", "suite": "src/eval/suite/scenes.json", "systems": ["passthrough", "ref", "gtcrn"]}`

| 项 | 新判法 | passthrough 旧 | passthrough 新 | gtcrn 旧 | gtcrn 新 | ref 旧 | ref 新 |
|---|---|---|---|---|---|---|---|
| V-NAN | 逐段：物理硬事实，任一段不过 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 |
| V-LAT | 逐段：物理硬事实，任一段不过 | 58 / 2 / 0 | 60 / 0 / 0 | 58 / 2 / 0 | 60 / 0 / 0 | 58 / 2 / 0 | 60 / 0 / 0 |
| V-SPLVL | 均值 + 95% 区间三态， | 60 / 0 / 0 | 60 / 0 / 0 | 38 / 22 / 0 | 58 / 0 / 2 | 60 / 0 / 0 | 60 / 0 / 0 |
| V-EAT-ASR | 逐段：比原声多错 ≥ 5 字 | 60 / 0 / 0 | 60 / 0 / 0 | 9 / 51 / 0 | 49 / 11 / 0 | 21 / 39 / 0 | 51 / 9 / 0 |
| V-ONSET | 均值 + 95% 区间三态， | 39 / 21 / 0 | 54 / 0 / 6 | 13 / 47 / 0 | 27 / 0 / 33 | 20 / 40 / 0 | 31 / 0 / 29 |
| V-SIM-GATE | 只报不判（人定 D85） | 0 / 60 / 0 | 0 / 58 / 2 | 1 / 59 / 0 | 30 / 2 / 28 | 4 / 56 / 0 | 21 / 16 / 23 |
| V-WEAK | 均值 + 95% 区间三态， | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 |
| V-PUMPV | 均值 + 95% 区间三态， | 12 / 48 / 0 | 60 / 0 / 0 | 0 / 60 / 0 | 17 / 1 / 42 | 2 / 58 / 0 | 60 / 0 / 0 |
| V-CLICK | 只报不判（人定 D85） | 60 / 0 / 0 | 60 / 0 / 0 | 1 / 59 / 0 | 1 / 59 / 0 | 1 / 59 / 0 | 1 / 58 / 1 |
| V-ESTOI | 均值 + 95% 区间三态， | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 | 60 / 0 / 0 |
| V-TIMBRE | 均值 + 95% 区间三态， | 14 / 46 / 0 | 60 / 0 / 0 | 26 / 34 / 0 | 60 / 0 / 0 | 15 / 45 / 0 | 60 / 0 / 0 |
| V-VACUUM | 只报不判（人定 D85） | 0 / 0 / 0 | 60 / 0 / 0 | 0 / 0 / 0 | 60 / 0 / 0 | 0 / 0 / 0 | 60 / 0 / 0 |
| V-TSOS | 只报不判（人定 D85） | 0 / 0 / 0 | 0 / 0 / 60 | 0 / 0 / 0 | 0 / 0 / 60 | 0 / 0 / 0 | 0 / 0 / 60 |
| V-SWALLOW | 只报不判（人定 D85） | 0 / 0 / 0 | 0 / 0 / 60 | 0 / 0 / 0 | 0 / 0 / 60 | 0 / 0 / 0 | 0 / 0 / 60 |
| V-MUSIC | 均值 + 95% 区间三态， | 36 / 0 / 0 | 36 / 0 / 0 | 12 / 24 / 0 | 26 / 0 / 10 | 16 / 20 / 0 | 18 / 0 / 18 |
| V-NR | 均值 + 95% 区间三态， | 0 / 60 / 0 | 0 / 60 / 0 | 8 / 52 / 0 | 22 / 9 / 29 | 0 / 60 / 0 | 2 / 38 / 20 |
| V-INTF | 均值 + 95% 区间三态， | 0 / 40 / 0 | 0 / 38 / 2 | 0 / 40 / 0 | 0 / 38 / 2 | 0 / 40 / 0 | 0 / 38 / 2 |
| V-SIM-FA | 只报不判（人定 D85） | 2 / 18 / 0 | 1 / 12 / 7 | 0 / 20 / 0 | 0 / 18 / 2 | 0 / 20 / 0 | 0 / 19 / 1 |
| V-BGSTAB | 均值 + 95% 区间三态， | 44 / 0 / 0 | 44 / 0 / 0 | 44 / 0 / 0 | 44 / 0 / 0 | 44 / 0 / 0 | 44 / 0 / 0 |
| V-LEAK | 只报不判（人定 D85） | 0 / 0 / 0 | 0 / 0 / 40 | 0 / 0 / 0 | 0 / 0 / 40 | 0 / 0 / 0 | 0 / 0 / 40 |
| V-PSE-OVL | 只报不判（人定 D85） | 0 / 0 / 0 | 0 / 0 / 40 | 0 / 0 / 0 | 0 / 0 / 40 | 0 / 0 / 0 | 0 / 0 / 40 |
| 听感 |  | 60 / 0 / 0 | 60 / 0 / 0 | 30 / 0 / 30 | 60 / 0 / 0 | 22 / 5 / 33 | 59 / 0 / 1 |
| （场景） |  | 0 / 60 / 0 | 0 / 60 / 0 | 0 / 60 / 0 | 0 / 40 / 20 | 0 / 60 / 0 | 0 / 50 / 10 |

## B3 背景稳不稳：改前（旧主值）vs 改后（扣输入侧偏差）

| 系统 | 段数 | 旧主值均值 | 新 run 的 uncorr_db 均值 | 与旧主值逐段最大差 | 改后主值均值 | 改后最大 |
|---|---|---|---|---|---|---|
| passthrough | 329 | 0.173 | 0.173 | 0.00e+00 | 0.000 | 0.000 |
| gtcrn | 329 | 2.569 | 2.569 | 0.00e+00 | 2.531 | 7.934 |
| ref | 329 | 1.279 | 1.279 | 0.00e+00 | 1.234 | 4.897 |

## 新项与干净人声那一路的基线（逐段均值 · 有值段数）

| 系统 | V-TSOS | V-SWALLOW | V-LEAK | V-PSE-OVL | V-PUMPV | V-TIMBRE | V-WEAK | V-LAT |
|---|---|---|---|---|---|---|---|---|
| passthrough | 0.001（480） | 0.000（432） | 100.000（100） | 0.000（242） | 0.000（480） | 0.000（480） | 0.000（480） | 27.667（480） |
| gtcrn | 4.717（480） | 7.904（432） | 97.650（100） | 1.230（242） | 1.066（480） | 0.181（480） | 1.591（480） | 43.667（480） |
| ref | 1.499（480） | 4.600（432） | 100.000（100） | 1.103（242） | 0.000（480） | 0.000（480） | 0.000（480） | 43.667（480） |

同时说话只留用户：失败率 = 1 的段数 {'passthrough': 242, 'ref': 97, 'gtcrn': 68}

## 场景总判的不过原因（判定项里不过的组合，只报项不算）

| 系统 | 场景总判 | 不过的判定项 | 组合数 |
|---|---|---|---|
| passthrough | fail | V-INTF + V-NR | 38 |
| passthrough | fail | V-NR | 22 |
| gtcrn | fail | V-INTF | 22 |
| gtcrn | unknown | — | 20 |
| gtcrn | fail | V-INTF + V-NR | 7 |
| gtcrn | fail | V-EAT-ASR + V-INTF | 7 |
| gtcrn | fail | V-EAT-ASR + V-INTF + V-NR | 2 |
| gtcrn | fail | V-EAT-ASR + V-PUMPV | 1 |
| gtcrn | fail | V-EAT-ASR | 1 |
| ref | fail | V-INTF + V-NR | 23 |
| ref | unknown | — | 10 |
| ref | fail | V-NR | 9 |
| ref | fail | V-INTF | 9 |
| ref | fail | V-EAT-ASR + V-INTF + V-NR | 4 |
| ref | fail | V-EAT-ASR + V-INTF | 2 |
| ref | fail | V-EAT-ASR + V-NR | 2 |
| ref | fail | V-EAT-ASR | 1 |

## 吃字 · 人声忽大忽小判不过的组合（参考降噪与 GTCRN）

| 系统 | 场景 | 档 | 设备 | 项 | 越线段 / 段数 | 最差段 | 均值（95% 区间） |
|---|---|---|---|---|---|---|---|
| gtcrn | open-office | stress | mba-builtin | V-EAT-ASR | 2 / 6 | 33.3（`open-office-s02-mba`） | 8.21（-0.79 – 19.35） |
| ref | open-office | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 33.3（`open-office-s02-mba`） | 5.78（-1.08 – 17.42） |
| ref | meeting-room | common | mba-builtin | V-EAT-ASR | 1 / 10 | 30.8（`meeting-room-c04-mba`） | 4.81（0.00 – 11.34） |
| gtcrn | meeting-room | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 19.2（`meeting-room-s01-mba`） | 3.21（0.00 – 9.62） |
| gtcrn | meeting-room | stress | mba-builtin | V-PUMPV | 5 / 6 | 2.9（`meeting-room-s03-mba`） | 1.95（1.43 – 2.48） |
| ref | meeting-room | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 46.2（`meeting-room-s01-mba`） | 9.90（0.60 – 24.69） |
| gtcrn | cafe | stress | external-mic | V-EAT-ASR | 1 / 6 | 14.0（`cafe-s02-ext`） | 3.38（-0.97 – 8.03） |
| ref | canteen | common | external-mic | V-EAT-ASR | 1 / 10 | 22.2（`canteen-c04-ext`） | 3.83（-1.04 – 9.30） |
| ref | canteen | common | mba-builtin | V-EAT-ASR | 1 / 10 | 37.0（`canteen-c04-mba`） | 7.84（2.45 – 15.16） |
| gtcrn | canteen | stress | external-mic | V-EAT-ASR | 1 / 6 | 22.7（`canteen-s04-ext`） | 1.72（-7.72 – 11.36） |
| gtcrn | transit-hub | stress | external-mic | V-EAT-ASR | 1 / 6 | 21.9（`transit-hub-s01-ext`） | 6.70（0.49 – 13.99） |
| ref | transit-hub | stress | external-mic | V-EAT-ASR | 2 / 6 | 23.1（`transit-hub-s00-ext`） | 8.44（0.88 – 16.47） |
| gtcrn | transit-hub | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 18.8（`transit-hub-s01-mba`） | 3.85（-2.06 – 10.76） |
| ref | transit-hub | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 21.9（`transit-hub-s01-mba`） | 3.95（-1.96 – 11.24） |
| gtcrn | car-passenger | stress | external-mic | V-EAT-ASR | 1 / 6 | 34.8（`car-passenger-s02-ext`） | 5.73（-2.36 – 17.99） |
| gtcrn | bus-metro-cabin | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 25.0（`bus-metro-cabin-s03-mba`） | 4.68（-1.52 – 13.40） |
| gtcrn | rail-air-cabin | common | external-mic | V-EAT-ASR | 1 / 10 | 20.0（`rail-air-cabin-c09-ext`） | 3.57（-1.51 – 9.01） |
| gtcrn | rail-air-cabin | common | mba-builtin | V-EAT-ASR | 1 / 10 | 19.2（`rail-air-cabin-c08-mba`） | 2.83（-1.88 – 8.49） |
| ref | rail-air-cabin | common | mba-builtin | V-EAT-ASR | 1 / 10 | 19.2（`rail-air-cabin-c08-mba`） | 2.88（-2.83 – 8.77） |
| ref | street-outdoor | stress | mba-builtin | V-EAT-ASR | 1 / 6 | 14.7（`street-outdoor-s03-mba`） | 4.50（1.01 – 9.37） |
| gtcrn | hotel-room | common | mba-builtin | V-EAT-ASR | 1 / 10 | 13.9（`hotel-room-c06-mba`） | 0.57（-2.45 – 4.17） |

## 场景总判：只重新汇总（`20261104-231355-da5821a4-resum-v2`）→ 重跑，变了的组合

| 系统 | 场景 | 档 | 设备 | 总判 | 判定项变化 | V-PUMPV 均值（95% 区间） |
|---|---|---|---|---|---|---|
| gtcrn | phone-booth | common | external-mic | fail → unknown | V-LAT fail → pass；V-WEAK pass（只报） → pass；V-PUMPV unknown（只报） → unknown；V-TIMBRE pass（只报） → pass | 1.03（0.65 – 1.48） |
| ref | phone-booth | common | external-mic | fail → unknown | V-LAT fail → pass；V-WEAK pass（只报） → pass；V-PUMPV pass（只报） → pass；V-TIMBRE pass（只报） → pass | 0.00（0.00 – 0.00） |
| gtcrn | phone-booth | common | mba-builtin | fail → unknown | V-LAT fail → pass；V-WEAK pass（只报） → pass；V-PUMPV unknown（只报） → unknown；V-TIMBRE pass（只报） → pass | 1.37（0.71 – 2.19） |
| ref | phone-booth | common | mba-builtin | fail → unknown | V-LAT fail → pass；V-WEAK pass（只报） → pass；V-PUMPV pass（只报） → pass；V-TIMBRE pass（只报） → pass | 0.00（0.00 – 0.00） |
| gtcrn | library-classroom | common | mba-builtin | pass → unknown | V-WEAK pass（只报） → pass；V-PUMPV unknown（只报） → unknown；V-TIMBRE pass（只报） → pass | 1.28（0.57 – 2.15） |
