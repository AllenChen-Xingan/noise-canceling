//! GT 卷积块（编码 / 解码共用）与 SFE。

use super::*;

// ───────────────────── GT 卷积块（编码 / 解码共用） ─────────────────────

pub(crate) const C: usize = 16;
pub(crate) const CH: usize = 8; // C/2

pub(crate) struct GtBlock {
    pub(crate) p1: Point,
    pub(crate) a1: f32,
    pub(crate) dw: Vec<f32>, // [16][3][3]，已折 BN（解码器的权重已由 convert_to_stream 翻转成普通卷积）
    pub(crate) db: Vec<f32>,
    pub(crate) a2: f32,
    pub(crate) p2: Point,
    pub(crate) gru: Gru,
    pub(crate) fc: Linear,
    pub(crate) dil: usize,
    // 缓存：环形，`2*dil` 帧，每帧 [16][33]
    pub(crate) ring: Vec<f32>,
    pub(crate) head: usize,
    pub(crate) hstate: [f32; 16],
    // 临时
    pub(crate) sfe: Vec<f32>, // [24][33]
    pub(crate) h1: Vec<f32>,  // [16][33]
    pub(crate) h2: Vec<f32>,  // [16][33]
    pub(crate) h3: Vec<f32>,  // [8][33]
}

impl GtBlock {
    pub(crate) fn load(w: &Weights, p: &str, dil: usize, deconv: bool) -> Result<Self, Err> {
        let dname = if deconv { "ConvTranspose2d" } else { "Conv2d" };
        let mut dw = w.get(&format!("{p}.depth_conv.{dname}.weight"), C * 9)?;
        let mut db = w.get(&format!("{p}.depth_conv.{dname}.bias"), C)?;
        let (s, t) = w.bn(&format!("{p}.depth_bn"), C)?;
        for c in 0..C {
            for v in &mut dw[c * 9..(c + 1) * 9] {
                *v *= s[c];
            }
            db[c] = db[c] * s[c] + t[c];
        }
        Ok(Self {
            p1: Point::load(
                w,
                &format!("{p}.point_conv1"),
                3 * CH,
                C,
                deconv,
                Some(&format!("{p}.point_bn1")),
            )?,
            a1: w.scalar(&format!("{p}.point_act.weight"))?,
            dw,
            db,
            a2: w.scalar(&format!("{p}.depth_act.weight"))?,
            p2: Point::load(
                w,
                &format!("{p}.point_conv2"),
                C,
                CH,
                deconv,
                Some(&format!("{p}.point_bn2")),
            )?,
            gru: Gru::load(w, &format!("{p}.tra.att_gru"), "", CH, 2 * CH)?,
            fc: Linear::load(w, &format!("{p}.tra.att_fc"), 2 * CH, CH)?,
            dil,
            ring: vec![0.0; 2 * dil * C * F2],
            head: 0,
            hstate: [0.0; 16],
            sfe: vec![0.0; 3 * CH * F2],
            h1: vec![0.0; C * F2],
            h2: vec![0.0; C * F2],
            h3: vec![0.0; CH * F2],
        })
    }

    pub(crate) fn reset(&mut self) {
        self.ring.fill(0.0);
        self.head = 0;
        self.hstate = [0.0; 16];
    }

    /// 深度卷积第 c 通道、时间抽头 kt 的输入行：inp 的时间下标 0..=2d，2d 是当前帧（h1），其余在环形缓存里
    #[inline(always)]
    pub(crate) fn dw_src(&self, c: usize, kt: usize) -> &[f32] {
        let (f, len, frame) = (F2, 2 * self.dil, C * F2);
        let ti = kt * self.dil;
        if ti == len {
            &self.h1[c * f..(c + 1) * f]
        } else {
            let slot = (self.head + ti) % len;
            &self.ring[slot * frame + c * f..slot * frame + (c + 1) * f]
        }
    }

    /// 原写法：h2 = PReLU(db + Σ_kt Σ_j' w·src)，频率补 1
    pub(crate) fn depthwise_scalar(&mut self) {
        let f = F2;
        for c in 0..C {
            let mut out = [0.0f32; F2];
            out.fill(self.db[c]);
            for kt in 0..3 {
                let src = self.dw_src(c, kt);
                let w = &self.dw[c * 9 + kt * 3..c * 9 + kt * 3 + 3];
                for j in 0..f {
                    let mut a = w[1] * src[j];
                    if j > 0 {
                        a += w[0] * src[j - 1];
                    }
                    if j + 1 < f {
                        a += w[2] * src[j + 1];
                    }
                    out[j] += a;
                }
            }
            self.h2[c * f..(c + 1) * f].copy_from_slice(&out);
        }
        for v in self.h2.iter_mut() {
            *v = prelu(*v, self.a2);
        }
    }

