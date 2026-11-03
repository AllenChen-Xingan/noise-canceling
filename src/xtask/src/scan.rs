//! tidy 各条规则共用的小工具：逐行分出代码与注释（规则按语法树判之后，只用它找**行尾放行注释**——注释不在语法树里）；
//! 列出 `src/` 下的源文件；认放行标记。不依赖任何 vdn crate。

use std::path::Path;

use crate::tidy::rs_files;

/// 逐行 `(代码, 注释)`：代码去掉了注释、字符串与字符字面量（字符串换成 `""`）；注释是这一行里 `//` 与 `/* */` 的文字（放行标记只从这里认，
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

/// `c[i]` 是 `r`，前面是独立的 `b` 或 `c`（`br"…"` 字节、`cr"…"` C 原始字符串的前缀；codex #71 第 6 轮 P2 #1）。
fn byte_prefix(c: &[char], i: usize) -> bool {
    i >= 1
        && matches!(c[i - 1], 'b' | 'c')
        && (i == 1 || !(c[i - 2].is_alphanumeric() || c[i - 2] == '_'))
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
