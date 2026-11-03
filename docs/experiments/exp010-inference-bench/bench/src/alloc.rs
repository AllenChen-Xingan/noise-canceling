//! 分配计数：Rust 全局分配器计数 + 全进程 malloc 计数（malloc_logger 钩子）。

use crate::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::AtomicU64;

// ── 分配计数 ──
pub(crate) struct Counting;
static ALLOCS: AtomicU64 = AtomicU64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        unsafe { System.realloc(p, l, n) }
    }
}
#[global_allocator]
static A: Counting = Counting;

// ── 全进程 malloc 计数（含 C / C++ / 系统框架）：挂 libmalloc 的 malloc_logger 钩子 ──
// 只在 `--count-malloc` 时挂上；钩子会让每次 malloc 多走一次回调，计时会略偏慢，所以和计时分开跑。
type MallocLogger = unsafe extern "C" fn(u32, usize, usize, usize, usize, u32);
unsafe extern "C" {
    static mut malloc_logger: Option<MallocLogger>;
}
static PROC_ALLOCS: AtomicU64 = AtomicU64::new(0);
pub(crate) const MALLOC_LOG_TYPE_ALLOCATE: u32 = 2;
// 调用栈采样：STACK_ARMED 为真时，把接下来最多 STACK_N 次分配的返回地址存进静态表（backtrace 不分配；
// 用 IN_HOOK 防重入）。只给 amx-probe 查「Accelerate 在哪个调用里分配」用。
pub(crate) static STACK_ARMED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static IN_HOOK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(crate) const STACK_N: usize = 4;
pub(crate) const STACK_DEPTH: usize = 16;
pub(crate) static mut STACKS: [[usize; STACK_DEPTH]; STACK_N] = [[0; STACK_DEPTH]; STACK_N];
pub(crate) static STACK_COUNT: AtomicU64 = AtomicU64::new(0);
unsafe extern "C" {
    pub(crate) fn backtrace(buf: *mut *mut libc::c_void, size: libc::c_int) -> libc::c_int;
}
unsafe extern "C" fn count_malloc(ty: u32, _: usize, _: usize, _: usize, _: usize, _: u32) {
    if ty & MALLOC_LOG_TYPE_ALLOCATE != 0 {
        PROC_ALLOCS.fetch_add(1, Relaxed);
        if STACK_ARMED.load(Relaxed) && !IN_HOOK.swap(true, Relaxed) {
            let i = STACK_COUNT.load(Relaxed) as usize;
            if i < STACK_N {
                unsafe {
                    let slot = &mut (*std::ptr::addr_of_mut!(STACKS))[i];
                    backtrace(slot.as_mut_ptr().cast(), STACK_DEPTH as libc::c_int);
                }
                STACK_COUNT.fetch_add(1, Relaxed);
            }
            IN_HOOK.store(false, Relaxed);
        }
    }
}
pub(crate) fn proc_allocs() -> u64 {
    PROC_ALLOCS.load(Relaxed)
}
pub(crate) fn install_malloc_counter() {
    unsafe { malloc_logger = Some(count_malloc) };
}
