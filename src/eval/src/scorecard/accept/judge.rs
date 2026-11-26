//! `vdn-eval accept` 的逐条判定（从 `accept.rs` 拆出，R-SIZE）：一条验收线一个函数，
//! 结论只有过 / 不过 / 判不了；阈值一律从 `thresholds::ACCEPT` 取，这里不写数。

use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::thresholds as th;

/// 区间能不能用来判：有差值、且源片段组数够（`SC-AGG.min_groups`）。
/// 组数不够时 compare 只给「不确定」——那是证据不足，不是「不适用」。
fn usable(i: &Item) -> bool {
    i.diff
        .is_some_and(|d| d.groups as f64 >= th::AGG.def("min_groups"))
}

/// 为什么这个区间判不了（写进报告）。
fn unusable(i: &Item) -> String {
    match i.diff {
        None => "差值区间算不出来".to_string(),
        Some(d) => format!(
            "只有 {} 个源片段组（要 ≥ {:.0} 组）",
            d.groups,
            th::AGG.def("min_groups")
        ),
    }
}

/// 一项在一个场景里「齐不齐」（正本：要涨与不许退都用它）：
/// - `Ok(false)` = 不适用（成绩单标准 `standard::items` 不在这个场景看这一项）
/// - `Err(为什么)` = 适用但不齐：这一项在 compare 里没有数，或配对段数少于清单给这个场景的段数
///   （有段缺分 / 只有一边有值，剩下的段判不了这一项）
/// - `Ok(true)` = 适用且齐，可以拿来判
fn item_complete(cmp: &Cmp, scene: &str, item: &str, expected: &Expected) -> Result<bool, String> {
    let Some(place) = MeetingPlace::ALL.into_iter().find(|p| p.as_str() == scene) else {
        return Err(format!("{scene} 不是场景键，认不出这一项适用不适用"));
    };
    if !crate::scorecard::standard::items(place)
        .iter()
        .any(|(i, _)| i.v == item)
    {
        return Ok(false);
    }
    let want = expected.clips_in(scene);
    match cmp.item(scene, item) {
        None => Err(format!(
            "{} · {scene} 没有这一项的数（清单有 {want} 段）",
            cmp.name()
        )),
        Some(i) if i.pairs < want => Err(format!(
            "{} · {scene} · {item} 只配上 {} 段，清单有 {want} 段",
            cmp.name(),
            i.pairs
        )),
        Some(_) => Ok(true),
    }
}

/// 种子齐不齐：`--vs` 必须正好 `SC-ACCEPT.seeds` 份、`--noise` 必须正好每两两互比一份。
///
/// 身份 = 成绩单内容 sha256 + 系统名（`compare.json` 的 `meta.a_sha256` / `meta.b_sha256` 与
/// `meta.a_system` / `meta.b_system`）。同一份文件里的两个系统是两个不同候选；换了系统不算重复。
/// 要求：三份 `--vs` 的对照是同一份、候选是三个不同身份、`--noise` 的集合恰好是这三个候选的全部两两组合。
/// 缺身份、端点不是这三个候选、缺一对 → `Some(为什么)`，要涨的线一律判不了（codex 第 1 · 2 轮 P1）。
fn seed_shortage(vs: &[Cmp], noise: &[Cmp]) -> Option<String> {
    let want = th::ACCEPT.def("seeds") as usize;
    if vs.len() != want {
        return Some(format!(
            "--vs 给了 {} 份，规则要 {want} 个种子（SC-ACCEPT.seeds · D99 第 1 条）",
            vs.len()
        ));
    }
    let pairs_want = want * (want - 1) / 2;
    if noise.len() != pairs_want {
        return Some(format!(
            "--noise 给了 {} 份，{want} 个种子要每两两互比 {pairs_want} 份（D99 第 1 条）",
            noise.len()
        ));
    }
    let Some(cands) = vs
        .iter()
        .map(|c| c.b_id.clone())
        .collect::<Option<Vec<Ident>>>()
    else {
        return Some(
            "有 --vs 没记候选成绩单的 sha256 或系统名（meta.b_sha256 / meta.b_system），核不了是哪个种子"
                .into(),
        );
    };
    let Some(controls) = vs
        .iter()
        .map(|c| c.a_id.clone())
        .collect::<Option<Vec<Ident>>>()
    else {
        return Some(
            "有 --vs 没记对照成绩单的 sha256 或系统名（meta.a_sha256 / meta.a_system）".into(),
        );
    };
    if controls.windows(2).any(|w| w[0] != w[1]) {
        return Some("这几份 --vs 的对照不是同一份成绩单（sha256 或系统不同）".into());
    }
    let set: BTreeSet<Ident> = cands.iter().cloned().collect();
    if set.len() != want {
        return Some(format!(
            "--vs 里只有 {} 个不同候选（成绩单 sha256 + 系统名有重复）",
            set.len()
        ));
    }
    let mut got: BTreeSet<(Ident, Ident)> = BTreeSet::new();
    for c in noise {
        let (Some(x), Some(y)) = (c.a_id.clone(), c.b_id.clone()) else {
            return Some(
                "有 --noise 没记两边成绩单的 sha256 或系统名，核不了是哪两个候选的互比".into(),
            );
        };
        if !set.contains(&x) || !set.contains(&y) || x == y {
            return Some(format!(
                "--noise 里有一对不是这次三个候选之间的互比（{} 对 {}）",
                x.1, y.1
            ));
        }
        got.insert(if x <= y { (x, y) } else { (y, x) });
    }
    if got.len() != pairs_want {
        return Some(format!(
            "--noise 只有 {} 对不同的候选互比，要 {pairs_want} 对",
            got.len()
        ));
    }
    None
}

