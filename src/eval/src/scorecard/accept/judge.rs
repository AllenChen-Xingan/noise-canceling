//! `vdn-eval accept` 的逐条判定（从 `accept.rs` 拆出，R-SIZE）：一条验收线一个函数，
//! 结论只有过 / 不过 / 判不了；阈值一律从 `thresholds::ACCEPT` 取，这里不写数。

use std::collections::BTreeMap;

use super::*;
use crate::thresholds as th;

/// 「要涨」的一行：每个种子都判「涨」、报告种子的差值均值过线、且大于种子噪声底。
fn gain_row(v: &str, key: &str, scene: &str, vs: &[Cmp], noise: &[Cmp]) -> Row {
    let line = th::ACCEPT.def(key);
    let what = format!("要涨 {v}（场景 {scene}；线 SC-ACCEPT.{key} = {line:+.3}）");
    let (mut bad, mut hole, mut seen) = (Vec::new(), Vec::new(), Vec::new());
    for c in vs {
        match c.item(scene, v) {
            None => hole.push(format!("{} 没有这个场景这一项的数", c.name())),
            Some(i) => {
                seen.push(i.call.zh().to_string());
                if i.call != Call::Up {
                    bad.push(format!("{} 判「{}」", c.name(), i.call.zh()));
                }
            }
        }
    }
    let rep = vs.first().expect("呼叫方保证 --vs 非空");
    if let Some(i) = rep.item(scene, v) {
        match i.diff {
            None => hole.push(format!("{} 这个场景这一项差值区间算不出来", rep.name())),
            Some(d) => {
                seen.push(format!("报告种子差值均值 {:+.3}", d.mean));
                if d.mean < line {
                    bad.push(format!("均值 {:+.3} < 线 {line:+.3}", d.mean));
                }
                if noise.is_empty() {
                    hole.push("没给 --noise，量不出种子噪声底".into());
                }
                let mut floor: Option<f64> = None;
                for c in noise {
                    match c.item(scene, v).and_then(|i| i.diff) {
                        None => hole.push(format!("{} 没有这个场景这一项的数", c.name())),
                        Some(x) => {
                            let a = x.mean.abs();
                            floor = Some(floor.map_or(a, |m: f64| m.max(a)));
                        }
                    }
                }
                if let Some(m) = floor {
                    seen.push(format!("种子噪声底 {m:.3}"));
                    if d.mean <= m {
                        bad.push(format!("涨幅 {:.3} 没过种子噪声底 {m:.3}", d.mean));
                    }
                }
            }
        }
    }
    let verdict = if !bad.is_empty() {
        Verdict::Fail
    } else if !hole.is_empty() {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    let mut detail = seen.join(" · ");
    if !bad.is_empty() {
        detail.push_str(&format!("；不过：{}", bad.join("；")));
    }
    if !hole.is_empty() {
        detail.push_str(&format!("；判不了：{}", hole.join("；")));
    }
    Row::new(what, verdict, detail)
}

/// 「不许退」的一行：报告种子在每个有这个项值的场景上，差值区间上界 ≤ 线。
fn regress_row(v: &str, key: &str, rep: &Cmp) -> Row {
    let line = th::ACCEPT.def(key);
    let what = format!("不许退 {v}（报告种子区间上界 ≤ SC-ACCEPT.{key} = {line:+.3} dB）");
    let (mut seen, mut bad) = (Vec::new(), Vec::new());
    for (scene, s) in rep.scenes() {
        let Some(i) = s.items.get(v).filter(|i| i.pairs > 0) else {
            continue; // 这个场景没有这一项（如 V-BGSTAB）：不算失败
        };
        let Some(d) = i.diff else { continue };
        seen.push(format!("{scene} 上界 {:+.3}", d.hi));
        if d.hi > line {
            bad.push(format!("{scene} 上界 {:+.3} > {line:+.3}", d.hi));
        }
    }
    if seen.is_empty() {
        return Row::new(
            what,
            Verdict::Undecided,
            format!("{} 没有任何场景有这个项的值", rep.name()),
        );
    }
    let verdict = if bad.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Fail
    };
    let detail = if bad.is_empty() {
        format!("{} 个场景：{} dB", seen.len(), seen.join(" · "))
    } else {
        format!("不过：{}", bad.join("；"))
    };
    Row::new(what, verdict, detail)
}

