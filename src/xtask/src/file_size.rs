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
//! - 基线格式错、路径重复、路径写法不规范（须以 `src/` 开头，不许 `./` `..` `\`）→ 红
//!
//! 基线本身不许悄悄放宽（codex 第 1 轮 P1 #1）：和目标分支的基线比对，**新增条目或调大上限**的那一行
//! 必须写 `# 放宽：<理由>`，否则红；理由随 PR diff 给人看。目标分支：CI 里是 `GITHUB_BASE_REF`
//! （浅检出时先 `git fetch` 它，取不到算红），本机是 `origin/main`（取不到只出注释）。
//! 目标分支上还没有基线文件（本规则首次引入）时不比对。
//!
//! `cargo run -p xtask -- size-baseline`：按现状**只降不升**地重写基线（删作废条目、把变小的条目收紧，
//! 保留 `# 放宽：` 理由）；变大的条目保持原值，仍会红。扫描或读基线出错时不写回。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 单文件行数上限。
pub const MAX_LINES: usize = 600;
/// 基线正本（相对 workspace 根）。
pub const BASELINE_FILE: &str = "src/xtask/file-size-baseline.txt";
/// 放宽理由的标记（写在基线条目行尾）。
pub const WIDEN_MARK: &str = "# 放宽：";

const HEADER: &str = "\
# R-SIZE 基线（src/xtask/src/file_size.rs）：存量超 600 行的文件，只许变小。
# 格式：<行数> <相对仓库根的路径>[ # 放宽：<理由>]。新增或调大条目必须写放宽理由；收紧跑 `cargo run -p xtask -- size-baseline`。
";

/// 一个文件：相对路径、行数。
pub type Sized = (String, usize);

/// 一条基线：路径、上限、放宽理由（没有就是 `None`）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub max: usize,
    pub reason: Option<String>,
}

/// 路径写法是否规范：`src/` 开头，不含 `./` `..` `\`、不以 `/` 结尾。
fn canonical_path(p: &str) -> bool {
    p.starts_with("src/")
        && !p.contains('\\')
        && !p.ends_with('/')
        && p.split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

/// 解析基线；格式错、路径不规范、路径重复的行记进 `problems`（重复时两条都不采信，免得结果依赖顺序）。
pub fn parse_baseline(text: &str, problems: &mut Vec<String>) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    let mut dup: Vec<String> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let (body, reason) = match l.split_once(WIDEN_MARK) {
            Some((b, r)) => (b.trim(), Some(r.trim().to_string())),
            None => (l, None),
        };
        let mut it = body.splitn(2, char::is_whitespace);
        let (n, p) = match (it.next().and_then(|n| n.parse::<usize>().ok()), it.next()) {
            (Some(n), Some(p)) if !p.trim().is_empty() => (n, p.trim().to_string()),
            _ => {
                problems.push(format!(
                    "{BASELINE_FILE}:{}：格式应为「<行数> <路径>[ {WIDEN_MARK}<理由>]」，现在是 {l:?}（R-SIZE）",
                    i + 1
                ));
                continue;
            }
        };
        if !canonical_path(&p) {
            problems.push(format!(
                "{BASELINE_FILE}:{}：路径 {p:?} 不规范（须以 src/ 开头，不许 ./ .. \\）（R-SIZE）",
                i + 1
            ));
            continue;
        }
        if reason.as_deref() == Some("") {
            problems.push(format!(
                "{BASELINE_FILE}:{}：「{WIDEN_MARK}」后面要写理由（R-SIZE）",
                i + 1
            ));
        }
        if out.iter().any(|e| e.path == p) {
            problems.push(format!(
                "{BASELINE_FILE}:{}：路径 {p} 重复；每个文件只许一条（R-SIZE）",
                i + 1
            ));
            dup.push(p);
            continue;
        }
        out.push(Entry {
            path: p,
            max: n,
            reason,
        });
    }
    out.retain(|e| !dup.contains(&e.path));
    out
}

