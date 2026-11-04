//! 认人 fbank 对拍（#102，D82「同一处代码，各自切法」）：固定信号上三种配置的 fbank 输出逐位压成一个哈希，
//! 与改代码**之前**生成的基准比。红了 = 认人数值行为变了（红线：不许变），先停下报。
//! 有声纹模型（`VDN_SPEAKER_ONNX` 或默认路径）且开了 `onnx` 时，再对拍声纹向量。
//!
//! 基准在 `tests/fixtures/`；只在没改过 `fbank.rs` 的代码（`d23bfe66`）上生成：
//! `VDN_BLESS=1 cargo test -p vdn-speaker --features onnx --test golden`。
//! 改前的代码里窗种类叫 `vdn_speaker::fbank::WindowType::{Povey, Hanning}`，生成基准时只把下面 [`old_names`]
//! 里的两行换成旧名（`docs/plan/013_frontend-once.md` 第 12 节写明）。
//! 基准带平台标签（含系统主版本）：ln / cos / powf 末位随 macOS 大版本可能变，别的平台打印原因后跳过。
mod common;

use vdn_speaker::fbank::{Fbank, FbankConfig};
use vdn_types::{ContentId, MODEL_RATE};

const FBANK_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fbank_golden.txt"
);
#[cfg(feature = "onnx")]
const EMBED_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/embed_golden.txt"
);

/// 窗种类的名字（改前改后只差这里）。
mod old_names {
    pub const POVEY: vdn_types::Window = vdn_types::Window::KaldiPovey;
    pub const HANNING: vdn_types::Window = vdn_types::Window::KaldiHanning;
}

/// 3 s 确定性信号：带直流偏置的噪声 → 有调制的谐波音（像说话）→ 静音 → 削波谐波 → 噪声 + 瞬态。
fn signal() -> Vec<f32> {
    let sr = MODEL_RATE as f32;
    let n = 3 * MODEL_RATE as usize;
    let mut rng: u32 = 0x2468_ace1;
    let mut noise = move || {
        rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    };
    (0..n)
        .map(|i| {
            let t = i as f32 / sr;
            let w = noise();
            let harm = |f0: f32| -> f32 {
                (1..6)
                    .map(|k| (2.0 * std::f32::consts::PI * f0 * k as f32 * t).sin() / k as f32)
                    .sum()
            };
            match (t / 0.6) as usize {
                0 => 0.05 * w + 0.02,
                1 => {
                    0.01 * w
                        + 0.2 * harm(130.0) * (0.5 + 0.5 * (2.0 * std::f32::consts::PI * 4.0 * t).sin())
                }
                2 => 0.0,
                3 => (0.9 * harm(210.0)).clamp(-0.5, 0.5),
                _ => 0.03 * w + if i % 3000 < 30 { 0.6 * w } else { 0.0 },
            }
        })
        .collect()
}

/// 三种配置：WeSpeaker（汉明 · ×32768）· 3D-Speaker CAM++（povey · 不放大）· 汉宁 · 256 不补零 · 不预加重。
fn configs() -> [FbankConfig; 3] {
    let w = FbankConfig::wespeaker();
    [
        w,
        FbankConfig {
            window: old_names::POVEY,
            input_scale: 1.0,
            ..w
        },
        FbankConfig {
            window: old_names::HANNING,
            frame_length: 256,
            frame_shift: 128,
            preemph: 0.0,
            input_scale: 1.0,
            num_mel_bins: 40,
            ..w
        },
    ]
}

/// 基准行：`<内容 ID> <条数> <平台>`。设了 `VDN_BLESS` 就写入并返回 `None`；平台不同返回 `None` 并打印跳过。
fn golden(path: &str, got: ContentId, n: usize) -> Option<(ContentId, usize)> {
    let line = format!("{got} {n} {}", common::platform());
    if std::env::var_os("VDN_BLESS").is_some() {
        std::fs::write(path, format!("{line}\n")).expect("写基准");
        eprintln!("已写基准：{line}");
        return None;
    }
    let want = std::fs::read_to_string(path).expect("缺基准文件（见测试文件头怎么生成）");
    let mut it = want.split_whitespace();
    let id = it.next().and_then(ContentId::parse).expect("第 1 列是内容 ID");
    let cnt: usize = it.next().and_then(|s| s.parse().ok()).expect("第 2 列是条数");
    let plat = it.next().unwrap_or_default();
    if plat != common::platform() {
        eprintln!(
            "跳过：基准生成于 {plat}，本机是 {}；逐位对拍只在生成基准的平台上成立",
            common::platform()
        );
        return None;
    }
    Some((id, cnt))
}

#[test]
fn fbank_matches_golden() {
    let x = signal();
    let mut bytes = Vec::new();
    let mut frames = 0;
    for cfg in configs() {
        let mut fb = Fbank::new(cfg);
        let nf = fb.num_frames(x.len());
        let mut out = vec![0.0f32; nf * cfg.num_mel_bins];
        assert_eq!(fb.compute_into(&x, &mut out), nf);
        assert!(out.iter().all(|v| v.is_finite()));
        frames += nf;
        for v in &out {
            bytes.extend_from_slice(&v.to_bits().to_le_bytes());
        }
    }
    let got = ContentId::of(&bytes);
    let Some((id, n)) = golden(FBANK_FIXTURE, got, frames) else {
        return;
    };
    assert_eq!(frames, n, "帧数变了");
    assert_eq!(got, id, "认人 fbank 与改前不同（红线），先停下报");
}

/// 声纹向量逐位对拍。没有模型时跳过（设 `VDN_REQUIRE_MODELS` 则不许跳）。
#[cfg(feature = "onnx")]
#[test]
fn embedding_matches_golden() {
    use vdn_speaker::embed::{Embedder, default_model_path};
    let Some(path) = default_model_path() else {
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到声纹模型"
        );
        eprintln!("跳过：找不到声纹模型（VDN_SPEAKER_ONNX 或默认路径）");
        return;
    };
    let x = signal();
    let env = ort::init().with_name("vdn-speaker-golden").build().expect("ORT 环境");
    let mut e = Embedder::open(&env, &path, FbankConfig::wespeaker(), x.len()).expect("打开声纹模型");
    let mut bytes = Vec::new();
    let mut n = 0;
    // 整段一次 + 两段 1.5 s
    for seg in [&x[..], &x[..x.len() / 2], &x[x.len() / 2..]] {
        let v = e.embed(seg).expect("声纹");
        n += v.len();
        for f in &v {
            bytes.extend_from_slice(&f.to_bits().to_le_bytes());
        }
    }
    let got = ContentId::of(&bytes);
    let Some((id, cnt)) = golden(EMBED_FIXTURE, got, n) else {
        return;
    };
    assert_eq!(n, cnt, "声纹维数变了");
    assert_eq!(got, id, "声纹向量与改前不同（红线），先停下报");
}

/// 本 crate 的 `platform.rs` 与场景那份逐字相同（测试代码不能跨 crate 共用，tidy 不许 `#[path]` 跳出 crate）。
#[test]
fn platform_helper_matches_scene() {
    assert_eq!(
        include_str!("common/platform.rs"),
        include_str!("../../scene/tests/common/platform.rs"),
        "两份 platform.rs 不同：改一份必须同步另一份"
    );
}
