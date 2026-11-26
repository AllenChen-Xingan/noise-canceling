//! `vdn-eval accept` 的端到端测试：用合成的逐段分数跑真的 `vdn-eval compare --by-scene`，
//! 再把那些 compare.json 与两份 scorecard.json 交给 accept，核每行的三态结论。
//!
//! 夹具在 `common/accept_fixture.rs`（默认三个种子、三对种子互比；配对差恒定，不受 bootstrap 抽样影响）。

#[path = "common/accept_fixture.rs"]
mod fx;

use serde_json::{Value, json};
use vdn_eval::scene::MeetingPlace;

use fx::{D, Fix, Spec, build, cand_cards, card_json, cards, good1, good2, good3, spec};

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

#[test]
fn all_good_is_pass() {
    let f = Fix::new("good");
    let g = build(&f, spec(good1(), good2(), good3()));
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(js["verdict"], "pass", "{js}");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "pass");
    assert_eq!(verdict(&js, "V-ESTOI（场景 meeting-room"), "pass");
    assert_eq!(verdict(&js, "V-NRMOS-SCOREQ（场景 cafe"), "pass");
    assert_eq!(verdict(&js, "V-NRMOS-DISTILL（场景 meeting-room"), "pass");
    assert_eq!(verdict(&js, "V-BGSTAB"), "pass", "cafe 没有这一项不算失败");
    assert_eq!(verdict(&js, "V-ONSET"), "pass");
    assert_eq!(verdict(&js, "V-TIMBRE"), "pass");
    assert_eq!(verdict(&js, "V-EAT-ASR"), "pass");
    assert_eq!(verdict(&js, "V-SPLVL"), "pass");
    assert_eq!(verdict(&js, "其他场景"), "pass");
    assert_eq!(verdict(&js, "延迟"), "pass");
    // 报告必须写明两句话（014 第三节第 5 · 6 条）
    let md = std::fs::read_to_string(f.root.join("acc-x").join("accept.md")).unwrap();
    assert!(md.contains("人耳未测"), "{md}");
    assert!(md.contains("只在合成集上判"), "{md}");
    assert!(md.contains("结论：过"), "{md}");
    assert!(detail(&js, "V-ESTOI（场景 cafe").contains("0.05"), "{js}");
}

/// codex 第 1 轮 P1 #1：种子不齐（少于三个种子、或三对互比不全）→ 要涨的线一律判不了，不许当「过」。
#[test]
fn missing_seeds_or_noise_pairs_is_undecided() {
    let f = Fix::new("seeds");
    let g = build(&f, spec(good1(), good2(), good3()));
    // 只有两个种子
    let js = f.accept(&g.vs[..2], &g.noise, &g.card_a, &g.card_b, "two");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "undecided", "{js}");
    assert!(detail(&js, "V-ESTOI（场景 cafe").contains("种子"), "{js}");
    assert_eq!(js["verdict"], "undecided");
    // 三个种子，但三对互比只给了一对
    let js = f.accept(&g.vs, &g.noise[..1], &g.card_a, &g.card_b, "onepair");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "undecided", "{js}");
    assert_eq!(js["verdict"], "undecided");
    // 三份 --vs 其实是同一个种子（同一份成绩单）
    let same = vec![g.vs[0].clone(), g.vs[0].clone(), g.vs[0].clone()];
    let js = f.accept(&same, &g.noise, &g.card_a, &g.card_b, "same");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "undecided", "{js}");
}

#[test]
fn estoi_gain_below_line_fails() {
    let f = Fix::new("low");
    let g = build(
        &f,
        spec(
            D {
                estoi: 0.005,
                ..good1()
            },
            D {
                estoi: 0.005,
                ..good2()
            },
            D {
                estoi: 0.005,
                ..good3()
            },
        ),
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "fail", "{js}");
    assert_eq!(verdict(&js, "V-ESTOI（场景 meeting-room"), "fail", "{js}");
    assert_eq!(js["verdict"], "fail");
}

#[test]
fn gain_not_beyond_noise_floor_fails() {
    // 种子 1 只涨 0.05，种子 2 · 3 却比种子 1 高 0.08：涨不过种子噪声底
    let f = Fix::new("floor");
    let g = build(
        &f,
        spec(
            D {
                estoi: 0.05,
                ..good1()
            },
            D {
                estoi: 0.13,
                ..good2()
            },
            D {
                estoi: 0.13,
                ..good3()
            },
        ),
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "fail", "{js}");
    assert!(detail(&js, "V-ESTOI（场景 cafe").contains("噪声"), "{js}");
    assert_eq!(verdict(&js, "V-NRMOS-SCOREQ（场景 cafe"), "pass");
}

