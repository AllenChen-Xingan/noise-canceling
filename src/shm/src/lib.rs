//! vdn-shm：收音进程 · 降噪进程 · 虚拟麦克风之间传声音的共享内存总线（规约 §2.1「适配·协议」，A 档）。
//!
//! 一块内存 = 头部（配置 · 心跳 · 计数器）+ 三个单生产者单消费者环（010 第十节链路图）：
//!
//! | 环 | 谁写 | 谁读 |
//! |---|---|---|
//! | `RingId::Raw` | 收音进程（原声） | 降噪进程 |
//! | `RingId::Processed` | 降噪进程 | 收音进程 |
//! | `RingId::Out` | 收音进程（处理后或延迟原声） | 虚拟麦克风 / 开发期扬声器 |
//!
//! 规矩：
//! - 构造（`create` / `open` / `heap`）在非实时线程做；`ring` · `push` · `pop` · `beat` 等不分配、不加锁、不做系统调用、不 panic
//! - 单生产者单消费者是**约定**：同一个环同一时刻只许一个线程 push、一个线程 pop（跨进程无法由类型系统保证）
//! - 样本槽是 `AtomicU32`（存 `f32::to_bits`）；读写索引 Release / Acquire，样本本身 Relaxed（由索引的 Release / Acquire 发布）
//! - 环满：`push` 只写得下的部分，其余丢掉并计溢出；环空：`pop` 只读得到的部分，欠载由调用方经 `note_underrun` 计
//!   （或用 `pop_or_zero`：余下填 0 并自动计欠载）
//! - 环满时丢的是**最新**样本，不覆盖未读的旧样本；两端时钟漂移时环会停在满、延迟停在最大，
//!   **由消费方负责按 `l_total` 用 `discard` 重新对齐**（本 crate 不自动纠正）
//! - `Ring` 是 `Copy`：同一环两端「各只一个线程」靠约定，类型挡不住（自审 P2，拆 Producer / Consumer 句柄待大总管定）
//! - 本 crate 不依赖任何 vdn crate；POSIX 部分（feature `posix`，默认开）只用 libc
//!
//! 唤醒（头部版本 2 起）：
//! - 生产方 push 完调 `Bus::notify(id)`：实时可调，一次原子加 + 一次非阻塞系统调用（规约 §4 A 档唯一例外）
//! - 消费方在**非实时线程**调 `Bus::wait(id, 至少几个样本, 超时)`：数据够了立刻返回，不够就睡到被叫醒或超时
//! - 低于 macOS 14.4（没有 `os_sync_wait_on_address`）或调用出错：`wait` 自动改 200 µs 轮询，并计 `wait_fallbacks`
//!
//! 建总线的一方用 `OwnedBus::create`（每个名字一把文件锁，拿到锁才清残留；Drop 时删一次名字）。

#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]

#[cfg(all(feature = "loom", feature = "posix"))]
compile_error!(
    "feature loom 只能配 --no-default-features（loom 的原子量不能放进 mmap 的共享内存）"
);

mod bus;
pub mod layout;
#[cfg(all(unix, feature = "posix"))]
mod owned;
mod region;
pub mod sync;
mod wait;
mod wake;

pub use bus::{Bus, Ring};
#[cfg(all(unix, feature = "posix"))]
pub use owned::OwnedBus;
pub use wait::FALLBACK_POLL;

/// `Bus::wait` 怎么等（每个进程自己选，不进共享内存）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitStrategy {
    /// 睡在唤醒计数字上，生产方 `notify` 一下就醒（默认）；系统不支持时自动退回 200 µs 轮询
    Wake,
    /// 按固定间隔 sleep 后看一眼（对照实验、或明确不想用唤醒时）
    Poll(core::time::Duration),
}

/// `Bus::wait` 的结果；两种都带返回那一刻的可读样本数。
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitResult {
    /// 可读样本数已达到要求
    Ready { available: usize },
    /// 超时还不够
    TimedOut { available: usize },
}

