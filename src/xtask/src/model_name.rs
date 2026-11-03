//! tidy 规则 R-MODEL：有过不止一种写法的模型名，只许在正本里写（CLAUDE.md 第五节第 5 条；issue #65）。纯文本检查，不依赖任何 vdn crate。
//!
//! 正本是 `src/types/src/params.rs` 的 `ModelId`：`as_str` 给正本名，`LEGACY` 给旧写法。本规则从那个文件读出
//! 「有旧写法的模型」的全部写法（正本名 + 旧写法），再查 `src/` 下其他 `.rs` 的字符串字面量里有没有写死它们。
//! 别处要用模型名，从 `ModelId::as_str` / `ModelId::NAMES` / `ModelId::parse` 取。
//!
//! | 查什么 | 不查 | 放行写法 |
//! |---|---|---|
//! | 字符串字面量里出现上述写法（前后不是字母、数字、`_`、`-`） | 正本文件本身；`tests/` 目录；`#[cfg(test)]` 管的那一项（单测要拿字面量造输入；它后面的代码照查）；注释 | 同一行注释写 `tidy: 不是模型名：<理由>`（如权重文件名） |
//!
//! 只查有过旧写法的模型：其他模型名（如 `gtcrn`）也是路径的一段，逐个查噪声太大；一旦给它登记旧写法，就自动进来。

use std::fs;
use std::path::Path;

use crate::scan::{code_lines, has_reason, lex_lines, src_files, strip_comments_keep_strings};

/// 模型名的正本文件（相对 workspace 根）。
pub const NAMES_FILE: &str = "src/types/src/params.rs";
const ALLOW: &str = "tidy: 不是模型名：";

pub fn check(root: &Path, problems: &mut Vec<String>) {
    let Ok(src) = fs::read_to_string(root.join(NAMES_FILE)) else {
        problems.push(format!("{NAMES_FILE} 不存在：模型名的正本（R-MODEL）"));
        return;
    };
    let names = match watched_names(&src) {
        Ok(n) => n,
        Err(e) => {
            problems.push(format!(
                "{NAMES_FILE}：{e}；解析失败会让 R-MODEL 漏查（R-MODEL）"
            ));
            return;
        }
    };
    for (rel, s) in src_files(root) {
        if rel != NAMES_FILE && !rel.contains("/tests/") {
            problems.extend(file_problems(&rel, &s, &names));
        }
    }
}

/// 一行里所有字符串字面量的内容（不含引号）。注释已先剥掉。
fn strings_in(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Option<String> = None;
    let mut esc = false;
    for ch in line.chars() {
        match cur.as_mut() {
            None if ch == '"' => cur = Some(String::new()),
            None => {}
            Some(s) => {
                if esc {
                    esc = false;
                    s.push(ch);
                } else if ch == '\\' {
                    esc = true;
                } else if ch == '"' {
                    out.push(cur.take().unwrap_or_default());
                } else {
                    s.push(ch);
                }
            }
        }
    }
    out
}

/// 正本文件里「有旧写法的模型」的全部写法：`LEGACY` 里的旧写法，加上同一变体在 `as_str` 里的正本名。
/// 容许 rustfmt 换行、`ModelId::X` 或 `Self::X` 两种写法。读不出旧写法、或某个旧写法找不到正本名 → `Err`
///（解析失败不能静默漏查，codex 第 1 轮 #3）。
pub fn watched_names(src: &str) -> Result<Vec<String>, String> {
    let flat = strip_comments_keep_strings(src).join(" ");
    let legacy_block = flat
        .split_once("const LEGACY")
        .and_then(|(_, r)| r.split_once('='))
        .and_then(|(_, r)| r.split_once("];"))
        .map(|(b, _)| b)
        .ok_or("找不到 ModelId::LEGACY")?;
    let as_str = flat
        .split_once("fn as_str")
        .map(|(_, r)| r.split_once("fn ").map_or(r, |(b, _)| b))
        .ok_or("找不到 ModelId::as_str")?;
    let variant_after = |s: &str| -> Option<String> {
        let s = s.trim_start();
        let rest = s
            .strip_prefix("ModelId::")
            .or_else(|| s.strip_prefix("Self::"))?;
        let v: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        (!v.is_empty()).then_some(v)
    };
    let mut out = Vec::new();
    for tuple in legacy_block.split('(').skip(1) {
        let Some((old, rest)) = tuple.split_once(',') else {
            continue;
        };
        let Some(old) = strings_in(old).into_iter().next() else {
            continue;
        };
        let variant = variant_after(rest).ok_or(format!("旧写法 `{old}` 后面不是模型变体"))?;
        let canon = ["ModelId::", "Self::"]
            .iter()
            .find_map(|p| {
                let arm = format!("{p}{variant}");
                as_str.match_indices(&arm).find_map(|(at, _)| {
                    let after = &as_str[at + arm.len()..];
                    let after = after.trim_start().strip_prefix("=>")?;
                    strings_in(after).into_iter().next()
                })
            })
            .ok_or(format!(
                "旧写法 `{old}` 的模型 {variant} 在 as_str 里找不到正本名"
            ))?;
        out.push(old);
        out.push(canon);
    }
    if out.is_empty() {
        return Err("ModelId::LEGACY 里读不出任何旧写法".into());
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn bounded_in(hay: &str, needle: &str) -> bool {
    let is_name = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    hay.match_indices(needle).any(|(i, _)| {
        let before = hay[..i].chars().next_back();
        let after = hay[i + needle.len()..].chars().next();
        !before.is_some_and(is_name) && !after.is_some_and(is_name)
    })
}

/// 正在跳过 `#[cfg(test)]` 管的项：还在属性 / 签名里，或在花括号块里（深度）。
#[derive(Clone, Copy, PartialEq)]
enum Skip {
    No,
    Attrs,
    Block(i64),
}

/// 去掉开头的一个 `#[…]` 属性（按方括号配对；字符串已被替换掉），返回剩下的部分。
fn strip_attr(t: &str) -> Option<&str> {
    let rest = t.strip_prefix("#[")?;
    let mut depth = 1;
    for (k, ch) in rest.char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&rest[k + 1..]);
                }
            }
            _ => {}
        }
    }
    Some("") // 属性跨行：这一行剩下的都是属性
}

