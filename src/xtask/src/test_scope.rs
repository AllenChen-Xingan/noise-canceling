//! 哪些代码属于测试：`#[cfg(test)]` 管的项（逐行标出）、只被测试引用的模块文件。tidy 的文本规则用它跳过测试代码
//!（现用于 R-MODEL，`model_name.rs`）。纯文本，不依赖任何 vdn crate。

use std::path::Path;

use crate::scan::{code_lines, strip_comments_keep_strings};

/// 路径归一：去掉 `.`、按 `..` 回退一级，免得 `./both.rs` 和 `both.rs` 被当成两个文件（codex 012 第 7 轮 #1）。
pub fn normalize(p: &Path) -> String {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out.display().to_string()
}

/// 每一行开头所在的内联模块路径（`mod a { mod b { … } }` 里的行是 `["a", "b"]`）。按代码行（字符串已去掉）数花括号。
pub fn inline_prefixes(code: &[String]) -> Vec<Vec<String>> {
    let mut out = Vec::with_capacity(code.len());
    let mut depth = 0i64;
    let mut stack: Vec<(String, i64)> = Vec::new();
    for line in code {
        out.push(stack.iter().map(|(n, _)| n.clone()).collect());
        for (k, ch) in line.char_indices() {
            match ch {
                '{' => {
                    depth += 1;
                    let before: Vec<&str> = line[..k].split_whitespace().collect();
                    if let [.., kw, name] = before.as_slice()
                        && *kw == "mod"
                        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
                    {
                        stack.push((name.to_string(), depth));
                    }
                }
                '}' => {
                    depth -= 1;
                    while stack.last().is_some_and(|(_, d)| *d > depth) {
                        stack.pop();
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// 这一行只有属性（`#[…]`，可连写几个），没有别的代码。
pub fn only_attrs(line: &str) -> bool {
    let mut t = line.trim();
    if !t.starts_with("#[") {
        return false;
    }
    while let Some(r) = strip_attr(t) {
        t = r.trim();
    }
    t.is_empty()
}

/// 只被测试引用的模块文件（相对路径）：经 `#[cfg(test)] mod x;`（可带 `#[path = "…"]`）引入、且没有任何生产代码的
/// `mod` 声明指向它（同一文件既做生产又做测试就照查，codex 012 第 6 轮 #2）。
/// 按 Rust 的规则找文件：`mod.rs` / `lib.rs` / `main.rs` 里声明的在同目录，别的文件 `foo.rs` 里声明的在 `foo/` 下；
/// 声明在内联模块 `mod outer { … }` 里的再拼上 `outer/`；`#[path]` 在文件顶层相对所在目录，在内联模块里相对上述模块目录
///（第 6 轮 #1）。
pub fn test_module_files(files: &[(String, String)]) -> Vec<String> {
    let mut test = Vec::new();
    let mut prod = Vec::new();
    for (rel, src) in files {
        // 代码行（字符串换成 ""）定位属性和 `mod`，原文行（保留字符串）取 `#[path]` 的值：
        // 字符串里写的伪声明不算（第 4 轮 #2）
        let code = code_lines(src);
        let raw = strip_comments_keep_strings(src);
        let prefixes = inline_prefixes(&code);
        for (i, line) in code.iter().enumerate() {
            // 这一行里的每一条 `mod 名字;`（外部模块声明；同一行写几条也都收，codex 012 第 7 轮 #2）
            for (at, _) in line.match_indices("mod ") {
                if at > 0
                    && line[..at]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    continue;
                }
                let after = &line[at + 4..];
                let name: String = after
                    .trim_start()
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let rest = after.trim_start()[name.len()..].trim_start();
                if name.is_empty() || !rest.starts_with(';') {
                    continue; // 内联模块 `mod x {` 或别的写法
                }
                // 它的属性：本行 `mod` 之前的部分 + 往上紧挨着的「整行只有属性」的行（不依赖属性顺序，第 4 轮 #3）
                // 本行里这条声明之前、上一条 `;` 之后的部分；只有它是本行第一条时，才往上收属性行
                let seg = line[..at].rfind(';').map_or(0, |p| p + 1);
                let mut start = i;
                while seg == 0 && start > 0 && only_attrs(&code[start - 1]) {
                    start -= 1;
                }
                let mut attrs_code = code[start..i].join(" ");
                attrs_code.push_str(&line[seg..at]);
                let attrs_raw = raw.get(start..=i).map(|r| r.join(" ")).unwrap_or_default();
                let is_test = attrs_code.contains("#[cfg(test)]");
                let path = Path::new(rel);
                let dir = path.parent().unwrap_or(Path::new(""));
                let inline = prefixes.get(i).cloned().unwrap_or_default();
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                let mut base = if ["mod", "lib", "main"].contains(&stem) {
                    dir.to_path_buf()
                } else {
                    dir.join(stem)
                };
                for m in &inline {
                    base = base.join(m);
                }
                let explicit = attrs_code
                    .contains("#[path")
                    .then(|| {
                        attrs_raw
                            .split_once("#[path")
                            .and_then(|(_, r)| strings_in(r).into_iter().next())
                    })
                    .flatten();
                let file = match explicit {
                    Some(p) if inline.is_empty() => dir.join(p),
                    Some(p) => base.join(p),
                    None => {
                        let flat_file = base.join(format!("{name}.rs"));
                        if files.iter().any(|(r, _)| Path::new(r) == flat_file) {
                            flat_file
                        } else {
                            base.join(&name).join("mod.rs")
                        }
                    }
                };
                let f = normalize(&file);
                if is_test { test.push(f) } else { prod.push(f) }
            }
        }
    }
    test.retain(|f| !prod.contains(f));
    test.sort();
    test.dedup();
    test
}

/// 一行里所有字符串字面量的内容（不含引号）。注释已先剥掉。
pub fn strings_in(line: &str) -> Vec<String> {
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

/// 正在跳过 `#[cfg(test)]` 管的项：还在属性 / 签名里，或在花括号块里（深度）。
#[derive(Clone, Copy, PartialEq)]
enum Skip {
    No,
    Attrs,
    Block(i64),
}

/// 去掉开头的一个 `#[…]` 属性（按方括号配对；字符串已被替换掉），返回剩下的部分。
pub fn strip_attr(t: &str) -> Option<&str> {
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
pub fn test_lines(code: &[String]) -> Vec<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `#[cfg(test)] mod x;` 引入的文件按 Rust 规则找到；内联测试模块、非测试模块不算。
    #[test]
    fn test_module_files_are_resolved() {
        let f = |rel: &str, src: &str| (rel.to_string(), src.to_string());
        let files = [
            f(
                "src/a/src/session.rs",
                "#[cfg(test)]\nmod tests;\nmod real;\n",
            ),
            f("src/a/src/session/tests.rs", ""),
            f(
                "src/a/src/policy.rs",
                "#[cfg(test)]\n#[path = \"policy_tests.rs\"]\nmod tests;\n",
            ),
            f(
                "src/a/src/lib.rs",
                "#[cfg(test)] mod t2;\n#[cfg(test)]\nmod inline { }\n",
            ),
            f("src/a/src/t2/mod.rs", ""),
            // 第 4 轮 #2：字符串里的伪声明不算；第 4 轮 #3：#[path] 写在 cfg 前面也认
            f(
                "src/b/src/lib.rs",
                "const S: &str = \"#[cfg(test)] mod real;\";\nmod real;\n#[path = \"x_tests.rs\"]\n#[cfg(test)]\nmod tests;\n",
            ),
            f("src/b/src/real.rs", ""),
            // 第 5 轮 #2：内联模块里声明的测试模块在 outer/ 下，根层同名的生产文件不豁免
            f(
                "src/c/src/lib.rs",
                "mod outer {\n    #[cfg(test)]\n    mod real;\n}\nmod real;\n",
            ),
            f("src/c/src/real.rs", ""),
            f("src/c/src/outer/real.rs", ""),
            // 第 6 轮 #1：内联模块里的 #[path] 按 outer/ 解析；第 6 轮 #2：生产也引用的文件不豁免
            f(
                "src/d/src/lib.rs",
                "mod outer {\n    #[cfg(test)]\n    #[path = \"t.rs\"]\n    mod tests;\n}\nmod both;\n#[cfg(test)]\n#[path = \"both.rs\"]\nmod again;\n",
            ),
            f("src/d/src/both.rs", ""),
            // 第 7 轮：路径写成 ./x.rs 也是同一个文件；同一行第二条生产声明也收
            f(
                "src/e/src/lib.rs",
                "#[path = \"./sub/../both.rs\"]\nmod real;\nmod helper; mod two;\n#[cfg(test)]\n#[path = \"both.rs\"]\nmod t1;\n#[cfg(test)]\n#[path = \"two.rs\"]\nmod t2;\n",
            ),
            f("src/e/src/both.rs", ""),
            f("src/e/src/two.rs", ""),
        ];
        let got = test_module_files(&files);
        assert_eq!(
            got,
            [
                "src/a/src/policy_tests.rs",
                "src/a/src/session/tests.rs",
                "src/a/src/t2/mod.rs",
                "src/b/src/x_tests.rs",
                "src/c/src/outer/real.rs",
                "src/d/src/outer/t.rs"
            ]
        );
    }
}
