//! GTCRN 对拍（013 第 8 节第 1 条；成绩单重跑靠它可比）：固定 3 s 输入逐 hop 过真模型，
//! 输出 f32 逐位压成一个内容 ID，与改 `gtcrn.rs` **之前**生成的基准比。红了 = GTCRN 数值行为变了（红线），先停下报。
//!
//! 基准 `tests/fixtures/gtcrn_golden.txt` 一行：`<输出内容 ID> <样本数> <权重内容 ID> <平台>`。
//! 生成（只在没改过 `gtcrn.rs` 的代码上）：`VDN_BLESS=1 VDN_GTCRN_ONNX=<gtcrn_simple.onnx> cargo test -p vdn-models --test golden`。
//! 找不到模型文件时同 `smoke.rs` 跳过（设 `VDN_REQUIRE_MODELS` 不许跳）；权重或平台和基准不同时跳过并打印原因
//! （ORT 与系统数学库在别的平台上末位可能不同）。
//! 本文件只用改前就有的接口，才能在 main 的原样源码上生成基准；借谱的检查在 `tap.rs`。
use vdn_models::{Gtcrn, ModelKind, artifact_id};
use vdn_types::{ContentId, MODEL_RATE, Model};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/gtcrn_golden.txt"
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

fn platform() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

#[test]
fn gtcrn_output_matches_golden() {
    let Some(path) = ModelKind::Gtcrn.default_path() else {
        assert!(
            std::env::var_os("VDN_REQUIRE_MODELS").is_none(),
            "VDN_REQUIRE_MODELS 已设，但找不到模型文件"
        );
        eprintln!("跳过：找不到 GTCRN 模型文件（设 VDN_GTCRN_ONNX）");
        return;
    };
    let weights = artifact_id(&path).expect("读权重");
    let mut m = Gtcrn::open(&path).expect("打开 GTCRN");
    let x = input();
    let hop = m.hop();
    let mut y = vec![0.0f32; x.len()];
    for (xi, yi) in x.chunks_exact(hop).zip(y.chunks_exact_mut(hop)) {
        m.process(xi, yi).expect("GTCRN 处理");
    }
    let bytes: Vec<u8> = y.iter().flat_map(|v| v.to_bits().to_le_bytes()).collect();
    let got = ContentId::of(&bytes);
    let line = format!("{got} {} {weights} {}", y.len(), platform());
    if std::env::var_os("VDN_BLESS").is_some() {
        std::fs::write(FIXTURE, format!("{line}\n")).expect("写基准");
        eprintln!("已写基准：{line}");
        return;
    }
    let want = std::fs::read_to_string(FIXTURE).expect("缺基准文件（见文件头怎么生成）");
    let f: Vec<&str> = want.split_whitespace().collect();
    let [id, n, w, plat] = f.as_slice() else {
        panic!("基准格式不对：{want}");
    };
    if *w != weights.to_string() || *plat != platform() {
        eprintln!(
            "跳过：基准的权重 / 平台是 {w} / {plat}，本机是 {weights} / {}",
            platform()
        );
        return;
    }
    assert_eq!(n.parse::<usize>().ok(), Some(y.len()), "样本数变了");
    let id = ContentId::parse(id).expect("基准第 1 列是 16 位十六进制");
    assert_eq!(got, id, "GTCRN 输出与改前不同（红线），先停下报");
}
