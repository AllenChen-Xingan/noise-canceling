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
- ⚠️ **这 7 份判不出「过」**：甲路只有声明延迟，`accept` 的延迟一行固定是「判不了」，所以总判定最多到「判不了」（大总管定）。其余各行的过 / 不过照常有效。要判延迟，得另跑乙路：A 转流式后再跑 3 份，转流式的工具还没有，见 #231（trainer），不挡本次。B 没有乙路（大总管已把这条记成 trainer 的风险）
- **评测代码回归检查**（`eval-plan.md` 第二节第 1 步；本单第三节第 5 步）：`EVAL_COMMIT` 和原版读数的提交 ab0f4f0a 不同，所以先在机 1 上用**内置**原版在 `EVAL_COMMIT` 上重跑一份，和补跑那份逐段逐项比判定、阈值和数值。过了才跑 7 份；不过就停。不设豁免

## 二、机器、时长、花费

**机器**：cpu3c 4 核 · US-MO-1 · 挂网络盘 bzpbedqdsi · 每小时 0.12 美元（`rp.py up --cpu`，正本 `cloud/config.py`）。

**分两台，先后开，不同时开**（大总管定）：
- 机 1：回归检查 + 原版 + `a-s0` + `a-s1`；机 2：`a-s2` + B 三个种子
- 一台跑完全部约 8.8 小时，超过 CPU 机单次 6 小时的上限
- 两台同时开会同时写网络盘上同一个识别账本。账本的锁是文件锁（`src/eval/src/qa/asr_ledger.rs`），跨机器在网络盘上灵不灵没验证过。评测在训练之后，多几个小时不要紧

**一份成绩单 46–49 分钟，按 55 分钟排（推断）**：
- 依据：识别补跑那次，4 片同时跑，每片总计 4061–4227 秒。其中识别 2056–2166 秒（每片约 488 次），内置 GTCRN 推理 318–327 秒
- 扣掉这两项，其余（两个评分模型等）每片 1687–1734 秒
- 新模型每片只识别约 258 次，按比例约 1087–1145 秒；甲路读现成输出，按 ≤ 60 秒算
- 合计每片 2774–2939 秒，即 46–49 分钟；排时间按 55 分钟（约 12% 余量）
- **回归检查那份**：识别全部命中缓存，约 1687–1734 + 320 秒，约 34–35 分钟，按 45 分钟排

**离线推理一个模型按 10 分钟排（推断）**：本机 4 片同时跑全量 480 段实测 3.2 分钟；云上这档 x86 CPU 没量过，按慢 3 倍估。机 1 第一个模型实测后再改估计。

| 机器 | 内容（分钟） | 估计时长 | 自删时限 `--max-hours` | 机费（估计 / 最坏） |
|---|---|---|---|---|
| 机 1 | 开机、编译、装环境 30 · 导出 7 个 ONNX 10 · 回归检查 45 · 3 个模型 ×（离线 10 + 成绩单 55）· 取回 10 | 约 4.8 小时 | 5.75 | 0.58 / 0.70 美元 |
| 机 2 | 开机、装环境 30 · 4 个模型 ×（离线 10 + 成绩单 55）· 取回 10 | 约 5.0 小时 | 5.75 | 0.60 / 0.70 美元 |
| **合计** | | 约 9.8 小时（先后跑） | | **约 1.18 / 最坏 1.40 美元**（D119 定 CPU 机 ≤ 1.5 美元） |

最坏按「每小时价 × (时限 + 看门狗宽限 5 分钟)」算。

**识别**：
- 每份成绩单约 1030 次：480 条输出 × 2 次 = 960 次；前两次不一致时补第 3 次，按补跑实测的 14.6% 算约 70 次
- 干净人声和原声与模型无关，从补跑的缓存预置，0 次。缓存键是送识别那条音频的 sha256（`src/eval/src/qa/mod.rs:113`）；干净人声和原声取自评测清单的 `clip.clean` / `clip.mix`（`qa/mod.rs:153–154`），与系统是内置还是外部无关【读源码】
- 回归检查那份输出不变，预期 0 次
- 单价：补跑实测 0.1875 美元 / 1953 次，约每次 0.000096 美元
- 7 份约 7200 次，约 **0.69 美元**
- 本批识别上限 **1.2 美元**（约 1.7 倍余量；大总管已批）

## 三、步骤

