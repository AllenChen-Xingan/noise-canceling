//! 配对比较的单元测试。
use super::*;
use crate::scene::{MeetingPlace, SoundGroup, Tier};
use crate::verifiers::info;

fn sc(id: &'static str, v: f64) -> Score {
    let b = info(id).map_or(Better::Lower, |i| i.better);
    Score::judged(id, v, "x", b, Some(0.0), &[])
}

fn row(scene: MeetingPlace, i: usize, scores: Vec<(&'static str, f64)>) -> ClipRow {
    ClipRow {
        clip: format!(
            "{}-c{:02}-{}",
            scene.as_str(),
            i / 2,
            if i.is_multiple_of(2) { "ext" } else { "mba" }
        ),
        scene,
        tier: Tier::Common,
        device: "mba-builtin".into(),
        system: "gtcrn".into(),
        sounds: vec![("bed-x".into(), SoundGroup::Bed)],
        scores: scores
            .into_iter()
            .map(|(v, x)| (v, standard::VETO.iter().any(|i| i.v == v), sc(v, x)))
            .collect(),
    }
}

/// 12 段（6 个源片段）：ESTOI 有起伏，SPLVL 在 0 附近。
fn base(scene: MeetingPlace) -> Vec<ClipRow> {
    (0..12)
        .map(|i| {
            let w = (i as f64 * 0.7).sin();
            row(scene, i, vec![("V-ESTOI", 0.6 + 0.05 * w), ("V-SPLVL", -0.5 + 0.3 * w)])
        })
        .collect()
}

fn item<'a>(c: &'a Compare, v: &str) -> &'a ItemDiff {
    c.items.iter().find(|i| i.v == v).unwrap()
}

#[test]
fn self_compare_is_all_unsure_and_zero() {
    let a = base(MeetingPlace::Cafe);
    let r = compare(&a, &a, None, false).unwrap();
    assert_eq!(r.len(), 1);
    let c = &r[0];
    assert_eq!(c.pairs, 12);
    for i in &c.items {
        assert_eq!(i.call, Call::Unsure, "{}", i.v);
        let d = i.diff.as_ref().unwrap();
        assert_eq!((d.mean, d.lo, d.hi), (0.0, 0.0, 0.0), "{}", i.v);
    }
    assert!(c.vetoes.iter().all(|v| v.new_in_b.is_empty() && v.gone_in_b.is_empty()));
}

#[test]
fn level_down_6db_is_a_regression_with_new_vetoes() {
    let a = base(MeetingPlace::Cafe);
    let mut b = a.clone();
    for r in &mut b {
        for (v, _, s) in &mut r.scores {
            if *v == "V-SPLVL" {
                s.value = s.value.map(|x| x - 6.0);
            }
        }
    }
    let c = &compare(&a, &b, None, false).unwrap()[0];
    let s = item(c, "V-SPLVL");
    assert_eq!(s.call, Call::Down);
    assert!((s.diff.as_ref().unwrap().mean + 6.0).abs() < 1e-9);
    // 新规则线 −4 dB：a 全在线内，b 全越线 → 12 段新增否决
    let v = c.vetoes.iter().find(|v| v.v == "V-SPLVL").unwrap();
    assert_eq!((v.a, v.b, v.new_in_b.len()), (0, 12, 12));
    assert_eq!(v.rule, VETO_LINE);
    // 没动的项仍不确定；反过来比是涨、否决段消失
    assert_eq!(item(c, "V-ESTOI").call, Call::Unsure);
    let back = &compare(&b, &a, None, false).unwrap()[0];
    assert_eq!(item(back, "V-SPLVL").call, Call::Up);
    assert_eq!(back.vetoes.iter().find(|v| v.v == "V-SPLVL").unwrap().gone_in_b.len(), 12);
}

#[test]
fn lower_is_better_direction() {
    // V-NR 越小越好：b 小 1 = 涨
    let a: Vec<ClipRow> = (0..8).map(|i| row(MeetingPlace::Cafe, i, vec![("V-NR", -8.0 + 0.1 * i as f64)])).collect();
    let mut b = a.clone();
    b.iter_mut().for_each(|r| r.scores[0].2.value = r.scores[0].2.value.map(|x| x - 1.0));
    assert_eq!(item(&compare(&a, &b, None, false).unwrap()[0], "V-NR").call, Call::Up);
}