/// 跳过状态下吃进一段代码，返回新状态。
fn advance(mut skip: Skip, text: &str) -> Skip {
    let mut t = text.trim();
    if skip == Skip::Attrs {
        while let Some(rest) = strip_attr(t) {
            t = rest.trim();
        }
        if t.is_empty() {
            return Skip::Attrs;
        }
    }
    let mut depth = if let Skip::Block(d) = skip { d } else { 0 };
    depth += t.matches('{').count() as i64;
    depth -= t.matches('}').count() as i64;
    let opened = matches!(skip, Skip::Block(_)) || t.contains('{');
    skip = if opened && depth > 0 {
        Skip::Block(depth)
    } else if !opened && !t.ends_with(';') {
        Skip::Attrs // 项还没写完（签名跨行）
    } else {
        Skip::No
    };
    skip
}

/// 每一行是否在 `#[cfg(test)]` 管的项里（属性本身、后续属性、跨行签名、以 `;` 结尾的项或一个花括号块）。
/// 属性和项写在同一行（`#[cfg(test)] mod tests;`）也认；项结束后的生产代码照查（codex 第 1 轮 #2、第 2 轮 #1）。
fn test_lines(code: &[String]) -> Vec<bool> {
    let mut out = vec![false; code.len()];
    let mut skip = Skip::No;
    for (i, line) in code.iter().enumerate() {
        let mut text = line.as_str();
        if skip == Skip::No {
            let Some(rest) = line.trim_start().strip_prefix("#[cfg(test)]") else {
                continue;
            };
            skip = Skip::Attrs;
            text = rest;
        }
        if let Some(o) = out.get_mut(i) {
            *o = true;
        }
        skip = advance(skip, text);
    }
    out
}

