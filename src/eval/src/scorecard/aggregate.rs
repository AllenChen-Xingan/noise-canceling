//! 新规则（v2，人定 D85 题 1 = B）的统计部分：组合内均值 + 按源片段成组的 bootstrap 95% 区间 → 三态；
//! 「不过」方向做 Holm 校正（调研 025 X1 · X2 · X4）。
//!
//! - **成组**：同一句话的两种设备版本（片段 id 只差末尾设备后缀，如 `-ext` / `-mba`）不独立，整组一起抽
//! - **三态**：区间整个在线的好的一侧 = 过；整个在坏的一侧 = 不过；跨线 = 待测
//! - **Holm**：每项一族，族 = 该项在这张成绩单里所有有区间的组合；只校正「不过」：区间判不过、但 Holm 不拒绝的改判待测
//!
//! 参数（区间水平、重采样次数、种子、Holm 的 α）正本在 `thresholds` 表的 `SC-AGG`。

use std::collections::BTreeMap;

use super::rules::CardStatus;
use crate::Better;
use crate::rng::Rng;
use crate::thresholds as th;

/// 一个组合里一项的区间结果。
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Ci {
    pub mean: f64,
    pub lo: f64,
    pub hi: f64,
    /// 参与抽样的源片段组数
    pub groups: usize,
    /// 单侧 p 值：H0「均值不比线差」下，重采样均值落在线的好的一侧（含线上）的比例（+1 平滑）
    pub p_fail: f64,
}

