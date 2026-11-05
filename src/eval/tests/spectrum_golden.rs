//! 评测器频谱计算对拍（#104，D82「所有频谱计算放在一个地方」）：评测器 4 处改调 `vdn-dsp` 统一模块后，
//! 声谱图 · ESTOI · 参考降噪器输出 · 全部 CI 层 verifier 的分数 · 边界输入（别的采样率、短于窗长、帧端点、空输入），
//! 逐位与改代码**之前**相同。红了 = 评分变了（红线），先停下报。
//!
//! 4 处：`analysis.rs` 声谱图 · `verifiers/estoi.rs` · `verifiers/speech.rs` 的 V-WEAK 分带电平 · `system.rs` 参考降噪器。
//! 每组压成一个内容 ID（f32 / f64 逐个 `to_bits`），基准在 `tests/fixtures/spectrum_golden.txt`。
//! 基准只在没改过这 4 处的代码（`f2782cf4`）上生成：`git archive f2782cf4` 导出到临时目录，放进本文件与
//! `common/platform.rs`（与现在逐字相同），旧源码只改一处——`verifiers/estoi.rs` 的 `const FS` 改成 `pub const FS`
//! （只改可见性，本文件要引用它），单独 target 目录里跑
//! `VDN_BLESS=1 cargo test -p vdn-eval --release --test spectrum_golden`（`docs/plan/013_frontend-once.md` 第 13 节）。
//! 基准带平台标签（含系统主版本）：cos / ln / powf 末位随 macOS 大版本可能变，别的平台打印原因后跳过。
//!
//! **「verifiers」组只守建基准时的项和字段**（[`FROZEN`]，#111 起，大总管定）：之后新加的 verifier 与旁报
//! 由各自的单元测试守，不让这条红线每加一列就变红；清单里的项和字段一个都不许少（`frozen_items_and_fields_all_present`）。
//! #111 改了 V-BGSTAB 的主值（扣掉输入侧偏差，025 B3）：本文件的合成片段停顿里干净人声是数字零，偏差恰为 0，
//! 基准文件未变；证明「其余项逐位不变」的做法是去掉 V-BGSTAB 后在改动前后各算一次内容 ID，两次相同。
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

/// 从 `start` 起截 `n` 个样本（真值区间一起平移、截断）。
fn excerpt(c: &Clip, start: usize, n: usize) -> Clip {
    let end = (start + n).min(c.len());
    let cut = |v: &[(usize, usize)]| -> Vec<(usize, usize)> {
        v.iter()
            .map(|&(a, b)| (a.clamp(start, end) - start, b.clamp(start, end) - start))
            .filter(|(a, b)| a < b)
            .collect()
    };
    let mut d = c.clone();
    for x in [&mut d.clean, &mut d.interf, &mut d.noise, &mut d.mix] {
        *x = x[start..end].to_vec();
    }
    d.truth.owner = cut(&c.truth.owner);
    d.truth.interf = cut(&c.truth.interf);
    d
}

/// 建基准时（`f2782cf4`，#104）登记表里的 verifier 与各自的旁报字段，按登记表顺序。
/// 「verifiers」组只守这些：之后新加的项和旁报（#111 起）由各自的单元测试守，不让这条红线每加一列就变红。
/// 清单里的项和字段一个都不许少（[`frozen_items_and_fields_all_present`]），防止悄悄删掉；只许改清单外的东西。
/// 怎么证明「原有读数逐位不变」：按本清单过滤后的内容 ID 必须等于 `fixtures/spectrum_golden.txt` 里的原值。
const FROZEN: &[(&str, &[&str])] = &[
    ("V-LAT", &["measured_algo_ms", "declared_algo_ms"]),
    ("V-FRAME", &[]),
    ("V-NAN", &["nonfinite", "over_4"]),
    ("V-MUTE", &[]),
    ("V-SWITCH", &[]),
    ("V-BGSTAB", &["signed_db", "frames"]),
    (
        "V-ONSET",
        &["onsets", "p90_onset_loss_db", "voiced_loss_db"],
    ),
    ("V-NR", &["frames", "snri_db"]),
    ("V-INTF", &["frames"]),
    ("V-SPLVL", &[]),
    ("V-MUSIC", &["kurt_in", "kurt_out", "isolated_peaks_per_s"]),
    ("V-VACUUM", &[]),
    // 调研 026（#119）：V-TIMBRE 换成 ERB 30 带的标准差，旁报字段随之换；基准的「verifiers」一行随之更新，
    // 「verifiers_sans_timbre」一行不变（证明其余 verifier 逐位不变）
    (
        "V-TIMBRE",
        &["slope_db", "max_db", "bands", "quiet_frames", "hi_db", "lo_db"],
    ),
    ("V-SIM-GATE", &["onset_miss", "noise_open", "owner_hops"]),
    ("V-SIM-FA", &["interf_only_hops"]),
    ("V-SIM-SWLAT", &[]),
    ("V-SIM-QUIET", &["quiet_hops"]),
    ("V-WEAK", &["delta_range_db", "frames"]),
    ("V-PUMPV", &["windows", "mean_db"]),
    ("V-CLICK", &["count"]),
    ("V-ESTOI", &["estoi_out", "estoi_mix"]),
    ("V-EAT-ASR", &[]),
    ("V-DNSMOS", &[]),
    ("V-NRMOS-SCOREQ", &[]),
    ("V-NRMOS-DISTILL", &[]),
];

