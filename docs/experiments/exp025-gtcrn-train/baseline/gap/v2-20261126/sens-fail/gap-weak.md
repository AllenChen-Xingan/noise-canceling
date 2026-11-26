# 场景差距表（gtcrn）

来源：`<主目录>/_tmp/20261126-gap-v2/sens/fail/card/scorecard.json`；算法与阈值见 `gap.py` 文件头。

| 名次 | 场景 | 不过的（项, 切片）数 | 弱点分（0 最好 · 1 最差） | 有值格数 | 临时选定 |
|---|---|---|---|---|---|
| 1 | rail-air-cabin | 15 | 0.599 | 60 |  |
| 2 | canteen | 13 | 0.676 | 56 |  |
| 3 | meeting-room | 12 | 0.654 | 52 |  |
| 4 | transit-hub | 11 | 0.477 | 60 | 是 |
| 5 | car-passenger | 11 | 0.445 | 60 | 是 |
| 6 | bus-metro-cabin | 10 | 0.497 | 60 |  |
| 7 | open-office | 10 | 0.487 | 56 |  |
| 8 | cowork | 9 | 0.592 | 56 |  |
| 9 | home-family | 9 | 0.456 | 56 |  |
| 10 | cafe | 8 | 0.479 | 56 | 是 |
| 11 | street-outdoor | 8 | 0.399 | 60 | 是 |
| 12 | phone-booth | 7 | 0.488 | 52 |  |
| 13 | library-classroom | 6 | 0.489 | 52 |  |
| 14 | home-quiet | 5 | 0.383 | 52 |  |
| 15 | hotel-room | 5 | 0.376 | 52 |  |

前 4：rail-air-cabin · canteen · meeting-room · transit-hub
临时选定：street-outdoor · transit-hub · car-passenger · cafe
重合 1 个（transit-hub；计入边界并列为 1–1 个），线 3 → **不一致**

结论（加训哪几个场景）由大总管在看过本表后下。
