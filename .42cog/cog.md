# Mac 开源实时语音降噪 — 认知模型

<meta>
  <document-id>vdn-cog</document-id>
  <version>1.0.0</version>
  <project>Mac 开源实时语音降噪</project>
  <type>Cognitive Model</type>
  <created>2026-10-29</created>
  <depends>intent.md, real.md</depends>
</meta>

## Document Purpose

这个系统里**有哪些东西**，谁产生它们，它们之间什么关系。

**列不出实体，后面的流程都是空转。** 先列全，再管关系。

---

<cog>

## 核心实体

| 实体 | 是什么 | 存在哪 | 谁产生 |
|---|---|---|---|
| **crate**（工作单元） | Rust workspace 里的一个包，一个职责 | `src/<crate>/` | AI 写，人审 PR |
| **评测集** | 合成场景矩阵 + 固定真实录音 | 仓外（大文件），仓里只留清单与哈希 | 脚本合成；真实录音由人录 |
| **模型权重** | 第三方开源模型的 ONNX 等文件 | 仓外，仓里只留来源、许可证、哈希 | 上游项目；以后可能有自己的微调 |
| **评测报告** | 一次改动在评测集上的三把尺子数值 | `docs/experiments/` 或 CI 产物 | `eval` crate |
| **标准样本 / 负例** | 「好的会议声音」与典型失败的样本 | 跟着作品住（`-proto` 定） | 人挑、人改 |
| **真相源** | 设计文档等原始材料 | `vault/raw/` | 人 |

### 已建的 crate（依赖方向见 `specs/rust/architecture.md` §2.2：types ← dsp ← engine；models → types + dsp；cli → engine + models）

| crate | 是什么 | 谁产生 | 谁是谁的输入 |
|---|---|---|---|
| `vdn-types`（`src/types/`） | 共享词汇，零依赖：`trait Model` / `Stage`、帧上下文与声纹判决、智能体部件接口（感知输出 · 参数组 · 场景参数表 · 策略 · 我的配置 · 学习接口 · 决策日志 · 延迟账 · 版本号） | AI 写（align，2026-10-31） | 被 engine、models 以及以后的 speaker、scene、polish 用；不依赖任何 crate |
| `vdn-dsp`（`src/dsp/`） | 纯信号处理积木：STFT/iSTFT、延迟线、快开慢关包络、窗 | AI 写（p1，2026-10-30） | 被 engine、models 用；不依赖任何本项目 crate |
| `vdn-engine`（`src/engine/`） | 逐帧调度 + 三件保命、故障注入、离线驱动、延迟账汇总（`latency_ledger`） | AI 写（p1；align 收尾） | 输入：dsp 积木 + 任一 `Model`；被 cli 用。不认识具体模型、不链接 ONNX Runtime |
| `vdn-models`（`src/models/`） | 模型适配器 GTCRN、FastEnhancer-M（ort 单线程 CPU）、`AnyModel`、各模型声明的 hop 与延迟 | AI 写（p1；align 收尾） | 输入：模型权重（仓外）；实现 types 的 `Model`；被 cli 用 |
| `vdn-cli`（`src/cli/`，可执行 `vdn`） | 命令行 `denoise` / `bench`，WAV 读写 | AI 写（p1） | 输入：WAV + 模型权重；产出降噪 WAV 与计时 JSON（评测报告的原料） |
| `vdn-speaker`（`src/speaker/`，tier B） | 认出你的声音：VAD 快开慢关、不对称门控（实时级，0 延迟）+ fbank、声纹、零设置注册、判断回路（约 0.5 s 一次） | AI 写（p2，2026-10-30） | 输入：降噪输出 + 原声 + 声纹模型（仓外）；产出 `SpeakerVerdict` 与门控后的声音；默认只依赖 dsp + earshot；声纹推理经 `onnx` feature 用 ort（过渡期例外，同 vdn-models）；离线命令行已挪到 `vdn-speaker-lab`；接口镜像 `vdn-types` |
| `vdn-speaker-lab`（`src/speaker-lab/`，可执行 `vdn-speaker-lab`，tier tool） | P2 离线实验命令行（exp005）：读写 WAV、跑 GTCRN、调 vdn-speaker 的实时级与判断回路 | AI 写（p2fix，2026-10-31，从 vdn-speaker 的 `offline` feature 挪出） | 依赖 vdn-speaker（onnx）+ engine + models + ort + hound + serde_json；不进实时线程、不进产品 bin；`vdn-eval` 建起来后并进去 |
| `vdn-eval`（`src/eval/`，可执行 `vdn-eval`） | 评测：verifier（CI 层纯 DSP 16 条 · QA 层 ASR / DNSMOS）、任务集（回归 · 能力 · 防作弊探针，`src/eval/suite/`）、定标（人耳答卷 → L0–L3，`suite/levels.json`）、报告（JSON + Markdown + 抽查样本） | AI 写（evalimpl，2026-10-31） | 输入：engine + models（同一份处理代码）、合成场景或仓外真实片段、人耳答卷；产出评测报告与级别表；`cargo run -p xtask -- eval ci` 挂在 CI |

