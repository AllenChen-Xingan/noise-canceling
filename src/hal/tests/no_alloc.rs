//! 证明 `HalSource::render` 不分配（含欠载补零路径）。
#![cfg(debug_assertions)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_hal::{FnSource, HalSource, RenderStatus};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn render_does_not_allocate() {
    let mut left = 300usize;
    let mut h = HalSource::new(FnSource(move |o: &mut [f32]| {
        let n = o.len().min(left).min(100); // 分段给，模拟环绕回
        o[..n].fill(0.25);
        left -= n;
        n
    }));
    let mut out = vec![0.0f32; 256];
    let (a, b) = assert_no_alloc(|| (h.render(&mut out), h.render(&mut out)));
    assert_eq!(a, RenderStatus::Full);
    assert_eq!(b, RenderStatus::Underrun { got: 44 });
}
