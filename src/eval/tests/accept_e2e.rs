//! `vdn-eval accept` 的端到端测试：先用合成的逐段分数跑真的 `vdn-eval compare --by-scene`，
//! 再把那些 compare.json 与两份 scorecard.json 交给 accept，核每行的三态结论。
//!
//! 候选的每一项都是对照加一个常数偏移，配对差因此在 8 段上完全一样、bootstrap 区间退化成一个点：
//! 判断只由差值符号与线决定，测试不受抽样影响。

use std::path::PathBuf;

use serde_json::{Value, json};
use vdn_eval::Score;
use vdn_eval::scene::{MeetingPlace, Tier};
use vdn_eval::scorecard::ClipRow;
use vdn_eval::thresholds as th;
use vdn_eval::verifiers::info;

/// 候选相对对照的偏移（逐项）。
#[derive(Clone, Copy, Default)]
struct D {
    estoi: f64,
    scoreq: f64,
    distill: f64,
    onset: f64,
    bgstab: f64,
    timbre: f64,
    /// 吃字：百分点（参考字数两边都是 20）
    eat_pp: f64,
    /// 人声被压低（dB）
    splvl: f64,
    lat: f64,
}

/// 种子 1（报告种子）相对对照：每项都更好一点。
fn good1() -> D {
    D {
        estoi: 0.052,
        scoreq: 0.082,
        distill: 0.082,
        onset: -0.15,
        bgstab: -0.15,
        timbre: -0.05,
        ..D::default()
    }
}

/// 种子 2：与种子 1 差一点抖动（种子噪声底的来源）。
fn good2() -> D {
    D {
        estoi: 0.05,
        scoreq: 0.08,
        distill: 0.08,
        onset: -0.2,
        bgstab: -0.2,
        timbre: -0.1,
        ..D::default()
    }
}

fn sc(v: &'static str, x: f64) -> (&'static str, bool, Score) {
    let b = info(v).map_or(vdn_eval::Better::Lower, |i| i.better);
    let thr = match v {
        "V-LAT" => Some(vdn_eval::system::BUDGET_MS),
        "V-SPLVL" => Some(th::SPLVL.def("min")),
        _ => Some(0.0),
    };
    (v, false, Score::judged(v, x, "x", b, thr, &[]))
}

/// 对照的一个场景：8 段 = 4 个源片段 × 2 个设备。目标场景 cafe 没有背景跳变这一项。
fn ctl(scene: MeetingPlace) -> Vec<ClipRow> {
    (0..8usize)
        .map(|i| {
            let w = 0.01 * i as f64;
            let mut scores = vec![
                sc("V-ESTOI", 0.6 + w),
                sc("V-NRMOS-SCOREQ", 3.4 + w),
                sc("V-NRMOS-DISTILL", 3.2 + w),
                sc("V-ONSET", 0.5 + w),
                sc("V-TIMBRE", 1.2 + w),
                sc("V-LAT", 43.666_666),
                sc("V-SPLVL", -1.0),
                (
                    "V-EAT-ASR",
                    false,
                    Score::judged(
                        "V-EAT-ASR",
                        20.0,
                        "pp",
                        vdn_eval::Better::Lower,
                        Some(1.0),
                        &[],
                    )
                    .with_extra("ref_len", 20.0),
                ),
            ];
            if scene != MeetingPlace::Cafe {
                scores.push(sc("V-BGSTAB", 4.0 + w));
            }
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
                system: "ref".into(),
                sounds: Vec::new(),
                scores,
                mix_sha256: None,
            }
        })
        .collect()
}

/// 候选：在对照的每个分数上加偏移；`drop` 里的项整段不出现（模拟「一边判不了」）。
fn cand(rows: &[ClipRow], d: D, drop: &[&str]) -> Vec<ClipRow> {
    rows.iter()
        .map(|r| {
            let mut n = r.clone();
            n.system = "cand".into();
            n.scores = r
                .scores
                .iter()
                .filter(|(v, _, _)| !drop.contains(v))
                .map(|(v, veto, s)| {
                    let dv = match *v {
                        "V-ESTOI" => d.estoi,
                        "V-NRMOS-SCOREQ" => d.scoreq,
                        "V-NRMOS-DISTILL" => d.distill,
                        "V-ONSET" => d.onset,
                        "V-BGSTAB" => d.bgstab,
                        "V-TIMBRE" => d.timbre,
                        "V-EAT-ASR" => d.eat_pp,
                        "V-SPLVL" => d.splvl,
                        "V-LAT" => d.lat,
                        _ => 0.0,
                    };
                    let mut x = s.clone();
                    if let Some(v0) = s.value {
                        let now = v0 + dv;
                        x.value = Some(now);
                        x.verdict = match s.threshold {
                            Some(t) if x.better.worse(now, t) => vdn_eval::Verdict::Fail,
                            Some(_) => vdn_eval::Verdict::Pass,
                            None => s.verdict.clone(),
                        };
                    }
                    (*v, *veto, x)
                })
                .collect();
            n
        })
        .collect()
}