/// 「不新增否决段」的一行：报告种子的 b 新增否决段 = 0；「一边判不了」只列数。
fn veto_row(v: &str, rep: &Cmp) -> Row {
    let what = format!("不新增否决段 {v}（报告种子）");
    let (mut a, mut b, mut new, mut und, mut found) = (0, 0, 0, 0, false);
    for (_, s) in rep.scenes() {
        if let Some(x) = s.vetoes.get(v) {
            found = true;
            a += x.a;
            b += x.b;
            new += x.new_in_b;
            und += x.undecided;
        }
    }
    if !found {
        return Row::new(
            what,
            Verdict::Undecided,
            format!("{} 里没有这个否决项", rep.name()),
        );
    }
    let verdict = if new > 0 {
        Verdict::Fail
    } else {
        Verdict::Pass
    };
    Row::new(
        what,
        verdict,
        format!(
            "对照否决 {a} 段 → 候选否决 {b} 段 · b 新增 {new} 段 · 一边判不了 {und} 段（不算增减）"
        ),
    )
}

/// 「其他场景不新增不过」的一行：按 场景 × 档 × 设备 对卡（两边的系统各自），目标场景以外
/// 从「过 / 待测」变成「不过」就不过。
fn cards_row(a: &CardIn, b: &CardIn, scenes: &[String]) -> Row {
    let what = "其他场景不新增「不过」（目标场景以外）".to_string();
    let key = |r: &CardRow| (r.scene.clone(), r.tier.clone(), r.device.clone());
    let idx: BTreeMap<_, &CardRow> = b.rows.iter().map(|r| (key(r), r)).collect();
    let (mut n, mut bad, mut hole) = (0, Vec::new(), Vec::new());
    for r in &a.rows {
        if scenes.contains(&r.scene) {
            continue; // 目标场景的卡归上面的「要涨 / 不许退」管
        }
        if r.status == "fail" {
            continue; // 对照本来就不不过：不是「新增」
        }
        n += 1;
        match idx.get(&key(r)) {
            None => hole.push(format!(
                "{} · {} · {}（{}）候选成绩单里没有这张卡",
                r.scene, r.tier, r.device, r.system
            )),
            Some(x) if x.status == "fail" => bad.push(format!(
                "{} · {} · {}：{} → 不过（{} → {}）",
                r.scene, r.tier, r.device, r.status, r.system, x.system
            )),
            Some(_) => {}
        }
    }
    if n == 0 {
        return Row::new(
            what,
            Verdict::Undecided,
            format!("{} 没有目标场景以外的卡，这一行没东西比", a.path),
        );
    }
    let verdict = if !bad.is_empty() {
        Verdict::Fail
    } else if !hole.is_empty() {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    let mut detail = format!("{n} 张非目标场景的卡，过 / 待测都没变");
    if !bad.is_empty() {
        detail = format!("不过：{}", bad.join("；"));
    }
    if !hole.is_empty() {
        detail.push_str(&format!("；判不了：{}", hole.join("；")));
    }
    Row::new(what, verdict, detail)
}

/// 「延迟仍等于基线」的一行：线 = 基线成绩单量出来的 V-LAT；候选是外部系统（只有声明值）→ 判不了。
fn lat_row(a: &CardIn, b: &CardIn) -> Row {
    let what = "延迟仍等于基线 V-LAT".to_string();
    if b.external {
        return Row::new(
            what,
            Verdict::Undecided,
            format!(
                "候选 `{}` 是外部系统：延迟只是声明值，判不了（014 第三节第 4 条；甲路没有实测延迟）",
                b.path
            ),
        );
    }
    match (a.lat_ms, b.lat_ms) {
        (Some(x), Some(y)) if (y - x).abs() <= LAT_SAME_MS => Row::new(
            what,
            Verdict::Pass,
            format!("候选 {y:.2} ms = 基线 {x:.2} ms"),
        ),
        (Some(x), Some(y)) => Row::new(
            what,
            Verdict::Fail,
            format!("候选 {y:.2} ms ≠ 基线 {x:.2} ms（线：与基线相等）"),
        ),
        _ => Row::new(
            what,
            Verdict::Undecided,
            "对照或候选成绩单里没有 V-LAT 的读数".to_string(),
        ),
    }
}

/// 逐条判。`vs` 非空，第一份是报告种子。
pub(super) fn evaluate(
    vs: &[Cmp],
    noise: &[Cmp],
    scenes: &[String],
    card_a: &CardIn,
    card_b: &CardIn,
) -> Result<Report, String> {
    let rep = vs
        .first()
        .ok_or("没有 --vs：没有候选对对照的比较，判不了")?;
    let mut rows = Vec::new();
    for scene in scenes {
        for (v, key) in GAINS {
            rows.push(gain_row(v, key, scene, vs, noise));
        }
    }
    for (v, key) in REGRESS {
        rows.push(regress_row(v, key, rep));
    }
    for v in VETO {
        rows.push(veto_row(v, rep));
    }
    rows.push(cards_row(card_a, card_b, scenes));
    rows.push(lat_row(card_a, card_b));
    let verdict = if rows.iter().any(|r| r.verdict == Verdict::Fail) {
        Verdict::Fail
    } else if rows.iter().any(|r| r.verdict == Verdict::Undecided) {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    Ok(Report { rows, verdict })
}
