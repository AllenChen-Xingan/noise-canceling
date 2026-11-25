//! 转写文本的比较：分词、「两次算不算一样」、编辑距离对齐（从 asr.rs 拆出，≤600 行）。

/// 两次转写算不算一样：按评分用的分词（[`tokens`]）比，大小写、标点、空白的差别不算（大总管 2026-11-25 冒烟：
/// 原先逐字比原文，评分看不见的差别也判成不一致）。
pub fn same_text(a: &str, b: &str, lang: &str) -> bool {
    tokens(a, lang) == tokens(b, lang)
}

/// 分词：中文按字，英文按词（小写、只留字母数字与撇号）。
pub fn tokens(text: &str, lang: &str) -> Vec<String> {
    if lang == "zh" {
        text.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_lowercase().to_string())
            .collect()
    } else {
        text.split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '\'')
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }
}

/// Levenshtein 对齐，返回 (删除, 替换, 插入)。回溯优先级与 exp003 `edit_ops` 一致：对角 → 删除 → 插入。
pub fn edit_ops(r: &[String], h: &[String]) -> (usize, usize, usize) {
    let (n, m) = (r.len(), h.len());
    let mut d = vec![vec![0usize; m + 1]; n + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, v) in d[0].iter_mut().enumerate() {
        *v = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let sub = d[i - 1][j - 1] + usize::from(r[i - 1] != h[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(sub);
        }
    }
    let (mut i, mut j) = (n, m);
    let (mut del, mut sub, mut ins) = (0, 0, 0);
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && d[i][j] == d[i - 1][j - 1] + usize::from(r[i - 1] != h[j - 1]) {
            sub += usize::from(r[i - 1] != h[j - 1]);
            i -= 1;
            j -= 1;
        } else if i > 0 && d[i][j] == d[i - 1][j] + 1 {
            del += 1;
            i -= 1;
        } else {
            ins += 1;
            j -= 1;
        }
    }
    (del, sub, ins)
}