/// 临时目录 + 造输入文件的小工具。
struct Fix {
    root: PathBuf,
}

impl Fix {
    fn new(tag: &str) -> Fix {
        let root = std::env::temp_dir().join(format!("vdn-accept-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Fix { root }
    }

    fn jsonl(&self, name: &str, rows: &[ClipRow]) -> String {
        let p = self.root.join(format!("{name}.jsonl"));
        std::fs::write(&p, vdn_eval::scorecard::load::to_jsonl(rows).unwrap()).unwrap();
        p.to_string_lossy().to_string()
    }

    /// 用真的 `compare --by-scene` 造一份 compare.json。
    fn compare(&self, tag: &str, a: &str, b: &str) -> String {
        let out = self.root.join(format!("cmp-{tag}"));
        let args: Vec<String> = ["--a", a, "--b", b, "--by-scene"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        vdn_eval::scorecard::compare_out::cli(&args, "t", out.clone()).unwrap();
        out.join("compare.json").to_string_lossy().to_string()
    }

    fn file(&self, name: &str, text: &str) -> String {
        let p = self.root.join(format!("{name}.json"));
        std::fs::write(&p, text).unwrap();
        p.to_string_lossy().to_string()
    }

    fn accept_args(&self, vs: &[&str], noise: &[&str], a: &str, b: &str, tag: &str) -> Vec<String> {
        self.accept_args_scenes(vs, noise, a, b, tag, "cafe,meeting-room")
    }

    fn accept_args_scenes(
        &self,
        vs: &[&str],
        noise: &[&str],
        a: &str,
        b: &str,
        tag: &str,
        scenes: &str,
    ) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        for v in vs {
            args.push("--vs".into());
            args.push((*v).into());
        }
        for n in noise {
            args.push("--noise".into());
            args.push((*n).into());
        }
        let out = self.root.join(format!("acc-{tag}"));
        for (k, v) in [
            ("--scenes", scenes.to_string()),
            ("--card-a", a.to_string()),
            ("--card-b", b.to_string()),
            ("--out", out.to_string_lossy().to_string()),
        ] {
            args.push(k.into());
            args.push(v);
        }
        args
    }

    fn accept(&self, vs: &[&str], noise: &[&str], a: &str, b: &str, tag: &str) -> Value {
        let args = self.accept_args(vs, noise, a, b, tag);
        let d = vdn_eval::scorecard::accept::cli(&args, "t", self.root.join("default"))
            .unwrap_or_else(|e| panic!("accept 出错：{e}"));
        serde_json::from_str(&std::fs::read_to_string(d.join("accept.json")).unwrap()).unwrap()
    }
}

/// 一份手写的成绩单（只带 accept 要读的字段；格式照 `scorecard::card::Card::to_json`）。
fn card_json(rows: &[(&str, &str, &str, &str, &str)], ext: Value) -> String {
    let scenes: Vec<Value> = rows
        .iter()
        .map(|(scene, tier, device, system, status)| {
            json!({
                "scene": scene, "tier": tier, "device": device, "system": system, "clips": 8,
                "missing": [], "status": status, "veto_failed": [],
                "items": [{"v": "V-LAT", "veto": true, "report_only": false, "unit": "ms",
                    "better": "lower", "pass": 8, "fail": 0, "unknown": 0, "mean": 43.666666,
                    "worst": 43.666666, "worst_clip": null, "threshold": 45.0,
                    "status": "pass", "ci": null, "rule": "逐段"}],
                "listen": {"models": [], "mean_scoreq": null, "mean_distill": null, "pass": 0,
                    "fail": 0, "unknown": 0, "to_human": [], "status": null},
                "drags": []
            })
        })
        .collect();
    json!({"doc": "测试用成绩单", "meta": {"run_id": "t", "systems": ["cand"], "external": ext},
        "rules": "v2", "clips_scored": 8 * scenes.len(), "scenes": scenes, "worst": [],
        "missing": [], "no_clean_pass": []})
    .to_string()
}

/// 三张卡：两张目标场景 + 一张非目标场景（library-classroom）。
fn cards(a_status: &str, b_status: &str) -> (String, String) {
    let a = [
        ("cafe", "common", "mba-builtin", "ref", a_status),
        ("meeting-room", "common", "mba-builtin", "ref", a_status),
        (
            "library-classroom",
            "common",
            "mba-builtin",
            "ref",
            a_status,
        ),
    ];
    let b = [
        ("cafe", "common", "mba-builtin", "cand", a_status),
        ("meeting-room", "common", "mba-builtin", "cand", a_status),
        (
            "library-classroom",
            "common",
            "mba-builtin",
            "cand",
            b_status,
        ),
    ];
    (card_json(&a, json!([])), card_json(&b, json!([])))
}

/// 候选三张卡：两张目标场景同状态，非目标场景（library-classroom）单独给状态。
fn cand_cards(target: &str, other: &str) -> String {
    let rows = [
        ("cafe", "common", "mba-builtin", "cand", target),
        ("meeting-room", "common", "mba-builtin", "cand", target),
        ("library-classroom", "common", "mba-builtin", "cand", other),
    ];
    card_json(&rows, json!([]))
}

/// 一份完整输入：两个目标场景 + 一个非目标场景，候选 = 对照 + `d1`（种子 1）/ `d2`（种子 2）。
struct Good {
    vs1: String,
    vs2: String,
    noise: String,
    card_a: String,
    card_b: String,
}

fn fixture(f: &Fix, d1: D, d2: D, drop1: &[&str]) -> Good {
    let mut a = ctl(MeetingPlace::Cafe);
    a.extend(ctl(MeetingPlace::MeetingRoom));
    a.extend(ctl(MeetingPlace::LibraryClassroom));
    let (b1, b2) = (cand(&a, d1, drop1), cand(&a, d2, &[]));
    let (ca, cb) = cards("pass", "pass");
    Good {
        vs1: f.compare("vs1", &f.jsonl("ctl", &a), &f.jsonl("c1", &b1)),
        vs2: f.compare("vs2", &f.jsonl("ctl", &a), &f.jsonl("c2", &b2)),
        noise: f.compare("noise", &f.jsonl("c1b", &b1), &f.jsonl("c2b", &b2)),
        card_a: f.file("a", &ca),
        card_b: f.file("b", &cb),
    }
}

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
    let g = fixture(&f, good1(), good2(), &[]);
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
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

#[test]
fn estoi_gain_below_line_fails() {
    let f = Fix::new("low");
    let g = fixture(
        &f,
        D {
            estoi: 0.005,
            ..good1()
        },
        D {
            estoi: 0.005,
            ..good2()
        },
        &[],
    );
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "fail", "{js}");
    assert_eq!(verdict(&js, "V-ESTOI（场景 meeting-room"), "fail", "{js}");
    assert_eq!(js["verdict"], "fail");
}

#[test]
fn gain_not_beyond_noise_floor_fails() {
    // 种子 1 只涨 0.05，种子 2 却比种子 1 高 0.08：涨不过种子噪声底
    let f = Fix::new("floor");
    let g = fixture(
        &f,
        D {
            estoi: 0.05,
            ..good1()
        },
        D {
            estoi: 0.13,
            ..good2()
        },
        &[],
    );
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "fail", "{js}");
    assert!(detail(&js, "V-ESTOI（场景 cafe").contains("噪声"), "{js}");
    assert_eq!(verdict(&js, "V-NRMOS-SCOREQ（场景 cafe"), "pass");
}

