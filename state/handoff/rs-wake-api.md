# vdn-shm 新签名（rs-wake，2026-11-02，头部版本 1 → 2）

这是签名**快照**，正本是源码（`src/shm/src/`）；两者不一致时以源码为准。已编译、`cargo test -p vdn-shm` 全过（自审修完后 30 过 1 忽略）、clippy 无警告、tidy 通过。代码在 `noise_canceling-rs-shm` worktree 的 `src/shm/`。

## 1. 头部版本 2（两端必须同版本，`open` 遇旧版本报 `ShmError::BadVersion`）

`BusConfig` 多两个字段（**结构体字面量要补上**，否则编译不过）：

```rust
pub struct BusConfig {
    pub rate: u32,
    pub capacity_frames: u32,
    pub l_total: u32,
    pub lead: u32,           // 新：跨进程缓冲余量（48 kHz 样本），≤ capacity_frames
    pub denoise_block: u32,  // 新：降噪处理块长（样本），≤ capacity_frames
}
```

- `Bus::session_id(&self) -> u64`：`create` / `heap` 时随机生成、非 0；打开方记下，重开后不同 = 总线换了
- `bus.config()` 读回 lead / denoise_block；`open` 时若它们大于容量报 `InvalidConfig`
- 缺口计数（每个环一个，消费方因落后主动丢积压时记，与欠载分开）：`Ring::note_gap(n)` · `Ring::gaps() -> u64` · `Ring::take_gaps() -> u64`。实时可调
- `Ring::wait_fallbacks() -> u64`：该环上 `wait` 退回轮询的次数

## 2. 唤醒

```rust
// 生产方 push 完调。实时可调（规约 §4 A 档唯一例外）：一次原子加 + 一次非阻塞系统调用（os_sync_wake_by_address_all）
pub fn Bus::notify(&self, id: RingId);
// push + notify 合写（顺序不会写反），实时可调；返回写进去的样本数
pub fn Bus::push_notify(&self, id: RingId, src: &[f32]) -> usize;

// 消费方调。只许在非实时线程调（会睡）
pub fn Bus::wait(&self, id: RingId, min_available: usize, timeout: Duration) -> WaitResult;

#[must_use]
pub enum WaitResult { Ready { available: usize }, TimedOut { available: usize } }
impl WaitResult { fn available(self) -> usize; fn is_ready(self) -> bool }

pub enum WaitStrategy { Wake /* 默认 */, Poll(Duration) }
pub fn Bus::set_wait_strategy(&mut self, s: WaitStrategy);
pub fn Bus::wake_supported() -> bool;       // 本构建有没有唤醒（macOS + feature posix 为 true）
pub const FALLBACK_POLL: Duration;          // 200 µs
```

语义：
- 数据够 `min_available` 立刻返回 `Ready`；不够就睡，生产方 `notify` 一下就醒；被叫醒但数据仍不够会接着睡
- 一定在 `timeout` 后不久返回（生产方崩了也只是 `TimedOut`）；`timeout = 0` 只看一眼
- `min_available` 大于容量按容量算
- 最低系统 macOS 14.4（人定 D67），系统函数直接链接；系统调用返回不可恢复的错时，自动改 200 µs 轮询，`wait_fallbacks` 加 1。Linux / Miri / loom 下没有唤醒，一律轮询
- **配对要求**：生产方每次 push（或一批 push）之后必须调 `notify(同一个环)`，否则消费方只能等到超时

### 接线建议（rs-entry）
- 送音回调：`bus.push_notify(RingId::Raw, &块)`（或 `push` 后 `notify`，**顺序不能反**）
- 降噪进程：把 `sleep(500 µs)` 循环换成 `bus.wait(RingId::Raw, denoise_block, Duration::from_millis(20))`，按返回值处理
- 降噪进程推完 Processed 后若收音端也要等它，同样 `notify(RingId::Processed)`（送音回调本身不许调 `wait`）

## 3. OwnedBus（建方守卫，替 rs-entry 的 `denoised::owned_bus`）

```rust
pub struct OwnedBus;   // Deref<Target = Bus>
impl OwnedBus {
    pub fn create(name: &str, cfg: BusConfig) -> Result<Self, ShmError>; // 同名正被活着的建方持有 → ShmError::Busy
    pub fn unlink(&self) -> Result<(), ShmError>;   // 只删一次
    pub fn bus_mut(&mut self) -> &mut Bus;          // 例如 set_wait_strategy
    pub fn lock_path(&self) -> &Path;
    pub fn lock_path_for(name: &str) -> PathBuf;    // $TMPDIR/vdn-shm_<名>.lock，如 /vdn-out → vdn-shm_vdn-out.lock
}
// Drop：删名字一次，再放锁
```

新错误变体：`ShmError::Busy`（若有穷举 match 要补）。和原件的差别：错误返回 `ShmError` 而不是 `String`；锁文件新名 `vdn-shm_<名>.lock`，**过渡期同时也拿原件用的 `vdn-bus-lock_<名>`**，所以新旧建方同时跑也互相锁得住（自审 P1 已修）。换完后请删掉 `denoised::owned_bus`。
