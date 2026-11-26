# 评审记录：数据消融 7 组（codex，read-only，effort medium）

范围 `f1efacbc..HEAD`（分支 feat/trainer-ablation-scenes，issue #212）。重点四条：
1. `data.dev_recipe` 不设时行为不变
2. judge 与 RULE 逐字一致
3. 变体配方的核对
4. 7 条命令

代码由 pi 写：`trainer-ablation` 写了 ad5980f6..baf24af4；`trainer-ablation-fix1` 写了 23d1b4db、1a07d666，是 trainer 验收时发现的两处。

## 验收前 trainer 自查（2026-11-26）

- 发现：基线在 judge 里被判不完整时，它的读数仍进 base / floor，旋钮照判。pi 修于 23d1b4db：先写测试，在旧代码上 3 败。
- 发现：judge 只比数据字段，模型、损失、步数不同的运行也会被判。pi 修于 1a07d666：先写测试，在旧代码上 4 败。
- 交叉核对 traindata 的真变体：用 `feat/data-batch3`（e8728ffe）的 5 个文件跑 `ablation.py check`，只剩 `data.shard_root` 占位一条。

## 第 1 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×2 P2×2。逐条核实，4 条都属实。
- **P1｜配置比较漏项**（第 1 轮 #1）：只比四个配置段和 max_steps，device、其他 schedule 字段、limits、run.out_dir 不同仍可判。改：比整份配置，只排除 run.name、run.seed、data.recipe。交 pi。
- **P1｜不查固定设计**（第 1 轮 #2）：7 份记录只互相比，大家都用 B / 5000 步也能过；测试的成功样例也是 B / 5000。改：每份必须是模型 A、1000 步，并设了 `data.dev_recipe`；测试默认值改成 A / 1000。交 pi。
- **P2｜不设 dev_recipe 时 run.json 多两个字段**（第 1 轮 #3）：不进停止线，开 #217。
- **P2｜metrics.jsonl 有非对象行时 judge 崩溃**（第 1 轮 #4）：改动小，与 P1 一起交 pi。
- 抽查引用中「不符」1 条：`test_dev_recipe_unset_keeps_todays_behaviour` 没证明 run.json 原样。与 #3 同源，记入 #217。

### 第 1 轮的处理（2026-11-26）

由 pi `trainer-ablation-fix2` 修，提交 45aae59b..025e1dbe，花费 0.0216 美元。
- 改动：judge 比整份配置，例外只有 run.name、run.seed、data.recipe（常量 `CFG_IGNORE`），缺键也算不同。每个运行先查设计固定项：模型 A、1000 步、`data.dev_recipe` 等于基线配方；基线不合格时每个旋钮都判 incomplete。metrics 有非对象行时，判该运行 incomplete，不再崩溃。
- 红灯：trainer 把新测试放到旧代码 1a07d666 上跑（测试里的新常量换成字面值，免得只因缺常量而红），13 败 46 过，都是新加的用例。
- 绿灯：合入后 trainer 亲手跑 test_ablation + test_formal + test_data + test_train，108 过 2 跳过；ablation.py 467 行，测试 585 行。

## 第 2 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×1 P2×2。逐条核实，3 条都属实。
- **P1｜judge 不核对运行的实际种子**（第 2 轮 #1）：两条基线都误用种子 0 时，噪声底失去意义，旋钮仍照判。trainer 补一条同类检查：每个运行的 data.recipe 必须是名字对应的那份变体。交 pi `trainer-ablation-fix3`。
- **P2｜缺键与 null 不分，空对象被拍平丢掉**（第 2 轮 #2）：改动小，一起交 pi。
- **P2｜不设 dev_recipe 时 run.json 多两个字段**（第 2 轮 #3）：同第 1 轮 #3，仍留在 #217。不进停止线，因为它不影响判结果，也不影响开训。
- 先拆测试文件：test_ablation.py 已 585 行，再加就超 600 行。

### 第 2 轮的处理（2026-11-26）

由 pi `trainer-ablation-fix3` 修，提交 d7c87bd5..51e42e1c，花费 0.0282 美元。
- 先拆测试：judge 的测试移到 test_ablation_judge.py，共用的部分放 _ablation_helpers.py。拆后仍是 59 过。
- 运行身份：run.seed 必须等于运行矩阵给这个名字的种子；data.recipe 必须是名字对应的那份（基线 = 基线配方，旋钮 = `variant_recipe_name`）。基线不合格时每个旋钮都判 incomplete。
- 结构比较：ablation.py 自带一份拍平器，空对象算叶子，缺键与 null 分开；judge 和 problems 都用它。formal._flat 不动。
- 红灯：trainer 在 f91acf8d 上复现 3 败，在 4e8d2538 上复现 2 败。
- 绿灯：合入后 test_ablation + test_ablation_judge + test_formal + test_data + test_train 共 114 过 2 跳过。行数：ablation.py 540，test_ablation_judge.py 372，test_ablation.py 301。

