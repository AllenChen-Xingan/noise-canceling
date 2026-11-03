//! 研究 020：前后处理（48↔16 kHz 重采样 · STFT / iSTFT）每帧计时与下界（子命令 `dsp-bench`）。
//!
//! 被测的正本是产品代码 `vdn-dsp`（`src/dsp`，本线不改）。这里另写一份「只改点积求和方式」的对照：
//! 同一组 FIR 系数（从 `vdn-dsp` 的升采样器冲激响应里取，不另抄系数），同一种多相结构，
//! 点积从「单累加器顺序求和」（浮点不许重排，编译器只能串行：每个输出一条 97 / 33 步的加法依赖链）
//! 换成「4 个 NEON 累加器 × 4 路」，数学上同一个和，只是加法顺序不同，误差有界（测试里与正本比）。
//! 结论若成立，转交「接起来」那件大事改 `src/dsp/src/resample.rs`。

use core::arch::aarch64::*;
use vdn_dsp::resample::{Decimator3, Interpolator3, TAPS};
use vdn_dsp::{StreamingStft, window};

use crate::layers::{tick_hz, ticks};

/// 每帧：48 kHz 768 个样本 → 16 kHz 256 个（与 GTCRN 一帧同长）
pub const HI: usize = 768;
pub const LO: usize = 256;
const PT: usize = TAPS.div_ceil(3); // 升采样每相抽头数

/// 4 × 4 路累加的点积（长度任意；尾巴标量）
#[inline(always)]
pub fn dot_neon(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let (pa, pb) = (a.as_ptr(), b.as_ptr());
    unsafe {
        let mut acc = [vdupq_n_f32(0.0); 4];
        let mut i = 0;
        while i + 16 <= n {
            for (k, s) in acc.iter_mut().enumerate() {
                *s = vfmaq_f32(*s, vld1q_f32(pa.add(i + 4 * k)), vld1q_f32(pb.add(i + 4 * k)));
            }
            i += 16;
        }
        while i + 4 <= n {
            acc[0] = vfmaq_f32(acc[0], vld1q_f32(pa.add(i)), vld1q_f32(pb.add(i)));
            i += 4;
        }
        let mut s = vaddvq_f32(vaddq_f32(vaddq_f32(acc[0], acc[1]), vaddq_f32(acc[2], acc[3])));
        while i < n {
            s = (*pa.add(i)).mul_add(*pb.add(i), s);
            i += 1;
        }
        s
    }
}

/// 从 `vdn-dsp` 升采样器的冲激响应取系数：输入单位冲激，第 n 个输出 = 3·h[n]（多相系数原值，不经换算）
pub fn coeffs_x3() -> Vec<f32> {
    let mut u = Interpolator3::new();
    let lo_n = TAPS.div_ceil(3) + 1;
    let mut x = vec![0.0f32; lo_n];
    x[0] = 1.0;
    let mut y = vec![0.0f32; 3 * lo_n];
    u.process(&x, &mut y);
    y.truncate(TAPS);
    y
}

/// 双份历史（同 vdn-dsp：任一时刻最近 n 个样本是连续切片）
struct Hist {
    buf: Vec<f32>,
    n: usize,
    pos: usize,
}
impl Hist {
    fn new(n: usize) -> Self {
        Self { buf: vec![0.0; 2 * n], n, pos: 0 }
    }
    #[inline(always)]
    fn push(&mut self, x: f32) {
        self.buf[self.pos] = x;
        self.buf[self.pos + self.n] = x;
        self.pos += 1;
        if self.pos == self.n {
            self.pos = 0;
        }
    }
    #[inline(always)]
    fn window(&self) -> &[f32] {
        &self.buf[self.pos + 1..self.pos + 1 + self.n].as_ref()[..0].as_ref().get(..0).map_or(&self.buf[self.pos..self.pos + self.n], |_| &self.buf[self.pos..self.pos + self.n])
    }
}

