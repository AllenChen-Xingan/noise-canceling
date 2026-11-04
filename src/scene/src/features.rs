//! 便宜的场景特征。两段：
//! - **逐帧**（每 16 ms 一次，喂带噪输入），两个入口只差「功率谱从哪来」，之后同一段代码：
//!   - [`FeatureExtractor::push_hop_with_power`]：借降噪模型算好的功率谱（`SpectrumTap::input_power`），不做正向 FFT
//!   - [`FeatureExtractor::push_hop`]：自算一次（拿不到谱的模型，如 FE-M），口径同 `vdn_types::FRONTEND`
//!
//!   另有一次逆 FFT（功率谱 → 自相关求谐波度），其余都是 257 个频点上的加法，结果存进 2 s 环形缓冲。不分配
//! - **每次判断**（[`FeatureExtractor::compute`]，约 0.5 s 一次）：在 2 s 缓冲上算分位数、调制谱等，得 [`NF`] 维特征。不分配
//!
//! 特征表（下标即 [`FeatureExtractor::compute`] 输出的位置）：
//! | # | 名字 | 想抓什么 |
//! |---|---|---|
//! | 0 | snr_est | 帧能量 p90 − p10（dB）：粗信噪比 |
//! | 1 | floor_db | 帧能量 p10（dBFS）：底噪绝对电平（安静） |
//! | 2 | mid_spread | p50 − p10 |
//! | 3 | flat_lo | 低能量帧的谱平坦度（稳态噪音 → 高） |
//! | 4 | flat_all | 全部帧的谱平坦度 |
//! | 5 | flux_mean | 频带能量的正向谱通量（dB/帧） |
//! | 6 | flux_peaky | log(通量 p90 / 均值)：瞬态（杯碟、键盘） |
//! | 7 | hf_mean | 4–8 kHz 对 1–4 kHz 能量比（dB）均值 |
//! | 8 | hf_p90 | 同上 p90：窄带蓝牙时连齿音也没有高频 |
//! | 9 | low_ratio | 300 Hz 以下能量占比（dB）：交通、空调的低频轰鸣 |
//! | 10 | centroid | 谱质心（kHz） |
//! | 11 | harm_frac | 谐波帧占比（自相关峰 > 0.45） |
//! | 12 | harm_mean | 自相关峰均值 |
//! | 13 | pitch_stab | 相邻谐波帧基音周期几乎不变的比例（音乐长音 → 高） |
//! | 14 | peak_persist | 谱峰帧间保持率（音乐 → 高） |
//! | 15–17 | mod_lo / mod_4 / mod_hi | 帧能量包络的调制谱：< 1.5 Hz / 2–8 Hz（音节）/ > 10 Hz 的占比 |
//! | 18 | e_std | 帧能量标准差（调制深度：单人说话深、多人 babble 浅） |
//! | 19 | band_stat | 低能量帧里各频带随时间的标准差均值（稳态度） |
//! | 20 | decay | 能量下降沿最陡的 10%（dB/s）：混响越大下降越慢 |
//! | 21 | clip | log10(平顶样本占比)：削波 |
//! | 22 | slope | 频带能量对 log 频率的斜率（dB/倍频程） |
//! | 23–26 | bmod_1..4 | 四组频带各自包络的 2–8 Hz 调制占比（人声在中频最强） |
//! | 27–34 | floor_1..8 | 底噪谱形：8 组频带（每组 2 个）各自时间 p20 的 dB，减去 8 组均值（与电平无关，只看形状） |

use std::sync::Arc;

use realfft::num_complex::Complex32;
use realfft::{ComplexToReal, RealFftPlanner};
use vdn_dsp::{StreamingStft, power_into};
use vdn_types::{FRONTEND, MODEL_RATE, StftSpec, Window};

pub const NF: usize = 35;
/// 环形缓冲长度（帧）：125 × 16 ms = 2 s
pub const CTX: usize = 125;
const NB: usize = 16;
const N_FFT: usize = FRONTEND.n_fft;
const BINS: usize = FRONTEND.bins();
// 分类器权重（weights.rs）按 512 点 / 帧移 256 / 16 kHz / 周期 sqrt-Hann 训练：正本任一项一改，这里编译不过，提醒重训
const _: () = assert!(
    FRONTEND.n_fft == 512
        && FRONTEND.hop == 256
        && FRONTEND.sample_rate == MODEL_RATE
        && matches!(FRONTEND.window, Window::SqrtHannPeriodic)
);

