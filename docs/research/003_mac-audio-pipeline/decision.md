# 取舍卡 · Mac 上的实时音频管线与虚拟麦克风怎么接

> 2026-10-29，手下 pipeline 起草，交付前经三视角自审修订。**一页纸。** 依据见 `gap.md`（逐项核对与候选横向表）和 `sources.md`（来源登记）。
> ⚠️ 最后一栏「需要你拍板的地方」**AI 不代填**：选项和建议是草稿，「拍了吗」一列留给你。正文里凡是依赖拍板结果的地方，都标了「⟨待拍板 N⟩」。
> 证据等级：【官】官方明载 ·【源】源码可证 ·【测】本机只读实测 ·【传】传言 ·【推】推断。

## 这次不可妥协的几条

1. **崩溃隔离**：我们的代码出错，不能让整台电脑没声音。
2. **延迟可量、稳定，能热重建**：附加延迟约 20 ms 且不抖；蓝牙切 16 kHz、热插拔、时钟漂移都不用重启程序。
3. **普通用户装得上**：不关 SIP；签名、公证、密码、重启次数都写清楚，且可以接受。

**建议增加（⟨待拍板 4、8⟩，未定）**：第 1 条加上「出事时对方听到原声，不是静音」。理由：所有先例断供时都送静音，而静音正是吃字（`gap.md` 第〇节）。

## 我选择什么

> **开发期（M0–M2）：Rust 用 cpal 开两条流，从真麦克风读、写进 BlackHole，中间用 rtrb 做缓冲，rubato 按缓冲水位做漂移补偿。最终产品：自写一个只做「隐藏输出 → 可见麦克风」回环的极简 C 插件，结构照 Apple 示例代码（NullAudio / SimpleAudio）；Rust 处理进程把它当普通输出设备写；推理永不进插件。**

「照 Apple 示例」的依据：libASPL 和 Background Music 都自述基于这两份示例（`resources/libASPL/README.md:643`；`resources/BackgroundMusic/BGMDriver/BGMDriver/BGM_Device.cpp:26-27,1381`）。示例原件本次没下载，现已下架，只能从这两处转述【源】。Apple 示例代码许可允许使用、修改、再分发，条件是保留声明（原文见 `resources/BackgroundMusic/LICENSE-Apple-Sample-Code`）【源】。

## 我放弃什么

> **放弃让插件和处理进程用共享内存直接传音频（留作二期，视实测），放弃 DriverKit，第一版插件不用 Rust 写（libASPL 只在首选卡住时启用）。** 是否把 BlackHole 阶段交给外部用户，见 ⟨待拍板 6⟩。

## 分阶段方案

