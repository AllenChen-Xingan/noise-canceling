# vdn-eval 任务清单格式（schema）

解析器正本：`src/eval/src/suite.rs` 的 `parse_suite`；现有清单：`src/eval/suite/ci.json`（CI 层）、`src/eval/suite/qa.json`（QA 层）。新清单照同一格式写，**人判任务放 `docs/experiments/human-prefs/`**，接入时由 vdn-eval 读取（加载入口待接，不改现有两份）。

## 顶层

```json
{ "name": "exp007-ci", "doc": "一句说明", "seed": 20261031,
  "defaults": { "dur_s": 10, "trials": 3 },
  "tasks": [ … ] }
```

## 任务字段

| 字段 | 必填 | 取值 | 说明 |
|---|---|---|---|
| `id` | ✓ | 字符串，全清单唯一 | 惯例 `R`回归 / `C`能力 / `P`探针 / `Q`真实片段 + 两位号 + slug |
| `set` | ✓ | `regression` · `capability` · `probe` | 门禁规则见 `src/eval/src/runner.rs` 头注释 |
| `polarity` |  | `suppress` · `preserve` · `both` · `robust` | 该压的 / 不该压的（正反两面） |
| `origin` | ✓（测试强制） | 字符串 | 取自哪次真实失败：实验号 + 片段 + 人原话或标签 |
| `desc` |  | 字符串 | 一句话 |
| `scene` | 三选一 | 对象，见下 | 合成场景（CI） |
| `real` | 三选一 | `{kind:"exp005", clips, audio_dir, clip}` 或 `{kind:"exp003", clips, clip}` | 仓外真实片段，按 clips.json 里的 sha256 核对 |
| `probe` | 三选一 | `{base:"<任务 id>", kind}`；kind ∈ `onset_cut_50ms` · `nonspeech_minus40` · `lowpass_1k` · `identity` · `mute` | 只在 `set: probe` 用 |
| `graders` | ✓ | `[{ "v": "<verifier id>", <参数>… }]` | 参数名：`max` / `min`（过线值）及各 verifier 自己的（如 V-SIM-GATE 的 `onset_max`、V-BGSTAB 的 `from_s`、V-MUTE 的 `max_dbfs` `settle_s`） |
| `info` |  | 同 graders | 只报，不计通过与门禁 |
| `trials` |  | 整数 | 合成任务 = 种子数；真实片段固定 1 |
| `safety` |  | bool，默认 true | false = 三件保命全关 |
| `faults` |  | `[{hop, kind}]`，kind ∈ `nan` · `error` · `explode` | 故障注入 |
| `lang` |  | `en` · `zh` | ASR 分词 |
| `xfail` |  | 字符串（原因 / 代码位置 / issue） | 已知失败：阻断降为报警；全过时提示转正 |

verifier id 必须在 `src/eval/src/verifiers.rs` 的 `REGISTRY` 里（`cargo run -p vdn-eval -- list` 列出），否则解析失败。

## scene

```json
{ "owner": { "level_dbov": -26, "f0": [100, 140], "tonal": false },   // null = 没有主讲人
  "owner_start_s": 0.5, "pause_s": [0.4, 1.2],
  "interferers": [ { "mode": "alternate|overlap|other_first", "sir_db": 6, "f0": [180, 230] } ],
  "noise": "none|white|pink|hum|babble|cafe|music", "snr_db": 10,
  "dur_s": 10, "tail_silence_s": 0 }
```

## 人判结果怎么接（建议，给 prefs）

人耳给的是「同屏谁更好」或「能用 / 勉强 / 不能用」，不是 verifier 分数。两种接法：
1. **定标数据**（主用）：答卷 JSON 按 exp003 的格式（`trials[].kind/clip/slots/by_model.{clearer,natural,usable}`）放 `docs/experiments/human-prefs/`，`vdn-eval calib --answers <答卷> --scores <指标表>` 读取；指标表格式同 exp003 `scores.json` 的 `per_clip[clip][model]`
2. **任务**：人标过「坏」的片段写成 `real` 任务，`origin` 写人原话 / 标签，`graders` 放应当抓住它的 verifier；人标「好」的写成 `polarity: preserve` 的反例任务
