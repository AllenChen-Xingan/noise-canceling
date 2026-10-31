# r3 · 省电核（E-core）上小更新的算力 / 能耗量级 + 「充电空闲才训练」先例

> 只读检索，未跑任何程序。等级：【官方】【论文】【源码】【二手】【推算】。

## 预判与证伪

我原本的预判是：小更新在 E 核上只要「秒级、焦耳级」。回到源头核对后，量级成立，但有两处要改：
- E 核功耗比我预想的还要低一个量级：单核约 0.1–0.3 W，而不是 0.5 W。
- macOS 上**没有** `requiresExternalPower` 这个开关（它只在 iOS 和 Catalyst 上有），「必须插电」得自己判断。

## 1. E 核硬件数字

| 项 | 数字 | 出处 · 等级 |
|---|---|---|
| E 核最高频率 | M1 2064 · M2 2424 · M3 2748 · M4 2892 · M5 3048 MHz（M5 空闲 972 MHz） | eclecticlight.co/2026/04/13/cpu-core-frequencies-updated-for-all-current-apple-silicon-macs/（powermetrics）【二手·实测】 |
| P 核最高频率 | M1 3228 · M4 4512 · M5 4380（S 核 4608）MHz | 同上【二手·实测】 |
| SIMD 流水线 | Icestorm（M1 E 核）有 2 条 128-bit FP/SIMD 管线（u6–7，吞吐 0.5 周期）；Firestorm（M1 P 核）有 4 条（u11–14） | dougallj.github.io/applecpu/icestorm-simd.html · firestorm-simd.html【源码·微基准】 |
| 单核 FP32 峰值 | M1 E 核：2×4×2 FLOP×2.064 GHz ≈ **33 GFLOPS**；M1 P 核：4×4×2×3.2 ≈ **102 GFLOPS**。M4/M5 的 E 核管线数**没找到**；若仍是 2 条，约 46–49 GFLOPS | 【推算】 |
| E 核能用 NEON 吗 | 能：E 核「完全支持浮点与 NEON」 | eclecticlight.co/2022/10/05/making-the-most-of-apple-silicon-power-2-core-capabilities/【二手】 |
| E 核能用 AMX 吗 | 每个簇（含 E 簇）各有一个 AMX，只能经 Accelerate 调用；E 簇的 AMX 小得多，性能不稳定 | eclecticlight.co/2023/12/13/finding-and-evaluating-amx-co-processors-in-apple-silicon-chips/ · github.com/corsix/amx【二手/逆向】 |
| E 核功耗 | M1 Pro 两核 E 簇满载、2064 MHz 时约 **210 mW**，四核 P 簇约 4 W（约 20 倍）；M1 四核 E 簇在约 1 GHz 时为 160–170 mW；每个 E 核最高约 300 mW，每个 P 核最高约 2.5 W | eclecticlight.co/2022/01/03/power-frequency-management-how-m1-e-cores-win/ ；2022/10/05 同上【二手·powermetrics】 |
| 同一任务的能耗 | M3 Pro：P 核每核约 1.0–1.3 J；E 核高频约 0.5 J，低频约 0.2 J（约为 P 核的 20%） | eclecticlight.co/2024/02/26/apple-silicon-3-but-does-it-save-energy/【二手·实测】 |
| E 核相对 P 核的速度 | M1 的 E 核在高频时为 P 核的 40–60%，真实压缩任务约 1/4；M4 的 E 核在低 QoS 下 NEON 为 M1 P 核的 73% | eclecticlight.co/2024/11/20/inside-m4-chips-cpu-core-performance/【二手】 |

AnandTech 和 Chips and Cheese 的 E 核单核功耗原文**没找到**（检索只得到转述）。

## 2. 小更新的耗时与能耗【推算】

