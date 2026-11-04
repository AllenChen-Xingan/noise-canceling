//! vdn-polish：修音链。降噪模型之后、输出之前的零前瞻修整，针对人听测标出的四种毛病：
//!
//! | 毛病 | 节点 | 做法 | 附加延迟 | 算力（16 kHz、hop 256，粗估） |
//! |---|---|---|---|---|
//! | 金属声 / 音乐噪声 | [`SpectralPolish`] 平滑 | 隐含增益 `|wet|/|dry|` 在无语音时做时间 + 频率平滑 | 0（挂在模型自己的 STFT 上） | 每帧约 257 点 × 十几次乘加 |
//! | 真空感 | [`SpectralPolish`] 下限 + 舒适噪声 | 增益下限；缺口处补随机相位、按稳态底噪谱塑形的舒适噪声 | 0 | 同上，含 257 个伪随机数 |
//! | 发闷 | [`SpectralPolish`] 去发闷均衡 | 语音帧上比较「带噪减底噪」与「降噪后」的 2–8 kHz 四个频带能量，慢慢补回，封顶 | 0 | 同上 |
//! | 音量忽大忽小 | [`Agc`] | 只在有人声时跟踪语音电平，限速调增益；分子帧的零前瞻限幅 | 0（只用本 hop 已在手的样本） | 每样本 2 次乘加 |
//!
//! **零前瞻**：所有节点只用当前帧与过去帧。频谱节点不自带 STFT——它吃的是模型适配器已经算好的
//! 带噪频谱与降噪后频谱（`specs/rust/architecture.md` §3.1「共用一次 STFT 以后用扩展 trait 补」，扩展 trait 见 [`spectral_stage`]，时域 `Stage` 用 `vdn-types`）。
//! GTCRN 的适配器本来就按正本 `vdn_types::FRONTEND` 做 STFT，挂上去不加延迟；FastEnhancer-M 现用 wav2wav 图（STFT 在图里），
//! 拿不到频谱，要么换非 wav2wav 导出、要么只用时域节点（[`Agc`]）——见 exp006 README「缺口」。
//! 若某个模型只能另做一次 STFT 才用得上频谱节点，那次 STFT 会多 `n_fft - hop` 个样本（512/256 时 16 ms），必须记进延迟账。
//!
//! 规矩：`new()` 里一次性分配；每帧方法不分配、不加锁、不 panic（`tests/no_alloc.rs` 证明不分配）。

#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::todo
    )
)]
pub mod agc;
pub mod chain;
pub mod params;
pub mod spectral;
pub mod spectral_stage;

pub use agc::Agc;
pub use chain::PolishChain;
pub use params::PolishParams;
pub use spectral::SpectralPolish;

/// 分贝 → 功率比。
#[inline]
pub(crate) fn db_to_pow(db: f32) -> f32 {
    10f32.powf(db / 10.0)
}

/// 功率 → 分贝（带下限，避免 log(0)）。
#[inline]
pub(crate) fn pow_to_db(p: f32) -> f32 {
    10.0 * p.max(1e-20).log10()
}
