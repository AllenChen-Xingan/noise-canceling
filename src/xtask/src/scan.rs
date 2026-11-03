//! tidy 各条文本规则共用的扫描小工具：逐行分出代码、注释、字符串；列出 `src/` 下的源文件；认放行标记。
//! 用它的规则：`no_mirror.rs`（R-MIRROR）· `param_range.rs`（R-PARAM）· `model_name.rs`（R-MODEL）。不依赖任何 vdn crate。

use std::path::Path;

use crate::tidy::rs_files;

/// 逐行剥掉 `//` 与 `/* */` 注释，字符串原样保留（给 `param_range::desc_keys` 与 `model_name` 用：它们要看字符串里的内容）。
pub fn strip_comments_keep_strings(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block = 0u32;
    for line in src.lines() {
        let c: Vec<char> = line.chars().collect();
        let mut o = String::new();
        let mut i = 0;
        let mut in_str = false;
        while i < c.len() {
            if in_block > 0 {
                if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    in_block -= 1;
                    i += 2;
                } else if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    in_block += 1;
                    i += 2;
                } else {
                    i += 1;
                }
            } else if in_str {
                o.push(c[i]);
                if c[i] == '\\' {
                    if let Some(&n) = c.get(i + 1) {
                        o.push(n);
                    }
                    i += 2;
                } else {
                    in_str = c[i] != '"';
                    i += 1;
                }
            } else if c[i] == '/' && c.get(i + 1) == Some(&'/') {
                break;
            } else if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                in_block = 1;
                i += 2;
            } else {
                in_str = c[i] == '"';
                o.push(c[i]);
                i += 1;
            }
        }
        out.push(o);
    }
    out
}

/// 逐行给出「只剩代码」的文本：去掉 `//` 行注释、`/* */` 块注释（可跨行）、字符串与字符字面量（可跨行，含 `r"…"` / `r#"…"#`）。
/// 字符串换成 `""`，免得测试里写在字符串中的负例被当成代码（codex 第 1 轮 P1 #1）。
pub fn code_lines(src: &str) -> Vec<String> {
    lex_lines(src).into_iter().map(|(c, _)| c).collect()
}

/// 逐行 `(代码, 注释)`：代码同 [`code_lines`]；注释是这一行里 `//` 与 `/* */` 的文字（放行标记只从这里认，
/// 字符串里写的标记不算，codex 第 2 轮 P2 #3）。
pub fn lex_lines(src: &str) -> Vec<(String, String)> {
    #[derive(Clone, Copy, PartialEq)]
    enum St {
        Code,
        Block(u32),
        Str,
        Raw(usize),
    }
    let mut st = St::Code;
    let mut out = Vec::new();
    for line in src.lines() {
        let c: Vec<char> = line.chars().collect();
        let mut o = String::new();
        let mut cm = String::new();
        let mut i = 0;
        while i < c.len() {
            match st {
                St::Block(d) => {
                    if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                        st = if d <= 1 { St::Code } else { St::Block(d - 1) };
                        i += 2;
                    } else if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                        st = St::Block(d + 1);
                        i += 2;
                    } else {
                        cm.push(c[i]);
                        i += 1;
                    }
                }
                St::Str => {
                    if c[i] == '\\' {
                        i += 2;
                    } else if c[i] == '"' {
                        o.push('"');
                        st = St::Code;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                St::Raw(h) => {
                    if c[i] == '"' && c[i + 1..].iter().take(h).filter(|&&x| x == '#').count() == h
                    {
                        o.push('"');
                        st = St::Code;
                        i += 1 + h;
                    } else {
                        i += 1;
                    }
                }
                St::Code => {
                    let ch = c[i];
                    let prev_ident = i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_');
                    if ch == '/' && c.get(i + 1) == Some(&'/') {
                        cm.extend(&c[i..]);
                        break;
                    } else if ch == '/' && c.get(i + 1) == Some(&'*') {
                        st = St::Block(1);
                        i += 2;
                    } else if ch == 'r'
                        && (!prev_ident || byte_prefix(&c, i))
                        && matches!(c.get(i + 1), Some('"') | Some('#'))
                    {
                        // r"…" / r#"…"# / br"…" / br#"…"#
                        let h = c[i + 1..].iter().take_while(|&&x| x == '#').count();
                        if c.get(i + 1 + h) == Some(&'"') {
                            o.push('"');
                            st = St::Raw(h);
                            i += 2 + h;
                        } else {
                            o.push(ch);
                            i += 1;
                        }
                    } else if ch == '"' {
                        o.push('"');
                        st = St::Str;
                        i += 1;
                    } else if ch == '\'' {
                        // 字符字面量 'x' / '\n'；否则是生命周期，原样保留
                        if c.get(i + 1) == Some(&'\\') {
                            let end = c
                                .get(i + 3..)
                                .and_then(|t| t.iter().position(|&x| x == '\''))
                                .map_or(c.len(), |k| i + 4 + k);
                            o.push_str("' '");
                            i = end;
                        } else if c.get(i + 2) == Some(&'\'') {
                            o.push_str("' '");
                            i += 3;
                        } else {
                            o.push(ch);
                            i += 1;
                        }
                    } else {
                        o.push(ch);
                        i += 1;
                    }
                }
            }
        }
        out.push((o, cm));
    }
    out
}

/// `c[i]` 是 `r`，前面是独立的 `b`（`br"…"` 字节原始字符串的前缀）。
fn byte_prefix(c: &[char], i: usize) -> bool {
    i >= 1 && c[i - 1] == 'b' && (i == 1 || !(c[i - 2].is_alphanumeric() || c[i - 2] == '_'))
}

/// `src/` 下全部 `.rs`：(相对 workspace 根的路径, 全文)。读不出的文件跳过。
pub fn src_files(root: &Path) -> Vec<(String, String)> {
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    all.into_iter()
        .filter_map(|f| {
            let rel = f
                .strip_prefix(root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| f.display().to_string());
            std::fs::read_to_string(&f).ok().map(|s| (rel, s))
        })
        .collect()
}

/// 放行标记后面写了理由（`tidy: …：<理由>`，理由非空）。
pub fn has_reason(line: &str, marker: &str) -> bool {
    line.split_once(marker)
        .is_some_and(|(_, why)| !why.trim().is_empty())
}

/// 整个文件里的字符串字面量：(起始行下标, 内容)。跨行字符串、行尾 `\` 续行（去掉换行和下一行开头的空白）、
/// `r"…"` / `r#"…"#` / `b"…"` 都认；注释、字符字面量（`'"'`）里的引号不算（R-MODEL 用；codex 012 第 2 轮 #2）。
pub fn string_literals(src: &str) -> Vec<(usize, String)> {
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
                            // Rust 允许在 \u{…} 的数字里写下划线（第 4 轮 #4）
                            let hex: String = hex.chars().filter(|c| *c != '_').collect();
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