impl WaitResult {
    pub fn available(self) -> usize {
        match self {
            Self::Ready { available } | Self::TimedOut { available } => available,
        }
    }

    pub fn is_ready(self) -> bool {
        matches!(self, Self::Ready { .. })
    }
}

/// 建总线时的配置；`open` 时从头部读回。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BusConfig {
    /// IO 采样率（Hz），本批固定 48 000
    pub rate: u32,
    /// 每个环的容量（帧 = 样本，单声道）
    pub capacity_frames: u32,
    /// 送原声用的总延迟（48 kHz 样本，规约 §3.6、ADR 005），由降噪进程按所有启用模型的最大延迟定
    pub l_total: u32,
    /// 跨进程缓冲余量（48 kHz 样本）；送音端读回来核对自己的设置。≤ `capacity_frames`
    pub lead: u32,
    /// 降噪进程的处理块长（样本）；≤ `capacity_frames`
    pub denoise_block: u32,
}

/// 三个环。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RingId {
    /// 原声：收音 → 降噪
    Raw,
    /// 降噪后：降噪 → 收音
    Processed,
    /// 最终输出：收音 → 虚拟麦克风
    Out,
}

impl RingId {
    pub const ALL: [RingId; 3] = [RingId::Raw, RingId::Processed, RingId::Out];
}

/// 构造期错误（实时路径不返回错误）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShmError {
    /// 名字必须以 `/vdn-` 开头、总长 ≤ 31 字节（macOS PSHMNAMLEN）、不含其他 `/` 与 NUL
    InvalidName,
    /// 配置非法（容量为 0 或超上限、采样率为 0 等）
    InvalidConfig(&'static str),
    /// `create`：同名对象已存在（上次没 unlink？先 `Bus::unlink_name`）
    AlreadyExists,
    /// `open`：没有这个名字
    NotFound,
    /// `open`：创建方还没写完头部，稍后重试
    NotReady,
    /// 头部 magic 不对：不是 vdn 总线
    BadMagic,
    /// 布局版本不一致：两端程序版本不同
    BadVersion {
        found: u32,
        expected: u32,
    },
    /// 头部记的大小与映射大小对不上
    BadSize {
        header: u64,
        mapped: u64,
    },
    /// 本平台 / 本构建没有 POSIX 共享内存（feature `posix` 关了或非 unix）
    Unsupported,
    OutOfMemory,
    /// `OwnedBus::create`：同名总线正被另一个活着的建方持有（文件锁拿不到）
    Busy,
    /// 系统调用失败
    Os {
        op: &'static str,
        errno: i32,
    },
}

impl core::fmt::Display for ShmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidName => write!(
                f,
                "共享内存名字须以 /vdn- 开头、≤31 字节、不含其他 / 与 NUL"
            ),
            Self::InvalidConfig(why) => write!(f, "总线配置非法：{why}"),
            Self::AlreadyExists => write!(f, "同名共享内存已存在"),
            Self::NotFound => write!(f, "没有这个共享内存"),
            Self::NotReady => write!(f, "创建方还没写完头部"),
            Self::BadMagic => write!(f, "头部 magic 不对：不是 vdn 总线"),
            Self::BadVersion { found, expected } => {
                write!(f, "布局版本不一致：对端 {found}，本端 {expected}")
            }
            Self::BadSize { header, mapped } => {
                write!(f, "大小对不上：头部记 {header} 字节，映射 {mapped} 字节")
            }
            Self::Unsupported => write!(f, "本构建不支持 POSIX 共享内存"),
            Self::OutOfMemory => write!(f, "分配失败"),
            Self::Busy => write!(f, "同名总线正被另一个进程建着用（换个名字，或先停掉它）"),
            Self::Os { op, errno } => write!(f, "{op} 失败（errno {errno}）"),
        }
    }
}

impl std::error::Error for ShmError {}