/// 整个文件里的字符串字面量：(起始行下标, 内容)。跨行字符串、行尾 `\` 续行（去掉换行和下一行开头的空白）、
/// `r"…"` / `r#"…"#` / `b"…"` 都认；注释、字符字面量（`'"'`）里的引号不算（codex 第 2 轮 #2）。
fn string_literals(src: &str) -> Vec<(usize, String)> {
    let c: Vec<char> = src.chars().collect();
    let at = |k: usize| c.get(k).copied().unwrap_or('\0');
    let ident = |ch: char| ch.is_alphanumeric() || ch == '_';
    let mut out = Vec::new();
    let (mut i, mut line) = (0usize, 0usize);
    while i < c.len() {
        let ch = at(i);
        if ch == '\n' {
            line += 1;
            i += 1;
        } else if ch == '/' && at(i + 1) == '/' {
            while i < c.len() && at(i) != '\n' {
                i += 1;
            }
        } else if ch == '/' && at(i + 1) == '*' {
            let mut depth = 1;
            i += 2;
            while i < c.len() && depth > 0 {
                if at(i) == '/' && at(i + 1) == '*' {
                    depth += 1;
                    i += 2;
                } else if at(i) == '*' && at(i + 1) == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if at(i) == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
        } else if ch == '\'' {
            // 字符字面量 'x' / '\…'；否则是生命周期 'a
            if at(i + 1) == '\\' {
                i += 2;
                while i < c.len() && at(i) != '\'' {
                    i += 1;
                }
                i += 1;
            } else if at(i + 2) == '\'' {
                i += 3;
            } else {
                i += 1;
            }
        } else if ch == 'r'
            && (at(i + 1) == '"' || at(i + 1) == '#')
            && (i == 0 || !ident(at(i - 1)) || (at(i - 1) == 'b' && (i < 2 || !ident(at(i - 2)))))
        {
            let mut k = i + 1;
            let mut hashes = 0;
            while at(k) == '#' {
                hashes += 1;
                k += 1;
            }
            if at(k) != '"' {
                i += 1;
                continue;
            }
            let start = line;
            let mut s = String::new();
            k += 1;
            while k < c.len() {
                if at(k) == '"' && (1..=hashes).all(|h| at(k + h) == '#') {
                    k += 1 + hashes;
                    break;
                }
                if at(k) == '\n' {
                    line += 1;
                }
                s.push(at(k));
                k += 1;
            }
            out.push((start, s));
            i = k;
        } else if ch == '"' {
            let start = line;
            let mut s = String::new();
            i += 1;
            while i < c.len() && at(i) != '"' {
                if at(i) == '\\' {
                    if at(i + 1) == '\n' {
                        // 续行：换行和下一行开头的空白都不进字符串
                        i += 1;
                        while at(i).is_whitespace() {
                            if at(i) == '\n' {
                                line += 1;
                            }
                            i += 1;
                        }
                        continue;
                    }
                    // 转义按 Rust 的意思还原（`\n` 是换行，不是字母 n；codex 第 3 轮 #1）
                    let (ch, used) = match at(i + 1) {
                        'n' => ('\n', 2),
                        't' => ('\t', 2),
                        'r' => ('\r', 2),
                        '0' => ('\0', 2),
                        'x' => {
                            let hex: String = [at(i + 2), at(i + 3)].iter().collect();
                            let v = u8::from_str_radix(&hex, 16).unwrap_or(b'?');
                            (char::from(v), 4)
                        }
                        'u' if at(i + 2) == '{' => {
                            let mut k = i + 3;
                            let mut hex = String::new();
                            while k < c.len() && at(k) != '}' {
                                hex.push(at(k));
                                k += 1;
                            }
                            let v = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                            (v.unwrap_or('?'), k + 1 - i)
                        }
                        other => (other, 2),
                    };
                    s.push(ch);
                    i += used;
                    continue;
                }
                if at(i) == '\n' {
                    line += 1;
                }
                s.push(at(i));
                i += 1;
            }
            out.push((start, s));
            i += 1;
        } else {
            i += 1;
        }
    }
    out
}

pub fn file_problems(rel: &str, src: &str, names: &[String]) -> Vec<String> {
    let lexed = lex_lines(src);
    let skipped = test_lines(&code_lines(src));
    let mut out = Vec::new();
    for (i, s) in string_literals(src) {
        if skipped.get(i).copied().unwrap_or(false) {
            continue;
        }
        let comment = lexed.get(i).map(|(_, c)| c.as_str()).unwrap_or("");
        if has_reason(comment, ALLOW) {
            continue;
        }
        if let Some(n) = names.iter().find(|n| bounded_in(&s, n)) {
            out.push(format!(
                "{rel}:{}：写死了模型名 `{n}`；正本是 {NAMES_FILE} 的 ModelId，改用 ModelId::as_str / NAMES / parse（R-MODEL）",
                i + 1
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARAMS: &str = r#"
impl ModelId {
    pub const fn as_str(self) -> &'static str {
        match self {
            ModelId::Gtcrn => "gtcrn",
            ModelId::FastEnhancerM => "fastenhancer-m",
        }
    }
    pub const LEGACY: [(&'static str, ModelId); 1] = [("fastenhancer_m", ModelId::FastEnhancerM)];
}
"#;

    fn names() -> Vec<String> {
        watched_names(PARAMS).unwrap()
    }

    #[test]
    fn watched_names_are_read_from_the_source() {
        assert_eq!(names(), ["fastenhancer-m", "fastenhancer_m"]);
        // 没有旧写法 → 报错（check 据此报红）
        assert!(
            watched_names("pub const LEGACY: [(&str, ModelId); 0] = [];\nfn as_str() {}").is_err()
        );
    }

    /// 多个模型、rustfmt 换行、`Self::` 写法都读得出；旧写法找不到正本名 → 报错（codex 第 1 轮 #3）。
    #[test]
    fn watched_names_handle_layout_and_missing_canonical() {
        let src = r#"
impl ModelId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Gtcrn =>
                "gtcrn",
            ModelId::FastEnhancerM => "fastenhancer-m",
        }
    }
    pub const LEGACY: [(&'static str, ModelId); 2] = [
        (
            "gtcrn_v1",
            Self::Gtcrn,
        ),
        ("fastenhancer_m", ModelId::FastEnhancerM),
    ];
}
"#;
        assert_eq!(
            watched_names(src).unwrap(),
            ["fastenhancer-m", "fastenhancer_m", "gtcrn", "gtcrn_v1"]
        );
        let missing = src.replace("ModelId::FastEnhancerM => \"fastenhancer-m\",", "");
        let e = watched_names(&missing).unwrap_err();
        assert!(e.contains("fastenhancer_m"), "{e}");
    }

    #[test]
    fn real_names_file_parses() {
        let src = fs::read_to_string(crate::root().join(NAMES_FILE)).unwrap();
        let n = watched_names(&src).unwrap();
        assert!(n.iter().any(|s| s == "fastenhancer-m"), "{n:?}");
        assert!(n.iter().any(|s| s == "fastenhancer_m"), "{n:?}");
    }

    #[test]
    fn hard_coded_names_are_flagged() {
        for bad in [
            "    #[value(name = \"fastenhancer-m\")]\n",
            "let x = [\"gtcrn\", \"fastenhancer_m\"];\n",
            "writeln!(s, \"`gtcrn` / `fastenhancer-m` = 真模型\");\n",
        ] {
            assert_eq!(file_problems("a.rs", bad, &names()).len(), 1, "{bad}");
        }
    }

    #[test]
    fn allowed_forms_pass() {
        for ok in [
            // 注释里提到
            "// 命令行写 fastenhancer-m\nlet a = 1;\n",
            // 更长名字的一部分（另一个模型 / 另一个档）
            "let p = \"fastenhancer-m2\";\nlet q = \"x_fastenhancer_m\";\n",
            // 放行
            "let p = \"w/fastenhancer_m.onnx\"; // tidy: 不是模型名：权重文件名\n",
            // 单测模块不查
            "fn f() {}\n#[cfg(test)]\nmod t {\n    const A: &str = \"fastenhancer-m\";\n}\n",
            "#[cfg(test)]\n#[allow(dead_code)]\nconst A: &str = \"fastenhancer-m\";\n",
            // 不认的字面量
            "let g = \"gtcrn\";\n",
            // 同一行的测试项
            "#[cfg(test)] const A: &str = \"fastenhancer-m\";\nfn f() {}\n",
            // 注释与生命周期里的引号不开字符串
            "fn f<'a>(x: &'a str) {} // \"fastenhancer-m\"\n/* \"fastenhancer_m\" */\n",
        ] {
            assert!(file_problems("a.rs", ok, &names()).is_empty(), "{ok}");
        }
        // `#[cfg(test)]` 管的项之后的生产代码照查（codex 第 1 轮 #2、第 2 轮 #1）；跨行字符串也查（第 2 轮 #2）
        for bad in [
            "#[cfg(test)] mod tests;\nfn prod() -> &'static str { \"fastenhancer-m\" }\n",
            "#[cfg(test)] #[path = \"t.rs\"] mod tests;\nconst P: &str = \"fastenhancer_m\";\n",
            "let s = \"模型：\\\n    fastenhancer-m\";\n",
            "let s = \"第一行\nfastenhancer-m 第二行\";\n",
            "let s = r#\"\n\"q\" fastenhancer_m\n\"#;\n",
            "let q = '\"'; let s = \"fastenhancer-m\";\n",
            // 转义紧挨着名字：\n \t \u{…} 还原成真字符后再判边界（codex 第 3 轮 #1）
            "let s = \"\\nfastenhancer-m\";\n",
            "let s = \"\\tfastenhancer_m\\t\";\n",
            "let s = \"\\u{66}astenhancer-m\";\n",
            "#[cfg(test)]\n#[path = \"t.rs\"]\nmod tests;\nfn prod() -> &'static str { \"fastenhancer-m\" }\n",
            "#[cfg(test)]\nmod t {\n    fn x() { if true { } }\n}\nconst B: &str = \"fastenhancer_m\";\n",
            "#[cfg(test)]\nfn helper(\n    a: u8,\n) {\n}\nconst C: &str = \"fastenhancer-m\";\n",
        ] {
            assert_eq!(file_problems("a.rs", bad, &names()).len(), 1, "{bad}");
        }
        // 放行要写理由
        let no_reason = "let p = \"fastenhancer_m\"; // tidy: 不是模型名：\n";
        assert_eq!(file_problems("a.rs", no_reason, &names()).len(), 1);
    }

    /// 整个仓库当前过这条规则。
    #[test]
    fn repo_passes() {
        let mut p = Vec::new();
        check(&crate::root(), &mut p);
        assert!(p.is_empty(), "{p:#?}");
    }
}