pub const FEATURE_NAMES: [&str; NF] = [
    "snr_est",
    "floor_db",
    "mid_spread",
    "flat_lo",
    "flat_all",
    "flux_mean",
    "flux_peaky",
    "hf_mean",
    "hf_p90",
    "low_ratio",
    "centroid",
    "harm_frac",
    "harm_mean",
    "pitch_stab",
    "peak_persist",
    "mod_lo",
    "mod_4",
    "mod_hi",
    "e_std",
    "band_stat",
    "decay",
    "clip",
    "slope",
    "bmod_1",
    "bmod_2",
    "bmod_3",
    "bmod_4",
    "floor_1",
    "floor_2",
    "floor_3",
    "floor_4",
    "floor_5",
    "floor_6",
    "floor_7",
    "floor_8",
];

#[derive(Clone, Copy, Default)]
struct Frame {
    e_db: f32,
    band: [f32; NB],
    flat: f32,
    flux: f32,
    hf: f32,
    low: f32,
    centroid: f32,
    harm: f32,
    lag: u16,
    persist: f32,
    clip: f32,
}

pub struct FeatureExtractor {
    hop: usize,
    sr: f32,
    /// 自算入口用的 STFT（借谱时只推进缓冲，不做 FFT）
    stft: StreamingStft,
    /// 逆 FFT 的输入（功率谱当实部）
    spec: Vec<Complex32>,
    pow: Vec<f32>,
    acorr: Vec<f32>,
    scratch_i: Vec<Complex32>,
    inv: Arc<dyn ComplexToReal<f32>>,
    band_edges: [usize; NB + 1],
    band_hz: [f32; NB],
    prev_band: [f32; NB],
    prev_peak: Vec<bool>,
    cur_peak: Vec<bool>,
    ring: Vec<Frame>,
    head: usize,
    filled: usize,
    prev_sample: f32,
    // compute() 用的暂存
    tmp: Vec<f32>,
    tmp2: Vec<f32>,
}

impl FeatureExtractor {
    /// 只支持 16 kHz、hop 256（与 GTCRN 和 p2 的 VAD 帧对齐）：缓冲长度、调制谱频率、分类器权重都按它训练。
    /// 换 hop（如 FE-M 的 160）必须按时长重算帧数并重训，所以这里直接拦下（自审 P2）。
    pub fn new(sr: f32, hop: usize) -> Self {
        assert!(
            hop == FRONTEND.hop && (sr - FRONTEND.sample_rate as f32).abs() < 1.0,
            "vdn-scene 只支持 FRONTEND 的采样率 {} Hz / hop {}，收到 {sr} Hz / hop {hop}",
            FRONTEND.sample_rate,
            FRONTEND.hop
        );
        let inv = RealFftPlanner::<f32>::new().plan_fft_inverse(N_FFT);
        // 16 个对数间隔频带，100 Hz – 8 kHz
        let mut band_edges = [0usize; NB + 1];
        let mut band_hz = [0f32; NB];
        let (lo, hi) = (100f32, sr / 2.0);
        for (b, e) in band_edges.iter_mut().enumerate() {
            let f = lo * (hi / lo).powf(b as f32 / NB as f32);
            *e = ((f / sr * N_FFT as f32).round() as usize).clamp(1, BINS - 1);
        }
        for b in 0..NB {
            if band_edges[b + 1] <= band_edges[b] {
                band_edges[b + 1] = band_edges[b] + 1;
            }
            band_hz[b] = lo * (hi / lo).powf((b as f32 + 0.5) / NB as f32);
        }
        Self {
            hop,
            sr,
            stft: StreamingStft::from_spec(&FRONTEND),
            spec: inv.make_input_vec(),
            pow: vec![0.0; BINS],
            acorr: vec![0.0; N_FFT],
            scratch_i: inv.make_scratch_vec(),
            inv,
            band_edges,
            band_hz,
            prev_band: [-100.0; NB],
            prev_peak: vec![false; BINS],
            cur_peak: vec![false; BINS],
            ring: vec![Frame::default(); CTX],
            head: 0,
            filled: 0,
            prev_sample: 0.0,
            tmp: vec![0.0; CTX],
            tmp2: vec![0.0; CTX],
        }
    }

    pub fn hop(&self) -> usize {
        self.hop
    }

    /// 缓冲里已有多少帧（< CTX 时特征还不稳）。
    pub fn filled(&self) -> usize {
        self.filled
    }

    /// 自算入口做过几次正向 FFT（测试用：借谱时应为 0）。
    pub fn own_fft_count(&self) -> u64 {
        self.stft.forward_count()
    }

