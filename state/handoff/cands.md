# 现状页 · cands（批次 20261104-b15）

- **状态**：进行中
- **手上**：exp016 候选模型（DFN3 低延迟版 · DPDFNet-2 · NoNoise-Mac）上同一张成绩单 · issue #112
- **交代**：主目录 `_tmp/20261104-b15/cands.md`
- **做到哪**（2026-11-05）：两个候选 480 段跑完；草稿成绩单（不带 --qa）出了，配对分析与 findings 草稿写好
- **下一步**：evalfix 套入 clean 补丁、runner 接 provides、PR 合并 → merge main → 补 system.rs 的 native_rate 一支与「少 4 ms」测试 → 申请名额：两个候选**新 run**（mix + clean 一次跑全，各约 21–23 分钟）+ 带 --qa 成绩单（约 1.7–2 小时）

## 改了哪些文件
| 文件 | 改了什么 |
|---|---|
| `src/eval/src/system.rs` | `SystemKind::External` 变体与分发；`build` 多收输入 `x`；外部系统计时给空 |
| `src/eval/src/system/external.rs` | 新：外部输出目录载入 · 自检 · 输入指纹索引 · 回放模型 |
| `src/eval/tests/external_e2e.rs` | 新：8 条端到端测试 |
| `src/eval/src/main.rs` | 成绩单元数据加 `external` 一行（evalfix 同意） |
| `docs/experiments/exp016-candidates/` | `external-entry.md` 入口形状 · `README.md` 候选路线 |
| `docs/reviews/016_candidates-codex.md` · `docs/reviews/README.md` | 评审记录 · 索引一行 |
| `docs/experiments/exp016-candidates/_ext.py` · `run_dpdfnet2.py` · `run_dfn3ll.py` · `tests/test_ext.py` | 离线推理公共模块 · 两个候选脚本 · 测试 |

## 发现
- codex-review 脚本的隐私扫描用 `file` 判纯文本：codex 输出以 `P1｜…` 开头时，`file` 把它认成 PBM 位图（`P1` 是该格式文件头），第 2 轮带 `--previous` 被硬拦。临时做法：给上一轮输出加一行标题。建议修 skill（判文本改看编码，或输出模板首行固定标题）
- NoNoise-Mac 用的 Core ML 模型是普通版还是 `_ll` 版无法从仓库判定（元数据只写 TorchScript 转换）

## 待人拍板
1. **（已按默认 C：这批不测，推迟到下一批）NoNoise-Mac 怎么进评测**（它有命令行文件降噪模式，不用驱动、不回录）
   - A. 下载它 v1.5.1 release 的 cli.zip（260 KB）+ app.zip（7.2 MB）到主目录 `_tmp/exp016/`，只在本机命令行跑文件降噪，不装 app、不装驱动 —— 代价：运行未公证的第三方二进制（MIT 开源，可对照源码） ← 建议
   - B. 下载源码用 `swift build` 自己编 —— 代价：多下源码与依赖，编译要一次名额；同样是第三方代码
   - C. 这一批不测 NoNoise-Mac —— 代价：少一个现成 Mac 产品对照（D77 要它当对照组）
   - 如果你不回，默认 C（先测另两个），后果是 NoNoise-Mac 推迟到下一批
2. **（大总管定：不做，记 backlog）DFN3 低延迟版要不要换产品自己的重采样器复测**（方案 `docs/experiments/exp016-candidates/plan-dfn3-resampler.md`）

## 转交（范围外，留给大总管）
- ~~exp014 `driver/Cargo.lock` 要手动提交~~：已随 8556b95b 一起被 hook 提交（已核对），不用再管
- 大总管定为记账错、转 evalfix 修：V-LAT 的重采样项只在模型原生采样率 ≠ IO 采样率时计入；外部系统声明原生采样率的字段待 evalfix 来定，`external.rs` 由 cands 改（带测试）
- 请大总管把「DFN3 换产品重采样器复测」记进 `state/backlog.md`（不在 cands 可写范围），方案 `docs/experiments/exp016-candidates/plan-dfn3-resampler.md`
- 入口评审第 1 轮留作 issue 的两条 P2：截半自检多切点 / 有声区切点 / `bg` 截半（#3）；同输入重复运行对照，区分数值抖动与偷看未来（#4）。详见 `docs/reviews/016_candidates-codex.md`
- 上面「发现」第 1 条：codex-review skill 的 `file` 误判

