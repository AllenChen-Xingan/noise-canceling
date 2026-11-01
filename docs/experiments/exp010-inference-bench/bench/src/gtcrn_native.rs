//! GTCRN 流式版手写实现（纯 Rust，无依赖）。
//!
//! 照 `resources/gtcrn/stream/gtcrn_stream.py` 的 `StreamGTCRN.forward` 逐层写，权重来自
//! `tools/dump_gtcrn.py` 导出的 state_dict（DNS3 权重，与 `gtcrn_simple.onnx` 同源）。
//! 构造期分配全部缓冲；`infer` 每帧零分配、无锁、无系统调用。
//! 布局：特征图一律 `[通道][频点]` 行主序；DPGRNN 内部用 `[频点][通道]`。

use std::collections::HashMap;
use std::path::Path;

const BINS: usize = 257;
const ERB_LOW: usize = 65; // erb_subband_1
const ERB_HIGH: usize = 64; // erb_subband_2
const F0: usize = ERB_LOW + ERB_HIGH; // 129
const F1: usize = 65;
const F2: usize = 33;
const BN_EPS: f32 = 1e-5;
const LN_EPS: f32 = 1e-8;

pub type Err = Box<dyn std::error::Error + Send + Sync>;

/// 扁平权重表：`<prefix>.bin` + `<prefix>.txt`。
pub struct Weights {
    data: Vec<f32>,
    index: HashMap<String, (Vec<usize>, usize, usize)>,
}

impl Weights {
    pub fn load(prefix: &Path) -> Result<Self, Err> {
        let bin = std::fs::read(prefix.with_extension("bin"))?;
        let txt = std::fs::read_to_string(prefix.with_extension("txt"))?;
        let data: Vec<f32> = bin
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let mut index = HashMap::new();
        for line in txt.lines() {
            let p: Vec<&str> = line.split(' ').collect();
            if p.len() != 4 {
                return Err(format!("坏行：{line}").into());
            }
            let shape = p[1].split(',').map(str::parse).collect::<Result<Vec<usize>, _>>()?;
            index.insert(p[0].to_string(), (shape, p[2].parse()?, p[3].parse()?));
        }
        Ok(Self { data, index })
    }

    fn get(&self, name: &str, numel: usize) -> Result<Vec<f32>, Err> {
        let (_, off, n) = self.index.get(name).ok_or_else(|| format!("缺权重 {name}"))?;
        if *n != numel {
            return Err(format!("{name} 个数 {n} ≠ 期望 {numel}").into());
        }
        Ok(self.data[*off..*off + *n].to_vec())
    }

    fn scalar(&self, name: &str) -> Result<f32, Err> {
        Ok(self.get(name, 1)?[0])
    }

    /// BN（推理态）折成逐通道 scale / shift。
    fn bn(&self, p: &str, c: usize) -> Result<(Vec<f32>, Vec<f32>), Err> {
        let g = self.get(&format!("{p}.weight"), c)?;
        let b = self.get(&format!("{p}.bias"), c)?;
        let m = self.get(&format!("{p}.running_mean"), c)?;
        let v = self.get(&format!("{p}.running_var"), c)?;
        let scale: Vec<f32> = g.iter().zip(&v).map(|(g, v)| g / (v + BN_EPS).sqrt()).collect();
        let shift = b.iter().zip(&m).zip(&scale).map(|((b, m), s)| b - m * s).collect();
        Ok((scale, shift))
    }
}

// ───────────────────────── 积木 ─────────────────────────

#[inline]
fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[inline]
fn prelu(x: f32, a: f32) -> f32 {
    if x >= 0.0 { x } else { a * x }
}

/// 1×1 卷积（逐点），权重 `[out][in]`，已折 BN（可选）。
struct Point {
    w: Vec<f32>,
    b: Vec<f32>,
    cin: usize,
    cout: usize,
}

