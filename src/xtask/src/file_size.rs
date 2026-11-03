//! tidy 的「单文件行数上限」（complexity-budget 动作 1；issue #62）。
//!
//! | 规则 | 查什么 | 怎么过 |
//! |---|---|---|
//! | R-SIZE | `src/**/*.rs` 每个文件 ≤ [`MAX_LINES`] 行；存量超标的记在基线 [`BASELINE_FILE`]（正本，只此一份），只许变小 | 按职责拆文件（类型 / 参数解析 / 运行 / 输出），不许机械切 part1/part2 |
//!
//! 棘轮：
//! - 不在基线里的文件超线 → 红
//! - 在基线里的文件比基线行数多 → 红
//! - 基线条目对应的文件已删除或已 ≤ 上限 → 红（条目作废，删掉它；跑 `cargo run -p xtask -- size-baseline`）
//! - 比基线少但仍超线 → 只出注释，提示收紧基线（并行的线改同一文件时不互相卡红）
//!
//! `cargo run -p xtask -- size-baseline`：按现状**只降不升**地重写基线（删作废条目、把变小的条目收紧）；
//! 变大的条目保持原值，仍会红——基线不许用来放宽。

use std::fs;
use std::path::Path;

use crate::tidy::rs_files;

/// 单文件行数上限。
pub const MAX_LINES: usize = 600;
/// 基线正本（相对 workspace 根）。
pub const BASELINE_FILE: &str = "src/xtask/file-size-baseline.txt";

const HEADER: &str = "\
# R-SIZE 基线（src/xtask/src/file_size.rs）：存量超 600 行的文件，只许变小。
# 格式：<行数> <相对仓库根的路径>。不要手工调大；收紧跑 `cargo run -p xtask -- size-baseline`。
";

/// 一个文件：相对路径、行数。
pub type Sized = (String, usize);

/// 解析基线；格式错的行记进 `problems`。
pub fn parse_baseline(text: &str, problems: &mut Vec<String>) -> Vec<Sized> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let mut it = l.splitn(2, char::is_whitespace);
        match (it.next().and_then(|n| n.parse::<usize>().ok()), it.next()) {
            (Some(n), Some(p)) if !p.trim().is_empty() => out.push((p.trim().to_string(), n)),
            _ => problems.push(format!(
                "{BASELINE_FILE}:{}：格式应为「<行数> <路径>」，现在是 {l:?}（R-SIZE）",
                i + 1
            )),
        }
    }
    out
}

/// 比对现状与基线：返回（问题，注释）。
pub fn size_problems(files: &[Sized], baseline: &[Sized]) -> (Vec<String>, Vec<String>) {
    let mut problems = Vec::new();
    let mut notes = Vec::new();
    for (rel, n) in files {
        let base = baseline.iter().find(|(p, _)| p == rel).map(|(_, b)| *b);
        match base {
            None if *n > MAX_LINES => problems.push(format!(
                "{rel}：{n} 行，超过上限 {MAX_LINES} 行（R-SIZE）。按职责拆成几个模块（类型 / 参数解析 / 运行 / 输出），每个 ≤ {MAX_LINES} 行"
            )),
            Some(b) if *n > b => problems.push(format!(
                "{rel}：{n} 行，比基线 {b} 行多 {}（R-SIZE 只许变小）。把新加的部分拆到别的模块",
                n - b
            )),
            Some(b) if *n > MAX_LINES && *n < b => notes.push(format!(
                "{rel}：{n} 行，比基线 {b} 行少；跑 `cargo run -p xtask -- size-baseline` 收紧基线（R-SIZE）"
            )),
            _ => {}
        }
    }
    for (p, b) in baseline {
        match files.iter().find(|(rel, _)| rel == p) {
            None => problems.push(format!(
                "{BASELINE_FILE}：条目 {p}（{b} 行）对应的文件已不存在，删掉这条（R-SIZE）"
            )),
            Some((_, n)) if *n <= MAX_LINES => problems.push(format!(
                "{BASELINE_FILE}：{p} 已降到 {n} 行（≤ {MAX_LINES}），删掉这条基线（R-SIZE）"
            )),
            _ => {}
        }
    }
    (problems, notes)
}