/// 对照：降采样，结构同 vdn-dsp（只算留下的输出，取每组第 1 个），点积换 `dot_neon`
pub struct FastDec {
    hr: Vec<f32>,
    hist: Hist,
    phase: usize,
}
impl FastDec {
    pub fn new() -> Self {
        let mut hr: Vec<f32> = coeffs_x3().iter().map(|v| v / 3.0).collect();
        hr.reverse();
        Self { hr, hist: Hist::new(TAPS), phase: 0 }
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let mut k = 0;
        for &x in input {
            self.hist.push(x);
            if self.phase == 0 {
                output[k] = dot_neon(&self.hr, self.hist.window());
                k += 1;
            }
            self.phase = (self.phase + 1) % 3;
        }
    }
}

/// 对照：升采样，3 相多相，点积换 `dot_neon`
pub struct FastInt {
    poly: Vec<f32>, // [3][PT]，倒序
    hist: Hist,
}
impl FastInt {
    pub fn new() -> Self {
        let h3 = coeffs_x3();
        let mut poly = vec![0.0f32; 3 * PT];
        for p in 0..3 {
            for j in 0..PT {
                if let Some(&v) = h3.get(p + 3 * j) {
                    poly[p * PT + (PT - 1 - j)] = v;
                }
            }
        }
        Self { poly, hist: Hist::new(PT) }
    }
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (m, &x) in input.iter().enumerate() {
            self.hist.push(x);
            let w = self.hist.window();
            for p in 0..3 {
                output[3 * m + p] = dot_neon(&self.poly[p * PT..(p + 1) * PT], w);
            }
        }
    }
}

