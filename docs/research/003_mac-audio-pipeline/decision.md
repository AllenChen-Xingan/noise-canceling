# 取舍卡 · Mac 上的实时音频管线与虚拟麦克风怎么接

> 2026-10-29，手下 pipeline 起草。**一页纸。** 依据见 `gap.md`（逐项核对与候选横向表）和 `sources.md`（来源登记）。
> ⚠️ 最后一栏「需要你拍板的地方」**AI 不代填**：选项和建议是草稿，「拍了吗」一列留给你。

## 这次不可妥协的几条

1. **崩溃隔离，而且出事时不吃字**：我们的代码出错，不能让整台电脑没声音；处理进程卡住或崩溃时，对方应该听到原声，不是静音。
2. **延迟可量、稳定，能热重建**：附加延迟约 20 ms 且不抖；蓝牙切 16 kHz、热插拔、时钟漂移都不用重启程序。
3. **普通用户装得上**：不关 SIP；签名、公证、密码、重启次数都写清楚，且可以接受。

（第 1 条比 `must-know.md` 的草稿多了「出事时不吃字」：产品派核实发现，所有先例断供时都送静音，而静音正是吃字。见 `gap.md` 第〇节。）

## 我选择什么

> **开发期（M0–M2）：Rust 用 cpal 开两条流，从真麦克风读、写进 BlackHole（固定 48 kHz），中间用 rtrb 做缓冲，rubato 按缓冲水位做漂移补偿；最终产品：自写一个只做「隐藏输出 → 可见麦克风」回环的极简 C 插件（照 Apple NullAudio 的结构），Rust 处理进程把它当普通输出设备写，推理永不进插件。**

## 我放弃什么

> **放弃让插件和处理进程用共享内存直接传音频（留作二期优化），放弃 DriverKit，放弃第一版就用 Rust 或 libASPL 写插件，放弃把 BlackHole 交给外部用户。**

## 分阶段方案

| 阶段 | 虚拟设备 | 处理进程 | 出事兜底 | 安装 |
|---|---|---|---|---|
| **M0 直通** | BlackHole 2ch（开发者自己 `brew install blackhole-2ch` 后重启） | cpal 输入流用麦克风的默认配置，输出流固定 48 kHz 写 BlackHole；输出回调里用 rubato `Async` poly 加 PI 控制器看 rtrb 水位调比率；`StreamInvalidated` / `DeviceNotAvailable` / `DeviceChanged` 汇到同一个防抖重建入口；启动时先预填半个缓冲、淡入 5 ms；用两端回调时间戳实测附加延迟 | 不处理（直通本身就是原声） | 只在作者本机 |
| **M1–M2 接模型** | 同上 | 加一个推理线程，按 10 ms 帧消费环 A、写环 B：`audio_thread_priority` 提优先级，再自建 work interval（约 60 行 FFI）；推理超时就把原声写进环 B（进程内旁路） | 模型卡顿：送原声。进程崩溃：BlackHole 送静音（已知缺口） | 同上，可加少量测试者 |
| **M3+ 自有设备** | 自写 C 插件：一个隐藏输出端、一个可见输入端，插件内部无锁回环；只做时间戳、静音门、运行中改采样率；IO 回调标 `CA_REALTIME_API`（macOS 27 SDK 编译期检查不阻塞） | 不变，只是输出设备从 BlackHole 换成我们的隐藏端 | 推荐拆成两层进程：不带模型的送音进程 + 推理进程。推理进程挂了，送音进程照样送原声（待你拍板） | 一个签名并公证的 pkg：输管理员密码、`killall coreaudiod`、授权麦克风（待你拍板） |
| **二期（视实测）** | 插件改读共享内存 | — | — | 只在回环那一跳实测超出 20 ms 预算时才做 |

**迁移时留下什么**：Rust 侧全部留下（采集、重采样、水位控制、重建逻辑、推理线程、旁路）。只有「输出设备是谁」这一个参数要改，所以从 M0 起就按设备 UID 或名字选输出设备，不写死 BlackHole。新写的只有那个 C 插件和安装包。

## 依据来自哪一类

- [x] 自有积累：设计稿 `vault/raw/20261029-voice-denoise-design.md` §6 / §7 / §11 / §16。目标【你定的】算数；系统层预判 P1 只部分成立，P4 不成立（见 `gap.md` 第〇节）
- [x] 他人积累：12 个克隆仓库逐条读到文件行号，外加 Apple SDK 头文件、官方文档、Developer Forums；补检索查了 5 个网页先例
- [x] 本机只读实测（macOS 27.0.1）：开机日志证明插件在独立宿主进程里加载；宿主进程的签名检查字符串与授权；本机没装 BlackHole；没有 meson
- [ ] 自己跑的实验：**还没跑**。插件加载与崩溃实验需要你批准（见待拍板第 3 条）

## 拿两把尺子量一遍

