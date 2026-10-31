# r7 实时音频的浸泡 / 混沌 / 模糊测试 · 无遥测下发现长尾

> 只做了检索和阅读，没有运行任何东西。证据等级：【官方】【论文】【源码】【二手】【推算】。核查日期 2026-10-30。

## 1 模糊测试（fuzzing）与性质测试（property testing）

- **opus**：OSS-Fuzz 的 build.sh 构建 6 个 fuzzer：`opus_encode` / `opus_multistream_decode` / `opus_multistream_encode` / `opus_projection_decoder` / `opus_projection_encoder` / `opus_repacketizer`（均为 `_fuzzer`，源在 `tests/*.cc`），每个按 floating / fixed / floating_qext / fixed_qext 四种配置各出一份，共 24 个二进制。https://github.com/google/oss-fuzz/blob/master/projects/opus/build.sh 【源码】。预判「有 opus_decode_fuzzer」与该脚本对不上，**已证伪**（只剩多流解码那一个）。
- **flac**：仓内 `oss-fuzz/` 下有 decoder / encoder / encoder_v2 / metadata / reencoder / seek / tool_flac / tool_metaflac。https://github.com/xiph/flac/tree/master/oss-fuzz 【源码】
- **libvorbis**：`contrib/oss-fuzz/decode_fuzzer.cc`（2018-04 加入）。https://gitlab.xiph.org/xiph/vorbis/blob/master/contrib/oss-fuzz/decode_fuzzer.cc 【源码】
- **ffmpeg**：`tools/target_{dec,enc,dem,bsf,swr,sws}_fuzzer.c`，其中 swr 是重采样，和我们最相关。https://github.com/FFmpeg/FFmpeg/tree/master/tools 【源码】
- **webrtc**：`test/fuzzers/` 下有 `aec3_fuzzer`、`agc_fuzzer`、`audio_processing_configs_fuzzer`、`residual_echo_detector_fuzzer`、`neteq_signal_fuzzer`，还有一批 opus/ilbc/isac 编解码 fuzzer。https://github.com/webrtc-sdk/webrtc/tree/master/test/fuzzers 【源码】。`audio_processing_configs_fuzzer` 让模糊器把**配置**和信号一起随机，这正是「判断回路改参数」该照着做的样子。
- **Rust**：cargo-fuzz 的目标放在 `fuzz/fuzz_targets/`，需要 nightly。https://rust-fuzz.github.io/book/ 【官方】。Rust DSP 专门的 cargo-fuzz 先例：**没找到**。
- **性质测试**：captchaforge 用 proptest 生成 [-1,1] 的 f32 序列，断言输出全部 `is_finite`、长度与采样率不变、峰值有界。https://docs.rs/crate/captchaforge/latest/source/tests/property_audio_dsp.rs 【源码】。「静音进 → 静音出」「延迟恒定」两条不变量的公开先例：**没找到**，要我们自己写。

## 2 插件与实时音频的验证工具

- **pluginval**：严格度 1–10，5 是「宿主兼容的最低线」。https://github.com/Tracktion/pluginval 【官方】。各项测试在源码里对应的级别：Audio processing（3 级，遍历采样率 × 块大小，查 NaN / Inf / 次正规数 subnormal）· Automation（3 级，按 32 样本切块并随机改参数；6 级起次正规数算错误）· auval（5 级，6 级起加 stress）· Non-releasing（6 级，切换采样率前不调 release）· Parameter thread safety（7 级）。https://raw.githubusercontent.com/Tracktion/pluginval/master/Source/tests/BasicTests.cpp 【源码】。`--rtcheck enabled` 用的是作者自己写的 rtcheck 库，不是 RTSan；macOS 端基本完成，代码还在 feature 分支（2025-10-29）。https://forum.juce.com/t/pluginval-real-time-safety-checking/67439 【二手·作者本人帖】
- **RTSan**：给函数标 `[[clang::nonblocking]]` 后，函数里调用 malloc / free / 加锁就报错；用 `-fsanitize=realtime` 开启，可配 `RTSAN_OPTIONS=halt_on_error`，文档称运行开销「可忽略」。https://clang.llvm.org/docs/RealtimeSanitizer.html 【官方】。Rust 端有封装 crate `rtsan-standalone`（用 `#[nonblocking]` 标注，只支持 Linux / macOS / iOS）。https://lib.rs/crates/rtsan-standalone 【二手】。另有 `assert_no_alloc`，只拦分配不拦锁，release 构建默认不生效。https://docs.rs/assert_no_alloc 【源码】
- **HAL 插件**：Apple 给了虚拟设备示例 "Creating an Audio Server Driver Plug-in"（双声道 f32，44.1 / 48 kHz）。https://developer.apple.com/documentation/coreaudio/creating-an-audio-server-driver-plug-in 【官方】。新系统里插件跑在独立进程，可以按 PID 附加调试。https://developer.apple.com/forums/thread/712359 【二手·DTS】。HALLab 的官方下载页**没找到**。
- **WebRTC 音质回归**：`low_bandwidth_audio_test.py` 先跑通话，再用 16 kHz 的 PESQ 和 48 kHz 的 POLQA 给输出打分；找不到工具时会跳过而不是报失败（这些工具因授权原因只在 Google 内部）。https://webrtc.googlesource.com/src/+/refs/branch-heads/4472/audio/test/low_bandwidth_audio_test.py 【源码】。卡顿（glitch）用 `concealedSamples / totalSamplesReceived` 和 `concealmentEvents` 衡量。https://developer.mozilla.org/en-US/docs/Web/API/RTCInboundRtpStreamStats 【官方】

## 3 浸泡测试（soak）

