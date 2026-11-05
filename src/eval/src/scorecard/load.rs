//! `clips.jsonl`（每段每项的分数）的写出与读回：成绩单可以不重跑音频、只按新规则重新汇总（调研 025「只汇总」档）。
//!
//! 行格式的正本是本文件的 [`row_json`]（exp014 驱动写出的格式与它相同，读回测试用 exp014 的真实一行核对）。
//! 文件里不存每项的 `note`，读回时「不知道」的原因写成 [`NOTE_LOST`]。

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde_json::{Value, json};

use super::ClipRow;
use crate::scene::{MeetingPlace, SoundGroup, Tier};
use crate::{Score, Verdict};

/// 读回的「不知道」没有原文原因（clips.jsonl 不存 note）。
pub const NOTE_LOST: &str = "从 clips.jsonl 读回：原因未存";

/// 一段一行：段 · 场景 · 档 · 设备 · 系统 · 声音 · 每项（分数、判定、旁证）。
pub fn row_json(r: &ClipRow) -> Value {
    let sounds: Vec<Value> = r
        .sounds
        .iter()
        .map(|(c, g)| json!({"cls": c, "group": g}))
        .collect();
    let scores: Vec<Value> = r
        .scores
        .iter()
        .map(|(v, veto, s)| {
            let extra: serde_json::Map<String, Value> = s
                .extra
                .iter()
                .map(|(k, x)| ((*k).to_string(), json!(x.is_finite().then_some(*x))))
                .collect();
            json!({
                "v": v, "veto": veto,
                "value": s.value.filter(|x| x.is_finite()),
                "threshold": s.threshold,
                "verdict": s.verdict.as_str(),
                "extra": extra,
            })
        })
        .collect();
    json!({
        "clip": r.clip, "scene": r.scene, "tier": r.tier, "device": r.device,
        "system": r.system, "sounds": sounds, "scores": scores,
    })
}

/// 把整批行写成 jsonl 文本。
pub fn to_jsonl(rows: &[ClipRow]) -> Result<String, String> {
    let mut s = String::new();
    for r in rows {
        s += &serde_json::to_string(&row_json(r)).map_err(|e| e.to_string())?;
        s.push('\n');
    }
    Ok(s)
}

/// 旁证名要 `&'static str`（[`Score::extra`] 的类型）：读回时登记一次、重复使用，进程里只增不减（名字只有几十个）。
fn intern(k: &str) -> &'static str {
    static NAMES: Mutex<BTreeMap<String, &'static str>> = Mutex::new(BTreeMap::new());
    let mut m = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = m.get(k) {
        return s;
    }
    let s: &'static str = Box::leak(k.to_string().into_boxed_str());
    m.insert(k.to_string(), s);
    s
}

fn score_of(v: &Value, line: usize) -> Result<(&'static str, bool, Score), String> {
    let err = |w: &str| format!("第 {line} 行：{w}");
    let id = v["v"].as_str().ok_or_else(|| err("缺 v"))?;
    let info = crate::verifiers::info(id).ok_or_else(|| err(&format!("{id} 没登记")))?;
    let veto = v["veto"].as_bool().ok_or_else(|| err("缺 veto"))?;
    let verdict = match v["verdict"].as_str() {
        Some("pass") => Verdict::Pass,
        Some("fail") => Verdict::Fail,
        Some("unknown") => Verdict::Unknown(NOTE_LOST.into()),
        x => return Err(err(&format!("{id} 的 verdict 不认识：{x:?}"))),
    };
    let mut extra = Vec::new();
    if let Some(m) = v["extra"].as_object() {
        for (k, x) in m {
            extra.push((intern(k), x.as_f64().unwrap_or(f64::NAN)));
        }
    }
    let s = Score {
        id: info.id,
        value: v["value"].as_f64(),
        unit: info.unit,
        better: info.better,
        threshold: v["threshold"].as_f64(),
        grade: None,
        verdict,
        extra,
        note: String::new(),
    };
    Ok((info.id, veto, s))
}