**假设：**
- 前向算力 33 MMACs/s ≈ 66 MFLOP/s（每秒音频）。
- 用实测锚点：最坏一帧 0.62 ms / 16 ms，实时因子（RTF）≈ 0.039（按 P 核算）。
- E 核取 P 核速度的 1/4（保守），即 E 核 RTF ≈ 0.155。
- E 核功率取 0.3 W（上限）。
- 电池按 MacBook Air 13" 的 53.8 Wh = 193.7 kJ 计（apple.com/macbook-air/specs/【官方】）。MBP 14" 为 72.4 Wh（apple.com/macbook-pro/specs/【官方】）。

**(a) 只重算归一化统计 / 调节向量（前向 60 s 音频）**
- 耗时：60 × 0.155 ≈ **9 s** E 核时间。
- 能耗：9 × 0.3 ≈ **2.8 J**，占电池 1.4×10⁻⁵（约 0.001%）。
- FLOP 法交叉核对：60 × 66M = 4 GFLOP。按 33 GFLOPS 峰值的 10% 算，约 1.2 s。
- 结论：秒级。

**(b) 一次梯度步（前向 + 反向 ≈ 3× 前向，N = 60 s 音频）**
- 耗时：3 × 9 ≈ **28 s**，能耗约 **8.4 J**。FLOP 法约 3.6 s。两种算法差 8 倍，取上界。
- 放大到「10 分钟音频 × 10 轮」：3 × 600 × 10 × 0.155 ≈ 2790 s ≈ 46 min，能耗 ≈ 837 J ≈ **0.23 Wh**，占 53.8 Wh 的 **0.43%**。
- 风险系数：通用框架在 CPU 上训练的开销可能 ×2–5。即便如此也低于 2%，而且是在插电时发生。
- 内存：反向传播（BPTT）要存激活。48K 参数本身很小，但长序列会撑大内存。按 1–4 s 截断即可（【推算】，未验证）。

## 3. 参照：实时推理本身

- 占用：0.62 / 16 ≈ 3.9% 单核（最坏帧，平均更低）。
- 落在 P 核：按每核 2.5 W 上限 × 4% ≈ **≤ 0.1 W**。
- 落在 E 核：约 15% 占用 × 0.3 W ≈ **0.05 W**。
- 每小时 0.05–0.1 Wh，占 53.8 Wh 的 **0.1–0.2%/h**【推算】。
- 结论：实时降噪本身比一次小更新贵得多（开 1 小时会 ≈ 180–360 J，是 (a) 的约 100 倍）。**省电的重心在反射回路，不在学习回路。**

## 4. 「充电空闲才训练」先例（只借调度，不借上传）

- **Google FL 博客（2017-04-06）**：「training happens only when the device is idle, plugged in, and on a free wireless connection」。research.google/blog/federated-learning-collaborative-machine-learning-without-centralized-training-data/【官方】
- **Towards FL at Scale（arXiv:1902.01046，§3，p.4）**：
  - 用 Android JobScheduler，「only invoke the job when the phone is idle, charging, and connected to an unmetered network」。
  - 「Once started, the FL runtime will abort … if these conditions are no longer met」。
  - 训练会话在设备上不并行。
  【论文】
- **FL for Mobile Keyboard Prediction（arXiv:1811.03604，§5.2，p.4）**：「at least 2 gigabytes of memory available … only allowed to participate if they are charging, connected to an un-metered network, and idle」。【论文】
- **Face ID（Face ID Security Guide，2017-11，p.4）**：
  - 「augments its stored mathematical representation over time」；新表征只用于「a finite number of additional unlocks before that data is discarded」。
  - 匹配质量高于阈值、紧接着又输入了密码时，才增补。
  - 在 Secure Enclave 内完成，不出设备。
  - apple.com/business-docs/FaceID_Security_Guide.pdf【官方】
  - 借鉴：**有门槛才吸收、有限寿命、自动过期** = 可回滚。
- **Hey Siri（ML Journal 2018）**：隐式注册「add the latest accepted speaker vector to the user profile until it contains 40」；明式注册 5 条。machinelearning.apple.com/research/personalized-hey-siri【官方】
  - 借鉴：声纹只追加「被接受」的样本，并设上限。