**命令块都用 `bash` 跑**（机器里的 shell；本机是 zsh，分词规则不同），开头 `set -euo pipefail`，任何一步失败就停。列表里的命令块前面那 3 格缩进只是排版，复制时去掉：缩进的 `EOF` 收不了 heredoc。顺序：先备齐环境、账本起点和闸门（第 1–4 步），再做回归检查（第 5 步，只在机 1），最后逐个模型离线推理加成绩单（第 6–7 步）。

```sh
# 0. 开机 + 开机检查
python3 rp.py up --cpu --max-hours 5.75 --yes
python3 rp.py preflight <机器> --repo <仓库> --commit <EVAL_COMMIT>
```

**固定变量**（每个块开头都先设好；`REPO` 是仓库根，后面不再改它）：

```sh
set -euo pipefail
EVAL_COMMIT=<开机时 main 的完整提交号>
REPO=/workspace/repo-${EVAL_COMMIT:0:8}
W=$REPO/_tmp/eval-formal                         # ONNX、批次起点、闸门脚本都放这里（在仓库里，离线脚本才认路径）
OUT=$W/onnx
RUNS=$REPO/docs/experiments/exp025-gtcrn-train/eval/runs
LEDGER=/workspace/asr-ledger/asr-openrouter-ledger.json
D=$REPO/docs/experiments/exp025-gtcrn-train/data
T=$REPO/docs/experiments/exp025-gtcrn-train/trainer
PY=$REPO/.venv/bin/python
G=/workspace/gtcrn-502ebfab
OLD=<补跑那份仓库副本>/docs/experiments/exp025-gtcrn-train/baseline/runs/20261126-013313-rerun   # 后面接 -s0..-s3、-merged
mkdir -p "$OUT"
```

1. **代码**：`$REPO` 是 main 的 `EVAL_COMMIT`（带 `.git`，不用 `git archive` 解包）。开机时报大总管提交号，回归检查和 7 份都用这一个；机 2 用同一个目录，不换提交。`cd $REPO && cargo build --release -p vdn-eval`
2. **密钥、评分模型、音频**：照 `cloud/plan-eval-baseline.md` 第一节（`put-secret`，先用假值冒烟）和第三节（两个评分模型核 sha256；`$REPO/_tmp/exp013/20261124165427-6323` 建链接）
3. **Python 环境与导出**（命令由 trainer 给并核过，2026-11-26）：
   ```sh
   export PATH=/workspace/bin:$PATH     # 外层先导出：cloud_steps.sh 只在自己的子 shell 里改 PATH
   cd $D && bash cloud_steps.sh setup && uv pip install --python $PY -r $D/requirements.txt -r $T/requirements.txt
   test -f $G/gtcrn.py || { git clone https://github.com/Xiaobin-Rong/gtcrn $G && git -C $G checkout 502ebfab64da7c4a9af78dcb9c6ceef1ebb01c73; }
   test -f $G/gtcrn.py
   ```
   - 两份 requirements 都要装：只装 trainer 那份会缺 data 的依赖。`$G` 是训练和消融用的同一份官方 GTCRN 源码，导出也要它（模型类直接从官方 `gtcrn.py` 建）
   - 机 1 导出全部 6 个（机 2 不再导）：
   ```sh
   cd $T
   for r in exp025-formal-A-s0 exp025-formal-A-s1 exp025-formal-A-s2 exp025-formal-B-s0 exp025-formal-B-s1 exp025-formal-B-s2; do
     VDN_GTCRN_DIR=$G $PY export.py --ckpt /workspace/runs/exp025-formal/$r/ckpt/best.pt --out $OUT/$r.onnx \
       || { echo "导出或对拍不过：$r"; exit 1; }
   done
   VDN_GTCRN_DIR=$G $PY export.py --official --out $OUT/official-recheck.onnx      # 只用来核 sha256
   ```
   - 退出码非 0 = ONNX 和 PyTorch 对不上：停下，报 trainer 和大总管，这个模型不评
   - 每个模型另有对拍报告 `$OUT/<运行名>.json`，记着来源步数。取回时带上
   - 原版：本机那份 `gtcrn-official.onnx` 用 `rp.py put` 传到 `$OUT/exp025-orig.onnx`（`put` 自己核 sha256），再核一遍 = `384e2c6818b38e5a32639f42a854e9b51273bdb3b30ee643deb7da0bd970205f`。`official-recheck.onnx` 的 sha256 和它不一样时只记下、不停：甲路对拍用的是上传那份
   - 最后 `sha256sum $OUT/*.onnx > $OUT/SHA256SUMS`
   - 系统名与 ONNX 的对应：`exp025-orig` ↔ `exp025-orig.onnx`；`exp025-a-sK` ↔ `exp025-formal-A-sK.onnx`；`exp025-b-sK` ↔ `exp025-formal-B-sK.onnx`