/// 源片段 id：去掉末尾的设备后缀（`phone-booth-c07-ext` → `phone-booth-c07`）。没有 `-` 时原样返回。
pub fn source_of(clip: &str) -> &str {
    clip.rsplit_once('-').map_or(clip, |(a, _)| a)
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// 按源片段成组的 bootstrap：每次有放回地抽组，取抽中各组全部段的均值。`vals` = (片段 id, 值)，非有限值不计。
/// 没有有限值时返回 `None`。种子、次数、水平取 `SC-AGG`。
pub fn bootstrap(vals: &[(&str, f64)], thr: f64, better: Better, seed: u64) -> Option<Ci> {
    let mut by: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for (c, x) in vals.iter().filter(|(_, x)| x.is_finite()) {
        by.entry(source_of(c)).or_default().push(*x);
    }
    if by.is_empty() {
        return None;
    }
    let groups: Vec<&Vec<f64>> = by.values().collect();
    let all: Vec<f64> = groups.iter().flat_map(|g| g.iter().copied()).collect();
    let m = mean(&all);
    let reps = th::AGG.def("reps") as usize;
    let level = th::AGG.def("level");
    let mut rng = Rng::new(seed);
    let mut bs = Vec::with_capacity(reps);
    for _ in 0..reps {
        let (mut s, mut n) = (0.0, 0usize);
        for _ in 0..groups.len() {
            let g = groups[rng.below(groups.len())];
            s += g.iter().sum::<f64>();
            n += g.len();
        }
        bs.push(s / n as f64);
    }
    let q = crate::analysis::quantile;
    let a = (1.0 - level) / 2.0;
    let good = bs.iter().filter(|&&b| !better.worse(b, thr)).count();
    Some(Ci {
        mean: m,
        lo: q(&bs, a),
        hi: q(&bs, 1.0 - a),
        groups: groups.len(),
        p_fail: (good + 1) as f64 / (reps + 1) as f64,
    })
}

/// 区间对过线值的三态（线上算好的一侧，与 [`crate::Score::judged`] 的「不比线差即过」一致）。
pub fn tri(ci: &Ci, thr: f64, better: Better) -> CardStatus {
    let (good_end, bad_end) = match better {
        Better::Lower => (ci.hi, ci.lo),
        Better::Higher => (ci.lo, ci.hi),
    };
    if !better.worse(good_end, thr) {
        CardStatus::Pass
    } else if better.worse(bad_end, thr) {
        CardStatus::Fail
    } else {
        CardStatus::Unknown
    }
}

/// Holm 逐步下降：返回每个 p 是否被拒绝（α 取 `SC-AGG`）。
pub fn holm(ps: &[f64]) -> Vec<bool> {
    let alpha = th::AGG.def("alpha");
    let m = ps.len();
    let mut idx: Vec<usize> = (0..m).collect();
    idx.sort_by(|&a, &b| ps[a].total_cmp(&ps[b]));
    let mut rej = vec![false; m];
    for (k, &i) in idx.iter().enumerate() {
        if ps[i] <= alpha / (m - k) as f64 {
            rej[i] = true;
        } else {
            break;
        }
    }
    rej
}

/// 种子：基础种子 + 项名与组合键的稳定哈希（FNV-1a），同一输入每次结果相同、不同组合互不相关。
pub fn seed_for(item: &str, unit: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in item.bytes().chain([0]).chain(unit.bytes()) {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (th::AGG.def("seed") as u64) ^ h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vals(xs: &[f64]) -> Vec<(String, f64)> {
        xs.iter()
            .enumerate()
            .map(|(i, x)| {
                (
                    format!("s-c{:02}-{}", i / 2, if i % 2 == 0 { "ext" } else { "mba" }),
                    *x,
                )
            })
            .collect()
    }

    fn ci(xs: &[f64], thr: f64, b: Better) -> Ci {
        let v = vals(xs);
        let r: Vec<(&str, f64)> = v.iter().map(|(c, x)| (c.as_str(), *x)).collect();
        bootstrap(&r, thr, b, 1).unwrap()
    }

    #[test]
    fn source_strips_device_suffix() {
        assert_eq!(source_of("phone-booth-c07-ext"), "phone-booth-c07");
        assert_eq!(source_of("x"), "x");
    }

    #[test]
    fn three_states() {
        // 均值 1，按组波动 ±0.1（同组两段同值）：线 2 → 过；线 0 → 不过；线 1 → 跨线待测（越小越好）
        let xs: Vec<f64> = (0..20)
            .map(|i| 1.0 + if (i / 2) % 2 == 0 { 0.1 } else { -0.1 })
            .collect();
        assert_eq!(
            tri(&ci(&xs, 2.0, Better::Lower), 2.0, Better::Lower),
            CardStatus::Pass
        );
        assert_eq!(
            tri(&ci(&xs, 0.0, Better::Lower), 0.0, Better::Lower),
            CardStatus::Fail
        );
        assert_eq!(
            tri(&ci(&xs, 1.0, Better::Lower), 1.0, Better::Lower),
            CardStatus::Unknown
        );
        // 越大越好时方向反过来
        assert_eq!(
            tri(&ci(&xs, 0.0, Better::Higher), 0.0, Better::Higher),
            CardStatus::Pass
        );
        assert_eq!(
            tri(&ci(&xs, 2.0, Better::Higher), 2.0, Better::Higher),
            CardStatus::Fail
        );
    }

    #[test]
    fn groups_are_resampled_together() {
        // 两种设备版本取同值：组数 = 段数 / 2
        let c = ci(&[1.0, 1.0, 2.0, 2.0, 3.0, 3.0], 0.0, Better::Lower);
        assert_eq!(c.groups, 3);
        assert!((c.mean - 2.0).abs() < 1e-12);
        assert!(c.lo >= 1.0 && c.hi <= 3.0);
    }

    #[test]
    fn interval_covers_true_mean_about_95_percent() {
        // 正态样本 20 组：区间盖住真值 0 的比例应在 95% 附近（bootstrap 小样本偏窄，放宽到 88–99%）
        let mut r = Rng::new(42);
        let mut hit = 0;
        for k in 0..200 {
            let xs: Vec<f64> = (0..40).map(|_| r.normal()).collect();
            let v = vals(&xs);
            let rr: Vec<(&str, f64)> = v.iter().map(|(c, x)| (c.as_str(), *x)).collect();
            let c = bootstrap(&rr, 0.0, Better::Lower, k).unwrap();
            hit += usize::from(c.lo <= 0.0 && 0.0 <= c.hi);
        }
        assert!((176..=198).contains(&hit), "覆盖 {hit}/200");
    }

    #[test]
    fn same_seed_same_result() {
        let xs: Vec<f64> = (0..12).map(|i| i as f64).collect();
        assert_eq!(ci(&xs, 5.0, Better::Lower), ci(&xs, 5.0, Better::Lower));
    }

    #[test]
    fn non_finite_values_skipped_and_empty_is_none() {
        assert!(bootstrap(&[("a-x", f64::NAN)], 0.0, Better::Lower, 1).is_none());
        let c = bootstrap(&[("a-x", 1.0), ("b-x", f64::NAN)], 0.0, Better::Lower, 1).unwrap();
        assert_eq!(c.groups, 1);
    }

    #[test]
    fn holm_matches_hand_example() {
        // α = 0.025，m = 4：排序 0.001 ≤ 0.025/4 拒；0.006 ≤ 0.025/3 拒；0.02 > 0.025/2 停
        let r = holm(&[0.02, 0.001, 0.5, 0.006]);
        assert_eq!(r, vec![false, true, false, true]);
        assert!(holm(&[]).is_empty());
    }

    #[test]
    fn p_fail_small_when_clearly_worse() {
        let xs: Vec<f64> = (0..20).map(|i| 5.0 + 0.01 * i as f64).collect();
        assert!(ci(&xs, 1.0, Better::Lower).p_fail < 0.001);
        assert!(ci(&xs, 9.0, Better::Lower).p_fail > 0.99);
    }
}