## 流水
- 报到；读共同规矩、交代、协议
- 起草入口形状 `external-entry.md` → 发 evalfix，抄送大总管
- 子代理只读查 NoNoise-Mac：有 `--denoise` 命令行模式，输出 48 kHz，模型 4.3 MB 在 app 里（来源：其仓库 v1.5.1 `Sources/CLI/main.swift:21,54-58`、`Sources/Core/AudioFileDenoiser.swift:28,80-89`、`Sources/Core/VoicePreset.swift:37-44`、`Sources/Core/AudioProcessing/DeepFilterNetDSP.swift:7-16,303-339`）
- evalfix 回「可以」+ 3 处改法（通用子目录 · 保留 bg · 元数据带目录与哈希）+ 3 条公平性（延迟只标声明 · 计时给空 · 截半查偷看）→ 全部实现
- 领名额 → `cargo test -p vdn-eval` 全绿、clippy 无警告、tidy 通过（1 处采样率字面量已改）→ 归还
- codex 入口第 1 轮：P1×1 P2×4 → 修 P1 #1、P2 #2 #5，#3 #4 留 issue；测试 8 条全绿
- codex 第 2 轮首次被隐私扫描误拦（见发现）→ 加标题后重跑
- codex 入口第 2 轮：P0/P1 0 条，P2×1（故障路径没测）→ 补测试 + 变异检查；中途评审停；报「入口可以开 PR」
- 写 `_ext.py`（三种输入同 Rust `Kind::input`、实测前视、对齐检查、finalize 查全）、`run_dpdfnet2.py`（导入 exp003 的 Dpdfnet/stream，声明 960 样本）、`run_dfn3ll.py`（Hush dylib + soxr 流式，声明 284 样本 = 160 + 重采样前视 124）
- 发现：soxr 流式重采样按样本序号对齐、自扣延迟，等于偷看约 124 样本（7.75 ms）未来 → 计入声明延迟
- Python 测试 11 条（含与 Rust 正本 SCHEMA / Kind 名比对、与 scenes.json 片段清单比对）全过
- 申请冒烟名额（两个 --probe，≤ 2 分钟）
- 大总管改名 84 → 26（已核对会话列表：84 不在、26 新起）；冒烟名额申请重发给 26
- 大总管定：草稿只跑 DSP 项（三个系统一次跑，吃字 / 听感写「尚无判据，待最终版」）；最终版不重跑 GTCRN——evalfix 新规则合并后它带 --qa 跑 passthrough · ref · gtcrn，cands 只带 --qa 跑 dpdfnet2 · dfn3-ll（约 1.7 h），同 build、同规则、同评分模型配对，README 记两个 run-id
- NoNoise-Mac：人没选，按默认 C 这批不测，findings 写「推迟到下一批，待人定」
- 入口 PR #115 已合并（Refs #112）；P2 开成 #113 #114
- 领名额跑两个候选 480 段：DPDFNet-2 probe 通过（滞后 0，A→B→A 差 0.0），正式跑中
- DPDFNet-2 run 20261104-235743-73c68379：971 文件 14 分 07 秒；finalize 通过（声明 960）
- DFN3 低延迟版 run 20261105-001245-201407de：probe 通过（A→B→A 差 0.0）；971 文件 15 分 32 秒；finalize 通过（声明 284，调度延迟 1393 只记录）
- 草稿成绩单 20261105-002946-e21f3a93（3 系统 × 480 段，不带 --qa，0 段没跑成，7 分 33 秒）→ 存 runs/；归还名额
- 编译 exp014 驱动时 cargo 改了它的 Cargo.lock → 先盖回原文件；大总管放开后再让 cargo 刷新留着
- 写 analyze.py（配对差值 + 成对片段自助重抽样区间 + 三态 + 前 5 处）与 6 条测试；修了排名口径（全局标准差、≥ 5 个配对单位）与同名模块导入
- 写 findings 草稿；自核时改掉一处说过头（DFN3 不是 15 个场景都压得更多）并换成评分项正式中文名
- 大总管确认 Cargo.lock 已随 8556b95b 提交（已核对）
- 写方案 plan-dfn3-resampler.md：换 vdn-dsp 3 倍重采样（产品正本），预计前视 192、V-LAT 约 39.7 ms；发现 V-LAT 对 48 kHz 原生模型重复记了重采样预算
- 大总管定：延迟账多记重采样 = 记账错，转 evalfix；换重采样器这批不做、记 backlog → findings 加局限、方案页首标状态、现状页改第 2 条
- 大总管：backlog 已记（21acbb21）；native_rate 的默认值与允许取值从 vdn-types 采样率正本取（`MODEL_RATE` · `IO_RATE`），不在 external.rs 手写；最终字段以 evalfix 定的为准
- evalfix 定 native_rate 字段（可选，只许 MODEL_RATE / IO_RATE，默认 MODEL_RATE）→ external.rs 实现 + 单测 + 端到端测试（8531a3bc），Python 收尾写 native_rate、测试比对 rate.rs；system.rs 那一支与「少 4 ms」测试等 evalfix PR 合并后补
- 问大总管：DFN3 旧 run.json 补 native_rate=48000（不重跑音频，builds/ 加 amend.json 记旧新哈希）；不回默认照做
- evalfix：native_rate 做法确认；最终 GTCRN run 20261105-005916-4ccf0638（规则 v2，约 03:30 完成）→ 记进 README runs 表
- 大总管同意补字段：先确认进仓 run.json 已提交，再改；amend.json 记旧新 sha256 · 字段 · 原因 · 日期；最终版元数据记新哈希、README 说明
- 大总管：合 main 后 v2 把干净人声单独过一遍，外部入口查不到 → external_e2e 主测试 Missing。与 evalfix 对齐：Kind 加 Clean（clean/ 可选，列了须 480 段齐）+ `External::provides`；没列时 runner 不跑这一路、三项只报不判（evalfix 定；大总管上一条写「报没跑成」，已请大总管知悉）。e2e 夹具写四种、加 clean 负例两条；_ext.py 输出 clean（owner 轨）；Rust 10+5、Python 25 条全过，clippy · tidy 过。补丁放主目录 _tmp/evalfix/cands-clean.patch（出自 fe09f692）
- 大总管定：另开新 run，mix + clean 一次跑全；旧 run 不补 native_rate、不写 amend.json（撤回前一条）；旧 run 在 README 标「草稿，被 <新 run-id> 取代」
- evalfix 套入 clean 补丁（bdf0ad57，出自 fe09f692），v2 下 vdn-eval 全绿；审 evalfix 的「没有 clean/ 判只报」测试：过，P2 一条（循环可能空过）已改 987a3af8；Kind::Clean 注释改为「没列时 runner 凭 provides 不跑这一路」
- 大总管：开跑前方法审 codex 1 轮。写 plan-final-run.md；codex ③′ 2 轮（P1×4 全修、P2×3 修 2 留 1）→ 停。改动：probe 对 mix 与 clean 都量前视；analyze 加锚点（本次成绩单加 passthrough，与 GTCRN run 逐段比 ≤1e-6）· 只挑基线与候选合并 · 片段覆盖核对；吞字头跨 run 结论在识别权重未核实时降为参考。Python 33 条过；草稿分析重出逐字相同。成绩单估计改为 2.3–2.6 h（多 passthrough），总计约 3–3.3 h
- 吞字头跨 run：直通锚点恒为 0 测不出 → 改用两次 run 的识别缓存同键逐条比（asr_drift.py + 4 条测试），阈值每段 ≤ 1 字照常比；大总管同意（2026-11-05）。识别服务无权重摘要，记 id + size_bytes
- 等 #116 合 main，大总管通知后申请名额
