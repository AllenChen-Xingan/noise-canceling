//! 头部布局的机读描述：`vdn-schema` 把它导出成 `specs/schema/shm-header.layout.json`（跨进程、跨语言的约定）。
//!
//! 正本仍是 `layout.rs` 里的结构；这里的偏移和大小全由 `offset_of!` / `size_of` 算出，不手写数字。
//! `describe!` 对每个结构做穷举解构：结构加了字段而这里没加，编译失败；字段类型写错，编译失败。
//! 导出文件进仓，`vdn-schema` 的测试逐字比对：改布局不重新导出 → 测试红；改布局不升 `VERSION` → 测试红。

use core::mem::{align_of, offset_of, size_of};

use crate::layout::{BeatLine, ConfigLine, ConsLine, Header, ProdLine, RINGS, RingHdr};
use crate::sync::{AtomicU32, AtomicU64};

/// 字段的类型。原子量在内存里与同宽的无符号整数相同（`layout.rs` 编译期断言）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    U32,
    U64,
    Struct(&'static StructDesc),
    /// 同一结构连排 n 个
    Array(&'static StructDesc, usize),
}

impl Kind {
    pub const fn size(self) -> usize {
        match self {
            Kind::U32 => 4,
            Kind::U64 => 8,
            Kind::Struct(s) => s.size,
            Kind::Array(s, n) => s.size * n,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDesc {
    pub name: &'static str,
    /// 相对所在结构起点的字节偏移
    pub offset: usize,
    pub size: usize,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructDesc {
    pub name: &'static str,
    pub size: usize,
    pub align: usize,
    pub fields: &'static [FieldDesc],
}

/// `describe!(S { a: T = kind, … })`：穷举解构 S（漏字段编译失败）、核对字段类型是 T、
/// 核对 `kind` 的大小等于 `size_of::<T>()`，再用 `offset_of!` 填偏移。
macro_rules! describe {
    ($s:ident { $($f:ident : $t:ty = $k:expr),* $(,)? }) => {{
        #[allow(dead_code)]
        fn exhaustive(x: &$s) {
            let $s { $($f),* } = x;
            $(let _: &$t = $f;)*
        }
        $(assert!(size_of::<$t>() == $k.size(), concat!(stringify!($s), ".", stringify!($f), "：kind 与类型大小不符"));)*
        StructDesc {
            name: stringify!($s),
            size: size_of::<$s>(),
            align: align_of::<$s>(),
            fields: &[$(FieldDesc { name: stringify!($f), offset: offset_of!($s, $f), size: size_of::<$t>(), kind: $k }),*],
        }
    }};
}

pub const CONFIG_LINE: StructDesc = describe!(ConfigLine {
    magic: AtomicU64 = Kind::U64,
    version: AtomicU32 = Kind::U32,
    rate: AtomicU32 = Kind::U32,
    capacity_frames: AtomicU32 = Kind::U32,
    l_total: AtomicU32 = Kind::U32,
    total_bytes: AtomicU64 = Kind::U64,
    lead: AtomicU32 = Kind::U32,
    denoise_block: AtomicU32 = Kind::U32,
    session: AtomicU64 = Kind::U64,
});

pub const BEAT_LINE: StructDesc = describe!(BeatLine {
    heartbeat: AtomicU64 = Kind::U64,
});

pub const PROD_LINE: StructDesc = describe!(ProdLine {
    write: AtomicU64 = Kind::U64,
    overflow: AtomicU64 = Kind::U64,
    wake: AtomicU32 = Kind::U32,
});

pub const CONS_LINE: StructDesc = describe!(ConsLine {
    read: AtomicU64 = Kind::U64,
    underrun: AtomicU64 = Kind::U64,
    wait_fallbacks: AtomicU64 = Kind::U64,
    gaps: AtomicU64 = Kind::U64,
});

pub const RING_HDR: StructDesc = describe!(RingHdr {
    prod: ProdLine = Kind::Struct(&PROD_LINE),
    cons: ConsLine = Kind::Struct(&CONS_LINE),
});

pub const HEADER: StructDesc = describe!(Header {
    config: ConfigLine = Kind::Struct(&CONFIG_LINE),
    beat: BeatLine = Kind::Struct(&BEAT_LINE),
    rings: [RingHdr; RINGS] = Kind::Array(&RING_HDR, RINGS),
});

/// 全部结构，被引用的在前（导出顺序）。
pub const STRUCTS: [&StructDesc; 6] = [
    &CONFIG_LINE,
    &BEAT_LINE,
    &PROD_LINE,
    &CONS_LINE,
    &RING_HDR,
    &HEADER,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::HEADER_BYTES;

    /// 每个结构的字段按偏移递增、互不重叠、不越出结构
    #[test]
    fn fields_ordered_and_inside() {
        for s in STRUCTS {
            let mut end = 0;
            for f in s.fields {
                assert!(f.offset >= end, "{}.{} 与前一个字段重叠", s.name, f.name);
                assert_eq!(f.size, f.kind.size(), "{}.{}", s.name, f.name);
                end = f.offset + f.size;
            }
            assert!(end <= s.size, "{} 的字段越界", s.name);
        }
        assert_eq!(HEADER.size, HEADER_BYTES);
    }
}