    /// `x`：[16][33] → `y`：[16][33]
    pub(crate) fn run<K: Kern, P: Probe>(&mut self, x: &[f32], y: &mut [f32], p: &mut P, blk: &'static str) {
        let f = F2;
        // x1 = 前 8 通道 → SFE → 24 通道
        sfe3(&x[..CH * f], CH, f, &mut self.sfe);
        K::point(&self.p1, &self.sfe, &mut self.h1, f);
        for v in self.h1.iter_mut() {
            *v = prelu(*v, self.a1);
        }
        p.mark(blk, "sfe+point1", || {
            let act = (C * f) as u64;
            self.p1.cost(f)
                + Cost {
                    other: (3 * CH * f) as u64 + act, // SFE 拷贝 + PReLU
                    chain_fma: 1,
                    ..Cost::default()
                }
        });
        // 流式深度卷积 + PReLU（算子写法可换）；之后把当前帧写进环形缓存（结构，两种写法共用）
        K::depthwise(self);
        let (len, frame) = (2 * self.dil, C * f);
        let slot = self.head;
        self.ring[slot * frame..(slot + 1) * frame].copy_from_slice(&self.h1);
        self.head = (self.head + 1) % len;
        p.mark(blk, "depthwise", || {
            let act = (C * f) as u64;
            Cost {
                macs: (C * 3 * (3 * f - 2)) as u64,
                other: act * 2, // PReLU + 写环形缓存
                w_bytes: (C * 10) as u64 * F32B,
                io_bytes: act * 5 * F32B, // 读 3 帧 + 写输出 + 写环形缓存
                chain_fma: clog2(9) + 2,
                ..Cost::default()
            }
        });
        K::point(&self.p2, &self.h2, &mut self.h3, f);
        p.mark(blk, "point2", || self.p2.cost(f));
        // TRA
        let mut zt = [0.0f32; CH];
        for c in 0..CH {
            zt[c] = self.h3[c * f..(c + 1) * f]
                .iter()
                .map(|v| v * v)
                .sum::<f32>()
                / f as f32;
        }
        K::gru(&self.gru, &zt, &mut self.hstate);
        let mut at = [0.0f32; CH];
        K::linear(&self.fc, &self.hstate, &mut at);
        // shuffle：y[2c] = h1[c]*at[c]，y[2c+1] = x2[c]
        for c in 0..CH {
            let a = sigmoid(at[c]);
            for j in 0..f {
                y[(2 * c) * f + j] = self.h3[c * f + j] * a;
            }
            y[(2 * c + 1) * f..(2 * c + 2) * f].copy_from_slice(&x[(CH + c) * f..(CH + c + 1) * f]);
        }
        p.mark(blk, "tra+shuffle", || {
            let e = Cost {
                macs: (CH * f) as u64, // 能量
                nonlin: CH as u64,     // sigmoid
                other: (C * f) as u64, // 乘注意力 + 拷贝
                io_bytes: (CH * f + 2 * C * f) as u64 * F32B,
                chain_fma: clog2(f) + 1 + 1, // 能量归约 + 乘注意力
                chain_nl: 1,
                ..Cost::default()
            };
            e + self.gru.step_cost() + self.fc.cost()
        });
    }
}

/// SFE（unfold 1×3，pad 1）：[c][f] → [c*3 + j][f] = x[c][f + j − 1]
pub(crate) fn sfe3(x: &[f32], c: usize, f: usize, y: &mut [f32]) {
    for ci in 0..c {
        let xi = &x[ci * f..(ci + 1) * f];
        for j in 0..3 {
            let yo = &mut y[(ci * 3 + j) * f..(ci * 3 + j + 1) * f];
            for k in 0..f {
                let idx = k as isize + j as isize - 1;
                yo[k] = if idx >= 0 && (idx as usize) < f {
                    xi[idx as usize]
                } else {
                    0.0
                };
            }
        }
    }
}
