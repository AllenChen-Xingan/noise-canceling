//! 频谱修整节点：平滑（金属声）· 增益下限 + 舒适噪声（真空感）· 去发闷均衡。
//!
//! 输入：同一帧的带噪频谱 `dry` 与模型输出频谱 `wet`（同一套 STFT）。只用当前帧与过去帧的统计 → 零前瞻、不加延迟。
//!
//! 每帧步骤：
//! 1. 底噪谱 `noise[k]`：对 `|dry|²` 做「快降慢升」的最小值跟踪（降 τ≈0.08 s，升 +3 dB/s）——稳态部分，人声峰带不进来
//! 2. 本帧有没有人声 `sp ∈ [0,1]`：有 VAD（`FrameCtx.voice_prob`）就用；没有就用降噪后帧能量对其自身底噪的后验信噪比粗判
//! 3. 隐含增益 `g = |wet| / |dry|`（封顶 1）。无人声时对 `g` 做「慢升快降」的时间平滑（系数随 `smooth`）和三点频率平滑，
//!    把孤立跳动的增益尖峰（音乐噪声的来源）抹平；有人声时几乎不平滑，避免拖尾、吃字头
//! 4. 下限：`g' = max(g_s, floor)`；再乘去发闷补偿 `b[k]`，但 `g'·b ≤ 1`；写回时按 `g'·b·|dry|` 定幅度，所以每个频点都不比带噪输入更响（舒适噪声补完也封顶到 |dry|）
//! 5. `wet ← wet · g'·b·|dry| / |wet|`（保留模型的相位）；`|wet|` 极小时用带噪相位
//! 6. 舒适噪声：若 `|wet|² < c²·noise[k]`，补上差额的随机相位分量（只补缺口，不叠加到已有声音上）
//!
//! 去发闷：只在语音帧上，以「带噪 − 底噪」估计干净语音在 2–3 / 3–4.5 / 4.5–6 / 6–8 kHz 四个频带的能量，与降噪后能量比，
//! 慢慢（τ≈3 s）补回一半差距，封顶 `demuffle_max_db`；该频带信噪比估计 < 6 dB 时不补（否则补回的是噪音）。

use realfft::num_complex::Complex32;

use crate::params::PolishParams;
use crate::{db_to_pow, pow_to_db};

const N_EQ: usize = 4;
/// 帧功率低于它当数字静音（|X|² 未归一，16-bit 满幅正弦约 1e10 量级）
const SILENT_POW: f32 = 1e-9;
/// 底噪跟踪的下限（防次正规数）
const NOISE_MIN: f32 = 1e-14;
const EQ_EDGES_HZ: [f32; N_EQ + 1] = [2000.0, 3000.0, 4500.0, 6000.0, 8000.0];
/// 不属于任何去发闷频带的频点
const NO_BAND: u8 = u8::MAX;

pub struct SpectralPolish {
    bins: usize,
    noise: Vec<f32>,
    wet_noise: f32,
    g_s: Vec<f32>,
    g_tmp: Vec<f32>,
    boost: Vec<f32>,
    /// 每个频点属于哪个去发闷频带（`NO_BAND` = 不属于）
    eq_band: Vec<u8>,
    /// 1.5–2 kHz 斜坡：该频点用第 0 带补偿的多少（其余频点 0 或 1）
    eq_ramp: Vec<f32>,
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
        let hz = sr / n_fft as f32;
        let mut eq_band = vec![NO_BAND; bins];
        let mut eq_ramp = vec![0.0; bins];
        for (k, (b, r)) in eq_band.iter_mut().zip(eq_ramp.iter_mut()).enumerate() {
            let f = k as f32 * hz;
            if let Some(i) = EQ_EDGES_HZ
                .windows(2)
                .position(|e| matches!(e, [lo, hi] if f >= *lo && f < *hi))
            {
                *b = i as u8;
                *r = 1.0;
            } else if (1500.0..2000.0).contains(&f) {
                *b = 0;
                *r = (f - 1500.0) / 500.0; // 1.5–2 kHz 斜坡过渡，避免台阶
            }
        }
        Self {
            bins,
            noise: vec![0.0; bins],
            wet_noise: 0.0,
            g_s: vec![1.0; bins],
            g_tmp: vec![1.0; bins],
            boost: vec![1.0; bins],
            eq_band,
            eq_ramp,
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
    fn rand_unit(rng: &mut u64) -> f32 {
        // xorshift64*：不分配、确定性
        *rng ^= *rng >> 12;
        *rng ^= *rng << 25;
        *rng ^= *rng >> 27;
        let v = rng.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 40) as f32 / (1u64 << 24) as f32
    }

