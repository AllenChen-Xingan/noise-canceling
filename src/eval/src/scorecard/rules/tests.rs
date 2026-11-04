//! 汇总规则的单元测试（从 rules.rs 拆出，守 600 行上限）。
use super::*;
use crate::{Better, Score};

fn s(id: &'static str, v: f64, thr: f64, better: Better) -> Score {
    Score::judged(id, v, "x", better, Some(thr), &[])
}

fn row(clip: &str, sounds: &[&str], weak: f64, dq: f64, dd: f64) -> ClipRow {
    ClipRow {
        clip: clip.into(),
        scene: MeetingPlace::Cafe,
        tier: Tier::Common,
        device: "mba-builtin".into(),
        system: "ref".into(),
        sounds: sounds
            .iter()
            .map(|c| (c.to_string(), SoundGroup::Pattern))
            .collect(),
        scores: vec![
            ("V-NAN", true, s("V-NAN", 0.0, 0.0, Better::Lower)),
            ("V-WEAK", false, s("V-WEAK", weak, 4.0, Better::Lower)),
            (
                "V-NRMOS-SCOREQ",
                false,
                s("V-NRMOS-SCOREQ", dq, 0.0, Better::Higher),
            ),
            (
                "V-NRMOS-DISTILL",
                false,
                s("V-NRMOS-DISTILL", dd, 0.0, Better::Higher),
            ),
        ],
    }
}

#[test]
fn listen_rules() {
    assert_eq!(
        listen_one(&row("a", &[], 0.0, 0.2, 0.1).scores),
        (CardStatus::Pass, None)
    );
    assert_eq!(
        listen_one(&row("a", &[], 0.0, -0.2, -0.1).scores).0,
        CardStatus::Fail
    );
    let (st, why) = listen_one(&row("a", &[], 0.0, 0.2, -0.1).scores);
    assert_eq!(st, CardStatus::Unknown);
    assert!(why.unwrap().contains("一个说好一个说差"));
    // 都过但相差 ≥ 0.3：仍过，送人耳
    let (st, why) = listen_one(&row("a", &[], 0.0, 0.5, 0.1).scores);
    assert_eq!(st, CardStatus::Pass);
    assert!(why.is_some());
}

#[test]
fn worst_clip_decides_and_listen_is_not_an_item() {
    let rows = vec![
        row("a", &[], 1.0, 0.1, 0.1),
        row("b", &[], 5.0, 0.1, 0.1),
        row("c", &[], 1.0, 0.1, 0.1),
    ];
    let c = &summarize(&rows, &[])[0];
    let weak = c.items.iter().find(|i| i.v == "V-WEAK").unwrap();
    assert_eq!(
        (weak.pass, weak.fail, weak.status),
        (2, 1, CardStatus::Fail)
    );
    assert_eq!(weak.worst_clip.as_deref(), Some("b"));
    assert!(c.items.iter().all(|i| !LISTEN.contains(&i.v)));
    assert_eq!(c.status, CardStatus::Fail);
    assert!(c.veto_failed.is_empty());
}

#[test]
fn report_only_item_does_not_decide_scene() {
    let mut r = row("a", &[], 1.0, 0.1, 0.1);
    r.scores
        .push(("V-CLICK", false, s("V-CLICK", 30.0, 0.0, Better::Lower)));
    let c = &summarize(&[r], &[])[0];
    let click = c.items.iter().find(|i| i.v == "V-CLICK").unwrap();
    assert!(click.report_only);
    assert_eq!(click.status, CardStatus::Fail, "照常判、照常列");
    assert_eq!(c.status, CardStatus::Pass, "但不进场景的过不过");
    assert!(c.drags.iter().all(|d| d.v != "V-CLICK"));
}

#[test]
fn missing_clips_block_pass_and_empty_combo_still_listed() {
    let rows = vec![row("a", &[], 1.0, 0.1, 0.1)];
    let miss = |scene: MeetingPlace, what: &str| Missing {
        scene,
        tier: Tier::Common,
        device: "mba-builtin".into(),
        system: "ref".into(),
        what: what.into(),
        why: "sha256 对不上".into(),
    };
    // 一段过、一段没读成 → 待测，不是过
    let c = summarize(&rows, &[miss(MeetingPlace::Cafe, "b")]);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].status, CardStatus::Unknown);
    assert_eq!(c[0].missing.len(), 1);
    // 整个场景一段都没读成 → 仍出一行、待测
    let c = summarize(&rows, &[miss(MeetingPlace::Canteen, "S-canteen")]);
    let canteen = c.iter().find(|x| x.scene == MeetingPlace::Canteen).unwrap();
    assert_eq!((canteen.clips, canteen.status), (0, CardStatus::Unknown));
    assert!(
        worst(&c)
            .iter()
            .any(|&i| c[i].scene == MeetingPlace::Canteen)
    );
}

#[test]
fn listen_with_unknown_clip_is_not_pass() {
    // 一段两个模型都说好，另一段一个说好一个说差 → 听感待测，场景不能判过
    let rows = vec![row("a", &[], 1.0, 0.2, 0.1), row("b", &[], 1.0, 0.2, -0.1)];
    let c = &summarize(&rows, &[])[0];
    assert_eq!(c.listen.status, Some(CardStatus::Unknown));
    assert_eq!(c.status, CardStatus::Unknown);
    assert_eq!(c.listen.to_human.len(), 1);
}

