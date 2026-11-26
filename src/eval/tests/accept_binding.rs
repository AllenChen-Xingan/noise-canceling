//! `vdn-eval accept` 的输入绑定与覆盖测试（codex 第 3–4 轮）：
//! 卡片 ↔ 报告种子（系统 + 逐段文件 sha256 + 清单指纹）、整场景漏跑、逐卡延迟、结论里那句 0.20。

#[path = "common/accept_fixture.rs"]
mod fx;

use serde_json::Value;
use vdn_eval::scene::MeetingPlace;

use fx::{
    Fix, Spec, build, cand, cand_cards, card_json, card_json_with, cards_for, ctl, good1, good2,
    good3, spec,
};

fn verdict(js: &Value, what_contains: &str) -> String {
    js["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["what"].as_str().unwrap().contains(what_contains))
        .unwrap_or_else(|| panic!("没有一行包含「{what_contains}」：{js}"))["verdict"]
        .as_str()
        .unwrap()
        .to_string()
}

fn detail(js: &Value, what_contains: &str) -> String {
    js["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["what"].as_str().unwrap().contains(what_contains))
        .unwrap_or_else(|| panic!("没有一行包含「{what_contains}」：{js}"))["detail"]
        .as_str()
        .unwrap()
        .to_string()
}

fn cli(f: &Fix, args: &[String]) -> Result<std::path::PathBuf, String> {
    vdn_eval::scorecard::accept::cli(args, "t", f.root.join("default"))
}

/// codex 第 3 轮 P1 #1：身份是「成绩单 sha256 + 系统名」——同一份文件里的不同系统是两个候选。
#[test]
fn same_file_different_system_is_a_distinct_candidate() {
    let f = Fix::new("multisys");
    let g = build(&f, spec(good1(), good2(), good3()));
    let mut a = ctl(MeetingPlace::Cafe, 4);
    a.extend(ctl(MeetingPlace::MeetingRoom, 4));
    a.extend(ctl(MeetingPlace::LibraryClassroom, 4));
    let mut multi = Vec::new();
    for (system, d) in [("cand1", good1()), ("cand2", good2()), ("cand3", good3())] {
        multi.extend(cand(&a, d, &[], system));
    }
    let ctl_p = f.jsonl("ctl", &a);
    let multi_p = f.jsonl("multi", &multi);
    let vs: Vec<String> = ["cand1", "cand2", "cand3"]
        .iter()
        .map(|c| f.compare_sys(&format!("vs-{c}"), &ctl_p, &multi_p, "ref", c))
        .collect();
    let noise: Vec<String> = [("cand1", "cand2"), ("cand1", "cand3"), ("cand2", "cand3")]
        .iter()
        .map(|(x, y)| f.compare_sys(&format!("n-{x}-{y}"), &multi_p, &multi_p, x, y))
        .collect();
    // 报告种子是 cand1：候选卡必须是 cand1 的系统、它的逐段文件
    let (ca, cb) = cards_for("ref", "cand1", "pass", "pass");
    let a_p = f.card("ms-a", &card_json_with(&ca, &g.fp), &ctl_p);
    let b_p = f.card("ms-b", &card_json_with(&cb, &g.fp), &multi_p);
    let js = f.accept(&vs, &noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "pass", "{js}");
}

/// codex 第 1 轮：一份成绩单里有两个系统时，卡会互相覆盖 → 报错。
#[test]
fn card_must_be_the_reported_seed() {
    let f = Fix::new("binding");
    let g = build(&f, spec(good1(), good2(), good3()));
    // 候选卡旁边放的是对照的逐段文件（系统 ref）：第一份 --vs 的候选是 cand，读不出这个系统 → 拒绝
    let (a, b) = cards_for("ref", "cand", "pass", "pass");
    let a_p = f.card("bnd-a", &card_json_with(&a, &g.fp), &g.ctl);
    let b_p = f.card("bnd-b", &card_json_with(&b, &g.fp), &g.ctl);
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &a_p, &b_p, "x")).unwrap_err();
    assert!(e.contains("没有系统") && e.contains("cand"), "{e}");
    // 对照卡旁边放的是候选的逐段文件（系统 cand）：第一份 --vs 的对照是 ref → 拒绝
    let (a2, b2) = cards_for("ref", "cand", "pass", "pass");
    let a2 = f.card("bnd-a2", &card_json_with(&a2, &g.fp), &g.cand1);
    let b2 = f.card("bnd-b2", &card_json_with(&b2, &g.fp), &g.cand1);
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &a2, &b2, "y")).unwrap_err();
    assert!(e.contains("没有系统") && e.contains("ref"), "{e}");
}

