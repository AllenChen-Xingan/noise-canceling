# 现状页 · 手下 rs-io（批次 20261102-b8-rust）

- 窗口：rs-io · 大总管 noise-canceling-47 · 分支 `feat/rust-skeleton-io`
- 管的实体：`vdn-io-macos`（新）· `vdn-hal`（新）

## 已收尾（2026-11-02）
- 两个 crate 已写完并自审修完。实测：`cargo test -p vdn-io-macos -p vdn-hal` 全过（io-macos 13 + 不分配 1，hal 8 + 不分配 1，真设备 2 个 ignored）；clippy 0 警告；tidy 通过；check-deps 通过
- 没跑：开真麦克风 / 扬声器的 2 个测试（会弹权限框、出声，要经大总管问人）。人工跑法：`cargo test -p vdn-io-macos --test real_device -- --ignored --nocapture`
- 合并后要做：`vdn-hal` 接 `vdn-shm` 的 `Out` 环（见下「最终签名」末条）

## 改了哪些文件
- 新：`src/io-macos/`（`Cargo.toml` · `src/lib.rs` · `src/pump.rs` · `src/stream.rs` · `tests/no_alloc.rs` · `tests/real_device.rs` · `examples/devices.rs`）
- 新：`src/hal/`（`Cargo.toml` · `src/lib.rs` · `src/source.rs` · `src/factory.rs` · `tests/no_alloc.rs` · `bundle/Info.plist`）
- 根 `Cargo.toml`：members 加 `src/io-macos` `src/hal`；workspace 依赖加 `cpal = "0.18.2"` · `vdn-io-macos` · `vdn-hal`（后两行大总管已同意保留）
- `Cargo.lock`：cpal 及其依赖随之进来（自动生成）
- 本页

## 新 crate 实体行（给大总管写进 `.42cog/cog.md`）
| 实体 | 是什么 | 谁产生 | 谁是谁的输入 |
|---|---|---|---|
| `vdn-io-macos` | Mac 音频进出：列设备、开麦克风输入流、开扬声器输出流，单声道 f32、48 kHz | 麦克风 / 扬声器硬件经 cpal | 输入流 → `vdn-capture` 的 Forwarder（推 Raw 环）；开发期输出流 ← `Out` 环 → 扬声器 |
| `vdn-hal` | 自有虚拟麦克风的读出端与插件外壳（Rust cdylib，D55） | 本项目；将来由 coreaudiod 加载 | `vdn-shm` 的 `Out` 环 → `HalSource::render` → 会议软件 |

## 新外部依赖与许可
- `cpal 0.18.2`：Apache-2.0，与 GPL-3.0 兼容。传递依赖里的 `coreaudio-rs 0.14.2`：MIT / Apache-2.0
- `vdn-hal` 没有外部依赖

## 最终签名（已发大总管与 rs-entry）

### vdn-io-macos
```rust
pub const IO_RATE: u32 = 48_000;
pub fn devices() -> Vec<DeviceInfo>;          // 不开流，不弹权限框
pub struct DeviceInfo { name, direction: Direction, default_rate: u32, channels: u16, is_default: bool, supports_io_rate: bool }
pub enum Direction { Input, Output }          // 双向设备出现两行

InputStream::open(device: Option<&str>, rate: u32, block: usize,
                  cb: impl FnMut(&[f32]) + Send + 'static) -> Result<InputStream, IoError>;
OutputStream::open(device: Option<&str>, rate: u32, block: usize,
                   cb: impl FnMut(&mut [f32]) + Send + 'static) -> Result<OutputStream, IoError>;
// 两者都有 info() -> &OpenedInfo { device, rate, channels, block, device_buffer: Option<u32> }
//         health() -> &StreamHealth { errors(), xruns(), device_lost(), needs_reopen(), device_changed(), callback_panicked() }
pub enum IoError { InvalidParam, NoDefaultDevice, DeviceNotFound{wanted, direction, candidates},
                   UnsupportedRate{device, wanted, supported: Vec<RateRange>}, Backend(String) }
```
- 回调拿单声道；每次 **≤ block 帧**，可能更少，不保证恰好 block
- 输出回调拿到已清零的块
- drop 句柄即停流；设备拔掉或设备采样率被别的程序改掉时不 panic，`health().needs_reopen()` 变 true，不自动重建
- 用户回调 panic：被接住，之后输入丢弃、输出静音，`callback_panicked()` 变 true
- 副作用：开流会改设备全局采样率与硬件缓冲帧数，同设备上的会议软件也受影响，失败或 drop 后不还原（还原留到自写 IOProc 那批）
- `device = None` 的默认输出流跟随系统默认输出切换（`device_changed()` 变 true）；按名字开的不跟随
- 名字为空、或两台设备同名：返回 `DeviceNotFound` 列出候选，不猜
- 设备硬件缓冲：设备接受就设成 block，不接受就用设备默认（`info().device_buffer` 为 `None`），不因此失败
- 设备名：先精确匹配，再找唯一一个不分大小写包含的
- 设备不支持 rate → `UnsupportedRate`，错误文字列出设备支持的「Hz × 声道」
- 公开纯函数：`pump::{fold_first_channel, fan_out, InputPump, OutputPump}`