/// 读一份 `clips.jsonl`（空行跳过）。出错时报行号。
pub fn parse_jsonl(text: &str) -> Result<Vec<ClipRow>, String> {
    let mut out = Vec::new();
    for (i, l) in text.lines().enumerate() {
        let line = i + 1;
        if l.trim().is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(l).map_err(|e| format!("第 {line} 行：{e}"))?;
        let s = |k: &str| -> Result<String, String> {
            v[k].as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("第 {line} 行：缺 {k}"))
        };
        let de = |k: &str| v[k].clone();
        let scene: MeetingPlace =
            serde_json::from_value(de("scene")).map_err(|e| format!("第 {line} 行 scene：{e}"))?;
        let tier: Tier =
            serde_json::from_value(de("tier")).map_err(|e| format!("第 {line} 行 tier：{e}"))?;
        let mut sounds = Vec::new();
        for x in v["sounds"].as_array().into_iter().flatten() {
            let cls = x["cls"].as_str().unwrap_or_default().to_string();
            let g: SoundGroup = serde_json::from_value(x["group"].clone())
                .map_err(|e| format!("第 {line} 行 sounds：{e}"))?;
            sounds.push((cls, g));
        }
        let scores = v["scores"]
            .as_array()
            .ok_or_else(|| format!("第 {line} 行：缺 scores"))?
            .iter()
            .map(|x| score_of(x, line))
            .collect::<Result<Vec<_>, _>>()?;
        out.push(ClipRow {
            clip: s("clip")?,
            scene,
            tier,
            device: s("device")?,
            system: s("system")?,
            sounds,
            scores,
        });
    }
    Ok(out)
}

/// 真空感（V-VACUUM，调研 025 B4）和残留噪声（V-NR）取的是同一个 TNLR：旧 run 的行里没有这一项时，
/// 用 V-NR 的数生成一列（只报不判，过线取表里的默认值）。已有这一项的行不动。
pub fn derive_vacuum(rows: &mut [ClipRow]) {
    let min = crate::thresholds::VACUUM.def("min");
    for r in rows {
        if r.scores.iter().any(|(v, _, _)| *v == "V-VACUUM") {
            continue;
        }
        // 有 V-NR 这一项的段都补一列（没值补「不知道」）：列的有无不能取决于哪一段排在前面（codex 中途 1 第 1 轮 P2 #4）
        let Some((_, _, nr)) = r.scores.iter().find(|(v, _, _)| *v == "V-NR") else {
            continue;
        };
        let s = match nr.value {
            Some(t) => Score::judged("V-VACUUM", t, "dB", crate::Better::Higher, Some(min), &[])
                .with_note("由同段 V-NR 的 TNLR 生成（025 B4）"),
            None => Score::unknown("V-VACUUM", "dB", crate::Better::Higher, "同段 V-NR 没有值"),
        };
        r.scores.push(("V-VACUUM", false, s));
    }
}

/// 同一次运行的「没跑成的段」：在 `clips.jsonl` 同目录的 `scorecard.json` 的 `missing` 里。
/// 找不到就报错——缺段决定组合最多判待测，丢了它重新汇总就和原表不等价（codex 中途 1 第 1 轮 P1 #1）。
pub fn missing_beside(clips: &std::path::Path) -> Result<Vec<super::Missing>, String> {
    let p = clips.with_file_name(super::card::FILES[0]);
    let text = std::fs::read_to_string(&p).map_err(|e| {
        format!(
            "{}：{e}（重新汇总要同目录的成绩单来读缺段记录）",
            p.display()
        )
    })?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}：{e}", p.display()))?;
    let m = v
        .get("missing")
        .ok_or_else(|| format!("{} 里没有 missing", p.display()))?;
    serde_json::from_value(m.clone()).map_err(|e| format!("{} 的 missing：{e}", p.display()))
}

