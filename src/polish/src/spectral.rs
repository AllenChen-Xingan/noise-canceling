//! 频谱修整节点：平滑（金属声）· 增益下限 + 舒适噪声（真空感）· 去发闷均衡。
//!
//! 输入：同一帧的带噪频谱 `dry` 与模型输出频谱 `wet`（同一套 STFT）。只用当前帧与过去帧的统计 → 零前瞻、不加延迟。
//!
//! 每帧步骤：
//! 1. 底噪谱 `noise[k]`：对 `|dry|²` 做「快降慢升」的最小值跟踪（降 τ≈0.08 s，升 +3 dB/s）——稳态部分，人声峰带不进来
//! 2. 本帧有没有人声 `sp ∈ [0,1]`：有 VAD（`FrameCtx.voice_prob`）就用；没有就用降噪后帧能量对其自身底噪的后验信噪比粗判
//! 3. 隐含增益 `g = |wet| / |dry|`（封顶 1）。无人声时对 `g` 做时间平滑（系数随 `smooth`）和三点频率平滑，
//!    把孤立跳动的增益尖峰（音乐噪声的来源）抹平；有人声时几乎不平滑，避免拖尾、吃字头
//! 4. 下限：`g' = max(g_s, floor)`；再乘去发闷补偿 `b[k]`，但 `g'·b ≤ 1`（永远不比带噪输入更响）
//! 5. `wet ← wet · g'·b / g`（保留模型的相位）；`g` 极小时用带噪相位
//! 6. 舒适噪声：若 `|wet|² < c²·noise[k]`，补上差额的随机相位分量（只补缺口，不叠加到已有声音上）
//!
//! 去发闷：只在语音帧上，以「带噪 − 底噪」估计干净语音在 2–3 / 3–4.5 / 4.5–6 / 6–8 kHz 四个频带的能量，与降噪后能量比，
//! 慢慢（τ≈3 s）补回一半差距，封顶 `demuffle_max_db`；该频带信噪比估计 < 6 dB 时不补（否则补回的是噪音）。

use realfft::num_complex::Complex32;

use crate::params::PolishParams;
use crate::{db_to_pow, pow_to_db};

const N_EQ: usize = 4;
const EQ_EDGES_HZ: [f32; N_EQ + 1] = [2000.0, 3000.0, 4500.0, 6000.0, 8000.0];

pub struct SpectralPolish {
    bins: usize,
    hz_per_bin: f32,
    noise: Vec<f32>,
    wet_noise: f32,
    g_s: Vec<f32>,
    g_tmp: Vec<f32>,
    boost: Vec<f32>,
    eq_clean: [f32; N_EQ],
    eq_wet: [f32; N_EQ],
    eq_snr: [f32; N_EQ],
    eq_gain_db: [f32; N_EQ],
    frames: u64,
    rng: u64,
    a_down: f32,
    rise: f32,
    a_eq: f32,
    last_sp: f32,
}

impl SpectralPolish {
    /// `n_fft` / `hop` / `sr` 必须与喂进来的频谱一致（GTCRN：512 / 256 / 16000）。
    pub fn new(n_fft: usize, hop: usize, sr: f32) -> Self {
        let bins = n_fft / 2 + 1;
        let fps = sr / hop as f32;
        Self {
            bins,
            hz_per_bin: sr / n_fft as f32,
            noise: vec![0.0; bins],
            wet_noise: 0.0,
            g_s: vec![1.0; bins],
            g_tmp: vec![1.0; bins],
            boost: vec![1.0; bins],
            eq_clean: [0.0; N_EQ],
            eq_wet: [0.0; N_EQ],
            eq_snr: [0.0; N_EQ],
            eq_gain_db: [0.0; N_EQ],
            frames: 0,
            rng: 0x9E37_79B9_7F4A_7C15,
            a_down: 1.0 - (-1.0 / (0.08 * fps)).exp(),
            rise: db_to_pow(3.0 / fps),
            a_eq: 1.0 - (-1.0 / (3.0 * fps)).exp(),
            last_sp: 0.0,
        }
    }

    pub fn bins(&self) -> usize {
        self.bins
    }

    /// 上一帧的人声存在度（AGC 没有外部 VAD 时借用）。
    pub fn last_speech_presence(&self) -> f32 {
        self.last_sp
    }

    /// 当前去发闷补偿（dB，四个频带），给日志与测试用。
    pub fn demuffle_gain_db(&self) -> [f32; N_EQ] {
        self.eq_gain_db
    }

    pub fn reset(&mut self) {
        self.noise.fill(0.0);
        self.wet_noise = 0.0;
        self.g_s.fill(1.0);
        self.boost.fill(1.0);
        self.eq_clean = [0.0; N_EQ];
        self.eq_wet = [0.0; N_EQ];
        self.eq_snr = [0.0; N_EQ];
        self.eq_gain_db = [0.0; N_EQ];
        self.frames = 0;
        self.last_sp = 0.0;
    }

