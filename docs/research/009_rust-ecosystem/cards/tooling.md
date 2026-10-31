# 009 · 工具链组 · 项目卡

> 2026-10-30 · 研究编号 009 · 模块组：工具链（许可证 / 测试 / 基准 / 实时安全检测 / 打包签名 / 依赖卫生）
> 路径约定：【源码】的路径相对 `/Users/allenchen/xingan-chen/B-MyCreate/5-App/noise_canceling`。【API】= GitHub / crates.io API 当天返回（2026-10-30），或用 `gh api .../contents` 远程读到的文件（路径相对该仓）。
> 「哪条规则用哪个工具」归 rustarch 线，这里只写项目卡与推荐。

## 本组覆盖

**深读（读了源码）**：cargo-deny · cargo-dist · loom · assert_no_alloc · rtsan-standalone-rs · approx（另：rtrb 的 Miri CI 作参照）
**半深读（API 远程读了关键文件）**：cargo-packager · cargo-bundle · divan · gungraun（原 iai-callgrind）
**浅读（只 API 与官方页）**：cargo-about · cargo-license · cargo-nextest · criterion · cargo-auditable · cargo-audit(rustsec) · cargo-machete · cargo-udeps · cargo-semver-checks · miri · proptest · insta · float-cmp · apple-codesign(rcodesign) · cargo-llvm-cov

**一条总前提**：除 assert_no_alloc / rtsan-standalone 外，本组全是**开发期工具或 dev-dependency**，不进发布的二进制，所以它们的许可证不影响我们 GPL-3.0 产物；但它们本身全是 MIT/Apache/BSD 类，即便进了也兼容。【推断】

## 推荐（每个子需求一行）

| 子需求 | 首选 | 备选 | 理由 |
|---|---|---|---|
| 依赖树许可证白名单（命门 1） | **cargo-deny** `licenses` | cargo-about 的 `accepted`（同样会拒） | 默认全拒、只放白名单；对 `A OR B` 表达式逐项求值；支持按 crate 的 exceptions、忽略自家私有 crate、限定 `targets` 只看 aarch64-apple-darwin |
| GPL 发布用第三方许可清单 | **cargo-about**（模板出 HTML/TXT） | cargo-license（只列表，不带全文，15 个月无提交） | GPL 发布要随附各依赖的许可全文；cargo-about 与 cargo-deny 同一家同一套 spdx 识别 |
| 漏洞通告 / 被撤版本 | **cargo-deny** `advisories` | cargo-audit | 一个工具一份配置管四件事；cargo-audit 维护活跃，可作交叉 |
| 禁用 crate / 重复版本 / 构建脚本审查 | **cargo-deny** `bans` | — | `deny` + `wrappers`、`allow-build-scripts` 白名单，能把「谁能拉进来 ort 这类带下载脚本的 crate」写死 |
| 热路径零分配**断言**（命门 2，单测期） | **assert_no_alloc 的思路自己写**（~60 行，在 dsp/engine 的测试里装计数分配器） | divan `AllocProfiler`（基准里顺带数分配次数） | 原 crate 5 年多没发版（1.1.2 · 2021-08-03）；思路极简：线程局部计数 + 自定义全局分配器 |
| 热路径零锁零系统调用**检测**（命门 2，集成期） | **rtsan-standalone**（RTSan，macOS arm64 支持） | 无对等替代；退而求其次用 assert_no_alloc 只抓分配 | 唯一能同时抓 malloc / mutex / 系统调用的；平时零开销 no-op，`RTSAN_ENABLE=1` 才生效 |
| 环形缓冲 / 原子发布的并发正确性 | **loom**（算法层，线程模型） | Miri（UB + 数据竞争，rtrb 已在 CI 用） | 两者互补：loom 穷举交错，Miri 抓未定义行为；二者都**测不了真正跨进程 mmap**，需把算法与共享内存映射拆开 |
| 跑测试 | **cargo-nextest** | cargo test | 每测试一进程、超时与重试、JUnit 输出；活跃（今天还有提交） |
| 逐值对拍 Python（fbank / 降噪输出） | **approx 的思路自己写**一个对拍函数（返回最大误差位置与统计） | approx 直接用（`assert_relative_eq!`） | approx 的切片比较只回 bool，对拍失败时说不出是第几帧第几个系数 |
| 离线 WAV 结果的回归 | **insta**（快照存**摘要指标**：帧数、RMS、SNR、哈希、误差统计） | 自写 golden 文件 + approx | 原始采样点不适合进快照（大、浮点噪声）；insta 管审阅/更新流程好用 |
| 性质测试（环形缓冲、重采样边界） | **proptest** | quickcheck（未查） | 活跃；有 state-machine 子 crate，适合「读写序列」类测试 |
| 墙钟基准（每帧 μs、≤45 ms 预算） | **criterion** | divan | criterion 已迁到 criterion-rs 组织、2026-02 发版；divan 维护者 2026-07 才回归，近 18 个月未发版 |
| 指令数基准（CI 稳定回归门） | **gungraun**（原 iai-callgrind）跑在 **Linux** runner | 无 | Valgrind 不支持 macOS arm64；只能在 ubuntu runner 上对纯 Rust 的 dsp 测相对回归 |
| macOS 打包 + 签名 + 公证 | **自写脚本**：`codesign --options runtime` → `pkgbuild`/`productbuild` → `productsign` → `notarytool` → `stapler` | cargo-packager（.app/.dmg 签名+公证齐全，但不出 .pkg） | 我们要装 HAL 驱动（.driver 进 /Library/Audio/Plug-Ins/HAL），只有 .pkg 能装到系统目录；cargo-dist 不签 .pkg 也不公证 |
| 二进制内嵌依赖清单 | **cargo-auditable** | — | 发布后可用 cargo-audit 直接扫二进制；macOS 链接器警告问题已关 |
| 未用依赖 | **cargo-machete**（stable、快） | cargo-udeps（需 nightly，更准） | CI 里 machete 每次跑，udeps 偶尔手动 |
| 公开 API 破坏检查 | **不要**（我们不发库到 crates.io） | cargo-semver-checks | 内部 crate 之间用 workspace 路径依赖，semver 无意义 |