| 阶段 | 虚拟设备 | 处理进程 | 出事兜底 | 安装 |
|---|---|---|---|---|
| **M0 直通** | BlackHole 2ch（开发者自己 `brew install blackhole-2ch` 后重启，⟨待拍板 1⟩） | ① 输入流按设备 ID 绑定真麦克风，**选设备时排除 BlackHole 和我们自己的设备**，否则会自己录自己形成回环；② 输出流写 BlackHole：第一次打开设 48 kHz，之后重建都**跟随设备当前采样率**，避免和会议软件互相改（cpal 建流会改设备的全局采样率，输入侧 `resources/cpal/src/host/coreaudio/macos/device.rs:560-571`，输出侧 `:716-727`）；③ 输出回调里 rubato 做漂移补偿：两端同为 48 kHz 用 `Slip`（近乎零开销零延迟），不同采样率用 `Async` sinc（poly 不做抗混叠，`resources/rubato/README.md:76-113`），用 PI 控制器看 rtrb 水位调比率；④ 重建入口：`StreamInvalidated` 和 `DeviceNotAvailable` 汇到同一个防抖入口；**默认输入设备变化 cpal 不报**（`DeviceChanged` 只对按默认设备建的流触发，`resources/cpal/src/error.rs:14-22`），要自己用 objc2-core-audio 监听 `kAudioHardwarePropertyDefaultInputDevice` 和设备列表；rtrb 不能重置，重建时新建一对读写端交给常驻线程；⑤ 启动先预填半个缓冲，淡入 5 ms；⑥ **端到端延迟实测**：再开第三条输入流读 BlackHole，模拟会议软件，注入脉冲做相关。两端回调时间戳量不到会议软件读 BlackHole 那一跳（`resources/cpal/src/timestamp.rs:45-52`） | 不需要（直通本身就是原声） | 只在作者本机。cpal 要用 git 依赖钉住某个提交：方案用到的 `CallbackInfo` / `xrun()` 都还在 [Unreleased]，最新正式版是 0.18.2（`resources/cpal/CHANGELOG.md:8-29,87`） |
| **M1–M2 接模型** | 同上 | 推理线程：入口先把采样率固定重采样成模型要的；按 10 ms 帧读环 A、写环 B；用 `audio_thread_priority` 提优先级（计算时间是周期的一半，约 5 ms，`resources/audio_thread_priority/src/rt_mach.rs:115-135`），再自建 work interval（约 60 行 FFI）。推理库必须单线程，否则每个工作线程都要加入 interval【推】，预算交 002 定。环 B 按 10 ms 台阶进出，PI 控制器要加低通或死区 | **原声旁路由输出回调做，不由推理线程做**：输入回调同时写一条原声环 C（先补上与模型相同的延迟）；输出回调发现环 B 欠载，就改读环 C，并在两路之间**交叉淡化**。挡得住模型卡顿；挡不住整个进程崩溃，此时 BlackHole 送静音（`resources/BlackHole/BlackHole/BlackHole.c:4561-4571`） | 同上，可加少量测试者 |
| **M3+ 自有设备** | 自写 C 插件：一个隐藏输出端、一个可见输入端，插件内部无锁回环，只做时间戳、静音门、运行中改采样率；IO 回调标 `CA_REALTIME_API`（macOS 27 SDK 编译期检查不阻塞，`AudioServerPlugIn.h:1014`）。CPU 架构与最低系统版本 ⟨待拍板 9⟩ | 输出设备从 BlackHole 换成我们的隐藏端；采样率归属改成由我们独占 | 进程整个崩溃时送什么 ⟨待拍板 4⟩。还要定：处理程序没启动、被用户退出、开机未登录时设备送什么；开机自启与「后台项目」授权【推】 | pkg：输管理员密码、生效方式 ⟨待拍板 7⟩、授权麦克风。签名 ⟨待拍板 2、3⟩。**卸载**：删插件、删 LaunchAgent、再生效一次，要能写成一页 |
| **二期（视实测）** | 插件改读共享内存 | — | — | 只在 M0 第⑥步实测的那一跳超出 20 ms 预算时才做 |

**迁移时留下什么**：单进程内的 Rust 代码全部留下：采集、重采样、水位控制、重建入口、推理线程、环 C 旁路。从 M0 起按设备 UID 选输出设备，不写死 BlackHole。**换掉的**：采样率策略（从「跟随会议软件」改成「我们独占」）；新写 C 插件和安装包。**如果 ⟨待拍板 4⟩ 选两层进程**：环 B、环 C、水位控制要改成跨进程共享内存重写。rtrb 放不进共享内存（`resources/rtrb/src/lib.rs:114,159`），这部分不能直接留下。

## 拿三把尺子量一遍（`.42cog/intent.md`）

| 尺子 | 本方案让它变好的 | 风险 / 不能让它变差的 |
|---|---|---|
| **清楚**（不吃字） | 推理卡顿时送原声，不送静音（M1 起） | 进程整个崩溃时仍是静音，直到 ⟨待拍板 4⟩ 落地；旁路切换如果没对齐延迟，会跳字或重字，所以先补延迟再切 |
| **自然**（不忽大忽小） | 漂移靠重采样平滑吸收，不靠丢帧（对比 MetalVoice 的丢帧兜底，`resources/MetalVoice/Sources/Core/AudioModel.swift:196-218`） | 旁路一切，背景噪音会突然冒出来，就是「忽大忽小」。靠交叉淡化缓解，淡化时长要在评测里调（交 004） |
| **轻快**（延迟低且稳） | 推理隔在插件外，插件 IO 永不等模型；端到端延迟在 M0 实测 | 回环比共享内存多一个 IO 周期（推算 1.5–3 ms【推】）；20 ms 够不够取决于模型前瞻（交 002） |