/// codex 第 4 轮 B：成绩单旁边必须有逐段文件，而且它的 sha256 就是 --vs 记的那一边。
#[test]
fn card_clips_jsonl_must_be_the_compared_file() {
    let f = Fix::new("bindsha");
    let g = build(&f, spec(good1(), good2(), good3()));
    // 缺 clips.jsonl → 拒绝
    let (ca, _) = cards_for("ref", "cand", "pass", "pass");
    let no_clips = f.card("noclips-a", &card_json_with(&ca, &g.fp), "");
    let e = cli(
        &f,
        &f.accept_args(&g.vs, &g.noise, &no_clips, &g.card_b, "x"),
    )
    .unwrap_err();
    assert!(e.contains("clips.jsonl"), "{e}");
    // 拿另一份逐段文件冒充（同一个系统、内容改了一处）→ sha256 对不上，拒绝
    let (ca2, _) = cards_for("ref", "cand", "pass", "pass");
    let mut alt = ctl(MeetingPlace::Cafe, 4);
    alt.extend(ctl(MeetingPlace::MeetingRoom, 4));
    alt.extend(ctl(MeetingPlace::LibraryClassroom, 4));
    alt[0].scores[0].2.value = Some(0.9);
    let other = f.jsonl("ctl-alt", &alt);
    let wrong = f.card("wrongclips-a", &card_json_with(&ca2, &g.fp), &other);
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &wrong, &g.card_b, "y")).unwrap_err();
    assert!(e.contains("sha256") || e.contains("逐段文件"), "{e}");
    // 卡自己记的 from_clips_sha256 也要对得上
    let (ca3, _) = cards_for("ref", "cand", "pass", "pass");
    let mut v: Value = serde_json::from_str(&card_json_with(&ca3, &g.fp)).unwrap();
    v["meta"]["from_clips_sha256"] = serde_json::json!("de".repeat(32));
    let bad_sha = f.card("badsha-a", &v.to_string(), &g.ctl);
    let e = cli(
        &f,
        &f.accept_args(&g.vs, &g.noise, &bad_sha, &g.card_b, "z"),
    )
    .unwrap_err();
    assert!(e.contains("sha256") || e.contains("逐段文件"), "{e}");
}

/// codex 第 4 轮 B：成绩单记的清单指纹必须等于 --suite 的指纹。
#[test]
fn card_suite_fingerprint_must_match_the_suite() {
    let f = Fix::new("fp");
    let g = build(&f, spec(good1(), good2(), good3()));
    let (ca, cb) = cards_for("ref", "cand", "pass", "pass");
    let a_p = f.card("fp-a", &card_json_with(&ca, "deadbeefdeadbeef"), &g.ctl);
    let b_p = f.card("fp-b", &card_json_with(&cb, &g.fp), &g.cand1);
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &a_p, &b_p, "x")).unwrap_err();
    assert!(e.contains("指纹") || e.contains("清单"), "{e}");
    assert!(e.contains("deadbeefdeadbeef"), "{e}");
}

/// codex 第 3 轮 P1 #2：对照成绩单有、比较里整场景没有（两份逐段文件都漏了这个场景）→ 判不了。
#[test]
fn missing_whole_scene_is_undecided() {
    let f = Fix::new("omit");
    let g = build(
        &f,
        Spec {
            omit: &[MeetingPlace::LibraryClassroom],
            ..spec(good1(), good2(), good3())
        },
    );
    // 候选成绩单也缺这个场景（两份都漏跑）
    let a = card_json(
        &[
            ("cafe", "common", "mba-builtin", "ref", "pass"),
            ("meeting-room", "common", "mba-builtin", "ref", "pass"),
            ("library-classroom", "common", "mba-builtin", "ref", "pass"),
        ],
        serde_json::json!([]),
        &g.fp,
    );
    let b = card_json(
        &[
            ("cafe", "common", "mba-builtin", "cand", "pass"),
            ("meeting-room", "common", "mba-builtin", "cand", "pass"),
        ],
        serde_json::json!([]),
        &g.fp,
    );
    let a_p = f.card("omit-a", &a, &g.ctl);
    let b_p = f.card("omit-b", &b, &g.cand1);
    let js = f.accept(&g.vs, &g.noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "覆盖"), "undecided", "{js}");
    assert!(detail(&js, "覆盖").contains("library-classroom"), "{js}");
    assert_eq!(verdict(&js, "V-ONSET"), "undecided", "{js}");
    assert_eq!(js["verdict"], "undecided");
}

