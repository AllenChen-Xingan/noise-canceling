# 问题 4 · 波束成形（2–3 麦小阵列）调研

## 一句话结论

**在 Mac 内置麦克风上，我们拿不到多路原始信号（只拿到 1 路），所以无法自己做波束成形。** 系统给出的这 1 路，很可能已经做过“定向”处理（Apple 官方只说阵列有定向波束成形，没说哪一层做，属【推断】）。就算接上 USB 双麦（间距 1–3 cm），经典波束成形（延迟求和 / GSC / MVDR / 超指向）在“旁边有人说话”场景的仿真增益也接近 0（arXiv 2412.18141，仿真房间、一组角度；babble 未单独测）。真正有用的是“神经网络 + 空间线索”这一类（双通道 GTCRN、CDUNet）。但它们都要另外训练，而且至今没有找到可直接用、许可兼容 GPL-3.0 的权重。**建议：主线（M1/M2）不做波束成形。只在场景层留一个接口：检测到多通道外接麦时启用，作为后续实验线。**

## 证据表

| # | 结论 | 数字 / 条件 | 出处 | 标记 |
|---|---|---|---|---|
| A1 | 在 macOS 上，内置麦 “MacBook Pro Microphone” 的流格式是 **1 路** | Stream Format 1 ch；Element Count 2（1 个全局元素 + 1 个声道），不是 2 支麦 | https://developer.apple.com/forums/thread/763325 （Apple 员工回答） | 【官方】（开发者论坛中的 Apple 员工回答） |
| A2 | Apple 称三麦阵列具备“高信噪比 + 定向（beamforming）拾音” | 用户手册原文：“The three-microphone array … high signal-to-noise ratio …”，后半句讲定向波束成形 | https://support.apple.com/guide/macbook-pro/apd2182293a1/2023/mac/14 | 【官方】 |
| A3 | 硬件实际是 3 支全向 PDM 麦。在 Linux 上它们以 3 路原始信号直接送到用户态，没有前置放大；不做波束成形就“太灵敏、全向、不可用” | “All three mics are plumbed directly to userspace on separate channels”；“some kind of beamforming needs to be applied” | https://asahilinux.org/docs/sw/audio-userspace/ | 【官方】（Asahi 项目文档） |
| A4 | Asahi 作者称，macOS 在用户态实现了自适应波束成形，并表示 Triforce “别指望胜过 Apple 的实现” | README 原句 | https://github.com/chadmed/triforce | 【传言】（第三方作者的判断，Apple 未证实） |
| A5 | 推论：我们在 macOS 上收到的 1 路信号，已经是 Apple 波束成形之后的结果。拿不到原始 3 路，也就无从自己做“第二次”波束成形 | 由 A1 + A2 + A4 推出 | — | 【推断】 |
| B1 | WebRTC 旧版 NonlinearBeamformer 已于 **2018-06-21** 从 APM 删除（删了 16 个 `beamformer/` 文件，bug webrtc:9402，#23694）。更早在 2016 年已删掉部分接口（#13451） | — | https://webrtc.googlesource.com/src/+/db38972edaf0c2c748343e72bf64682a1351844c ；https://codereview.webrtc.org/2129663002 | 【源码】 |
| B2 | PulseAudio 9.0 曾通过 `beamforming=true mic_geometry= target_direction=` 调用它。2020 年接入 AEC3 时这些选项被删除 | — | https://wiki.freedesktop.org/www/Software/PulseAudio/Notes/9.0 ；https://build.opensuse.org/projects/GNOME:STABLE:45/packages/pulseaudio/files/echo-cancel-add-webrtc-AEC3-support.patch?expand=0 | 【官方】/【源码】 |
| B3 | sonora（BSD-3，移植自 WebRTC M145，后续修复跟到 M156）**没有波束成形**，多通道输入只做平均下混 | `crates/sonora/src/audio_converter.rs:5`（只支持下混到单声道）；`audio_buffer.rs:44`（`downmix_by_averaging`）；README 只列 AEC3 / AGC2 / NS | 本地 `resources/sonora` | 【源码】 |
| B4 | SpeexDSP 没有波束成形模块 | 头文件只有 buffer / echo / jitter / preprocess / resampler / types | https://github.com/xiph/speexdsp/tree/master/include/speex | 【源码】 |
| B5 | ODAS：C 语言，MIT 许可，做定位、跟踪、分离和后置滤波，面向嵌入式；ManyEars 是它的前身（GPL-3.0-or-later，已不再推荐，README 让用户改用 ODAS） | 未查到 2–3 麦、cm 级间距下的增益数字 | https://github.com/introlab/odas ；https://github.com/introlab/manyears | 【官方】 |
| B6 | BeamformIt：C++，**离线批处理**，加权滤波求和 + GCC-PHAT（可选 Viterbi），面向会议录音；不能实时使用；许可页面未写明 | — | https://github.com/xanguera/BeamformIt | 【官方】 |
| B7 | Pyroomacoustics 提供 DAS / MVDR / Max-SINR / Perceptual 等 Rake 系列算法，只适合当参照实现 | 许可是 MIT（据项目 LICENSE；本次只看了文档页，文档页上没写） | https://pyroomacoustics.readthedocs.io/en/pypi-release/pyroomacoustics.beamforming.html | 【官方】 |
| B8 | Triforce：Rust，**GPL-2.0-or-later**（可升级为 GPL-3，与本项目兼容），3 麦 MVDR，LV2 插件。实现是时域、窄带：用 Hilbert 变换得到解析信号，在单一频点上做导向；协方差用块平均 + 1e-4 对角加载；没有显式 SIMD，作者自称“性能差”。`steering_vec` 里的波数写成了 `2π/(f/C)`，疑似写错（应为 2πf/C） | Cargo.toml `license = "GPL-2.0-or-later"`；lib.rs 约 37 行 | https://github.com/chadmed/triforce （Cargo.toml、src/lib.rs） | 【源码】（波数写错是我的阅读判断，属【推断】） |
| C1 | **双麦 3 cm、干扰为另一个说话人、SNR 0 dB 时，经典波束成形几乎没有收益**。PESQ：带噪 2.08 → DAS 2.09 / GSC 2.12 / JNF 2.09；神经网络 CDUNet（74.4k 参数）2.50 | 表 I（目标 90°，干扰 0–180°） | arXiv 2412.18141 表 I | 【论文】 |
| C2 | 同上条件下的 WER（NeMo FastConformer）：带噪 6.65 → DAS 6.39 / GSC 5.36 / JNF 6.97 / CDUNet 4.35（干净参考 2.20） | 表 IV，0 dB | arXiv 2412.18141 表 IV | 【论文】 |
| C3 | CDUNet 的 STFT 窗长 512、步长 256（LibriSpeech 16 kHz → 窗 32 ms），因果，需要输入目标方向和角宽；代码与许可未查到 | §III-B | arXiv 2412.18141 | 【论文】；代码【未查到】 |
| C4 | **双通道 GTCRN**（4 cm 线阵，DNS-3 + DCASE 噪声，SNR −12.5 / −7.5 / −2.5 dB）。PESQ：单通道 GTCRN 1.09 / 1.16 / 1.31 → DC-GTCRN 1.15 / 1.29 / 1.50 → IVA + GTCRN 1.39 / 1.58 / 1.71。STOI：47.5 / 61.1 / 72.9 → 60.2 / 71.3 / 79.7 → 72.4 / 79.0 / 82.0 | 参数 23.4k / 23.9k / 24.4k；MACs 32.1 / 35.6 / 43.2 M/s；论文未报 SI-SDR，也未专门测 babble | arXiv 2505.19597 表 2 | 【论文】 |
| C5 | 双通道 GTCRN 仓库 github.com/Max1Wz/H-GTCRN 只确认有音频样例，代码与许可未确认 | — | arXiv 2505.19597 | 【未查到】 |
| C6 | FaSNet / FaSNet-TAC（时域神经波束成形，适用于自组织阵列和不同麦克风数）：摘要里没有窗长、麦克风数与具体增益 | — | arXiv 1910.14104 | 【论文】（细节未查到） |
| D1 | 理论（扩散噪声场）：N 麦超指向阵的最大指向因子是 N²。2 麦约 6 dB，3 麦约 9.5 dB；一阶差分阵列中，超心形 DF = 4（6.0 dB），心形 DF = 3（4.8 dB）。代价是低频白噪声增益严重下降，间距 1–2 cm 时低频（< 500 Hz）放大自噪声几十 dB，实际必须正则化，可得增益明显缩水。间距远小于波长时，延迟求和对扩散噪声的增益趋近 0 dB | 教科书结论 | Benesty & Chen, *Study and Design of Differential Microphone Arrays*, Springer 2013（章节 / 页码本次未核对） | 【论文】（具体页码未核对） |
| D2 | 神经掩码驱动 MVDR（Heymann / Erdogan 2016 一类）需要较长时间平滑协方差；在 2 麦小间距下，空间自由度只有 1 个零点，最多压住 1 个方向性干扰，对 6 人 babble（近似扩散）帮助有限 | — | 由阵列自由度推出 | 【推断】 |

