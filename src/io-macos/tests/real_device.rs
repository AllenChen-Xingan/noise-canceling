//! 开真设备的测试：会弹麦克风权限框或出声，默认不跑。人工跑：
//! `cargo test -p vdn-io-macos --test real_device -- --ignored --nocapture`
//!
//! `default_mic_delivers_frames` 跑的时候要有环境声或有人说话：它还判「0.5 s 内不是全零」
//! （全零 = 疑似终端没有麦克风权限：系统设置 › 隐私与安全性 › 麦克风）。

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use vdn_io_macos::{IO_RATE, InputStream, OutputStream, fmt_dbfs};

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
    let level = s.health().level();
    drop(s);
    println!(
        "0.5 s 收到 {got} 帧 · 峰值 {} · 均方根 {}",
        fmt_dbfs(level.peak_dbfs()),
        fmt_dbfs(level.rms_dbfs())
    );
    assert!(got > IO_RATE as usize / 4, "0.5 s 只收到 {got} 帧");
    assert!(
        !level.all_zero(),
        "0.5 s 收到的全是 0：疑似没有麦克风权限（系统设置 › 隐私与安全性 › 麦克风）或设备静音"
    );
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
    // 送出去的是 0.1 振幅正弦：峰值约 −20 dBFS（只证明回调写进了设备缓冲，响不响仍由人耳判）
    let l = s.health().level();
    assert!((l.peak_dbfs() + 20.0).abs() < 0.5, "{l:?}");
}
