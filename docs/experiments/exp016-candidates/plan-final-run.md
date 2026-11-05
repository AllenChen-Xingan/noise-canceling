# 方法：两个候选的最终版（mix + clean 新 run · 带评分模型的成绩单 · 与 GTCRN 配对）

> 状态：**待跑**。开跑前 codex 审 1 轮（大总管定 2026-11-05），只修 P0/P1。前提：evalfix 的 PR（成绩单新规则 v2 + 外部入口 `clean/`）已合进 main，本分支 merge main 后再跑。

## 一、要回答什么

GTCRN 在哪几个开会场景、哪种声音上落后于两个候选（DPDFNet-2 · DFN3 低延迟版），落后多少。最终版和草稿的区别：规则换成 v2；带评分模型（`--qa`：听感分 · 吃字）；干净人声单独过一遍的三项（人声忽大忽小 `V-PUMPV` · 音色 `V-TIMBRE` · 弱音 `V-WEAK`）能判。

## 二、步骤（全部要名额的步骤用同一个名额顺序跑）

1. **merge main**，补 `system.rs` 的 `SystemKind::native_rate()` 外部一支（返回 `Some(e.native_rate())`）+ 测试「声明 48000 的外部系统，延迟账比 16000 少两跳重采样 = 4 ms」。单 crate 增量编译 + 单测，不占名额
2. **冒烟**（占名额，各 ≤ 1 分钟）：`run_dpdfnet2.py --probe` · `run_dfn3ll.py --probe`。过线：
   - **mix 与 clean 两种输入各自**截半量前视、互相关核对齐（`_ext.probe`，`measured_lookahead_by_kind` · `lag_after_align_by_kind` 进 `params.json`）；声明延迟取两者与标称的最大值，任一滞后 > 1 样本即停。测试：只在 clean 上偷看 50 样本的假模型会被量出（`tests/test_ext.py::Probe`）。入口的截半自检只查 mix，clean 的因果性由这一步把关
   - A→B→A 差 ≤ 1e-6：每次 `process` 开头复位模型状态（DFN3：`weya_nc_reset` 同一会话；DPDFNet-2：`state = state0.copy()`），A→B→A 一致是复位干净的实测证据；复位 ≠ 新建实例，若 A→B→A 不过即停
   - 与 runner 对内置系统的口径对照【读源码 `src/eval/src/runner.rs` `prepare_clip`，evalfix 分支】：干净人声那一路用同一系统、`faults: Vec::new()`（不注入故障）、新的 `run` 调用（新引擎状态）；外部回放同样进新引擎、同一声明延迟补回
3. **离线跑音频，各开新 run-id**（不续旧 run、不覆盖）：`run_<候选>.py`（单分片）→ `--finalize --run-id <新 id>`。每个候选 1451 个文件 = 480 mix + 480 clean + bg（additive 段）+ 15 trunc。`finalize` 逐个校验 wav、原子发布 `run.json`（`inputs` 含 `clean`；DFN3 写 `native_rate: 48000`）。耗时按上次实测 × 1451/971 推算：DPDFNet-2 约 21 分钟 · DFN3 约 23 分钟
4. **跑前核对**（不占名额，结果写进 `runs/<run-id>/README.md`）：
   - 规则版本：两边都 `--rules v2`（成绩单 `meta.rules`）
   - 场景集与清单：两边 `meta.suite` 同为 `src/eval/suite/scenes.json`，外部 `run.json.clips` 指向 exp013 build `20261103230756-17f5`，其 sha256 与 `builds/…/params.json` 的 `clips_sha256` 相同
   - 评分代码：`git diff --stat 4ccf0638 <本次提交> -- src/ Cargo.lock`，排除外部入口（`system.rs` 分发 · `system/external.rs` · `scorecard/mod.rs` 里 `provides` 那一处 · `tests/`）后**必须为空**；二进制用 `cargo build --release` 在本次提交上重编（`meta.commit` 不带 `-dirty`）。不为空 → 停下，报大总管，选项是把 `gtcrn` 一起放进本次成绩单（约多 55 分钟）
   - 评分模型：SCOREQ `adapt_nr_telephone.onnx` 与 Distill-MOS 的 sha256 与 GTCRN run 的 README 记的一致（逐个 `shasum -a 256`）；识别服务记模型名与服务端报的权重摘要（拿不到摘要就写「未核实」，由下面的锚点兜底）
   - **锚点（实测可比性，codex 016 最终版方法第 1 轮 P1 #1）**：本次成绩单加 `passthrough`（原声直通，不经模型），逐段与 GTCRN run 里的 `passthrough` 比。过线：DSP 类项与听感分（SCOREQ · Distill-MOS，确定性）逐段差 ≤ 1e-6；吃字项（识别有随机性）只报两边平均差与区间，写进 findings。DSP 或听感分任一段超差 → 两次 run 不可比，停下报大总管。代价：多一个系统，成绩单约多 35 分钟（推断：直通无模型推理，只多打分）
   - 供电：接电源（`meta.power = ac`），`caffeinate -i`
