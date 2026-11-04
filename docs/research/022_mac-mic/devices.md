# 设备键表（022 · Mac 内置麦克风）

> 状态：**v2（2026-11-03）**，调研已汇总。v1 → v2 只补了数字和出处，键名一个没改。
> 用途：scenes、mtgdata 给数据和场景打「设备」标签时，一律用下表第一列的键。缺键发消息给 macmic 加，不自造。
> ⚠️ 机器读取：mtgdata 的 `check_cards.py` 直接读本表第一列，每行须以 `` | `键` | `` 开头。改表格式、改键名前先通知 mtgdata。本目录 `tools/check_devices.py` 检查键名格式和重复（带测试）。
> 来源标记：【读系统文件】本机 macOS 自带文件 ·【官方文档】·【源码】·【评测网站】（DXOMARK 等，每款 1 台）·【论坛 / 博客，个人说法】·【推断】。证据细节见 `findings/`，出处总表见 `sources.md`。

## 一、键表

每行一种「从说话人嘴到降噪模型输入」之间的设备链。「第三方拿到的」指我们用 cpal 走普通采集（HAL）时拿到的信号。

| 键 | 是什么 | 第三方拿到的 | 系统已做的处理（标准模式） | 频响 / 底噪 / 自动增益 |
|---|---|---|---|---|
| `mba-builtin` | Apple 芯片 MacBook Air 内置麦（M1–M5，13 / 15 寸） | 1 声道 48 kHz【实测，011：本机 M5 Air】；硬件进系统是 3 路【读系统文件：ioreg「Digital Mic」3 声道】 | 候选链（本机用哪份调音未查到）：直流隔离 → 每麦底噪谱降噪 → 波束成形（3 合 1）→ 均衡 → 输入音量 ±12 dB【读系统文件，`findings/tunings.md`】。图里没有回声消除单元；推断的 Air 组没有风扇 / 环境降噪单元；波束与降噪在运行时是否自适应未知【推断】 | 频响（M2 Air 13″，1 台，DXOMARK；录音用的 app 与麦克风模式页面没写，不等于 HAL 标准模式）：100 Hz 约 −10 dB，150–300 Hz 在 −2 到 −3 dB 间起伏；4–11 kHz 起伏抬升 +3 到 +9 dB，8 kHz 有凹口；高端 −3 dB 点约 12.6 kHz，16 kHz 约 −13 dB【评测网站】。指向性（同一台）：90° 处低 / 中 / 高频段 −3.4 / −13.6 / −31 dB【评测网站】。底噪、灵敏度：未查到。HAL 路径上的自动增益：未查到开关，推断没有 |
| `mbp-builtin` | Apple 芯片 MacBook Pro 内置麦（13 / 14 / 16 寸） | 推断同上 1 声道 | 同上，部分机型标准模式还多一级风扇噪声与环境噪声抑制（`ocns`）【读系统文件；哪个编号是 Pro 为推断】。DXOMARK 却写「Noise reduction is disabled by default」（MBP 14″ M2 Pro）【评测网站】，两者矛盾待实测 | 较平：100 Hz −2 到 −4 dB；高端 −3 dB 点约 14.5 kHz【评测网站，2 台】。官方写「studio-quality」「high signal-to-noise ratio」，Air 从没写过【官方文档】 |
| `mac-intel-builtin` | Intel 时代 Mac 笔记本内置麦（2020 年前）。分两段：2013–2017 楔形 Air 双麦；2018–2020 Retina Air 三麦 | 未查到 | T2 之前的机型有设备级「环境降噪」开关，Apple 芯片与 T2 机型上没有【官方文档】。T2 机型由 T2 集成音频控制器【论坛 / 博客】 | 未查到。DR-VCTK 有 2013–2017 双麦 Air 的整条链录音（含 2 m 回放与房间），可反推相对频响到 8 kHz【推断】 |
| `imac-builtin` | iMac 24 寸（M 系列）内置麦 | 未查到 | 官方写三麦阵列 + 定向波束成形 + studio-quality【官方文档】；处理链推断同 `mbp-builtin` | 未查到 |
| `studio-display` | Apple Studio Display 三麦阵列（USB 接入） | 未查到 | 官方写法同 iMac【官方文档】。Firefox 在它输入输出配对时禁用苹果语音处理【源码】 | 未查到 |
| `airpods-hfp` | AirPods 作麦克风（蓝牙通话链路） | 1 声道窄带；采样率官方未公开，说法有 16 kHz（mSBC）与 24 kHz（AAC-ELD）两种【论坛 / 博客，各 1 人】 | 耳机端降噪与波束成形【推断】；编解码压缩。开麦后蓝牙音质降低【官方文档】。2025 年「studio-quality 录音」在 Mac 上走什么链路：未查到 | 未查到 |
| `bt-hfp-generic` | 非 Apple 蓝牙耳机作麦（HFP：CVSD 8 kHz 或 mSBC 16 kHz） | 1 声道窄带 | 耳机端处理因品牌而异 | exp006 现用「蓝牙窄带」近似（假设） |
| `usb-headset` | USB 或 3.5 mm 有线耳麦（嘴边近讲） | 1–2 声道 | 多数不处理，少数带硬件降噪【推断】 | 因品牌而异 |
| `external-mic` | 桌面外接麦（USB 电容 / 动圈，如 Shure MV5） | 1–2 声道 | 一般不处理 | 因型号而异 |
| `iphone-continuity` | iPhone 当 Mac 的麦（连续互通） | 未查到 | 未查到（可能带 iPhone 端处理）。帮助页场景写的是「没有内置或外接麦的 Mac」【官方文档】 | 未查到 |
| `mac-unknown` | Mac 笔记本，型号未知、是否内置麦未核（如 DNS 2021 众包者自填 macbook / mac 的约 31 条） | — | — | — |
| `laptop-unknown` | 非 Mac 或品牌未写的笔记本内置麦（3D-Speaker、DNS 文件名里的 laptop、Chromebook、Surface Book、MODALITY） | — | — | — |
| `non-laptop` | 父类：不是笔记本。打标签时尽量用下面的子键 | — | — | — |
| `non-laptop-phone` | 手机 | — | — | — |
| `non-laptop-tablet` | 平板 | — | — | — |
| `non-laptop-desktop` | 台式机 / 摄像头自带麦 | — | — | — |
| `non-laptop-conf-device` | 会议终端（会议室一体机、全向会议麦） | — | — | — |
| `non-laptop-mic-array` | 研究用麦克风阵列 | — | — | — |
| `non-laptop-close-talk` | 近讲 / 领夹 / 头戴参考麦 | — | — | — |
| `non-laptop-studio` | 录音棚麦 | — | — | — |
| `non-laptop-unknown` | 设备未写 | — | — | — |

