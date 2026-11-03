//! 算子写法的分派（`Kind`）与接口（`Kern`）；`Scalar` = 原写法。

use super::*;

// ───────────────────── 算子写法（研究 020）─────────────────────
// 模型结构（层的顺序、连接、状态）只在 `infer_impl` / `GtBlock::run` / `Dp::run` 里写一份；
// 各算子怎么算由 `Kern` 决定：`Scalar` = 原写法（逐位不变），`v2::V2<FAST>` = NEON 重写（研究 020）。

/// 用哪组算子写法（构造后设一次，`infer` 每帧开头按它分派一次）
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Scalar,
    V2,
    V2Fast,
    /// 研究 020 第二批（D79，按能效核写）：v2-fast 的算子 + 倒数代替除法 + 深度卷积 / LayerNorm / ERB 向量化
    V3,
    /// v3 + GRU 显式交错（第 4 条改写）
    V3i,
    /// v3 的深度卷积 / LN / ERB 向量化，非线性改回除法（v3 的倒数版实测更慢）
    V4,
    /// v4 + GRU 显式交错
    V4i,
}

impl Kind {
    /// 后端名 → 写法（后缀 -pf 是预取开关，不影响写法）。名字的正本在这里，bench 与 gtcrn-layers 都从这里取
    pub fn from_backend(k: &str) -> Option<Kind> {
        match k.strip_suffix("-pf").unwrap_or(k) {
            "native-v2" => Some(Kind::V2),
            "native-v2-fast" => Some(Kind::V2Fast),
            "native-v3" => Some(Kind::V3),
            "native-v3i" => Some(Kind::V3i),
            "native-v4" => Some(Kind::V4),
            "native-v4i" => Some(Kind::V4i),
            _ => None,
        }
    }
}

pub(crate) trait Kern {
    fn point(p: &Point, x: &[f32], y: &mut [f32], f: usize);
    fn conv(c: &mut ConvF, x: &[f32], y: &mut [f32]);
    fn deconv(d: &mut DeconvF, x: &[f32], y: &mut [f32]);
    fn gru(g: &Gru, x: &[f32], h: &mut [f32]);
    fn linear(l: &Linear, x: &[f32], y: &mut [f32]);
    /// GT 块的深度卷积 + PReLU（读 h1 与环形缓存，写 h2）
    fn depthwise(b: &mut GtBlock) {
        b.depthwise_scalar()
    }
    /// x += LN(z)（DPGRNN）
    fn ln_add(x: &mut [f32], z: &[f32], w: &[f32], b: &[f32]) {
        layer_norm_add(x, z, w, b)
    }
    /// TRA 注意力里的标量 sigmoid。原写法读全局开关 FAST（native-fast）；v2 / v3 / v4 固定用 libm，
    /// 不受同一进程里先后开的其他后端影响（codex 020 第 1 轮 #2）
    fn sig1(x: f32) -> f32 {
        sigmoid(x)
    }
    /// DPGRNN 的全连接是否交给 Accelerate（native-blas）。原写法读全局开关 BLAS；v2 / v3 / v4 固定不走
    fn use_blas() -> bool {
        blas()
    }
    /// ERB 稀疏矩阵乘
    fn sparse(s: &Sparse, x: &[f32], y: &mut [f32]) {
        s.run(x, y)
    }
    /// 4 条互相独立的 GRU 各走一步（沿频率 GRU：2 组 × 正 / 反向）。默认逐条调 `gru`
    fn gru4(gs: [&Gru; 4], xs: [&[f32]; 4], hs: &mut [[f32; 4]; 4]) {
        for ((g, x), h) in gs.into_iter().zip(xs).zip(hs.iter_mut()) {
            Self::gru(g, x, h);
        }
    }
    /// 同一个 GRU 在 n 行上各走一步，行间互相独立（沿时间 GRU：33 个频点）。
    /// 第 r 行的输入在 `xs[r·stride ..]`、状态在 `hs[r·stride ..]`。默认逐行调 `gru`
    fn gru_rows(g: &Gru, xs: &[f32], hs: &mut [f32], stride: usize, n: usize) {
        for r in 0..n {
            Self::gru(g, &xs[r * stride..r * stride + g.i], &mut hs[r * stride..r * stride + g.h]);
        }
    }
}

/// 原写法
pub(crate) struct Scalar;
impl Kern for Scalar {
    #[inline(always)]
    fn point(p: &Point, x: &[f32], y: &mut [f32], f: usize) {
        p.run(x, y, f)
    }
    #[inline(always)]
    fn conv(c: &mut ConvF, x: &[f32], y: &mut [f32]) {
        c.run(x, y)
    }
    #[inline(always)]
    fn deconv(d: &mut DeconvF, x: &[f32], y: &mut [f32]) {
        d.run(x, y)
    }
    #[inline(always)]
    fn gru(g: &Gru, x: &[f32], h: &mut [f32]) {
        g.step(x, h)
    }
    #[inline(always)]
    fn linear(l: &Linear, x: &[f32], y: &mut [f32]) {
        l.run(x, y)
    }
}