5. **成绩单**（占名额，约 2.3–2.6 小时，推断）：`vdn-eval scorecard --suite src/eval/suite/scenes.json --system passthrough,external:<dpdfnet2 新 run>,external:<dfn3-ll 新 run> --qa --rules v2`。过线：0 段没跑成；两个外部系统截半自检通过（`meta.external[*].trunc_max_abs_diff` ≤ 容差）；`inputs` 含 `clean`，三项带 `clean_pass` 标记
6. **配对分析**：`analyze.py <GTCRN run 目录>,<本次 run 目录> <exp013 clips.jsonl> --out analysis.md --json analysis.json`。`merge` 会拒绝：元数据缺规则版本 · 场景集 · 是否带评分模型，或两边不同；评分项方向或单位不同；任一 run 内或跨 run 的（片段, 系统）重复；同一片段两边的场景 / 声音标签不同（测试 `tests/test_analyze.py::Merge`）。锚点比对用两边的 `passthrough` 行，配对只用 `gtcrn`（GTCRN run）与两个候选（本次 run）
7. 旧的两个候选 run 与草稿成绩单在 README 标「草稿，被 <新 run-id> 取代」，不删不改

## 三、配对对象

GTCRN 一侧用 evalfix 跑的 `20261105-005916-4ccf0638`（规则 v2 · 带评分模型 · 提交 `4ccf0638` · 480 段 0 段没跑成 · 接电源），读数在 `docs/experiments/exp014-scorecard-gtcrn/runs/20261105-005916-4ccf0638/clips.jsonl`。不重跑 GTCRN（大总管定），条件是第二节第 4 步的评分代码核对为空。

## 四、已知的不对等（写进 findings，不在本次修）

| 项 | GTCRN | 候选 | 影响 |
|---|---|---|---|
| 跑的时间 | 一个 run | 另一个 run | 吃字项的识别每段 2 次、有随机性；两边各自独立识别，和同 run 内不同系统一样，不额外引入偏差【推断】 |
| 延迟 | 引擎实测 | 声明值（`latency_declared_only`） | 延迟项不进配对（`analyze.py` 的 `SKIP`），只在代价对照里列 |
| 计时 | 实测 | 判「不知道」 | 同上 |
| 采样率 | 16 kHz 原生 | DFN3 是 48 kHz 模型，吃的是升采样后的窄带 | 对 DFN3 不利【推断】，README「已知局限」已写 |

## 五、失败了怎么办

- 冒烟或截半自检不过 → 不跑成绩单，查脚本；同一类失败 2–3 次就停下报大总管
- 成绩单有段没跑成 → 看报因；是入口 / 输出目录的问题就修完重出新 run，不在旧 run 上补
- 评分代码核对不为空 → 见第二节第 4 步