    pub fn reset(&mut self) {
        self.stft.reset();
        self.prev_band = [-100.0; NB];
        self.prev_peak.fill(false);
        self.head = 0;
        self.filled = 0;
        self.prev_sample = 0.0;
    }

    /// 自算：喂一个 hop 的带噪输入（判断用的是麦克风原声，不是降噪后的声音），自己做一次正向 FFT。
    /// `x.len()` 应等于 `hop()`。
    pub fn push_hop(&mut self, x: &[f32]) {
        let spec = self.stft.analyze(x);
        power_into(spec, &mut self.pow);
        self.finish_frame(x);
    }

    /// 借谱：`power` 是同一帧带噪输入的功率谱（`vdn_types::SpectrumTap::input_power`），`spec` 是借出方的口径
    /// （`SpectrumTap::spectrum_spec`）。口径不等于 `FRONTEND` 或长度不对时退回自算——分类器权重只认正本口径；
    /// 退回可由 `own_fft_count` 看到。
    /// `x` 仍要：削波按时域算；并推进自算缓冲，切回自算（换成 FE-M）时第一帧不吃旧数据。
    pub fn push_hop_with_power(&mut self, x: &[f32], spec: &StftSpec, power: &[f32]) {
        if *spec != FRONTEND || power.len() != BINS {
            self.push_hop(x);
            return;
        }
        self.stft.feed(x);
        self.pow.copy_from_slice(power);
        self.finish_frame(x);
    }

    /// 两个入口共用：`self.pow` 已是本帧功率谱（未加下限），算一帧的量存进环形缓冲。
    fn finish_frame(&mut self, x: &[f32]) {
        let h = self.hop.min(x.len());
        // 削波：平顶样本（相邻两点几乎相等且幅度不小）
        let mut flat_top = 0usize;
        let mut prev = self.prev_sample;
        for &v in &x[..h] {
            if v.abs() > 0.3 && (v - prev).abs() < 1e-5 {
                flat_top += 1;
            }
            prev = v;
        }
        self.prev_sample = prev;

        let mut total = 0.0f32;
        let (mut e_hf, mut e_mf, mut e_low, mut cen_num) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let (mut log_sum, mut lin_sum, mut cnt) = (0.0f32, 0.0f32, 0usize);
        let hz = self.sr / N_FFT as f32;
        for (k, p) in self.pow.iter_mut().enumerate() {
            *p += 1e-12;
            let f = k as f32 * hz;
            total += *p;
            cen_num += *p * f;
            if f < 300.0 {
                e_low += *p;
            } else if (1000.0..4000.0).contains(&f) {
                e_mf += *p;
            } else if f >= 4000.0 {
                e_hf += *p;
            }
            if f >= 100.0 {
                log_sum += p.ln();
                lin_sum += *p;
                cnt += 1;
            }
        }
        let e_db = 10.0 * (total / (N_FFT as f32 * N_FFT as f32)).max(1e-12).log10();
        let flat = if cnt > 0 {
            (log_sum / cnt as f32).exp() / (lin_sum / cnt as f32)
        } else {
            0.0
        };
        // 频带能量与正向通量
        let mut band = [0f32; NB];
        let mut flux = 0.0;
        for (b, (bd, pb)) in band.iter_mut().zip(&self.prev_band).enumerate() {
            let s: f32 = self.pow[self.band_edges[b]..self.band_edges[b + 1]]
                .iter()
                .sum();
            *bd = 10.0 * s.max(1e-12).log10();
            flux += (*bd - pb).max(0.0);
        }
        self.prev_band = band;
        // 谐波度：功率谱逆变换 = 自相关；在 40–200 样本（80–400 Hz）里找归一化峰
        for (c, p) in self.spec.iter_mut().zip(&self.pow) {
            *c = Complex32::new(*p, 0.0);
        }
        let (mut harm, mut lag) = (0.0f32, 0u16);
        if self
            .inv
            .process_with_scratch(&mut self.spec, &mut self.acorr, &mut self.scratch_i)
            .is_ok()
            && self.acorr[0] > 0.0
        {
            let r0 = self.acorr[0];
            for (l, &r) in self.acorr.iter().enumerate().take(201).skip(40) {
                let v = r / r0;
                if v > harm {
                    harm = v;
                    lag = l as u16;
                }
            }
        }
        // 谱峰保持率（200–4000 Hz，高出两侧 6 dB 的局部极大）
        let (k0, k1) = ((200.0 / hz) as usize, (4000.0 / hz) as usize);
        let (mut peaks, mut kept) = (0usize, 0usize);
        for k in 0..BINS {
            self.cur_peak[k] = false;
        }
        for k in k0.max(2)..k1.min(BINS - 2) {
            let p = self.pow[k];
            if p > 4.0 * self.pow[k - 2]
                && p > 4.0 * self.pow[k + 2]
                && p >= self.pow[k - 1]
                && p >= self.pow[k + 1]
            {
                self.cur_peak[k] = true;
                peaks += 1;
                if self.prev_peak[k - 1] || self.prev_peak[k] || self.prev_peak[k + 1] {
                    kept += 1;
                }
            }
        }
        std::mem::swap(&mut self.prev_peak, &mut self.cur_peak);
        let fr = Frame {
            e_db,
            band,
            flat,
            flux,
            hf: 10.0 * (e_hf / e_mf.max(1e-12)).max(1e-12).log10(),
            low: 10.0 * (e_low / total).max(1e-12).log10(),
            centroid: cen_num / total / 1000.0,
            harm,
            lag,
            persist: if peaks > 0 {
                kept as f32 / peaks as f32
            } else {
                0.0
            },
            clip: flat_top as f32 / h.max(1) as f32,
        };
        self.ring[self.head] = fr;
        self.head = (self.head + 1) % CTX;
        self.filled = (self.filled + 1).min(CTX);
    }