- PipeWire 的 `pw-top` 有 ERR 列，统计 xrun 和错误的总数；跟随节点的 xrun 指「开始处理了，但没在周期截止前做完」。https://man.archlinux.org/man/pw-top.1 【官方】
- 公开的浸泡时长标准、漂移阈值，以及 Bitwig / Ableton / JACK 在 CI 里做浸泡的工程文章：**都没找到**。只有零散论坛帖，比如「跑几小时后开始 xrun」最后查出是次正规数，不是内存泄漏。https://cm-mail.stanford.edu/pipermail/planetccrma/2005-January/007922.html 【二手】，正好说明 pluginval 的次正规数检查有必要。

## 4 本地客户端上的混沌测试

- 睡眠 / 唤醒可以脚本化：`pmset relative wake <秒>` 加 `pmset sleepnow`，再用 `pmset -g sched` 查看排程。https://sudo.is/docs/macos/man_pmset.html 【二手·man 镜像】
- 打满 CPU：`stress-ng` 有 Homebrew 版（0.22.01，带 Apple Silicon 预编译包）。https://formulae.brew.sh/formula/stress-ng 【官方】
- 设备增删与格式切换：HAL 插件的结构变化要走宿主的 `RequestDeviceConfigurationChange`，不能只发 `PropertiesChanged`。https://developer.apple.com/forums/thread/84892 【二手】。现成的「虚拟设备拔插模拟器」**没找到**，只能自己在插件里加一个测试开关。
- 论文：arXiv:2602.06223 讲的是移动端遭遇后端故障，不是在客户端内部注故障；arXiv:2505.13654 统计到 GitHub 上应用层故障注入只占 3.0%。【论文】结论：客户端混沌测试没有成套工具。

## 5 没有遥测时怎么发现长尾

- **Apple 崩溃报告（预判已证伪）**：Xcode Organizer 只收 TestFlight 或 App Store 分发的 app 的崩溃报告，metrics 只收 App Store 分发的。https://help.apple.com/xcode/mac/current/en.lproj/dev9a80ab71d.html 【官方】。Mac 版 TestFlight 要走 App Store Connect 并开沙盒。https://developer.apple.com/forums/thread/766796 【二手】。HAL 驱动装在 `/Library/Audio/Plug-Ins/HAL`，进不了沙盒 app → **Developer ID 分发拿不到 Apple 崩溃报告**【推算】。Apple 给的替代办法是用户在 Console.app 里找到崩溃报告，自己发给开发者。https://developer.apple.com/documentation/xcode/acquiring-crash-reports-and-diagnostic-logs 【官方】
- **MetricKit**：数据**直接在本机交给 app**，文档里没写分发渠道要求。macOS 12 起诊断（崩溃 / 卡死）立即送达；按日的 metrics 要 macOS 26 起；`MXMetricManager` 在 27.2 标为弃用，改用 `MetricManager`。https://developer.apple.com/documentation/metrickit/mxmetricmanager 【官方】。它能不能覆盖 coreaudiod 里的驱动进程：**没找到**，估计只覆盖 app 自己的进程【推算】。
- **诊断包先例**：Firefox 的 about:support 复制时自动去掉 profile 路径。https://support.mozilla.org/kb/Using+the+Troubleshooting+Information+page 【官方】。OBS 用 Help→Log Files→Upload，上传后得到一个可分享的链接；保留期限**没找到**。https://obsproject.com/forum/threads/please-post-a-log-with-your-issue-heres-how.23074/ 【二手】。GitHub issue 表单里可以用 `render: shell` 的 textarea 贴日志，还支持 `upload` 字段。https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms 【官方】
- 「确定性回放 / beta 渠道 / 设备矩阵」在音频行业的公开先例：**没找到**专门的文章。

## 对我们的含义

- **放在哪条回路**：模糊测试、性质测试、RTSan、pluginval 式矩阵、浸泡测试、混沌测试都只在 CI 和开发机上跑，**不进产品**。产品里只留**反射回路**的几个计数器：xrun 次数、最坏帧耗时、NaN 钳位次数、各分位数。音频线程只写预先分配好的原子计数，由**判断回路**每 0.5 s 读一次，写进本机环形日志。**学习回路**用的「影子比对」复用同一套不变量当护栏。
- **预算量级**【推算】：每个 PR 每个模糊目标跑 5 分钟，约 6 个目标 ≈ 30 CPU·分钟；每晚 1 小时；浸泡分 8 h（每晚）/ 72 h（发版前）两档。验收线：RSS 斜率 ≈ 0、端到端延迟漂移 < 1 ms、xrun = 0。计数器开销是每帧几次原子加法，相对 0.62 ms / 16 ms 可以忽略。
- **护栏**：诊断包**只在本机生成，先给用户预览、用户自己导出**，不带音频，路径脱敏；产品里没有任何上传代码。
- **首选**：`proptest` 不变量（有限值、有界、静音进静音出、延迟样本数恒定、改参数不炸）+ `cargo-fuzz` 覆盖模型输入和配置 + `rtsan-standalone` 守住音频回调 + 自写的 HAL 混沌脚本（pmset、stress-ng、采样率切换、杀 coreaudiod）。
- **备选**：用 `assert_no_alloc` 替代 RTSan；用 PESQ 替代 POLQA（PESQ 有开源实现，不受授权限制）。
- **三个风险**：① Developer ID 分发没有 Apple 崩溃渠道，驱动进程的崩溃可能谁都看不见；② 设备拔插只能自己造夹具，覆盖不到真实 USB / 蓝牙的边角情况；③ RTSan 和 rtcheck 管不到 `unsafe` FFI 或 CoreML 内部的阻塞调用，要把这些挪出音频线程，或者配一份抑制清单再单独审。