4. **账本、批次起点、闸门脚本和跑一份成绩单的函数**：
   ```sh
   $PY - "$LEDGER" <<'EOF'
   import json, sys
   d = json.load(open(sys.argv[1]))
   assert not d.get("review") and d["reserved_usd"] == 0, ("账本有待核或在途预留，先报", d.get("review"), d["reserved_usd"])
   print(d["calls"], d["spent_usd"])
   EOF
   sha256sum "$LEDGER"
   # 本批起点只在机 1 第一次写；机 2 沿用同一个文件（在网络盘上），不重写
   test -f $W/batch-start.json || $PY -c 'import json,sys; d=json.load(open(sys.argv[1])); json.dump({"calls": d["calls"], "spent_usd": d["spent_usd"]}, open(sys.argv[2], "x"))' "$LEDGER" $W/batch-start.json
   ```
   闸门脚本 `$W/gate.py` 和函数 `run_card` 见第四节：机 1 先写好，机 2 直接用（都在网络盘上）。`run_card` 每次跑一份成绩单：预置并核对缓存 → 启动闸门、确认它活着 → 启动 4 片 → 一直看着，闸门停了（越线或出错）就杀掉 4 片 → 全部正常退出才合并。
5. **回归检查（只在机 1，第一份）**：
   ```sh
   cd $REPO; set -a; . /root/.env.local; set +a
   VDN_GTCRN_ONNX=<官方流式权重，plan-eval-baseline.md 第三节> run_card gtcrn regress-gtcrn
   $PY docs/experiments/exp025-gtcrn-train/eval/parity.py --a "$RUNS/$RUN/clips.jsonl" --b "$OLD-merged/clips.jsonl" \
     --system-a gtcrn --system-b gtcrn                              # 数值：逐段逐项，容差 1e-3
   $PY - "$RUNS/$RUN/clips.jsonl" "$OLD-merged/clips.jsonl" <<'EOF'   # 判定与阈值：逐段逐项要完全相同
   import json, sys
   def load(p):
       out = {}
       for line in open(p):
           r = json.loads(line); k = (r["clip"], r["scene"], r["tier"], r["device"], r["mix_sha256"])
           for s in r["scores"]:
               out[k + (s["v"],)] = (s["verdict"], s.get("threshold"), s.get("veto"))
       return out
   a, b = load(sys.argv[1]), load(sys.argv[2])
   if a.keys() != b.keys():
       sys.exit(f"段或项对不上：{len(a.keys() ^ b.keys())} 处")
   soft = {"V-EAT-ASR"}               # 只报不判：吃字（eval-plan.md 第二节第 1 步唯一的例外）
   hard = [k for k in a if a[k] != b[k] and k[-1] not in soft]
   print("判定、阈值或否决变了：", len(hard), "· 吃字变了（只报）：", sum(a[k] != b[k] for k in a if k[-1] in soft))
   sys.exit(1 if hard else 0)
   EOF
   ```
   - 补跑合并那份 `clips.jsonl` 的 sha256 应是 `c00ed213a209a19a4aee965589aa06376271e203b08ba0fb359045be7a2e1ddf`，先核
   - 两个检查都过才往下。`parity.py` 只比数值，跳过延迟和最坏一帧（`parity.py` 文件头）；判定、阈值和否决标记由第二段逐段逐项比，除吃字外一处都不许变（最坏一帧也算在内：它按墙钟计时，换机器可能变；只有它变了也停，报 evalcmp，由大总管定）。场景卡状态由这些判定按同一规则汇总，所以判定全同、卡也同
   - 不过：停，报 evalcmp 和大总管，不跑后面的模型