impl Point {
    /// `transposed`：ConvTranspose 的权重是 `[in][out]`，转成 `[out][in]`。
    fn load(w: &Weights, p: &str, cin: usize, cout: usize, transposed: bool, bn: Option<&str>) -> Result<Self, Err> {
        let raw = w.get(&format!("{p}.weight"), cin * cout)?;
        let mut ww = vec![0.0; cin * cout];
        for o in 0..cout {
            for i in 0..cin {
                ww[o * cin + i] = if transposed { raw[i * cout + o] } else { raw[o * cin + i] };
            }
        }
        let mut b = w.get(&format!("{p}.bias"), cout)?;
        if let Some(bn) = bn {
            let (s, t) = w.bn(bn, cout)?;
            for o in 0..cout {
                for i in 0..cin {
                    ww[o * cin + i] *= s[o];
                }
                b[o] = b[o] * s[o] + t[o];
            }
        }
        Ok(Self { w: ww, b, cin, cout })
    }

    fn run(&self, x: &[f32], y: &mut [f32], f: usize) {
        for o in 0..self.cout {
            let yo = &mut y[o * f..(o + 1) * f];
            yo.fill(self.b[o]);
            for i in 0..self.cin {
                let wi = self.w[o * self.cin + i];
                for (a, &xv) in yo.iter_mut().zip(&x[i * f..(i + 1) * f]) {
                    *a += wi * xv;
                }
            }
        }
    }
}

/// 沿频率轴的分组一维卷积（编码器 ConvBlock：k=5，stride 2，pad 2）。权重 `[out][in/g][k]`，已折 BN。
struct ConvF {
    w: Vec<f32>,
    b: Vec<f32>,
    cin: usize,
    cout: usize,
    groups: usize,
    fin: usize,
    fout: usize,
    alpha: f32,
}

const K: usize = 5;

impl ConvF {
    fn load(w: &Weights, p: &str, cin: usize, cout: usize, groups: usize, fin: usize) -> Result<Self, Err> {
        let cig = cin / groups;
        let mut ww = w.get(&format!("{p}.conv.weight"), cout * cig * K)?;
        let mut b = w.get(&format!("{p}.conv.bias"), cout)?;
        let (s, t) = w.bn(&format!("{p}.bn"), cout)?;
        for o in 0..cout {
            for v in &mut ww[o * cig * K..(o + 1) * cig * K] {
                *v *= s[o];
            }
            b[o] = b[o] * s[o] + t[o];
        }
        let fout = (fin + 2 * 2 - K) / 2 + 1;
        Ok(Self { w: ww, b, cin, cout, groups, fin, fout, alpha: w.scalar(&format!("{p}.act.weight"))? })
    }

    fn run(&self, x: &[f32], y: &mut [f32]) {
        let (cig, cog) = (self.cin / self.groups, self.cout / self.groups);
        for o in 0..self.cout {
            let g = o / cog;
            let yo = &mut y[o * self.fout..(o + 1) * self.fout];
            yo.fill(self.b[o]);
            for ci in 0..cig {
                let xi = &x[(g * cig + ci) * self.fin..(g * cig + ci + 1) * self.fin];
                for t in 0..K {
                    let wv = self.w[(o * cig + ci) * K + t];
                    for (j, a) in yo.iter_mut().enumerate() {
                        let idx = (j * 2 + t) as isize - 2;
                        if idx >= 0 && (idx as usize) < self.fin {
                            *a += wv * xi[idx as usize];
                        }
                    }
                }
            }
            for v in yo.iter_mut() {
                *v = prelu(*v, self.alpha);
            }
        }
    }
}

/// 沿频率轴的分组一维转置卷积（解码器 ConvBlock：k=5，stride 2，pad 2）。权重 `[in][out/g][k]`，已折 BN。
struct DeconvF {
    w: Vec<f32>,
    b: Vec<f32>,
    cin: usize,
    cout: usize,
    groups: usize,
    fin: usize,
    fout: usize,
    alpha: f32,
    tanh: bool,
}

impl DeconvF {
    fn load(w: &Weights, p: &str, cin: usize, cout: usize, groups: usize, fin: usize, tanh: bool) -> Result<Self, Err> {
        let cog = cout / groups;
        let mut ww = w.get(&format!("{p}.conv.weight"), cin * cog * K)?;
        let mut b = w.get(&format!("{p}.conv.bias"), cout)?;
        let (s, t) = w.bn(&format!("{p}.bn"), cout)?;
        let cig = cin / groups;
        for ic in 0..cin {
            let g = ic / cig;
            for ol in 0..cog {
                for k in 0..K {
                    ww[(ic * cog + ol) * K + k] *= s[g * cog + ol];
                }
            }
        }
        for o in 0..cout {
            b[o] = b[o] * s[o] + t[o];
        }
        let fout = (fin - 1) * 2 - 2 * 2 + K;
        let alpha = if tanh { 0.0 } else { w.scalar(&format!("{p}.act.weight"))? };
        Ok(Self { w: ww, b, cin, cout, groups, fin, fout, alpha, tanh })
    }

