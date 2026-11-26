# exp025 · 甲路（外部离线输出）离线脚本与两条路对拍

用途：把整段版 GTCRN（`trainer/export.py` 导出的 ONNX）离线跑成「外部输出目录」，交给
`vdn-eval scorecard --system external:<目录>` 打分；再拿**内置流式 GTCRN** 的同一张成绩单逐段逐项对拍，
证明甲路与内置路同口径（`eval-plan.md` 第二节第 2 步：原版（甲）只能用来对候选（甲））。

两个脚本：`run_gtcrn_offline.py`（跑输出）· `parity.py`（对拍）。测试：`test_offline.py`（不跑真模型）。

## 前提

- Python 环境要有 `numpy` · `soundfile` · `onnxruntime`；导出 ONNX 还要 `torch`。用 trainer 的环境就够：
  `uv venv --python 3.13 .venv && uv pip install -p .venv -r docs/experiments/exp025-gtcrn-train/trainer/requirements.txt`
- `vdn-eval` 与两端同一提交、同一个 build（云上也照 `baseline/README.md` 第一节）
- 内置流式权重：`resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx`（或设 `VDN_GTCRN_ONNX`）

下面 5 步都在**项目根**下执行，`PY` = 上面那个 python，`MAIN` = 主目录（git 公共目录的父目录；`run_gtcrn_offline.py` 只接受工作副本或主目录里的路径）。

## 1. 冒烟测试集（≤ 5 段，只缩小成绩单范围）

```sh
IDS=$(python3 - <<'EOF'
import json
s = json.load(open("src/eval/suite/scenes.json"))
p = {t["real"]["clips"] for t in s["tasks"]}.pop()
rows = [json.loads(l) for l in open(p)]
print(",".join(r["id"] for r in rows[:5]))
EOF
)
python3 docs/experiments/exp025-gtcrn-train/baseline/subset.py --clips "$IDS" --out "$MAIN/_tmp/exp025-smoke"/suite
```

## 2. 导出整段版 ONNX（原版（甲）= 官方起点权重）

```sh
(cd docs/experiments/exp025-gtcrn-train/trainer && \
  $PY export.py --official --out "$MAIN/_tmp/exp025-smoke"/gtcrn-official.onnx)
```

导出脚本会另写同名 `.json`（含与 PyTorch 的对拍与输入名）；对拍不过就停，别往下走。

## 3. 离线跑（音频写主目录 `_tmp/`，清单写本目录 `builds/`）

```sh
$PY docs/experiments/exp025-gtcrn-train/eval/run_gtcrn_offline.py \
  --onnx "$MAIN/_tmp/exp025-smoke"/gtcrn-official.onnx --name gtcrn-offline --probe   # 先量前视 / 对齐 / 一段耗时
$PY .../run_gtcrn_offline.py --onnx … --name gtcrn-offline --run-id r1 --shard 0/1   # 多进程：每片一个 --shard
$PY .../run_gtcrn_offline.py --onnx … --name gtcrn-offline --run-id r1 --finalize    # 全部跑完再收尾一次
```

输出：`_tmp/exp016/gtcrn-offline/r1/{mix,bg,trunc,clean}/`（`run.json` 最后才写）；清单在
`docs/experiments/exp025-gtcrn-train/eval/builds/gtcrn-offline-r1/`（进仓）。

⚠️ **这一步必须跑全量 480 段**：入口 `external.rs` 按 `run.json` 里那份片段清单逐段自检（每种输入一个不缺、
一个不多），缺一段整个系统不打分。所以 `--limit` 只能配 `--probe` 做冒烟，不能拿去 `--finalize`；
第 1 步的 5 段只省成绩单的时间，不省这一步。

## 4. 成绩单（两个系统一起跑，一份 `clips.jsonl` 带 `system` 字段）

```sh
VDN_GTCRN_ONNX=resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx \
  target/release/vdn-eval scorecard --suite "$MAIN/_tmp/exp025-smoke"/suite/suite.json \
  --system gtcrn,external:_tmp/exp016/gtcrn-offline/r1 \
  --no-safety --rules v2 --out "$MAIN/_tmp/exp025-smoke"/score
```

⚠️ **不要加 `--qa`**（要买识别服务）。没有 `--qa` 时吃字、两个听感项写 `null`，对拍会自动把它们算作
「两边没有共同值」，不影响其它项。

## 5. 对拍

```sh
$PY docs/experiments/exp025-gtcrn-train/eval/parity.py \
  --a "$MAIN/_tmp/exp025-smoke"/score/clips.jsonl --b "$MAIN/_tmp/exp025-smoke"/score/clips.jsonl \
  --system-a gtcrn --system-b gtcrn-offline
```

打印一张短表（项 · 配对段数 · 最大绝对差 · PASS/FAIL），退出码 0 = 全部在容差内。

- 容差 `parity.TOL = 1e-3`（verifier 原单位）：两条路只在 STFT 边界与 float32 舍入上不同（1e-6 量级），
  而最小判定线是 0.05 / 1 dB；超了就是真不同，不是噪声
- **跳过的项只有两项**，表下面逐个列出：延迟（`V-LAT`，外部只是声明值）与最坏一帧（`V-FRAME`，回放不耗时）；
  出处 `src/eval/src/system/external.rs:9-10`
- 只在一边出现的片段、或一项都没比成 → 也算 FAIL

判读：PASS 只说明**实际比上的那些段和项**两条路同口径（「未验证」的项不算；不带 `--qa` 时吃字与两个听感项都未验证，要带 `--qa` 另核），甲路这些项的读数可以拿来对原版；任一项 FAIL → 先查 STFT 边界 / 对齐，**别**拿甲路读数下结论。

## 已知限制

- 甲路的延迟只是声明值，产品延迟仍看乙路（内置流式）；`V-LAT` / `V-FRAME` 不比
- 声明延迟由 `_ext.probe` 实测前视后取「标称 256 样本（`src/models/src/lib.rs`）与实测值的较大者」；
  STFT 分帧会看到比 256 更多一点的未来（窗长 − 帧移 = 256 是输出延迟，对齐后逐样本最远看 511），
  所以 `run.json` 里的数一般略大于 256，这是截半自检（查「偷看未来」）要求的下限
- `parity.py` 只判「两条路读数一不一样」，不判好坏；好坏看 `compare`（规约第十节）
## 实测（2026-11-26，evalcmp 在本机跑）

- 环境：trainer 的 `.venv`（torch · onnxruntime，只读调用）；`vdn-eval` release 版，提交见本 PR
- 官方权重整段版 ONNX（`export.py --official`，导出对拍过，ONNX 离 float64 真值 ≤ 4.9e-7）
- 离线推理全量 480 段 · 1451 个文件，4 片并发约 3.2 分钟（0.045 倍实时）。探针：前视 165 样本，对齐后滞后 0，声明延迟 256
- 冒烟集 5 段（transit-hub · meeting-room · rail-air-cabin · canteen · cowork 各 1 段，都是 c00-ext），不带 `--qa`
- 对拍：**PASS**，22 项 76 对，最大绝对差 3.8e-5（V-BGSTAB），容差 1e-3；单侧缺值 0 处；9 项两边都没有值、未验证（不带 `--qa` 时吃字与两个听感项为空，其余是该段不适用）。原样输出与哈希：`runs/parity-20261126/`
- ⚠️ 本机负载：4 片并发每片约 170% CPU。之后本机最多同时 2 片，开跑前看 `uptime`
