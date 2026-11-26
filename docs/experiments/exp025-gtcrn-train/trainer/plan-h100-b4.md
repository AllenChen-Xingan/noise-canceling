# 执行单：新基线 build 上的冒烟 + 数据消融 7 组（H100，给 cloudrun）

这份执行单回答一件事：在新基线 build `bc0e9e7b566421f0` 上，先确认训练管线照常（冒烟 + 「训练有没有用」），再跑数据消融 7 组。
它沿用 `plan-h100-smoke.md` 第四节的跑法、出错收尾和定时限的方法，这里只写不同的地方。
下文「实测」= 2026-11-25 乙段读数（`smoke_reports/20261125-h100-stageB-timing.json`），「推断」= 由实测推算或没测过。

## 一、开跑前提（缺一条就不开机）

1. traindata 的分支已进 main，里面有：
   - build 记录 `data/builds/bc0e9e7b566421f0/build.json`
   - 新基线配方 `data/recipe.json`
   - 5 份变体配方 `data/recipes/ablation/*.json`，各自的 build 也已登记
2. 本分支 `feat/trainer-b4-run` 已进 main。在那个 main 上，本机跑这两条都退出 0：
   - `python formal.py check`
   - `python ablation.py check`
   两条都会核对：配方的 sha256 等于 build 记录里的值；data 的 6 个代码文件和登记时一字不差。代码文件改过，loader 会拒读这个 build（见 `data/loader.py` 的 `CODE`）。
3. 分片在网络盘 `/workspace/shards/batch4`（traindata `data/cloud-batch4b.md` 第 22 行，cloudrun 2026-11-26 实测 6 个 build 都 valid、交集为 0）。
4. 消融的输出目录 `/workspace/runs/exp025-ablation` 在网络盘上还不存在。已存在就停下报大总管：同名运行会被当成续训。
5. 大总管批了花费（见第四节）。这一单不是正式训练；正式训练另要人点头（开训前清单）。

下面 `R=/workspace/repo-<提交前 8 位>`（新目录，不覆盖旧的 checkout），`T`、`D`、`PY`、`G` 的含义同 `plan-h100-smoke.md` 第四节，
`S=/workspace/shards/batch4`，`O=/workspace/runs/exp025-b4-smoke-<UTC 时间>`。

## 二、甲段：CPU 机（cpu3c 4 核，约 0.12 美元 / 小时）

照 `plan-h100-smoke.md` 第四节甲段做，只改三处：
- 第 3 步单元测试：`VDN_LOADER_SHARDS=$S VDN_LOADER_RECIPE=$D/recipe.json`
- 第 4 步 CPU 参照：`--set data.shard_root=$S --set data.recipe=$D/recipe.json --write-known-dev $O/known-dev-cpu.json`
  - 必须重算：参照里记着开发份的 build id，旧参照（build f032a75e）对新 build 一律判不过
- 第 3 步多跑一条：`cd $T && $PY formal.py check && $PY ablation.py check`，两条都要退出 0
- 第 4 步之后加一步：**补测酒店房间 Room112 底噪的 250 Hz 单音**（大总管定；traindata 的 issue #221）
  - 命令和判法只在 `data/cloud-room112-tone.md`（traindata 写，PR #223 起在 main，a6bb1ced），这里不抄。在 `$R` 下照它跑，只读，不改仓里文件
  - 清单不是 248 条就停这一步，把数字发 traindata；这一步不过不挡乙段（它只决定正式训练前要不要出新 build，冒烟和消融结果照样有效）
  - `room112-tone.jsonl`、它的 sha256 和 diagnose 的摘要行随第 5 步一起取回，发 traindata，抄送大总管

| 步骤 | 时长 | 来源 |
|---|---|---|
| 开机 | 3–5 分钟 | 推断 |
| 拉代码 + 装环境 | 约 4 分钟（235 s） | 实测（旧机器） |
| 单元测试 | 约 5 分钟（301 s），新分片更大，可能更久 | 实测 + 推断 |
| CPU 参照（建开发份 640 段 + 评估一次） | 约 1.5 分钟（86 s） | 实测 |
| Room112 单音补测（248 条） | 约 10 分钟 | traindata 推算（按办公室 402 条的耗时） |
| 取回 + 删机 | 约 3 分钟 | 推断 |
| **合计** | **约 25–35 分钟，约 0.07 美元**（大总管批的上限 0.1 美元） | 推断 |

## 三、乙段：H100（约 4.39 美元 / 小时）

时限照旧算：`H = min(1.5, 7 / p − 5/60)`，H < 1.0 就不开。开机后装 git（镜像没有 git，训练器会拒跑）。

**第 1–3 步是闸门**：第 1 步 `pass` 为 true，且第 2 步 A、B 两个 dev_gate 都退出 0，才做第 4 步。任何一个不过：照样做第 3 步，跳过第 4 步，直接第 5、6 步收尾，报大总管。

1. **冒烟**：
   `cd $T && VDN_GTCRN_DIR=$G $PY smoke.py --config configs/default.yaml --set data.shard_root=$S --set data.recipe=$D/recipe.json --timing-steps 100 --known-dev $O/known-dev-cpu.json --out $O/smoke-h100.json`