6. **离线推理**（每个模型；`NAME` 是第一节的系统名，`ONNX` 是第 3 步对应的文件）：
   ```sh
   cd $REPO
   E=docs/experiments/exp025-gtcrn-train/eval/run_gtcrn_offline.py
   $PY $E --onnx $ONNX --name $NAME --probe                       # 先量前视、对齐、一段耗时
   pids=(); trap 'kill "${pids[@]}" 2>/dev/null || true' EXIT; trap 'exit 143' INT TERM   # 本块中途退出（失败或被中断）就收掉各片
   for i in 0 1 2 3; do $PY $E --onnx $ONNX --name $NAME --run-id formal1 --shard $i/4 & pids+=($!); done
   for p in "${pids[@]}"; do wait $p; done                        # 任何一片非 0 退出，set -e 就停，trap 收掉其余
   trap - EXIT INT TERM
   $PY $E --onnx $ONNX --name $NAME --run-id formal1 --finalize
   ```
   - ONNX 必须在 `$REPO` 里（`$W/onnx/`）：离线脚本把路径写进仓前只认工作副本或主目录里的文件（`run_gtcrn_offline.py` 的 `rel()`），仓外的路径直接拒绝，软链接也不行
   - 输出写到 `$REPO/_tmp/exp016/$NAME/formal1/`，清单写到 `eval/builds/$NAME-formal1/`（这个要取回、进仓）
   - 探针的「声明延迟」应是 256。不是 256 就停下报 evalcmp
   - 离线推理跟成绩单不同时跑：两样都吃满 4 核
7. **成绩单**（每个模型，机 1 先跑 `exp025-orig`）：
   ```sh
   cd $REPO; set -a; . /root/.env.local; set +a
   run_card external:_tmp/exp016/$NAME/formal1 $NAME
   ```
   - **第一份（原版）跑完先核缓存命中**：识别次数应约 1030；超过 1600 说明干净人声或原声的缓存没命中，停下报 evalcmp，别跑后面几份
8. **取回、删机**：见第五节。机 1 取回、核对、删机后，再开机 2

## 四、花费闸与 `run_card`（必做）

程序本身只有一道闸：发每条识别前在账本里预留，`已花 + 在途 + 这一条` 超过 15 美元就不发（`src/eval/src/qa/asr_or.rs` 的 `BUDGET_USD`，D103 定为软上限）。本批 1.2 美元和每份的上限靠闸门脚本：每 30 秒读一次账本，越线或读不了就退出；`run_card` 一看到闸门退出，就杀掉这一份的 4 片。

