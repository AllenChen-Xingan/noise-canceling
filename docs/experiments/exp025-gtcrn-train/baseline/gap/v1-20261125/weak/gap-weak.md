# 场景差距表（gtcrn）

来源：`<主目录>/_tmp/20261125-gap-v1/weak/scorecard-no-eat.json`；算法与阈值见 `gap.py` 文件头。

| 名次 | 场景 | 不过的（项, 切片）数 | 弱点分（0 最好 · 1 最差） | 有值格数 | 临时选定 |
|---|---|---|---|---|---|
| 1 | rail-air-cabin | 11 | 0.585 | 56 |  |
| 2 | canteen | 9 | 0.657 | 52 |  |
| 3 | transit-hub | 9 | 0.468 | 56 | 是 |
| 4 | meeting-room | 8 | 0.657 | 48 |  |
| 5 | phone-booth | 7 | 0.523 | 48 |  |
| 6 | open-office | 7 | 0.486 | 52 |  |
| 7 | home-family | 7 | 0.460 | 52 |  |
| 8 | car-passenger | 7 | 0.428 | 56 | 是 |
| 9 | bus-metro-cabin | 6 | 0.494 | 56 |  |
| 10 | cowork | 5 | 0.570 | 52 |  |
| 11 | library-classroom | 5 | 0.518 | 48 |  |
| 12 | home-quiet | 5 | 0.403 | 48 |  |
| 13 | street-outdoor | 5 | 0.397 | 56 | 是 |
| 14 | hotel-room | 5 | 0.396 | 48 |  |
| 15 | cafe | 4 | 0.469 | 52 | 是 |

前 4：rail-air-cabin · canteen · transit-hub · meeting-room
临时选定：street-outdoor · transit-hub · car-passenger · cafe
重合 1 个（transit-hub；计入边界并列为 1–1 个），线 3 → **不一致**

结论（加训哪几个场景）由大总管在看过本表后下。