/// 比对现状与基线：返回（问题，注释）。
pub fn size_problems(files: &[Sized], baseline: &[Entry]) -> (Vec<String>, Vec<String>) {
    let mut problems = Vec::new();
    let mut notes = Vec::new();
    for (rel, n) in files {
        let base = baseline.iter().find(|e| &e.path == rel).map(|e| e.max);
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
    for e in baseline {
        match files.iter().find(|(rel, _)| rel == &e.path) {
            None => problems.push(format!(
                "{BASELINE_FILE}：条目 {}（{} 行）对应的文件已不存在，删掉这条（R-SIZE）",
                e.path, e.max
            )),
            Some((_, n)) if *n <= MAX_LINES => problems.push(format!(
                "{BASELINE_FILE}：{} 已降到 {n} 行（≤ {MAX_LINES}），删掉这条基线（R-SIZE）",
                e.path
            )),
            _ => {}
        }
    }
    (problems, notes)
}

/// 和目标分支的基线比：新增条目、调大上限的，必须带放宽理由。
pub fn widen_problems(current: &[Entry], base: &[Entry]) -> Vec<String> {
    let mut problems = Vec::new();
    for e in current {
        let old = base.iter().find(|b| b.path == e.path).map(|b| b.max);
        let widened = match old {
            None => Some(format!("新增条目 {}（{} 行）", e.path, e.max)),
            Some(o) if e.max > o => Some(format!("{} 的上限从 {o} 调大到 {}", e.path, e.max)),
            _ => None,
        };
        if let Some(w) = widened
            && e.reason.as_deref().is_none_or(str::is_empty)
        {
            problems.push(format!(
                "{BASELINE_FILE}：{w}，比目标分支宽；先拆文件，确需放宽就在这行末尾写「{WIDEN_MARK}<理由>」（R-SIZE）"
            ));
        }
    }
    problems
}

/// 按现状只降不升地重写基线条目（保留放宽理由）。
pub fn tightened(files: &[Sized], baseline: &[Entry]) -> Vec<Entry> {
    let mut out: Vec<Entry> = baseline
        .iter()
        .filter_map(|e| {
            let (_, n) = files.iter().find(|(rel, _)| rel == &e.path)?;
            (*n > MAX_LINES).then(|| Entry {
                max: (*n).min(e.max),
                ..e.clone()
            })
        })
        .collect();
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

pub fn render(entries: &[Entry]) -> String {
    let mut s = HEADER.to_string();
    for e in entries {
        s.push_str(&format!("{} {}", e.max, e.path));
        if let Some(r) = &e.reason {
            s.push_str(&format!(" {WIDEN_MARK}{r}"));
        }
        s.push('\n');
    }
    s
}

/// 递归列出 `.rs` 文件；读目录出错就报错（不静默跳过，codex 第 1 轮 P2 #3）。
fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let rd = fs::read_dir(dir).map_err(|e| format!("读目录 {}：{e}", dir.display()))?;
    let mut entries = Vec::new();
    for e in rd {
        entries.push(
            e.map_err(|e| format!("读目录 {}：{e}", dir.display()))?
                .path(),
        );
    }
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out)?;
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
    Ok(())
}

