# 现状页 · single（批次 20261102-b9 · 分支 `feat/types-single-source` · issue #56）

管的实体：`vdn-types`（参数描述）· `vdn-speaker` · `vdn-polish` · `vdn-scene`。

## 做到哪
- [x] 读交代、规约、三个镜像与使用方；报到（大总管同意：先比对，再按 polish → speaker → scene 删）
- [x] 镜像 ↔ 正本逐项比对（下表）
- [x] 删 polish 镜像（`SpectralStage` 按方案 A 留在 polish，改名 `spectral_stage.rs`）
- [x] 删 speaker 镜像（speaker-lab 一并改用 `vdn-types`）
- [x] 删 scene 镜像（训练脚本 train.py 生成的 import 一并改）；加测试：路由表名字经 `vdn-types` 解析回来 · 标定滞回 = 正本默认
- [x] 参数描述表 `src/types/src/desc.rs`（39 条；codex 第 1 轮后补 speaker 判断回路 8 条 + 注册器 11 条，共 58 条）；`ParamField` · `ParamSet` · `Hysteresis` · `Transition` 改从表取
- [x] speaker（`GateParams` · `EnvelopeParams` · `FloorStage`）· polish（`PolishParams` · `PolishChain`）· scene（`set_tuning`）· engine（`SafetyConfig::default`、Pipeline 淡化时长夹值）· cli（默认值与越界检查）改从表取；各自带「同表 + 越界拒收 / 夹住」单测；`validate()` 都穷举解构防漏字段
- [x] `xtask tidy` 新规则 R-MIRROR · R-PARAM（`src/xtask/src/single_source.rs`，含正负例测试与「整仓通过」测试）；speaker 的 VAD `Hysteresis` 写了同名放行理由
- [x] 改 `.42cog/cog.md`（types · speaker · polish · scene 四行）、`src/speaker/README.md`
- [x] 编译测试（锁空档，2026-11-02 约 20:2x）：`cargo run -p xtask -- ci` 全绿（含 `test --workspace`）。首跑被新 tidy 规则抓到 polish AGC 的私有底噪变量 `floor_db`（撞参数名），改名 `noise_floor_db`，行为不变
- [x] codex 第 1 轮：P1×1 P2×5 全部属实已改（记录 `docs/reviews/011_types-single-source-codex.md`）
- [x] codex 第 2 轮：P0×0 P1×0 P2×5，全部属实已改
- [x] codex 第 3 轮：P0×0 P1×0 P2×3，全部属实已改；**连续两轮无 P0/P1，已达停止线**
- [x] 复跑编译测试（锁空档，2026-11-02）：`cargo fmt --all` + `cargo run -p xtask -- ci` 全绿；测试 372 过 · 0 败 · 5 忽略（都是本分支之前就有的人工跑测试，本分支没加 `#[ignore]`）
- [x] PR 正文草稿 `state/handoff/single-pr.md`

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
| 9 | polish | `SpectralStage`（频域后处理 trait）+ `Complex32` 再导出 | 有 | **没有** | ⚠️ 正本缺类型，已报大总管，定方案 A：它不是镜像而是 polish 自己的草案扩展 trait，留在 polish，搬到 `spectral_stage.rs` |

## 下一步
收尾报告已发大总管；等验收。不做 git 写操作。

## 技术决定（我定的，理由）
- **描述表的范围是 AI 按物理意义 + 现有取值定的**（默认值全部沿用原初值，行为不变）；写在 `desc.rs` 头注释「范围待标定」
- **同义参数共用一条**：`floor_db` 被 ParamSet · engine · 门控 · 包络共用；`open_db` / `release_s` / `hangover_s` / `attack_s` 被 ParamSet（或 engine）与包络共用；`vad_on` / `vad_off` 被门控与包络共用。**同名不同义分开**：修音链 `polish.floor_db`（频谱增益下限，−100 = 关）、门控 `gate.release_s`（0.5 s）、`gate.attack_s`（5 ms）
- **门控快路径 `gate.level_close_db` 范围 0–60 dB**（语义是「比主人电平低多少」，负数无意义）。speaker-lab 原有一条解析测试用 −40，改成 12，另加一条「−40 被拒」的测试
- **使用方两件套**：`validate()` 越界拒收（命令行 / 配置入口用）；`clamped()` 夹进范围（各 `new()` / `set_target` / `set_tuning` 先夹再用，实时侧不会见到越界值）
- 不进描述表的：采样率 `sr` / `clock_rate`（接线配置）、开关量（`fallback` / `floor` / `fast_open`）、`deadline`（`Option<Duration>`）、分类器阈值（`BT_HF_P90_MAX` / `CLIP_MIN`，训练产物）、修音链预设表（数据，单测保证每个预设都在范围内）
- tidy 不读 `vdn-types`（xtask 的 forbid 禁止依赖它），按文本解析 `desc.rs`；边界写在 `single_source.rs` 头注释

