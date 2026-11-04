//! 场景测试共用：确定性的测试信号、按判断周期跑一遍、把结果压成一个哈希（对拍用）。
#![allow(dead_code)] // 各测试文件只用其中一部分

use vdn_scene::{Judgement, SceneAnalyzer};
use vdn_types::{ContentId, MODEL_RATE};

pub const HOP: usize = 256;
/// 约 0.5 s 判断一次（同 `examples/scene_offline.rs`）
pub const TICK_HOPS: usize = 31;

/// 12 s 确定性信号，几段不同的声音轮流出现，让 35 维特征和场景判断都动起来：
/// 白噪（稳态）→ 噪声 + 有调制的谐波音（像说话）→ 静音 → 低频轰鸣 → 削波的谐波音 → 噪声 + 瞬态敲击。
pub fn signal() -> Vec<f32> {
    let sr = MODEL_RATE as f32;
    let n = 12 * MODEL_RATE as usize;
    let mut rng: u32 = 0x1234_5678;
    let mut noise = move || {
        // LCG：确定、跨平台一致
        rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    };
    (0..n)
        .map(|i| {
            let t = i as f32 / sr;
            let seg = (t / 2.0) as usize;
            let w = noise();
            let harm = |f0: f32| -> f32 {
                (1..6)
                    .map(|k| (2.0 * std::f32::consts::PI * f0 * k as f32 * t).sin() / k as f32)
                    .sum()
            };
            match seg {
                0 => 0.05 * w,
                1 => 0.02 * w + 0.2 * harm(140.0) * (0.5 + 0.5 * (2.0 * std::f32::consts::PI * 4.0 * t).sin()),
                2 => 0.0,
                3 => 0.01 * w + 0.3 * (2.0 * std::f32::consts::PI * 60.0 * t).sin(),
                4 => (0.8 * harm(220.0)).clamp(-0.5, 0.5),
                _ => 0.03 * w + if i % 4000 < 40 { 0.6 * w } else { 0.0 },
            }
        })
        .collect()
}

/// 每次判断记下：35 维特征 · 未平滑后验 · 本次原始细场景 · 生效细场景。
pub fn record(j: &Judgement) -> Vec<u8> {
    let mut b = Vec::new();
    for v in j.features.iter().chain(j.raw.iter()) {
        b.extend_from_slice(&v.to_bits().to_le_bytes());
    }
    b.extend_from_slice(&j.label.fine.0.to_le_bytes());
    b.extend_from_slice(&j.committed.1.0.to_le_bytes());
    b
}

/// 用 `push` 逐 hop 喂 `x`，每 [`TICK_HOPS`] 判断一次，返回全部判断。
pub fn run(x: &[f32], mut push: impl FnMut(&mut SceneAnalyzer, &[f32])) -> Vec<Judgement> {
    let mut sa = SceneAnalyzer::new(MODEL_RATE, HOP);
    let mut out = Vec::new();
    for (k, h) in x.chunks_exact(HOP).enumerate() {
        push(&mut sa, h);
        if (k + 1) % TICK_HOPS == 0 {
            out.push(sa.judge());
        }
    }
    out
}

/// 全部判断压成一个内容 ID。
pub fn digest(js: &[Judgement]) -> ContentId {
    let bytes: Vec<u8> = js.iter().flat_map(record).collect();
    ContentId::of(&bytes)
}

/// 基准文件的平台标签：sin / ln / exp 等由系统数学库算，不同平台末位可能不同，基准只在同平台上比。
pub fn platform() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

/// 基准行：`<内容 ID> <条数> <平台>`。设了 `VDN_BLESS` 就写入并返回 `None`；平台不同返回 `None` 并打印跳过。
/// 否则返回 (期望 ID, 期望条数)。
pub fn golden(path: &str, got: ContentId, n: usize) -> Option<(ContentId, usize)> {
    let line = format!("{got} {n} {}", platform());
    if std::env::var_os("VDN_BLESS").is_some() {
        std::fs::write(path, format!("{line}\n")).expect("写基准");
        eprintln!("已写基准：{line}");
        return None;
    }
    let want = std::fs::read_to_string(path).expect("缺基准文件（见测试文件头怎么生成）");
    let mut it = want.split_whitespace();
    let id = it.next().and_then(ContentId::parse).expect("基准第 1 列是 16 位十六进制");
    let cnt: usize = it.next().and_then(|s| s.parse().ok()).expect("基准第 2 列是条数");
    let plat = it.next().unwrap_or_default();
    if plat != platform() {
        eprintln!("跳过：基准生成于 {plat}，本机是 {}", platform());
        return None;
    }
    Some((id, cnt))
}
