# 执行单：正式训练的 6 个模型 + 原版，上云跑 7 份成绩单（给 cloudrun）

> 2026-11-26 · evalcmp 写 · 交大总管 noise-canceling-meta。**只写，没跑；开机等大总管批。**
> 依据：D119（练完接着评）· D120（判法三条按默认）· `eval-plan.md`（怎么评）· `trainer/plan-h100-formal.md`（6 个运行的名字和位置）。
> 跑法沿用 `cloud/plan-eval-baseline.md`（密钥上机、评分模型、音频链接）和识别补跑那次（4 片 + 预置缓存 + 花费闸）。这里只写不同的地方。
> 标「推断」的数没有实测。大总管 2026-11-26 已定：报告种子 s1 · 本批识别上限 1.2 美元 · 两台先后开 · 延迟一行总判定最多到「判不了」。

## 一、跑哪 7 份

| 系统名（`--name`） | 模型 | ONNX 从哪来 | 在判读里当什么 |
|---|---|---|---|
| `exp025-orig` | 原版（甲） | 本机对拍用的那份，上传：`<主目录>/_tmp/20261126-eval-parity/gtcrn-official.onnx`，sha256 `384e2c68…0205f`（全值见 `builds/gtcrn-offline-parity1/run.json`） | A 的对照 |
| `exp025-a-s0` · `exp025-a-s1` · `exp025-a-s2` | A × 种子 0、1、2 | 云上导出（第三节第 3 步） | A 的候选；**a-s1 也是 B 的对照** |
| `exp025-b-s0` · `exp025-b-s1` · `exp025-b-s2` | B × 种子 0、1、2 | 同上 | B 的候选 |

- **检查点**：用每个运行的 `ckpt/best.pt`，也就是开发份上 hybrid 损失最低的那一次（trainer 提议，我同意）。开发份和评测集来源不同，不算拿评测集挑模型；这一点是 trainer 的推断，没逐段比过。最后一步的 `step-0005000.pt` 不评；要评就是 12 份，时间翻倍
- **报告种子 = s1**（大总管定，2026-11-26）：人看到的题面写的是「种子 1」（D120），trainer 的种子编号是 0、1、2，按字面就是 `a-s1`、`b-s1`。开评前已定，不许看完结果再挑
- **都走甲路**（离线整段，`eval-plan.md` 第二节第 2 步），同一个评测代码提交 `EVAL_COMMIT`（开机时的 main），同样 `--no-safety --qa --rules v2`
- ⚠️ **这 7 份判不出「过」**：甲路只有声明延迟，`accept` 的延迟一行固定是「判不了」，所以总判定最多到「判不了」（大总管定）。其余各行的过 / 不过照常有效。要判延迟，得另跑乙路：A 转流式后再跑 3 份，转流式的工具还没有，另开 #232 交 trainer，不挡本次。B 没有乙路（大总管已把这条记成 trainer 的风险）
- 本单不跑：内置原版在 `EVAL_COMMIT` 上的重跑（`eval-plan.md` 第二节第 1 步）。7 份都在同一个提交上跑，判读只用这 7 份，不用旧读数；所以这一步只在要和旧差距表对照时才需要

## 二、机器、时长、花费

**机器**：cpu3c 4 核 · US-MO-1 · 挂网络盘 bzpbedqdsi · 每小时 0.12 美元（`rp.py up --cpu`，正本 `cloud/config.py`）。

**分两台，先后开，不同时开**（大总管定）：
- 机 1 跑原版 + A 三个种子（4 份），机 2 跑 B 三个种子（3 份）
- 一台跑 7 份约 6.6 小时，超过 CPU 机单次 6 小时的上限
- 两台同时开能省约 3.5 小时，但两台会同时写网络盘上同一个识别账本。账本的锁是文件锁（`src/eval/src/qa/asr_ledger.rs`），跨机器在网络盘上灵不灵没验证过。所以先后开；评测在训练之后，多约 3.5 小时不要紧

**每份成绩单约 48–51 分钟（推断）**：
- 依据：识别补跑那次，4 片同时跑，每片 4061–4227 秒。其中识别 2056–2166 秒（每片约 488 次），两个评分模型合计约 1500 秒，内置 GTCRN 推理约 320 秒
- 新模型每片只识别约 258 次，按比例约 1100 秒。甲路不在成绩单里推理（输出已经算好），省掉约 320 秒
- 合计每片约 2900–3050 秒