/// 读一份 `clips.jsonl` 与同目录成绩单里的缺段记录，补上真空感一列，按 `rules` 重新汇总。
/// 返回成绩单与 `clips.jsonl` 的 sha256（写进元数据，可追溯）。
pub fn resummarize(
    path: &std::path::Path,
    rules: super::card::Rules,
) -> Result<(super::card::Card, String), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}：{e}", path.display()))?;
    let sha = crate::wav::sha256(path)?;
    let mut rows = parse_jsonl(&text)?;
    let missing = missing_beside(path)?;
    if rows.is_empty() && missing.is_empty() {
        return Err(format!("{} 里没有段", path.display()));
    }
    if rules == super::card::Rules::V1 {
        // v2 跑出的三项读数在干净人声那一路上（人定 D85 题 2），没有 v1 要的带噪读数：拒绝，免得把新口径当旧口径
        let clean = rows.iter().flat_map(|r| &r.scores).find(|(_, _, s)| {
            s.extra
                .iter()
                .any(|(k, v)| *k == super::standard::CLEAN_PASS_MARK && *v == 1.0)
        });
        if let Some((id, _, _)) = clean {
            return Err(format!(
                "{}：{id} 等项是干净人声那一路的读数（clean_pass），没有 v1 要的带噪读数；要旧规则请用 --rules v1 重跑音频",
                path.display()
            ));
        }
    }
    derive_vacuum(&mut rows);
    Ok((super::card::Card::build_with(&rows, missing, rules), sha))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// exp014 run `20261104-000444-e4511abc` 的真实一行（截短到 3 项）：格式与本文件写出的一致。
    const EXP014_LINE: &str = r#"{"clip":"home-quiet-c00-mba","device":"mba-builtin","scene":"home-quiet","scores":[{"extra":{"nonfinite":0.0,"over_4":0.0},"threshold":0.0,"v":"V-NAN","value":0.0,"verdict":"pass","veto":true},{"extra":{"noise_open":0.37362637362637363,"onset_miss":0.6666666666666666,"owner_hops":472.0},"threshold":0.05,"v":"V-SIM-GATE","value":0.125,"verdict":"fail","veto":false},{"extra":{},"threshold":null,"v":"V-ESTOI","value":null,"verdict":"unknown","veto":false}],"sounds":[{"cls":"bed-ace","group":"bed"},{"cls":"page-turn","group":"event"}],"system":"gtcrn","tier":"stress"}"#;

    #[test]
    fn reads_exp014_line_and_round_trips() {
        let rows = parse_jsonl(EXP014_LINE).unwrap();
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.scene, MeetingPlace::HomeQuiet);
        assert_eq!(r.tier, Tier::Stress);
        assert_eq!(r.sounds[1], ("page-turn".to_string(), SoundGroup::Event));
        let (v, veto, s) = &r.scores[1];
        assert_eq!((*v, *veto), ("V-SIM-GATE", false));
        assert!(s.verdict.is_fail());
        assert_eq!(s.value, Some(0.125));
        assert!(matches!(&r.scores[2].2.verdict, Verdict::Unknown(n) if n == NOTE_LOST));
        // 写出再读回：逐字段相同（serde_json 的键按字母序，与 exp014 文件同序）
        let back: Value = serde_json::from_str(to_jsonl(&rows).unwrap().trim()).unwrap();
        let orig: Value = serde_json::from_str(EXP014_LINE).unwrap();
        assert_eq!(back, orig);
    }

    #[test]
    fn vacuum_column_derived_from_nr() {
        let mut rows = parse_jsonl(EXP014_LINE).unwrap();
        derive_vacuum(&mut rows);
        assert!(
            rows[0].scores.iter().all(|(v, _, _)| *v != "V-VACUUM"),
            "没有 V-NR 就不生成"
        );
        let mut nr = Score::judged("V-NR", -45.0, "dB", crate::Better::Lower, Some(-6.0), &[]);
        nr.extra.clear();
        rows[0].scores.push(("V-NR", false, nr));
        derive_vacuum(&mut rows);
        derive_vacuum(&mut rows);
        let vac: Vec<_> = rows[0]
            .scores
            .iter()
            .filter(|(v, _, _)| *v == "V-VACUUM")
            .collect();
        assert_eq!(vac.len(), 1, "重复调用不重复加");
        assert_eq!(vac[0].2.value, Some(-45.0));
        assert!(vac[0].2.verdict.is_fail(), "−45 dB 低于 −40 dB 真空线");
    }

    /// 缺段记录跟着读回：有缺段的组合不能变成「过」，整组没跑成的组合不能消失；没有成绩单就报错。
    #[test]
    fn resummarize_keeps_missing_from_scorecard() {
        use super::super::card::Rules;
        let dir = std::env::temp_dir().join(format!("vdn-resum-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // 一段：坏值过、听感两项都 +0.1 → 旧规则下该组合「过」
        let ok = |id: &'static str, v: f64| {
            Score::judged(
                id,
                v,
                "x",
                crate::verifiers::info(id).unwrap().better,
                Some(0.0),
                &[],
            )
        };
        let row = ClipRow {
            clip: "cafe-c00-mba".into(),
            scene: MeetingPlace::Cafe,
            tier: Tier::Common,
            device: "mba-builtin".into(),
            system: "gtcrn".into(),
            sounds: Vec::new(),
            scores: vec![
                ("V-NAN", true, ok("V-NAN", 0.0)),
                ("V-NRMOS-SCOREQ", false, ok("V-NRMOS-SCOREQ", 0.1)),
                ("V-NRMOS-DISTILL", false, ok("V-NRMOS-DISTILL", 0.1)),
            ],
        };
        let clips = dir.join("clips.jsonl");
        std::fs::write(&clips, to_jsonl(&[row]).unwrap()).unwrap();
        assert!(
            resummarize(&clips, Rules::V1)
                .unwrap_err()
                .contains("scorecard.json")
        );
        let miss = |device: &str, what: &str| super::super::Missing {
            scene: MeetingPlace::Cafe,
            tier: Tier::Common,
            device: device.into(),
            system: "gtcrn".into(),
            what: what.into(),
            why: "读不了".into(),
        };
        let m = vec![
            miss("mba-builtin", "cafe-c01-mba"),
            miss("external-mic", "cafe-task"),
        ];
        std::fs::write(
            dir.join("scorecard.json"),
            json!({"missing": m}).to_string(),
        )
        .unwrap();
        let (card, _) = resummarize(&clips, Rules::V1).unwrap();
        assert_eq!(card.missing, m);
        assert_eq!(card.cards.len(), 2, "整组没跑成的组合也要出一行");
        let st = |dev: &str| card.cards.iter().find(|c| c.device == dev).unwrap().status;
        assert_eq!(
            st("mba-builtin"),
            super::super::rules::CardStatus::Unknown,
            "有缺段不能判过"
        );
        assert_eq!(st("external-mic"), super::super::rules::CardStatus::Unknown);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 真空感一列对每段都补（没值补「不知道」），所以列集合不随行序变（codex 中途 1 第 1 轮 P2 #4）。
    #[test]
    fn vacuum_column_present_whatever_row_order() {
        let nr = |v: Option<f64>| match v {
            Some(x) => Score::judged("V-NR", x, "dB", crate::Better::Lower, Some(-6.0), &[]),
            None => Score::unknown("V-NR", "dB", crate::Better::Lower, "x"),
        };
        let mut rows = parse_jsonl(&format!("{EXP014_LINE}\n{EXP014_LINE}")).unwrap();
        rows[0].scores.push(("V-NR", false, nr(None)));
        rows[1].scores.push(("V-NR", false, nr(Some(-20.0))));
        derive_vacuum(&mut rows);
        for r in &rows {
            assert!(r.scores.iter().any(|(v, _, _)| *v == "V-VACUUM"));
        }
        let cards = super::super::rules::summarize(&rows, &[]);
        assert!(cards[0].items.iter().any(|i| i.v == "V-VACUUM"));
    }

    #[test]
    fn unknown_verifier_and_bad_verdict_are_errors_with_line() {
        let bad = EXP014_LINE.replace("V-NAN", "V-NOPE");
        assert!(
            parse_jsonl(&format!("\n{bad}"))
                .unwrap_err()
                .contains("第 2 行")
        );
        let bad = EXP014_LINE.replace(r#""verdict":"pass""#, r#""verdict":"maybe""#);
        assert!(parse_jsonl(&bad).is_err());
    }

    /// v2 跑出来的 clips.jsonl 里，人声忽大忽小等三项是干净人声那一路的读数（带 `clean_pass`），没有 v1 要的带噪读数；
    /// 切回 v1 重新汇总必须拒绝并提示重跑，不能把干净路读数当旧口径（收尾评审第 1 轮 P2 #1）。
    #[test]
    fn v1_refuses_rows_scored_on_clean_pass() {
        use super::super::card::Rules;
        let dir = std::env::temp_dir().join(format!("vdn-resum-clean-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let pumpv = Score::judged(
            "V-PUMPV",
            0.5,
            "dB²",
            crate::verifiers::info("V-PUMPV").unwrap().better,
            Some(1.0),
            &[],
        )
        .with_extra(super::super::standard::CLEAN_PASS_MARK, 1.0);
        let row = ClipRow {
            clip: "cafe-c00-mba".into(),
            scene: MeetingPlace::Cafe,
            tier: Tier::Common,
            device: "mba-builtin".into(),
            system: "gtcrn".into(),
            sounds: Vec::new(),
            scores: vec![("V-PUMPV", false, pumpv)],
        };
        let clips = dir.join("clips.jsonl");
        std::fs::write(&clips, to_jsonl(&[row]).unwrap()).unwrap();
        std::fs::write(
            dir.join("scorecard.json"),
            json!({"missing": []}).to_string(),
        )
        .unwrap();
        let e = resummarize(&clips, Rules::V1).unwrap_err();
        assert!(e.contains("重跑") && e.contains("V-PUMPV"), "{e}");
        assert!(resummarize(&clips, Rules::V2).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
