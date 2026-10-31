# 两类来源 · Rust 生态：每个模块用哪个依赖、学谁的设计

> 2026-10-30 rusteco。逐项目的核查表在 `cards/`，本页只登记「从哪来、还算不算数」。

## 一、自有积累

| 来源 | 还算数吗 | 用在哪 |
|---|---|---|
| `docs/plan/003_tech-design.md`（两进程架构、engine 接口、离线实时同一份代码）| 算数（2026-10-30）| 每个模块的需求清单 |
| `docs/plan/001_decisions-20261029-b1.md` D03 / D04 / D31 / D32 / D33 | 算数 | 三条命门 |
| p1 的 exp004 实测（ort rc.15 跑 GTCRN 每帧 0 次分配、FE-M 每帧 7 次、CoreML 坑）| 算数，本机实测 | decision.md §一.4 |
| 003 号调研 `docs/research/003_mac-audio-pipeline/`（BlackHole、libASPL 等 C/C++ 对照）| 算数 | io 组对照 |

## 二、他人积累（全部只读借鉴，铁律 3）

本轮新浅克隆进主目录 `resources/`（单仓均 < 100 MB）：
- io：voice-strip · tympan-aspl · prism · duomic · opennc-app · cubeb-coreaudio-rs · memmap2-rs · m-audio-fast-track-ultra-macos-driver
- rtsafe：ringbuf · basedrop · triple-buffer · arc-swap · rust-assert-no-alloc · nice-assert-no-alloc · rtsan-standalone-rs · no_denormals · atomic_float · crossbeam · seqlock
- dsp：realfft · RustFFT · sonora · biquad-rs · ebur128 · accelerate-rs · dasp
- infer：ort · tract
- speaker：pyannote-rs · kaldi-native-fbank · kaldi-native-fbank-rs · kaldi-fbank · earshot · silero-findit · voice_activity_detector · speakrs · sherpa-onnx
- arch：nih-plug · Firewheel · clack · kira
- tooling：cargo-deny · loom · approx · gungraun · cargo-dist

此前已有：cpal · coreaudio-rs · rtrb · rubato · audio_thread_priority · nnnoiseless · biglinux-gtcrn · DeepFilterNet · webrtc-audio-processing · fastenhancer · wespeaker · 3D-Speaker。

只用 API / 官方文档、没克隆的：objc2 家族、iceoryx2、ipc-channel、shared_memory、pipewire-rs、candle、burn、wonnx、hound、symphonia、fundsp、knyst、glicol、rodio、oddio、Meadowlark、cargo-about、nextest、criterion、divan、cargo-packager、cargo-auditable、cargo-machete、proptest、insta 等（各卡标「浅读」）。

**今天还算数吗**：维护日期都是 2026-10-30 当日从 GitHub / crates.io API 取的；近 12 个月无发版的已在卡里标出（RustFFT、hound、no_denormals、basedrop、rtsan-standalone、assert_no_alloc、objc2-core-audio、divan、loom），归 decision.md 待人拍板 1。