/// codex 第 4 轮 A：预期场景取自 --suite（独立于成绩单）。清单里有、对照成绩单里没有 → 判不了。
#[test]
fn per_card_latency_mismatch_fails() {
    let f = Fix::new("lat");
    let g = build(
        &f,
        Spec {
            lat1: &[(MeetingPlace::MeetingRoom, 1.0)],
            ..spec(good1(), good2(), good3())
        },
    );
    let (ca, cb) = cards_for("ref", "cand", "pass", "pass");
    let a_p = f.card("lat-a", &card_json_with(&ca, &g.fp), &g.ctl);
    let b_p = f.card("lat-b", &card_json_with(&cb, &g.fp), &g.cand1);
    let js = f.accept(&g.vs, &g.noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "延迟"), "fail", "{js}");
    assert!(detail(&js, "延迟").contains("meeting-room"), "{js}");
    assert_eq!(js["verdict"], "fail");
}

/// codex 第 4 轮 C：预期卡有一张在候选成绩单里没有读数 → 延迟判不了（不是拿别的卡顶上）。
#[test]
fn per_card_latency_missing_is_undecided() {
    let f = Fix::new("latmiss");
    let g = build(
        &f,
        Spec {
            omit1: &[MeetingPlace::MeetingRoom],
            ..spec(good1(), good2(), good3())
        },
    );
    let a_p = g.card_a.clone();
    let b_p = g.card_b.clone();
    let js = f.accept(&g.vs, &g.noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "延迟"), "undecided", "{js}");
    assert!(detail(&js, "延迟").contains("meeting-room"), "{js}");
}

/// codex 第 5 轮 P1 #1：覆盖按**片段 id** 核。候选漏掉某一切片里的一段 → 判不了。
#[test]
fn clip_missing_on_candidate_side_is_undecided() {
    let f = Fix::new("onclip");
    let gone = ctl(MeetingPlace::MeetingRoom, 4)[2].clip.clone();
    let g = build(
        &f,
        Spec {
            omit_clip1: &[gone.as_str()],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "覆盖"), "undecided", "{js}");
    let d = detail(&js, "覆盖");
    assert!(d.contains(&gone), "要说清是哪一段：{d}");
    assert_eq!(js["verdict"], "undecided");
}

