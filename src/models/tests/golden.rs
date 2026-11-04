//! GTCRN 对拍（013 第 8 节第 1 条；成绩单重跑靠它可比）：固定约 3 s 输入（截到整 hop）逐 hop 过真模型，
//! 输出 f32 逐位压成一个内容 ID，与改 `gtcrn.rs` **之前**生成的基准比。红了 = GTCRN 数值行为变了（红线），先停下报。
//!
//! 基准 `tests/fixtures/gtcrn_golden.txt` 一行：`<输出内容 ID> <样本数> <权重内容 ID> <平台>`。
//! 生成（只在没改过 `gtcrn.rs` 的代码上）：`VDN_BLESS=1 VDN_GTCRN_ONNX=<gtcrn_simple.onnx> cargo test -p vdn-models --test golden`。
//! 找不到模型文件时同 `smoke.rs` 跳过（设 `VDN_REQUIRE_MODELS` 不许跳）；权重或平台和基准不同时跳过并打印原因
//! （ORT 与系统数学库在别的平台上末位可能不同）。平台标签含系统主版本（如 `aarch64-macos-27`）：
//! 换 macOS 大版本数学库末位就可能变，逐位对拍只在生成基准的那个平台上成立，CI 机型不同时跳过。
//! 另一条对拍引擎最终输出（48 kHz `Pipeline`，基准 `tests/fixtures/pipeline_golden.txt`），因为成绩单测的是它。
//! 本文件只用改前就有的接口，才能在 main 的原样源码上生成基准；借谱的检查在 `tap.rs`。
use vdn_engine::{Engine, Pipeline, SafetyConfig};
use vdn_models::{Gtcrn, ModelKind, artifact_id};
use vdn_types::{ContentId, IO_RATE, MODEL_RATE, Model};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gtcrn_golden.txt"
);
const FIXTURE_PIPELINE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pipeline_golden.txt"
);

/// 3 s：噪声 + 一强一弱交替的 150 Hz 正弦 + 一段静音，只用乘加和 LCG，跨平台逐位一致。
fn input() -> Vec<f32> {
    let n = 3 * MODEL_RATE as usize;
    let mut rng: u32 = 0x0BAD_5EED;
    // 二阶递推正弦振荡器：y[n] = c·y[n-1] − y[n-2]，c = 2cos(2π·150/16000)（字面量，不调 cos）
    let c = 1.996_531_2_f32;
    let (mut y1, mut y2) = (0.058_870_8_f32, 0.0_f32);
    (0..n)
        .map(|i| {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let w = (rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0;
            let y = c * y1 - y2;
            (y2, y1) = (y1, y);
            let gate = if (i / 2000) % 2 == 0 { 1.0 } else { 0.3 };
            if (MODEL_RATE as usize * 2..MODEL_RATE as usize * 2 + 4000).contains(&i) {
                0.0
            } else {
                0.05 * w + 0.3 * gate * y
            }
        })
        .collect()
}

#[path = "common/platform.rs"]
mod platform;
use platform::platform;

/// 平台标签的两份实现（本 crate 与 `vdn-scene` 测试各一份）必须逐字相同。
#[test]
fn platform_helper_matches_scene() {
    assert_eq!(
        include_str!("common/platform.rs"),
        include_str!("../../scene/tests/common/platform.rs"),
        "两份 platform.rs 不同：改一份必须同步另一份"
    );
}

/// 找模型文件；找不到时按 `VDN_REQUIRE_MODELS` 决定失败还是跳过。
fn model_path() -> Option<std::path::PathBuf> {
    let p = ModelKind::Gtcrn.default_path();
    if p.is_none() {
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到模型文件"
        );
        eprintln!("跳过：找不到 GTCRN 模型文件（设 VDN_GTCRN_ONNX）");
    }
    p
}

/// 输出逐位压成内容 ID，与基准 `<ID> <样本数> <权重 ID> <平台>` 比；设 `VDN_BLESS` 时写基准。
fn check(fixture: &str, y: &[f32], weights: ContentId, what: &str) {
    let bytes: Vec<u8> = y.iter().flat_map(|v| v.to_bits().to_le_bytes()).collect();
    let got = ContentId::of(&bytes);
    let line = format!("{got} {} {weights} {}", y.len(), platform());
    if std::env::var_os("VDN_BLESS").is_some() {
        std::fs::write(fixture, format!("{line}\n")).expect("写基准");
        eprintln!("已写基准：{line}");
        return;
    }
    let want = std::fs::read_to_string(fixture).expect("缺基准文件（见文件头怎么生成）");
    let f: Vec<&str> = want.split_whitespace().collect();
    let [id, n, w, plat] = f.as_slice() else {
        panic!("基准格式不对：{want}");
    };
    if *w != weights.to_string() || *plat != platform() {
        eprintln!(
            "跳过：基准的权重 / 平台是 {w} / {plat}，本机是 {weights} / {}；逐位对拍只在生成基准的平台上成立",
            platform()
        );
        return;
    }
    assert_eq!(n.parse::<usize>().ok(), Some(y.len()), "样本数变了");
    let id = ContentId::parse(id).expect("基准第 1 列是 16 位十六进制");
    assert_eq!(got, id, "{what}与改前不同（红线），先停下报");
}

#[test]
fn gtcrn_output_matches_golden() {
    let Some(path) = model_path() else { return };
    let weights = artifact_id(&path).expect("读权重");
    let mut m = Gtcrn::open(&path).expect("打开 GTCRN");
    let hop = m.hop();
    // 截到整 hop：哈希里的每个点都真过了模型（不留没处理的尾巴）
    let mut x = input();
    x.truncate(x.len() / hop * hop);
    let mut y = vec![0.0f32; x.len()];
    for (xi, yi) in x.chunks_exact(hop).zip(y.chunks_exact_mut(hop)) {
        m.process(xi, yi).expect("GTCRN 处理");
    }
    check(FIXTURE, &y, weights, "GTCRN 输出");
}

/// 成绩单测的是引擎最终输出（含门控、混回原声、48 kHz 重采样），不只是模型输出：
/// GTCRN 走 48 kHz `Pipeline`（默认安全参数），块长轮换 96 / 256 / 4096，整段输出逐位与改前比。
#[test]
fn pipeline_output_matches_golden() {
    let Some(path) = model_path() else { return };
    let weights = artifact_id(&path).expect("读权重");
    let m = Gtcrn::open(&path).expect("打开 GTCRN");
    let mut p =
        Pipeline::new(Engine::new(m, SafetyConfig::default()), IO_RATE).expect("48 kHz 应可用");
    // 同一段确定性信号按 3 倍拉长成 48 kHz（每点重复 3 次，只用复制，跨平台一致）
    let x: Vec<f32> = input().iter().flat_map(|&v| [v, v, v]).collect();
    let mut y = vec![0.0f32; x.len()];
    let mut i = 0;
    for n in [96usize, 256, 4096].iter().cycle() {
        if i >= x.len() {
            break;
        }
        let end = (i + n).min(x.len());
        p.process(&x[i..end], &mut y[i..end]);
        i = end;
    }
    check(FIXTURE_PIPELINE, &y, weights, "引擎最终输出");
}
