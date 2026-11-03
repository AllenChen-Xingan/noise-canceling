//! tidy 规则 R-PARAM：参数范围只在参数描述表里写（CLAUDE.md 第五节第 5 条「唯一真相源」；issue #56）。纯文本检查，不依赖任何 vdn crate。
//!
//! | 查什么 | 放行写法 |
//! |---|---|
//! | 参数描述表（`src/types/src/desc/` 目录）之外，一行代码里既出现某个参数名（键名末段，如 `floor_db`，或它的驼峰形 `FloorDb`），又出现数字字面量写的范围：`a..b` / `a..=b` · `clamp(a, b)` · `.max(a).min(b)` / `.min(a).max(b)` · `=> (a, b)` | 行尾写 `// tidy: 非参数范围：<理由>` |
//!
//! 能做到的严格度与边界（写明，免得以为查全了）：
//! - 只按**行**查：被 rustfmt 拆成多行的 `x.clamp(\n -60.0,\n 0.0)`、或范围先赋给一个名字不带参数名的变量再用，查不到
//! - 参数名按描述表的键名末段认（`polish.floor_db` 认 `floor_db`），与参数同名但含义不同的标识符也会被查（宁可多报，用放行写法说明）
//! - 默认值的字面量（如 `floor_db: -12.0`）不查：测试里造输入也要写字面量，文本上分不出来；默认值同源由各 crate 的单测比对描述表

use std::fs;
use std::path::Path;

use crate::scan::{code_lines, has_reason, lex_lines, src_files, strip_comments_keep_strings};
use crate::tidy::rs_files;

/// 描述表正本所在目录（相对 workspace 根）：`mod.rs` 加按参数所属分的几个文件。
pub const DESC_DIR: &str = "src/types/src/desc/";
const RANGE_ALLOW: &str = "tidy: 非参数范围：";

pub fn check(root: &Path, problems: &mut Vec<String>) {
    let Some(desc) = read_desc(root) else {
        problems.push(format!(
            "{DESC_DIR} 不存在或没有 .rs：参数描述表是参数范围的正本（R-PARAM）"
        ));
        return;
    };
    let keys = desc_keys(&desc);
    let consts = desc_const_count(&desc);
    if keys.is_empty() || keys.len() != consts {
        problems.push(format!(
            "{DESC_DIR}：解析出 {} 个键名，但有 {consts} 个 ParamDesc 常量；解析失败的条目会让 R-PARAM 漏查（R-PARAM）",
            keys.len()
        ));
    }
    let tokens = param_tokens(&keys);
    for (rel, src) in src_files(root) {
        if !rel.starts_with(DESC_DIR) {
            problems.extend(range_problems(&rel, &src, &tokens));
        }
    }
}

/// 描述表目录下全部 `.rs` 按文件名排序拼起来；目录不存在或一个文件都没有 → `None`。
pub fn read_desc(root: &Path) -> Option<String> {
    let mut files = Vec::new();
    rs_files(&root.join(DESC_DIR), &mut files);
    files.sort();
    let srcs: Vec<String> = files.iter().filter_map(|p| fs::read_to_string(p).ok()).collect();
    (!srcs.is_empty() && srcs.len() == files.len()).then(|| srcs.join("\n"))
}

