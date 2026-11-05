# 外部离线输出入口：形状草案（cands 起草，待 evalfix 回「可以」）

> 用途：让 `vdn-eval scorecard` 给「事先离线算好的降噪输出」打分，候选模型（DFN3 低延迟版 · DPDFNet-2 · NoNoise-Mac）和 GTCRN 上同一张成绩单。
> 状态：evalfix 已回「可以」并补 3 条公平性要求（2026-11-04），已按下文实现：`src/eval/src/system/external.rs` · issue #112

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
├── run.json                # 见第 3 节
├── mix/<片段 id>.wav       # 候选对混合轨 mix.wav 的输出（已对齐，去掉了声明延迟）
├── bg/<片段 id>.wav        # 候选对「噪音 + 干扰」逐样本相加的输出；只给 additive = true 的片段
└── trunc/<片段 id>.wav     # 自检：每个场景清单顺序第一段的 mix，从第 floor(n/2) 样本起补零后再跑一遍
```

- 子目录名 = 输入种类名（正本 `Kind`）；以后的 `clean/` 只在 `Kind` 加一支。run.json 没列的种类不许有子目录
- 片段 id = exp013 片段清单 `clips.jsonl` 的 `id`，不另起编号
- `bg/` 给「背景稳不稳」（`V-BGSTAB`）用：现有流程对内置模型也是把噪音 + 干扰单独再跑一遍
- wav：单声道 · **16 kHz** · 32 位浮点（推荐）或 16 位整型

## 3. `run.json`

```json
{
  "schema": "vdn-external-output/1",
  "name": "dfn3-ll",
  "clips": "docs/experiments/exp013-scene-suite/builds/20261103230756-17f5/clips.jsonl",
  "sr": 16000,
  "latency_samples": 640,
  "inputs": ["mix", "bg", "trunc"],
  "model": { "repo": "...", "weights_sha256": "...", "mode": "...", "resample": "..." }
}
```

- `clips`：片段清单路径，原样照抄 `scenes.json` 里的字符串（指针，不是副本）
- `latency_samples`：算法延迟（16 kHz 样本数，含重采样滤波的延迟），离线脚本按模型声明填；输出文件本身已对齐
- `inputs`：必须含 `mix` 与 `trunc`；没列 `bg` 时，`V-BGSTAB` 那一路报「外部输出没提供 bg」进「没跑成」，不回退
- `model`：自由字段，原样进成绩单元数据

## 4. 入口怎么算（不碰 `scorecard/` · `verifiers/` · `runner`）

1. `SystemKind::parse("external:<目录>")` 读 run.json 与片段清单并自检（第 5 节）；不过就 `eprintln!` 原因、返回 `None`
2. 索引：**键 = 输入样本指纹**（长度 + 两个 64 位哈希，按 `f32` 位模式算；Rust 自己经 `scorecard::load_scene_clip` 读输入、按 `Kind::input` 构造）→ 值 = 输出文件。`system::run(spec, x)` 只凭 `x` 找输出
3. 输出开头补 `latency_samples` 个零还原成「带延迟的原始流」，包成回放模型放进同一个 `Engine`（三件保命照开），逐 hop = 256 吐出
4. 公平性三条（evalfix 审）：
   - 延迟只是声明值：成绩单元数据 `external[].latency_declared_only = true`，`V-LAT` 只能证明「与声明一致」
   - 回放不耗时：外部系统 `engine_times` 给空，计时类项判「不知道」
   - 偷看未来：`trunc/` 输出在 `[0, n/2 − latency_samples)` 上与 `mix/` 输出逐样本差 ≤ 1e-4（任一为 16 位整型时 2/32768），超了整个系统不打分
5. 成绩单元数据 `external[]`：名字 · 目录（相对主目录）· run.json 的 sha256 · 输入种类 · 声明延迟 · 截半自检段数与最大差 · `model`

## 5. 自检（对不上就报错不打分）

| 项 | 规则 |
|---|---|
| 片段数 | 每种列出的输入：适用片段一个不缺、清单外的一个不多 |
| 采样率 · 时长 · 数值 | 16000 · = `dur_samples` · 全是有限数 |
| 撞键 | 两路输入样本完全相同 → 报错 |
| 名字 | `[a-z0-9-]+`，不撞内置 |
| 截半 | 第 4 节第 4 条 |

## 6. 测试

`src/eval/tests/external_e2e.rs`（6 条）+ `external.rs` 单元测试（4 条）：逐位一致（外部回放内置 passthrough = 内置成绩单逐位相等，且外部「没跑成」为 0）· 声明延迟补回 · 偷看 L+100 被拒、如实声明通过 · 没提供 bg 报错 · 8 条自检负例 · 命令行解析。

## 7. 要 evalfix 确认的三件事

1. 新增 `src/eval/src/external.rs` + `system.rs` 约 30 行分发改动，和你的改动面不冲突吗？（我不碰 `scorecard/` · `verifiers/` · `thresholds/` · `qa/` · `runner`）
2. `bg/`：你的新规则还用不用「对噪音 + 干扰单独再跑一遍」？不用了我就不要求 `bg/`
3. 成绩单 JSON 的系统名直接用 `run.json.name`（如 `dfn3-ll`）可以吗？你那边按系统名做配对差值，名字规则（第 1 节）有没有要加的