### 一、复用分级：能用到什么程度

| 来源 | 权限 | 这次实际用到哪一步 |
|---|---|---|
| **自家的** | 可照搬，先确认权利与时效 | 设计稿的目标照搬；它的系统层预判按核实结果改写 |
| **外部的** | 只读借鉴，读懂再自己写，先查协议 | BlackHole：开发期**当设备用**，不复制代码，不分发二进制（GPL-3.0，二进制与商标保留，`resources/BlackHole/LICENSE:1-9`）；Apple NullAudio：照结构自己写（许可证待核）；libASPL：MIT，备选骨架；Rust 零件：MIT / Apache / BSD / MPL，按依赖正常引用；Background Music（GPL-2.0+）、roc-vad（MPL-2.0）、MetalVoice（MIT）：只学架构，一行不抄 |

⚠️ 交付前已对照：本目录各文件只有转述和 `文件:行号`，没有成段的源码。

### 二、满意与最优：什么时候够

**这次的停止线**：三条命门在首选方案下都有「已实现 / 部分 / 没有」的判定和出处；剩下的未知数要么能在 M0 里自然测出来，要么只能靠实验、而实验需要你批准。到这里就停，不再读第 13 个仓库。

## 支持它的最强依据

> 1. 唯一跑在今天 macOS 上、维护活跃的开源虚拟麦克风 BlackHole，用的就是「插件内部回环、用户态把它当普通设备写」（`resources/BlackHole/BlackHole/BlackHole.c:339-340`，最近提交 2026-10-28）。查过的 5 个开源插件没有一个用共享内存传音频。
> 2. Apple 官方：插件宿主有沙箱，IO 跑在实时线程上不得阻塞（SDK `AudioServerPlugIn.h:38-42,83-89`）；虚拟设备应该用 HAL 插件，AudioDriverKit 只支持实体设备（developer.apple.com/forums/thread/775341）。
> 3. 零件都在、都活着：cpal 已经报出全部重建所需的事件（`resources/cpal/src/host/coreaudio/macos.rs:251-268,481-483`）；rubato 运行中可调比率（`resources/rubato/src/asynchro.rs:806-830`）；rtrb 实时安全（`resources/rtrb/src/lib.rs:1-9`）。

## 反对它的最强依据

> **没有一个开源先例做成过我们这整条路：真麦克风 → 独立进程推理 → 自有虚拟麦克风，同时满足 20 ms 稳定、漂移补偿、出事送原声。** 唯一同路的 MetalVoice 三条命门全踩（模型跑在实时回调里，`resources/MetalVoice/Sources/Core/AudioProcessing/DeepFilterNetDSP.swift:262-272`；没有漂移补偿，`resources/MetalVoice/Sources/Core/AudioModel.swift:196-218`；没有设备列表监听）。回环做法比共享内存直供多一个 IO 周期，推算约 1.5–3 ms，没实测。如果模型前瞻已经吃掉大部分预算，这一跳可能逼我们回到共享内存。另外，「原声兜底」和「两层进程」完全没有先例。

## 仍未补齐的缺口

> **插件加载与崩溃实验**：在 macOS 27 上装一个 ad-hoc 签名的空插件，看它能不能加载；再让它故意崩溃，看 `coreaudiod` 的 pid 变不变、设备是消失还是恢复。这一个实验同时回答命门 1（最坏结果到底是不是只掉本设备）和命门 3（开发期要不要付费证书）。要碰 `/Library/Audio/Plug-Ins/HAL` 和 `coreaudiod`，越过本批红线，等你批准。

## 三个风险

1. **签名门槛未定**：macOS 15 以后 ad-hoc 签名插件能否加载，第三方说法互相矛盾（`gap.md` 第〇节 P5 行）。如果不能，M3 开发期就得先付 99 美元/年。缓解：M3 第一天先做上面那个实验。
2. **出事送静音 = 吃字**：BlackHole 阶段处理进程一崩，对方就听到静音，双方都不知道（`resources/BlackHole/BlackHole/BlackHole.c:4561-4571`）。launchd 默认每 10 秒最多拉起一次（本机 `man launchd.plist`），连续崩溃至少丢 10 秒。缓解：M1 起做进程内旁路，M3 做两层进程。
3. **新系统与新芯片的变化**：macOS 26.x 在 M5 上有第三方插件因内存标签检查（MTE）被杀，结果是设备一直不注册【传】github.com/Jocala/eqvol/issues/1；macOS 28 不再兼容 Intel 版 HAL 插件，27 起不加载带隔离属性的 launchd plist【官】developer.apple.com/documentation/macos-release-notes/macos-27-release-notes；26 和 27 的 release notes 都没有 Core Audio 节，变化只能从 SDK 头文件里找。缓解：插件只出 arm64 原生版，写得越笨越好，每个系统大版本都重跑一次插件实验。

## 四选一判断

