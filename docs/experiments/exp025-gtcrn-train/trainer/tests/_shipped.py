"""仓里真正在用的那批数据：shipped 配置必须钉的 build 与分片目录（测试里只此一处）。

这里不是真相源，只把真相源指过来的一处汇总：
- 数据 build 的正本是数据线登记的那份记录 data/builds/<id>/build.json（`valid` 为真、配方 sha256 与
  data/recipe.json 一致）；
- 分片目录的正本是打包清单 data/cloud-batch5.md 第 24 行的 VDN_SHARDS。
改批次时这两处一起改，三份测试（test_buildpin.py · test_formal.py · test_ablation.py）跟着走。
"""

# 第五批基线 build：正本 data/builds/df759bb8d606a209/build.json
SHIPPED_BUILD = "df759bb8d606a209"

# 第五批分片的云盘目录：正本 data/cloud-batch5.md 第 24 行（VDN_SHARDS=/workspace/shards/batch5）
SHIPPED_SHARDS = "/workspace/shards/batch5"
