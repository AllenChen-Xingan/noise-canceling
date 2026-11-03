//! tidy 的「采样率只写一处」（R-RATE；CLAUDE.md 第五节第 5 条；issue #62）。
//!
//! | 规则 | 查什么 | 放行 |
//! |---|---|---|
//! | R-RATE | `src/**/*.rs` 的代码里（注释、字符串不算）不许出现 `16000` / `16_000` / `48000` / `48_000` 字面量（可带 `.0` 与类型后缀）；**测试与示例也一样**，一律引用 `vdn_types::{IO_RATE, MODEL_RATE}` | 见下 |
//!
//! 放行只有两种：
//! - [`RATE_FILE`]：正本本身
//! - 单行：行尾注释写 `tidy: 非采样率：<理由>`（数值恰好相同、含义不是采样率时；理由不能空）
//!
//! 为什么测试代码也查（大总管 8d 定，2026-11-02）：要放行测试代码就得用文本猜「哪段是测试」，
//! codex 连续几轮挑出绕过点（同一行闭合、`mod tests;` 声明等）。测试改用常量没有坏处；不依赖 `vdn-types` 的
//! 底层 crate（shm、rt-thread、dsp）把它加成 dev 依赖即可（forbid 只管普通依赖闭包，shm 已有先例）。
//!
//! 边界（写明，免得以为查全了）：只认这四种写法；`48 * 1000`、`3 * 16_000` 这类算出来的查不到。

use std::fs;
use std::path::Path;

use crate::scan::lex_lines;
use crate::tidy::rs_files;

/// 采样率正本。
pub const RATE_FILE: &str = "src/types/src/rate.rs";
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
                    "u16"
                        | "u32"
                        | "u64"
                        | "u128"
                        | "usize"
                        | "i16"
                        | "i32"
                        | "i64"
                        | "i128"
                        | "isize"
                        | "f32"
                        | "f64"
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

/// 放行标记后面有没有写理由。
fn has_allow_reason(comment: &str) -> bool {
    comment
        .split_once(ALLOW)
        .is_some_and(|(_, r)| !r.trim().is_empty())
}

/// 一个文件的问题。
pub fn rate_problems(rel: &str, src: &str) -> Vec<String> {
    if rel == RATE_FILE {
        return Vec::new();
    }
    let raw: Vec<&str> = src.lines().collect();
    let mut out = Vec::new();
    for (i, (code, comment)) in lex_lines(src).iter().enumerate() {
        if has_rate_literal(code) && !has_allow_reason(comment) {
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
    for f in &all {
        let Ok(rel) = f.strip_prefix(root).map(|p| p.display().to_string()) else {
            continue;
        };
        match fs::read_to_string(f) {
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
            // 合法整数后缀都认（codex 第 8 轮 P2 #3）
            "48000u16",
            "16000i16",
            "48_000u128",
            "16_000_i128",
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

    /// 负例（会红）：测试代码、示例、底层 crate、内联测试模块里的字面量一律照挡（不再识别测试区）
    #[test]
    fn test_code_is_checked_too() {
        let src = "fn f() -> u32 { 48_000 }\n#[cfg(test)]\nmod tests {\n    fn g() { let _ = 48_000; }\n}\n";
        assert_eq!(rate_problems("src/shm/src/wait.rs", src).len(), 2);
        for rel in [
            "src/shm/tests/ring.rs",
            "src/dsp/examples/x.rs",
            "src/eval/tests/a.rs",
            "src/shm/src/tests.rs",
        ] {
            assert_eq!(rate_problems(rel, "let r = 48_000;\n").len(), 1, "{rel}");
        }
    }

    /// 负例（codex 第 2 轮 P2 #5）：放行标记后面没写理由不算放行
    #[test]
    fn empty_allow_reason_is_flagged() {
        let src = "let n = 48_000; // tidy: 非采样率：\nlet m = 48_000; // tidy: 非采样率：   \n";
        assert_eq!(rate_problems("src/eval/src/a.rs", src).len(), 2);
    }
}