**离线推理每个模型约 7–10 分钟（推断）**：本机 4 片同时跑全量 480 段实测 3.2 分钟；云上这档 x86 CPU 没量过，按慢 2–3 倍估。机 1 第一个模型实测后再改估计。

| 机器 | 内容 | 估计时长 | 自删时限 `--max-hours` | 机费（估计 / 最坏） |
|---|---|---|---|---|
| 机 1 | 开机 · 编译 · 装 Python · 导出 7 个 ONNX · 离线推理 4 个模型 · 4 份成绩单 · 取回 | 约 4.6 小时 | 6.0 | 0.55 / 0.73 美元 |
| 机 2 | 开机 · 装 Python · 离线推理 3 个模型 · 3 份成绩单 · 取回 | 约 3.6 小时 | 5.0 | 0.43 / 0.61 美元 |
| **合计** | | 约 8.2 小时（先后跑） | | **约 1.0 / 最坏 1.34 美元**（D119 定 CPU 机 ≤ 1.5 美元） |

最坏按「每小时价 × (时限 + 看门狗宽限 5 分钟)」算。

**识别**：
- 每份成绩单约 1030 次：480 条输出 × 2 次 = 960 次；前两次不一致时补第 3 次，按补跑实测的 14.6% 算约 70 次
- 干净人声和原声与模型无关，从补跑的缓存预置，0 次（第三节第 6 步）
- 单价：补跑实测 0.1875 美元 / 1953 次，约每次 0.000096 美元
- 7 份约 7200 次，约 **0.69 美元**
- 本批识别上限 **1.2 美元**（约 1.7 倍余量；大总管已批）

## 三、步骤（每台机器都照做；机 2 跳过第 3 步里已导出的部分）

```sh
# 0. 开机 + 开机检查（自删时限见上表）
python3 rp.py up --cpu --max-hours 6.0 --yes          # 机 2 用 5.0
python3 rp.py preflight <机器> --repo <仓库> --commit <EVAL_COMMIT>
```

1. **代码**：网络盘新目录 `/workspace/repo-<EVAL_COMMIT 前 8 位>`，内容为 main 的 `EVAL_COMMIT`。开机时报大总管提交号，7 份都用这一个；机 2 用同一个目录，不换提交。`cargo build --release -p vdn-eval`
2. **密钥、评分模型、音频**：照 `cloud/plan-eval-baseline.md` 第一节（`put-secret`，先用假值冒烟）和第三节（两个评分模型核 sha256；仓库根下 `_tmp/exp013/20261124165427-6323` 建链接）
3. **Python 环境与导出**（命令由 trainer 给，2026-11-26）：
   - 环境（trainer 核过，2026-11-26）：`R` = 第 1 步的仓库目录（带 `.git`，不用 `git archive` 解包），`D=$R/docs/experiments/exp025-gtcrn-train/data`，`T=$R/docs/experiments/exp025-gtcrn-train/trainer`，`PY=$R/.venv/bin/python`。两份 requirements 都要装：只装 trainer 那份会缺 data 的依赖
   ```sh
   export PATH=/workspace/bin:$PATH     # 外层先导出：cloud_steps.sh 只在自己的子 shell 里改 PATH
   cd $D && bash cloud_steps.sh setup && uv pip install --python $PY -r $D/requirements.txt -r $T/requirements.txt
   ```
   - 官方 GTCRN 源码目录 `G=/workspace/gtcrn-502ebfab`（训练和消融用的同一份）。导出也要它：模型类直接从官方 `gtcrn.py` 建。先核 `test -f $G/gtcrn.py`；不在就照钉死的提交取：`git clone https://github.com/Xiaobin-Rong/gtcrn $G && git -C $G checkout 502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73`
   - 机 1 导出全部 6 个，存到网络盘 `$OUT=/workspace/eval-formal/onnx/`：
   ```sh
   cd $T
   for r in exp025-formal-A-s0 exp025-formal-A-s1 exp025-formal-A-s2 exp025-formal-B-s0 exp025-formal-B-s1 exp025-formal-B-s2; do
     VDN_GTCRN_DIR=$G $PY export.py --ckpt /workspace/runs/exp025-formal/$r/ckpt/best.pt --out $OUT/$r.onnx \
       || { echo "导出或对拍不过：$r"; exit 1; }
   done
   $PY export.py --official --out $OUT/official-recheck.onnx      # 只用来核 sha256
   sha256sum $OUT/*.onnx > $OUT/SHA256SUMS
   ```
   - 退出码非 0 = ONNX 和 PyTorch 对不上：停下，报 trainer 和大总管，这个模型不评
   - 每个模型另有对拍报告 `$OUT/<运行名>.json`，记着来源步数。取回时带上
   - 原版：上传本机那份 `gtcrn-official.onnx`，核 sha256 = `384e2c68…`。`official-recheck.onnx` 的 sha256 和它不一样时只记下、不停：甲路对拍用的是上传那份