```sh
cat > $W/gate.py <<'EOF'
# 用法：gate.py <账本> <批次起点> <本份起点文件> <本份分片命令行特征> [--once]
# 本份起点文件不存在就按当前账本写一份（开跑前）；越线或出错时先按特征杀掉本份分片再退出，
# 所以就算调它的 shell 已经没了，分片也停得下来。--once 只核一次，用于合并前的最后复核。
import json, os, subprocess, sys, time
ledger, start, run0f, pattern = sys.argv[1], json.load(open(sys.argv[2])), sys.argv[3], sys.argv[4]
once = "--once" in sys.argv[5:]
def stop(msg, code):
    print(msg, flush=True)
    if not once:
        subprocess.run(["pkill", "-f", "--", pattern])
    sys.exit(code)
def used(d):
    return d["spent_usd"] + d["reserved_usd"]          # 已花 + 在途预留
try:
    if not os.path.exists(run0f):
        d = json.load(open(ledger)); json.dump({"calls": d["calls"], "used": used(d)}, open(run0f, "x"))
    run0 = json.load(open(run0f))
    while True:
        d = json.load(open(ledger))
        run_calls, run_usd, batch_usd = d["calls"] - run0["calls"], used(d) - run0["used"], used(d) - start["spent_usd"]
        why = ("账本有待核" if d.get("review") else
               "本份 calls > 1600" if run_calls > 1600 else
               "本份 > 0.16 美元" if run_usd > 0.16 else
               "本批 > 1.2 美元" if batch_usd > 1.2 else None)
        print(time.strftime("%H:%M:%SZ", time.gmtime()), run_calls, round(run_usd, 6), round(batch_usd, 6), why, flush=True)
        if why:
            stop(f"停：{why}", 2)
        if once:
            sys.exit(0)
        time.sleep(30)
except Exception as e:                                   # 读不了账本也按越线处理
    stop(f"闸门出错，停：{e!r}", 3)
EOF

_rc_cleanup() {   # run_card 中途退出（失败、被中断、shell 被终止）时收掉本份的分片和闸门
  local p; for p in ${RC_PIDS[@]+"${RC_PIDS[@]}"} ${RC_GATE:-}; do kill $p 2>/dev/null || true; done
}
run_card() {   # $1 = --system 的值，$2 = 系统名，$3 = 缓存来源（默认 $OLD，续跑时给中断那份的 $RUNS/<RUN>）
  local sys=$1 name=$2 cache=${3:-$OLD} i rc=0 left=4
  RC_PIDS=(); RC_GATE=""; trap _rc_cleanup EXIT; trap '_rc_cleanup; exit 143' INT TERM
  RUN=$(date -u +%Y%m%d-%H%M%S)-$name
  for i in 0 1 2 3; do
    test -f "$cache-s$i/qa/asr_cache.json"
    mkdir -p "$RUNS/$RUN-s$i/qa"
    cp "$cache-s$i/qa/asr_cache.json" "$RUNS/$RUN-s$i/qa/"
    cmp "$cache-s$i/qa/asr_cache.json" "$RUNS/$RUN-s$i/qa/asr_cache.json"
  done
  local G0="$RUNS/${RUN}-gate.start.json" GLOG="$RUNS/${RUN}-gate.log" gate
  $PY $W/gate.py "$LEDGER" $W/batch-start.json "$G0" "vdn-eval scorecard .*--out $RUNS/$RUN-s" > "$GLOG" 2>&1 & gate=$!; RC_GATE=$gate
  { sleep 5 & wait $!; }; kill -0 $gate; test -f "$G0"                  # 闸门没起来、没写下本份起点，就不开跑
  local pids=()
  for i in 0 1 2 3; do
    target/release/vdn-eval scorecard --system "$sys" --no-safety --qa --rules v2 --shard $i/4 \
      --asr-ledger "$LEDGER" --out "$RUNS/$RUN-s$i" > "$RUNS/$RUN-s$i.log" 2>&1 & pids[i]=$!; RC_PIDS+=($!)
  done
  while [ $left -gt 0 ] && [ $rc = 0 ]; do               # 每 10 秒：收已结束分片的退出码、看闸门
    for i in 0 1 2 3; do
      [ -n "${pids[i]}" ] || continue
      kill -0 ${pids[i]} 2>/dev/null && continue
      if wait ${pids[i]}; then :; else echo "第 $i 片失败：${RUN}"; rc=1; fi
      pids[i]=""; left=$((left - 1))
    done
    if ! kill -0 $gate 2>/dev/null; then echo "闸门停了：${RUN}"; rc=1; fi
    [ $left -gt 0 ] && [ $rc = 0 ] && { sleep 10 & wait $! || true; }   # 后台 sleep + wait：收到信号立刻进 trap
  done
  for i in 0 1 2 3; do                                   # 出错时收掉还在跑的分片
    [ -n "${pids[i]}" ] && { kill ${pids[i]} 2>/dev/null || true; wait ${pids[i]} 2>/dev/null || true; }
  done
  kill $gate 2>/dev/null || true; wait $gate 2>/dev/null || true
  if [ $rc = 0 ]; then                                   # 合并前按最终账本再核一次
    $PY $W/gate.py "$LEDGER" $W/batch-start.json "$G0" - --once >> "$GLOG" 2>&1 || { echo "最后复核越线：${RUN}"; rc=1; }
  fi
  trap - EXIT INT TERM; RC_PIDS=(); RC_GATE=""
  [ $rc = 0 ] || { echo "有分片失败或越线：${RUN}，看 ${GLOG} 和各片日志"; return 1; }
  target/release/vdn-eval scorecard --merge-shards "$RUNS/$RUN-s0,$RUNS/$RUN-s1,$RUNS/$RUN-s2,$RUNS/$RUN-s3" \
    --suite src/eval/suite/scenes.json --out "$RUNS/$RUN"
}
```