## 依据来自哪一类

- [x] 自有积累：设计稿 `vault/raw/20261029-voice-denoise-design.md` §6 / §7 / §11 / §16。【你定的】目标都算数；系统层预判里 P1 只部分成立，P4 不成立（`gap.md` 第〇节）
- [x] 他人积累：12 个克隆仓库逐条读到文件行号；Apple SDK 头文件、官方文档、Developer Forums；补检索看了 5 个网页先例（eqMac 等，没克隆）
- [x] 本机只读实测（macOS 27.0.1）：开机日志显示插件在独立宿主进程里加载；宿主进程的签名检查字符串与授权；本机没装 BlackHole、没有 meson
- [ ] 自己跑的实验：**还没跑**。插件加载与崩溃实验 ⟨待拍板 3⟩

## 复用分级与停止线

| 来源 | 权限 | 这次实际用到哪一步 |
|---|---|---|
| **自家的** | 可照搬，先确认权利与时效 | 设计稿的目标照搬；它的系统层预判按核实结果改写 |
| **外部的** | 只读借鉴，读懂再自己写，先查协议 | BlackHole 开发期只当设备用，不复制代码、不分发二进制（GPL-3.0，二进制与商标保留，`resources/BlackHole/LICENSE:1-9`）；Apple 示例代码照结构自己写，保留声明；libASPL（MIT）是备选；Rust 零件按依赖正常引用；Background Music（GPL-2.0+）、roc-vad（MPL-2.0）、MetalVoice（MIT）只学架构 |

⚠️ 交付前已对照：本目录各文件只有转述和 `文件:行号`，没有成段的源码。

**停止线**：三条命门在首选方案下都有「已实现 / 部分 / 没有」的判定和出处，每个缺口都有去处：方案接住、M0 实测、或者交人拍板。到这里就停。

## 支持它的最强依据

> 1. 维护最活跃的开源虚拟麦克风 BlackHole 用的就是「插件内部回环、用户态把它当普通设备写」（ReadInput / WriteMix，`resources/BlackHole/BlackHole/BlackHole.c:4558-4611`，最近提交 2026-10-28；维护者称 macOS 27 上能用【传】，本机未验证）。克隆的 4 个插件仓库（BlackHole · Background Music · roc-vad · libASPL）grep `shm_open` / `mmap` 零命中。
> 2. Apple 官方：插件宿主有沙箱，IO 跑在实时线程上不得阻塞（SDK `AudioServerPlugIn.h:38-42,83-89`）；虚拟设备应该用 HAL 插件，AudioDriverKit 只支持实体设备（developer.apple.com/forums/thread/775341）。
> 3. 零件都在、都活着：rubato 运行中可调比率（`resources/rubato/src/asynchro.rs:806-830`）；rtrb 实时安全（`resources/rtrb/src/lib.rs:1-9`）；cpal 会报流作废和设备断开（`resources/cpal/src/host/coreaudio/macos.rs:251-268`）。

## 反对它的最强依据

> **没有一个开源先例做成过我们这整条路：真麦克风 → 独立推理 → 自有虚拟麦克风，同时满足 20 ms 稳定、漂移补偿、出事送原声。** 唯一同路的 MetalVoice 三条命门全踩：模型跑在实时回调里（`resources/MetalVoice/Sources/Core/AudioProcessing/DeepFilterNetDSP.swift:262-272`）、没有漂移补偿（`resources/MetalVoice/Sources/Core/AudioModel.swift:196-218`）、没有设备列表监听。回环那一跳没人量过；如果模型前瞻已经吃掉大部分预算，这一跳可能逼我们回到共享内存。原声旁路和两层进程都没有先例。

## 仍未补齐的缺口

