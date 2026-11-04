//! 评测器频谱计算对拍（#104，D82「所有频谱计算放在一个地方」）：评测器 4 处改调 `vdn-dsp` 统一模块后，
//! 声谱图 · ESTOI · 参考降噪器输出 · 全部 CI 层 verifier 的分数，逐位与改代码**之前**相同。红了 = 评分变了（红线），先停下报。
//!
//! 4 处：`analysis.rs` 声谱图 · `verifiers/estoi.rs` · `verifiers/speech.rs` 的 V-WEAK 分带电平 · `system.rs` 参考降噪器。
//! 每组压成一个内容 ID（f32 / f64 逐个 `to_bits`），基准在 `tests/fixtures/spectrum_golden.txt`。
//! 基准只在没改过这 4 处的代码（`f2782cf4`）上生成：
//! `VDN_BLESS=1 cargo test -p vdn-eval --release --test spectrum_golden`（只用公开接口；生成基准时本文件与现在只差一处：
//! ESTOI 内部采样率当时写成字面量 `10_000`，`estoi::FS` 是之后才公开的，值相同）。
//! 基准带平台标签（含系统主版本）：cos / ln / powf 末位随 macOS 大版本可能变，别的平台打印原因后跳过。
#[path = "common/platform.rs"]
mod platform;
use platform::platform;

use vdn_eval::analysis::{Spectrogram, masks};
use vdn_eval::synth::{Clip, InterfMode, InterfSpec, NoiseKind, SceneSpec, TalkerSpec, render};
use vdn_eval::system::{SystemKind, SystemSpec, run};
use vdn_eval::verifiers::estoi::{self, estoi};
use vdn_eval::verifiers::{Ctx, Params, REGISTRY, eval};
use vdn_types::{ContentId, MODEL_RATE};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/spectrum_golden.txt"
);

/// 确定性合成片段：咖啡馆噪声 + 主讲人 + 一个轮流说话的干扰者，10 s（V-WEAK 要 ≥ 20 个 125 ms 有声帧）。
fn clip(seed: u64, snr_db: f64) -> Clip {
    render(&SceneSpec {
        dur_s: 10.0,
        seed,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 160.0,
            tonal: true,
        }),
        owner_start_s: 1.0,
        pause_s: (0.3, 0.9),
        interferers: vec![InterfSpec {
            mode: InterfMode::Alternate,
            sir_db: 6.0,
            f0_lo: 180.0,
            f0_hi: 240.0,
        }],
        noise: NoiseKind::Cafe,
        snr_db,
        tail_silence_s: 0.5,
    })
}

#[derive(Default)]
struct Acc {
    bytes: Vec<u8>,
    n: usize,
}

impl Acc {
    fn f32s(&mut self, v: &[f32]) {
        for x in v {
            self.bytes.extend_from_slice(&x.to_bits().to_le_bytes());
        }
        self.n += v.len();
    }
    fn f64(&mut self, x: f64) {
        self.bytes.extend_from_slice(&x.to_bits().to_le_bytes());
        self.n += 1;
    }
    fn text(&mut self, s: &str) {
        self.bytes.extend_from_slice(s.as_bytes());
        self.bytes.push(0);
    }
    fn line(&self, name: &str) -> String {
        format!(
            "{name} {} {} {}",
            ContentId::of(&self.bytes),
            self.n,
            platform()
        )
    }
}

fn ref_spec() -> SystemSpec {
    SystemSpec {
        kind: SystemKind::Ref,
        safety: true,
        faults: Vec::new(),
    }
}

/// 四组：(名字, 内容 ID 行)。
fn lines() -> Vec<String> {
    let clips = [clip(7, 5.0), clip(11, 15.0)];
    let (mut sg, mut es, mut rw, mut vf) = (
        Acc::default(),
        Acc::default(),
        Acc::default(),
        Acc::default(),
    );
    for c in &clips {
        // 1. 声谱图（成绩单里用 512 / 256；另比一组 1024 / 256）
        for (n_fft, hop) in [(512, 256), (1024, 256)] {
            for x in [&c.mix, &c.clean] {
                let s = Spectrogram::new(x, n_fft, hop);
                for f in &s.frames {
                    sg.f32s(f);
                }
            }
        }
        // 3. 参考降噪器
        let r = run(&ref_spec(), &c.mix).expect("参考降噪器");
        rw.f32s(&r.out);
        rw.f32s(&r.raw);
        rw.text(&format!("{:?}{:?}", r.voiced, r.bypass));
        // 2. ESTOI：16 kHz（先重采样）与 10 kHz（不重采样）两条路
        for (a, b) in [(&c.clean, &c.mix), (&c.clean, &r.out), (&c.clean, &c.clean)] {
            for sr in [MODEL_RATE, estoi::FS as u32] {
                es.f64(estoi(a, b, sr).unwrap_or(f64::NAN));
            }
        }
        // 4. 全部 verifier（V-FRAME 量的是耗时，不确定，不比）
        let noise_run = run(&ref_spec(), &c.noise).expect("参考降噪器（只有噪音）");
        let m = masks(&c.truth, &c.mix, 0.03);
        let ctx = Ctx {
            clip: c,
            out: &r.out,
            run: Some(&r),
            masks: &m,
            noise_out: Some(&noise_run.out),
            ci: true,
        };
        for info in REGISTRY.iter().filter(|i| i.id != "V-FRAME") {
            let s = eval(info.id, &ctx, &Params::default());
            vf.text(s.id);
            vf.f64(s.value.unwrap_or(f64::NAN));
            vf.f64(s.threshold.unwrap_or(f64::NAN));
            vf.text(&format!("{:?}{}", s.grade, s.verdict.as_str()));
            for (k, v) in &s.extra {
                vf.text(k);
                vf.f64(*v);
            }
            vf.text(&s.note);
        }
    }
    vec![
        sg.line("spectrogram"),
        es.line("estoi"),
        rw.line("ref_wiener"),
        vf.line("verifiers"),
    ]
}

#[test]
fn eval_spectra_match_golden() {
    let got = lines();
    if std::env::var_os("VDN_BLESS").is_some() {
        std::fs::write(FIXTURE, got.join("\n") + "\n").expect("写基准");
        eprintln!("已写基准：\n{}", got.join("\n"));
        return;
    }
    let want = std::fs::read_to_string(FIXTURE).expect("缺基准文件（见测试文件头怎么生成）");
    let want: Vec<&str> = want.lines().collect();
    assert_eq!(want.len(), got.len(), "基准行数不对");
    let plat = want[0].rsplit(' ').next().unwrap_or_default();
    if plat != platform() {
        eprintln!(
            "跳过：基准生成于 {plat}，本机是 {}；逐位对拍只在生成基准的平台上成立",
            platform()
        );
        return;
    }
    for (w, g) in want.iter().zip(&got) {
        assert_eq!(*w, g.as_str(), "评测器频谱计算与改前不同（红线），先停下报");
    }
}

/// 本 crate 的 `platform.rs` 与场景那份逐字相同（测试代码不能跨 crate 共用）。
#[test]
fn platform_helper_matches_scene() {
    assert_eq!(
        include_str!("common/platform.rs"),
        include_str!("../../scene/tests/common/platform.rs"),
        "两份 platform.rs 不同：改一份必须同步另一份"
    );
}
