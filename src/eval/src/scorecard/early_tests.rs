//! 早期回声参考旁列的测试。
//!
//! 关键一条（唯一真相源 + 交叉校验）：[`standard::EARLY_REF`] 是「读干净参考」的项的手写清单，
//! 这里**按行为**核对它——把参考换成带 100 ms 回声的版本，读数变了的非物理 CI 项必须正好是这份清单。

use super::*;
use crate::runner::{Baseline, Opts, Prepared, prepare_clip, score_one};
use crate::scene::MeetingPlace;
use crate::suite::{Grader, LEVELS, Task, parse_levels};
use crate::synth::{Clip, InterfMode, InterfSpec, NoiseKind, SceneSpec, TalkerSpec, render};
use crate::system::SystemKind;
use crate::verifiers::Params;

fn task() -> Task {
    crate::suite::parse_suite(include_str!("../../suite/ci.json"))
        .unwrap()
        .tasks[0]
        .clone()
}

fn clip() -> Clip {
    render(&SceneSpec {
        dur_s: 10.0,
        seed: 11,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 140.0,
            tonal: false,
        }),
        owner_start_s: 0.5,
        pause_s: (0.5, 1.0),
        interferers: vec![InterfSpec {
            mode: InterfMode::Overlap,
            sir_db: 5.0,
            f0_lo: 180.0,
            f0_hi: 240.0,
        }],
        noise: NoiseKind::Pink,
        snr_db: 10.0,
        tail_silence_s: 0.0,
    })
}

/// 带 100 ms 晚期回声的「全混响」参考（早期参考 = 原 clean）。
fn with_echo(x: &[f32]) -> Vec<f32> {
    let d = (SR as usize) / 10;
    (0..x.len())
        .map(|i| x[i] + if i >= d { 0.5 * x[i - d] } else { 0.0 })
        .collect()
}

fn opts() -> Opts {
    Opts {
        ci: true,
        audio_dir: None,
        baseline: Baseline::new(),
        levels: parse_levels(LEVELS).unwrap(),
    }
}

fn grader(i: &standard::Item) -> Grader {
    Grader {
        v: i.v.to_string(),
        params: Params(
            i.params
                .iter()
                .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
                .collect(),
        ),
        info_only: false,
    }
}

/// 同 scorecard::run_with 的取法：标了 clean_input 的项在「干净人声单独过一遍」的输出上算。
fn score(i: &standard::Item, p: &Prepared, t: &Task) -> Option<f64> {
    let g = grader(i);
    let o = opts();
    let mut none: Option<&mut dyn crate::runner::Extra> = None;
    let s = match (i.clean_input, p.clean_view()) {
        (true, Some((cp, co))) => score_one(&g, SystemKind::Ref, t, &cp, &co, &o, &mut none),
        _ => score_one(&g, SystemKind::Ref, t, p, &p.run.out, &o, &mut none),
    };
    s.value
}

/// 每个开会场景都看的全部 CI 层、非物理项（去重）。
fn ci_items() -> Vec<standard::Item> {
    let mut out: Vec<standard::Item> = Vec::new();
    for s in MeetingPlace::ALL {
        for (i, _) in standard::items(s) {
            let ci = crate::verifiers::info(i.v).is_some_and(|x| x.layer == "ci");
            if ci && !standard::PHYSICAL.contains(&i.v) && !out.iter().any(|o| o.v == i.v) {
                out.push(i);
            }
        }
    }
    out
}

#[test]
fn early_ref_list_matches_behaviour() {
    let t = task();
    let mut c = clip();
    let dry = c.clean.clone();
    c.clean = with_echo(&dry);
    let p = prepare_clip(&t, SystemKind::Ref, c, true, true).unwrap();
    let pe = swap_ref(&p, &dry);
    let mut dep: Vec<&str> = Vec::new();
    for i in ci_items() {
        let (a, b) = (score(&i, &p, &t), score(&i, &pe, &t));
        let changed = match (a, b) {
            (Some(x), Some(y)) => (x - y).abs() > 1e-9 * x.abs().max(1.0),
            (None, None) => false,
            _ => true,
        };
        if changed {
            dep.push(i.v);
        }
    }
    dep.sort_unstable();
    let mut want: Vec<&str> = standard::EARLY_REF.to_vec();
    want.sort_unstable();
    assert_eq!(dep, want, "换参考后读数变了的项 ≠ standard::EARLY_REF");
}

#[test]
fn early_ref_items_are_registered_ci_and_not_physical() {
    for v in standard::EARLY_REF {
        let i = crate::verifiers::info(v).unwrap_or_else(|| panic!("{v} 没登记"));
        assert_eq!(i.layer, "ci", "{v}");
        assert!(!standard::PHYSICAL.contains(&v), "{v}");
        assert!(
            ci_items().iter().any(|x| x.v == v),
            "{v} 不在任何场景的标准里"
        );
    }
}

#[test]
fn swap_ref_changes_only_the_reference() {
    let t = task();
    let p = prepare_clip(&t, SystemKind::Passthrough, clip(), true, false).unwrap();
    let e = vec![0.25f32; p.clip.clean.len()];
    let q = swap_ref(&p, &e);
    assert_eq!(q.clip.clean, e);
    assert_eq!(q.clip.mix, p.clip.mix);
    assert_eq!(q.run.out, p.run.out);
    assert_eq!(q.masks.owner, p.masks.owner);
}

#[test]
fn attach_and_read_back() {
    let s = Score::unknown("V-ESTOI", "", crate::Better::Higher, "x");
    let mut e = s.clone();
    e.value = Some(0.5);
    assert_eq!(value_of(&attach(s.clone(), &e)), Some(0.5));
    // 算不出 → NaN → 读回 None
    assert_eq!(value_of(&attach(s.clone(), &s)), None);
    assert_eq!(value_of(&s), None);
}
