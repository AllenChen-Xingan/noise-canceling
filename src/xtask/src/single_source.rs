//! tidy 的「唯一真相源」两条规则（CLAUDE.md 第五节第 5 条；issue #56）。纯文本检查，不依赖任何 vdn crate。
//!
//! | 规则 | 查什么 | 放行写法 |
//! |---|---|---|
//! | R-MIRROR | ① `src/` 下不许有名叫 `mirror.rs` 的文件；② `vdn-types` 之外的代码不许定义与 `vdn-types` 公开类型**同名**的 `struct` / `enum` / `trait` / `type`（镜像副本的特征） | 定义行或紧邻上方的注释里写 `tidy: 同名不同物：<理由>` |
//! | R-PARAM | 参数描述表（`src/types/src/desc/` 目录）之外，一行代码里既出现某个参数名（键名末段，如 `floor_db`，或它的驼峰形 `FloorDb`），又出现数字字面量写的范围：`a..b` / `a..=b` · `clamp(a, b)` · `.max(a).min(b)` / `.min(a).max(b)` · `=> (a, b)` | 行尾写 `// tidy: 非参数范围：<理由>` |
//!
//! 能做到的严格度与边界（写明，免得以为查全了）：
//! - 只按**行**查：被 rustfmt 拆成多行的 `x.clamp(\n -60.0,\n 0.0)`、或范围先赋给一个名字不带参数名的变量再用，查不到
//! - 参数名按描述表的键名末段认（`polish.floor_db` 认 `floor_db`），与参数同名但含义不同的标识符也会被查（宁可多报，用放行写法说明）
//! - 默认值的字面量（如 `floor_db: -12.0`）不查：测试里造输入也要写字面量，文本上分不出来；默认值同源由各 crate 的单测比对描述表
//! - 同名类型只比名字不比字段；改名后的手抄副本查不到

use std::fs;
use std::path::Path;

use crate::lex::{code_lines, lex_lines, strip_comments_keep_strings};
use crate::tidy::rs_files;

/// 描述表正本所在目录（相对 workspace 根）：`mod.rs` 加按参数所属分的几个文件。
pub const DESC_DIR: &str = "src/types/src/desc/";
const TYPES_SRC: &str = "src/types/src";
const MIRROR_ALLOW: &str = "tidy: 同名不同物：";
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

    let mut type_files = Vec::new();
    rs_files(&root.join(TYPES_SRC), &mut type_files);
    let type_srcs: Vec<String> = type_files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect();
    let names = types_names(type_srcs.iter().map(String::as_str));

    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    for f in all {
        let rel = f
            .strip_prefix(root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| f.display().to_string());
        let Ok(src) = fs::read_to_string(&f) else {
            continue;
        };
        if !rel.starts_with(TYPES_SRC) {
            problems.extend(mirror_problems(&rel, &src, &names));
        }
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

/// `vdn-types` 源码里定义的公开类型名（`pub struct` / `pub enum` / `pub trait`）。
pub fn types_names<'a>(srcs: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for src in srcs {
        for line in src.lines() {
            let t = line.trim_start();
            for kw in ["pub struct ", "pub enum ", "pub trait "] {
                if let Some(r) = t.strip_prefix(kw) {
                    let name = ident(r);
                    if !name.is_empty() && !out.iter().any(|n| n == name) {
                        out.push(name.to_string());
                    }
                }
            }
        }
    }
    out
}

fn ident(s: &str) -> &str {
    let n = s
        .char_indices()
        .find(|&(_, c)| !(c.is_alphanumeric() || c == '_'))
        .map_or(s.len(), |(i, _)| i);
    &s[..n]
}

/// 去掉可见性前缀后的类型定义名：`struct X` / `enum X` / `trait X` / `type X`。
fn defined_type(line: &str) -> Option<&str> {
    let mut t = line.trim_start();
    if let Some(r) = t.strip_prefix("pub") {
        let r = r.trim_start();
        t = if r.starts_with('(') {
            r.split_once(')').map_or(r, |(_, x)| x).trim_start()
        } else {
            r
        };
    }
    for kw in ["struct ", "enum ", "trait ", "type "] {
        if let Some(r) = t.strip_prefix(kw) {
            let name = ident(r.trim_start());
            return (!name.is_empty()).then_some(name);
        }
    }
    None
}


fn has_reason(line: &str, marker: &str) -> bool {
    line.split_once(marker)
        .is_some_and(|(_, why)| !why.trim().is_empty())
}

pub fn mirror_problems(rel: &str, src: &str, names: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    if Path::new(rel).file_name().is_some_and(|n| n == "mirror.rs") {
        out.push(format!(
            "{rel}：镜像文件不许存在，直接依赖 vdn-types（R-MIRROR）"
        ));
    }
    let lexed = lex_lines(src);
    for (i, (code, comment)) in lexed.iter().enumerate() {
        let Some(name) = defined_type(code) else {
            continue;
        };
        if !names.iter().any(|n| n == name) {
            continue;
        }
        // 放行：同行，或紧邻上方连续的注释 / 属性行的**注释**里写了理由
        let mut ok = has_reason(comment, MIRROR_ALLOW);
        let mut j = i;
        while !ok && j > 0 {
            j -= 1;
            let (c, cm) = &lexed[j];
            let t = c.trim();
            let comment_only = t.is_empty() && !cm.trim().is_empty();
            if !(comment_only || t.starts_with("#[")) {
                break;
            }
            ok = has_reason(cm, MIRROR_ALLOW);
        }
        if !ok {
            out.push(format!(
                "{rel}:{}：定义了与 vdn-types 同名的类型 `{name}`，像镜像副本；改用 vdn_types::{name}，或注释写「{MIRROR_ALLOW}<理由>」（R-MIRROR）",
                i + 1
            ));
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
#[path = "single_source_tests.rs"]
mod tests;