fn frozen_keys(id: &str) -> Option<&'static [&'static str]> {
    FROZEN.iter().find(|(v, _)| *v == id).map(|(_, k)| *k)
}

/// 一个 verifier 的分数逐位记进 `acc`：只记 [`FROZEN`] 里登记的旁报字段（原顺序）。
/// QA 层项在这里没接模型、id 记为 `V-?`，没有旁报，原样记。
fn record(acc: &mut Acc, s: &vdn_eval::Score) {
    acc.text(s.id);
    acc.f64(s.value.unwrap_or(f64::NAN));
    acc.f64(s.threshold.unwrap_or(f64::NAN));
    acc.text(&format!("{:?}{}", s.grade, s.verdict.as_str()));
    let keep = frozen_keys(s.id).unwrap_or(&[]);
    for (k, v) in s.extra.iter().filter(|(k, _)| keep.contains(k)) {
        acc.text(k);
        acc.f64(*v);
    }
    acc.text(&s.note);
}

/// 在片段 `c` 上跑参考降噪器，按 `ids` 评分（`None` = 全部，V-FRAME 量耗时、不确定，不比）。
fn score(acc: &mut Acc, c: &Clip, ids: Option<&[&str]>) -> Vec<vdn_eval::Score> {
    let (r, noise_run) = match (run(&ref_spec(), &c.mix), run(&ref_spec(), &c.noise)) {
        (Ok(r), Ok(n)) => (r, n),
        (r, n) => {
            // 输入太短等：把报错也记进对拍（改前改后应一样）
            acc.text(&format!("{:?} {:?}", r.err(), n.err()));
            return Vec::new();
        }
    };
    let m = masks(&c.truth, &c.mix, 0.03);
    let ctx = Ctx {
        clip: c,
        out: &r.out,
        run: Some(&r),
        masks: &m,
        noise_out: Some(&noise_run.out),
        ci: true,
    };
    // 默认评分清单 = 登记表里属于 FROZEN 的项（保持登记表顺序）
    let all: Vec<&str> = REGISTRY
        .iter()
        .map(|i| i.id)
        .filter(|id| frozen_keys(id).is_some())
        .collect();
    let out: Vec<_> = ids
        .unwrap_or(&all)
        .iter()
        .filter(|id| **id != "V-FRAME")
        .map(|id| eval(id, &ctx, &Params::default()))
        .collect();
    out.iter().for_each(|s| record(acc, s));
    out
}

/// V-WEAK 真进了评分分支（不是「不知道」）：有限分数、有声帧 ≥ 20。
fn assert_weak_scored(scores: &[vdn_eval::Score]) {
    let w = scores.iter().find(|s| s.id == "V-WEAK").expect("有 V-WEAK");
    let frames = w.extra.iter().find(|(k, _)| *k == "frames").map(|e| e.1);
    assert!(
        w.value.is_some_and(f64::is_finite) && frames.is_some_and(|f| f >= 20.0),
        "V-WEAK 没进评分分支：{:?} · {frames:?} · {}",
        w.value,
        w.note
    );
}

