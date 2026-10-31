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

### 已建的 crate（依赖单向 dsp ← engine ← models ← cli）

| crate | 是什么 | 谁产生 | 谁是谁的输入 |
|---|---|---|---|
| `vdn-dsp`（`src/dsp/`） | 纯信号处理积木：STFT/iSTFT、延迟线、快开慢关包络、窗 | AI 写（p1，2026-10-30） | 被 engine、models 用；不依赖任何本项目 crate |
| `vdn-engine`（`src/engine/`） | 模型接口 `trait Model`、逐帧调度 + 三件保命、故障注入、离线驱动 | AI 写（p1） | 输入：dsp 积木 + 任一 `Model`；被 models、cli 用。不链接 ONNX Runtime |
| `vdn-models`（`src/models/`） | 模型适配器 GTCRN、FastEnhancer-M（ort 单线程 CPU）、`AnyModel` | AI 写（p1） | 输入：模型权重（仓外）；实现 engine 的 `Model`；被 cli 用 |
| `vdn-cli`（`src/cli/`，可执行 `vdn`） | 命令行 `denoise` / `bench`，WAV 读写 | AI 写（p1） | 输入：WAV + 模型权重；产出降噪 WAV 与计时 JSON（评测报告的原料） |

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