## 第 3 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×1 P2×1。逐条核实，都属实。
- **P1｜续训不核对开发份 build**（第 3 轮 #1）：开发配方原地改了而训练配方没变时，续训照常，run.json 仍留旧的 dev_build_id，judge 会把新开发份的读数当旧的比。改：check_resume 在记录里有 dev_build_id 时比对当前值，不同就拒绝续训；旧记录没有这个字段的照常续训。交 pi `trainer-ablation-fix4`。
- **P2｜不设 dev_recipe 时 run.json 多两个字段**（第 3 轮 #2）：第三次提出，仍留在 #217。理由同第 2 轮：不影响判结果，也不影响开训。

### 第 3 轮的处理（2026-11-26）

由 pi `trainer-ablation-fix4` 修，提交 060fe3f9..b4d67554，花费 0.0084 美元。
- 改动：`check_resume` 加了参数 `dev_build_id`。记录里有、这次也有、两者不同时，拒绝续训和重建。train.py 只多一个变量，两处调用都传它。
- 红灯：trainer 在 060fe3f9 上复现（复制件是 git 仓库，带模型源码），2 败 2 过。失败的两条都是「DID NOT RAISE」：开发份变了，旧代码仍放行续训和重建。照常续训的两条本来就过。
- 绿灯：合入后 test_ablation、test_ablation_judge、test_formal、test_data、test_train、test_review_fixes 共 157 过 2 跳过。

## 第 4 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×1。核实属实。
- **P1｜旧运行续训被拒**（第 4 轮 #1）：前三轮的 P2（#217）这轮给了新证据，升为 P1。旧代码开的运行，run.json 里没有 `config.data.dev_recipe`；新代码把它写成空串。config_diff 把缺键当成不同，于是 check_resume 拒绝续训。原有的「老 run.json 兼容」测试只删了 dev_build_id，留着 dev_recipe，没有模拟真的旧记录。改：旧记录缺的键，如果新值等于 config.py 的默认值，就不算改了；缺键而新值不是默认值的照样拒绝。测试改用真的旧格式。交 pi `trainer-ablation-fix5`，本 PR 一并关闭 #217。

### 第 4 轮的处理（2026-11-26）

由 pi `trainer-ablation-fix5` 修，提交 cfdd3031..3b4b1782，花费 0.0075 美元。
- 改动：runrec 的 `config_diff` 加可选参数 `defaults`，默认值从 config.py 取，不手抄。旧记录缺的键，新值等于默认值时跳过；缺键而新值不是默认值、或键在但值不同，仍算改了。check_resume 传入默认值。train.py 不动。
- 测试改用真的旧格式：既没有 `config.data.dev_recipe`，也没有 `dev_build_id`。覆盖照常续训、重建、缺键而新值非默认仍拒绝这三种。
- 红灯：trainer 在 cfdd3031 上复现，3 败 1 过。3 条都报「配置改了：data.dev_recipe」。
- 绿灯：合入后跑 test_ablation、test_ablation_judge、test_formal、test_data、test_train、test_review_fixes、test_git_record、test_guards，共 196 过 2 跳过。

## 第 5 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×1 P2×1。核实属实。
- **P1｜「训练有没有用」检查（dev_gate）与冒烟参照仍用训练 build 当开发份身份**（第 5 轮 #1）：dev_gate.py 拿 data_build_id 认开发份，smoke.py 写、比参照读数时也传训练 build。设了 dev_recipe 后，用 X 训、在 Y 上评的运行，能和在 X 上算的参照对上。pi 在第一次交活时提过 smoke 这一处。改：开发份身份一律用 dev_build_id，旧记录没有时退回 data_build_id。交 pi `trainer-ablation-fix6`。
- **P2｜不设 dev_recipe 时 run.json 仍多两个字段**（第 5 轮 #2）：续训已兼容（第 4 轮），#217 写的两种做法里选了「兼容」那种。结构本身仍会变，所以本 PR 只写 `Refs #217`，不关它。

### 第 5 轮的处理（2026-11-26）

由 pi `trainer-ablation-fix6` 修，提交 4c30bb29..95afdd65，花费 0.0066 美元。
- 改动：dev_gate 用 `dev_build_id`，旧记录没有时退回 `data_build_id`，拿它比参照的 build_id。smoke.py 写参照（`--write-known-dev`）和比参照（`--known-dev`）都传开发份的 build id。
- 红灯：trainer 在 4c30bb29 上复现，3 败 40 过。

## 第 6 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×0 P2×1。连续第 1 轮无 P0/P1。合入后 10 个测试文件共 239 过 2 跳过。
- **P2｜不设 dev_recipe 时 run.json 仍多两个字段**（第 6 轮 #1）：同第 5 轮 #2，留在 #217，本 PR 不关它。它不影响判结果，也不影响续训和开训。

## 第 7 轮（2026-11-26）· 模型 gpt-6-astra · 总评「修改」

P0×0 P1×0 P2×1。连续第 2 轮无 P0/P1，**达停止线**（脚本判「满足停止线」）。
- **P2｜不设 dev_recipe 时 run.json 仍多两个字段**（第 7 轮 #1）：同前，补记一处：旧运行续训时，也会把这个配置字段写进记录。留在 #217。
