//! `vdn-eval accept` 端到端测试共用的夹具：合成逐段分数 → 真的 `vdn-eval compare --by-scene` 的
//! compare.json → 评测清单（`--suite`）与两份带 `clips.jsonl` 的成绩单。被 `accept_e2e.rs` 用 `#[path]` 引入。
//!
//! 候选的每一项都是对照加一个常数偏移，配对差因此在每段上都一样、bootstrap 区间退化成一个点：
//! 判断只由差值符号与线决定，测试不受抽样影响。

#![allow(dead_code)]

#[path = "accept_manifest.rs"]
mod manifest;

use std::cell::RefCell;
use std::path::PathBuf;

use serde_json::{Value, json};
use vdn_eval::Score;
use vdn_eval::scene::{MeetingPlace, Tier};
use vdn_eval::scorecard::ClipRow;
use vdn_eval::thresholds as th;
use vdn_eval::verifiers::info;

/// 候选相对对照的偏移（逐项）。
#[derive(Clone, Copy, Default)]
pub struct D {
    pub estoi: f64,
    pub scoreq: f64,
    pub distill: f64,
    pub onset: f64,
    pub bgstab: f64,
    pub timbre: f64,
    /// 吃字：百分点（参考字数两边都是 20）
    pub eat_pp: f64,
    /// 人声被压低（dB）
    pub splvl: f64,
    pub lat: f64,
}

