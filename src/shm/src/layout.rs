//! 共享内存里的字节布局（跨进程的约定，改了必须升 `VERSION`）。
//!
//! ```text
//! 偏移 0      Header（#[repr(C, align(64))]，大小 = HEADER_BYTES）
//!               ├ 第 0 行：magic · version · rate · capacity_frames · l_total · total_bytes · lead · denoise_block · session（创建方写一次）
//!               ├ 第 1 行：heartbeat                                                          （降噪进程写）
//!               └ rings[3]：每个环两行，生产方一行（write · overflow · wake）、消费方一行（read · underrun · wait_fallbacks · gaps）
//! 偏移 HEADER_BYTES   样本槽：3 × capacity_frames 个 AtomicU32（f32::to_bits），按 Raw · Processed · Out 排
//! ```
//!
//! 全部字段都是原子量：另一个进程随时可能在读写，这里不出现普通字段，也就不需要对「对方写了一半」做假设。
//! 读写索引是单调递增的 u64（每秒 48 000 样本，约 1200 万年才回绕），可用量 = write − read。

use crate::sync::{AtomicU32, AtomicU64};
use core::mem::size_of;
#[cfg(not(feature = "loom"))]
use core::mem::{align_of, offset_of};

/// 数值读作 ASCII "VDN_SHM1"（按本机小端存，十六进制工具里看到的字节是倒序的）。0 = 创建方还没写完头。
pub const MAGIC: u64 = 0x5644_4E5F_5348_4D31;
/// 布局版本。改了本文件里任何一个字段、顺序或大小，就加 1。
/// - 1：初版
/// - 2：生产方行加 `wake`（唤醒计数字，`Bus::notify` / `Bus::wait` 用）；消费方行加 `wait_fallbacks` · `gaps`；
///   配置行加 `lead` · `denoise_block` · `session`
pub const VERSION: u32 = 2;
/// 环的个数（`RingId` 的变体数）。
pub const RINGS: usize = 3;
/// 单环容量上限（帧）：防止对方写进来的头字段让我们算出离谱的映射大小。
pub const MAX_CAPACITY_FRAMES: u32 = 1 << 24;

/// 生产方独占的一行：写索引、溢出计数、唤醒计数字。
#[repr(C, align(64))]
pub struct ProdLine {
    pub write: AtomicU64,
    pub overflow: AtomicU64,
    /// 唤醒计数字：`notify` 每次加 1 再叫醒；`wait` 拿它当 `os_sync_wait_on_address` 的比较值。
    /// 必须是 4 字节、按 4 对齐（系统调用的要求），并且跟 `write` 分开：`push` 不碰它
    pub wake: AtomicU32,
}

/// 消费方独占的一行：读索引、欠载计数、等待退回轮询的次数。
#[repr(C, align(64))]
pub struct ConsLine {
    pub read: AtomicU64,
    pub underrun: AtomicU64,
    /// `wait` 因系统不支持（低于 macOS 14.4）或系统调用出错而改用轮询的次数（每次 `wait` 调用最多记 1）
    pub wait_fallbacks: AtomicU64,
    /// 缺口样本数：消费方因落后而主动丢掉的积压（`Ring::note_gap`），与欠载分开计
    pub gaps: AtomicU64,
}

#[repr(C)]
pub struct RingHdr {
    pub prod: ProdLine,
    pub cons: ConsLine,
}

/// 创建方写一次、之后只读的配置行。
#[repr(C, align(64))]
pub struct ConfigLine {
    /// 最后写（Release）；打开方先读它（Acquire），不是 MAGIC 就不往下读
    pub magic: AtomicU64,
    pub version: AtomicU32,
    pub rate: AtomicU32,
    pub capacity_frames: AtomicU32,
    pub l_total: AtomicU32,
    /// 整块共享内存的字节数（= HEADER_BYTES + 3 × capacity_frames × 4），打开方核对
    pub total_bytes: AtomicU64,
    /// 跨进程缓冲余量（48 kHz 样本）：送音端校验自己的设置用
    pub lead: AtomicU32,
    /// 降噪进程的处理块长（样本）
    pub denoise_block: AtomicU32,
    /// 会话 ID：`create` 时随机生成、非 0；打开方记下它，重开后不同就说明总线换了
    pub session: AtomicU64,
}

#[repr(C, align(64))]
pub struct BeatLine {
    pub heartbeat: AtomicU64,
}