2. **训练有没有用**（300 步，A、B 各一次，一个跑完再跑下一个）：
   `cd $T && VDN_GTCRN_DIR=$G $PY train.py --config configs/default.yaml --set run.out_dir=$O --set run.name=speed-B --set schedule.max_steps=300 --set schedule.eval_every=100 --set schedule.ckpt_every=100 --set data.shard_root=$S --set data.recipe=$D/recipe.json`
   A 同样，`run.name=speed-A`，加 `--set model.variant=A`。然后：
   `cd $T && $PY dev_gate.py $O/speed-B --known-dev $O/known-dev-cpu.json > $O/dev-gate-B.json`，A 同样
3. **续训核对**：照 `plan-h100-smoke.md` 乙段第 3 步，加上 `--set data.shard_root=$S --set data.recipe=$D/recipe.json`。
   续训代码在 #212 改过（会核对开发份 build），所以再跑一次。退出码 1（比出来不同）只记录，不挡第 4 步。
4. **数据消融 7 组**：命令由 `cd $T && $PY ablation.py commands` 生成，共 3 批（3 + 3 + 1）。
   - 每批里的运行用 `job-start` 同时发，一批全部退出后再发下一批
   - 每条命令都带 `--set data.dev_recipe=<基线配方>`：7 组在同一份开发份上比
   - 任何一条退出码非 0：不发下一批，做第 5、6 步
5. 打包取回：`$O` 和 `/workspace/runs/exp025-ablation` 各打一个包（不含 `*.pt`），先 `cp /root/jobs/*.log /root/jobs/*.rc` 进去，取回主仓 `_tmp/<日期>-trainer-b4/`，不覆盖
6. 删机，报账上 0 台和本次花费

| 步骤 | 时长 | 依据 |
|---|---|---|
| 开机 + 装 git + 记显卡 | 约 5 分钟 | 推断（上次乙段的非训练时间） |
| 1 冒烟 | 5–10 分钟 | 推断（plan-h100-smoke.md 的估计，上次跑过） |
| 2 A、B 各 300 步 + 判 | 约 10 分钟：B 294 + 300 × 0.1665 ≈ 344 s；A 176 + 300 × 0.148 ≈ 220 s；另加 6 次评估约 4 s | 实测推算 |
| 3 续训核对 | 2–3 分钟 | 推断 |
| 4 消融第 1、2 批（各 3 个 A 同时跑，1000 步，评估 2 次） | 每批约 8–9 分钟：准备 158 s + 1000 × 0.32–0.39 s + 2 × 2.2 s ≈ 480–550 s | 0.39 s 是 3 个 B 同时跑的实测；A 按 0.81 倍推断成 0.32 s |
| 4 消融第 3 批（1 个 A 单跑） | 约 5.5 分钟：176 + 1000 × 0.148 + 2 × 0.65 ≈ 325 s | 实测推算 |
| 5–6 打包取回 + 删机 | 3–5 分钟 | 推断 |
| **合计** | **约 47–57 分钟** | 推断 |

## 四、花费

| 段 | 时长 | 按 4.39 美元 / 小时 | 按 5.27 美元 / 小时（高 20%） |
|---|---|---|---|
| 甲 CPU 机 | 25–35 分钟 | 约 0.07 美元 | 约 0.08 美元 |
| 乙 H100 | 47–57 分钟 | 3.4–4.2 美元 | 4.1–5.0 美元 |
| **合计** | | **约 3.5–4.3 美元** | **约 4.2–5.1 美元**（甲段多出的 0.02 美元不改两位数） |

- 其中消融本身（第 4 步，约 22–24 分钟）约 1.6–1.8 美元，与之前估的 1.8 美元相符。其余是新 build 上的冒烟和判训练有没有用
- 上限（大总管批 2026-11-26）：甲段 ≤ 0.1 美元；乙段 1.2 小时、7 美元（定时限的公式会保证最坏 (H + 5/60) × p ≤ 7 美元）
- 已花：cloudrun 记 8.97 美元（总上限 100 美元，cloudrun 2026-11-26 交接页）

## 五、带回后谁判

- 冒烟和训练有没有用：机器判。`smoke-h100.json` 的 `pass`、两个 dev_gate 的退出码，规则同 `plan-h100-smoke.md` 第六节
- 消融：trainer 在本机跑 `python ablation.py judge <取回的 exp025-ablation 目录> --out smoke_reports/<日期>-ablation-judge.json`
  - 判法在 `ablation.py` 的 `RULE`（大总管定 2026-11-26，跑之前写定）：差值超过两个基线种子之差才算有用或有害，否则记「消融分不出」，保持原样
  - 退出码 2（有运行不完整）：报大总管，不补跑、不改判法
- trainer 把读数和结论写进 `README.md`，报大总管。消融结论只决定配方开关；改配方归 traindata

## 六、如果出意外

照 `plan-h100-smoke.md` 第七节。另加两条：
- loader 报「build 没登记或交集检查不过」：代码提交不对，或 data 的代码文件改过。停下报大总管转 traindata，不在机器上改代码
- 消融某一批比估计慢 1.5 倍以上（第 1 批超过 14 分钟）：照跑完，带回日志，trainer 重估后再报
