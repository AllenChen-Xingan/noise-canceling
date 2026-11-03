//! tidy 的「采样率只写一处」（R-RATE；CLAUDE.md 第五节第 5 条；issue #62）。
//!
//! | 规则 | 查什么 | 放行 |
//! |---|---|---|
//! | R-RATE | `src/**/*.rs` 的代码里（注释、字符串不算）不许出现 `16000` / `16_000` / `48000` / `48_000` 字面量（可带 `.0` 与类型后缀）；一律引用 `vdn_types::{IO_RATE, MODEL_RATE}` | 见下 |
//!
//! 放行清单（要窄，每条有理由）：
//! - [`RATE_FILE`]：正本本身
//! - [`LOW_LEVEL`] 里的底层 crate 的**测试代码**（`tests/`、`examples/` 目录，与源文件里 `#[cfg(test)]` 起的测试模块）：
//!   这些 crate 不依赖 `vdn-types`（shm、rt-thread 是 forbid 写死的），测试里造信号要写采样率。
//!   它们的非测试代码照样挡——要用采样率就改成参数传入（大总管定，2026-11-02）
//! - 单行：行尾注释写 `tidy: 非采样率：<理由>`（数值恰好相同、含义不是采样率时）
//!
//! 边界（写明，免得以为查全了）：只认这四种写法；`48 * 1000`、`3 * 16_000` 这类算出来的查不到；
//! 「测试模块」按 `#[cfg(test)]` 后紧跟 `mod` 的那一行起算到文件末尾（本仓约定测试模块放文件尾）。

use std::fs;
use std::path::Path;

use crate::single_source::lex_lines;
use crate::tidy::rs_files;

/// 采样率正本。
pub const RATE_FILE: &str = "src/types/src/rate.rs";
/// 不依赖 `vdn-types` 的底层 crate 目录（测试代码放行）。
pub const LOW_LEVEL: [&str; 5] = [
    "src/shm/",
    "src/rt-thread/",
    "src/hal/",
    "src/dsp/",
    "src/backend-ort/",
];
const ALLOW: &str = "tidy: 非采样率：";
const RATES: [&str; 4] = ["16000", "16_000", "48000", "48_000"];

fn ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// 一行代码里有没有采样率字面量（前后不能紧挨标识符字符或数字、小数点后的数字，免得把 `160000`、`x48000` 当成它）。
pub fn has_rate_literal(code: &str) -> bool {
    for lit in RATES {
        let mut from = 0;
        while let Some(off) = code[from..].find(lit) {
            let at = from + off;
            let end = at + lit.len();
            // 前面是 `.` 时只有区间 `..` 算数字开头（单个 `.` 是小数或字段访问）
            let ok_before = match code[..at].chars().next_back() {
                Some('.') => code[..at].ends_with(".."),
                Some(c) => !ident_char(c),
                None => true,
            };
            let rest = &code[end..];
            let mut tail = rest;
            // 允许 `.0` / `.0_f32` 之类的小数部分（只认全零）
            if let Some(t) = tail.strip_prefix('.') {
                let zeros = t.len() - t.trim_start_matches(['0', '_']).len();
                if zeros > 0 {
                    tail = &t[zeros..];
                }
            }
            // 允许类型后缀
            let suffix = tail.len() - tail.trim_start_matches(ident_char).len();
            let sfx = &tail[..suffix];
            let ok_suffix = sfx.is_empty()
                || matches!(
                    sfx.trim_start_matches('_'),
                    "u32" | "u64" | "usize" | "i32" | "i64" | "isize" | "f32" | "f64"
                );
            // 后面紧跟 `..`（区间）可以；单个 `.` 接非零小数不算
            let after = &tail[suffix..];
            let ok_after = after.starts_with("..")
                || !after.starts_with(|c: char| c.is_ascii_digit() || c == '.');
            if ok_before && ok_suffix && ok_after {
                return true;
            }
            from = end;
        }
    }
    false
}

/// 这个文件是不是底层 crate 的测试目录 / 示例目录。
fn low_level_test_file(rel: &str) -> bool {
    LOW_LEVEL.iter().any(|c| {
        rel.strip_prefix(c)
            .is_some_and(|r| r.starts_with("tests/") || r.starts_with("examples/"))
    })
}