/// 按现状只降不升地重写基线条目。
pub fn tightened(files: &[Sized], baseline: &[Sized]) -> Vec<Sized> {
    let mut out: Vec<Sized> = baseline
        .iter()
        .filter_map(|(p, b)| {
            let (_, n) = files.iter().find(|(rel, _)| rel == p)?;
            (*n > MAX_LINES).then(|| (p.clone(), (*n).min(*b)))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

pub fn render(entries: &[Sized]) -> String {
    let mut s = HEADER.to_string();
    for (p, n) in entries {
        s.push_str(&format!("{n} {p}\n"));
    }
    s
}

/// `src/**/*.rs` 的（相对路径，行数）。
pub fn scan(root: &Path) -> Vec<Sized> {
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    all.into_iter()
        .filter_map(|f| {
            let rel = f.strip_prefix(root).ok()?.display().to_string();
            let n = fs::read_to_string(&f).ok()?.lines().count();
            Some((rel, n))
        })
        .collect()
}

pub fn check(root: &Path, problems: &mut Vec<String>, notes: &mut Vec<String>) {
    let text = fs::read_to_string(root.join(BASELINE_FILE)).unwrap_or_default();
    let baseline = parse_baseline(&text, problems);
    let (p, n) = size_problems(&scan(root), &baseline);
    problems.extend(p);
    notes.extend(n);
}

/// `size-baseline` 子命令：收紧基线并写回。返回是否成功。
pub fn write_tightened(root: &Path) -> bool {
    let path = root.join(BASELINE_FILE);
    let text = fs::read_to_string(&path).unwrap_or_default();
    let mut bad = Vec::new();
    let baseline = parse_baseline(&text, &mut bad);
    if !bad.is_empty() {
        for b in bad {
            eprintln!("{b}");
        }
        return false;
    }
    let next = tightened(&scan(root), &baseline);
    match fs::write(&path, render(&next)) {
        Ok(()) => {
            println!("size-baseline：{} 条（原 {} 条）", next.len(), baseline.len());
            true
        }
        Err(e) => {
            eprintln!("写不了 {BASELINE_FILE}：{e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(p: &str, n: usize) -> Sized {
        (p.to_string(), n)
    }

    #[test]
    fn new_file_over_limit_is_flagged() {
        let (p, _) = size_problems(&[f("src/a.rs", 601)], &[]);
        assert_eq!(p.len(), 1);
        assert!(p[0].contains("src/a.rs") && p[0].contains("601"), "{p:?}");
    }

    #[test]
    fn file_at_limit_passes() {
        let (p, n) = size_problems(&[f("src/a.rs", 600)], &[]);
        assert!(p.is_empty() && n.is_empty());
    }

    #[test]
    fn baseline_file_may_not_grow() {
        let base = [f("src/a.rs", 700)];
        assert!(size_problems(&[f("src/a.rs", 700)], &base).0.is_empty());
        let (p, _) = size_problems(&[f("src/a.rs", 701)], &base);
        assert_eq!(p.len(), 1, "{p:?}");
    }

    #[test]
    fn shrunk_baseline_file_gives_note_not_error() {
        let (p, n) = size_problems(&[f("src/a.rs", 650)], &[f("src/a.rs", 700)]);
        assert!(p.is_empty());
        assert_eq!(n.len(), 1);
    }

    #[test]
    fn stale_baseline_entries_are_flagged() {
        let base = [f("src/gone.rs", 700), f("src/small.rs", 700)];
        let (p, _) = size_problems(&[f("src/small.rs", 600)], &base);
        assert_eq!(p.len(), 2, "{p:?}");
    }

    #[test]
    fn malformed_baseline_line_is_flagged() {
        let mut p = Vec::new();
        let b = parse_baseline("# x\n\n700 src/a.rs\nabc src/b.rs\n700\n", &mut p);
        assert_eq!(b, vec![f("src/a.rs", 700)]);
        assert_eq!(p.len(), 2, "{p:?}");
    }

    #[test]
    fn tighten_only_lowers() {
        let files = [f("src/a.rs", 650), f("src/b.rs", 900), f("src/c.rs", 500)];
        let base = [f("src/b.rs", 800), f("src/a.rs", 700), f("src/c.rs", 700), f("src/d.rs", 700)];
        assert_eq!(tightened(&files, &base), vec![f("src/a.rs", 650), f("src/b.rs", 800)]);
        let mut p = Vec::new();
        assert_eq!(parse_baseline(&render(&tightened(&files, &base)), &mut p).len(), 2);
        assert!(p.is_empty());
    }

    /// 真仓库的基线能解析，且每条都确实超线（作废条目由 tidy_passes 兜底）。
    #[test]
    fn repo_baseline_parses() {
        let root = crate::root();
        let text = fs::read_to_string(root.join(BASELINE_FILE)).expect("基线文件存在");
        let mut p = Vec::new();
        let b = parse_baseline(&text, &mut p);
        assert!(p.is_empty(), "{p:?}");
        assert!(b.iter().all(|(_, n)| *n > MAX_LINES));
    }
}
