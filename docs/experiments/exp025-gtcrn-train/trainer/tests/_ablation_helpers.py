"""两份消融测试文件共用的常量与助手（`test_ablation.py` · `test_ablation_judge.py`）。

只放两边都要用的那一份，各文件自己的助手留在各自文件里（不抄副本）。
"""

# 基线配方：相对 trainer/（train.py 在那里跑）。两份测试文件都用它当 data.recipe / data.dev_recipe。
BASE_RECIPE = "../data/recipe.json"
