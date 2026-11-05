# 现状页 · cands（批次 20261104-b15）

- **状态**：进行中
- **手上**：exp016 候选模型（DFN3 低延迟版 · DPDFNet-2 · NoNoise-Mac）上同一张成绩单 · issue #112
- **交代**：主目录 `_tmp/20261104-b15/cands.md`
- **做到哪**（2026-11-04）：外部入口完成，codex 中途 2 轮 P0/P1 清零，已报「入口可以开 PR」；DPDFNet-2 与 DFN3 低延迟版离线脚本写好（Python 测试 11 条全过），等名额冒烟
- **下一步**：冒烟 → codex 审两个脚本（第②次中途评审）→ 报耗时估计、申请名额跑 480 段

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
1. **NoNoise-Mac 怎么进评测**（它有命令行文件降噪模式，不用驱动、不回录）
   - A. 下载它 v1.5.1 release 的 cli.zip（260 KB）+ app.zip（7.2 MB）到主目录 `_tmp/exp016/`，只在本机命令行跑文件降噪，不装 app、不装驱动 —— 代价：运行未公证的第三方二进制（MIT 开源，可对照源码） ← 建议
   - B. 下载源码用 `swift build` 自己编 —— 代价：多下源码与依赖，编译要一次名额；同样是第三方代码
   - C. 这一批不测 NoNoise-Mac —— 代价：少一个现成 Mac 产品对照（D77 要它当对照组）
   - 如果你不回，默认 C（先测另两个），后果是 NoNoise-Mac 推迟到下一批

## 转交（范围外，留给大总管）
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
