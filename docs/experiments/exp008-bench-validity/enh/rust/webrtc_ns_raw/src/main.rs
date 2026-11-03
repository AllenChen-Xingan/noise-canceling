//! stdin: f32le mono 16 kHz in [-1, 1]. stdout: f32le, padded to whole 10 ms frames + one
//! extra frame (raw output, no trimming; the caller compensates delay).
//! Mirrors sonora's capture path for one band: analyze(frame) then process(frame).
use sonora_ns::config::{NS_FRAME_SIZE, SuppressionLevel};
use sonora_ns::noise_suppressor::NoiseSuppressor;
use std::io::{Read, Write};

fn main() {
    let level = match std::env::args().nth(1).as_deref() {
        Some("6") => SuppressionLevel::K6dB,
        Some("12") => SuppressionLevel::K12dB,
        Some("18") => SuppressionLevel::K18dB,
        Some("21") => SuppressionLevel::K21dB,
        a => panic!("level must be 6|12|18|21, got {a:?}"),
    };
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).expect("read stdin");
    let mut x: Vec<f32> = buf
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]) * 32768.0)
        .collect();
    let n = NS_FRAME_SIZE;
    let total = (x.len() + n).div_ceil(n) * n;
    x.resize(total, 0.0);
    let mut ns = NoiseSuppressor::with_level(level);
    let mut w = std::io::BufWriter::new(std::io::stdout().lock());
    let mut frame = [0.0f32; NS_FRAME_SIZE];
    for chunk in x.chunks_exact(n) {
        frame.copy_from_slice(chunk);
        ns.analyze(&frame);
        ns.process(&mut frame);
        for v in &frame {
            w.write_all(&(v / 32768.0).to_le_bytes()).unwrap();
        }
    }
    w.flush().unwrap();
}