/// 描述表里每条的键名：`p(<编号>, "<键名>", …` 的第二个参数（容许 rustfmt 把参数拆到多行）。
pub fn desc_keys(src: &str) -> Vec<String> {
    // 先剥掉注释（参数之间夹注释也能解析，codex 第 3 轮 P2 #3）；字符串原样保留，键名在里面
    let stripped: String = strip_comments_keep_strings(src).join("\n");
    let mut out = Vec::new();
    let mut rest = stripped.as_str();
    while let Some(i) = rest.find("p(") {
        let before_ok = i == 0
            || !rest[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_');
        let after = &rest[i + 2..];
        rest = after;
        if !before_ok {
            continue;
        }
        let a = after.trim_start();
        let digits = a.chars().take_while(char::is_ascii_digit).count();
        if digits == 0 {
            continue;
        }
        let a = a[digits..].trim_start();
        let Some(a) = a.strip_prefix(',') else {
            continue;
        };
        let Some(a) = a.trim_start().strip_prefix('"') else {
            continue;
        };
        if let Some(end) = a.find('"') {
            out.push(a[..end].to_string());
        }
    }
    out
}

/// 描述表里 `ParamDesc` 常量的个数（`pub const X: ParamDesc`），用来核对 [`desc_keys`] 一条不漏。
pub fn desc_const_count(src: &str) -> usize {
    code_lines(src)
        .iter()
        .filter(|l| {
            l.trim_start()
                .strip_prefix("pub const ")
                .and_then(|r| r.split_once(':'))
                .is_some_and(|(_, ty)| ty.trim_start().starts_with("ParamDesc"))
        })
        .count()
}


/// 参数名在代码里的写法：键名末段（`floor_db`）与驼峰形（`FloorDb`，`ParamField` 的变体名）。
pub fn param_tokens(keys: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for k in keys {
        let short = k.rsplit('.').next().unwrap_or(k).to_string();
        let camel: String = short
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                    .unwrap_or_default()
            })
            .collect();
        for t in [short, camel] {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    out
}

/// `s[i..]` 开头是不是数字字面量（可带负号、小数点、f32/f64 后缀）；是就返回结束位置。
fn num_at(s: &str, i: usize) -> Option<usize> {
    let b = s.as_bytes();
    let mut j = i;
    if b.get(j) == Some(&b'-') {
        j += 1;
    }
    let start = j;
    while b.get(j).is_some_and(|c| c.is_ascii_digit() || *c == b'_') {
        j += 1;
    }
    if j == start || !b[start].is_ascii_digit() {
        return None;
    }
    if b.get(j) == Some(&b'.') && b.get(j + 1).is_some_and(u8::is_ascii_digit) {
        j += 1;
        while b.get(j).is_some_and(|c| c.is_ascii_digit() || *c == b'_') {
            j += 1;
        }
    } else if b.get(j) == Some(&b'.')
        && !b
            .get(j + 1)
            .is_some_and(|c| *c == b'.' || c.is_ascii_alphabetic() || *c == b'_')
    {
        // `60.` 这种无小数位的浮点（后面不是 `..` 范围、也不是方法调用）
        j += 1;
    }
    // 指数：1e3 / 6e-1 / 6E+1（codex 第 1 轮 P2 #5）
    if matches!(b.get(j), Some(b'e') | Some(b'E')) {
        let mut k = j + 1;
        if matches!(b.get(k), Some(b'+') | Some(b'-')) {
            k += 1;
        }
        if b.get(k).is_some_and(u8::is_ascii_digit) {
            j = k;
            while b.get(j).is_some_and(|c| c.is_ascii_digit() || *c == b'_') {
                j += 1;
            }
        }
    }
    if let Some(suf) = NUM_SUFFIXES.iter().find(|suf| s[j..].starts_with(**suf)) {
        j += suf.len();
    }
    Some(j)
}

/// 数字字面量的类型后缀（长的在前，免得 `u128` 先匹配成 `u1`）。
const NUM_SUFFIXES: [&str; 14] = [
    "u128", "i128", "usize", "isize", "f32", "f64", "u16", "u32", "u64", "i16", "i32", "i64", "u8",
    "i8",
];

/// `s[..end]` 结尾是不是数字字面量（前面不是标识符字符，免得把 `x.0..` 的元组下标当数字）。
fn num_ends_at(s: &str, end: usize) -> bool {
    let b = s.as_bytes();
    let mut j = end;
    if let Some(suf) = NUM_SUFFIXES.iter().find(|suf| s[..j].ends_with(**suf)) {
        j -= suf.len();
    }
    let stop = j;
    while j > 0 && (b[j - 1].is_ascii_digit() || b[j - 1] == b'.' || b[j - 1] == b'_') {
        j -= 1;
    }
    // 指数：…e3 / …e-1，再往前接着走尾数
    let mut k = j;
    if k > 0 && matches!(b[k - 1], b'+' | b'-') {
        k -= 1;
    }
    if j < stop && k > 1 && matches!(b[k - 1], b'e' | b'E') && b[k - 2].is_ascii_digit() {
        j = k - 1;
        while j > 0 && (b[j - 1].is_ascii_digit() || b[j - 1] == b'.' || b[j - 1] == b'_') {
            j -= 1;
        }
    }
    if j == stop || !b[j].is_ascii_digit() {
        return false;
    }
    !(j > 0 && (b[j - 1].is_ascii_alphanumeric() || b[j - 1] == b'_'))
}

fn skip_ws(s: &str, i: usize) -> usize {
    i + s[i..].len() - s[i..].trim_start().len()
}

/// `s[i..]` 是不是「数字, 数字)」。
fn two_nums_then_close(s: &str, i: usize) -> bool {
    let i = skip_ws(s, i);
    let Some(e) = num_at(s, i) else { return false };
    let e = skip_ws(s, e);
    if !s[e..].starts_with(',') {
        return false;
    }
    let i = skip_ws(s, e + 1);
    let Some(e) = num_at(s, i) else { return false };
    let mut e = skip_ws(s, e);
    // 可选尾逗号：`clamp(-60.0, 0.0,)`
    if s[e..].starts_with(',') {
        e = skip_ws(s, e + 1);
    }
    s[e..].starts_with(')')
}

/// 一行代码里有没有数字字面量写的范围。
pub fn has_literal_range(code: &str) -> bool {
    // a..b / a..=b
    let mut from = 0;
    while let Some(k) = code[from..].find("..") {
        let at = from + k;
        from = at + 2;
        if code[at..].starts_with("...") || (at > 0 && code.as_bytes()[at - 1] == b'.') {
            continue;
        }
        let mut r = at + 2;
        if code[r..].starts_with('=') {
            r += 1;
        }
        let r = skip_ws(code, r);
        let l = code[..at].trim_end().len();
        if num_ends_at(code, l) && num_at(code, r).is_some() {
            return true;
        }
    }
    // clamp(a, b)
    let mut from = 0;
    while let Some(k) = code[from..].find("clamp(") {
        let at = from + k + "clamp(".len();
        from = at;
        if two_nums_then_close(code, at) {
            return true;
        }
    }
    // .max(a).min(b) / .min(a).max(b)
    for (a, b) in [(".max(", ".min("), (".min(", ".max(")] {
        let mut from = 0;
        while let Some(k) = code[from..].find(a) {
            let at = from + k + a.len();
            from = at;
            let i = skip_ws(code, at);
            let Some(e) = num_at(code, i) else { continue };
            let e = skip_ws(code, e);
            if !code[e..].starts_with(')') {
                continue;
            }
            if let Some(rest) = code[e + 1..].trim_start().strip_prefix(b) {
                let i = skip_ws(rest, 0);
                if num_at(rest, i).is_some() {
                    return true;
                }
            }
        }
    }
    // => (a, b)
    let mut from = 0;
    while let Some(k) = code[from..].find("=>") {
        let at = from + k + 2;
        from = at;
        let i = skip_ws(code, at);
        if code[i..].starts_with('(') && two_nums_then_close(code, i + 1) {
            return true;
        }
    }
    false
}

fn has_token(code: &str, tok: &str) -> bool {
    let b = code.as_bytes();
    let mut from = 0;
    while let Some(k) = code[from..].find(tok) {
        let at = from + k;
        let end = at + tok.len();
        from = end;
        let left_ok = at == 0 || !(b[at - 1].is_ascii_alphanumeric() || b[at - 1] == b'_');
        let right_ok = end >= b.len() || !(b[end].is_ascii_alphanumeric() || b[end] == b'_');
        if left_ok && right_ok {
            return true;
        }
    }
    false
}

pub fn range_problems(rel: &str, src: &str, tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, (code, comment)) in lex_lines(src).iter().enumerate() {
        if has_reason(comment, RANGE_ALLOW) {
            continue;
        }
        if !has_literal_range(code) {
            continue;
        }
        if let Some(tok) = tokens.iter().find(|t| has_token(code, t)) {
            out.push(format!(
                "{rel}:{}：参数 `{tok}` 的范围写成了数字字面量；范围的正本是 {DESC_DIR}，改用 vdn_types::desc 的 check / clamp / min / max（R-PARAM）",
                i + 1
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toks() -> Vec<String> {
        param_tokens(&[
            "floor_db".into(),
            "polish.smooth".into(),
            "gate.confirm_n".into(),
        ])
    }

    #[test]
    fn desc_keys_are_parsed_even_when_split_by_rustfmt() {
        let src = "pub const A: ParamDesc = p(1, \"floor_db\", Unit::Db, -60.0, 0.0, -12.0, \"x\");\n\
                   pub const B: ParamDesc = p(\n    202,\n    \"polish.floor_db\",\n    Unit::Db,\n);\n\
                   const fn p(id: u16, key: &'static str) {}\n";
        assert_eq!(desc_keys(src), ["floor_db", "polish.floor_db"]);
    }

    #[test]
    fn real_desc_file_parses() {
        let src = read_desc(&crate::root()).unwrap();
        let keys = desc_keys(&src);
        assert_eq!(keys.len(), desc_const_count(&src), "{keys:?}");
        assert!(keys.iter().any(|k| k == "floor_db"));
    }

    /// 条目里夹注释也能解析；解析不出时条数对不上（`check` 据此报错）。
    #[test]
    fn desc_keys_survive_comments_and_count_mismatch_is_detected() {
        let src = "pub const A: ParamDesc = p(106, /* 编号说明 */ \"gate.confirm_n\", Unit::Count);\n\
                   pub const B: ParamDesc = p(\n    2, // 衰减下限\n    \"floor_db\",\n);\n";
        assert_eq!(desc_keys(src), ["gate.confirm_n", "floor_db"]);
        assert_eq!(desc_const_count(src), 2);
        let broken = "pub const A: ParamDesc = make(\"x\");\n";
        assert_ne!(desc_keys(broken).len(), desc_const_count(broken));
    }

    #[test]
    fn tokens_include_short_and_camel_forms() {
        let t = toks();
        for want in [
            "floor_db",
            "FloorDb",
            "smooth",
            "Smooth",
            "confirm_n",
            "ConfirmN",
        ] {
            assert!(t.iter().any(|x| x == want), "{want}");
        }
    }

    // ── R-PARAM 负例（该报）──
    #[test]
    fn literal_param_ranges_are_flagged() {
        let bad = [
            "        let ok = (-30.0..=0.0).contains(&p.floor_db);",
            "    if !(-60.0..0.0).contains(&floor_db) {",
            "    self.smooth = s.smooth.clamp(0.0, 1.0);",
            "    let f = floor_db.max(-60.0).min(0.0);",
            "    let n = confirm_n.min(20).max(1);",
            "            ParamField::FloorDb => (-60.0, 0.0),",
            "    let f = floor_db.clamp(-60f32, 0f32);",
            "    let f = floor_db.clamp(-6e1, 0.0);",
            "    let ok = (-6E+1..=0e0).contains(&floor_db);",
            "    let f = /* 夹一下 */ floor_db.clamp(-60.0, 0.0);",
            "    let n = confirm_n.clamp(1u32, 20u32);",
            "    let n = (1usize..=20usize).contains(&confirm_n);",
            "    let f = floor_db.clamp(-60., 0.);",
            // 字符串里写放行标记不算放行
            "    let s = \"tidy: 非参数范围：x\"; let f = floor_db.clamp(-60.0, 0.0);",
            // 尾逗号
            "    let f = floor_db.clamp(-60.0, 0.0,);",
            "            ParamField::FloorDb => (-60.0, 0.0,),",
            // 字节原始字符串里的放行标记不算放行
            "    let s = br#\"tidy: 非参数范围：x\"#; let f = floor_db.clamp(-60.0, 0.0);",
        ];
        for line in bad {
            assert_eq!(range_problems("x.rs", line, &toks()).len(), 1, "{line}");
        }
    }

    // ── R-PARAM 正例（不该报）──
    #[test]
    fn non_param_ranges_and_values_pass() {
        let good = [
            // 范围来自描述表
            "    let f = FLOOR_DB.clamp(p.floor_db);",
            // 不是参数的范围
            "    } else if (1500.0..2000.0).contains(&f) {",
            "    let a = alpha.clamp(0.0, 1.0);",
            // 参数的单个值（默认值 / 测试输入），不是范围
            "        floor_db: -12.0,",
            "    assert_eq!((c.floor_db, c.detect_db), (-20.0, 12.0));",
            // 注释里的范围
            "    // floor_db 原来写成 -60.0..=0.0，现在在 desc.rs",
            // 元组下标 / 切片 / 整数方法调用
            "    let s = &floor_db_buf[x.0..n];",
            "    for i in 0..floor_db.len() {",
            "    let n = 3.max(confirm_n);",
            // 放行
            "    let r = (0.0..1.0).contains(&smooth); // tidy: 非参数范围：这是归一化后的插值位置",
            // 字符串里的负例（测试数据）不是代码
            "        \"    let ok = (-30.0..=0.0).contains(&p.floor_db);\",",
            "    let s = r#\"floor_db.clamp(-60.0, 0.0)\"#;",
            "    let s = br#\"\" floor_db.clamp(-60.0, 0.0) \"#;",
            "    let s = br\"floor_db.clamp(-60.0, 0.0)\";",
            // 块注释里的
            "    /* floor_db.clamp(-60.0, 0.0) */ let x = 1;",
            // 生命周期与字符字面量不打乱剥离
            "    fn f<'a>(x: &'a str) -> char { let _ = floor_db; '\"' }",
            "    let q = ['\\'', '\\\\', '\"']; let s = \"floor_db.clamp(-60.0, 0.0)\";",
        ];
        for line in good {
            assert!(range_problems("x.rs", line, &toks()).is_empty(), "{line}");
        }
    }

    #[test]
    fn code_lines_strip_strings_and_comments_across_lines() {
        let src = "let a = \"x\n\\\n  floor_db.clamp(-60.0, 0.0)\";\n/* 开始\nfloor_db.clamp(-60.0, 0.0)\n结束 */ let b = 1;\nlet c = r#\"\na \"q\" floor_db\n\"#;\n";
        let c = code_lines(src);
        assert!(c.iter().all(|l| !l.contains("floor_db")), "{c:#?}");
        assert!(c.iter().any(|l| l.contains("let b = 1;")));
        assert!(c.iter().any(|l| l.contains("let c = \"")));
        assert!(range_problems("x.rs", src, &toks()).is_empty());
    }

    /// 整个仓库当前过这条规则（新加的违规会在这里和 `tidy` 里同时红）。
    #[test]
    fn repo_passes() {
        let mut p = Vec::new();
        check(&crate::root(), &mut p);
        assert!(p.is_empty(), "{p:#?}");
    }
}