/// 一个文件的问题。
pub fn rate_problems(rel: &str, src: &str) -> Vec<String> {
    if rel == RATE_FILE || low_level_test_file(rel) {
        return Vec::new();
    }
    let low_level = LOW_LEVEL.iter().any(|c| rel.starts_with(c));
    let raw: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    let mut in_tests = false;
    let mut cfg_test_prev = false;
    for (i, (code, comment)) in lex_lines(src).iter().enumerate() {
        let t = code.trim();
        if cfg_test_prev && (t.starts_with("mod ") || t.starts_with("pub mod ")) {
            in_tests = true;
        }
        if !t.is_empty() {
            cfg_test_prev = t == "#[cfg(test)]";
        }
        if low_level && in_tests {
            continue;
        }
        if has_rate_literal(code) && !comment.contains(ALLOW) {
            let line = raw.get(i).map_or("", |l| l.trim());
            out.push(format!(
                "{rel}:{}：采样率字面量（R-RATE）——改用 vdn_types::IO_RATE / MODEL_RATE（正本 {RATE_FILE}）；不是采样率就在行尾写 `// {ALLOW}<理由>`：{line}",
                i + 1
            ));
        }
    }
    out
}

pub fn check(root: &Path, problems: &mut Vec<String>) {
    if !root.join(RATE_FILE).exists() {
        problems.push(format!("{RATE_FILE} 不存在：采样率正本（R-RATE）"));
    }
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    for f in all {
        let Ok(rel) = f.strip_prefix(root).map(|p| p.display().to_string()) else {
            continue;
        };
        match fs::read_to_string(&f) {
            Ok(src) => problems.extend(rate_problems(&rel, &src)),
            Err(e) => problems.push(format!("读 {rel}：{e}（R-RATE）")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_forms_are_caught() {
        for code in [
            "let r = 48_000;",
            "let r = 16000u32;",
            "f(48000.0)",
            "x * 16_000.0_f32",
            "const SR: usize = 16_000usize;",
            "(a, 48_000)",
            "for i in 0..16000 {",
            "x[..48_000 - l]",
            "(0..=48_000)",
            "48_000..96_000",
        ] {
            assert!(has_rate_literal(code), "{code}");
        }
    }

    #[test]
    fn other_numbers_pass() {
        for code in [
            "let n = 160000;",
            "let n = 48_0000;",
            "let x = 1.48000;",
            "let id = v48000;",
            "let r = IO_RATE;",
            "let n = 4800;",
            "let x = 48000.5;",
            "let x = a.48000;",
        ] {
            assert!(!has_rate_literal(code), "{code}");
        }
    }

    #[test]
    fn comments_and_strings_do_not_count() {
        let src = "// 48_000 Hz\nlet s = \"16000\";\n/* 48000 */\n";
        assert!(rate_problems("src/eval/src/a.rs", src).is_empty());
    }

    #[test]
    fn allow_marker_needs_reason_text() {
        let ok = "let n = 48_000; // tidy: 非采样率：一分钟的毫秒数之类\n";
        assert!(rate_problems("src/eval/src/a.rs", ok).is_empty());
        let bad = "let n = 48_000; // 非采样率\n";
        assert_eq!(rate_problems("src/eval/src/a.rs", bad).len(), 1);
    }

    #[test]
    fn rate_file_itself_is_allowed() {
        assert!(rate_problems(RATE_FILE, "pub const IO_RATE: u32 = 48_000;\n").is_empty());
    }

    /// 底层 crate：测试代码放行，非测试代码照挡；别的 crate 测试也挡
    #[test]
    fn low_level_tests_only() {
        let src = "fn f() -> u32 { 48_000 }\n#[cfg(test)]\nmod tests {\n    fn g() { let _ = 48_000; }\n}\n";
        let p = rate_problems("src/shm/src/wait.rs", src);
        assert_eq!(p.len(), 1, "{p:?}");
        assert!(p[0].contains(":1："));
        assert!(rate_problems("src/shm/tests/ring.rs", "let r = 48_000;\n").is_empty());
        assert!(rate_problems("src/dsp/examples/x.rs", "let r = 48_000;\n").is_empty());
        assert_eq!(rate_problems("src/eval/src/a.rs", src).len(), 2);
        assert_eq!(
            rate_problems("src/eval/tests/a.rs", "let r = 48_000;\n").len(),
            1
        );
    }
}