#[test]
fn listen_failure_gets_sound_split() {
    let mut rows: Vec<ClipRow> = (0..3)
        .map(|i| row(&format!("m{i}"), &["coffee-machine"], 1.0, -0.5, -0.4))
        .collect();
    rows.extend((0..3).map(|i| row(&format!("q{i}"), &[], 1.0, 0.1, 0.1)));
    let c = &summarize(&rows, &[])[0];
    assert_eq!(c.listen.status, Some(CardStatus::Fail));
    assert_eq!(c.listen.models.len(), 2);
    let d = c
        .drags
        .iter()
        .find(|d| d.v == "V-NRMOS-SCOREQ" && d.cls == "coffee-machine")
        .expect("听感不过时要按声音分组");
    assert!((d.gap.unwrap() - 0.6).abs() < 1e-9);
}

#[test]
fn listen_means_are_reported() {
    let rows = vec![row("a", &[], 1.0, 0.2, 0.1), row("b", &[], 1.0, 0.4, 0.3)];
    let c = &summarize(&rows, &[])[0];
    assert!((c.listen.mean_scoreq.unwrap() - 0.3).abs() < 1e-9);
    assert!((c.listen.mean_distill.unwrap() - 0.2).abs() < 1e-9);
}

#[test]
fn veto_failure_is_listed_and_fails_scene() {
    let mut r = row("a", &[], 1.0, 0.1, 0.1);
    r.scores[0].2 = s("V-NAN", 3.0, 0.0, Better::Lower);
    let c = &summarize(&[r], &[])[0];
    assert_eq!(c.veto_failed, vec!["V-NAN"]);
    assert_eq!(c.status, CardStatus::Fail);
}

#[test]
fn drag_finds_the_sound_and_needs_enough_clips() {
    // 有咖啡机的 3 段更差（6 dB），没有的 3 段 1 dB
    let mut rows: Vec<ClipRow> = (0..3)
        .map(|i| {
            row(
                &format!("m{i}"),
                &["coffee-machine", "bed-cafe"],
                6.0,
                0.1,
                0.1,
            )
        })
        .collect();
    rows.extend((0..3).map(|i| row(&format!("q{i}"), &["bed-cafe"], 1.0, 0.1, 0.1)));
    let c = &summarize(&rows, &[])[0];
    let top = &c.drags[0];
    assert_eq!(top.cls, "coffee-machine");
    assert!((top.gap.unwrap() - 5.0).abs() < 1e-9);
    // 每段都有的声音分不出组 → 样本不够
    let bed = c.drags.iter().find(|d| d.cls == "bed-cafe").unwrap();
    assert!(bed.gap.is_none());
    // 有这类声音反而更好的（gap < 0）不列
    assert!(c.drags.iter().all(|d| d.gap.is_none_or(|g| g > 0.0)));
    // 只有 2 段有：样本不够，不下结论
    let c2 = &summarize(&rows[1..], &[])[0];
    assert!(
        c2.drags
            .iter()
            .all(|d| d.cls != "coffee-machine" || d.gap.is_none())
    );
}

#[test]
fn worst_orders_fail_before_unknown_and_skips_pass() {
    let mk = |scene: MeetingPlace, weak: f64, dq: f64| {
        let mut r = row("a", &[], weak, dq, dq);
        r.scene = scene;
        r
    };
    let rows = vec![
        mk(MeetingPlace::Cafe, 1.0, 0.1),           // 过
        mk(MeetingPlace::Canteen, 6.0, 0.1),        // 不过
        mk(MeetingPlace::HomeQuiet, 1.0, f64::NAN), // 听感判不了 → 待测
    ];
    let cards = summarize(&rows, &[]);
    let w = worst(&cards);
    assert_eq!(w.len(), 2);
    assert_eq!(cards[w[0]].scene, MeetingPlace::Canteen);
    assert_eq!(cards[w[1]].scene, MeetingPlace::HomeQuiet);
}

#[test]
fn listen_failure_counts_in_ranking_and_summary() {
    // 两个场景否决项、普通项都一样；只有一个听感不过 → 它排在前面，摘要里写「听感」
    let mut a = row("a", &[], 5.0, 0.1, 0.1);
    a.scene = MeetingPlace::Cafe;
    let mut b = row("b", &[], 5.0, -0.2, -0.3);
    b.scene = MeetingPlace::Canteen;
    let cards = summarize(&[a, b], &[]);
    let w = worst(&cards);
    assert_eq!(cards[w[0]].scene, MeetingPlace::Canteen);
    assert!(cards[w[0]].failed_items().contains(&"听感"));
    // 只有听感不过的场景：摘要不能写「没有不过的项」
    let only = summarize(&[row("c", &[], 1.0, -0.2, -0.3)], &[]);
    let card = crate::scorecard::card::Card {
        worst: worst(&only),
        cards: only,
        missing: Vec::new(),
        clips: 1,
    };
    let md = card.to_markdown("t");
    assert!(md.contains("不过的项 听感"), "{md}");
    assert!(
        md.contains("| V-NRMOS-SCOREQ | 0 / 1 / 0"),
        "Markdown 要有模型明细：{md}"
    );
}
