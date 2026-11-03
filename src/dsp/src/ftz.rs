//! FTZ 守卫：作用域内把非正规数（极小的浮点数，CPU 算它会慢几十倍）冲成 0，出作用域恢复原状（规约 R9）。
//!
//! - aarch64：FPCR 的 FZ 位（bit 24）。AArch64 上 FZ 同时冲掉非正规的输入与输出，没有单独的 DAZ
//! - x86_64：MXCSR 的 FTZ（bit 15）+ DAZ（bit 6）
//! - 其他平台与 Miri 下是空操作（Miri 跑不了内联汇编），[`SUPPORTED`] 为 false
//!
//! **只包 DSP 段，不包推理调用**（R9）：`with_intra_threads(1)` 时 ORT 就在调用线程上算，守卫开着等于对 ORT 开 FTZ，
//! 正撞 DeepFilterNet #713（FTZ + 单位范数 0/0 出 NaN；`docs/research/009_rust-ecosystem/decision.md` 去非正规数一行）。
//! 写法学 nih-plug `ScopedFtz`（思路，未拷代码）。
//!
//! 注意：编译器默认浮点环境是「不冲」。常量折叠发生在编译期，不受守卫影响；运行期算出来的值才受影响（测试用 `black_box`）。

#![allow(unsafe_code)] // dsp 进 unsafe 白名单只为这一个模块（大总管 2026-11-02 定）

use core::marker::PhantomData;

/// 本平台能否真的开 FTZ（false 时守卫是空操作）。
pub const SUPPORTED: bool = imp::SUPPORTED;

/// 进作用域开 FTZ / DAZ，出作用域恢复进来前的状态（可嵌套）。只作用于当前线程，所以不能跨线程送（`!Send`）。
#[must_use = "守卫一 drop 就恢复，绑到一个变量上：let _ftz = FtzGuard::new();"]
pub struct FtzGuard {
    prev: imp::Reg,
    _thread_bound: PhantomData<*const ()>,
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl FtzGuard {
    /// 开 FTZ。实时线程可调：不分配、不 panic，只读写一次控制寄存器。
    #[inline]
    pub fn new() -> Self {
        let prev = imp::read();
        let want = prev | imp::MASK;
        if want != prev {
            imp::write(want);
        }
        Self {
            prev,
            _thread_bound: PhantomData,
        }
    }
}

impl Default for FtzGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FtzGuard {
    #[inline]
    fn drop(&mut self) {
        if imp::read() != self.prev {
            imp::write(self.prev);
        }
    }
}

/// 当前线程此刻是否开着 FTZ（测试与自检用；不支持的平台恒为 false）。
#[inline]
pub fn active() -> bool {
    SUPPORTED && imp::read() & imp::MASK == imp::MASK
}

#[cfg(all(target_arch = "aarch64", not(miri)))]
mod imp {
    pub type Reg = u64;
    pub const SUPPORTED: bool = true;
    /// FPCR.FZ
    pub const MASK: Reg = 1 << 24;

    #[inline]
    pub fn read() -> Reg {
        let v: Reg;
        // SAFETY: 读 FPCR 只把值放进一个通用寄存器，不碰内存与栈、不改任何状态；aarch64 用户态可读
        unsafe {
            core::arch::asm!("mrs {}, fpcr", out(reg) v, options(nostack, preserves_flags));
        }
        v
    }

    #[inline]
    pub fn write(v: Reg) {
        // SAFETY: 写 FPCR 只改当前线程的浮点模式；v 来自 read() 的值或只多置了 FZ 位，其余位原样写回。
        // 不给 nomem：让编译器把它当作有内存副作用，不把前后的缓冲读写挪过它
        unsafe {
            core::arch::asm!("msr fpcr, {}", in(reg) v, options(nostack, preserves_flags));
        }
    }
}

#[cfg(all(target_arch = "x86_64", not(miri)))]
mod imp {
    pub type Reg = u32;
    pub const SUPPORTED: bool = true;
    /// MXCSR.FTZ | MXCSR.DAZ
    pub const MASK: Reg = (1 << 15) | (1 << 6);

    #[inline]
    pub fn read() -> Reg {
        let mut v: Reg = 0;
        // SAFETY: stmxcsr 把 MXCSR 存进 v（4 字节、对齐的栈变量），只写这一处内存
        unsafe {
            core::arch::asm!("stmxcsr [{}]", in(reg) &raw mut v, options(nostack, preserves_flags));
        }
        v
    }

    #[inline]
    pub fn write(v: Reg) {
        // SAFETY: ldmxcsr 从 v 读 MXCSR；v 来自 read() 的值或只多置了 FTZ/DAZ 位，保留位原样，不会触发 #GP
        unsafe {
            core::arch::asm!("ldmxcsr [{}]", in(reg) &raw const v, options(nostack, readonly, preserves_flags));
        }
    }
}

#[cfg(any(miri, not(any(target_arch = "aarch64", target_arch = "x86_64"))))]
mod imp {
    pub type Reg = u8;
    pub const SUPPORTED: bool = false;
    pub const MASK: Reg = 0;

    #[inline]
    pub fn read() -> Reg {
        0
    }

    #[inline]
    pub fn write(_v: Reg) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::hint::black_box;

    /// 运行期算一个非正规数：MIN_POSITIVE / 4。
    fn subnormal() -> f32 {
        black_box(f32::MIN_POSITIVE) * black_box(0.25)
    }

    #[test]
    fn flushes_inside_guard_only() {
        assert!(!active(), "测试线程起始不该开着 FTZ");
        let before = subnormal();
        assert!(before.is_subnormal(), "守卫外不冲：{before:e}");
        {
            let _g = FtzGuard::new();
            assert_eq!(active(), SUPPORTED);
            let inside = subnormal();
            if SUPPORTED {
                assert_eq!(inside, 0.0, "守卫内应冲成 0：{inside:e}");
            }
        }
        assert!(!active(), "出作用域应恢复");
        assert!(subnormal().is_subnormal());
    }

    #[test]
    fn nested_guards_restore_outer_state() {
        let outer = FtzGuard::new();
        {
            let _inner = FtzGuard::new();
            assert_eq!(active(), SUPPORTED);
        }
        // 内层恢复到「外层已开」的状态，不是关掉
        assert_eq!(active(), SUPPORTED);
        drop(outer);
        assert!(!active());
    }

    #[test]
    fn guard_does_not_touch_normal_numbers() {
        let _g = FtzGuard::new();
        let x = black_box(1.5f32) * black_box(2.0);
        assert_eq!(x, 3.0);
        assert_eq!(black_box(f32::MIN_POSITIVE) * black_box(1.0), f32::MIN_POSITIVE);
    }
}