#[test]
fn one_seed_not_up_fails() {
    // 种子 2 可懂度原地不动：compare 判「不确定」，不是每个种子都涨
    let f = Fix::new("seed2");
    let g = fixture(
        &f,
        good1(),
        D {
            estoi: 0.0,
            ..good2()
        },
        &[],
    );
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "fail", "{js}");
}

#[test]
fn onset_upper_end_above_line_fails() {
    let f = Fix::new("onset");
    let g = fixture(
        &f,
        D {
            onset: 1.5,
            ..good1()
        },
        good2(),
        &[],
    );
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-ONSET"), "fail", "{js}");
    assert_eq!(verdict(&js, "V-TIMBRE"), "pass");
}

#[test]
fn one_new_eat_veto_clip_fails() {
    // 报告种子的每一段都比原声多错 5 字（25% × 20 字）→ 新增否决段
    let f = Fix::new("eat");
    let g = fixture(
        &f,
        D {
            eat_pp: 5.0,
            ..good1()
        },
        good2(),
        &[],
    );
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-EAT-ASR"), "fail", "{js}");
    assert_eq!(js["verdict"], "fail");
}

#[test]
fn only_undecided_vetoes_still_pass() {
    // 候选整份成绩单没有吃字与压人声两项：一边判不了，不算新增否决段，不挡过
    let f = Fix::new("und");
    let g = fixture(&f, good1(), good2(), &["V-EAT-ASR", "V-SPLVL"]);
    let js = f.accept(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "x");
    assert_eq!(verdict(&js, "V-EAT-ASR"), "pass", "{js}");
    assert!(detail(&js, "V-EAT-ASR").contains("一边判不了"), "{js}");
    assert_eq!(verdict(&js, "V-SPLVL"), "pass", "{js}");
    assert_eq!(js["verdict"], "pass", "{js}");
}

