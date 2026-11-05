//! 新规则（v2）的单元测试。
use super::*;
use crate::scene::{MeetingPlace, SoundGroup, Tier};
use crate::verifiers::info;

fn sc(id: &'static str, v: f64, thr: f64) -> Score {
    let b = info(id).map_or(Better::Lower, |i| i.better);
    Score::judged(id, v, "x", b, Some(thr), &[])
}

/// 一段：听感两项恒为 +0.1（过），再加 `extra` 里的项。
fn row(i: usize, scene: MeetingPlace, extra: Vec<(&'static str, bool, Score)>) -> ClipRow {
    let mut scores = vec![
        ("V-NRMOS-SCOREQ", false, sc("V-NRMOS-SCOREQ", 0.1, 0.0)),
        ("V-NRMOS-DISTILL", false, sc("V-NRMOS-DISTILL", 0.1, 0.0)),
    ];
    scores.splice(0..0, extra);
    ClipRow {
        clip: format!("{}-c{:02}-{}", scene.as_str(), i / 2, if i % 2 == 0 { "ext" } else { "mba" }),
        scene,
        tier: Tier::Common,
        device: "mba-builtin".into(),
        system: "gtcrn".into(),
        sounds: vec![("bed-x".into(), SoundGroup::Bed)],
        scores,
    }
}

fn item<'a>(c: &'a SceneCard, v: &str) -> &'a ItemCell {
    c.items.iter().find(|i| i.v == v).unwrap()
}

fn one(rows: &[ClipRow]) -> SceneCard {
    let mut v = summarize(rows, &[]);
    assert_eq!(v.len(), 1);
    v.remove(0)
}

#[test]
fn physical_item_still_vetoes_on_one_clip() {
    let rows: Vec<ClipRow> = (0..10)
        .map(|i| row(i, MeetingPlace::Cafe, vec![("V-NAN", true, sc("V-NAN", if i == 3 { 1.0 } else { 0.0 }, 0.0))]))
        .collect();
    let c = one(&rows);
    assert_eq!(item(&c, "V-NAN").status, CardStatus::Fail);
    assert_eq!(item(&c, "V-NAN").rule, RULE_PHYS);
    assert_eq!(c.veto_failed, vec!["V-NAN"]);
    assert_eq!(c.status, CardStatus::Fail);
}

#[test]
fn one_bad_clip_no_longer_fails_a_mean_item() {
    // V-ONSET 线 3：9 段 0.5、1 段 5 → 旧规则不过；均值 0.95，区间整个在线内 → 过
    let rows: Vec<ClipRow> = (0..10)
        .map(|i| row(i, MeetingPlace::Cafe, vec![("V-ONSET", false, sc("V-ONSET", if i == 0 { 5.0 } else { 0.5 }, 3.0))]))
        .collect();
    assert_eq!(item(&rules::summarize(&rows, &[])[0], "V-ONSET").status, CardStatus::Fail);
    let c = one(&rows);
    let it = item(&c, "V-ONSET");
    assert_eq!((it.status, it.rule), (CardStatus::Pass, RULE_MEAN));
    assert_eq!(it.fail, 1, "不过段数照报");
    assert!(it.ci.as_ref().unwrap().hi <= 3.0);
    assert_eq!(c.status, CardStatus::Pass);
}

#[test]
fn clearly_bad_mean_fails_and_straddling_is_unknown() {
    let mk = |base: f64| -> Vec<ClipRow> {
        (0..10)
            .map(|i| row(i, MeetingPlace::Cafe, vec![("V-ONSET", false, sc("V-ONSET", base + 0.5 * (i % 3) as f64, 3.0))]))
            .collect()
    };
    assert_eq!(item(&one(&mk(6.0)), "V-ONSET").status, CardStatus::Fail);
    assert_eq!(item(&one(&mk(2.5)), "V-ONSET").status, CardStatus::Unknown);
}

#[test]
fn eat_vetoes_only_at_five_chars() {
    // 参考 27 字：4 字 = 14.8 百分点 → 过；5 字 = 18.5 百分点 → 这段不过 → 否决
    let eat = |chars: f64| {
        let mut s = sc("V-EAT-ASR", chars / 27.0 * 100.0, 1.0);
        s.extra.push(("ref_len", 27.0));
        ("V-EAT-ASR", true, s)
    };
    let rows: Vec<ClipRow> = (0..6).map(|i| row(i, MeetingPlace::Cafe, vec![eat(4.0)])).collect();
    let c = one(&rows);
    assert_eq!((item(&c, "V-EAT-ASR").status, item(&c, "V-EAT-ASR").rule), (CardStatus::Pass, RULE_EAT));
    let mut rows = rows;
    rows[2].scores[0] = eat(5.0);
    assert_eq!(item(&one(&rows), "V-EAT-ASR").status, CardStatus::Fail);
    assert_eq!(one(&rows).veto_failed, vec!["V-EAT-ASR"]);
}