> **插件加载与崩溃实验**：在 macOS 27 上装一个 ad-hoc 签名的空插件，看它能不能加载；再让它故意崩溃，看 `coreaudiod` 的 pid 变不变、设备是消失还是恢复。这一个实验同时回答命门 1（最坏结果是不是只掉本设备，目前是【推】）和命门 3（开发期要不要付费证书）。它要碰 `/Library/Audio/Plug-Ins/HAL` 和 `coreaudiod`，越过本批红线，见 ⟨待拍板 3⟩。

## 三个风险

1. **签名门槛未定**：macOS 15 以后 ad-hoc 签名插件能否加载，第三方说法互相矛盾（`gap.md` 第〇节 P5 行）。如果不能，M3 开发期就得先付 99 美元/年。缓解：M3 第一天先做 ⟨待拍板 3⟩ 的实验。
2. **整个进程崩溃时送静音 = 吃字**：BlackHole 和所有先例断供时都填零（`resources/BlackHole/BlackHole/BlackHole.c:4561-4571`）；launchd 频繁崩溃时最多每 10 秒拉起一次【推】（本机 `man launchd.plist` 的 ThrottleInterval 一节）。缓解：M1 做输出端旁路，M3 依 ⟨待拍板 4⟩。
3. **新系统与新芯片的变化**：macOS 26.x 在 M5 上，第三方插件因内存标签检查（MTE）被杀，结果是设备一直不注册【传】github.com/Jocala/eqvol/issues/1；macOS 28 不再兼容 Intel 版 HAL 插件，27 起不加载带隔离属性的 launchd plist【官】developer.apple.com/documentation/macos-release-notes/macos-27-release-notes；26 和 27 的 release notes 都没有 Core Audio 节。缓解：插件写得越笨越好，每个系统大版本重跑一次插件实验；架构与最低版本 ⟨待拍板 9⟩。

## 四选一判断

| 候选 | 判断 | 理由 |
|---|---|---|
| **首选：自写 C 插件（照 Apple 示例结构）+ Rust 处理进程** | **在它基础上改造** | 官方示例许可宽松；BlackHole 证明这个形状能用 |
| **备选：libASPL 做插件骨架**（首选卡住时才启用） | **在它基础上改造** | MIT，属性分发和配置变更都包好了。代价：要引入 C++17 和 CMake，最近提交 2025-04。实时路径里持锁（`resources/libASPL/src/Device.cpp:1218`），但 BlackHole 的时间戳函数里同样持锁（`resources/BlackHole/BlackHole/BlackHole.c:4421`），不算它独有 |
| 开发期 BlackHole | **直接用** | 只在作者和测试者机器上用，不分发 |
| roc-vad · Background Music · MetalVoice | 只借鉴架构 | 重活放错了地方，或者方向相反 |
| AudioDriverKit · tympan-aspl · eqMac · Soundflower | 不适合 | 拿不到 entitlement · 太新、没验证 · 和 BlackHole 同一类【传】 · 已废弃 |

## ⚠️ 需要你拍板的地方

「默认」一列 = 你不拍时 AI 实际怎么做、做到哪个里程碑为止。

