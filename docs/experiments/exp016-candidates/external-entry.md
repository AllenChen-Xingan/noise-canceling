# 外部离线输出入口：形状草案（cands 起草，待 evalfix 回「可以」）

> 用途：让 `vdn-eval scorecard` 给「事先离线算好的降噪输出」打分，候选模型（DFN3 低延迟版 · DPDFNet-2 · NoNoise-Mac）和 GTCRN 上同一张成绩单。
> 状态：草案 · 2026-11-04 · issue #112

## 1. 命令行

```
vdn-eval scorecard --system gtcrn,external:_tmp/exp016/dfn3-ll/<run-id> --qa
```

- 写法 `external:<目录>`，目录相对主目录（`main_root()`），也可以是绝对路径
- **系统名不在命令行写**，只取目录里 `run.json` 的 `name`（一个正本）；成绩单、配对差值里用这个名
- 名字规则：`[a-z0-9-]+`，不能和内置名（`passthrough` · `ref` · `gtcrn` · `fastenhancer-m` 及其别名）撞；撞了报错

## 2. 目录结构

```
_tmp/exp016/<候选>/<run-id>/
├── run.json              # 见第 3 节
├── out/<片段 id>.wav     # 候选对混合轨 mix.wav 的输出
└── bg/<片段 id>.wav      # 候选对「噪音 + 干扰」（noise.wav + interf.wav 逐样本相加）的输出；只给 additive = true 的片段
```

- 片段 id = exp013 片段清单 `clips.jsonl` 的 `id`（如 `bus-metro-cabin-c00-ext`），不另起编号
- `bg/` 给「背景稳不稳」（`V-BGSTAB`）用：现有流程对 GTCRN 也是把噪音 + 干扰单独再跑一遍
- wav：单声道 · **16 kHz** · 32 位浮点（推荐，免量化）或 16 位整型都收

## 3. `run.json`

```json
{
  "schema": "vdn-external-output/1",
  "name": "dfn3-ll",
  "clips": "docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/clips.jsonl",
  "sr": 16000,
  "latency_samples": 640,
  "model": { "repo": "...", "weights_sha256": "...", "mode": "...", "resample": "16k→48k→16k soxr" }
}
```

- `clips`：片段清单路径，**原样照抄 `scenes.json` 里的那个字符串**（指针，不是副本）
- `latency_samples`：候选的算法延迟（16 kHz 样本数），由离线脚本按模型声明填；输出文件本身**已对齐**（离线脚本已把延迟去掉）
- `model`：自由字段，入口不读，只进成绩单元数据，方便追溯

## 4. 入口怎么算（不碰 `scorecard/` · `verifiers/` · `runner`）

1. `SystemKind::parse("external:<目录>")` 读 `run.json` 与片段清单，做自检（第 5 节）；任一项不过就 `eprintln!` 原因并返回 `None`（`main.rs` 不改）
2. 自检时为每段建索引：**键 = 输入样本的 sha256**（Rust 自己读 `mix.wav`、自己把 `noise + interf` 相加后算）→ 值 = `out/` 或 `bg/` 里的文件。这样 `system::run(spec, x)` 只凭输入 `x` 就找得到对应输出，不用改 `runner` 和 `scorecard` 传片段 id
3. 输出包成一个「回放模型」放进**同一个 `Engine`**（三件保命照开）：先在开头补 `latency_samples` 个零还原成「带延迟的原始流」，再按 hop = 256 逐 hop 吐给 Engine。门控、送原声、延迟账、对齐和 GTCRN 完全同一口径
4. `system.rs` 只改：`SystemKind` 加一个 `External(&'static External)` 变体、`parse` / `as_str` / `available` / `EvalModel` 分发各加一支、`build` 多收一个 `x`。实现全在新文件 `src/eval/src/external.rs`（预计 < 300 行，含测试）
5. 查不到输入（输出目录与成绩单清单不是同一 build）→ `run` 返回错误 → 这段进成绩单的「没跑成」，不静默跳过

## 5. 自检（对不上就报错不打分）

| 项 | 规则 |
|---|---|
| 片段数 | 清单里 `has_clean_ref = true` 的每段都必须有 `out/<id>.wav`；`additive = true` 的还必须有 `bg/<id>.wav`；目录里多出清单没有的 wav 也报错 |
| 采样率 | 每个文件 = 16000，且 = `run.json.sr` |
| 时长 | 每个文件样本数 = 清单 `dur_samples` |
| 数值 | 不许 NaN / Inf（防止离线脚本写坏；Engine 的保命也会兜，但要在入口就报） |
| 撞键 | 两段输入样本完全相同 → 报错（索引会错配） |
| 名字 | 第 1 节规则 |

## 6. 测试

- **逐位一致**（主测试）：小片段清单（临时目录，合成 3 段）→ 内置 `passthrough` 跑一遍 → 把它的模型输出（= 输入）写成 `out/` · `bg/`，`latency_samples = 0` → `external:` 打分 → 两边每段每项分数逐位相等（只差系统名）
- **延迟还原**：用 `DelayGain` 式的已知延迟，写对齐后的输出 + `latency_samples = L`，`V-LAT` 量出的算法延迟 = L
- 自检每条一个会红的测试：缺片段 · 多片段 · 采样率错 · 时长错 · NaN · 名字撞内置
- 全部 `cargo test -p vdn-eval external`，单 crate、不占重活名额

## 7. 要 evalfix 确认的三件事

1. 新增 `src/eval/src/external.rs` + `system.rs` 约 30 行分发改动，和你的改动面不冲突吗？（我不碰 `scorecard/` · `verifiers/` · `thresholds/` · `qa/` · `runner`）
2. `bg/`：你的新规则还用不用「对噪音 + 干扰单独再跑一遍」？不用了我就不要求 `bg/`
3. 成绩单 JSON 的系统名直接用 `run.json.name`（如 `dfn3-ll`）可以吗？你那边按系统名做配对差值，名字规则（第 1 节）有没有要加的
