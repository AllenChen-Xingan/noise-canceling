# vdn-shm / vdn-rt-thread 对外签名（rs-shm，2026-11-02，编译通过、测试中）

依赖：`vdn-shm = { path = "src/shm" }`（默认 feature `posix`）；`vdn-rt-thread = { path = "src/rt-thread" }`。
（根 Cargo.toml 的 `[workspace.dependencies]` 我不加 vdn-shm 行，用的人自己按 path 引或由大总管统一加。）

## vdn-shm
```rust
pub struct BusConfig { pub rate: u32, pub capacity_frames: u32, pub l_total: u32 }
pub enum RingId { Raw, Processed, Out }          // RingId::ALL
pub enum ShmError { InvalidName, InvalidConfig(&'static str), AlreadyExists, NotFound, NotReady,
                    BadMagic, BadVersion{found,expected}, BadSize{header,mapped}, Unsupported, OutOfMemory,
                    Os{op,errno} }                // Display + Error, Copy

// 构造（非实时）
Bus::heap(cfg) -> Result<Bus, ShmError>          // 进程内
Bus::create(name, cfg) -> Result<Bus, ShmError>  // 降噪进程；同名已存在 → AlreadyExists
Bus::open(name) -> Result<Bus, ShmError>         // 收音进程 / HAL；校验 magic·版本·大小；NotReady 可重试
bus.unlink() / Bus::unlink_name(name)            // Drop 只解映射，不删名字
bus.name() -> Option<&str>;  bus.config() -> BusConfig
// 名字：以 "/vdn-" 开头，总长 ≤ 31 字节，之后不含 "/"

// 实时可调（不分配、不锁、不 panic）；Bus: Send + Sync，&Bus 可分给两个线程
bus.ring(RingId) -> Ring<'_>                     // Copy，便宜，可每块取一次
bus.beat(); bus.heartbeat() -> u64               // 降噪进程每块 beat
bus.l_total() -> u32; bus.rate() -> u32; bus.capacity() -> usize

ring.push(&[f32]) -> usize        // 写不下的丢掉并计溢出
ring.pop(&mut [f32]) -> usize     // 不够只读到的部分，dst 余下不动；欠载调用方自己 note_underrun
ring.pop_or_zero(&mut [f32]) -> usize  // 不够的部分填 0 并自动计欠载（HAL 输出用）
ring.discard(n) -> usize          // 消费方丢样本重对齐
ring.available() / free_space() / capacity()
ring.note_underrun(n); overflows(); underruns(); take_overflows(); take_underruns()   // take_* = swap(0)
```
单生产者单消费者是约定：同一环同一时刻只一个线程 push、一个线程 pop。

## vdn-rt-thread
```rust
pub struct RtParams { pub period_ns: u64, pub computation_ns: u64, pub constraint_ns: u64 }
RtParams::for_block(block, rate, load) -> RtParams     // period = constraint = 块时长，computation = × load
promote_current(RtParams) -> Result<(), RtError>
check_current() -> Result<RtStatus, RtError>           // RtStatus { time_constraint: bool, params: Option<RtParams>, preemptible: bool }
join_audio_workgroup() -> Result<(), RtError>          // 本批恒为 Err(Unimplemented)
pub enum RtError { InvalidParams(&'static str), Kernel{op, code}, Unsupported, Unimplemented(&'static str) }
```
注意：010 写的是 `check_current() -> RtStatus`，我改成 `Result<RtStatus, RtError>`（非 macOS 与内核调用失败要能报出来）——语义不变，待大总管确认。