4. **离线推理**（每个模型；`NAME` 是第一节的系统名，`ONNX` 是对应文件）：
   ```sh
   cd $R      # 第 4、6 步都在仓库根下跑
   E=docs/experiments/exp025-gtcrn-train/eval/run_gtcrn_offline.py
   $PY $E --onnx $ONNX --name $NAME --probe                       # 先量前视、对齐、一段耗时
   for i in 0 1 2 3; do $PY $E --onnx $ONNX --name $NAME --run-id formal1 --shard $i/4 & done; wait
   $PY $E --onnx $ONNX --name $NAME --run-id formal1 --finalize
   ```
   - 输出写到仓库根 `_tmp/exp016/$NAME/formal1/`，清单写到 `eval/builds/$NAME-formal1/`（这个要取回、进仓）
   - 探针的「声明延迟」应是 256。不是 256 就停下报 evalcmp
   - 离线推理跟成绩单不同时跑：两样都吃满 4 核
5. **账本**：开跑前记 sha256、`calls`、`spent_usd`；没有 `review`，`reserved_usd` 为 0。不是这样就先报，别跑
6. **成绩单**（每个模型；先跑 `exp025-orig`）：
   ```sh
   RUN=$(date -u +%Y%m%d-%H%M%S)-$NAME
   R=docs/experiments/exp025-gtcrn-train/eval/runs
   OLD=<补跑那份仓库副本>/docs/experiments/exp025-gtcrn-train/baseline/runs/20261126-013313-rerun   # 后面接 -s0..-s3
   for i in 0 1 2 3; do mkdir -p "$R/$RUN-s$i/qa" && cp "$OLD-s$i/qa/asr_cache.json" "$R/$RUN-s$i/qa/"; done
   set -a; . /root/.env.local; set +a
   for i in 0 1 2 3; do
     nohup target/release/vdn-eval scorecard --system external:_tmp/exp016/$NAME/formal1 \
       --no-safety --qa --rules v2 --shard $i/4 --asr-ledger "$LEDGER" --out "$R/$RUN-s$i" > "$R/$RUN-s$i.log" 2>&1 &
   done; wait
   target/release/vdn-eval scorecard --merge-shards "$R/$RUN-s0,$R/$RUN-s1,$R/$RUN-s2,$R/$RUN-s3" \
     --suite src/eval/suite/scenes.json --out "$R/$RUN"
   ```
   - 预置缓存时片号要对得上（第 i 片用补跑第 i 片的缓存）：分片规则是确定的，同一段永远分到同一片（`baseline/README.md` 分片一节）
   - **第一份（原版）跑完先核缓存命中**：识别次数应约 1030；超过 1600 说明干净人声或原声的缓存没命中，停下报 evalcmp，别跑后面几份
7. **取回、删机**：见第五节。机 1 取回、核对、删机后，再开机 2

## 四、花费闸（必做，每 30 秒读一次账本）

下面任一条满足，就停掉全部 vdn-eval 进程，报 evalcmp 和大总管：
- 一份成绩单的 `calls` 比它开跑前多 1600 次以上，或 `spent_usd` 多 0.16 美元以上（正常约 1030 次、0.10 美元）
- 本批识别累计超过 1.2 美元
- 任何一片的日志或 `meta.asr.halted` 出现停发：别自己清「待核」，照 `baseline/README.md` 用 `clear_review.py`，先报

## 五、取回（放主目录 `_tmp/<日期>-cloudrun-eval-formal/`，不覆盖）

- 每份成绩单合并目录里的 `scorecard.json`、`scorecard.md`、`clips.jsonl`，附 sha256
- 每片的 `scorecard.json`（只要 meta）和日志
- 每个模型的 `eval/builds/<系统名>-formal1/`（参数、每个输出文件的哈希、分片耗时）
- `$OUT/SHA256SUMS` 和 6 份导出对拍报告 `<运行名>.json`
- 跑后的账本快照（附 sha256）和本批新增的明细行
- 不取回：音频（离线输出、`qa/asr-wav/`）、ONNX。取回前照旧查一遍有没有密钥字样