    fn run(&self, x: &[f32], y: &mut [f32]) {
        let (cig, cog) = (self.cin / self.groups, self.cout / self.groups);
        for o in 0..self.cout {
            y[o * self.fout..(o + 1) * self.fout].fill(self.b[o]);
        }
        for ic in 0..self.cin {
            let g = ic / cig;
            let xi = &x[ic * self.fin..(ic + 1) * self.fin];
            for ol in 0..cog {
                let o = g * cog + ol;
                let yo = &mut y[o * self.fout..(o + 1) * self.fout];
                for k in 0..K {
                    let wv = self.w[(ic * cog + ol) * K + k];
                    for (i, &xv) in xi.iter().enumerate() {
                        let j = (i * 2 + k) as isize - 2;
                        if j >= 0 && (j as usize) < self.fout {
                            yo[j as usize] += wv * xv;
                        }
                    }
                }
            }
        }
        for v in y[..self.cout * self.fout].iter_mut() {
            *v = if self.tanh { v.tanh() } else { prelu(*v, self.alpha) };
        }
    }
}

/// PyTorch GRU 单步（门序 r, z, n）。
struct Gru {
    wi: Vec<f32>, // [3H][I]
    wh: Vec<f32>, // [3H][H]
    bi: Vec<f32>,
    bh: Vec<f32>,
    i: usize,
    h: usize,
}

const MAX_G: usize = 48;

impl Gru {
    fn load(w: &Weights, p: &str, sfx: &str, i: usize, h: usize) -> Result<Self, Err> {
        Ok(Self {
            wi: w.get(&format!("{p}.weight_ih_l0{sfx}"), 3 * h * i)?,
            wh: w.get(&format!("{p}.weight_hh_l0{sfx}"), 3 * h * h)?,
            bi: w.get(&format!("{p}.bias_ih_l0{sfx}"), 3 * h)?,
            bh: w.get(&format!("{p}.bias_hh_l0{sfx}"), 3 * h)?,
            i,
            h,
        })
    }

    /// `x` 长 I，`h` 长 H，原地更新 `h`。
    #[inline]
    fn step(&self, x: &[f32], h: &mut [f32]) {
        let (ni, nh) = (self.i, self.h);
        let mut gi = [0.0f32; MAX_G];
        let mut gh = [0.0f32; MAX_G];
        for g in 0..3 * nh {
            let mut a = self.bi[g];
            for (w, v) in self.wi[g * ni..(g + 1) * ni].iter().zip(x) {
                a += w * v;
            }
            gi[g] = a;
            let mut b = self.bh[g];
            for (w, v) in self.wh[g * nh..(g + 1) * nh].iter().zip(h.iter()) {
                b += w * v;
            }
            gh[g] = b;
        }
        for k in 0..nh {
            let r = sigmoid(gi[k] + gh[k]);
            let z = sigmoid(gi[nh + k] + gh[nh + k]);
            let n = (gi[2 * nh + k] + r * gh[2 * nh + k]).tanh();
            h[k] = (1.0 - z) * n + z * h[k];
        }
    }
}

/// 全连接，权重 `[out][in]`。
struct Linear {
    w: Vec<f32>,
    b: Vec<f32>,
    i: usize,
    o: usize,
}

impl Linear {
    fn load(w: &Weights, p: &str, i: usize, o: usize) -> Result<Self, Err> {
        Ok(Self { w: w.get(&format!("{p}.weight"), o * i)?, b: w.get(&format!("{p}.bias"), o)?, i, o })
    }
    #[inline]
    fn run(&self, x: &[f32], y: &mut [f32]) {
        for k in 0..self.o {
            let mut a = self.b[k];
            for (w, v) in self.w[k * self.i..(k + 1) * self.i].iter().zip(x) {
                a += w * v;
            }
            y[k] = a;
        }
    }
}

