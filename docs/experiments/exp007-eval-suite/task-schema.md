# vdn-eval 任务清单格式（schema）

解析器正本：`src/eval/src/suite.rs` 的 `parse_suite`；现有清单：`src/eval/suite/ci.json`（CI 层）、`src/eval/suite/qa.json`（QA 层）。新清单照同一格式写，**人判任务放 `docs/experiments/human-prefs/`**，接入时由 vdn-eval 读取（加载入口待接，不改现有两份）。

## 格式正本在哪

本文不再手写字段表（手写的一份会和代码走散）。字段、取值、缺省值以这两处为准：

- **Rust 类型（正本）**：`src/eval/src/files.rs` 的 `SuiteFile` · `TaskFile` · `SceneFile` 等，每个字段的注释写了取值与缺省值
- **导出的 JSON Schema**：`specs/schema/eval-suite.schema.json`（由 `cargo run -p vdn-schema` 导出，测试逐字比对；`ci.json` / `qa.json` 必须过它）。编辑器里给清单加 `"$schema"` 不行（多出的键会报错），可在编辑器设置里把这份 schema 关联到 `src/eval/suite/*.json`

要点（详情看上面两处）：
- 多出的键一律报错，防拼错键名静默失效；给人看的备注写在任务的 `note` 里
- `id` 全清单唯一，惯例 `R` 回归 / `C` 能力 / `P` 探针 / `Q` 真实片段 + 两位号 + slug
- `scene` · `real` · `probe` 至少一个；`probe` 只在 `set: probe` 用，基础任务必须是本清单里的合成场景
- `graders[].v` 必须在 `src/eval/src/verifiers.rs` 的 `REGISTRY` 里（schema 里是同一份枚举，`cargo run -p vdn-eval -- list` 列出）；其余键是数值参数
- 门禁规则见 `src/eval/src/runner.rs` 头注释

## 人判结果怎么接（建议，给 prefs）

人耳给的是「同屏谁更好」或「能用 / 勉强 / 不能用」，不是 verifier 分数。两种接法：
1. **定标数据**（主用）：答卷 JSON 按 exp003 的格式（`trials[].kind/clip/slots/by_model.{clearer,natural,usable}`）放 `docs/experiments/human-prefs/`，`vdn-eval calib --answers <答卷> --scores <指标表>` 读取；指标表格式同 exp003 `scores.json` 的 `per_clip[clip][model]`
2. **任务**：人标过「坏」的片段写成 `real` 任务，`origin` 写人原话 / 标签，`graders` 放应当抓住它的 verifier；人标「好」的写成 `polarity: preserve` 的反例任务