**核对（cloudrun 先核，再报）**：7 份都是 480 行；各片 `meta.asr` 没有 `error`，`halted` 为 null；每份只有一个系统；账本 `calls` 增量约等于各片 `meta.asr.calls` 之和。

## 六、取回后：配对比较与判读（evalcmp 在本机跑，不花钱，几分钟）

先照本机负载规则看 `uptime`（1 分钟负载 > 20 就等），再查有没有 `_tmp/heavy.lock`。下面 `C` 是取回目录，`X/` 是系统 X 的合并成绩单目录。

```sh
S=src/eval/suite/scenes.json; T=rail-air-cabin,canteen,meeting-room,transit-hub; V=target/release/vdn-eval
# A 对原版（三个种子）+ A 组种子噪声底（三对）
for s in 0 1 2; do $V compare --a $C/exp025-orig/clips.jsonl --b $C/exp025-a-s$s/clips.jsonl --by-scene --out $C/cmp/a-s$s-vs-orig; done
for p in "0 1" "0 2" "1 2"; do set -- $p; $V compare --a $C/exp025-a-s$1/clips.jsonl --b $C/exp025-a-s$2/clips.jsonl --by-scene --out $C/cmp/noise-a-s$1-s$2; done
# B 对 A（对照都是报告种子 a-s1）+ B 组种子噪声底
for s in 0 1 2; do $V compare --a $C/exp025-a-s1/clips.jsonl --b $C/exp025-b-s$s/clips.jsonl --by-scene --out $C/cmp/b-s$s-vs-a-s1; done
for p in "0 1" "0 2" "1 2"; do set -- $p; $V compare --a $C/exp025-b-s$1/clips.jsonl --b $C/exp025-b-s$2/clips.jsonl --by-scene --out $C/cmp/noise-b-s$1-s$2; done
# 判读：第一个 --vs 是报告种子（s1）
$V accept --suite $S --scenes $T \
  --vs $C/cmp/a-s1-vs-orig/compare.json --vs $C/cmp/a-s0-vs-orig/compare.json --vs $C/cmp/a-s2-vs-orig/compare.json \
  --noise $C/cmp/noise-a-s0-s1/compare.json --noise $C/cmp/noise-a-s0-s2/compare.json --noise $C/cmp/noise-a-s1-s2/compare.json \
  --card-a $C/exp025-orig/scorecard.json --card-b $C/exp025-a-s1/scorecard.json --out $C/accept-a-vs-orig
$V accept --suite $S --scenes $T \
  --vs $C/cmp/b-s1-vs-a-s1/compare.json --vs $C/cmp/b-s0-vs-a-s1/compare.json --vs $C/cmp/b-s2-vs-a-s1/compare.json \
  --noise $C/cmp/noise-b-s0-s1/compare.json --noise $C/cmp/noise-b-s0-s2/compare.json --noise $C/cmp/noise-b-s1-s2/compare.json \
  --card-a $C/exp025-a-s1/scorecard.json --card-b $C/exp025-b-s1/scorecard.json --out $C/accept-b-vs-a
```

- 小文件（成绩单的 `scorecard.json`、`scorecard.md`、`clips.jsonl.gz`，12 份 compare，2 份 accept，各模型的 builds 清单）复制进 `eval/runs/formal1/` 和 `eval/builds/`，提交；音频不进仓
- 报大总管：两份 `accept.md` 的总判定和逐行结果，带上「人耳未测」「只在合成集上判」两句（`accept.md` 顶部固定有）；延迟一行「判不了」的原因照第一节写

## 七、如果出意外

- 导出对拍不过 → 那个模型不评，报 trainer；其余照跑。少了一个种子，`accept` 要涨那几行会判不了（种子数要正好 3）
- 离线推理 `--finalize` 报缺段 → 不跑成绩单，查这一片的日志，用同一个 run-id 续跑（已写的文件会跳过）
- 原版那份识别次数超过 1600 → 停（第三节第 6 步）
- 机 1 跑到 5.5 小时还没跑完 → 已经合并好的成绩单先取回，没跑完的那份留到机 2 接着跑，报大总管
- 任何一份不是 480 行 → 不进判读，先查