## 本组最值得学的设计

1. **rtsan 的「平时零开销、打开才拦截」开关**：构建脚本按 target 白名单 + 环境变量 `RTSAN_ENABLE` 决定是否设 `cfg(rtsan_enabled)`（白名单在 `resources/rtsan-standalone-rs/build.rs:10-17`，环境变量判断在 `:27-30`），API 函数体全包在 `#[cfg(rtsan_enabled)]` 里（`resources/rtsan-standalone-rs/src/lib.rs:32-37`），宏只在函数开头插一个 RAII 守卫（`resources/rtsan-standalone-rs/crates/rtsan-standalone-macros/src/lib.rs:33`）。→ 我们 engine 的 `process_frame` 和送音进程的 IO 回调可以**永久**标 `#[nonblocking]`，发布版零成本，CI 开一个 job 带 `RTSAN_ENABLE=1` 跑离线 WAV 评测，等于每次提交都验一遍命门 2。
2. **cargo-deny 许可证求值 = 对 SPDX 表达式的每个叶子先查 per-crate exception 再查全局白名单**（`resources/cargo-deny/src/licenses.rs:74-97`）。→ 我们只需维护一张「GPL-3.0 兼容」白名单，`MIT OR Apache-2.0` 这类双许可自动过；个别只在一处出现的许可（如某 crate 的 `Unicode-3.0`）放 exceptions，不污染全局。

---

