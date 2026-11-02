//! 共享内存里的字节布局（跨进程的约定，改了必须升 `VERSION`）。
//!
//! ```text
//! 偏移 0      Header（#[repr(C, align(64))]，大小 = HEADER_BYTES）
//!               ├ 第 0 行：magic · version · rate · capacity_frames · l_total · total_bytes   （创建方写一次）
//!               ├ 第 1 行：heartbeat                                                          （降噪进程写）
//!               └ rings[3]：每个环两行，生产方一行（write · overflow）、消费方一行（read · underrun）
//! 偏移 HEADER_BYTES   样本槽：3 × capacity_frames 个 AtomicU32（f32::to_bits），按 Raw · Processed · Out 排
//! ```
//!
//! 全部字段都是原子量：另一个进程随时可能在读写，这里不出现普通字段，也就不需要对「对方写了一半」做假设。
//! 读写索引是单调递增的 u64（每秒 48 000 样本，约 1200 万年才回绕），可用量 = write − read。

use core::mem::{align_of, offset_of, size_of};
use core::sync::atomic::{AtomicU32, AtomicU64};

/// 数值读作 ASCII "VDN_SHM1"（按本机小端存，十六进制工具里看到的字节是倒序的）。0 = 创建方还没写完头。
pub const MAGIC: u64 = 0x5644_4E5F_5348_4D31;
/// 布局版本。改了本文件里任何一个字段、顺序或大小，就加 1。
pub const VERSION: u32 = 1;
/// 环的个数（`RingId` 的变体数）。
pub const RINGS: usize = 3;
/// 单环容量上限（帧）：防止对方写进来的头字段让我们算出离谱的映射大小。
pub const MAX_CAPACITY_FRAMES: u32 = 1 << 24;

/// 生产方独占的一行：写索引与溢出计数。
#[repr(C, align(64))]
pub struct ProdLine {
    pub write: AtomicU64,
    pub overflow: AtomicU64,
}

/// 消费方独占的一行：读索引与欠载计数。
#[repr(C, align(64))]
pub struct ConsLine {
    pub read: AtomicU64,
    pub underrun: AtomicU64,
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
pub const SLOT_BYTES: usize = size_of::<AtomicU32>();

/// 整块大小；容量非法（0 或超上限）时返回 None。
pub const fn total_bytes(capacity_frames: u32) -> Option<usize> {
    if capacity_frames == 0 || capacity_frames > MAX_CAPACITY_FRAMES {
        return None;
    }
    Some(HEADER_BYTES + RINGS * capacity_frames as usize * SLOT_BYTES)
}

// 跨进程要求 64 位原子操作无锁（arm64 / x86_64 都满足）
#[cfg(not(target_has_atomic = "64"))]
compile_error!("vdn-shm 需要无锁的 64 位原子操作");

// 布局钉死：任何一项变了编译就失败，提醒升 VERSION 并通知 HAL / 对端（若驱动是 C，cbindgen 出头文件）。
// 这些是编译期断言，不在运行期执行。
const _: () = {
    assert!(size_of::<ProdLine>() == 64 && size_of::<ConsLine>() == 64);
    assert!(size_of::<RingHdr>() == 128);
    assert!(offset_of!(RingHdr, prod) == 0 && offset_of!(RingHdr, cons) == 64);
    assert!(offset_of!(ProdLine, write) == 0 && offset_of!(ProdLine, overflow) == 8);
    assert!(offset_of!(ConsLine, read) == 0 && offset_of!(ConsLine, underrun) == 8);
    assert!(size_of::<ConfigLine>() == 64);
    assert!(offset_of!(ConfigLine, magic) == 0);
    assert!(offset_of!(ConfigLine, version) == 8);
    assert!(offset_of!(ConfigLine, rate) == 12);
    assert!(offset_of!(ConfigLine, capacity_frames) == 16);
    assert!(offset_of!(ConfigLine, l_total) == 20);
    assert!(offset_of!(ConfigLine, total_bytes) == 24);
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