#[test]
fn one_seed_not_up_fails() {
    // 种子 2 可懂度原地不动：compare 判「不确定」，不是每个种子都涨
    let f = Fix::new("seed2");
    let g = build(
        &f,
        spec(
            good1(),
            D {
                estoi: 0.0,
                ..good2()
            },
            good3(),
        ),
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "fail", "{js}");
}

#[test]
fn onset_upper_end_above_line_fails() {
    let f = Fix::new("onset");
    let g = build(
        &f,
        spec(
            D {
                onset: 1.5,
                ..good1()
            },
            good2(),
            good3(),
        ),
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ONSET"), "fail", "{js}");
    assert_eq!(verdict(&js, "V-TIMBRE"), "pass");
}

/// codex 第 1 轮 P1 #2：一个场景里「组数不足、区间算不出来」是证据不足，不是不适用；
/// 另一个场景有合格区间也不能把整项判过。
#[test]
fn regress_evidence_shortage_is_undecided() {
    let f = Fix::new("thin");
    let g = build(
        &f,
        Spec {
            thin: &[MeetingPlace::MeetingRoom],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ONSET"), "undecided", "{js}");
    assert_eq!(verdict(&js, "V-TIMBRE"), "undecided", "{js}");
    assert!(
        detail(&js, "V-ONSET").contains("meeting-room"),
        "要说清哪个场景缺区间：{js}"
    );
    // cafe 根本没有背景跳变这一项（不适用），不会被算成证据不足；缺证据的是 meeting-room
    let bg = detail(&js, "V-BGSTAB");
    assert!(bg.contains("meeting-room"), "{bg}");
    assert!(!bg.contains("cafe"), "cafe 不适用却被当成缺证据：{bg}");
}

/// 一项在某个场景整段不存在（不适用）时不算证据不足：ONSET 在 meeting-room 缺席，cafe 有数 → 过。
#[test]
fn regress_item_absent_in_a_scene_is_not_a_shortage() {
    let f = Fix::new("absent");
    let g = build(
        &f,
        Spec {
            absent: &[
                (MeetingPlace::Cafe, "V-BGSTAB"),
                (MeetingPlace::MeetingRoom, "V-ONSET"),
            ],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ONSET"), "pass", "{js}");
}

#[test]
fn one_new_eat_veto_clip_fails() {
    // 报告种子的每一段都比原声多错 5 字（25% × 20 字）→ 新增否决段
    let f = Fix::new("eat");
    let g = build(
        &f,
        spec(
            D {
                eat_pp: 5.0,
                ..good1()
            },
            good2(),
            good3(),
        ),
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-EAT-ASR"), "fail", "{js}");
    assert_eq!(js["verdict"], "fail");
}

#[test]
fn only_undecided_vetoes_still_pass() {
    // 候选整份成绩单没有吃字与压人声两项：一边判不了，不算新增否决段，不挡过
    let f = Fix::new("und");
    let g = build(
        &f,
        Spec {
            drop1: &["V-EAT-ASR", "V-SPLVL"],
            ..spec(good1(), good2(), good3())
        },
    );
    let js = f.accept(&g.vs, &g.noise, &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-EAT-ASR"), "pass", "{js}");
    assert!(detail(&js, "V-EAT-ASR").contains("一边判不了"), "{js}");
    assert_eq!(verdict(&js, "V-SPLVL"), "pass", "{js}");
    assert_eq!(js["verdict"], "pass", "{js}");
}

#[test]
fn non_target_card_to_fail_fails_and_target_card_change_does_not() {
    let f = Fix::new("card");
    let g = build(&f, spec(good1(), good2(), good3()));
    let (ca, _) = cards("pass", "pass");
    let a = f.file("a", &ca);
    let js = f.accept(
        &g.vs,
        &g.noise,
        &a,
        &f.file("b-t", &cand_cards("fail", "pass")),
        "t",
    );
    assert_eq!(
        verdict(&js, "其他场景"),
        "pass",
        "目标场景的卡变了不算这一行：{js}"
    );
    let js = f.accept(
        &g.vs,
        &g.noise,
        &a,
        &f.file("b-o", &cand_cards("pass", "fail")),
        "o",
    );
    assert_eq!(verdict(&js, "其他场景"), "fail", "{js}");
    assert!(
        detail(&js, "其他场景").contains("library-classroom"),
        "{js}"
    );
    assert_eq!(js["verdict"], "fail");
}

/// codex 第 1 轮 P1 #4：非目标卡在对照里本来就全是「不过」时，没有可新增的 → 这一行是过，不是判不了。
#[test]
fn all_non_target_cards_already_fail_is_pass() {
    let f = Fix::new("allfail");
    let g = build(&f, spec(good1(), good2(), good3()));
    let a = f.file("a", &cand_cards("fail", "fail"));
    let js = f.accept(
        &g.vs,
        &g.noise,
        &a,
        &f.file("b", &cand_cards("pass", "fail")),
        "x",
    );
    assert_eq!(verdict(&js, "其他场景"), "pass", "{js}");
    assert!(detail(&js, "其他场景").contains("不过"), "{js}");
    assert_eq!(js["verdict"], "pass", "{js}");
}

/// codex 第 1 轮 P1 #3：一份成绩单里有两个系统时，卡会互相覆盖、延迟也会混着求均值 → 报错。
#[test]
fn two_systems_in_one_scorecard_is_an_error() {
    let f = Fix::new("twosys");
    let g = build(&f, spec(good1(), good2(), good3()));
    let rows = [
        ("cafe", "common", "mba-builtin", "ref", "pass"),
        ("cafe", "common", "mba-builtin", "cand", "fail"),
    ];
    let bad = f.file("a-two", &card_json(&rows, json!([])));
    let args = f.accept_args(&g.vs, &g.noise, &bad, &g.card_b, "x");
    let e = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap_err();
    assert!(e.contains("系统"), "要说清是系统数不对：{e}");
    assert!(e.contains("ref") && e.contains("cand"), "{e}");
}

#[test]
fn external_candidate_latency_undecided() {
    let f = Fix::new("ext");
    let g = build(&f, spec(good1(), good2(), good3()));
    let (ca, _) = cards("pass", "pass");
    let ext = json!([{"name": "cand", "latency_samples": 768, "latency_declared_only": true}]);
    let rows = [
        ("cafe", "common", "mba-builtin", "cand", "pass"),
        ("meeting-room", "common", "mba-builtin", "cand", "pass"),
        ("library-classroom", "common", "mba-builtin", "cand", "pass"),
    ];
    let js = f.accept(
        &g.vs,
        &g.noise,
        &f.file("a", &ca),
        &f.file("b", &card_json(&rows, ext)),
        "x",
    );
    assert_eq!(verdict(&js, "延迟"), "undecided", "{js}");
    assert_eq!(js["verdict"], "undecided", "其他行都过，只有延迟判不了");
}

#[test]
fn errors_are_reported_not_silently_ignored() {
    let f = Fix::new("err");
    let g = build(&f, spec(good1(), good2(), good3()));
    // 没加 --by-scene 的 compare.json：报错，不硬判
    let a = f.root.join("ctl.jsonl").to_string_lossy().to_string();
    let b = f.root.join("c1.jsonl").to_string_lossy().to_string();
    let out = f.root.join("cmp-all");
    vdn_eval::scorecard::compare_out::cli(
        &["--a".to_string(), a, "--b".to_string(), b],
        "t",
        out.clone(),
    )
    .unwrap();
    let all = out.join("compare.json").to_string_lossy().to_string();
    let vs = vec![all];
    let args = f.accept_args(&vs, &g.noise, &g.card_a, &g.card_b, "nos");
    let e = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap_err();
    assert!(e.contains("by-scene") || e.contains("逐场景"), "{e}");
    // 正常跑一次；同一目录再跑：拒绝覆盖
    let args = f.accept_args(&g.vs, &g.noise, &g.card_a, &g.card_b, "ok");
    vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap();
    let e = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap_err();
    assert!(e.contains("不覆盖"), "{e}");
    // 缺 --vs
    let e = vdn_eval::scorecard::accept::cli(
        &["--scenes".to_string(), "cafe".to_string()],
        "t",
        f.root.join("default"),
    )
    .unwrap_err();
    assert!(e.contains("--vs"), "{e}");
    // 没有 --noise：要涨的线判不了（不是不过）
    let args = f.accept_args(&g.vs, &[], &g.card_a, &g.card_b, "nonoise");
    let d = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap();
    let js: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join("accept.json")).unwrap()).unwrap();
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "undecided", "{js}");
    assert_eq!(js["verdict"], "undecided");
}

/// 真的 `Card::to_json` 写的成绩单也能读（成绩单格式漂了这里先红）。
#[test]
fn real_scorecard_json_is_read() {
    use fx::ctl;
    use vdn_eval::scorecard::card::{Card, Rules};
    let f = Fix::new("realcard");
    let g = build(&f, spec(good1(), good2(), good3()));
    let write = |name: &str| {
        let rows = ctl(MeetingPlace::Cafe, 4);
        let card = Card::build_with(&rows, Vec::new(), Rules::V2);
        f.file(
            name,
            &card
                .to_json(json!({"run_id": "t", "external": []}))
                .to_string(),
        )
    };
    let (a, b) = (write("ra"), write("rb"));
    let args = f.accept_args_scenes(&g.vs, &g.noise, &a, &b, "real", "cafe");
    let d = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap();
    let js: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join("accept.json")).unwrap()).unwrap();
    assert_eq!(verdict(&js, "延迟"), "pass", "{js}");
}
