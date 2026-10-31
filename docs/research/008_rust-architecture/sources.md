# 来源登记 · 008 Rust 架构

> 2026-10-30。证据等级：【官】【源】【测】【传】【推】。逐条出处在 `notes/` 与规约正文里，这里只登记来源本身、还算不算数。

## 一、自有积累

| 来源 | 用到什么 | 还算数吗 |
|---|---|---|
| 设计稿 `vault/raw/20261029-voice-denoise-design.md` §6 / §7 / §11 | 初拟 7 个 crate、关键设计 4 条、实时纪律、延迟预算 | 目标算数；crate 划分按本次调研改（ADR 001）；「插件在 coreaudiod 里」已被 003 更正 |
| `docs/plan/003_tech-design.md` | 两层进程、P1–P4、门控、音色链 | 算数 |
| `docs/research/003_mac-audio-pipeline/decision.md` | cpal · rtrb · rubato、原声旁路由输出回调做、rtrb 放不进共享内存、二期共享内存 | 算数 |
| `docs/plan/001_decisions-20261029-b1.md` | D03 送原声、D04 GPL-3.0、D32 ≤ 45 ms、D33 双模型 | 算数 |
| `.42cog/intent.md` · `.42cog/cog.md` | 三把尺子、作品区「同一份处理代码」「实时安全检查」、crate 住 `src/<crate>/` | 算数；「音频线程不分配」措辞待按 ADR 002 调（⟨待拍板 1⟩） |
| p1 骨架（分支 `feat/p1-offline-gtcrn`，`src/{dsp,engine,models,cli}`） | 现有 trait、Engine、三件保命、ort 调法、Rust 侧 0 分配实测 | 算数，规约从它长出来 |
| rusteco 009 `decision.md`、`cards/arch.md` | 逐项目选型、延迟账 56 ms、FE-M 7 进 7 出、p1 实测坑 | 算数（同批，已复核） |

## 二、他人积累

| 来源 | 读到哪一步 | 还算数吗 |
|---|---|---|
| matklad「Large Rust Workspaces」「Fast Rust Builds」「ARCHITECTURE.md」、rust-analyzer `docs/dev/architecture.md` 与 `xtask/src/tidy.rs` | 原文引文已复核（r7 C21、C22） | 算数（作者一手，非官方规范） |
| Cargo Book（workspace 继承、features、lints）、Rust API Guidelines、Clippy 配置文档 | 官方页 | 算数 |
| cargo-deny 文档与源码 `resources/cargo-deny/src/bans.rs` | 读到源码，确认 wrappers 只查直接依赖 | 算数 |
| rtsan-standalone（README、build.rs、macros）、Clang RTSan 文档、rustc PR #147935、Unstable Book | 源码 + 本机实测 | 算数；rustc 内置版仍 unstable |
| assert_no_alloc 1.1.2、no-panic、rtrb、rubato、cpal、coreaudio-rs、audio_thread_priority、ort 2.0.0-rc.15、tract | 源码行号 + 本机实测（见 `appendix-probes/`） | 算数；cpal 本机是未发布 master，钉 0.18.2 |
| nih-plug、firewheel、fundsp、tympan-aspl、DeepFilterNet libDF | trait 分层与实时 / 非实时分离的做法（源码行号在 r3 / r7） | 只读借鉴 |
| ADC 演讲：Eckes 2024（G3WWr4uhP0I）、Hobson 2025（YLPglu2enaE、hobson.dev/talks/adc_25）、Trevelyan & Apple 2024 RTSan（会议页）、Rowland 2024；Doumler 锁文章与演讲；CppCon 2024 RTSan（KvhgNdxX6Uw） | 14 个视频 ID 逐个 oEmbed 核过，无编造（r7 C16） | 算数 |
| RustConf / EuroRust / RustNation / RustWeek | 没找到 Rust 音频演讲 | —（如实记为未找到） |
| GNU 许可兼容列表、GitHub Actions 计费 | 官方页 | 算数 |

## 三、自己跑的（做法，不是来源）
本机探针 6 组，见 `appendix-probes/README.md`。⚠️ 构建前没申请名额（已向大总管交代）。
