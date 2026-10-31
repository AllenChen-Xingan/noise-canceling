//! 自写的行文本格式：场景参数表、我的配置、会话记录共用。人能直接读、能 diff、能进 git。
//!
//! ```text
//! # 注释
//! vdn-scene-table = 1        ← 第一行：类型 = 格式号
//! version = 3
//! [scene 1]                  ← 小节：[种类 名字]
//! user = cafe
//! floor_db = -12
//! ```
//!
//! 规则：一行一个 `键 = 值`；值不加引号、不含换行；`#` 开头整行是注释；空行忽略。
//! 只在非实时线程用（解析会分配）。

/// 解析出的一行。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line<'a> {
    /// `[kind name]`；`name` 可为空
    Section { kind: &'a str, name: &'a str },
    /// `key = value`
    Kv { key: &'a str, value: &'a str },
}

/// 解析错误：第几行（从 1 起）、什么错。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextError {
    pub line: usize,
    pub what: &'static str,
}

impl core::fmt::Display for TextError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "第 {} 行：{}", self.line, self.what)
    }
}

impl std::error::Error for TextError {}

/// 逐行解析；回调拿到 (行号, 行)。注释与空行跳过。
pub fn parse<'a>(
    src: &'a str,
    mut f: impl FnMut(usize, Line<'a>) -> Result<(), TextError>,
) -> Result<(), TextError> {
    for (i, raw) in src.lines().enumerate() {
        let line_no = i + 1;
        let s = raw.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let line = if let Some(inner) = s.strip_prefix('[') {
            let inner = inner.strip_suffix(']').ok_or(TextError {
                line: line_no,
                what: "小节缺右方括号",
            })?;
            let inner = inner.trim();
            let (kind, name) = match inner.split_once(char::is_whitespace) {
                Some((k, n)) => (k, n.trim()),
                None => (inner, ""),
            };
            Line::Section { kind, name }
        } else {
            let (k, v) = s.split_once('=').ok_or(TextError {
                line: line_no,
                what: "不是「键 = 值」",
            })?;
            let key = k.trim();
            if key.is_empty() {
                return Err(TextError {
                    line: line_no,
                    what: "键为空",
                });
            }
            Line::Kv {
                key,
                value: v.trim(),
            }
        };
        f(line_no, line)?;
    }
    Ok(())
}

pub(crate) fn num<T: core::str::FromStr>(line: usize, v: &str) -> Result<T, TextError> {
    v.parse().map_err(|_| TextError {
        line,
        what: "数值写错了",
    })
}

pub(crate) fn err(line: usize, what: &'static str) -> TextError {
    TextError { line, what }
}

/// 写 f32：最短可往返表示（Rust `{}` 对 f32 保证往返）。
pub(crate) fn f32_str(v: f32) -> String {
    format!("{v}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sections_kv_and_skips_comments() {
        let src = "# c\nfmt = 1\n\n[scene cafe]\nfloor_db = -12\n[end]\n";
        let mut got = Vec::new();
        parse(src, |n, l| {
            got.push((n, l));
            Ok(())
        })
        .unwrap();
        assert_eq!(
            got,
            vec![
                (2, Line::Kv { key: "fmt", value: "1" }),
                (4, Line::Section { kind: "scene", name: "cafe" }),
                (5, Line::Kv { key: "floor_db", value: "-12" }),
                (6, Line::Section { kind: "end", name: "" }),
            ]
        );
    }

    #[test]
    fn reports_line_numbers() {
        let e = parse("a = 1\n[oops\n", |_, _| Ok(())).unwrap_err();
        assert_eq!(e.line, 2);
        let e = parse("novalue\n", |_, _| Ok(())).unwrap_err();
        assert_eq!(e.line, 1);
    }

    #[test]
    fn f32_roundtrips() {
        for v in [0.1f32, -12.0, 1e-7, 0.333_333_34] {
            assert_eq!(f32_str(v).parse::<f32>().unwrap(), v);
        }
    }
}