| # | 需要拍板的 | 选项 | 代价 | 我的建议 | 默认（不拍时） | 拍了吗 |
|---|---|---|---|---|---|---|
| 1 | **本机现在装 BlackHole 吗**（M0 的前提，本机还没装） | A 现在装：`brew install blackhole-2ch` 后重启电脑 / B 先不装 | A 要输一次密码、重启一次；B 卡住 M0 | A | 不装；M0 停在写代码与离线测试，不跑实时 | ☐ |
| 2 | **什么时候办 Apple Developer Program**（99 美元/年，给别人装必需） | A 现在办 / B M3 之前办 / C 不办，只让会用终端的人从源码装 / D 挂靠非营利组织或找赞助 | A、B 要花钱；C 违背「任何在 Mac 上开会的人」；D 慢，而且不确定。个人不能申请费用减免（developer.apple.com/support/fee-waiver） | B | 不办；M0–M2 不受影响，到 M3 分发前停下再问 | ☐ |
| 3 | **批准一次插件加载与崩溃实验**（`docs/experiments/exp001-hal-plugin-load/`） | A 批准：往 `/Library/Audio/Plug-Ins/HAL` 装 ad-hoc 签名的空插件，`killall coreaudiod`，让它故意崩溃一次，做完卸掉 / B 等办了开发者账号再做 / C 不做 | A 全机音频会断几秒，要在不开会时做；B、C 让命门 1、3 的最坏结果一直停在推断 | A，放在 M3 开头 | 不做；M3 开工前停下再问 | ☐ |
| 4 | **处理进程整个崩溃时，对方听到什么**（M1 的输出端旁路只挡模型卡顿，不在本题内，AI 照做） | A 原声：拆成「不带模型的送音进程 + 推理进程」两层 / B 静音（照先例）/ C 让设备报不可用，赌会议软件自动换麦克风（要实验） | A 多一个进程，跨进程音频通道要重写，没有先例；B 违背「从不吃字」；C 依赖 Zoom 等软件的行为，没有出处 | A | 单进程；整个崩溃时送静音，到 M3 前停下再问 | ☐ |
| 5 | **回声消除要不要用 process tap**（多一道「系统录音」权限） | A 默认关，作为可选项 / B 默认开 / C 暂不做，靠会议软件自带的回声消除 | A、B 多一个权限，降噪软件要「录系统声音」看起来像在录对方；C 用扬声器开会的用户可能有回声 | A，先评估 macOS 27 新增的参考设备属性（SDK `AudioHardware.h:1006`） | C | ☐ |
| 6 | **BlackHole 阶段给不给外部用户** | A 只给自己和测试者 / B 公开发布，让用户自己装 BlackHole | B 用户要先装一个陌生软件、重启电脑，还有厂方「非 GPL 应用需买授权」的主张（`resources/BlackHole/README.md:314`，比 GPL 本身管得宽），算不算传染是法律判断 | A | A | ☐ |
| 7 | **装完怎么生效** | A `killall coreaudiod`，提示「请先离开会议」/ B 要求重启电脑（BlackHole 的做法） | A 全机断音几秒，部分 app 扛不住（`resources/roc-vad/README.md:184-190`）；B 稳，但劝退 | A，失败时再提示重启 | B | ☐ |
| 8 | **`must-know.md` 的三条命门是否照核实结果改写** | A 由 AI 照改：第 1 条删去「插件在 coreaudiod 里」的前提，加上 ⟨4⟩ 的结论 / B 你自己改 | A 由 AI 改动你专属的命门文件 | B | 保持草稿和更正注记 | ☐ |
| 9 | **支持哪些 Mac**（最低 macOS 版本、要不要支持 Intel） | A 只支持 Apple Silicon，macOS 14.2+ / B 也支持 Intel / C 只支持最近两个大版本 | A 排除 Intel Mac 用户，和「任何在 Mac 上开会的人」有张力；B 插件要出通用二进制，macOS 28 起 Intel 插件不再兼容【官】，测试面翻倍；C 维护最省，排除的人最多 | A | A（只在作者的 Apple Silicon 上开发） | ☐ |
| 10 | **本项目用什么开源许可证** | A MIT / Apache-2.0 双许可（与 Rust 生态一致）/ B GPL-3.0 / C 其他 | A 不能复制任何 GPL 代码，只能借架构；B 可以借 BlackHole 的代码，但不能用它的名字，下游也受限 | A | 不定；只借架构、不复制代码，两条路都留着 | ☐ |

---

## 停，还是再补一个

能列出上面 10 个拍板点，三条命门在首选方案下都有判定和出处 → **停**。唯一值得补的是 ⟨待拍板 3⟩ 的实验，它要你批准。

## 收尾三件

1. **判断落回真相源**：本卡的结论（插件在独立宿主进程、P4 改成自建 work interval、DriverKit 排除）等你拍板后整理进 `vault/truth/`。本手下没有那里的写权限，已转交大总管。
2. **交付前回一趟参考区**：已对照，没有抄（见上「复用分级」）。
3. **放行**：新开一个空会话只喂本卡，问三句：选了什么？舍了什么？哪里要我拍板？（自审时已做过一次，答得齐。）