    /// 第 i 旧的帧（0 = 最旧）。
    #[inline]
    fn at(&self, i: usize) -> &Frame {
        let start = (self.head + CTX - self.filled) % CTX;
        &self.ring[(start + i) % CTX]
    }

    /// 在缓冲上算 [`NF`] 维特征。缓冲不足 8 帧时返回全零。
    pub fn compute(&mut self, out: &mut [f32; NF]) {
        out.fill(0.0);
        let n = self.filled;
        if n < 8 {
            return;
        }
        let nf = n as f32;
        // 帧能量分位数
        for i in 0..n {
            self.tmp[i] = self.at(i).e_db;
        }
        let (p10, p50, p90) = {
            let s = &mut self.tmp[..n];
            s.sort_unstable_by(f32::total_cmp);
            (s[n / 10], s[n / 2], s[(n * 9) / 10])
        };
        out[0] = p90 - p10;
        out[1] = p10;
        out[2] = p50 - p10;
        // 平坦度、通量、高频、低频、质心、谐波、保持率、削波
        let (mut flat_lo, mut nlo, mut flat_all, mut flux_m, mut hf_m, mut low_m, mut cen_m) =
            (0.0f32, 0usize, 0.0, 0.0, 0.0, 0.0, 0.0);
        let (mut hcount, mut hsum, mut stable, mut pairs, mut persist, mut clip) =
            (0usize, 0.0f32, 0usize, 0usize, 0.0f32, 0.0f32);
        let mut prev_h: Option<u16> = None;
        for i in 0..n {
            let f = *self.at(i);
            flat_all += f.flat;
            if f.e_db < p50 {
                flat_lo += f.flat;
                nlo += 1;
            }
            flux_m += f.flux;
            self.tmp2[i] = f.flux;
            hf_m += f.hf;
            low_m += f.low;
            cen_m += f.centroid;
            hsum += f.harm;
            persist += f.persist;
            clip += f.clip;
            if f.harm > 0.45 {
                hcount += 1;
                if let Some(pl) = prev_h {
                    pairs += 1;
                    if (pl as i32 - f.lag as i32).abs() <= 1 {
                        stable += 1;
                    }
                }
                prev_h = Some(f.lag);
            } else {
                prev_h = None;
            }
        }
        out[3] = if nlo > 0 { flat_lo / nlo as f32 } else { 0.0 };
        out[4] = flat_all / nf;
        out[5] = flux_m / nf;
        {
            let s = &mut self.tmp2[..n];
            s.sort_unstable_by(f32::total_cmp);
            out[6] = ((s[(n * 9) / 10] + 0.1) / (out[5] + 0.1)).ln();
        }
        out[7] = hf_m / nf;
        for i in 0..n {
            self.tmp[i] = self.at(i).hf;
        }
        {
            let s = &mut self.tmp[..n];
            s.sort_unstable_by(f32::total_cmp);
            out[8] = s[(n * 9) / 10];
        }
        out[9] = low_m / nf;
        out[10] = cen_m / nf;
        out[11] = hcount as f32 / nf;
        out[12] = hsum / nf;
        out[13] = if pairs > 0 {
            stable as f32 / pairs as f32
        } else {
            0.0
        };
        out[14] = persist / nf;
        // 调制谱：帧率 = sr / hop；对去均值的包络在若干调制频率上做 DFT（直接求和，n ≤ 125）
        let fps = self.sr / self.hop as f32;
        for i in 0..n {
            self.tmp[i] = self.at(i).e_db;
        }
        let (lo, m4, hi, std) = mod_split(&mut self.tmp[..n], fps);
        out[15] = lo;
        out[16] = m4;
        out[17] = hi;
        out[18] = std;
        // 稳态度：低能量帧里各频带的时间标准差
        let mut bs = 0.0f32;
        for b in 0..NB {
            let (mut s, mut s2, mut c) = (0.0f32, 0.0f32, 0usize);
            for i in 0..n {
                let f = self.at(i);
                if f.e_db < p50 {
                    s += f.band[b];
                    s2 += f.band[b] * f.band[b];
                    c += 1;
                }
            }
            if c > 1 {
                let m = s / c as f32;
                bs += (s2 / c as f32 - m * m).max(0.0).sqrt();
            }
        }
        out[19] = bs / NB as f32;
        // 衰减：两帧差分最陡的 10%（dB/s）
        if n > 3 {
            for i in 2..n {
                self.tmp2[i - 2] = self.at(i).e_db - self.at(i - 2).e_db;
            }
            let m = n - 2;
            let s = &mut self.tmp2[..m];
            s.sort_unstable_by(f32::total_cmp);
            out[20] = s[m / 10] * fps / 2.0;
        }
        out[21] = (clip / nf + 1e-4).log10();
        // 频谱斜率：各带平均 dB 对 log2(频率) 的回归
        let (mut sx, mut sy, mut sxx, mut sxy) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for b in 0..NB {
            let mut m = 0.0;
            for i in 0..n {
                m += self.at(i).band[b];
            }
            m /= nf;
            let x = self.band_hz[b].log2();
            sx += x;
            sy += m;
            sxx += x * x;
            sxy += x * m;
        }
        let nbf = NB as f32;
        out[22] = (nbf * sxy - sx * sy) / (nbf * sxx - sx * sx).max(1e-6);
        // 四组频带各自的 2–8 Hz 调制占比
        for g in 0..4 {
            for i in 0..n {
                let f = self.at(i);
                let mut s = 0.0f32;
                for b in g * 4..g * 4 + 4 {
                    s += 10f32.powf(f.band[b] / 10.0);
                }
                self.tmp[i] = 10.0 * s.max(1e-12).log10();
            }
            out[23 + g] = mod_split(&mut self.tmp[..n], fps).1;
        }
        // 底噪谱形
        let mut fl = [0.0f32; 8];
        for (g, fg) in fl.iter_mut().enumerate() {
            for i in 0..n {
                let f = self.at(i);
                let s = 10f32.powf(f.band[2 * g] / 10.0) + 10f32.powf(f.band[2 * g + 1] / 10.0);
                self.tmp[i] = 10.0 * s.max(1e-12).log10();
            }
            let s = &mut self.tmp[..n];
            s.sort_unstable_by(f32::total_cmp);
            *fg = s[n / 5];
        }
        let m = fl.iter().sum::<f32>() / 8.0;
        for g in 0..8 {
            out[27 + g] = fl[g] - m;
        }
        for v in out.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
    }
}