/// 六组：(名字, 内容 ID 行)。第 6 组「verifiers_sans_timbre」= 第 4 组去掉 V-TIMBRE（调研 026，#119 起）：
/// 改 V-TIMBRE 量法、更新基准时，第 4 组会变、第 6 组必须不变——证明其余 verifier 逐位不变。
fn lines() -> Vec<String> {
    let clips = [clip(7, 5.0), clip(11, 15.0)];
    let (mut sg, mut es, mut rw, mut vf, mut eg, mut vx) = (
        Acc::default(),
        Acc::default(),
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
        // 2. ESTOI：16 kHz（先重采样）与 10 kHz（不重采样）两条路，都必须给出有限分数
        for (a, b) in [(&c.clean, &c.mix), (&c.clean, &r.out), (&c.clean, &c.clean)] {
            for sr in [MODEL_RATE, estoi::FS as u32] {
                let v = estoi(a, b, sr);
                assert!(v.is_some_and(f64::is_finite), "ESTOI 没出分：sr={sr}");
                es.f64(v.unwrap_or(f64::NAN));
            }
        }
        // 4. 全部 verifier；6. 同上去掉 V-TIMBRE
        let all = score(&mut vf, c, None);
        all.iter()
            .filter(|s| s.id != "V-TIMBRE")
            .for_each(|s| record(&mut vx, s));
        assert_weak_scored(&all);
    }
    // 5. 边界：别的采样率（16384 → 窗长 2048 恰为 2 的幂；22050 → 2756 补零到 4096）· 短于窗长 · 帧端点 · 空输入
    let speech: &[&str] = &["V-WEAK", "V-ESTOI"];
    for sr in [16_384, 22_050] {
        let c = Clip {
            sr,
            ..clips[0].clone()
        };
        assert_weak_scored(&score(&mut eg, &c, Some(speech)));
    }
    // 截取从主讲人第一次开口处开始（不是开头的静音），帧端点与短于窗长的情形才真走到分帧
    let s0 = clips[0].truth.owner.first().expect("有主讲人").0;
    for n in [0, 1999, 2000, 2001, 3000, 4000] {
        score(&mut eg, &excerpt(&clips[0], s0, n), Some(speech));
    }
    for n in [0, 100, 255, 256, 257, 383, 384, 511, 512, 513, 3000, 6000] {
        let c = excerpt(&clips[0], s0, n);
        for (n_fft, hop) in [(512, 256), (256, 128)] {
            let s = Spectrogram::new(&c.mix, n_fft, hop);
            eg.text(&s.frames.len().to_string());
            s.frames.iter().for_each(|f| eg.f32s(f));
        }
        for sr in [MODEL_RATE, estoi::FS as u32] {
            eg.f64(estoi(&c.clean, &c.mix, sr).unwrap_or(f64::NAN));
        }
    }
    // ESTOI 去静音后刚够 / 刚不够 30 帧附近（10 kHz 下 30 帧 ≈ 3968 个样本），必须有出分的
    let mut scored = 0;
    for n in [5000, 6000, 6500, 7000, 8000] {
        let c = excerpt(&clips[0], s0, n);
        let v = estoi(&c.clean, &c.mix, estoi::FS as u32);
        scored += usize::from(v.is_some());
        eg.f64(v.unwrap_or(f64::NAN));
    }
    assert!(scored > 0, "ESTOI 短片段一个都没出分，边界没测到");
    vec![
        sg.line("spectrogram"),
        es.line("estoi"),
        rw.line("ref_wiener"),
        vf.line("verifiers"),
        eg.line("edges"),
        vx.line("verifiers_sans_timbre"),
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

/// 冻结清单里的项都还在登记表里，每个旁报字段都还会在基准片段上出现（不许悄悄删掉）。
#[test]
fn frozen_items_and_fields_all_present() {
    for (id, keys) in FROZEN {
        assert!(
            REGISTRY.iter().any(|i| i.id == *id),
            "{id} 从登记表里删掉了"
        );
        if keys.is_empty() {
            continue;
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in [clip(7, 5.0), clip(11, 15.0)] {
            let mut acc = Acc::default();
            for s in score(&mut acc, &c, Some(&[id])) {
                seen.extend(s.extra.iter().map(|(k, _)| *k));
            }
        }
        for k in *keys {
            assert!(seen.contains(k), "{id} 的旁报 {k} 不见了");
        }
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