- 每份成绩单各起一个闸门：本份起点在开跑前写进 `${RUN}-gate.start.json`，两份之间不会串账
- 每 10 秒收一次已结束分片的退出码：任何一片非 0、或闸门退出（越线、读不了账本），立刻收掉其余分片和闸门，`run_card` 返回非 0，`set -e` 让整个流程停
- 4 片都正常结束后，按最终账本再核一次（`gate.py --once`），过了才合并：最后一次轮询之后的越线也拦得住
- 中断：`run_card` 被 Ctrl-C（INT）或 TERM 打断时，trap 收掉本份的分片和闸门再退出。调它的 shell 被 `kill -9` 时 trap 跑不了，分片会接着跑完；这时闸门还在，越线照样按分片命令行杀掉它们。本机模拟过 TERM、INT、KILL 三种
- 停下以后报 evalcmp 和大总管，附 `$RUN-gate.log`。正常一份约 1030 次、0.10 美元
- **越线多少没有保证**：闸门每 30 秒看一次，已在途的请求（4 片 × 每片 2 路 = 8 路）停不回来。按补跑的速度，30 秒约发 10 次、约 0.001 美元，但这只是估计。硬兜底是程序的 15 美元软上限和 OpenRouter 侧这把密钥的 100 美元额度（D103）
- 任何一片的日志或 `meta.asr.halted` 出现停发：别自己清「待核」，照 `baseline/README.md` 用 `clear_review.py`，先报

## 五、取回（放主目录 `_tmp/<日期>-cloudrun-eval-formal/`，不覆盖）

- 每份成绩单合并目录里的 `scorecard.json`、`scorecard.md`、`clips.jsonl`，附 sha256
- 每片的 `scorecard.json`（只要 meta）和日志
- 每个模型的 `eval/builds/<系统名>-formal1/`（参数、每个输出文件的哈希、分片耗时）
- `$OUT/SHA256SUMS`、6 份导出对拍报告 `<运行名>.json`、`$W/src-diff.txt`、`$W/batch-start.json`、每份的 `$RUN-gate.log`
- 跑后的账本快照（附 sha256）和本批新增的明细行
- 不取回：音频（离线输出、`qa/asr-wav/`）、ONNX。取回前照旧查一遍有没有密钥字样

**核对（cloudrun 先核，再报）**：7 份都是 480 行；各片 `meta.asr` 没有 `error`，`halted` 为 null；每份只有一个系统；账本 `calls` 增量约等于各片 `meta.asr.calls` 之和。

## 六、取回后：配对比较与判读（evalcmp 在本机跑，不花钱，几分钟）

先照本机负载规则看 `uptime`（1 分钟负载 > 20 就等），再查有没有 `_tmp/heavy.lock`。下面 `C` 是取回目录，`X/` 是系统 X 的合并成绩单目录。用 `bash` 跑（本机默认是 zsh）。

```sh
set -euo pipefail
S=src/eval/suite/scenes.json; T=rail-air-cabin,canteen,meeting-room,transit-hub; V=target/release/vdn-eval
# A 对原版（三个种子）+ A 组种子噪声底（三对）
for s in 0 1 2; do $V compare --a $C/exp025-orig/clips.jsonl --b $C/exp025-a-s$s/clips.jsonl --by-scene --out $C/cmp/a-s$s-vs-orig; done
for p in 0-1 0-2 1-2; do x=${p%-*}; y=${p#*-}; $V compare --a $C/exp025-a-s$x/clips.jsonl --b $C/exp025-a-s$y/clips.jsonl --by-scene --out $C/cmp/noise-a-s$x-s$y; done
# B 对 A（对照都是报告种子 a-s1）+ B 组种子噪声底
for s in 0 1 2; do $V compare --a $C/exp025-a-s1/clips.jsonl --b $C/exp025-b-s$s/clips.jsonl --by-scene --out $C/cmp/b-s$s-vs-a-s1; done
for p in 0-1 0-2 1-2; do x=${p%-*}; y=${p#*-}; $V compare --a $C/exp025-b-s$x/clips.jsonl --b $C/exp025-b-s$y/clips.jsonl --by-scene --out $C/cmp/noise-b-s$x-s$y; done
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
- 任何一台跑到 5.25 小时还没跑完 → 停掉正在跑的那份（中断 `run_card`，trap 会收掉分片和闸门），已经合并好的成绩单先取回，报大总管。没跑完的那份到下一台重跑，缓存用中断那份的：`run_card <系统> <系统名> $RUNS/<中断那份的 RUN>`。它的各片缓存里已有补跑的转写和这次已付费的转写，重跑只补没识别过的
- 开下一台之前先核机费：已删机器的实际花费 + 下一台最坏（每小时价 × (时限 + 5 分钟)）≤ 1.5 美元才开。要开第三台、或要超过 1.5 美元，先报大总管批
- 回归检查不过 → 停，不跑 7 份（第三节第 5 步）
- 任何一份不是 480 行 → 不进判读，先查