### cargo-deny（https://github.com/EmbarkStudios/cargo-deny）· 深读
| 项 | 内容 |
|---|---|
| 做什么 | 对 Cargo 依赖图做四类检查：licenses / bans / advisories / sources |
| 许可证 | `MIT OR Apache-2.0`【API crates.io】；兼容：是（宽松许可）；开发工具不进产物 |
| 维护状态 | 0.20.2（2026-07-09）；最近提交 2026-10-26；open issues 85；最近 4 个 issue 维护者未回复，但 PR 合入很勤（#911–#915 一周内）【API】 |
| 实时安全做法 | 不适用（构建期工具） |
| 值得学的一处 | ① 许可证求值：表达式每个叶子先查 exceptions 再查 allow，都不中则拒（`resources/cargo-deny/src/licenses.rs:74-97`）。② 自 0.18.4 起 GNU 许可**按字面匹配**：允许 `GPL-3.0-or-later` 不等于允许 `GPL-3.0-only`，要逐个写（`resources/cargo-deny/docs/src/checks/licenses/cfg.md:79-83`）。③ `private = { ignore = true }` 跳过自家 `publish = false` 的 crate（同文件 :163-183）。④ `targets` 只看实际构建目标，非目标平台专属依赖被剔除（`resources/cargo-deny/docs/src/checks/cfg.md:50`）。⑤ `bans.build.allow-build-scripts` 置空列表即「谁都不许有构建脚本」，再逐个放行（`resources/cargo-deny/docs/src/checks/bans/cfg.md:301`）——ort 的构建脚本会下载 ONNX Runtime，可用它把这件事显式化 |
| 我们怎么用 | **直接用**（CI 工具，workspace 根放 `deny.toml`）。白名单草案【推断，需人核】：MIT、Apache-2.0、Apache-2.0 WITH LLVM-exception、BSD-2-Clause、BSD-3-Clause、BSD-1-Clause、ISC、Zlib、0BSD、CC0-1.0、Unlicense、BSL-1.0、Unicode-3.0、MPL-2.0、LGPL-2.1-or-later、LGPL-3.0-or-later、GPL-2.0-or-later、GPL-3.0-or-later、GPL-3.0-only；**不放** GPL-2.0-only、EPL、CDDL、AGPL 之外的强 copyleft 变体 |
| 风险 | ① 只看 Cargo 元数据与 LICENSE 文件，**看不到构建脚本下载的原生库**（ONNX Runtime dylib/静态库、模型权重），要在 cargo-about 清单里手工补【推断】。② 官方 GitHub Action 是 Docker 动作（action.yml `using: "docker"`【API EmbarkStudios/cargo-deny-action】），**macOS runner 跑不了**；放 ubuntu runner 并在 `targets` 写 `aarch64-apple-darwin` 即可，或在 macOS 上装预编译二进制。③ 「兼容」语义靠我们写对白名单，工具不懂 GPL 兼容性 |
| 证据 | 【源码】licenses.rs、docs/*.md、CI 的 Test job 在 `[ubuntu-24.04, macos-14]` 上跑（`resources/cargo-deny/.github/workflows/ci.yaml:61`），Release job 出 aarch64-apple-darwin 二进制（`:168-170`）（复核 3 更正：原引的 :170 是 release 矩阵，不是测试）·【API】版本/issue/action.yml ·【推断】白名单、原生库盲区 |

### cargo-about（https://github.com/EmbarkStudios/cargo-about）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 按模板（handlebars）生成第三方许可清单（含全文） |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 0.9.3（2026-10-26）；提交 2026-10-26；open 30；最近 4 issue 无维护者回复【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | 未读源码 |
| 我们怎么用 | **直接用**：发布流程生成 `THIRD_PARTY_LICENSES.html` 放进 .pkg 与 About 窗口；ONNX Runtime、模型权重的许可手工追加 |
| 风险 | 原生库与模型不在 Cargo 图里，必须手补；与 cargo-deny 两份配置要保持一致 |
| 证据 | 【API】 |

### cargo-license（https://github.com/onur/cargo-license）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 列出依赖及其许可证（表格/JSON） |
| 许可证 | MIT【API】；兼容：是 |
| 维护状态 | 0.7.0（2025-07-29）；最近提交 2025-07-30（**超过 12 个月**）；open 9【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（cargo-about 覆盖且带全文） |
| 风险 | 维护停滞 |
| 证据 | 【API】 |

### cargo-dist（https://github.com/axodotdev/cargo-dist）· 深读
| 项 | 内容 |
|---|---|
| 做什么 | 生成发布 CI 与安装包（shell/brew/msi/macOS .pkg 等） |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | GitHub v0.33.1（2026-10-21，修一个被吊销 CA 的信任问题，旧二进制网络认证失效，见 issue #2550）；crates.io 上 `cargo-dist` 仍停在 0.32.0（2026-05-22）；open 354【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | **CI 里的临时钥匙串**：建临时 keychain → 导入 p12 → `set-key-partition-list` → codesign → 随 TempDir 销毁（`resources/cargo-dist/cargo-dist/src/sign/macos.rs:14-19`，签名在 :200-207）。我们自写签名脚本照这个流程做，避免证书留在 runner 上 |
| 我们怎么用 | **只看看**。不用它的原因：① macOS 只签**可执行文件**，不签 bundle（`resources/cargo-dist/cargo-dist/src/sign/mod.rs:83-85`），我们的 HAL 驱动是 bundle；② **不公证**（`.../sign/macos.rs:21`「In the future… notarization」）；③ .pkg 只调 `pkgbuild`+`productbuild`，不调 `productsign`（`resources/cargo-dist/cargo-dist/src/backend/installer/macpkg.rs:78`、`:89`）→ 未签名 .pkg 在 Gatekeeper 下装不了 |
| 风险 | 供应链：用预编译版要跟住安全修复版本 |
| 证据 | 【源码】sign/*.rs、macpkg.rs ·【API】版本、issue #2550 |

### cargo-packager（https://github.com/crabnebula-dev/cargo-packager）· 半深读
| 项 | 内容 |
|---|---|
| 做什么 | 打 .app / .dmg / deb / AppImage / NSIS / WiX 等，含签名 |
| 许可证 | `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | 0.11.8（2025-11-27）；提交 2026-10-22；open 43【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | macOS 签名带 hardened runtime 与 entitlements，再走 `notarytool` 提交、`stapler staple`（远程读 `crates/packager/src/codesign/macos.rs:232-239`、`:311`、`:373`；凭据从 `APPLE_ID` / `APPLE_API_KEY` 环境变量读，`:426-441`）【API】 |
| 我们怎么用 | **备选**：给 control（菜单栏 App）出 .app/.dmg 时可直接用；**不出 .pkg**（包类型目录里没有 pkg，`crates/packager/src/package/` 列表【API】；官方页也只列 .dmg/.app【官方 https://docs.crabnebula.dev/packager/】）→ HAL 驱动仍要 .pkg |
| 风险 | 0.11.8 后 11 个月未发版 |
| 证据 | 【API】远程文件 ·【官方】 |

### cargo-bundle（https://github.com/burtonageo/cargo-bundle）· 半深读
| 项 | 内容 |
|---|---|
| 做什么 | 把 Rust 二进制打成 .app / .dmg / msi / deb 等 |
| 许可证 | crates.io `MIT OR Apache-2.0`，GitHub 显示 NOASSERTION【API】；兼容：是 |
| 维护状态 | 0.12.0（2026-09-20）；open 74；最近 3 issue 仅 1 条有回复（#249 编译失败）【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | 用纯 Rust 的 `apple-codesign` 签 bundle / DMG，可选 hardened runtime（远程读 `src/bundle/signing.rs:7-8`、`:49`）【API】 |
| 我们怎么用 | **不要**（无公证、无 .pkg，最新版有编译失败报告） |
| 风险 | 维护者响应慢 |
| 证据 | 【API】 |

### apple-codesign / rcodesign（https://github.com/indygreg/apple-platform-rs）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 纯 Rust 实现的 Apple 代码签名与公证（可在 Linux 上签 macOS 产物） |
| 许可证 | MPL-2.0【API】；兼容：是（MPL-2.0 §3.3 允许与 GPL 组合）；且只是构建工具 |
| 维护状态 | 0.29.0（2024-11-29）；提交 2026-08-09；open 120；近期 issue 多为崩溃/编译失败报告且无回复（#362 「0.29.0 cannot compile」）【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | 未读 |
| 我们怎么用 | **只看看**。我们 runner 就是 macOS，直接用 Apple 自带 `codesign`/`productsign`/`notarytool` 更稳 |
| 风险 | 发版停在 2024-11 |
| 证据 | 【API】 |

### loom（https://github.com/tokio-rs/loom）· 深读
| 项 | 内容 |
|---|---|
| 做什么 | 并发模型检查：在 C11 内存模型下穷举线程交错，测原子/锁代码 |
| 许可证 | MIT【API】；兼容：是（dev-dependency） |
| 维护状态 | 0.7.2（**2024-04-23**，两年半未发版）；最近提交 2026-02-20（12 个月内）；open 130；最近 5 个 issue 全无回复【API】 |
| 实时安全做法 | 不适用（测试工具）；但它就是用来验证我们无锁结构的 |
| 值得学的一处 | **sync 门面模块**：`#[cfg(loom)] use loom::sync::atomic::…` / `#[cfg(not(loom))] use std::…`，业务代码只从自家 `sync` 模块取原子类型（`resources/loom/src/lib.rs:120-131`）。→ dsp 的环形缓冲与 engine 的「原子发布声纹结果」都从一个 `sync.rs` 取类型，`RUSTFLAGS="--cfg loom"` 时整体换成 loom 版。抢占上界用 `LOOM_MAX_PREEMPTIONS` 控制爆炸（`resources/loom/src/model.rs:41-42`、`:108`） |
| 我们怎么用 | **直接用**（dev-dependency，dsp / engine 的并发单测） |
| 风险 | ① 不完整：`SeqCst` 当 `AcqRel` 处理（会误报），**不探索 load buffering**（会漏报）（`resources/loom/README.md:68-79`）。② 模拟线程不模拟进程：**送音进程↔推理进程的 mmap 共享内存本身测不了**，要把「环形缓冲算法」写成对任意 `&[AtomicU32]`/裸指针都成立的结构，用堆内存喂 loom，mmap 那层另测【推断】。③ 维护停滞，issue 无人回 |
| 证据 | 【源码】README、lib.rs、model.rs ·【API】·【推断】跨进程盲区 |

### Miri（https://github.com/rust-lang/miri）· 浅读 + 参照 rtrb
| 项 | 内容 |
|---|---|
| 做什么 | Rust MIR 解释器，检测未定义行为与数据竞争 |
| 许可证 | `MIT OR Apache-2.0`【推断，GitHub 显示 Apache-2.0】；rustup 组件，不进产物 |
| 维护状态 | 最近提交 2026-10-30；open 195【API】；随 nightly 发布 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | rtrb 的 CI 跑三遍 Miri：默认、`-Zmiri-preemption-rate=0`、`-Zmiri-tree-borrows`（`resources/rtrb/.github/workflows/main.yml:105-130`），且跑在 ubuntu 上。→ 我们的 unsafe（共享内存视图、环形缓冲）照抄这三遍组合 |
| 我们怎么用 | **直接用**（CI nightly job，ubuntu runner 即可，测的是平台无关的 unsafe） |
| 风险 | 不支持 FFI：ONNX Runtime、CoreAudio、`shm_open`/`mmap` 相关测试要 `#[cfg_attr(miri, ignore)]`【推断】；需 nightly |
| 证据 | 【源码】rtrb CI ·【API】·【推断】 |

### assert_no_alloc（https://github.com/Windfisch/rust-assert-no-alloc）· 深读
| 项 | 内容 |
|---|---|
| 做什么 | 自定义全局分配器：在闭包内一旦分配/释放就 abort 或计数告警 |
| 许可证 | BSD-1-Clause（`resources/rust-assert-no-alloc/Cargo.toml:6`；GitHub 显示 NOASSERTION）；兼容：是（宽松许可） |
| 维护状态 | 1.1.2（**2021-08-03**）；最近提交 2022-09（**超过 12 个月**）；issue #17「New crates.io release?」（2024-03）无维护者回复【API】 |
| 实时安全做法 | 线程局部计数器 `ALLOC_FORBID_COUNT`（`resources/rust-assert-no-alloc/src/lib.rs:35-36`）；RAII 守卫进出 +1/-1，panic 展开也能复原（`:60-75`）；分配器 `alloc`/`dealloc` 先 `check` 再转给 `System`（`:180-189`），违规时 `handle_alloc_error`（`:151-176`）。默认特性 `disable_release` 让 release 版变 no-op（`resources/rust-assert-no-alloc/Cargo.toml:15`；复核 3 把原写的 :16 改成 :15） |
| 值得学的一处 | 整个机制约 60 行，「线程局部计数 + 全局分配器包一层」——自写一份放进 dsp 的测试工具模块，给 `process_frame` 的单测包一层 |
| 我们怎么用 | **抄思路自己写**（engine/dsp 的 `#[cfg(test)]`）。不直接依赖：不维护，且全局分配器一个进程只能有一个 |
| 风险 | 只抓分配，**抓不到锁和系统调用**；ONNX Runtime 走 C++ 自己的分配器，**Rust 全局分配器看不到 ORT 内部的 malloc**【推断】→ 推理线程要靠 rtsan |
| 证据 | 【源码】lib.rs、Cargo.toml ·【API】·【推断】ORT 盲区 |

### rtsan-standalone-rs（https://github.com/realtime-sanitizer/rtsan-standalone-rs）· 深读
| 项 | 内容 |
|---|---|
| 做什么 | LLVM RealtimeSanitizer 的独立版 Rust 包装：标了 `#[nonblocking]` 的函数里，拦截 malloc/锁/阻塞系统调用并报栈 |
| 许可证 | Apache-2.0（`resources/rtsan-standalone-rs/Cargo.toml:10`）；底层运行时库来自 LLVM compiler-rt（Apache-2.0 WITH LLVM-exception【推断】）；兼容：是 |
| 维护状态 | 0.3.0（2025-09-27）；**最近提交 2025-09-27（13 个月，刚超 12 个月线）**；open 1【API】。上游 RTSan 在 LLVM 主线里持续开发【推断】 |
| 实时安全做法 | 未启用时零开销：构建脚本只对 6 个 target 设 `rtsan_supported`，含 `aarch64-apple-darwin`（`resources/rtsan-standalone-rs/build.rs:10-17`），只有设了 `RTSAN_ENABLE` 才设 `rtsan_enabled`（`:27-30`）；`realtime_enter` 等函数体在 `#[cfg(rtsan_enabled)]` 下才调 FFI（`resources/rtsan-standalone-rs/src/lib.rs:32-37`）；宏在函数开头插 RAII 守卫（`resources/rtsan-standalone-rs/crates/rtsan-standalone-macros/src/lib.rs:33`）。macOS 链 `libclang_rt.rtsan_osx_dynamic.dylib`（`resources/rtsan-standalone-rs/crates/rtsan-standalone-sys/build.rs:23-24`） |
| 值得学的一处 | 见文首「最值得学」第 1 条：永久标注、按需开启 |
| 我们怎么用 | **直接依赖**（engine 的 `process_frame`、io-macos 的 IO 回调、dsp 的重采样/环形缓冲热路径标 `#[nonblocking]`）；CI 一条 macOS job 带 `RTSAN_ENABLE=1` 跑 eval 的离线 WAV。顺带能**量化 ONNX Runtime 在推理线程里到底分配/加锁多少**——这正是 D33 架构要知道的 |
| 风险 | ① 默认特性 `prebuilt-libs`（`resources/rtsan-standalone-rs/Cargo.toml:43`）用 `curl -L` 从 rtsan-libs 的 GitHub Release 下载预编译库，**无哈希校验**（`resources/rtsan-standalone-rs/crates/rtsan-standalone-sys/build.rs:78-95`）。只有设了 `RTSAN_ENABLE` 才会走到下载这一步，平时构建不会下载（`:40-44`）。对策：关掉 `prebuilt-libs`，用 **`RTSAN_LIB_PATH`**（复核 3 更正：原写的 `RTSAN_LIBRARY_PATH` 不对，见 `:54`）指向我们自己核过哈希的库。② 维护刚过 12 个月线。③ ORT 内部的违规会刷屏，需要 `ScopedDisabler`/`no_sanitize_realtime` 有意识地圈出来并登记 |
| 证据 | 【源码】build.rs、src/lib.rs、macros、sys/build.rs ·【API】·【推断】LLVM 许可、上游状态 |

### divan（https://github.com/nvzqz/divan）· 半深读
| 项 | 内容 |
|---|---|
| 做什么 | 微基准框架，带分配计数 |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 0.1.21（**2025-04-10**）；最近提交 2026-07-19；issue #91「Is Divan still maintained?」维护者 2026-07-19 才回复说会恢复【API】 |
| 实时安全做法 | `AllocProfiler<Alloc = System>` 包一层全局分配器统计每次迭代分配次数（远程读 `src/alloc.rs:125-130`）【API】 |
| 值得学的一处 | 基准里同时报「耗时 + 分配次数」，可作「每帧 0 次分配」的旁证 |
| 我们怎么用 | **备选**（criterion 为首选） |
| 风险 | 维护不确定 |
| 证据 | 【API】 |

### criterion（https://github.com/criterion-rs/criterion.rs）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 统计型墙钟基准 |
| 许可证 | `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | 0.8.2（2026-02-04）；仓库已从 bheisler 迁到 criterion-rs 组织；最近提交 2026-10-02；最近 issue 无回复【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | rtrb 用 criterion 0.8 做 push/pop 基准（`resources/rtrb/Cargo.toml:30`、`resources/rtrb/benches/push_pop.rs:8`） |
| 我们怎么用 | **直接用**：dsp/engine 每帧耗时基准，本机 M 系列跑，**不在 GitHub macOS runner 上当回归门**（共享虚机噪声大）【推断】 |
| 风险 | CI 墙钟不稳 |
| 证据 | 【源码】rtrb ·【API】 |

### gungraun（原 iai-callgrind，https://github.com/gungraun/gungraun）· 半深读
| 项 | 内容 |
|---|---|
| 做什么 | 基于 Valgrind（Callgrind/Cachegrind/DHAT）与 Linux perf 的指令数基准 |
| 许可证 | `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | 已改名：`iai-callgrind` 最后 0.16.1（2025-07-30），新名 `gungraun` 0.20.0（2026-09-26）；open 29【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | DHAT 模式能精确数堆分配（README 提到 DHAT 已集成，`resources/gungraun/README.md:82-86`） |
| 我们怎么用 | **只在 Linux runner 上用**：对纯 Rust 的 dsp（fbank、重采样）做指令数回归门。**macOS 不可用**：README 写明不支持 Valgrind 与 perf 都没有的目标（`resources/gungraun/README.md:97-98`），Valgrind 官方平台表不含 arm64 macOS（【官方】https://valgrind.org/info/platforms.html） |
| 风险 | Linux 指令数 ≠ Apple Silicon 实际耗时，只作相对回归 |
| 证据 | 【源码】README ·【官方】·【API】 |

### cargo-nextest（https://github.com/nextest-rs/nextest）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 替代 `cargo test` 的测试运行器（每测试一进程、超时、重试、分片、JUnit） |
| 许可证 | `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | 0.9.150（2026-10-26）；提交 2026-10-30；open 105【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | 每测试一进程：装了自定义全局分配器 / 改了线程优先级的测试互不污染【推断】 |
| 我们怎么用 | **直接用**（CI 与本地）；doctest 仍需 `cargo test --doc` 另跑【推断】 |
| 风险 | 低 |
| 证据 | 【API】·【推断】 |

### proptest（https://github.com/proptest-rs/proptest）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 性质测试 + 自动缩小反例 |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 1.12.0（2026-10-10，crates.io；GitHub Releases 页停在 2020，不用它）；提交 2026-10-10；open 152【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | 未读；有 proptest-state-machine（issue #671 提到）适合「随机读写序列 vs 参考 VecDeque」 |
| 我们怎么用 | **直接用**（dsp：环形缓冲、重采样任意块长、fbank 任意输入长度） |
| 风险 | 低 |
| 证据 | 【API】 |

### insta（https://github.com/mitsuhiko/insta）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 快照测试 + `cargo insta review` 审阅更新 |
| 许可证 | Apache-2.0【API】；兼容：是（GPL-3.0 兼容 Apache-2.0） |
| 维护状态 | 1.49.0（2026-10-03）；提交 2026-10-26；open 77【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | 未读 |
| 我们怎么用 | **直接用**（eval）：快照存每个测试 WAV 的**摘要**（帧数、RMS、与 Python 的最大误差、SNR、输出哈希），浮点先四舍五入到固定位数；原始采样点不进快照 |
| 风险 | 浮点在不同 CPU/编译选项下末位抖动 → 快照只存舍入后的指标 |
| 证据 | 【API】·【推断】 |

### approx（https://github.com/brendanzab/approx）· 深读
| 项 | 内容 |
|---|---|
| 做什么 | 浮点近似相等：绝对差 / 相对差 / ULPs |
| 许可证 | Apache-2.0（`resources/approx/approx/Cargo.toml:5`）；兼容：是 |
| 维护状态 | crates.io 最新稳定版 0.5.1 发布于 **2022-01-23**；2026-02-05 是 crate 的 `updated_at`，也就是 0.6.0-rc2 预发布版上 crates.io 的日期（rc1 已被 yank）【API】（复核 3 更正）；提交 2026-07-09；open 15；issue #108「0.6.0 Release」无回复【API】 |
| 实时安全做法 | 纯计算无分配 |
| 值得学的一处 | `relative_eq` 先判相等/无穷，再「绝对差 ≤ epsilon 即过（接近 0 时）」，否则「≤ 较大绝对值 × max_relative」（`resources/approx/approx/src/relative_eq.rs:86-115`）——这正是 fbank 对拍要的混合判据（log 能量接近 0 处用绝对容差）。但切片比较只返回 bool（`:278-282`），说不出第几帧第几维挂了 |
| 我们怎么用 | **抄思路自己写**：eval 里一个 `compare(rust, python, eps_abs, eps_rel) -> Report{max_abs, max_rel, argmax(frame,bin), n_fail}`；断言简单场合可直接用 approx 宏 |
| 风险 | 0.6 发版悬而未决 |
| 证据 | 【源码】relative_eq.rs、Cargo.toml ·【API】 |

### float-cmp（https://github.com/mikedilger/float-cmp）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | ULPs/epsilon 浮点比较 |
| 许可证 | MIT【API】；兼容：是 |
| 维护状态 | 0.10.0（2024-09-20）；提交 2026-05-02；open 5【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（与 approx 重叠，选一个） |
| 风险 | 发版稀 |
| 证据 | 【API】 |

### cargo-auditable（https://github.com/rust-secure-code/cargo-auditable）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 把依赖清单压缩嵌进二进制的一个链接段，事后可扫 |
| 许可证 | `MIT OR Apache-2.0`【API】；兼容：是（运行期不引入代码，只嵌数据）【推断】 |
| 维护状态 | 0.7.7（2026-10-02）；提交 2026-10-21；open 17；最近 issue 有回复且多已关，含 #266「macOS 链接器警告」已关【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **直接用**（发布构建用 `cargo auditable build --release`） |
| 风险 | 需核 HAL 驱动（被 coreaudiod 加载的 dylib）里嵌段是否影响签名/加载【推断，未查】 |
| 证据 | 【API】 |

### cargo-audit / rustsec（https://github.com/rustsec/rustsec）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 按 RustSec 通告库扫 Cargo.lock 或可审计二进制 |
| 许可证 | crates.io `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | cargo-audit 0.22.2（2026-06-05）；提交 2026-10-29；open 122【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **备选**（cargo-deny advisories 为首选，数据源相同）；扫已发布二进制时用它 |
| 风险 | 低 |
| 证据 | 【API】 |

### cargo-machete（https://github.com/bnjbvr/cargo-machete）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 文本扫描找未用依赖（stable、快、可能误报） |
| 许可证 | MIT【API】；兼容：是 |
| 维护状态 | 0.9.2（2026-04-15）；提交 2026-10-19；open 32【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **直接用**（CI 每次） |
| 风险 | 宏引入的依赖会误报，需 `[package.metadata.cargo-machete] ignored` |
| 证据 | 【API】 |

### cargo-udeps（https://github.com/est31/cargo-udeps）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 基于编译结果找未用依赖（准，但要 nightly） |
| 许可证 | crates.io `MIT OR Apache-2.0`【API】；兼容：是 |
| 维护状态 | 0.1.61（2026-04-29）；提交 2026-10-22；最近 5 issue 仅 1 条有回复（#337 stable 装不上）【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **备选**（偶尔手动） |
| 风险 | 依赖 nightly，安装易坏 |
| 证据 | 【API】 |

### cargo-semver-checks（https://github.com/obi1kenobi/cargo-semver-checks）· 浅读
| 项 | 内容 |
|---|---|
| 做什么 | 检查公开 API 的 semver 破坏 |
| 许可证 | `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | 0.51.0（2026-10-03）；提交 2026-10-26；open 194，维护者回复活跃（#1776 6 条）【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **不要**（不对外发库）。若将来把 dsp 单独发 crates.io 再启用 |
| 风险 | — |
| 证据 | 【API】 |

### cargo-llvm-cov（https://github.com/taiki-e/cargo-llvm-cov）· 浅读（任务外补充）
| 项 | 内容 |
|---|---|
| 做什么 | 基于 LLVM 插桩的覆盖率 |
| 许可证 | `Apache-2.0 OR MIT`【API】；兼容：是 |
| 维护状态 | 0.9.1（2026-09-06）；提交 2026-10-08；最近 issue 有回复【API】 |
| 实时安全做法 | 不适用 |
| 值得学的一处 | — |
| 我们怎么用 | **只看看**（可选：看 dsp 的边界分支有没有被性质测试覆盖） |
| 风险 | — |
| 证据 | 【API】 |

---

## 没查到 / 存疑

1. **Rust 编译器原生 `-Zsanitizer=realtime`**：是否已进 nightly、能否替代 rtsan-standalone，未查。【存疑】
2. **ONNX Runtime 在推理线程里的分配/加锁**：assert_no_alloc 看不到 C++ 侧 malloc 是推断；rtsan 能拦到（它拦的是 libc 层）也是推断，需一次有界小实验（要重活名额）。
3. **cargo-deny 白名单**：上面列的「GPL-3.0 兼容」许可清单是我按 FSF 兼容表的记忆写的，未逐条对 FSF 原页核对；`GPL-3.0-only` vs `-or-later` 我们自己选哪个也要人定（D04 只写 GPL-3.0）。
4. **Miri 对 `mmap`/`shm_open` 的支持程度**未核；当前假设「不支持，相关测试在 Miri 下跳过」。
5. **cargo-auditable 嵌段对 HAL 驱动签名/加载的影响**未查。
6. **cargo-nextest 在 macOS 上的已知问题**、**insta / proptest 源码**未读（浅读）。
7. **HAL 驱动 .pkg 的签名公证全流程**没有现成 Rust 工具覆盖：cargo-dist（不签 pkg、不公证）、cargo-packager（不出 pkg）、cargo-bundle（不公证）都不够，结论是自写脚本；BlackHole / BackgroundMusic（已在 resources/）的发布脚本可能是更好的参照，本组未读。
8. GitHub macOS runner 上墙钟基准的噪声大小是推断，未测。

## 复核记录（怀疑派 3）

复核日 2026-10-30。只读源码、GitHub API 和 crates.io API；没克隆新仓，没构建。

**查了多少**：本地【源码】行号约 40 处，逐行打开看过（cargo-deny 的 licenses.rs 和 3 份 docs、CI；cargo-dist 的 sign/macos.rs、sign/mod.rs、macpkg.rs；loom 的 lib.rs、model.rs、README；rtrb 的 CI、Cargo.toml、benches；assert_no_alloc 的 lib.rs、Cargo.toml；rtsan 的 build.rs、src/lib.rs、macros、sys/build.rs；gungraun 的 README；approx 的 relative_eq.rs、Cargo.toml）。GitHub API 复查了 23 个仓，crates.io 复查了 23 个 crate，另外核了 cargo-deny-action 的 action.yml、cargo-dist 和 rtsan 的 releases、loom 和 divan 的 commits。远程读的那些文件（cargo-packager、cargo-bundle、divan）**没有复核行号**。

**改了哪些**：
1. cargo-deny 的 CI 引用：原引 `ci.yaml:170` 是 release 矩阵；测试矩阵实际在 `:61`，已改。
2. assert_no_alloc 的 `disable_release` 默认特性在 `Cargo.toml:15`，不是 :16，已改。
3. **rtsan 的环境变量名写错了**：应该是 `RTSAN_LIB_PATH`（`sys/build.rs:54`），不是 `RTSAN_LIBRARY_PATH`。另外补上：只有设了 `RTSAN_ENABLE` 才会下载（`:40-44`），平时构建不联网。
4. **approx 的日期写错了**：0.5.1 发布于 2022-01-23；2026-02-05 是 0.6.0-rc2 预发布的日期，已改。
5. assert_no_alloc 原写「4 年没发版」，实际 5 年多；cargo-license 原写「14 个月」，实际 15 个月，已改。

**查对了的（没改）**：
- cargo-deny 的许可证求值顺序（先 exceptions、再 allow、否则 deny）、GNU 许可证按字面匹配、`private.ignore`、`targets`、`allow-build-scripts`，都核过。
- cargo-deny-action 确实是 `using: "docker"`。
- cargo-dist 只签可执行文件，也不做公证。`productsign` 只出现在钥匙串 ACL 的 `-T` 参数里（`sign/macos.rs:108`），没有被调用，所以「.pkg 不签名」成立。
- loom 的 sync 门面、`LOOM_MAX_PREEMPTIONS`、README 列的不支持项，rtrb 跑三遍 Miri，rtsan 的 6 个 target 和 dylib 名，assert_no_alloc 的计数加 RAII 机制，都核过。
- 所有许可证与 API 一致，各版本日期也都对得上（cargo-dist v0.33.1 在 GitHub 上，crates.io 停在 0.32.0）。

**查不了的**：
- cargo-packager、cargo-bundle、divan 远程文件的行号（没在本地）。
- Rust nightly 有没有 `-Zsanitizer=realtime`（存疑 1）。
- ORT 内部分配能不能被 rtsan 拦到（要实验）。
- FSF 兼容表的逐条核对（存疑 3）。
- 「最近 N 个 issue 无回复」只抽查了 nih-plug（在 arch 卡），本卡的没有逐条复核。

**对推荐的异议**：
1. **rtsan-standalone 首选「直接依赖」违反命门 3**：默认分支最后一次提交是 2025-09-27，超过 12 个月（arch 卡原写的「推送 2026-09-15」是 `pushed_at`，不是提交）。另外它带一个默认会联网下载的构建特性。建议降为 `[dev-dependencies]`，或者做成一个关掉 `prebuilt-libs` 的可选 feature（`rtsan`），不进发布二进制的依赖树；`#[nonblocking]` 标注通过 `cfg_attr(feature = "rtsan", ...)` 挂上。要是嫌它停更，可以直接调 `__rtsan_realtime_enter/exit` 这两个 C 符号（几行 FFI），不必依赖这个 crate。
2. 白名单草案里 `Unicode-3.0` 既放进了全局 allow，「最值得学 2」又拿它举例说要放 exceptions，口径自相矛盾，人定的时候二选一。
3. 「热路径零分配断言」首选自写守卫，站得住。补一个限制：全局分配器一个进程只能有一个，跟 divan 的 `AllocProfiler` 不能同时装在同一个测试二进制里，基准和断言要分开放。
4. 「macOS 打包」结论成立（三个现成工具都不能覆盖 .pkg + 签名 + 公证）。没漏掉更好的 Rust 备选；本组也没读 BlackHole 和 BackgroundMusic 的发布脚本（存疑 7），建议 hal-plugin 线去读。
