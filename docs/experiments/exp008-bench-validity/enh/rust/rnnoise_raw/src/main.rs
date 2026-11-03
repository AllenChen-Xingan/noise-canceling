//! stdin: f32le mono 48 kHz in [-1, 1]. stdout: f32le, same length + FRAME_SIZE
//! (raw frame-by-frame output, no trimming; the caller compensates delay).
use nnnoiseless::DenoiseState;
use std::io::{Read, Write};

fn main() {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf).expect("read stdin");
    let x: Vec<f32> = buf
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]) * 32768.0)
        .collect();
    let n = DenoiseState::FRAME_SIZE;
    // pad so every input sample gets through plus one extra frame for the internal delay
    let total = (x.len() + n).div_ceil(n) * n;
    let mut xin = x.clone();
    xin.resize(total, 0.0);
    let mut st = DenoiseState::new();
    let mut out = vec![0.0f32; n];
    let mut w = std::io::BufWriter::new(std::io::stdout().lock());
    for chunk in xin.chunks_exact(n) {
        st.process_frame(&mut out, chunk);
        for v in &out {
            w.write_all(&(v / 32768.0).to_le_bytes()).unwrap();
        }
    }
    w.flush().unwrap();
}