## 对本项目的含义

- **内置麦（大多数用户）**：只能拿到 1 路已经处理过的信号（A1、A5），所以 **M1 / M2 不可能自己做波束成形，也没有必要**。“再做一次”在技术上就不成立，因为没有多路输入。代价是：我们的 GTCRN 拿到的是 Apple 波束成形之后的信号，babble 场景里已经吃到了 Apple 的空间增益，剩下的那部分就是 exp003 吃字的部分。
- **系统里的另一个开关**：Voice Isolation / Wide Spectrum 等麦克风模式（`AVCaptureDevice.MicrophoneMode`）。各平台从哪个版本起支持、是否只作用于 VPIO 通路，本次**未查到官方原文**，归入 gap。
- **外接 USB 双麦 / 多通道接口**：
  - 经典方法（DAS / GSC / MVDR / DMA）：延迟方面，时域 DAS / DMA 只有分数延迟，约 0–1 ms；频域 MVDR 如果和 GTCRN 共用同一个 STFT（512 / 256 @ 16 kHz），**额外延迟约等于 0**。算力方面，257 个频点 × 2×2 或 3×3 矩阵求逆，每 16 ms 一次，【推断】远低于 0.1 ms。但对 babble 和旁人说话的实测增益约等于 0（C1、C2），**不值得作为对付多人说话的主手段**。
  - 神经多通道（DC-GTCRN、CDUNet）：有明确增益（C4：低 SNR 下 STOI +8~13 点；C2：WER 6.65 → 4.35），算力和 GTCRN 同一量级（35–43 M MACs/s）。但需要我们自己训练（GTCRN 是 MIT 许可，可以改结构），没有现成权重；训练数据必须匹配目标设备的麦克风间距。