| 候选 | 判断 | 理由 |
|---|---|---|
| **首选：自写 C 插件（照 NullAudio 的结构）+ Rust 处理进程** | **在它基础上改造** | 官方示例、结构最小、许可证干净；BlackHole 证明这个形状在 macOS 27 上能用 |
| **备选：libASPL 做插件骨架** | **在它基础上改造** | MIT，属性分发和配置变更都包好了；代价是要引入 C++17 和 CMake，最近一次提交是 2025-04；实时路径里持锁（`resources/libASPL/src/Device.cpp:1218`），不过 Apple 示例里也有同样的锁，不算它独有 |
| 开发期 BlackHole | **直接用** | 只在作者和测试者机器上用，不分发 |
| roc-vad · Background Music · MetalVoice | 只借鉴架构 | 重活放错了地方，或者方向相反 |
| AudioDriverKit · tympan-aspl · eqMac · Soundflower | 不适合 | 拿不到 entitlement · 太新、没验证 · 和 BlackHole 同一类 · 已废弃 |

## ⚠️ 需要你拍板的地方

| # | 需要拍板的 | 选项 | 代价 | 我的建议 | 默认（不拍时） | 拍了吗 |
|---|---|---|---|---|---|---|
| 1 | **本机现在装 BlackHole 吗**（M0 的前提，本机还没装） | A 现在装：`brew install blackhole-2ch` 后重启电脑 / B 先不装，M0 往后推 | A 要输一次密码、重启一次；B 卡住 M0 | A | B | ☐ |
| 2 | **什么时候办 Apple Developer Program**（99 美元/年，给别人装必需） | A 现在办 / B M3 之前办 / C 不办，只让会用终端的人从源码装 / D 挂靠非营利组织或找赞助 | A、B 要花钱；C 违背「任何在 Mac 上开会的人」；D 慢，而且不确定。个人不能申请费用减免（developer.apple.com/support/fee-waiver） | B | C | ☐ |
| 3 | **批准一次插件加载与崩溃实验**（`docs/experiments/exp001-hal-plugin-load/`） | A 批准：往 `/Library/Audio/Plug-Ins/HAL` 装 ad-hoc 签名的空插件，`killall coreaudiod`，让它故意崩溃一次，做完卸掉 / B 等办了开发者账号再做 / C 不做 | A 全机音频会断几秒，要在不开会时做；B、C 让命门 1、3 的最坏结果一直是推断 | A，放在 M3 开头 | C | ☐ |
| 4 | **处理进程出事时，对方听到什么** | A 原声（M1 做进程内旁路，M3 做两层进程）/ B 静音（照先例，最省事）/ C 让设备报不可用，赌会议软件自动换麦克风（要实验） | A 多一个进程、无先例；B 违背「从不吃字」；C 依赖 Zoom 等软件的行为，没有出处 | A | B | ☐ |
| 5 | **回声消除要不要用 process tap**（多一道「系统录音」权限） | A 默认关，作为可选项 / B 默认开 / C 暂不做，靠会议软件自带的回声消除 | A、B 多一个权限，降噪软件要「录系统声音」看起来像在录对方；C 戴扬声器开会的用户可能有回声 | A，先评估 macOS 27 新增的参考设备属性 | C | ☐ |
| 6 | **BlackHole 阶段给不给外部用户** | A 只给自己和测试者 / B 公开发布，让用户自己装 BlackHole | B 用户要先装一个陌生软件、重启电脑，还有厂方「非 GPL 应用需买授权」的主张（`resources/BlackHole/README.md:314`，比 GPL 本身管得宽），算不算传染是法律判断 | A | A | ☐ |
| 7 | **装完怎么生效** | A `killall coreaudiod`，提示「请先离开会议」/ B 要求重启电脑（BlackHole 的做法） | A 全机断音几秒，部分 app 扛不住（`resources/roc-vad/README.md:184-190`）；B 稳，但劝退 | A，失败时再提示重启 | B | ☐ |
| 8 | **`must-know.md` 三条命门是否照本卡第一节改写** | A 照改（第 1 条加上「出事时不吃字」，删去「插件在 coreaudiod 里」的前提）/ B 你自己改 | — | A | 保持草稿 | ☐ |

---

## 停，还是再补一个

能列出上面 8 个拍板点，三条命门在首选方案下都有判定和出处 → **停**。唯一值得补的是第 3 条那个实验，它要你批准。

## 收尾三件

1. **判断落回真相源**：本卡确认的结论（插件在独立宿主进程、P4 改成自建 work interval、DriverKit 排除、出事送原声）等你拍板后整理进 `vault/truth/`。本手下没有那里的写权限，已转交大总管。
2. **交付前回一趟参考区**：已对照，没有抄（见上「复用分级」）。
3. **放行**：新开一个空会话只喂本卡，问三句：选了什么？舍了什么？哪里要我拍板？
