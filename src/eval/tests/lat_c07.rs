//! #109 回归：exp014 里 phone-booth-c07 两段，原声直通被量出多 106 样本（人声 1.9 s 才出现，
//! 旧公式只看开头 2 s 且不归一化）。用这两段的 mix.wav 当原声直通输出，必须量出 0。
//! 音频不进仓（在主目录 `_tmp/exp013/<build>/`），所以标 ignore，人工或验收时跑：
//! `cargo test -p vdn-eval --test lat_c07 -- --ignored`

use vdn_eval::analysis::{busiest_start, xcorr_lag};

const BUILD: &str = "_tmp/exp013/20261103230756-17f5";

fn lag_of(clip: &str) -> Option<usize> {
    let dir = vdn_eval::suite::main_root().join(BUILD).join(clip);
    let (mix, sr) = vdn_eval::wav::read(&dir.join("mix.wav")).expect("读 mix.wav");
    let (owner, _) = vdn_eval::wav::read(&dir.join("owner.wav")).expect("读 owner.wav");
    let win = 4 * sr as usize;
    let start = busiest_start(&owner, win, sr as usize / 10);
    xcorr_lag(&mix, &mix, 2048, start, win)
}

#[test]
#[ignore = "要主目录 _tmp/exp013 的场景音频（不进仓），人工跑"]
fn phone_booth_c07_passthrough_is_zero() {
    for clip in ["phone-booth-c07-ext", "phone-booth-c07-mba"] {
        assert_eq!(lag_of(clip), Some(0), "{clip}");
    }
}