/// 「要涨」的一行：每个种子都判「涨」、报告种子的差值均值过线、且大于种子噪声底。
fn gain_row(
    v: &str,
    key: &str,
    scene: &str,
    vs: &[Cmp],
    noise: &[Cmp],
    expected: &Expected,
) -> Row {
    let line = th::ACCEPT.def(key);
    let what = format!("要涨 {v}（场景 {scene}；线 SC-ACCEPT.{key} = {line:+.3}）");
    if let Some(why) = seed_shortage(vs, noise) {
        return Row::new(what, Verdict::Undecided, format!("种子不齐，判不了：{why}"));
    }
    // 三个种子与三对互比里任何一份缺预期片段 → 这一项的涨跌没覆盖全，判不了（codex 第 7 轮 P1）
    let cov = expected.coverage_compares(vs, noise);
    if !cov.is_empty() {
        return Row::new(
            what,
            Verdict::Undecided,
            format!("覆盖不齐，判不了：{}", cov.join("；")),
        );
    }
    let (mut bad, mut hole, mut seen) = (Vec::new(), Vec::new(), Vec::new());
    for c in vs {
        match item_complete(c, scene, v, expected) {
            Err(why) => hole.push(why),
            Ok(false) => hole.push(format!("{} 这个场景不看 {v}", c.name())),
            Ok(true) => {
                let i = c.item(scene, v).expect("item_complete 已保证有数");
                if !usable(&i) {
                    hole.push(format!(
                        "{} 这个场景这一项的{}，判不了",
                        c.name(),
                        unusable(&i)
                    ));
                } else {
                    seen.push(i.call.zh().to_string());
                    if i.call != Call::Up {
                        bad.push(format!("{} 判「{}」", c.name(), i.call.zh()));
                    }
                }
            }
        }
    }
    let rep = vs.first().expect("呼叫方保证 --vs 非空");
    if let Some(i) = rep.item(scene, v).filter(usable) {
        match i.diff {
            None => hole.push(format!("{} 这个场景这一项{}", rep.name(), unusable(&i))),
            Some(d) => {
                seen.push(format!("报告种子差值均值 {:+.3}", d.mean));
                if d.mean < line {
                    bad.push(format!("均值 {:+.3} < 线 {line:+.3}", d.mean));
                }
                let mut floor: Option<f64> = None;
                for c in noise {
                    match item_complete(c, scene, v, expected) {
                        Err(why) => hole.push(why),
                        Ok(false) => hole.push(format!("{} 这个场景不看 {v}", c.name())),
                        Ok(true) => {
                            let i = c.item(scene, v).expect("item_complete 已保证有数");
                            match i.diff.filter(|_| usable(&i)) {
                                Some(x) => {
                                    let a = x.mean.abs();
                                    floor = Some(floor.map_or(a, |m: f64| m.max(a)));
                                }
                                None => hole.push(format!(
                                    "{} 这个场景这一项的{}",
                                    c.name(),
                                    unusable(&i)
                                )),
                            }
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
fn regress_row(v: &str, key: &str, rep: &Cmp, expected: &Expected) -> Row {
    let line = th::ACCEPT.def(key);
    let what = format!("不许退 {v}（报告种子区间上界 ≤ SC-ACCEPT.{key} = {line:+.3} dB）");
    let (mut seen, mut bad, mut hole) = (Vec::new(), Vec::new(), Vec::new());
    // 对照成绩单有的场景，比较里必须有逐场景的数：整场景漏跑时这一项没被检查过，不能算过
    for scene in expected.scenes() {
        if !rep.scopes.contains_key(&scene) {
            hole.push(format!("compare 里没有场景 {scene}（清单有）"));
        }
    }
    for (scene, s) in rep.scenes() {
        // 「不适用」与「齐不齐」都走同一个函数（codex 第 10 轮 P1）
        match item_complete(rep, scene, v, expected) {
            Ok(false) => continue,
            Err(why) => {
                hole.push(why);
                continue;
            }
            Ok(true) => {}
        }
        let Some(i) = s.items.get(v) else {
            hole.push(format!("{scene} 适用这一项，但两边都没有这一项的数"));
            continue;
        };
        let Some(d) = i.diff.filter(|_| usable(i)) else {
            hole.push(format!("{scene} {}", unusable(i)));
            continue;
        };
        seen.push(format!("{scene} 上界 {:+.3}", d.hi));
        if d.hi > line {
            bad.push(format!("{scene} 上界 {:+.3} > {line:+.3}", d.hi));
        }
    }
    let verdict = if !bad.is_empty() {
        Verdict::Fail
    } else if !hole.is_empty() || seen.is_empty() {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    let mut detail = if seen.is_empty() {
        format!("{} 没有任何场景有这个项的值", rep.name())
    } else {
        format!("{} 个场景：{} dB", seen.len(), seen.join(" · "))
    };
    if !bad.is_empty() {
        detail = format!("不过：{}", bad.join("；"));
    }
    if !hole.is_empty() {
        detail.push_str(&format!("；证据不足，判不了：{}", hole.join("；")));
    }
    Row::new(what, verdict, detail)
}

/// 「不新增否决段」的一行：报告种子的 b 新增否决段 = 0；「一边判不了」只列数。
fn veto_row(v: &str, rep: &Cmp) -> Row {
    let what = format!("不新增否决段 {v}（报告种子）");
    let (mut a, mut b, mut new, mut und, mut judged, mut found) = (0, 0, 0, 0, 0, false);
    for (_, s) in rep.scenes() {
        if let Some(x) = s.vetoes.get(v) {
            found = true;
            a += x.a;
            b += x.b;
            new += x.new_in_b;
            und += x.undecided;
            judged += s.pairs.saturating_sub(x.undecided);
        }
    }
    if !found {
        return Row::new(
            what,
            Verdict::Undecided,
            format!("{} 里没有这个否决项", rep.name()),
        );
    }
    if judged == 0 {
        // 「一边判不了」不算增减：一段都没判过时，没有任何证据说明「不新增否决段」（codex 第 2 轮 P1 #3）
        return Row::new(
            what,
            Verdict::Undecided,
            format!("两边都判得了的段 0 段（一边判不了 {und} 段）：证明不了「不新增」"),
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

/// 「覆盖」的一行：清单要的场景 / 切片 / 片段都要在两份逐段文件、两份成绩单与 compare 里
/// （`Expected::coverage`）。双方误用同一份子集（都漏跑一个场景）时涨退两项都恰好没检查它；
/// 这一行把它暴露出来（codex 第 2 · 3 · 4 轮）。
fn coverage_row(expected: &Expected, a: &CardIn, b: &CardIn, vs: &[Cmp], noise: &[Cmp]) -> Row {
    let what = "覆盖：清单要的场景与片段都在每一份比较里".to_string();
    let hole = expected.coverage(a, b, vs, noise);
    let verdict = if hole.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Undecided
    };
    let detail = if hole.is_empty() {
        format!(
            "{} 个场景、{} 段都在比较里",
            expected.scenes().len(),
            expected.keys.len()
        )
    } else {
        format!("证据不足，判不了：{}", hole.join("；"))
    };
    Row::new(what, verdict, detail)
}

/// 「其他场景不新增不过」的一行：按 场景 × 档 × 设备 对卡（两边的系统各自），目标场景以外
/// 从「过 / 待测」变成「不过」就不过。
fn cards_row(a: &CardIn, b: &CardIn, scenes: &[String]) -> Row {
    let what = "其他场景不新增「不过」（目标场景以外）".to_string();
    let key = |r: &CardRow| (r.scene.clone(), r.tier.clone(), r.device.clone());
    let idx: BTreeMap<_, &CardRow> = b.rows.iter().map(|r| (key(r), r)).collect();
    let (mut total, mut checkable, mut bad, mut hole) = (0, 0, Vec::new(), Vec::new());
    for r in &a.rows {
        if scenes.contains(&r.scene) {
            continue; // 目标场景的卡归上面的「要涨 / 不许退」管
        }
        total += 1;
        if r.status == "fail" {
            continue; // 对照本来就不不过：不是「新增」
        }
        checkable += 1;
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
    if total == 0 {
        return Row::new(
            what,
            Verdict::Undecided,
            format!("{} 没有目标场景以外的卡，这一行没东西比", a.path),
        );
    }
    if checkable == 0 {
        return Row::new(
            what,
            Verdict::Pass,
            format!("{total} 张非目标场景的卡在对照里全是不过，没有可新增的"),
        );
    }
    let verdict = if !bad.is_empty() {
        Verdict::Fail
    } else if !hole.is_empty() {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    let mut detail = format!("{checkable} 张可比（共 {total} 张）非目标场景的卡，过 / 待测都没变");
    if !bad.is_empty() {
        detail = format!("不过：{}", bad.join("；"));
    }
    if !hole.is_empty() {
        detail.push_str(&format!("；判不了：{}", hole.join("；")));
    }
    Row::new(what, verdict, detail)
}

/// 「延迟仍等于基线」的一行：**逐卡**比（场景 × 档 × 设备），不取均值。
///
/// 每张清单要的卡两边都得有 V-LAT 读数，差过 `LAT_SAME_MS` 即不过；缺一张读数就判不了
/// （codex 第 4 轮 C）。候选是外部系统（只有声明值）时整项判不了。
fn lat_row(a: &CardIn, b: &CardIn, expected: &Expected) -> Row {
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
    let find = |c: &CardIn, scene: &str, tier: &str, device: &str| {
        c.rows
            .iter()
            .find(|r| r.scene == scene && r.tier == tier && r.device == device)
            .cloned()
    };
    let (mut seen, mut bad, mut hole) = (Vec::new(), Vec::new(), Vec::new());
    for (scene, tier, device) in expected.slices() {
        let k = format!("{scene} · {tier} · {device}");
        let (x, y) = (
            find(a, &scene, &tier, &device),
            find(b, &scene, &tier, &device),
        );
        match (
            x.as_ref().and_then(|r| r.lat_ms),
            y.as_ref().and_then(|r| r.lat_ms),
        ) {
            (Some(x), Some(y)) => {
                seen.push(format!("{k} {y:.2} ms"));
                if (y - x).abs() > LAT_SAME_MS {
                    bad.push(format!("{k} 候选 {y:.2} ms ≠ 基线 {x:.2} ms"));
                }
            }
            _ => hole.push(format!("{k} 缺 V-LAT 读数")),
        }
    }
    let verdict = if !bad.is_empty() {
        Verdict::Fail
    } else if !hole.is_empty() || seen.is_empty() {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    let mut detail = format!("{} 张卡：{}", seen.len(), seen.join(" · "));
    if !bad.is_empty() {
        detail = format!("不过：{}", bad.join("；"));
    }
    if !hole.is_empty() {
        detail.push_str(&format!("；判不了：{}", hole.join("；")));
    }
    Row::new(what, verdict, detail)
}

/// 逐条判。`vs` 非空，第一份是报告种子。
pub(super) fn evaluate(
    vs: &[Cmp],
    noise: &[Cmp],
    scenes: &[String],
    expected: &Expected,
    card_a: &CardIn,
    card_b: &CardIn,
) -> Result<Report, String> {
    let rep = vs
        .first()
        .ok_or("没有 --vs：没有候选对对照的比较，判不了")?;
    let mut rows = Vec::new();
    for scene in scenes {
        for (v, key) in GAINS {
            rows.push(gain_row(v, key, scene, vs, noise, expected));
        }
    }
    for (v, key) in REGRESS {
        rows.push(regress_row(v, key, rep, expected));
    }
    for v in VETO {
        rows.push(veto_row(v, rep));
    }
    rows.push(coverage_row(expected, card_a, card_b, vs, noise));
    rows.push(cards_row(card_a, card_b, scenes));
    rows.push(lat_row(card_a, card_b, expected));
    let verdict = if rows.iter().any(|r| r.verdict == Verdict::Fail) {
        Verdict::Fail
    } else if rows.iter().any(|r| r.verdict == Verdict::Undecided) {
        Verdict::Undecided
    } else {
        Verdict::Pass
    };
    Ok(Report { rows, verdict })
}
