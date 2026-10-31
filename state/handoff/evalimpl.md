# 现状页 · evalimpl（批次 20261031-b4）

分支 `feat/eval-verifiers` · worktree `noise_canceling-evalimpl` · 大总管 `noise-canceling-69`

## 记录
- 2026-10-31 报到；读完 Anthropic《Demystifying evals for AI agents》、012 verifiers/decision、rubric 第 2/7 节、exp003 答卷与 scores.json、exp005 人耳反馈
- 发现：exp003 的候选音频已不在盘上（只剩 scores.json 与答卷）→ 定标先用 scores.json 现有指标；exp005 三轨素材在主目录 `_tmp/20261031-exp005/src/`（含 owner/interf/noise/mix + 时间线 + 原文）→ 作 QA 层真实任务
- 设计：xtask 不许依赖 vdn crate → `xtask eval ci` 转调 `cargo run -p vdn-eval -- ci`；CI 无模型权重 → CI 层被测系统 = 真 Engine（三件保命）+ crate 内参考降噪器（维纳，用 vdn-dsp STFT）；本机有 GTCRN 权重时自动加跑
- 开写 src/eval/
- src/eval/ 写完（lib：synth · analysis · verifiers · system · suite · probes · runner · report · calib · qa/{asr,dnsmos} · wav；bin vdn-eval ci|qa|calib|list）；`cargo clippy -p vdn-eval` 干净
- 任务集：`src/eval/suite/ci.json`（16 回归 + 9 能力 + 5 探针）、`qa.json`（11 个真实片段：exp005 c01–c08 + exp003 c01–c03）、`levels.json`（级别正本）
- xtask 加 `eval ci|qa`（子进程转调，不依赖 vdn crate）；ci.yml 加 `eval ci` 一步；规约 §7.1 + G12；cog.md 加 vdn-eval 一行
- 申请名额（cargo test -p vdn-eval + eval ci + calib），排队第一
- 待办：main 已合 PR #34（Cargo.toml members、cog.md 有改动）→ 收尾前同步 main（大总管要求）
- 名额跑完：cargo test -p vdn-eval 21 过（修了 Kendall τ 的 signum(0)=1 bug）；eval ci --system ref：回归 pass^k 12/16、能力 0/10、探针 5/5 全抓住，门禁通过；冻结基线 87 项写入 suite/baseline-ci.json；报告记录 results/20261030-235322-f3e9757-ci.*
- 首跑发现：① R01/R03 字头门控漏开（停顿 < −70 dBFS 绝对门限，底噪跟踪只看到语音帧）→ 挪进能力集 C10；② 纯噪声停顿里门控 ~50% 时间开着（noise_open），停顿只压约 5 dB——可能是 exp005「都没降噪」的引擎层原因（ref 上测得，GTCRN 待验）
- 定标第 1 轮（exp003）：48 对 12 屏，没有一条 ≥ L1（最好 Δ(del+sub) 0.64 [0.41, 0.84]；DNSMOS OVRL 0.48）；按 012 第 1 轮是定标轮 → 正式全 L0；calib/20261031-exp003-round1.*
- 2026-10-31 大总管合了 main（8d236e9），给名额跑 gtcrn + qa
- GTCRN CI 层（release，VDN_GTCRN_ONNX 指主目录权重）：回归 pass^k 11/16，门禁通过；**门控停顿误开在真模型上成立且更严重**：R02 noise_open 0.99；R04 停顿只压 5.2 dB。复算根因：GTCRN 停顿输出 p10/p50/p90 = −66.5/−53.2/−39.7 dB，底噪跟踪钉在 ≈ −64 → 46% 停顿帧判有声 + 150 ms 拖尾 → 82% 开。代码：src/dsp/src/envelope.rs:91-97 + src/engine/src/engine.rs:355
- 新 verifier V-SIM-QUIET（L3 模拟）+ 回归任务 R17（xfail，留给修复线）；xfail 机制：阻断降为报警、全过时提示转正
- 自审（红军 · 实现派）已回，修了：ort 双 Environment（DNSMOS 改子进程）· V-VACUUM 不进基线 · 报告路径改相对 · 缺基线项报警 · R01/R03 放宽护栏 · ASR key 读环境变量、curl -f · 规约 :58 口径
- 跑 QA 层中（后台）；怀疑派自审未回