/// `src/**/*.rs` 的（相对路径，行数）；任何文件读不了都报错。
pub fn scan(root: &Path) -> Result<Vec<Sized>, String> {
    let mut all = Vec::new();
    walk(&root.join("src"), &mut all)?;
    all.into_iter()
        .map(|f| {
            let rel = f
                .strip_prefix(root)
                .map_err(|_| format!("{} 不在仓库根下", f.display()))?
                .display()
                .to_string();
            let n = fs::read_to_string(&f)
                .map_err(|e| format!("读 {rel}：{e}"))?
                .lines()
                .count();
            Ok((rel, n))
        })
        .collect()
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// 目标分支的基线原文。`Ok(None)` = 目标分支上还没有这个文件（不比对）；
/// `Err` = 取不到目标分支（CI 里算红，本机只出注释，由调用方决定）。
fn base_baseline(root: &Path) -> Result<Option<String>, String> {
    let ci_base = std::env::var("GITHUB_BASE_REF")
        .ok()
        .filter(|b| !b.is_empty());
    let rref = match &ci_base {
        Some(b) => {
            let spec = format!("+refs/heads/{b}:refs/remotes/origin/{b}");
            // 浅检出时目标分支不在本地：取一层
            let _ = git(root, &["fetch", "-q", "--depth=1", "origin", &spec]);
            format!("origin/{b}")
        }
        None => "origin/main".to_string(),
    };
    if git(
        root,
        &["rev-parse", "--verify", "-q", &format!("{rref}^{{commit}}")],
    )
    .is_none()
    {
        return Err(format!("取不到目标分支 {rref}，没法核对基线有没有被放宽"));
    }
    Ok(git(root, &["show", &format!("{rref}:{BASELINE_FILE}")]))
}

pub fn check(root: &Path, problems: &mut Vec<String>, notes: &mut Vec<String>) {
    let text = match fs::read_to_string(root.join(BASELINE_FILE)) {
        Ok(t) => t,
        Err(e) => {
            problems.push(format!("读 {BASELINE_FILE}：{e}（R-SIZE）"));
            return;
        }
    };
    let baseline = parse_baseline(&text, problems);
    match scan(root) {
        Ok(files) => {
            let (p, n) = size_problems(&files, &baseline);
            problems.extend(p);
            notes.extend(n);
        }
        Err(e) => problems.push(format!("R-SIZE 扫描失败：{e}")),
    }
    match base_baseline(root) {
        Ok(Some(base_text)) => {
            let mut ignored = Vec::new();
            let base = parse_baseline(&base_text, &mut ignored);
            problems.extend(widen_problems(&baseline, &base));
        }
        Ok(None) => notes.push(format!(
            "目标分支上还没有 {BASELINE_FILE}，本次不比对基线放宽（R-SIZE）"
        )),
        Err(e) if std::env::var_os("GITHUB_BASE_REF").is_some_and(|b| !b.is_empty()) => {
            problems.push(format!("{e}（R-SIZE）"));
        }
        Err(e) => notes.push(format!("{e}；本机跳过（R-SIZE）")),
    }
}

/// `size-baseline` 子命令：收紧基线并写回。返回是否成功；基线或扫描有错时不写。
pub fn write_tightened(root: &Path) -> bool {
    let path = root.join(BASELINE_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("读 {BASELINE_FILE}：{e}");
            return false;
        }
    };
    let mut bad = Vec::new();
    let baseline = parse_baseline(&text, &mut bad);
    let files = match scan(root) {
        Ok(f) => f,
        Err(e) => {
            bad.push(format!("扫描失败：{e}"));
            Vec::new()
        }
    };
    if !bad.is_empty() {
        for b in bad {
            eprintln!("{b}");
        }
        eprintln!("size-baseline：有错，未写回");
        return false;
    }
    let next = tightened(&files, &baseline);
    match fs::write(&path, render(&next)) {
        Ok(()) => {
            println!(
                "size-baseline：{} 条（原 {} 条）",
                next.len(),
                baseline.len()
            );
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

    fn e(p: &str, n: usize) -> Entry {
        Entry {
            path: p.to_string(),
            max: n,
            reason: None,
        }
    }

    fn er(p: &str, n: usize, r: &str) -> Entry {
        Entry {
            reason: Some(r.to_string()),
            ..e(p, n)
        }
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
        let base = [e("src/a.rs", 700)];
        assert!(size_problems(&[f("src/a.rs", 700)], &base).0.is_empty());
        let (p, _) = size_problems(&[f("src/a.rs", 701)], &base);
        assert_eq!(p.len(), 1, "{p:?}");
    }

    #[test]
    fn shrunk_baseline_file_gives_note_not_error() {
        let (p, n) = size_problems(&[f("src/a.rs", 650)], &[e("src/a.rs", 700)]);
        assert!(p.is_empty());
        assert_eq!(n.len(), 1);
    }

    #[test]
    fn stale_baseline_entries_are_flagged() {
        let base = [e("src/gone.rs", 700), e("src/small.rs", 700)];
        let (p, _) = size_problems(&[f("src/small.rs", 600)], &base);
        assert_eq!(p.len(), 2, "{p:?}");
    }

    #[test]
    fn malformed_baseline_line_is_flagged() {
        let mut p = Vec::new();
        let b = parse_baseline("# x\n\n700 src/a.rs\nabc src/b.rs\n700\n", &mut p);
        assert_eq!(b, vec![e("src/a.rs", 700)]);
        assert_eq!(p.len(), 2, "{p:?}");
    }

    /// 重复路径两种顺序都报错，且两条都不采信（结果不依赖顺序）
    #[test]
    fn duplicate_baseline_paths_are_flagged_in_any_order() {
        for text in [
            "900 src/a.rs\n700 src/a.rs\n",
            "700 src/a.rs\n900 src/a.rs\n",
        ] {
            let mut p = Vec::new();
            let b = parse_baseline(text, &mut p);
            assert_eq!(p.len(), 1, "{text:?} → {p:?}");
            assert!(b.is_empty());
            // 800 行的文件因此按「不在基线里」判红
            assert_eq!(size_problems(&[f("src/a.rs", 800)], &b).0.len(), 1);
        }
    }

    #[test]
    fn non_canonical_paths_are_flagged() {
        for bad in [
            "./src/a.rs",
            "src/../src/a.rs",
            "src\\a.rs",
            "lib/a.rs",
            "src//a.rs",
        ] {
            let mut p = Vec::new();
            let b = parse_baseline(&format!("700 {bad}\n"), &mut p);
            assert!(b.is_empty() && p.len() == 1, "{bad} → {p:?}");
        }
    }

    /// 基线被放宽（调大上限、新增条目）而没写理由 → 红；写了理由 → 过；收紧 → 过
    #[test]
    fn widening_needs_a_reason() {
        let base = [e("src/a.rs", 700)];
        assert_eq!(widen_problems(&[e("src/a.rs", 900)], &base).len(), 1);
        assert_eq!(widen_problems(&[e("src/b.rs", 900)], &base).len(), 1);
        assert!(widen_problems(&[er("src/a.rs", 900, "合并 #57 带来")], &base).is_empty());
        assert!(widen_problems(&[e("src/a.rs", 650)], &base).is_empty());
        assert!(widen_problems(&[], &base).is_empty());
    }

    #[test]
    fn widen_reason_parses_and_empty_reason_is_flagged() {
        let mut p = Vec::new();
        let b = parse_baseline("900 src/a.rs # 放宽：合并 #57 带来\n", &mut p);
        assert!(p.is_empty(), "{p:?}");
        assert_eq!(b, vec![er("src/a.rs", 900, "合并 #57 带来")]);
        let mut p = Vec::new();
        parse_baseline("900 src/a.rs # 放宽：\n", &mut p);
        assert_eq!(p.len(), 1, "{p:?}");
    }

    #[test]
    fn tighten_only_lowers_and_keeps_reason() {
        let files = [f("src/a.rs", 650), f("src/b.rs", 900), f("src/c.rs", 500)];
        let base = [
            e("src/b.rs", 800),
            er("src/a.rs", 700, "理由"),
            e("src/c.rs", 700),
            e("src/d.rs", 700),
        ];
        let t = tightened(&files, &base);
        assert_eq!(t, vec![er("src/a.rs", 650, "理由"), e("src/b.rs", 800)]);
        let mut p = Vec::new();
        assert_eq!(parse_baseline(&render(&t), &mut p), t);
        assert!(p.is_empty());
    }

    /// 读不了的目录报错，不当成「没有文件」
    #[test]
    fn scan_reports_missing_src() {
        let d = std::env::temp_dir().join(format!("xtask-size-{}", std::process::id()));
        let _ = fs::create_dir_all(&d);
        assert!(scan(&d).is_err());
        let _ = fs::remove_dir_all(&d);
    }

    /// 真仓库的基线能解析，且每条都确实超线（作废条目由 tidy_passes 兜底）。
    #[test]
    fn repo_baseline_parses() {
        let root = crate::root();
        let text = fs::read_to_string(root.join(BASELINE_FILE)).expect("基线文件存在");
        let mut p = Vec::new();
        let b = parse_baseline(&text, &mut p);
        assert!(p.is_empty(), "{p:?}");
        assert!(b.iter().all(|e| e.max > MAX_LINES));
    }
}