#[repr(C)]
pub struct Header {
    pub config: ConfigLine,
    pub beat: BeatLine,
    pub rings: [RingHdr; RINGS],
}

pub const HEADER_BYTES: usize = size_of::<Header>();
/// 每个样本槽 4 字节（u32 位模式）。写死而不用 `size_of::<AtomicU32>()`：loom 下原子量更大，但字节布局约定不变
pub const SLOT_BYTES: usize = 4;

/// 整块大小；容量非法（0 或超上限）时返回 None。
pub const fn total_bytes(capacity_frames: u32) -> Option<usize> {
    if capacity_frames == 0 || capacity_frames > MAX_CAPACITY_FRAMES {
        return None;
    }
    Some(HEADER_BYTES + RINGS * capacity_frames as usize * SLOT_BYTES)
}

/// loom 下没有「一块全零内存就是合法头部」这回事：逐字段 new。
#[cfg(feature = "loom")]
impl Header {
    pub fn new_zeroed() -> Self {
        let u = || AtomicU64::new(0);
        let w = || AtomicU32::new(0);
        let ring = || RingHdr {
            prod: ProdLine {
                write: u(),
                overflow: u(),
                wake: w(),
            },
            cons: ConsLine {
                read: u(),
                underrun: u(),
                wait_fallbacks: u(),
                gaps: u(),
            },
        };
        Self {
            config: ConfigLine {
                magic: u(),
                version: w(),
                rate: w(),
                capacity_frames: w(),
                l_total: w(),
                total_bytes: u(),
                lead: w(),
                denoise_block: w(),
                session: u(),
            },
            beat: BeatLine { heartbeat: u() },
            rings: [ring(), ring(), ring()],
        }
    }
}

// 跨进程要求 64 位原子操作无锁（arm64 / x86_64 都满足）
#[cfg(not(target_has_atomic = "64"))]
compile_error!("vdn-shm 需要无锁的 64 位原子操作");

// 布局钉死：任何一项变了编译就失败，提醒升 VERSION 并通知 HAL / 对端（若驱动是 C，cbindgen 出头文件）。
// 这些是编译期断言，不在运行期执行。loom 下原子量大小不同，不查（loom 只测协议，不测布局）。
#[cfg(not(feature = "loom"))]
const _: () = {
    assert!(size_of::<ProdLine>() == 64 && size_of::<ConsLine>() == 64);
    assert!(size_of::<RingHdr>() == 128);
    assert!(offset_of!(RingHdr, prod) == 0 && offset_of!(RingHdr, cons) == 64);
    assert!(offset_of!(ProdLine, write) == 0 && offset_of!(ProdLine, overflow) == 8);
    assert!(offset_of!(ProdLine, wake) == 16);
    assert!(offset_of!(ConsLine, read) == 0 && offset_of!(ConsLine, underrun) == 8);
    assert!(offset_of!(ConsLine, wait_fallbacks) == 16);
    assert!(offset_of!(ConsLine, gaps) == 24);
    assert!(size_of::<ConfigLine>() == 64);
    assert!(offset_of!(ConfigLine, magic) == 0);
    assert!(offset_of!(ConfigLine, version) == 8);
    assert!(offset_of!(ConfigLine, rate) == 12);
    assert!(offset_of!(ConfigLine, capacity_frames) == 16);
    assert!(offset_of!(ConfigLine, l_total) == 20);
    assert!(offset_of!(ConfigLine, total_bytes) == 24);
    assert!(offset_of!(ConfigLine, lead) == 32);
    assert!(offset_of!(ConfigLine, denoise_block) == 36);
    assert!(offset_of!(ConfigLine, session) == 40);
    assert!(offset_of!(Header, beat) == 64);
    assert!(offset_of!(Header, rings) == 128);
    assert!(HEADER_BYTES == 128 + 3 * 128);
    assert!(align_of::<Header>() == 64);
    // 样本槽紧跟头部，对齐到 AtomicU32
    assert!(HEADER_BYTES.is_multiple_of(align_of::<AtomicU32>()));
    // AtomicU32 / AtomicU64 与 u32 / u64 同大小同对齐（跨进程、跨语言读同一块内存的前提）
    assert!(size_of::<AtomicU32>() == 4 && align_of::<AtomicU32>() == 4);
    assert!(size_of::<AtomicU64>() == 8 && align_of::<AtomicU64>() == 8);
};