#[test]
fn non_target_card_to_fail_fails_and_target_card_change_does_not() {
    let f = Fix::new("card");
    let g = fixture(&f, good1(), good2(), &[]);
    let (ca, _) = cards("pass", "pass");
    let a = f.file("a", &ca);
    let js = f.accept(
        &[&g.vs1, &g.vs2],
        &[&g.noise],
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
        &[&g.vs1, &g.vs2],
        &[&g.noise],
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

#[test]
fn external_candidate_latency_undecided() {
    let f = Fix::new("ext");
    let g = fixture(&f, good1(), good2(), &[]);
    let (ca, _) = cards("pass", "pass");
    let ext = json!([{"name": "cand", "latency_samples": 768, "latency_declared_only": true}]);
    let rows = [
        ("cafe", "common", "mba-builtin", "cand", "pass"),
        ("meeting-room", "common", "mba-builtin", "cand", "pass"),
        ("library-classroom", "common", "mba-builtin", "cand", "pass"),
    ];
    let js = f.accept(
        &[&g.vs1, &g.vs2],
        &[&g.noise],
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
    let g = fixture(&f, good1(), good2(), &[]);
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
    let args = f.accept_args(&[&all], &[&g.noise], &g.card_a, &g.card_b, "nos");
    let e = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap_err();
    assert!(e.contains("by-scene") || e.contains("逐场景"), "{e}");
    // 正常跑一次；同一目录再跑：拒绝覆盖
    let args = f.accept_args(&[&g.vs1, &g.vs2], &[&g.noise], &g.card_a, &g.card_b, "ok");
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
    let args = f.accept_args(&[&g.vs1, &g.vs2], &[], &g.card_a, &g.card_b, "nonoise");
    let d = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap();
    let js: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join("accept.json")).unwrap()).unwrap();
    assert_eq!(verdict(&js, "V-ESTOI（场景 cafe"), "undecided", "{js}");
    assert_eq!(js["verdict"], "undecided");
}

/// 真的 `Card::to_json` 写的成绩单也能读（成绩单格式漂了这里先红）。
#[test]
fn real_scorecard_json_is_read() {
    use vdn_eval::scorecard::card::{Card, Rules};
    let f = Fix::new("realcard");
    let g = fixture(&f, good1(), good2(), &[]);
    let write = |name: &str| {
        let rows = ctl(MeetingPlace::Cafe);
        let card = Card::build_with(&rows, Vec::new(), Rules::V2);
        f.file(
            name,
            &card
                .to_json(json!({"run_id": "t", "external": []}))
                .to_string(),
        )
    };
    let (a, b) = (write("ra"), write("rb"));
    let args = f.accept_args_scenes(&[&g.vs1, &g.vs2], &[&g.noise], &a, &b, "real", "cafe");
    let d = vdn_eval::scorecard::accept::cli(&args, "t", f.root.join("default")).unwrap();
    let js: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join("accept.json")).unwrap()).unwrap();
    assert_eq!(verdict(&js, "延迟"), "pass", "{js}");
}
