# 训练前清单第 3 条的证据 · 基准 build bc0e9e7b566421f0

本文件由 `build_evidence.py` 从 `builds/<id>/build.json` · 配方文件 · 切分表算出（重算即得，不手抄）：`python3 build_evidence.py` 打标准输出，`--check <本文件>` 逐字节核对。「排除表」一列先是 build 记录里 `leak_check.against` 的名字，后面是本机 `leakcheck.EXCLUDE` 这份 exclude.json 的 sha256 前 12 位（`b7292768d7f9`）；build 记录不存哈希，所以它只说明这次重算用的是哪份表。「交集重算一致」一列按记录里列出的来源重跑一遍 `leakcheck.check`，再与记录的那份逐键比；「交集来源齐全」一列核记录里列的来源正好是 `split_rules.json` 的全部来源、用到的切分表都在其中。「目标场景（build 记录）」一列读的是 build 记录里的 `noise_scenes`：分片索引在云上，本机只能核记录。

| build | 配方 | 配方哈希（前 12 位） | 交集检查的来源数 | 排除表 | id 可重算 | valid | 交集通过 | 命中 0 | DEMAND 0 | 交集重算一致 | 交集来源齐全 | missing 空 | 切分表哈希 | 代码哈希 | 目标场景（build 记录） | 只差一个旋钮 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| bc0e9e7b566421f0 | recipe.json | e1fe1bbce425 | 9 | exp013 build 20261103230756-17f5 exclude.json（本机这份 sha256 b7292768d7f9） | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 574ef5c7353859d2 | recipes/ablation/no-eq.json | 3bc7355dd1b0 | 9 | exp013 build 20261103230756-17f5 exclude.json（本机这份 sha256 b7292768d7f9） | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 748aee58bdd68266 | recipes/ablation/no-clip.json | 0a708353e785 | 9 | exp013 build 20261103230756-17f5 exclude.json（本机这份 sha256 b7292768d7f9） | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 36ea9561adbb7d7f | recipes/ablation/no-n_noises.json | be7a23094d45 | 9 | exp013 build 20261103230756-17f5 exclude.json（本机这份 sha256 b7292768d7f9） | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 966a229da3abed21 | recipes/ablation/no-reverb.json | 0e9f4909f3c6 | 9 | exp013 build 20261103230756-17f5 exclude.json（本机这份 sha256 b7292768d7f9） | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |
| 10ebb73bf735f7ab | recipes/ablation/high_snr-on.json | 92a96ef6d453 | 9 | exp013 build 20261103230756-17f5 exclude.json（本机这份 sha256 b7292768d7f9） | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 | 是 |

全部通过：是