/// codex 第 9 轮 P1：段都在、但某个目标场景片段的 V-NRMOS-SCOREQ 在候选那边是 null
/// → 这一项配对段数少于清单的段数 → 要涨的线判不了（不能拿剩下的段判过）。
#[test]
fn null_value_shrinks_gain_pairs_is_undecided() {
    let f = Fix::new("nullscoreq");
    let clip = ctl(MeetingPlace::Cafe, 4)[3].clip.clone();
    let g = build(
        &f,
        Spec {
            null1: &[(&clip, "V-NRMOS-SCOREQ")],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(
        verdict(&js, "V-NRMOS-SCOREQ（场景 cafe"),
        "undecided",
        "{js}"
    );
    let d = detail(&js, "V-NRMOS-SCOREQ（场景 cafe");
    assert!(d.contains("7") && d.contains("8"), "要写出段数：{d}");
    // 别的项、别的场景不受影响
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "pass", "{js}");
    assert_eq!(js["verdict"], "undecided");
}

/// codex 第 11 轮 P1：延迟逐卡比还不够——清单要的每一段的 V-LAT 都得有值。
/// 候选有一段 V-LAT 是 null、两张卡的均值又恰好相同 → 仍判不了。
#[test]
fn null_lat_value_on_one_clip_is_undecided() {
    let f = Fix::new("nulllat");
    let clip = ctl(MeetingPlace::Cafe, 4)[1].clip.clone();
    let g = build(
        &f,
        Spec {
            null1: &[(&clip, "V-LAT")],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "延迟"), "undecided", "{js}");
    assert!(
        detail(&js, "延迟").contains(&clip),
        "要说清是哪一段没有值：{js}"
    );
    assert_eq!(js["verdict"], "undecided");
}

/// codex 第 10 轮 P1：「不许退」的项也要核齐。段都在、但候选有一段的 V-TIMBRE 是 null → 判不了。
#[test]
fn null_value_shrinks_regress_pairs_is_undecided() {
    let f = Fix::new("nulltimbre");
    let clip = ctl(MeetingPlace::Cafe, 4)[2].clip.clone();
    let g = build(
        &f,
        Spec {
            null1: &[(&clip, "V-TIMBRE")],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-TIMBRE"), "undecided", "{js}");
    let d = detail(&js, "V-TIMBRE");
    assert!(d.contains("7") && d.contains("8"), "要写出段数：{d}");
    assert_eq!(verdict(&js, "V-ONSET"), "pass", "{js}");
    assert_eq!(js["verdict"], "undecided");
}

/// codex 第 7 轮 P1：覆盖要核每一份 compare，不止报告种子那份。
/// 种子 2 的 compare（以及它参与的两对互比）少了一段目标场景的片段 → 要涨的线判不了。
#[test]
fn other_seed_clip_missing_is_undecided() {
    let f = Fix::new("seed2clip");
    let gone = ctl(MeetingPlace::Cafe, 4)[1].clip.clone();
    let g = build(
        &f,
        Spec {
            omit_clip2: &[gone.as_str()],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "undecided", "{js}");
    assert!(
        detail(&js, "V-ESTOI（场景 cafe").contains(&gone),
        "要说清哪一份 compare 少了哪一段：{js}"
    );
    assert_eq!(verdict(&js, "覆盖"), "undecided", "{js}");
    assert_eq!(js["verdict"], "undecided");
}

/// codex 第 5 轮 P1 #2：卡片状态从绑定的逐段文件重新汇总。旧 scorecard.json 写成「过」也不能改结论。
#[test]
fn old_scorecard_status_is_ignored() {
    let f = Fix::new("oldjson");
    // 候选那边 library-classroom 有一段否决项不过 → 重新汇总出来这张卡是「不过」
    let g = build(
        &f,
        Spec {
            poison1: &[(MeetingPlace::LibraryClassroom, "V-NAN")],
            ..spec(good1(), good2(), good3())
        },
    );
    // scorecard.json 里把它写成「过」（旧的 / 手工改过的），旁边是新的逐段文件
    let (ca, cb) = cards_for("ref", "cand", "pass", "pass");
    let js = f.accept(
        &g.vs,
        &g.noise,
        &f.card("oldjson-a", &card_json_with(&ca, &g.fp), &g.ctl),
        &f.card("oldjson-b", &card_json_with(&cb, &g.fp), &g.cand1),
        "x",
    );
    assert_eq!(verdict(&js, "其他场景"), "fail", "{js}");
    assert!(
        detail(&js, "其他场景").contains("library-classroom"),
        "{js}"
    );
}

/// codex 第 4 轮 D：结论里固定写「评分模型之一只抓得到 0.20 的坏段」（014 第三节第 5 条 · 调研 031）。
#[test]
fn ear_sentence_is_in_both_outputs() {
    let f = Fix::new("ear");
    let g = build(&f, spec(good1(), good2(), good3()));
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert!(
        js["notes"].to_string().contains("0.20"),
        "accept.json 没写那句话：{js}"
    );
    let md = std::fs::read_to_string(f.root.join("acc-x").join("accept.md")).unwrap();
    assert!(md.contains("0.20"), "accept.md 没写那句话：{md}");
    assert!(
        md.contains("人耳未测") && md.contains("只在合成集上判"),
        "{md}"
    );
}

/// 夹具自己的卡片（`Good::card_a/card_b`）是合法输入：不该被绑定检查拒掉。
#[test]
fn fixture_cards_pass_binding() {
    let f = Fix::new("okbind");
    let g = build(&f, spec(good1(), good2(), good3()));
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(js["verdict"], "pass", "{js}");
    let _ = cand_cards("pass", "pass");
}
