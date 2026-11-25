# 提升空间表（dfn3-nomix 比 gtcrn-h-nomix）

来源：`docs/experiments/exp025-gtcrn-train/baseline/runs/uplift-exp021-dfn3-nomix/compare.json`；算法与阈值见 `gap.py` 文件头。

| 名次 | 场景 | 判定项里「涨」 | 「退」 | 提升分（0 最小 · 1 最大） | 有值项数 | 临时选定 |
|---|---|---|---|---|---|---|
| 1 | cowork | 6 | 1 | 0.715 | 11 |  |
| 2 | car-passenger | 6 | 1 | 0.621 | 12 | 是 |
| 3 | cafe | 6 | 1 | 0.618 | 11 | 是 |
| 4 | home-family | 6 | 2 | 0.601 | 11 |  |
| 5 | street-outdoor | 6 | 1 | 0.528 | 12 | 是 |
| 6 | canteen | 6 | 2 | 0.491 | 11 |  |
| 7 | home-quiet | 5 | 1 | 0.504 | 10 |  |
| 8 | transit-hub | 5 | 3 | 0.467 | 12 | 是 |
| 9 | library-classroom | 4 | 0 | 0.566 | 10 |  |
| 10 | open-office | 4 | 1 | 0.519 | 11 |  |
| 11 | hotel-room | 4 | 2 | 0.427 | 10 |  |
| 12 | rail-air-cabin | 4 | 3 | 0.351 | 12 |  |
| 13 | bus-metro-cabin | 4 | 3 | 0.334 | 12 |  |
| 14 | phone-booth | 3 | 1 | 0.404 | 10 |  |
| 15 | meeting-room | 2 | 2 | 0.351 | 10 |  |

前 4：cowork · car-passenger · cafe · home-family
临时选定：street-outdoor · transit-hub · car-passenger · cafe
重合 2 个（cafe · car-passenger；计入边界并列为 2–2 个），线 3 → **数据不足**（配对段数与基线成绩单不符的场景：bus-metro-cabin（32/288） · cafe（32/288） · canteen（32/288） · car-passenger（32/288） · cowork（32/288） · home-family（32/288） · home-quiet（32/288） · hotel-room（32/288） · library-classroom（32/288） · meeting-room（32/288） · open-office（32/288） · phone-booth（32/288） · rail-air-cabin（32/288） · street-outdoor（32/288） · transit-hub（32/288））

结论（加训哪几个场景）由大总管交人定。