/// `dsp-bench <wav> [--frames N] [--warmup N] [--qos rt|ui|bg] [--pace-ms X]`
/// wav 按 16 kHz 读入后用 vdn-dsp 升到 48 kHz 当输入（只为有真实频谱的信号；计时不含这一步）。
pub fn run(args: &[String], audio16: &[f32]) -> Result<(), crate::Err> {
    use crate::{arg_str, arg_usize};
    let frames = arg_usize(args, "--frames", 6000);
    let warm = arg_usize(args, "--warmup", 200);
    let pace: f64 = arg_str(args, "--pace-ms", "0").parse()?;
    let qos = arg_str(args, "--qos", "rt");
    if frames == 0 || audio16.len() < LO || !pace.is_finite() || !(0.0..=10_000.0).contains(&pace) {
        return Err("--frames 要 > 0，音频至少一帧，--pace-ms 在 0..=10000".into());
    }
    crate::set_thread_qos(qos, 16.0)?;
    // 输入：16 kHz 音频循环 → 升到 48 kHz
    let total = warm + frames;
    let mut hi = vec![0.0f32; total * HI];
    let mut up = Interpolator3::new();
    for f in 0..total {
        let s = (f * LO) % (audio16.len() - LO + 1);
        up.process(&audio16[s..s + LO], &mut hi[f * HI..(f + 1) * HI]);
    }
    let (mut dec, mut int) = (Decimator3::new(), Interpolator3::new());
    let (mut fdec, mut fint) = (FastDec::new(), FastInt::new());
    let mut stft = StreamingStft::new(512, LO, window::sqrt_hann_periodic(512));
    let (mut lo, mut lo2, mut out, mut out2) = ([0.0f32; LO], [0.0f32; LO], [0.0f32; HI], [0.0f32; HI]);
    let mut syn = [0.0f32; LO];
    // 每帧 6 段：原降采样 · 快降采样 · STFT 分析 · iSTFT 合成 · 原升采样 · 快升采样
    const S: usize = 6;
    let mut t = vec![0u32; frames * S];
    let (mut dmax, mut umax) = (0.0f32, 0.0f32);
    let w0 = std::time::Instant::now();
    for f in 0..total {
        if pace > 0.0 && f >= warm {
            let due = w0 + std::time::Duration::from_secs_f64((f - warm) as f64 * pace * 1e-3);
            if let Some(d) = due.checked_duration_since(std::time::Instant::now()) {
                std::thread::sleep(d);
            }
        }
        let x = &hi[f * HI..(f + 1) * HI];
        let mut k = [0u64; S + 1];
        k[0] = ticks();
        dec.process(x, &mut lo);
        k[1] = ticks();
        fdec.process(x, &mut lo2);
        k[2] = ticks();
        let spec = stft.analyze(&lo);
        std::hint::black_box(&spec);
        k[3] = ticks();
        stft.synthesize(&mut syn);
        k[4] = ticks();
        int.process(&syn, &mut out);
        k[5] = ticks();
        fint.process(&syn, &mut out2);
        k[6] = ticks();
        if f >= warm {
            for s in 0..S {
                t[(f - warm) * S + s] = (k[s + 1] - k[s]) as u32;
            }
            for (a, b) in lo.iter().zip(&lo2) {
                dmax = dmax.max((a - b).abs());
            }
            for (a, b) in out.iter().zip(&out2) {
                umax = umax.max((a - b).abs());
            }
        }
    }
    let nspt = 1e9 / tick_hz() as f64;
    let names = ["dec_vdn", "dec_neon", "stft_analyze", "istft_synth", "int_vdn", "int_neon"];
    let mut segs = Vec::new();
    for (s, name) in names.iter().enumerate() {
        let mut v: Vec<u32> = (0..frames).map(|f| t[f * S + s]).collect();
        let mean = v.iter().map(|x| *x as f64).sum::<f64>() / frames as f64 * nspt;
        v.sort_unstable();
        segs.push(format!(
            "\"{name}\":{{\"mean_ns\":{mean:.1},\"p50_ns\":{:.1},\"p99_ns\":{:.1}}}",
            v[frames / 2] as f64 * nspt,
            v[frames * 99 / 100] as f64 * nspt
        ));
    }
    println!(
        "{{\"what\":\"dsp\",\"frames\":{frames},\"warmup\":{warm},\"pace_ms\":{pace},\"qos\":\"{qos}\",\"dec_macs\":{},\"int_macs\":{},\"dec_max_abs_diff\":{dmax:.3e},\"int_max_abs_diff\":{umax:.3e},{}}}",
        LO * TAPS,
        HI * PT,
        segs.join(",")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(n: usize) -> Vec<f32> {
        (0..n).map(|i| ((i as f32) * 0.37).sin() * 0.5 + ((i as f32) * 0.011).cos() * 0.3).collect()
    }

    #[test]
    fn dot_neon_matches_f64() {
        for n in [1, 3, 4, 15, 16, 17, 33, 97, 100] {
            let (a, b) = (sig(n), sig(n + 5)[5..].to_vec());
            let r: f64 = a.iter().zip(&b).map(|(x, y)| *x as f64 * *y as f64).sum();
            assert!((dot_neon(&a, &b) as f64 - r).abs() < 1e-5, "n = {n}");
        }
    }

    /// 对照版与产品正本（vdn-dsp）分块喂同一段信号，输出差有界
    #[test]
    fn fast_resamplers_match_vdn_dsp() {
        let x = sig(HI * 20);
        let (mut d, mut fd) = (Decimator3::new(), FastDec::new());
        let (mut u, mut fu) = (Interpolator3::new(), FastInt::new());
        let (mut worst_d, mut worst_u) = (0.0f32, 0.0f32);
        for c in x.chunks(HI) {
            let (mut a, mut b) = ([0.0f32; LO], [0.0f32; LO]);
            d.process(c, &mut a);
            fd.process(c, &mut b);
            worst_d = a.iter().zip(&b).fold(worst_d, |m, (p, q)| m.max((p - q).abs()));
            let (mut ua, mut ub) = ([0.0f32; HI], [0.0f32; HI]);
            u.process(&a, &mut ua);
            fu.process(&a, &mut ub);
            worst_u = ua.iter().zip(&ub).fold(worst_u, |m, (p, q)| m.max((p - q).abs()));
        }
        assert!(worst_d < 1e-5 && worst_u < 1e-5, "降 {worst_d}，升 {worst_u}");
    }

    #[test]
    fn coeffs_sum_to_three() {
        let s: f32 = coeffs_x3().iter().sum();
        assert!((s - 3.0).abs() < 1e-4, "3·h 之和应为 3（直流增益 1）：{s}");
    }
}
