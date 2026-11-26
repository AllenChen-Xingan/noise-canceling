//! `vdn-eval accept` 的输入绑定与覆盖测试（codex 第 3–4 轮）：
//! 卡片 ↔ 报告种子（系统 + 逐段文件 sha256 + 清单指纹）、整场景漏跑、逐卡延迟、结论里那句 0.20。

#[path = "common/accept_fixture.rs"]
mod fx;

use serde_json::Value;
use vdn_eval::scene::MeetingPlace;

use fx::{
    Fix, Spec, build, cand, cand_cards, card_json, card_json_with, cards_for, ctl, good1, good2,
    good3, set_lat, spec,
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
fn two_systems_in_one_scorecard_is_an_error() {
    let f = Fix::new("twosys");
    let g = build(&f, spec(good1(), good2(), good3()));
    let rows = [
        ("cafe", "common", "mba-builtin", "ref", "pass"),
        ("cafe", "common", "mba-builtin", "cand", "fail"),
    ];
    let bad = f.card(
        "twosys-a",
        &card_json(&rows, serde_json::json!([]), &g.fp),
        &g.ctl,
    );
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &bad, &g.card_b, "x")).unwrap_err();
    assert!(e.contains("系统"), "要说清是系统数不对：{e}");
    assert!(e.contains("ref") && e.contains("cand"), "{e}");
}

/// codex 第 3 轮 P1 #1：卡片的系统必须等于第一份 --vs 那一边的系统。
#[test]
fn card_must_be_the_reported_seed() {
    let f = Fix::new("binding");
    let g = build(&f, spec(good1(), good2(), good3()));
    let (a, b) = cards_for("ref", "cand9", "pass", "pass");
    let a_p = f.card("bnd-a", &card_json_with(&a, &g.fp), &g.ctl);
    let b_p = f.card("bnd-b", &card_json_with(&b, &g.fp), &g.cand1);
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &a_p, &b_p, "x")).unwrap_err();
    assert!(e.contains("cand9") && e.contains("系统"), "{e}");
    let (a2, b2) = cards_for("ref9", "cand", "pass", "pass");
    let a2 = f.card("bnd-a2", &card_json_with(&a2, &g.fp), &g.ctl);
    let b2 = f.card("bnd-b2", &card_json_with(&b2, &g.fp), &g.cand1);
    let e = cli(&f, &f.accept_args(&g.vs, &g.noise, &a2, &b2, "y")).unwrap_err();
    assert!(e.contains("ref9"), "{e}");
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
    // 拿另一份逐段文件冒充（种子 2 的）→ 拒绝
    let (ca2, _) = cards_for("ref", "cand", "pass", "pass");
    let other = f.root.join("c2.jsonl").to_string_lossy().to_string();
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
fn suite_scene_missing_from_cards_is_undecided() {
    let f = Fix::new("suiteonly");
    let g = build(&f, spec(good1(), good2(), good3()));
    // 两份成绩单都没有 library-classroom 这张卡，但清单里有这个场景
    let a = card_json(
        &[
            ("cafe", "common", "mba-builtin", "ref", "pass"),
            ("meeting-room", "common", "mba-builtin", "ref", "pass"),
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
    let a_p = f.card("so-a", &a, &g.ctl);
    let b_p = f.card("so-b", &b, &g.cand1);
    let js = f.accept(&g.vs, &g.noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "覆盖"), "undecided", "{js}");
    assert!(detail(&js, "覆盖").contains("library-classroom"), "{js}");
}

/// codex 第 4 轮 C：延迟按卡比（场景 × 档 × 设备），不取均值；一张卡差过容差就是不过。
#[test]
fn per_card_latency_mismatch_fails() {
    let f = Fix::new("lat");
    let g = build(&f, spec(good1(), good2(), good3()));
    let (ca, cb) = cards_for("ref", "cand", "pass", "pass");
    let a_p = f.card("lat-a", &card_json_with(&ca, &g.fp), &g.ctl);
    let b = set_lat(&card_json_with(&cb, &g.fp), "meeting-room", 44.7);
    let b_p = f.card("lat-b", &b, &g.cand1);
    let js = f.accept(&g.vs, &g.noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "延迟"), "fail", "{js}");
    assert!(detail(&js, "延迟").contains("meeting-room"), "{js}");
    assert_eq!(js["verdict"], "fail");
}

/// codex 第 4 轮 C：预期卡有一张在候选成绩单里没有读数 → 延迟判不了（不是拿别的卡顶上）。
#[test]
fn per_card_latency_missing_is_undecided() {
    let f = Fix::new("latmiss");
    let g = build(&f, spec(good1(), good2(), good3()));
    let (ca, _) = cards_for("ref", "cand", "pass", "pass");
    let b = card_json(
        &[
            ("cafe", "common", "mba-builtin", "cand", "pass"),
            ("library-classroom", "common", "mba-builtin", "cand", "pass"),
        ],
        serde_json::json!([]),
        &g.fp,
    );
    let a_p = f.card("latmiss-a", &card_json_with(&ca, &g.fp), &g.ctl);
    let b_p = f.card("latmiss-b", &b, &g.cand1);
    let js = f.accept(&g.vs, &g.noise, &a_p, &b_p, "x");
    assert_eq!(verdict(&js, "延迟"), "undecided", "{js}");
    assert!(detail(&js, "延迟").contains("meeting-room"), "{js}");
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
