//! 同一条音频几次转写不一样时怎么判（正本，大总管 2026-11-25 定 A）：前 2 次分词后相同就用第 1 次；
//! 不同就调第 [`TIE_BREAK_REP`] 次，几次里有 [`MAJORITY`] 次相同就用其中最早的那次；都不同判「不知道」。
//! 起因：温度 0 钉不住（钉温度探针 15 条里 1 条三次不一致，6.7%，线 2%；报告在主目录 `_tmp/20261125-asr-probe/`）。
//! 「相同」一律按评分用的分词比（[`super::asr::same_text`]）。缓存键不变（`sha|型号|语言|次`），第 3 次就是「次」= 3。

use super::asr::same_text;

/// 前两次分词后不同时补调的那一次（缓存键里的「次」）。
pub const TIE_BREAK_REP: usize = 3;
/// 几次转写里至少这么多次分词后相同，才算数。
pub const MAJORITY: usize = 2;
// 多数要过半，才不会两种文本各占多数（编译期核）
const _: () = assert!(MAJORITY * 2 > TIE_BREAK_REP);

/// 判的结果。
#[derive(Debug, PartialEq, Eq)]
pub enum Vote {
    /// 用第几次（0 起）的文本
    Use(usize),
    /// 还不够：要补调第 [`TIE_BREAK_REP`] 次
    NeedTieBreak,
    /// 已调满 [`TIE_BREAK_REP`] 次仍没有多数：判「不知道」
    NoMajority,
}

/// 按次序给出已有的几次转写（第 1 次在前），判用哪一次。
pub fn vote(texts: &[&str], lang: &str) -> Vote {
    let agree = |i: usize| {
        texts
            .iter()
            .filter(|t| same_text(t, texts[i], lang))
            .count()
    };
    if let Some(i) = (0..texts.len()).find(|&i| agree(i) >= MAJORITY) {
        return Vote::Use(i);
    }
    if texts.len() < TIE_BREAK_REP {
        Vote::NeedTieBreak
    } else {
        Vote::NoMajority
    }
}

/// 同 [`vote`]，收自有字符串。
pub fn vote_on(texts: &[String], lang: &str) -> Vote {
    vote(&texts.iter().map(String::as_str).collect::<Vec<_>>(), lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_table() {
        let v = |t: &[&str]| vote(t, "en");
        // 前两次相同：用第 1 次，不补调（标点、大小写不算不同）
        assert_eq!(v(&["Hello, world.", "hello world"]), Vote::Use(0));
        // 前两次不同：补调第 3 次
        assert_eq!(v(&["a b", "a c"]), Vote::NeedTieBreak);
        // 第 3 次同第 1 次 → 用第 1 次；同第 2 次 → 用第 2 次
        assert_eq!(v(&["a b", "a c", "a b"]), Vote::Use(0));
        assert_eq!(v(&["a b", "a c", "A c."]), Vote::Use(1));
        // 三次都不同 → 不知道
        assert_eq!(v(&["a b", "a c", "a d"]), Vote::NoMajority);
        // 中文按字比：探针里那条（多了个「五」）
        let z = |t: &[&str]| vote(t, "zh");
        assert_eq!(
            z(&[
                "第一二三四点，四。",
                "第一二三四点，四五。",
                "第一、二、三四点，四。"
            ]),
            Vote::Use(0)
        );
    }

    /// 常量与规则对得上：要的是「3 次里 2 次」，不是「全部相同」。
    #[test]
    fn constants() {
        assert_eq!((TIE_BREAK_REP, MAJORITY), (3, 2));
    }
}