## R-PARAM 挡得住 / 挡不住（读规则源码 + 正负例单测，`src/xtask/src/single_source.rs`）

前提：同一行代码（去掉 `//` 注释后）里既有参数名（描述表键名末段如 `floor_db`，或驼峰 `FloorDb`），又有下面某种「两个数字字面量」写的范围，才报。

挡得住（单测里逐条有负例）：
- `(-30.0..=0.0).contains(&p.floor_db)` · `(-60.0..0.0)`（区间字面量）
- `floor_db.clamp(-60.0, 0.0)` · 带后缀 `clamp(-60f32, 0f32)`
- `floor_db.max(-60.0).min(0.0)` · `confirm_n.min(20).max(1)`
- `ParamField::FloorDb => (-60.0, 0.0)`（match 臂返回范围元组）
- 文件名叫 `mirror.rs`；vdn-types 外定义与 vdn-types 公开类型同名的 struct / enum / trait / type

挡不住（已知边界）：
- **常量运算 / 别名**：`const LO: f32 = -60.0;` 再 `floor_db.clamp(LO, HI)`；`let lo = -60.0;` 后用 `lo`；`(-60.0 * 1.0, 0.0)` 这类表达式
- **跨行**：rustfmt 把 `clamp(` 的参数拆到多行
- **参数名不在同一行**：先 `let r = -60.0..=0.0;` 再在别处 `r.contains(&x)`；或变量改了名（`let f = p.floor_db; f.clamp(-60.0, 0.0)`）
- **单边界**：只写一边（`floor_db.max(-60.0)`、`if floor_db < -60.0`）
- **默认值字面量**（`floor_db: -12.0`）不查：测试造输入也写字面量，文本上分不开；默认值同源靠各 crate 单测比对描述表
- **镜像改了名**：手抄副本换个类型名就查不到；同名但字段不同也只按名字判
- 放行写法 `// tidy: 非参数范围：<理由>` / `tidy: 同名不同物：<理由>` 理由内容不查真假

补位：各使用方的 `validate()` 穷举解构 + 单测「取描述表边界外一点造输入 → 拒收 / 夹住」，保证**已有**字段与表同源；tidy 只防**新写**的字面量范围。

## 转交
1. **给第二阶段处理图**：`vdn_polish::spectral_stage::SpectralStage`（频域后处理 trait）在 `vdn-types` 里没有对应物；接 engine 时定要不要搬进 `vdn-types`（大总管定方案 A）
2. **FE-M 模型名两种写法（不在我实体内，未改）**：`vdn-types` 的 `ModelId` 用 `fastenhancer_m`；以下仍用 `fastenhancer-m`：命令行 `vdn --model`（`src/cli/src/main.rs:188`）· `vdn-denoised --model`（`src/denoised/src/main.rs:55`）· 配置 JSON `model`（`src/control/src/lib.rs:296` 的测试）· 评测系统名（`src/eval/src/system.rs:184,193`）与基线 `src/eval/suite/baseline-ci.json` 的键 · exp004/exp006/exp007/human-prefs 的已提交结果与脚本。历史结果不改；建议由 schema 线或处理图线统一：解析处同时认两种写法，或给这几处加「名字能被 `ModelId::parse` 解析」的交叉测试

## 待拍板
（无。`SpectralStage` 放哪：大总管定 A，2026-11-02）