    /// 处理一帧。`dry.len() == wet.len() == bins()`，长度不对就原样放过（不 panic）。
    #[deny(
        clippy::disallowed_macros,
        clippy::disallowed_methods,
        clippy::disallowed_types
    )]
    pub fn process(
        &mut self,
        dry: &[Complex32],
        wet: &mut [Complex32],
        p: &PolishParams,
        voice_prob: Option<f32>,
    ) {
        if dry.len() != self.bins || wet.len() != self.bins {
            return;
        }
        // 1. 底噪跟踪：第一帧「非静音」时初始化；数字静音帧（设备刚打开常送全零）不更新，
        //    否则跟踪值掉到 0 / 次正规数后「乘法慢升」永远升不回来（自审 P0）
        let e_dry: f32 = dry.iter().map(|c| c.norm_sqr()).sum();
        let e_wet: f32 = wet.iter().map(|c| c.norm_sqr()).sum();
        let silent = e_dry.is_nan() || e_dry <= SILENT_POW || !e_wet.is_finite();
        if !silent {
            for (d, nk) in dry.iter().zip(self.noise.iter_mut()) {
                let px = d.norm_sqr().max(NOISE_MIN);
                if self.frames == 0 {
                    *nk = px;
                } else if px < *nk {
                    *nk += self.a_down * (px - *nk);
                } else {
                    *nk = (*nk * self.rise).min(px);
                }
            }
            // 2. 降噪后帧能量的底噪（无 VAD 时判人声用）
            let ew = e_wet.max(NOISE_MIN);
            if self.frames == 0 {
                self.wet_noise = ew;
            } else if ew < self.wet_noise {
                self.wet_noise += self.a_down * (ew - self.wet_noise);
            } else {
                self.wet_noise = (self.wet_noise * self.rise).min(ew);
            }
            self.frames += 1;
        }
        let sp = match voice_prob {
            Some(v) => v.clamp(0.0, 1.0),
            None => {
                let snr = pow_to_db(e_wet) - pow_to_db(self.wet_noise);
                ((snr - 4.0) / 8.0).clamp(0.0, 1.0)
            }
        };
        self.last_sp = sp;
        // 全关时真透传（只更新跟踪量，不碰 wet）
        if p.smooth <= 0.0
            && p.floor_db <= -99.0
            && p.comfort_db <= -99.0
            && p.demuffle_max_db <= 0.0
        {
            self.g_s.fill(1.0);
            return;
        }
        if self.frames == 0 {
            return; // 还没见过非静音帧：没有底噪估计，不处理
        }

        // 3. 隐含增益与平滑
        let s = vdn_types::desc::POLISH_SMOOTH.clamp(p.smooth);
        let alpha = 0.85 * s * (1.0 - sp); // 无人声时最强（s=1 时 τ≈6 帧≈0.1 s）；有人声时 → 0，不拖字头
        for ((d, w), (gs, gt)) in dry
            .iter()
            .zip(wet.iter())
            .zip(self.g_s.iter_mut().zip(self.g_tmp.iter_mut()))
        {
            let dx = d.norm();
            let g = if dx > 1e-9 {
                (w.norm() / dx).min(1.0)
            } else {
                0.0
            };
            // 慢升快降：孤立的增益尖峰（音乐噪声）升不起来，降下去很快。
            // 第一版写反了（快升慢降 = 峰值保持，反而把尖峰拖长，自审 P1）。起音不靠这里保护，靠 sp → 1 时 alpha → 0
            let a = if g > *gs { alpha } else { alpha * 0.5 };
            *gs = a * *gs + (1.0 - a) * g;
            *gt = g;
        }
        let beta = 0.5 * s * (1.0 - sp);
        if beta > 0.0 {
            // 三点频率平滑（两端用自身补齐）；用 boost 当暂存，下面会重算
            for (k, out) in self.boost.iter_mut().enumerate() {
                let c = self.g_s.get(k).copied().unwrap_or(1.0);
                let l = k
                    .checked_sub(1)
                    .and_then(|j| self.g_s.get(j))
                    .copied()
                    .unwrap_or(c);
                let r = self.g_s.get(k + 1).copied().unwrap_or(c);
                *out = (1.0 - beta) * c + beta * 0.5 * (l + r);
            }
            self.g_s.copy_from_slice(&self.boost);
        }

        // 去发闷：更新四个频带统计（只在语音帧）
        if p.demuffle_max_db > 0.0 && sp > 0.5 {
            let mut c = [0.0f32; N_EQ];
            let mut w = [0.0f32; N_EQ];
            let mut nn = [0.0f32; N_EQ];
            for (((d, wt), nk), (&b, &r)) in dry
                .iter()
                .zip(wet.iter())
                .zip(&self.noise)
                .zip(self.eq_band.iter().zip(&self.eq_ramp))
            {
                if r < 1.0 {
                    continue; // 只统计 2–8 kHz（斜坡区不算）
                }
                if let (Some(cb), Some(wb), Some(nb)) = (
                    c.get_mut(usize::from(b)),
                    w.get_mut(usize::from(b)),
                    nn.get_mut(usize::from(b)),
                ) {
                    *cb += (d.norm_sqr() - nk).max(0.0);
                    *wb += wt.norm_sqr();
                    *nb += nk;
                }
            }
            // 模型出一帧 NaN 不能把统计永久污染（自审 P2）
            if c.iter().chain(&w).chain(&nn).all(|v| v.is_finite()) {
                for ((((ec, ew), es), &cb), (&wb, &nb)) in self
                    .eq_clean
                    .iter_mut()
                    .zip(self.eq_wet.iter_mut())
                    .zip(self.eq_snr.iter_mut())
                    .zip(&c)
                    .zip(w.iter().zip(&nn))
                {
                    *ec += self.a_eq * (cb - *ec);
                    *ew += self.a_eq * (wb - *ew);
                    *es += self.a_eq * (pow_to_db(cb) - pow_to_db(nb) - *es);
                }
            }
        }
        for (((g, &ec), &ew), &es) in self
            .eq_gain_db
            .iter_mut()
            .zip(&self.eq_clean)
            .zip(&self.eq_wet)
            .zip(&self.eq_snr)
        {
            let target = if p.demuffle_max_db > 0.0 && es > 6.0 && ew > 0.0 {
                (0.5 * (pow_to_db(ec) - pow_to_db(ew))).clamp(0.0, p.demuffle_max_db)
            } else {
                0.0
            };
            // 补偿值本身也慢慢走（每帧最多 0.05 dB）
            *g += (target - *g).clamp(-0.05, 0.05);
        }
        for ((bst, &b), &r) in self.boost.iter_mut().zip(&self.eq_band).zip(&self.eq_ramp) {
            let gdb = self.eq_gain_db.get(usize::from(b)).copied().unwrap_or(0.0) * r;
            *bst = 10f32.powf(gdb / 20.0);
        }

        // 4–6. 下限、补偿、写回、舒适噪声
        let floor = if p.floor_db <= -99.0 {
            0.0
        } else {
            10f32.powf(p.floor_db / 20.0)
        };
        let comfort = if p.comfort_db <= -99.0 {
            0.0
        } else {
            10f32.powf(p.comfort_db / 20.0)
        };
        let rng = &mut self.rng;
        for ((d, w), ((&gs, &bst), &nk)) in dry
            .iter()
            .zip(wet.iter_mut())
            .zip(self.g_s.iter().zip(&self.boost).zip(&self.noise))
        {
            let dx = d.norm();
            let gt = (gs.max(floor) * bst).min(1.0);
            // 按「目标幅度 = gt·|dry|」缩放（保留模型相位）；这样任何频点都不会比带噪输入更响
            let wm = w.norm();
            if wm > 1e-12 && wm.is_finite() {
                *w *= gt * dx / wm;
            } else {
                *w = *d * gt;
            }
            if comfort > 0.0 {
                let target = comfort * comfort * nk;
                let have = w.norm_sqr();
                if have < target {
                    // 补差额的随机相位分量（期望功率 = target），再封顶到 |dry|
                    let mag = (target - have).sqrt();
                    let ph = Self::rand_unit(rng) * std::f32::consts::TAU;
                    *w += Complex32::new(mag * ph.cos(), mag * ph.sin());
                    let m = w.norm();
                    if m > dx && m > 0.0 {
                        *w *= dx / m;
                    }
                }
            }
        }
    }
}
