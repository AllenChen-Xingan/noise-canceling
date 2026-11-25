# 场景差距表（gtcrn）

来源：`_tmp/exp025-smoke-20261124-191531/merged-card/scorecard.json`；算法与阈值见 `gap.py` 文件头。

| 名次 | 场景 | 不过的（项, 切片）数 | 弱点分（0 最好 · 1 最差） | 有值格数 | 临时选定 |
|---|---|---|---|---|---|
| 1 | transit-hub | 0 | 0.583 | 12 | 是 |
| 2 | car-passenger | 0 | 0.500 | 13 | 是 |
| 3 | street-outdoor | 0 | 0.500 | 12 | 是 |
| 4 | home-quiet | 0 | 0.477 | 11 |  |
| 5 | cafe | 0 | 0.432 | 11 | 是 |

前 4：transit-hub · car-passenger · street-outdoor · home-quiet
临时选定：street-outdoor · transit-hub · car-passenger · cafe
重合 3 个（car-passenger · street-outdoor · transit-hub；计入边界并列为 3–3 个），线 3 → **数据不足**（参与排名的场景只有 5 个 < 15；缺切片的场景：cafe · car-passenger · home-quiet · street-outdoor · transit-hub）

结论（加训哪几个场景）由大总管在看过本表后下。