#[test]
fn report_only_and_old_input_do_not_decide_scene() {
    // 门控项不过、轻声字没在干净人声上算：都只报不判，场景照样过
    let rows: Vec<ClipRow> = (0..10)
        .map(|i| {
            row(i, MeetingPlace::Cafe, vec![
                ("V-SIM-GATE", false, sc("V-SIM-GATE", 0.9, 0.05)),
                ("V-WEAK", false, sc("V-WEAK", 9.0, 4.0)),
            ])
        })
        .collect();
    let c = one(&rows);
    assert!(item(&c, "V-SIM-GATE").report_only);
    assert_eq!(item(&c, "V-WEAK").rule, RULE_OLD_INPUT);
    assert_eq!(c.status, CardStatus::Pass);
    // 带「干净人声单独过一遍」旁证的轻声字照常判
    let rows: Vec<ClipRow> = (0..10)
        .map(|i| {
            let mut s = sc("V-WEAK", 9.0, 4.0);
            s.extra.push((standard::CLEAN_PASS_MARK, 1.0));
            row(i, MeetingPlace::Cafe, vec![("V-WEAK", false, s)])
        })
        .collect();
    let c = one(&rows);
    assert_eq!((item(&c, "V-WEAK").rule, item(&c, "V-WEAK").status), (RULE_MEAN, CardStatus::Fail));
}

#[test]
fn splvl_uses_four_db_line() {
    // 均值约 −3.5 dB：旧线 −3 不过，新线 −4 过
    let rows: Vec<ClipRow> = (0..10)
        .map(|i| row(i, MeetingPlace::Cafe, vec![("V-SPLVL", true, sc("V-SPLVL", -3.5 + 0.02 * i as f64, -3.0))]))
        .collect();
    let it = item(&one(&rows), "V-SPLVL").clone();
    assert_eq!((it.threshold, it.status), (Some(-4.0), CardStatus::Pass));
}

#[test]
fn listen_uses_delta_and_sends_worst_to_human() {
    let mk = |d: f64| -> Vec<ClipRow> {
        (0..10)
            .map(|i| {
                let mut r = row(i, MeetingPlace::Cafe, vec![]);
                let x = d + 0.01 * i as f64;
                r.scores[0].2 = sc("V-NRMOS-SCOREQ", x, 0.0);
                r.scores[1].2 = sc("V-NRMOS-DISTILL", x, 0.0);
                r
            })
            .collect()
    };
    // Δ 约 −0.05：旧规则每段都「比原声差」→ 不过；新规则 δ = 0.1 → 过
    let c = one(&mk(-0.08));
    assert_eq!(c.listen.status, Some(CardStatus::Pass));
    assert_eq!(c.listen.models[0].threshold, Some(-0.1));
    assert!(c.listen.to_human.len() >= LISTEN_WORST);
    assert!(c.listen.to_human.iter().any(|(x, _)| x == "cafe-c00-ext"));
    assert_eq!(one(&mk(-0.6)).listen.status, Some(CardStatus::Fail));
}

#[test]
fn too_few_valued_clips_is_unknown() {
    let rows: Vec<ClipRow> = (0..10)
        .map(|i| {
            let s = if i < 6 {
                Score::unknown("V-ONSET", "dB", Better::Lower, "x")
            } else {
                sc("V-ONSET", 0.1, 3.0)
            };
            row(i, MeetingPlace::Cafe, vec![("V-ONSET", false, s)])
        })
        .collect();
    assert_eq!(item(&one(&rows), "V-ONSET").status, CardStatus::Unknown);
}

#[test]
fn holm_downgrades_weak_fails_in_a_big_family() {
    // 一个明显不过、一个刚好不过：单独看都不过；同族 20 个组合时，刚好不过的被 Holm 改判待测
    let mut cells: Vec<ItemCell> = Vec::new();
    let base = rules::summarize(
        &[row(0, MeetingPlace::Cafe, vec![("V-ONSET", false, sc("V-ONSET", 0.1, 3.0))])],
        &[],
    )[0]
    .items[0]
    .clone();
    for k in 0..20 {
        let mut c = base.clone();
        c.rule = RULE_MEAN;
        c.status = if k < 2 { CardStatus::Fail } else { CardStatus::Pass };
        c.ci = Some(Ci {
            mean: 0.0,
            lo: 0.0,
            hi: 0.0,
            groups: 5,
            p_fail: match k {
                0 => 0.0005,
                1 => 0.02,
                _ => 0.9,
            },
        });
        cells.push(c);
    }
    let mut refs: Vec<&mut ItemCell> = cells.iter_mut().collect();
    apply_holm(&mut refs);
    assert_eq!(cells[0].status, CardStatus::Fail);
    assert_eq!(cells[1].status, CardStatus::Unknown);
}
