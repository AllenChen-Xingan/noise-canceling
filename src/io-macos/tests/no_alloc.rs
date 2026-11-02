//! 证明回调路径不分配：在 assert_no_alloc 里喂输入泵 / 输出泵，分配就 panic。
// AllocDisabler 只在 debug_assertions 下存在：release 测试跳过这个文件
#![cfg(debug_assertions)]

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use vdn_io_macos::pump::{InputPump, OutputPump};

#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[test]
fn pump_callbacks_do_not_allocate() {
    let mut sum = 0.0f32;
    let mut inp = InputPump::new(128, 2, move |m: &[f32]| sum += m.iter().sum::<f32>());
    let mut out = OutputPump::new(128, 2, |m: &mut [f32]| m.fill(0.5));
    let data_in = vec![0.25f32; 2 * 1000]; // 1000 帧立体声，比块长大，会切段
    let mut data_out = vec![0.0f32; 2 * 1000];
    assert_no_alloc(|| {
        inp.on_data(&data_in);
        out.on_data(&mut data_out);
    });
    assert!(data_out.iter().all(|&s| s == 0.5));
}