/// 种子 1（报告种子）相对对照：每项都更好一点。
pub fn good1() -> D {
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
pub fn good2() -> D {
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

/// 种子 3：另一个方向的抖动。
pub fn good3() -> D {
    D {
        estoi: 0.048,
        scoreq: 0.078,
        distill: 0.078,
        onset: -0.18,
        bgstab: -0.18,
        timbre: -0.08,
        ..D::default()
    }
}

/// 一项的分数：过线取正本（`thresholds` 表），否则重新汇总出来的卡片状态会跟真跑的不一样。
fn sc(v: &'static str, x: f64) -> (&'static str, bool, Score) {
    let b = info(v).map_or(vdn_eval::Better::Lower, |i| i.better);
    let thr = match v {
        "V-ESTOI" => Some(th::ESTOI.def("min")),
        "V-ONSET" => Some(th::ONSET.def("max")),
        "V-TIMBRE" => Some(th::TIMBRE.def("max")),
        "V-BGSTAB" => Some(th::BGSTAB.def("max")),
        "V-SPLVL" => Some(th::SPLVL.def("min")),
        "V-LAT" => Some(vdn_eval::system::BUDGET_MS),
        _ => Some(0.0),
    };
    (v, false, Score::judged(v, x, "x", b, thr, &[]))
}

/// 对照的一个场景：`sources` 个源片段 × 2 个设备。`sources = 1` 时区间算不出来（组数不足）。
/// 打哪些项按成绩单标准的适用范围（`standard::items`）：V-BGSTAB 在安静类场景不适用。
pub fn ctl(scene: MeetingPlace, sources: usize) -> Vec<ClipRow> {
    let applicable = |v: &str| {
        vdn_eval::scorecard::standard::items(scene)
            .iter()
            .any(|(i, _)| i.v == v)
    };
    (0..sources * 2)
        .map(|i| {
            let w = 0.01 * i as f64;
            let mut scores = vec![
                sc("V-ESTOI", 0.6 + w),
                sc("V-NRMOS-SCOREQ", 3.4 + w),
                sc("V-NRMOS-DISTILL", 3.2 + w),
                sc("V-ONSET", 0.5 + w),
                sc("V-TIMBRE", 1.2 + w),
                sc("V-BGSTAB", 4.0 + w),
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
            scores.retain(|(v, _, _)| applicable(v));
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
pub fn cand(rows: &[ClipRow], d: D, drop: &[&str], system: &str) -> Vec<ClipRow> {
    rows.iter()
        .map(|r| {
            let mut n = r.clone();
            n.system = system.into();
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

/// 临时目录 + 造输入文件的小工具。`suite` 由 [`build`] 填，`accept_args` 自动带上 `--suite`。
pub struct Fix {
    pub root: PathBuf,
    suite: RefCell<Option<String>>,
}

impl Fix {
    pub fn new(tag: &str) -> Fix {
        let root = std::env::temp_dir().join(format!("vdn-accept-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        Fix {
            root,
            suite: RefCell::new(None),
        }
    }

    pub fn set_suite(&self, p: &str) {
        *self.suite.borrow_mut() = Some(p.to_string());
    }

    pub fn jsonl(&self, name: &str, rows: &[ClipRow]) -> String {
        let p = self.root.join(format!("{name}.jsonl"));
        std::fs::write(&p, vdn_eval::scorecard::load::to_jsonl(rows).unwrap()).unwrap();
        p.to_string_lossy().to_string()
    }

    /// 用真的 `compare --by-scene` 造一份 compare.json。
    pub fn compare(&self, tag: &str, a: &str, b: &str) -> String {
        let out = self.root.join(format!("cmp-{tag}"));
        let args: Vec<String> = ["--a", a, "--b", b, "--by-scene"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        vdn_eval::scorecard::compare_out::cli(&args, "t", out.clone()).unwrap();
        out.join("compare.json").to_string_lossy().to_string()
    }

    /// 指定两边系统各跑一次（同一份文件含多个系统时用它）。
    pub fn compare_sys(&self, tag: &str, a: &str, b: &str, sa: &str, sb: &str) -> String {
        let out = self.root.join(format!("cmp-{tag}"));
        let args: Vec<String> = [
            "--a",
            a,
            "--b",
            b,
            "--by-scene",
            "--system-a",
            sa,
            "--system-b",
            sb,
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        vdn_eval::scorecard::compare_out::cli(&args, "t", out.clone()).unwrap();
        out.join("compare.json").to_string_lossy().to_string()
    }

    pub fn file(&self, name: &str, text: &str) -> String {
        let p = self.root.join(format!("{name}.json"));
        std::fs::write(&p, text).unwrap();
        p.to_string_lossy().to_string()
    }

    /// 写一份成绩单目录：`scorecard.json` + 旁边的 `clips.jsonl`（accept 会核它的 sha256）。
    /// `clips_from` 填哪份逐段文件的路径（空 = 不写 clips.jsonl，造「缺文件」）。
    pub fn card(&self, name: &str, text: &str, clips_from: &str) -> String {
        let d = self.root.join(name);
        std::fs::create_dir_all(&d).unwrap();
        if !clips_from.is_empty() {
            std::fs::copy(clips_from, d.join("clips.jsonl")).unwrap();
        }
        let p = d.join("scorecard.json");
        std::fs::write(&p, text).unwrap();
        p.to_string_lossy().to_string()
    }

    pub fn accept_args(
        &self,
        vs: &[String],
        noise: &[String],
        a: &str,
        b: &str,
        tag: &str,
    ) -> Vec<String> {
        self.accept_args_scenes(vs, noise, a, b, tag, "cafe,meeting-room")
    }

    pub fn accept_args_scenes(
        &self,
        vs: &[String],
        noise: &[String],
        a: &str,
        b: &str,
        tag: &str,
        scenes: &str,
    ) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();
        for v in vs {
            args.push("--vs".into());
            args.push(v.clone());
        }
        for n in noise {
            args.push("--noise".into());
            args.push(n.clone());
        }
        let out = self.root.join(format!("acc-{tag}"));
        let suite = self
            .suite
            .borrow()
            .clone()
            .unwrap_or_else(|| self.root.join("suite.json").to_string_lossy().to_string());
        for (k, v) in [
            ("--suite", suite),
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

    pub fn accept(&self, vs: &[String], noise: &[String], a: &str, b: &str, tag: &str) -> Value {
        let args = self.accept_args(vs, noise, a, b, tag);
        let d = vdn_eval::scorecard::accept::cli(&args, "t", self.root.join("default"))
            .unwrap_or_else(|e| panic!("accept 出错：{e}"));
        serde_json::from_str(&std::fs::read_to_string(d.join("accept.json")).unwrap()).unwrap()
    }
}

/// 一份手写的成绩单（只带 accept 要读的字段；格式照 `scorecard::card::Card::to_json`）。
/// `fp` = 评测清单指纹（accept 会核）；`ext` = `meta.external`。
pub fn card_json(rows: &[(&str, &str, &str, &str, &str)], ext: Value, fp: &str) -> String {
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
    json!({"doc": "测试用成绩单",
        "meta": {"run_id": "t", "systems": ["cand"], "external": ext,
                 "suite_fingerprint": fp},
        "rules": "v2", "clips_scored": 8 * scenes.len(), "scenes": scenes, "worst": [],
        "missing": [], "no_clean_pass": []})
    .to_string()
}

/// 把某张卡的 V-LAT 均值改成 `ms`（造延迟不一致）。
pub fn set_lat(text: &str, scene: &str, ms: f64) -> String {
    let mut v: Value = serde_json::from_str(text).unwrap();
    for c in v["scenes"].as_array_mut().into_iter().flatten() {
        if c["scene"] == scene {
            for i in c["items"].as_array_mut().into_iter().flatten() {
                if i["v"] == "V-LAT" {
                    i["mean"] = json!(ms);
                }
            }
        }
    }
    v.to_string()
}

/// 三张卡：两张目标场景 + 一张非目标场景（library-classroom）；系统名可指定
/// （`--card-a` 必须是对照的系统、`--card-b` 必须是候选的系统）。
pub fn cards_for(a_sys: &str, b_sys: &str, a_status: &str, b_status: &str) -> (String, String) {
    let a = [
        ("cafe", "common", "mba-builtin", a_sys, a_status),
        ("meeting-room", "common", "mba-builtin", a_sys, a_status),
        (
            "library-classroom",
            "common",
            "mba-builtin",
            a_sys,
            a_status,
        ),
    ];
    let b = [
        ("cafe", "common", "mba-builtin", b_sys, a_status),
        ("meeting-room", "common", "mba-builtin", b_sys, a_status),
        (
            "library-classroom",
            "common",
            "mba-builtin",
            b_sys,
            b_status,
        ),
    ];
    (card_json(&a, json!([]), ""), card_json(&b, json!([]), ""))
}

/// 候选三张卡：两张目标场景同状态，非目标场景（library-classroom）单独给状态。
pub fn cand_cards(target: &str, other: &str) -> String {
    let rows = [
        ("cafe", "common", "mba-builtin", "cand", target),
        ("meeting-room", "common", "mba-builtin", "cand", target),
        ("library-classroom", "common", "mba-builtin", "cand", other),
    ];
    card_json(&rows, json!([]), "")
}

/// 一份输入的构成。
#[derive(Clone, Copy)]
pub struct Spec<'a> {
    pub d1: D,
    pub d2: D,
    pub d3: D,
    /// 种子 1 里整段不出现的项
    pub drop1: &'a [&'a str],
    /// 种子 1 里这些场景整段不出现的项（别的场景还有 → 一边判不了，但也有判得了的段）
    pub drop1_in: &'a [(MeetingPlace, &'a [&'a str])],
    /// 这些场景只放 1 个源片段（2 段）：区间算不出来（证据不足）
    pub thin: &'a [MeetingPlace],
    /// 这些场景整段不放进比较用的逐段文件（模拟整场景漏跑）
    pub omit: &'a [MeetingPlace],
    /// 两边都整段不出现这一项（模拟「不适用」）
    pub absent: &'a [(MeetingPlace, &'a str)],
    /// 种子 1 里这些场景的 V-LAT 平移（造逐卡延迟不一致）
    pub lat1: &'a [(MeetingPlace, f64)],
    /// 种子 1 里这些场景整段不放进逐段文件（候选漏跑一个场景）
    pub omit1: &'a [MeetingPlace],
    /// 种子 1 里这些片段 id 整段不放进逐段文件（候选漏跑一段）
    pub omit_clip1: &'a [&'a str],
    /// 种子 2 里这些片段 id 整段不放进逐段文件（另一个种子漏跑一段）
    pub omit_clip2: &'a [&'a str],
    /// 种子 1 里这些 (片段 id, 项) 的分数置空（段在、值没有）
    pub null1: &'a [(&'a str, &'static str)],
    /// 对照里这些场景塞一段否决项不过的分（卡状态由逐段文件重新汇总，靠这个把它弄成不过）
    pub poison_ctl: &'a [(MeetingPlace, &'static str)],
    /// 种子 1 里这些场景塞一段否决项不过的分
    pub poison1: &'a [(MeetingPlace, &'static str)],
}

/// 三个种子都更好、不额外丢项。
pub fn spec(d1: D, d2: D, d3: D) -> Spec<'static> {
    Spec {
        d1,
        d2,
        d3,
        drop1: &[],
        drop1_in: &[],
        thin: &[],
        omit: &[],
        absent: &[],
        lat1: &[],
        omit1: &[],
        omit_clip1: &[],
        omit_clip2: &[],
        null1: &[],
        poison_ctl: &[],
        poison1: &[],
    }
}

/// 把某个场景里某些项改成「已判不过」的分数（卡片状态由逐段文件重新汇总，靠这个造不过的卡）。
pub fn poison(rows: &mut [ClipRow], scene: MeetingPlace, item: &'static str) {
    for r in rows.iter_mut().filter(|r| r.scene == scene) {
        let veto = vdn_eval::scorecard::standard::VETO
            .iter()
            .any(|i| i.v == item);
        let mut s = Score::judged(item, 1.0, "x", vdn_eval::Better::Lower, Some(0.0), &[]);
        s.verdict = vdn_eval::Verdict::Fail;
        r.scores.retain(|(v, _, _)| *v != item);
        r.scores.push((item, veto, s));
    }
}

/// 一份完整输入：两个目标场景 + 一个非目标场景，三个种子 + 三对种子互比，
/// 加一份覆盖全部片段的评测清单（`--suite`）与两张带 `clips.jsonl` 的成绩单。
pub struct Good {
    pub vs: Vec<String>,
    pub noise: Vec<String>,
    pub card_a: String,
    pub card_b: String,
    pub suite: String,
    /// 对照 / 报告种子的逐段文件（写别的卡时用）
    pub ctl: String,
    pub cand1: String,
    /// 评测清单指纹（写别的卡时用）
    pub fp: String,
}

pub fn build(f: &Fix, s: Spec<'_>) -> Good {
    let scenes = [
        MeetingPlace::Cafe,
        MeetingPlace::MeetingRoom,
        MeetingPlace::LibraryClassroom,
    ];
    let mut a = Vec::new();
    for sc in scenes {
        if s.omit.contains(&sc) {
            continue;
        }
        let n = if s.thin.contains(&sc) { 1 } else { 4 };
        let mut rows = ctl(sc, n);
        for r in rows.iter_mut() {
            for (_, v) in s.absent.iter().filter(|(x, _)| *x == sc) {
                r.scores.retain(|(id, _, _)| id != v);
            }
        }
        a.extend(rows);
    }
    for (sc, item) in s.poison_ctl {
        poison(&mut a, *sc, item);
    }
    let ctl_p = f.jsonl("ctl", &a);
    let mut seed1 = cand(&a, s.d1, s.drop1, "cand");
    for (sc, items) in s.drop1_in {
        for r in seed1.iter_mut().filter(|r| r.scene == *sc) {
            r.scores.retain(|(v, _, _)| !items.contains(v));
        }
    }
    for (sc, item) in s.poison1 {
        poison(&mut seed1, *sc, item);
    }
    for (sc, ms) in s.lat1 {
        for r in seed1.iter_mut().filter(|r| r.scene == *sc) {
            for (v, _, x) in r.scores.iter_mut().filter(|(v, _, _)| *v == "V-LAT") {
                if let Some(x0) = x.value {
                    x.value = Some(x0 + ms);
                    let _ = v;
                }
            }
        }
    }
    seed1.retain(|r| !s.omit1.contains(&r.scene) && !s.omit_clip1.contains(&r.clip.as_str()));
    // 段都在、但这一项没有值（模拟 null）：配对段数会因此变少
    for (clip, item) in s.null1 {
        for r in seed1.iter_mut().filter(|r| r.clip == *clip) {
            for (_, _, x) in r.scores.iter_mut().filter(|(v, _, _)| v == item) {
                x.value = None;
            }
        }
    }
    let cand1 = f.jsonl("c1", &seed1);
    let mut seed2 = cand(&a, s.d2, &[], "cand");
    seed2.retain(|r| !s.omit_clip2.contains(&r.clip.as_str()));
    let c2 = f.jsonl("c2", &seed2);
    let c3 = f.jsonl("c3", &cand(&a, s.d3, &[], "cand"));

    // 评测清单：每个场景一条任务，指向「全部 4 个源片段 × 2 设备」的片段清单（预期集合）
    let listed: Vec<(MeetingPlace, Vec<String>)> = scenes
        .iter()
        .map(|p| (*p, ctl(*p, 4).into_iter().map(|r| r.clip).collect()))
        .collect();
    let (suite_path, fp) = manifest::write(&f.root, &listed);
    let suite = suite_path;
    f.set_suite(&suite);

    let (ca, cb) = cards_for("ref", "cand", "pass", "pass");
    let ca = card_json_with(&ca, &fp);
    let cb = card_json_with(&cb, &fp);
    Good {
        vs: vec![
            f.compare("vs1", &ctl_p, &cand1),
            f.compare("vs2", &ctl_p, &c2),
            f.compare("vs3", &ctl_p, &c3),
        ],
        noise: vec![
            f.compare("n12", &cand1, &c2),
            f.compare("n13", &cand1, &c3),
            f.compare("n23", &c2, &c3),
        ],
        card_a: f.card("a", &ca, &ctl_p),
        card_b: f.card("b", &cb, &cand1),
        suite,
        ctl: ctl_p,
        cand1,
        fp,
    }
}

/// 给成绩单填上清单指纹。
pub fn card_json_with(text: &str, fp: &str) -> String {
    let mut v: Value = serde_json::from_str(text).unwrap();
    v["meta"]["suite_fingerprint"] = json!(fp);
    v.to_string()
}