| `xtask`（`src/xtask/`） | 项目规矩的机器检查：`cargo run -p xtask -- tidy` / `-- check-deps`，也随 `cargo test -p xtask` 跑 | AI 写（align，2026-10-31） | 输入：`cargo metadata` 与源码文本；不依赖任何 vdn crate |

### 智能体部件的数据（D35 · D40 · D41；类型在 `vdn-types`，规约 §3.7）

| 实体 | 是什么 | 存在哪 | 谁产生 | 谁是谁的输入 |
|---|---|---|---|---|
| **场景标签** | 两层：用户层基本场景 + 内部细场景、置信度、时间戳 | 内存（判断回路每 0.5 s 一个） | 感知（scene 线的分类器） | 策略的输入 |
| **场景参数表** | 每个场景一组参数；版本号 + 内容 ID；行文本 | 随软件发布（文本文件） | 开发方在模拟环境里调出、过门禁 | 策略查表 |
| **我的配置** | 用户按场景拧过的旋钮；只存参数、不存音频 | 本机，不上传（D39） | 用户操作 + 学习部件（空闲时提议、过门禁） | 策略叠加在表上 |
| **决策日志 / 会话记录** | 每个决定与人的每次操作；不含音频 | 本机，不上传 | 策略、engine、用户操作 | 学习部件、事后追溯 |
| **延迟账** | 每个部件声明的延迟及合计（≤ 45 ms） | 代码（测试里断言） | 各部件声明，engine 汇总 | 改模型 / 改链路时的门禁 |
| `vdn-polish`（`src/polish/`） | 修音链：频谱平滑（金属声）· 下限 + 舒适噪声（真空感）· 去发闷 · AGC + 限幅（忽大忽小）；全部零前瞻，预设编号 = `ToneId` | AI 写（scene，2026-10-31） | 输入：模型 STFT 上的带噪 / 降噪频谱 + 降噪后时域 hop；依赖 dsp；接入 engine 待下一批 |
| `vdn-scene`（`src/scene/`） | 场景识别与路由：35 维便宜特征 → 7 个噪音大类 × 设备 = 细场景 → 用户层场景；滞回；细场景 → 参数组（模型 + 修音链预设） | AI 写（scene，2026-10-31）；分类器权重由 exp006 模拟数据训练生成 | 输入：麦克风原声（判断回路约 0.5 s）；产出 `SceneLabel` 与 `ParamSet`；依赖 dsp、polish；接口暂为 vdn-types 镜像 |

**建一个，就在这张表里加一行**（是什么 · 谁产生 · 谁是谁的输入）。初拟划分见 `vault/raw/20261029-voice-denoise-design.md` §6（dsp · engine · speaker · io-macos · hal-plugin · control · eval），以 `-research` 结论为准。

## 参与者

| | 负责 | 不负责 |
|---|---|---|
| **人** | 方向 · 拍板 · 定标 · 用耳朵听 · 录真实录音 · 真会议试用 · 合并 PR | 不做能被脚本判对错的检查 |
| **AI** | 调研、写代码、跑评测、挑出要人听的片段、整理待拍板题面 | 不定方向、不定标准、不替人听 |

## 关键关系

- **评测集 + 评测报告是所有改动的输入**：没有报告的改动不知道好坏
- **`eval` 和实时运行用同一份处理代码**：`eval` 只是把 WAV 逐帧喂给同一个 `engine`
- **人挑中、改过的样本定标准**：之后的评测阈值从它推出来
- **模型换了，评测基线就要重跑**

</cog>