/// 包络调制谱：返回 (<1.5 Hz 占比, 2–8 Hz 占比, >10 Hz 占比, 标准差)。直接 DFT，O(n²)，n ≤ 125，每次判断约 1.6 万次乘加。
fn mod_split(env: &mut [f32], fps: f32) -> (f32, f32, f32, f32) {
    let n = env.len();
    let mean = env.iter().sum::<f32>() / n as f32;
    let mut var = 0.0;
    for v in env.iter_mut() {
        *v -= mean;
        var += *v * *v;
    }
    let std = (var / n as f32).sqrt();
    let (mut lo, mut mid, mut hi, mut tot) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for k in 1..=n / 2 {
        let f = k as f32 * fps / n as f32;
        let w = std::f32::consts::TAU * k as f32 / n as f32;
        let (mut re, mut im) = (0.0f32, 0.0f32);
        for (i, v) in env.iter().enumerate() {
            let a = w * i as f32;
            re += v * a.cos();
            im -= v * a.sin();
        }
        let p = re * re + im * im;
        tot += p;
        if f < 1.5 {
            lo += p;
        } else if (2.0..=8.0).contains(&f) {
            mid += p;
        } else if f > 10.0 {
            hi += p;
        }
    }
    let t = tot.max(1e-9);
    (lo / t, mid / t, hi / t, std)
}
