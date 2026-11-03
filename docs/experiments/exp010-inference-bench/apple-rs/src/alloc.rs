//! 两种分配计数。
//! - `rust_allocs`：Rust 全局分配器次数，和 rustbe 的 `rust_allocs_per_frame` 同一个量。
//! - `process_allocs`：全进程 malloc 次数（所有线程、所有语言），挂 libmalloc 的 `malloc_logger`
//!   （Instruments 的 MallocStackLogging 用的同一个钩子）。看不到别的进程（如神经引擎守护进程）里的分配。
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

pub struct Counting;
static RUST: AtomicU64 = AtomicU64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        RUST.fetch_add(1, Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        RUST.fetch_add(1, Relaxed);
        unsafe { System.realloc(p, l, n) }
    }
}

type Logger = unsafe extern "C" fn(u32, usize, usize, usize, usize, u32);
unsafe extern "C" {
    static mut malloc_logger: Option<Logger>;
}
static PROC: AtomicU64 = AtomicU64::new(0);
const LOG_ALLOCATE: u32 = 2; // MALLOC_LOG_TYPE_ALLOCATE

unsafe extern "C" fn hook(ty: u32, _: usize, _: usize, _: usize, _: usize, _: u32) {
    if ty & LOG_ALLOCATE != 0 {
        PROC.fetch_add(1, Relaxed);
    }
}

pub fn install_process_hook() {
    unsafe { malloc_logger = Some(hook) };
}
pub fn rust_allocs() -> u64 {
    RUST.load(Relaxed)
}
pub fn process_allocs() -> u64 {
    PROC.load(Relaxed)
}
