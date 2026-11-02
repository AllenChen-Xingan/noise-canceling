# 现状页 · 手下 rs-io（批次 20261102-b8-rust）

- 窗口：rs-io · 大总管 noise-canceling-47 · 分支 `feat/rust-skeleton-io`
- 管的实体：`vdn-io-macos`（新）· `vdn-hal`（新）

## 手上 · 做到哪 · 下一步
- 手上：两个 crate 接口已编译通过，单测全过，clippy 0 警告
- 下一步：列本机设备 → 自审 → 收尾

## 最终签名（2026-11-02，编译通过、单测全过）

依赖：`vdn-io-macos.workspace = true` / `vdn-hal.workspace = true`（根 Cargo.toml 已加这两行 workspace 依赖）

### vdn-io-macos（src/io-macos）
```rust
pub const IO_RATE: u32 = 48_000;
pub fn devices() -> Vec<DeviceInfo>;          // 不开流，不弹权限框
pub struct DeviceInfo { name: String, direction: Direction, default_rate: u32, channels: u16, is_default: bool, supports_io_rate: bool }
pub enum Direction { Input, Output }          // 双向设备出现两行

impl InputStream {
  pub fn open(device: Option<&str>, rate: u32, block: usize,
              cb: impl FnMut(&[f32]) + Send + 'static) -> Result<Self, IoError>;
  pub fn info(&self) -> &OpenedInfo;          // 实际设备名、rate、设备声道数、block
  pub fn health(&self) -> &StreamHealth;      // errors() 次数、device_lost()
}
impl OutputStream {
  pub fn open(device: Option<&str>, rate: u32, block: usize,
              cb: impl FnMut(&mut [f32]) + Send + 'static) -> Result<Self, IoError>;
  // info() / health() 同上
}
pub enum IoError { InvalidParam(String), NoDefaultDevice(Direction),
  DeviceNotFound{wanted, direction, candidates: Vec<String>},
  UnsupportedRate{device, wanted: u32, supported: Vec<RateRange>}, Backend(String) }  // impl Display + Error
```
语义：
- 回调拿单声道；每次 **≤ block 帧**（cpal 一次给多了会切段；可能少于 block，不保证恰好 block）
- 输出回调拿到的是**已清零**的块，写多少算多少
- drop 句柄即停流；设备拔掉不 panic，`health().device_lost()` 变 true，不自动重建
- 设备名：先精确匹配，再找唯一一个不分大小写包含的；不唯一 / 找不到返回 DeviceNotFound 并列候选
- 设备不支持 rate → UnsupportedRate，Display 里列出设备支持的「Hz × 声道」
- 附带公开的纯函数：`pump::{fold_first_channel, fan_out, InputPump, OutputPump}`（chain-test 可复用）

### vdn-hal（src/hal）
```rust
pub trait SampleSource { fn pop(&mut self, out: &mut [f32]) -> usize; }   // 等 rs-shm 的 Out 环消费端实现
impl<S: SampleSource> HalSource<S> {
  pub fn new(source: S) -> Self;
  pub fn render(&mut self, out: &mut [f32]) -> RenderStatus;   // 实时块，零分配；源分段给会循环取
  pub fn counters(&self) -> Arc<HalCounters>;                   // underruns() 块数 · underrun_samples() · renders()
}
#[must_use] pub enum RenderStatus { Full, Underrun { got: usize } }   // 不够补零
#[no_mangle] extern "C" fn VdnHalFactory(*const c_void, *const c_void) -> *mut c_void  // 返回空 = 不加载
```

## 流水
- 读完交代与 010 第十节、规约 §4/§5、D05/D54/D55；向大总管报到
- 查 cpal 0.18.2 源码（registry）：许可 Apache-2.0，与 GPL-3.0 兼容；`Stream` 是 `Send`
- 建 `src/io-macos`（pump 回调泵 · stream 开流 · lib 设备与错误）、`src/hal`（source 读出端 · factory 工厂外壳 · bundle/Info.plist）
- 根 Cargo.toml：加 members `src/io-macos` `src/hal`；workspace 依赖 `cpal = "0.18.2"`、`vdn-io-macos`、`vdn-hal`（后两行给 rs-entry 用）
- `cargo test -p vdn-io-macos -p vdn-hal`：hal 7 过 · io-macos 10 + 1（不分配）过 · 2 个真设备测试 ignored
- `nm` 确认 `libvdn_hal.dylib` 导出 `_VdnHalFactory`
