//! 开真设备的测试：会弹麦克风权限框或出声，默认不跑。人工跑：
//! `cargo test -p vdn-io-macos --test real_device -- --ignored --nocapture`

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use vdn_io_macos::{IO_RATE, InputStream, OutputStream};

#[test]
#[ignore = "要真设备，人工跑（会弹麦克风权限框）"]
fn default_mic_delivers_frames() {
    let n = Arc::new(AtomicUsize::new(0));
    let n2 = Arc::clone(&n);
    let s = InputStream::open(None, IO_RATE, 480, move |m: &[f32]| {
        n2.fetch_add(m.len(), Ordering::Relaxed);
    })
    .expect("打开默认麦克风");
    std::thread::sleep(Duration::from_millis(500));
    let got = n.load(Ordering::Relaxed);
    drop(s);
    println!("0.5 s 收到 {got} 帧");
    assert!(got > IO_RATE as usize / 4, "0.5 s 只收到 {got} 帧");
}

#[test]
#[ignore = "要真设备，人工跑（会从扬声器发出 0.5 s 的 440 Hz 小音量正弦）"]
fn default_speaker_plays_sine() {
    let mut ph = 0.0f32;
    let step = 2.0 * std::f32::consts::PI * 440.0 / IO_RATE as f32;
    let s = OutputStream::open(None, IO_RATE, 480, move |m: &mut [f32]| {
        for x in m.iter_mut() {
            *x = 0.1 * ph.sin();
            ph = (ph + step) % (2.0 * std::f32::consts::PI);
        }
    })
    .expect("打开默认扬声器");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(s.health().errors(), 0);
}
