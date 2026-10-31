//! 细场景分类器：标准化 → 一层 tanh 隐层（[`NH`] 个）→ softmax（[`NC`] 类）。
//! 每次判断约 NF·NH + NH·NC ≈ 800 次乘加，可忽略。权重见 [`crate::weights`]。

use crate::features::NF;
use crate::weights::{B1, B2, MEAN, NC, NH, STD, W1, W2};

/// 噪音大类（细场景的第一维）。顺序 = 分类器输出顺序 = exp006 `recipes.json` 的 `fine` 键顺序。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoiseClass {
    Quiet,
    Hvac,
    Babble,
    Neighbor,
    Clatter,
    Traffic,
    Music,
}

impl NoiseClass {
    pub const ALL: [NoiseClass; NC] = [
        NoiseClass::Quiet,
        NoiseClass::Hvac,
        NoiseClass::Babble,
        NoiseClass::Neighbor,
        NoiseClass::Clatter,
        NoiseClass::Traffic,
        NoiseClass::Music,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            NoiseClass::Quiet => "quiet",
            NoiseClass::Hvac => "hvac",
            NoiseClass::Babble => "babble",
            NoiseClass::Neighbor => "neighbor",
            NoiseClass::Clatter => "clatter",
            NoiseClass::Traffic => "traffic",
            NoiseClass::Music => "music",
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

/// 算一次后验。不分配。
pub fn posterior(x: &[f32; NF], out: &mut [f32; NC]) {
    let mut z = [0.0f32; NF];
    for i in 0..NF {
        z[i] = (x[i] - MEAN[i]) / STD[i];
    }
    let mut h = [0.0f32; NH];
    for (j, hj) in h.iter_mut().enumerate() {
        let mut s = B1[j];
        for i in 0..NF {
            s += W1[j][i] * z[i];
        }
        *hj = s.tanh();
    }
    let mut mx = f32::NEG_INFINITY;
    for (c, o) in out.iter_mut().enumerate() {
        let mut s = B2[c];
        for j in 0..NH {
            s += W2[c][j] * h[j];
        }
        *o = s;
        mx = mx.max(s);
    }
    let mut sum = 0.0;
    for o in out.iter_mut() {
        *o = (*o - mx).exp();
        sum += *o;
    }
    for o in out.iter_mut() {
        *o /= sum;
    }
}
