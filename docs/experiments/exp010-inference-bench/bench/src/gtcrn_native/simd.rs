//! 研究 020：NEON 非线性小函数（v2 / v3 共用）：向量 exp、sigmoid、tanh、PReLU。
//! 非线性算法由 `NL` 选（常量与倒数函数的正本在 v2.rs）。

use super::v2::{NL_DIV, NL_RCP, vrecip};
use core::arch::aarch64::*;

#[inline(always)]
unsafe fn lanes(x: float32x4_t, f: impl Fn(f32) -> f32) -> float32x4_t {
    let mut a = [0.0f32; 4];
    unsafe {
        vst1q_f32(a.as_mut_ptr(), x);
        for v in &mut a {
            *v = f(*v);
        }
        vld1q_f32(a.as_ptr())
    }
}

/// exp：n = round(x·log2e)，r = x − n·ln2（ln2 拆两段），p(r) 用 Cephes expf 系数，结果 = 2^n · p
#[inline(always)]
pub(super) unsafe fn vexp(x: float32x4_t) -> float32x4_t {
    unsafe {
        let x = vminq_f32(vmaxq_f32(x, vdupq_n_f32(-87.0)), vdupq_n_f32(88.0));
        let n = vrndnq_f32(vmulq_n_f32(x, std::f32::consts::LOG2_E));
        let r = vfmsq_f32(x, n, vdupq_n_f32(0.693_359_4));
        let r = vfmsq_f32(r, n, vdupq_n_f32(-2.121_944_4e-4));
        let mut p = vdupq_n_f32(1.987_569_1e-4);
        p = vfmaq_f32(vdupq_n_f32(1.398_199_9e-3), p, r);
        p = vfmaq_f32(vdupq_n_f32(8.333_452e-3), p, r);
        p = vfmaq_f32(vdupq_n_f32(4.166_579_6e-2), p, r);
        p = vfmaq_f32(vdupq_n_f32(1.666_666_5e-1), p, r);
        p = vfmaq_f32(vdupq_n_f32(5.000_000_1e-1), p, r);
        let y = vfmaq_f32(vaddq_f32(r, vdupq_n_f32(1.0)), p, vmulq_f32(r, r));
        let e = vreinterpretq_f32_s32(vshlq_n_s32::<23>(vaddq_s32(vcvtq_s32_f32(n), vdupq_n_s32(127))));
        vmulq_f32(e, y)
    }
}

#[inline(always)]
pub(super) unsafe fn sig4<const NL: u8>(x: float32x4_t) -> float32x4_t {
    unsafe {
        if NL == NL_RCP {
            let one = vdupq_n_f32(1.0);
            let x = vminq_f32(vmaxq_f32(x, vdupq_n_f32(-87.0)), vdupq_n_f32(87.0));
            vrecip(vaddq_f32(one, vexp(vnegq_f32(x))))
        } else if NL == NL_DIV {
            // 输入夹到 ±87：x < −87 时 1/(1+e^{−x}) 会落进次正规数（单元测试抓到 6e-39），夹后最小 1.6e-38 仍是正规数；
            // 被夹的部分真值 < 1.7e-38，绝对差 < 2e-38
            let one = vdupq_n_f32(1.0);
            let x = vminq_f32(vmaxq_f32(x, vdupq_n_f32(-87.0)), vdupq_n_f32(87.0));
            vdivq_f32(one, vaddq_f32(one, vexp(vnegq_f32(x))))
        } else {
            lanes(x, |v| 1.0 / (1.0 + (-v).exp()))
        }
    }
}

#[inline(always)]
pub(super) unsafe fn tanh4<const NL: u8>(x: float32x4_t) -> float32x4_t {
    unsafe {
        if NL == NL_RCP {
            let one = vdupq_n_f32(1.0);
            let e = vexp(vaddq_f32(x, x));
            vfmsq_f32(one, vdupq_n_f32(2.0), vrecip(vaddq_f32(e, one)))
        } else if NL == NL_DIV {
            // tanh(x) = 1 − 2 / (e^{2x} + 1)
            let one = vdupq_n_f32(1.0);
            let e = vexp(vaddq_f32(x, x));
            vsubq_f32(one, vdivq_f32(vdupq_n_f32(2.0), vaddq_f32(e, one)))
        } else {
            lanes(x, f32::tanh)
        }
    }
}

#[inline(always)]
pub(super) unsafe fn prelu4(x: float32x4_t, a: f32) -> float32x4_t {
    unsafe { vbslq_f32(vcgeq_f32(x, vdupq_n_f32(0.0)), x, vmulq_n_f32(x, a)) }
}