> v1（2026-11-03）：应 mtgdata 加 `mac-unknown` 与 `non-laptop` 的 8 个子键。子键统一写成 `non-laptop-<子类>`。
> v2（2026-11-03）：补数字与出处；`mac-intel-builtin` 注明分双麦、三麦两段（不拆键）。

## 二、系统处理模式字段 `mic_mode`

设备键不含「语音突显开没开」。这件事另记一个字段，只对 Mac 设备有意义。

| 值 | 含义 | 第三方 HAL 采集会不会经过 |
|---|---|---|
| `standard` | 系统标准模式（第一节那条候选链） | 推断会：HAL 普通采集很可能走这条【读系统文件 + 推断，本机运行时选图未确认】 |
| `voice-isolation` | 语音突显 | 已证实开了苹果语音处理（VPIO）的 app 能用；HAL 采集会不会受影响未核实【官方文档 + 推断】。Zoom Mac 客户端会触发它【论坛 / 博客，引 Zoom 界面原文】 |
| `wide-spectrum` | 宽频谱（「处理减到最少」，不是不处理） | 同上【官方文档】 |
| `unknown` | 查不到 | 公开数据集几乎都是这个，数据卡写「系统处理模式未查到」 |

## 三、已用标签（供 mtgdata 核对）

- MOBIO（2008 款 MacBook）→ `mac-intel-builtin`
- **DR-VCTK → `mac-intel-builtin`（2013–2017 楔形双麦 Air）**。v1 草稿曾建议 `mba-builtin`（年份未知），那是错的：论文原文写「MacBookAir's two microhpones」，Apple 芯片 Air 是三麦【论文 + 官方文档】
- DNS 2021 Mac 众包 31 条 → `mac-unknown`
- 3D-Speaker、MODALITY → `laptop-unknown`

## 四、和现在评测集的对应

- exp006 `recipes.json` 的 `device` 是场景分类器的三类配比（none 0.6 / 蓝牙 0.2 / 削波 0.2），不是在模拟真实削波频率【读源码，sim 怀疑派 #要点7b】。`none` ≈ `external-mic`；`bt_narrow` ≈ `bt-hfp-generic`
- 016 F8 的 `laptop`（100 Hz 高通 + 7 kHz 低通）对应 `mba-builtin`，但 7 kHz 低通与实测频响（到约 12.6 kHz）不符。替换方案见 `simulation.md`