#[test]
fn missing_and_one_sided_clips_reported_not_paired() {
    let a = base(MeetingPlace::Cafe);
    let mut b = a.clone();
    let gone = b.remove(0).clip;
    b.push(row(MeetingPlace::Cafe, 40, vec![("V-ESTOI", 0.6)]));
    b[0].scores[0].2.value = None; // ESTOI 只有 a 有值
    let c = &compare(&a, &b, None, false).unwrap()[0];
    assert_eq!(c.pairs, 11);
    assert_eq!(c.only_a, vec![gone]);
    assert_eq!(c.only_b, vec!["cafe-c20-ext".to_string()]);
    let e = item(c, "V-ESTOI");
    assert_eq!((e.pairs, e.one_sided), (10, 1));
    // 非有限值同样不算
    let mut b2 = a.clone();
    b2[1].scores[0].2.value = Some(f64::NAN);
    assert_eq!(item(&compare(&a, &b2, None, false).unwrap()[0], "V-ESTOI").one_sided, 1);
}

#[test]
fn too_few_groups_is_unsure() {
    // 只有 1 个源片段（两段同源）：组数 < SC-AGG.min_groups → 不确定，哪怕差得很大
    let a: Vec<ClipRow> = (0..2).map(|i| row(MeetingPlace::Cafe, i, vec![("V-ESTOI", 0.5)])).collect();
    let mut b = a.clone();
    b.iter_mut().for_each(|r| r.scores[0].2.value = Some(0.9));
    let e = item(&compare(&a, &b, None, false).unwrap()[0], "V-ESTOI").clone();
    assert_eq!(e.diff.unwrap().groups, 1);
    assert_eq!(e.call, Call::Unsure);
}

#[test]
fn bad_inputs_are_errors() {
    let a = base(MeetingPlace::Cafe);
    // 同一 id 两次
    let mut dup = a.clone();
    dup.push(a[0].clone());
    assert!(compare(&dup, &a, None, false).is_err());
    // 场景对不上（不同 build）
    let mut other = a.clone();
    other[0].scene = MeetingPlace::Canteen;
    assert!(compare(&a, &other, None, false).is_err());
    // 没有共同段
    let far = base(MeetingPlace::Canteen);
    assert!(compare(&a, &far, None, false).is_err());
    // 多系统不选
    let mut two = a.clone();
    two[0].system = "ref".into();
    assert!(select(two.clone(), None, "a").is_err());
    assert_eq!(select(two.clone(), Some("ref"), "a").unwrap().len(), 1);
    assert!(select(two, Some("nope"), "a").is_err());
}

#[test]
fn noise_floor_is_attached_and_does_not_change_call() {
    let a = base(MeetingPlace::Cafe);
    let mut b = a.clone();
    b.iter_mut().for_each(|r| r.scores[1].2.value = r.scores[1].2.value.map(|x| x - 6.0));
    let mut n2 = a.clone();
    n2.iter_mut().enumerate().for_each(|(i, r)| r.scores[0].2.value = r.scores[0].2.value.map(|x| x + 0.01 * (i as f64).cos()));
    let with = &compare(&a, &b, Some((&a, &n2)), false).unwrap()[0];
    let without = &compare(&a, &b, None, false).unwrap()[0];
    for (x, y) in with.items.iter().zip(&without.items) {
        assert_eq!(x.call, y.call);
        assert_eq!(x.diff, y.diff);
        assert!(x.noise.is_some() && y.noise.is_none());
    }
}

#[test]
fn by_scene_adds_one_scope_per_scene() {
    let mut a = base(MeetingPlace::Cafe);
    a.extend(base(MeetingPlace::Canteen));
    let r = compare(&a, &a, None, true).unwrap();
    let scopes: Vec<&str> = r.iter().map(|c| c.scope.as_str()).collect();
    assert_eq!(scopes, vec!["all", "cafe", "canteen"]);
    assert_eq!(r[0].pairs, 24);
    assert_eq!(r[1].pairs, 12);
}

