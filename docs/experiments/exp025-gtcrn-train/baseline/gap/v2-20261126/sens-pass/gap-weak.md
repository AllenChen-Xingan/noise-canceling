# 场景差距表（gtcrn）

来源：`<主目录>/_tmp/20261126-gap-v2/sens/pass/card/scorecard.json`；算法与阈值见 `gap.py` 文件头。

| 名次 | 场景 | 不过的（项, 切片）数 | 弱点分（0 最好 · 1 最差） | 有值格数 | 临时选定 |
|---|---|---|---|---|---|
| 1 | rail-air-cabin | 14 | 0.579 | 60 |  |
| 2 | canteen | 13 | 0.676 | 56 |  |
| 3 | meeting-room | 11 | 0.655 | 52 |  |
| 4 | transit-hub | 11 | 0.483 | 60 | 是 |
| 5 | car-passenger | 11 | 0.448 | 60 | 是 |
| 6 | bus-metro-cabin | 10 | 0.486 | 60 |  |
| 7 | cowork | 8 | 0.588 | 56 |  |
| 8 | open-office | 8 | 0.488 | 56 |  |
| 9 | cafe | 8 | 0.488 | 56 | 是 |
| 10 | street-outdoor | 8 | 0.407 | 60 | 是 |
| 11 | phone-booth | 7 | 0.490 | 52 |  |
| 12 | home-family | 7 | 0.452 | 56 |  |
| 13 | library-classroom | 6 | 0.492 | 52 |  |
| 14 | home-quiet | 5 | 0.385 | 52 |  |
| 15 | hotel-room | 5 | 0.382 | 52 |  |

前 4：rail-air-cabin · canteen · meeting-room · transit-hub
临时选定：street-outdoor · transit-hub · car-passenger · cafe
重合 1 个（transit-hub；计入边界并列为 1–1 个），线 3 → **不一致**

结论（加训哪几个场景）由大总管在看过本表后下。
