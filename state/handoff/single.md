# 现状页 · single（批次 20261102-b9 · 分支 `feat/types-single-source` · issue #56）

管的实体：`vdn-types`（参数描述）· `vdn-speaker` · `vdn-polish` · `vdn-scene`。

## 做到哪
- [x] 读交代、规约、三个镜像与使用方；报到（大总管同意：先比对，再按 polish → speaker → scene 删）
- [x] 镜像 ↔ 正本逐项比对（下表）
- [ ] 删 polish 镜像
- [ ] 删 speaker 镜像
- [ ] 删 scene 镜像
- [ ] 参数描述表（`vdn-types`）+ 各使用方改从表取
- [ ] `xtask tidy` 新规则 + 正负例测试
- [ ] 改 `.42cog/cog.md` 三行、`src/speaker/README.md`
- [ ] 全量测试 · codex 评审

## 镜像 ↔ 正本分歧（读源码比对，2026-11-02）

以 `vdn-types` 为准。「处理」一栏是我的做法。

| # | 镜像 | 东西 | 镜像里 | 正本 `vdn-types` 里 | 处理 |
|---|---|---|---|---|---|
| 1 | scene | `ModelId::as_str(FastEnhancerM)` | `"fastenhancer-m"`（连字符） | `"fastenhancer_m"`（下划线） | 用正本。影响：`route::table_text()` 输出里这一列改成下划线；仓内没有别处读这段文本（grep 过） |
| 2 | scene | `SampleClock` 的单位 | 模型采样率（离线 16 kHz） | IO 采样率 | 类型直接换成正本；scene 内部仍按「喂进来的采样率」算秒（`step(label, rate)` 传同一个 rate），逻辑不变。注释改成「`rate` 与 `at` 同口径」 |
| 3 | scene | `Hysteresis` 默认值 | 取 `weights::HYSTERESIS`（训练脚本生成，1.0 / 0.4 / 3.0） | 写死 1.0 / 0.4 / 3.0 | 数值一致。正本改从描述表取；`weights::HYSTERESIS` 是标定产物，加测试比对二者 |
| 4 | scene | `HysteresisGate::new` | 显式写初值 | `..Default::default()` | 初值相同（`committed_user` 默认 Quiet），无行为差 |
| 5 | scene | `ModelId` / `UserScene` 等 | 少 `ALL` / `parse` / `Hash` 派生 | 有 | 用正本，是超集 |
| 6 | speaker | `SpeakerVerdict` 默认值 | `derive(Default)` | 手写 `Default`，值相同 | 无差 |
| 7 | speaker | `Stage` | 没有 `set_params` | 有 `set_params`（默认空） | 用正本；门控 / 包络暂不实现 `set_params`（接 engine 是第二阶段的事） |
| 8 | polish | `FrameCtx` | 只有 `voice_prob` | `now` · `speaker` · `voice_prob` | 用正本（超集）；调用处用 `..Default::default()` |
| 9 | polish | `SpectralStage`（频域后处理 trait）+ `Complex32` 再导出 | 有 | **没有** | ⚠️ 正本缺类型，已报大总管（见待拍板）。在得到答复前，我的建议做法：它不是镜像而是 polish 自己的草案扩展 trait，留在 polish、改名搬出 `mirror.rs` |

## 下一步
删 polish 镜像（`FrameCtx` / `Stage` 改用 `vdn_types`）。

## 转交
（暂无）

## 待拍板（大总管定，技术题）
1. `SpectralStage` 放哪：A 留在 `vdn-polish`（新文件 `spectral_stage.rs`，不再叫镜像），第二阶段处理图再定是否进 `vdn-types` · B 现在加进 `vdn-types`（要自带复数类型，因为 `vdn-types` 零依赖）。建议 A：不扩大 `vdn-types`，接 engine 时一起定。不回默认 A。