    #[inline]
    fn rand_unit(&mut self) -> f32 {
        // xorshift64*：不分配、确定性
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        let v = self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 40) as f32 / (1u64 << 24) as f32
    }

    /// 处理一帧。`dry.len() == wet.len() == bins()`。
    pub fn process(&mut self, dry: &[Complex32], wet: &mut [Complex32], p: &PolishParams, voice_prob: Option<f32>) {
        let n = self.bins.min(dry.len()).min(wet.len());
        // 1. 底噪跟踪（第一帧直接初始化）
        let mut e_wet = 0.0f32;
        for k in 0..n {
            let px = dry[k].norm_sqr();
            let nk = &mut self.noise[k];
            if self.frames == 0 {
                *nk = px;
            } else if px < *nk {
                *nk += self.a_down * (px - *nk);
            } else {
                *nk = (*nk * self.rise).min(px);
            }
            e_wet += wet[k].norm_sqr();
        }
        // 2. 人声存在度
        if self.frames == 0 {
            self.wet_noise = e_wet;
        } else if e_wet < self.wet_noise {
            self.wet_noise += self.a_down * (e_wet - self.wet_noise);
        } else {
            self.wet_noise = (self.wet_noise * self.rise).min(e_wet);
        }
        let sp = match voice_prob {
            Some(v) => v.clamp(0.0, 1.0),
            None => {
                let snr = pow_to_db(e_wet) - pow_to_db(self.wet_noise);
                ((snr - 4.0) / 8.0).clamp(0.0, 1.0)
            }
        };
        self.last_sp = sp;
        self.frames += 1;

        // 3. 隐含增益与平滑
        let s = p.smooth.clamp(0.0, 1.0);
        let alpha = 0.85 * s * (1.0 - sp); // 无人声时最强（s=1 时 τ≈6 帧≈0.1 s）
        for k in 0..n {
            let dx = dry[k].norm();
            let g = if dx > 1e-9 { (wet[k].norm() / dx).min(1.0) } else { 0.0 };
            // 上升（人声起音）时只用 alpha 的四分之一，避免吃字头
            let a = if g > self.g_s[k] { alpha * 0.25 } else { alpha };
            self.g_s[k] = a * self.g_s[k] + (1.0 - a) * g;
            self.g_tmp[k] = g;
        }
        let beta = 0.5 * s * (1.0 - sp);
        if beta > 0.0 {
            // 三点频率平滑写回 g_tmp 之外的临时量：用 boost 当暂存（下面会重算 boost）
            for k in 0..n {
                let l = self.g_s[k.saturating_sub(1)];
                let r = self.g_s[(k + 1).min(n - 1)];
                self.boost[k] = (1.0 - beta) * self.g_s[k] + beta * 0.5 * (l + r);
            }
            self.g_s[..n].copy_from_slice(&self.boost[..n]);
        }

        // 去发闷：更新四个频带统计（只在语音帧）
        if p.demuffle_max_db > 0.0 && sp > 0.5 {
            let mut c = [0.0f32; N_EQ];
            let mut w = [0.0f32; N_EQ];
            let mut nn = [0.0f32; N_EQ];
            for k in 0..n {
                let f = k as f32 * self.hz_per_bin;
                if f < EQ_EDGES_HZ[0] || f >= EQ_EDGES_HZ[N_EQ] {
                    continue;
                }
                let b = EQ_EDGES_HZ.iter().rposition(|&e| f >= e).unwrap_or(0).min(N_EQ - 1);
                c[b] += (dry[k].norm_sqr() - self.noise[k]).max(0.0);
                w[b] += wet[k].norm_sqr();
                nn[b] += self.noise[k];
            }
            for b in 0..N_EQ {
                self.eq_clean[b] += self.a_eq * (c[b] - self.eq_clean[b]);
                self.eq_wet[b] += self.a_eq * (w[b] - self.eq_wet[b]);
                self.eq_snr[b] += self.a_eq * (pow_to_db(c[b]) - pow_to_db(nn[b]) - self.eq_snr[b]);
            }
        }
        for b in 0..N_EQ {
            let target = if p.demuffle_max_db > 0.0 && self.eq_snr[b] > 6.0 && self.eq_wet[b] > 0.0 {
                (0.5 * (pow_to_db(self.eq_clean[b]) - pow_to_db(self.eq_wet[b]))).clamp(0.0, p.demuffle_max_db)
            } else {
                0.0
            };
            // 补偿值本身也慢慢走（每帧最多 0.05 dB）
            let d = (target - self.eq_gain_db[b]).clamp(-0.05, 0.05);
            self.eq_gain_db[b] += d;
        }
        for k in 0..n {
            let f = k as f32 * self.hz_per_bin;
            let mut gdb = 0.0;
            if f >= EQ_EDGES_HZ[0] {
                let b = EQ_EDGES_HZ.iter().rposition(|&e| f >= e).unwrap_or(0).min(N_EQ - 1);
                gdb = self.eq_gain_db[b];
            } else if f >= 1500.0 {
                gdb = self.eq_gain_db[0] * (f - 1500.0) / 500.0; // 1.5–2 kHz 斜坡过渡，避免台阶
            }
            self.boost[k] = 10f32.powf(gdb / 20.0);
        }

        // 4–6. 下限、补偿、写回、舒适噪声
        let floor = if p.floor_db <= -99.0 { 0.0 } else { 10f32.powf(p.floor_db / 20.0) };
        let comfort = if p.comfort_db <= -99.0 { 0.0 } else { 10f32.powf(p.comfort_db / 20.0) };
        for k in 0..n {
            let g = self.g_tmp[k];
            let gt = (self.g_s[k].max(floor) * self.boost[k]).min(1.0);
            if g > 1e-4 {
                wet[k] *= gt / g;
            } else {
                wet[k] = dry[k] * gt;
            }
            if comfort > 0.0 {
                let target = comfort * comfort * self.noise[k];
                let have = wet[k].norm_sqr();
                if have < target {
                    let mag = (target - have).sqrt();
                    let ph = self.rand_unit() * std::f32::consts::TAU;
                    wet[k] += Complex32::new(mag * ph.cos(), mag * ph.sin());
                }
            }
        }
    }
}