### vdn-hal
```rust
pub trait SampleSource {
    fn pop(&mut self, out: &mut [f32]) -> usize;
    fn note_underrun(&mut self, _missing: usize) {}   // 默认空；接 shm 时转 ring.note_underrun
}
pub struct FnSource<F>(pub F);                         // 闭包当源
HalSource::new(source) / render(&mut [f32]) -> RenderStatus { Full | Underrun { got } } / counters() -> Arc<HalCounters>
#[unsafe(no_mangle)] extern "C" fn VdnHalFactory(*const c_void, *const c_void) -> *mut c_void   // 返回空 = 不加载
```
- 接 `vdn-shm`（合并后做，rs-shm 已给签名）：`struct OutSource(Arc<Bus>)`，`pop` 转 `bus.ring(RingId::Out).pop(out)`，`note_underrun` 转 `ring.note_underrun(n)`。本批 worktree 里没有 vdn-shm，所以没加依赖

## 本机设备清单（`cargo run -p vdn-io-macos --example devices`，2026-11-02 实测，只列不开流）
| 名字 | 方向 | 默认采样率 | 声道 | 默认 | 支持 48 kHz |
|---|---|---|---|---|---|
| External Headphones | 输出 | 48000 | 2 | 是 | 是 |
| MacBook Air Microphone | 输入 | 48000 | 1 | 是 | 是 |
| MacBook Air Speakers | 输出 | 48000 | 2 |  | 是 |
| （人的 iPhone）Microphone（连续互通麦克风，名字含人名，此处略去） | 输入 | 48000 | 1 |  | 是 |

## 已知违规清单（A 档；cpal 期间不算通过，M1 自写 IOProc 起清零）
读源码确认，路径都在 `~/.cargo/registry/src/index.crates.io-*/`：
1. `cpal-0.18.2/src/host/coreaudio/mod.rs:69-71`：每次回调调 `mach_timebase_info`（系统调用）
2. `coreaudio-rs-0.14.2/src/audio_unit/render_callback.rs:622-649`：输入回调里帧数一变就读 AudioUnit 属性并 `Vec::resize`（分配）
3. `cpal-0.18.2/src/host/coreaudio/macos/device.rs:781`：时间戳出错时走 `try_emit_error`（`host/error_emit.rs:25-31`：在实时线程上对错误回调的 `Mutex` 做 `try_lock`，拿到锁就在实时线程上跑错误回调。我们的错误回调 `StreamHealth::record` 只做原子操作；但 `cpal::Error` 可能带堆上的消息字符串，构造与 drop 都在实时线程，属分配 / 释放，只在出错时发生）
4. `cpal-0.18.2/src/traits.rs:279`（输出侧 `:336`）：`build_input_stream` / `build_output_stream` 包装里有 `.expect("host supplied incorrect sample type")`；我们只要 f32，实际走不到，但它是回调路径上的 panic 点
5. `cpal macos/mod.rs:112-116, 181-191`：处理器过载通知在实时线程上经 `try_lock` 调我们的错误回调。我们的错误回调只做原子操作，**今后也必须保持这样**
6. `cpal macos/device.rs:785`（输入）· `:913`（输出）：`len / channels`，设备若报 0 声道会除零 panic（理论上）
7. `coreaudio-rs-0.14.2/src/audio_unit/render_callback.rs:667`：`NonNull::new(..).unwrap()`
8. 我们自己的回调泵：零分配（`tests/no_alloc.rs` 实测）、无锁、无 panic 路径（crate 根 panic 类 deny + 实时块 deny 禁用清单）

## 待人拍板
- 无（cli 禁 cpal 的冲突已报大总管，不是人题）

## 流水
- 读完交代与 010 第十节、规约 §4/§5、D05/D54/D55；向大总管报到
- 查 cpal 0.18.2 源码：许可 Apache-2.0；`Stream` 是 `Send`
- 建两个 crate；根 Cargo.toml 加成员与依赖
- 单测：hal 8 + 不分配 1；io-macos 11 + 不分配 1；2 个真设备测试 ignored
- `nm` 确认 `libvdn_hal.dylib` 导出 `_VdnHalFactory`
- 签名发大总管与 rs-entry；问 rs-shm 拿到 Out 环签名，加 `note_underrun` 钩子与 `FnSource`
- 自审第一轮（自己）：硬件缓冲设不了改为回退默认；空段不调用户回调；文档写明用户回调不许 panic
- tidy 通过、check-deps 通过；发现 `src/cli` 的 forbid 含 cpal，报大总管
- 派一个子代理做三视角对抗自审（实时安全 · 拔插健壮性 · 插件外壳）：P0 0 条，P1 2 条，P2 4 条，P3 3 条
- 修：P1-1 健康计数分出 xrun / 需重开 / 设备切换；P1-2 改设备全局设置写进文档（不还原，留后批）；P2-3 回调 panic 用 catch_unwind 接住并静音；P2-4 补进已知违规清单；P2-5 默认设备跟随写进文档并置位；P2-6 同名设备不猜；P3-8 空名拒绝；P3-9 plist 测试不再依赖缩进。P3-7 无需改
- 插件外壳自审结论：不会被系统误加载（只产出 `target/` 下的 dylib；即使打包，工厂返回空也会被跳过）；工厂签名与 Info.plist 两张表正确
- 打包时要做（记给 D05 那批）：可执行文件改名进 `Contents/MacOS/VdnVirtualMic`；真工厂要核对请求的类型 UUID 并返回驱动接口；每个导出的 C 入口都要 catch_unwind（panic 会拖垮 coreaudiod，全机没声）；签名；加 `AudioServerPlugIn_MachServices`
- 修完复跑：测试全过、clippy 0 警告、tidy 通过、check-deps 通过