- **Photos 人物识别**：聚类「runs periodically, typically overnight during device charging」。machinelearning.apple.com/research/recognizing-people-photos【官方】
- **Core ML MLUpdateTask**：可在设备上更新「updatable」模型，只能更新编译后的 `.mlmodelc`。developer.apple.com/documentation/coreml/personalizing-a-model-with-on-device-updates【官方】
  - 可更新层只限全连接和卷积：这条只见于二手资料。
  - Apple 文档本身**没有**写调度条件。
- **macOS 调度 API**：
  - `NSBackgroundActivityScheduler` 由系统「based on energy usage, thermal conditions, and CPU use」决定何时执行；默认 QoS 为 Background。
  - `shouldDefer` 的典型场景是「the user has unplugged the Mac」。
  - developer.apple.com/documentation/foundation/nsbackgroundactivityscheduler【官方】
  - `BGProcessingTaskRequest.requiresExternalPower` 的可用平台只有 iOS / iPadOS / Catalyst / tvOS / visionOS，**没有原生 macOS**。developer.apple.com/documentation/backgroundtasks/bgprocessingtaskrequest/requiresexternalpower【官方】

## 5. 用户感知阈值

- **Energy Impact 的定义**：「A relative measure of the current energy consumption of the app (lower is better)」，没有单位。support.apple.com/guide/activity-monitor/actmntr43697【官方】
- **它怎么算**：
  - 10.10 以后综合 CPU、唤醒、QoS、磁盘、GPU、网络，系数在 `/usr/share/pmenergy/Mac-<id>.plist`。
  - 「Apps Using Significant Energy」大约在 EI ≥ 20 时出现，通常一分钟内上榜（经验观察，作者自称不可靠）。
  - firefox-source-docs.mozilla.org/performance/activity_monitor_and_top.html【二手】
- **2026 年的实测**：EI 与 powermetrics 的 CPU 核功率强相关、与 GPU 无关；P/E 的 EI 比约 173，功率比约 200。所以**放在 E 核，EI 几乎看不见**。eclecticlight.co/2026/06/29/what-does-activity-monitor-measure/【二手·实测】
- **风扇起转阈值**：**没找到官方数字**。
  - M3 起风扇由硬件启动（TG Pro 开发者，【二手】）。
  - MacBook Air 无风扇，发热直接降频。
  - 【推算】一个 E 核 0.3 W 远低于整机常见负载，不会是起转主因。

## 对我们的含义

- **放哪条回路**：
  - 归一化统计 / 调节向量的重算，判断回路可做在线 EMA 版，学习回路做整批版。
  - 梯度步只进学习回路。
- **预算量级**：
  - 单次更新：E 核 ≤ 60 s、≤ 20 J。
  - 每天上限：≤ 1 Wh（< 2% 电池），且只在插电时。
  - 实时推理：≤ 0.1 W，作为反射回路的红线。
- **护栏**：
  - 用 `NSBackgroundActivityScheduler`，QoS = background（自动落在 E 核）。
  - 自己用 IOKit 判断「插电 + 未开会 + `ProcessInfo.thermalState == nominal`」。
  - 用 `shouldDefer` 或条件一变就中止（照抄 1902.01046 的 abort 语义）。
  - 一次只跑一个会话。
  - 更新产物有寿命，过期自动丢弃（Face ID 式）。

**首选**：学习回路只做「前向重算统计 + 调节向量的几十步梯度」，用 NEON / Accelerate 跑在 background QoS 的 E 核上，按 Face ID 式门槛吸收、有限寿命、影子比对后再替换。

**备选**：彻底不做梯度，只做统计重估和「在现成配置里选」（Siri 式：追加被接受的样本，上限 N）。成本是 (a) 级，约几 J。

**三个风险**：
1. E 核实际训练效率未知（M4/M5 的管线数没找到，框架开销 ×2–5）。上界推算仍 < 2% 电池，但要实测一次 powermetrics。
2. macOS 没有现成的「需外接电源」开关，自己的判断写错会在电池上跑。
3. background QoS 的作业可能被系统长期推迟，导致更新迟迟不发生。另外 BPTT 的内存峰值未验证。