- **许可**：可直接参照或移植的只有 Triforce（GPL-2.0+，兼容）、ODAS（MIT）、Pyroomacoustics（MIT，只作参照）、sonora（BSD-3，没有波束模块）。WebRTC 旧波束器是 BSD 许可，但已删除，只能从 2018 年之前的历史版本里取，而且无人维护。
- **在哪一层**：放在**场景层**。检测到输入设备通道数 ≥ 2 且不是内置麦时，提示或自动切换到“多麦模式”。判断回路只改参数（目标方向、开关），不碰声音。**M1 / M2 不排期**，建议另开实验线 `exp/<NNN>-dualmic`。

## gap（需实测或未查到）

1. 在 macOS 上能否通过某种 HAL 属性或聚合设备拿到内置 3 麦的原始信号：未查到，需要实测（枚举 `kAudioDevicePropertyStreamConfiguration`）。
2. Apple 的内置波束成形在默认（Standard）模式下是否一直开着、是否跟随 Voice Isolation 变化：Apple 没有公开，需要做 A/B 录音实测。
3. 常见 USB “双麦 / 立体声”麦克风（X/Y 心形、网络摄像头、耳机）到底是吐原始双路，还是机内已经处理：要逐款实测。不少设备机内带 DSP，或者是 X/Y 指向咪头，这时 DAS 的假设不成立。
4. DC-GTCRN / CDUNet 的代码与许可：未查到。需要自训时，要在 1–3 cm 间距、6 人 babble、SNR −5 dB 的条件下测 SI-SDR / WER（复用 exp003 的素材，并补录双通道版本）。
5. WebRTC 旧 NonlinearBeamformer 的算法延迟与增益数字：未查到。
6. Triforce 在 Apple Silicon 上的 CPU 占用，以及“波数写错”是否属实：需要读完整源码或实测。
7. Benesty & Chen 教科书中 DF / WNG 数值的具体页码：未核对。

## 来源清单

- Apple 开发者论坛 thread 763325：https://developer.apple.com/forums/thread/763325
- MacBook Pro 用户手册：https://support.apple.com/guide/macbook-pro/apd2182293a1/2023/mac/14
- Asahi 音频用户态文档：https://asahilinux.org/docs/sw/audio-userspace/
- Triforce：https://github.com/chadmed/triforce （Cargo.toml、src/lib.rs）
- WebRTC 删除波束器的提交：https://webrtc.googlesource.com/src/+/db38972edaf0c2c748343e72bf64682a1351844c ；https://codereview.webrtc.org/2129663002
- PulseAudio 9.0 发布说明：https://wiki.freedesktop.org/www/Software/PulseAudio/Notes/9.0 ；AEC3 补丁：https://build.opensuse.org/projects/GNOME:STABLE:45/packages/pulseaudio/files/echo-cancel-add-webrtc-AEC3-support.patch?expand=0
- sonora 本地源码：`/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling/resources/sonora/crates/sonora/src/audio_converter.rs:5`、`audio_buffer.rs:44`
- SpeexDSP：https://github.com/xiph/speexdsp/tree/master/include/speex
- ODAS：https://github.com/introlab/odas ；ManyEars：https://github.com/introlab/manyears ；BeamformIt：https://github.com/xanguera/BeamformIt
- Pyroomacoustics：https://pyroomacoustics.readthedocs.io/en/pypi-release/pyroomacoustics.beamforming.html
- arXiv 2412.18141（CDUNet，双麦 3 cm，表 I / 表 IV）
- arXiv 2505.19597（IVA + 双通道 GTCRN，4 cm，表 2）
- arXiv 1910.14104（FaSNet-TAC）
- Benesty & Chen, *Study and Design of Differential Microphone Arrays*, Springer 2013