#[test]
fn same_input_same_output() {
    let a = base(MeetingPlace::Cafe);
    let mut b = a.clone();
    b.iter_mut().enumerate().for_each(|(i, r)| r.scores[0].2.value = r.scores[0].2.value.map(|x| x + 0.01 * (i as f64)));
    assert_eq!(compare(&a, &b, None, true).unwrap(), compare(&a, &b, None, true).unwrap());
}

#[test]
fn veto_rules_per_item() {
    // 物理项看逐段判定；吃字看多错字数；人声被压低看新规则线
    let nan = Score::judged("V-NAN", 1.0, "个", Better::Lower, Some(0.0), &[]);
    assert_eq!(vetoed("V-NAN", &nan), Some(true));
    let eat = Score::judged("V-EAT-ASR", 25.0, "pp", Better::Lower, Some(1.0), &[]).with_extra("ref_len", 20.0);
    assert_eq!(vetoed("V-EAT-ASR", &eat), Some(true)); // 25% × 20 字 = 5 字 ≥ 5
    let eat4 = Score::judged("V-EAT-ASR", 20.0, "pp", Better::Lower, Some(1.0), &[]).with_extra("ref_len", 20.0);
    assert_eq!(vetoed("V-EAT-ASR", &eat4), Some(false));
    let lv = |x| Score::judged("V-SPLVL", x, "dB", Better::Higher, Some(-3.0), &[]);
    assert_eq!(vetoed("V-SPLVL", &lv(-3.5)), Some(false)); // v2 线是 −4，不是分数里记的 −3
    assert_eq!(vetoed("V-SPLVL", &lv(-4.5)), Some(true));
    for i in standard::VETO {
        let _ = veto_rule(i.v);
    }
}

/// 真实打分链路：同一批合成段，b 的输出整体压低 6 dB（×0.5）→ 人声被压低（V-SPLVL）判退，
/// 差值约 −6 dB；没动输出的一边自己比自己全部不确定。
#[test]
fn real_scoring_output_6db_down_is_down() {
    use crate::runner::{Baseline, Opts, prepare_clip, score_one};
    use crate::suite::{Grader, LEVELS, parse_levels};
    use crate::synth::{NoiseKind, SceneSpec, TalkerSpec, render};
    use crate::system::SystemKind;
    use crate::verifiers::Params;
    let t = crate::suite::parse_suite(include_str!("../../suite/ci.json")).unwrap().tasks[0].clone();
    let o = Opts { ci: true, audio_dir: None, baseline: Baseline::new(), levels: parse_levels(LEVELS).unwrap() };
    let g = Grader { v: "V-SPLVL".into(), params: Params::default(), info_only: false };
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for i in 0..8 {
        let c = render(&SceneSpec {
            dur_s: 6.0,
            seed: 300 + i as u64,
            owner: Some(TalkerSpec { level_dbov: -26.0, f0_lo: 100.0, f0_hi: 140.0, tonal: false }),
            owner_start_s: 0.5,
            pause_s: (0.4, 1.0),
            interferers: Vec::new(),
            noise: NoiseKind::Pink,
            snr_db: 20.0,
            tail_silence_s: 0.0,
        });
        let p = prepare_clip(&t, SystemKind::Passthrough, c, false, false).unwrap();
        let mut none: Option<&mut dyn crate::runner::Extra> = None;
        let sa = score_one(&g, SystemKind::Passthrough, &t, &p, &p.run.out, &o, &mut none);
        let low: Vec<f32> = p.run.out.iter().map(|x| x * 0.5).collect();
        let sb = score_one(&g, SystemKind::Passthrough, &t, &p, &low, &o, &mut none);
        let mut ra = row(MeetingPlace::Cafe, i, vec![]);
        ra.scores = vec![("V-SPLVL", true, sa)];
        let mut rb = ra.clone();
        rb.scores = vec![("V-SPLVL", true, sb)];
        a.push(ra);
        b.push(rb);
    }
    let c = &compare(&a, &b, None, false).unwrap()[0];
    let s = item(c, "V-SPLVL");
    assert_eq!(s.pairs, 8);
    assert_eq!(s.call, Call::Down);
    let m = s.diff.as_ref().unwrap().mean;
    assert!((m + 6.02).abs() < 0.3, "差值 {m}");
    assert!(compare(&a, &a, None, false).unwrap()[0].items.iter().all(|i| i.call == Call::Unsure));
}