// ───────────────────── GT 卷积块（编码 / 解码共用） ─────────────────────

const C: usize = 16;
const CH: usize = 8; // C/2

struct GtBlock {
    p1: Point,
    a1: f32,
    dw: Vec<f32>, // [16][3][3]，已折 BN（解码器的权重已由 convert_to_stream 翻转成普通卷积）
    db: Vec<f32>,
    a2: f32,
    p2: Point,
    gru: Gru,
    fc: Linear,
    dil: usize,
    // 缓存：环形，`2*dil` 帧，每帧 [16][33]
    ring: Vec<f32>,
    head: usize,
    hstate: [f32; 16],
    // 临时
    sfe: Vec<f32>, // [24][33]
    h1: Vec<f32>,  // [16][33]
    h2: Vec<f32>,  // [16][33]
    h3: Vec<f32>,  // [8][33]
}

impl GtBlock {
    fn load(w: &Weights, p: &str, dil: usize, deconv: bool) -> Result<Self, Err> {
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
            p1: Point::load(w, &format!("{p}.point_conv1"), 3 * CH, C, deconv, Some(&format!("{p}.point_bn1")))?,
            a1: w.scalar(&format!("{p}.point_act.weight"))?,
            dw,
            db,
            a2: w.scalar(&format!("{p}.depth_act.weight"))?,
            p2: Point::load(w, &format!("{p}.point_conv2"), C, CH, deconv, Some(&format!("{p}.point_bn2")))?,
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

    fn reset(&mut self) {
        self.ring.fill(0.0);
        self.head = 0;
        self.hstate = [0.0; 16];
    }

    /// `x`：[16][33] → `y`：[16][33]
    fn run(&mut self, x: &[f32], y: &mut [f32]) {
        let f = F2;
        // x1 = 前 8 通道 → SFE → 24 通道
        sfe3(&x[..CH * f], CH, f, &mut self.sfe);
        self.p1.run(&self.sfe, &mut self.h1, f);
        for v in self.h1.iter_mut() {
            *v = prelu(*v, self.a1);
        }
        // 流式深度卷积：inp 的时间下标 0..=2d，2d 是当前帧；kt 取 inp[kt*d]
        let len = 2 * self.dil;
        let frame = C * f;
        for c in 0..C {
            let out = &mut self.h2[c * f..(c + 1) * f];
            out.fill(self.db[c]);
            for kt in 0..3 {
                let ti = kt * self.dil;
                let src: &[f32] = if ti == len {
                    &self.h1[c * f..(c + 1) * f]
                } else {
                    let slot = (self.head + ti) % len;
                    &self.ring[slot * frame + c * f..slot * frame + (c + 1) * f]
                };
                let w = &self.dw[c * 9 + kt * 3..c * 9 + kt * 3 + 3];
                // 频率 pad 1
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
        }
        // 当前帧写进最旧的槽位
        let slot = self.head;
        self.ring[slot * frame..(slot + 1) * frame].copy_from_slice(&self.h1);
        self.head = (self.head + 1) % len;
        for v in self.h2.iter_mut() {
            *v = prelu(*v, self.a2);
        }
        self.p2.run(&self.h2, &mut self.h3, f);
        // TRA
        let mut zt = [0.0f32; CH];
        for c in 0..CH {
            zt[c] = self.h3[c * f..(c + 1) * f].iter().map(|v| v * v).sum::<f32>() / f as f32;
        }
        self.gru.step(&zt, &mut self.hstate);
        let mut at = [0.0f32; CH];
        self.fc.run(&self.hstate, &mut at);
        // shuffle：y[2c] = h1[c]*at[c]，y[2c+1] = x2[c]
        for c in 0..CH {
            let a = sigmoid(at[c]);
            for j in 0..f {
                y[(2 * c) * f + j] = self.h3[c * f + j] * a;
            }
            y[(2 * c + 1) * f..(2 * c + 2) * f].copy_from_slice(&x[(CH + c) * f..(CH + c + 1) * f]);
        }
    }
}

/// SFE（unfold 1×3，pad 1）：[c][f] → [c*3 + j][f] = x[c][f + j − 1]
fn sfe3(x: &[f32], c: usize, f: usize, y: &mut [f32]) {
    for ci in 0..c {
        let xi = &x[ci * f..(ci + 1) * f];
        for j in 0..3 {
            let yo = &mut y[(ci * 3 + j) * f..(ci * 3 + j + 1) * f];
            for k in 0..f {
                let idx = k as isize + j as isize - 1;
                yo[k] = if idx >= 0 && (idx as usize) < f { xi[idx as usize] } else { 0.0 };
            }
        }
    }
}

// ───────────────────────── DPGRNN ─────────────────────────

struct Dp {
    intra: [Gru; 4], // rnn1 fwd, rnn1 bwd, rnn2 fwd, rnn2 bwd（H=4）
    intra_fc: Linear,
    intra_lnw: Vec<f32>,
    intra_lnb: Vec<f32>,
    inter: [Gru; 2], // H=8
    inter_fc: Linear,
    inter_lnw: Vec<f32>,
    inter_lnb: Vec<f32>,
    cache: Vec<f32>, // [33][16]
    xt: Vec<f32>,    // [33][16]
    y: Vec<f32>,
    z: Vec<f32>,
}

impl Dp {
    fn load(w: &Weights, p: &str) -> Result<Self, Err> {
        let g = |n: &str, s: &str, h: usize| Gru::load(w, &format!("{p}.{n}"), s, CH, h);
        Ok(Self {
            intra: [
                g("intra_rnn.rnn1", "", 4)?,
                g("intra_rnn.rnn1", "_reverse", 4)?,
                g("intra_rnn.rnn2", "", 4)?,
                g("intra_rnn.rnn2", "_reverse", 4)?,
            ],
            intra_fc: Linear::load(w, &format!("{p}.intra_fc"), C, C)?,
            intra_lnw: w.get(&format!("{p}.intra_ln.weight"), F2 * C)?,
            intra_lnb: w.get(&format!("{p}.intra_ln.bias"), F2 * C)?,
            inter: [g("inter_rnn.rnn1", "", 8)?, g("inter_rnn.rnn2", "", 8)?],
            inter_fc: Linear::load(w, &format!("{p}.inter_fc"), C, C)?,
            inter_lnw: w.get(&format!("{p}.inter_ln.weight"), F2 * C)?,
            inter_lnb: w.get(&format!("{p}.inter_ln.bias"), F2 * C)?,
            cache: vec![0.0; F2 * C],
            xt: vec![0.0; F2 * C],
            y: vec![0.0; F2 * C],
            z: vec![0.0; F2 * C],
        })
    }

    fn reset(&mut self) {
        self.cache.fill(0.0);
    }

    /// 原地：`feat` [16][33]
    fn run(&mut self, feat: &mut [f32]) {
        // 转成 [F][C]
        for c in 0..C {
            for f in 0..F2 {
                self.xt[f * C + c] = feat[c * F2 + f];
            }
        }
        // intra：沿频率的双向分组 GRU，初值 0
        for grp in 0..2 {
            let (fw, bw) = (&self.intra[2 * grp], &self.intra[2 * grp + 1]);
            let mut h = [0.0f32; 4];
            for f in 0..F2 {
                fw.step(&self.xt[f * C + grp * CH..f * C + grp * CH + CH], &mut h);
                self.y[f * C + grp * CH..f * C + grp * CH + 4].copy_from_slice(&h);
            }
            let mut h = [0.0f32; 4];
            for f in (0..F2).rev() {
                bw.step(&self.xt[f * C + grp * CH..f * C + grp * CH + CH], &mut h);
                self.y[f * C + grp * CH + 4..f * C + grp * CH + 8].copy_from_slice(&h);
            }
        }
        for f in 0..F2 {
            self.intra_fc.run(&self.y[f * C..(f + 1) * C], &mut self.z[f * C..(f + 1) * C]);
        }
        layer_norm_add(&mut self.xt, &self.z, &self.intra_lnw, &self.intra_lnb);
        // inter：沿时间（单步）的分组 GRU，状态逐频点缓存
        for f in 0..F2 {
            for grp in 0..2 {
                let r = f * C + grp * CH..f * C + grp * CH + CH;
                self.inter[grp].step(&self.xt[r.clone()], &mut self.cache[r.clone()]);
            }
            self.inter_fc.run(&self.cache[f * C..(f + 1) * C], &mut self.z[f * C..(f + 1) * C]);
        }
        layer_norm_add(&mut self.xt, &self.z, &self.inter_lnw, &self.inter_lnb);
        for c in 0..C {
            for f in 0..F2 {
                feat[c * F2 + f] = self.xt[f * C + c];
            }
        }
    }
}

/// x += LN(z)，LN 在整个 (33,16) 上归一化，带逐元素仿射。
fn layer_norm_add(x: &mut [f32], z: &[f32], w: &[f32], b: &[f32]) {
    let n = z.len() as f32;
    let mean = z.iter().sum::<f32>() / n;
    let var = z.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let inv = 1.0 / (var + LN_EPS).sqrt();
    for (((xv, zv), wv), bv) in x.iter_mut().zip(z).zip(w).zip(b) {
        *xv += (zv - mean) * inv * wv + bv;
    }
}

// ───────────────────────── ERB ─────────────────────────

/// 稀疏的 ERB 矩阵：每行只存非零区间（跳过的都是精确 0，结果与稠密乘法相同）。
struct Sparse {
    rows: Vec<(usize, Vec<f32>)>, // (起始列, 非零段)
}

impl Sparse {
    fn from_dense(w: &[f32], rows: usize, cols: usize) -> Self {
        let mut out = Vec::with_capacity(rows);
        for r in 0..rows {
            let row = &w[r * cols..(r + 1) * cols];
            let first = row.iter().position(|v| *v != 0.0).unwrap_or(0);
            let last = row.iter().rposition(|v| *v != 0.0).map_or(first, |p| p + 1);
            out.push((first, row[first..last.max(first)].to_vec()));
        }
        Self { rows: out }
    }
    #[inline]
    fn run(&self, x: &[f32], y: &mut [f32]) {
        for (yv, (s, seg)) in y.iter_mut().zip(&self.rows) {
            let mut a = 0.0;
            for (w, v) in seg.iter().zip(&x[*s..]) {
                a += w * v;
            }
            *yv = a;
        }
    }
}

// ───────────────────────── 整网 ─────────────────────────

pub struct GtcrnNative {
    erb: Sparse,  // 64 × 192
    ierb: Sparse, // 192 × 64
    en0: ConvF,
    en1: ConvF,
    en_gt: [GtBlock; 3],
    dp: [Dp; 2],
    de_gt: [GtBlock; 3],
    de3: DeconvF,
    de4: DeconvF,
    feat0: Vec<f32>, // [3][257]
    feat1: Vec<f32>, // [3][129]
    feat2: Vec<f32>, // [9][129]
    e0: Vec<f32>,    // [16][65]
    e: [Vec<f32>; 4], // e[0]=en_outs[1] [16][33]，e[1..4]=en_outs[2..5]
    t0: Vec<f32>,    // [16][33]
    t1: Vec<f32>,    // [16][33]
    d3: Vec<f32>,    // [16][65]
    d4: Vec<f32>,    // [2][129]
    m: Vec<f32>,     // [2][257]
}

impl GtcrnNative {
    pub fn load(prefix: &Path) -> Result<Self, Err> {
        let w = Weights::load(prefix)?;
        let ew = w.get("erb.erb_fc.weight", ERB_HIGH * (BINS - ERB_LOW))?;
        let iw = w.get("erb.ierb_fc.weight", (BINS - ERB_LOW) * ERB_HIGH)?;
        let gt = |i: usize, dil: usize, de: bool| {
            let p = if de { format!("decoder.de_convs.{i}") } else { format!("encoder.en_convs.{i}") };
            GtBlock::load(&w, &p, dil, de)
        };
        Ok(Self {
            erb: Sparse::from_dense(&ew, ERB_HIGH, BINS - ERB_LOW),
            ierb: Sparse::from_dense(&iw, BINS - ERB_LOW, ERB_HIGH),
            en0: ConvF::load(&w, "encoder.en_convs.0", 9, C, 1, F0)?,
            en1: ConvF::load(&w, "encoder.en_convs.1", C, C, 2, F1)?,
            en_gt: [gt(2, 1, false)?, gt(3, 2, false)?, gt(4, 5, false)?],
            dp: [Dp::load(&w, "dpgrnn1")?, Dp::load(&w, "dpgrnn2")?],
            de_gt: [gt(0, 5, true)?, gt(1, 2, true)?, gt(2, 1, true)?],
            de3: DeconvF::load(&w, "decoder.de_convs.3", C, C, 2, F2, false)?,
            de4: DeconvF::load(&w, "decoder.de_convs.4", C, 2, 1, F1, true)?,
            feat0: vec![0.0; 3 * BINS],
            feat1: vec![0.0; 3 * F0],
            feat2: vec![0.0; 9 * F0],
            e0: vec![0.0; C * F1],
            e: [vec![0.0; C * F2], vec![0.0; C * F2], vec![0.0; C * F2], vec![0.0; C * F2]],
            t0: vec![0.0; C * F2],
            t1: vec![0.0; C * F2],
            d3: vec![0.0; C * F1],
            d4: vec![0.0; 2 * F0],
            m: vec![0.0; 2 * BINS],
        })
    }

    pub fn reset(&mut self) {
        for b in self.en_gt.iter_mut().chain(self.de_gt.iter_mut()) {
            b.reset();
        }
        for d in &mut self.dp {
            d.reset();
        }
    }

    /// 一帧：`re`/`im` 各 257 → 原地写成增强后的频谱。
    pub fn infer(&mut self, re: &mut [f32], im: &mut [f32]) {
        // 特征：mag, re, im
        for k in 0..BINS {
            self.feat0[k] = (re[k] * re[k] + im[k] * im[k] + 1e-12).sqrt();
            self.feat0[BINS + k] = re[k];
            self.feat0[2 * BINS + k] = im[k];
        }
        // ERB 压缩
        for c in 0..3 {
            let (src, dst) = (&self.feat0[c * BINS..(c + 1) * BINS], &mut self.feat1[c * F0..(c + 1) * F0]);
            dst[..ERB_LOW].copy_from_slice(&src[..ERB_LOW]);
            self.erb.run(&src[ERB_LOW..], &mut dst[ERB_LOW..]);
        }
        sfe3(&self.feat1, 3, F0, &mut self.feat2);
        // 编码
        self.en0.run(&self.feat2, &mut self.e0);
        let [e1, e2, e3, e4] = &mut self.e;
        self.en1.run(&self.e0, e1);
        self.en_gt[0].run(e1, e2);
        self.en_gt[1].run(e2, e3);
        self.en_gt[2].run(e3, e4);
        // DPGRNN ×2
        self.t0.copy_from_slice(e4);
        self.dp[0].run(&mut self.t0);
        self.dp[1].run(&mut self.t0);
        // 解码（跳连相加）
        add_into(&mut self.t0, &self.e[3]);
        self.de_gt[0].run(&self.t0, &mut self.t1);
        add_into(&mut self.t1, &self.e[2]);
        self.de_gt[1].run(&self.t1, &mut self.t0);
        add_into(&mut self.t0, &self.e[1]);
        self.de_gt[2].run(&self.t0, &mut self.t1);
        add_into(&mut self.t1, &self.e[0]);
        self.de3.run(&self.t1, &mut self.d3);
        add_into(&mut self.d3, &self.e0);
        self.de4.run(&self.d3, &mut self.d4);
        // ERB 还原
        for c in 0..2 {
            let (src, dst) = (&self.d4[c * F0..(c + 1) * F0], &mut self.m[c * BINS..(c + 1) * BINS]);
            dst[..ERB_LOW].copy_from_slice(&src[..ERB_LOW]);
            self.ierb.run(&src[ERB_LOW..], &mut dst[ERB_LOW..]);
        }
        // 复数比值掩码
        for k in 0..BINS {
            let (m0, m1) = (self.m[k], self.m[BINS + k]);
            let (r, i) = (re[k], im[k]);
            re[k] = r * m0 - i * m1;
            im[k] = i * m0 + r * m1;
        }
    }
}

fn add_into(x: &mut [f32], y: &[f32]) {
    for (a, b) in x.iter_mut().zip(y) {
        *a += b;
    }
}
