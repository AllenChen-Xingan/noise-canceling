//! 停止标志的守卫：`chain-test` 与 `vdn run` 共用（原来两份各写一个，issue #62 合成一处）。

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 离开作用域时把这几个停止标志都置位：任何返回路径（含某个线程 panic、`?` 提前返回）都让后台线程停下。
/// 用 `Release`：读方用 `Relaxed` / `Acquire` 都能看到。
pub struct StopOnDrop<const N: usize>(pub [Arc<AtomicBool>; N]);

impl<const N: usize> Drop for StopOnDrop<N> {
    fn drop(&mut self) {
        for f in &self.0 {
            f.store(true, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sets_every_flag_on_drop() {
        let (a, b) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));
        {
            let _g = StopOnDrop([a.clone(), b.clone()]);
            assert!(!a.load(Ordering::Acquire) && !b.load(Ordering::Acquire));
        }
        assert!(a.load(Ordering::Acquire) && b.load(Ordering::Acquire));
    }

    #[test]
    fn sets_flag_on_early_return() {
        fn early(f: Arc<AtomicBool>) -> Result<(), &'static str> {
            let _g = StopOnDrop([f]);
            Err("提前返回")
        }
        let f = Arc::new(AtomicBool::new(false));
        assert!(early(f.clone()).is_err());
        assert!(f.load(Ordering::Acquire));
    }
}
