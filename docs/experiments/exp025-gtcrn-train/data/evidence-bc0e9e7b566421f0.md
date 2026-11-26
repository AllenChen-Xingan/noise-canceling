# 训练前清单第 3 条的证据 · 基准 build bc0e9e7b566421f0

本文件由 `build_evidence.py` 从 `builds/<id>/build.json` · 配方文件 · 切分表算出（重算即得，不手抄）：`python3 build_evidence.py` 打标准输出，`--check <本文件>` 逐字节核对。「排除表」一列是 build 记录里 `leak_check.against` 的名字；build 记录不存它的哈希，这里也不编一个。

| build | 配方 | 配方哈希（前 12 位） | 交集检查的来源数 | 排除表 | id 可重算 | valid | 交集通过 | 命中 0 | DEMAND 0 | missing 空 | 切分表哈希 | 代码哈希 | 目标场景 | 只差一个旋钮 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| bc0e9e7b566421f0 | recipe.json | e1fe1bbce425 | 9 | exp013 build 20261103230756-17f5 exclude.json | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 574ef5c7353859d2 | recipes/ablation/no-eq.json | 3bc7355dd1b0 | 9 | exp013 build 20261103230756-17f5 exclude.json | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 748aee58bdd68266 | recipes/ablation/no-clip.json | 0a708353e785 | 9 | exp013 build 20261103230756-17f5 exclude.json | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 36ea9561adbb7d7f | recipes/ablation/no-n_noises.json | be7a23094d45 | 9 | exp013 build 20261103230756-17f5 exclude.json | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 966a229da3abed21 | recipes/ablation/no-reverb.json | 0e9f4909f3c6 | 9 | exp013 build 20261103230756-17f5 exclude.json | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 10ebb73bf735f7ab | recipes/ablation/high_snr-on.json | 92a96ef6d453 | 9 | exp013 build 20261103230756-17f5 exclude.json | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |

全部通过：是
