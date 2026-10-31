//! 声纹向量小工具（不受 feature 控制，embed / enroll / judge 共用）。

/// 原地 L2 归一化；模为 0 或非有限时不动。
pub fn l2_normalize(v: &mut [f32]) {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if n > 0.0 && n.is_finite() {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

/// 两个已归一化向量的余弦相似度。
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// 全部分量有限。
pub fn all_finite(v: &[f32]) -> bool {
    v.iter().all(|x| x.is_finite())
}

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
