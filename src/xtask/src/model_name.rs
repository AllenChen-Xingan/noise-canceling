//! tidy 规则 R-MODEL：有过不止一种写法的模型名，只许在正本里写（CLAUDE.md 第五节第 5 条；issue #65）。纯文本检查，不依赖任何 vdn crate。
//!
//! 正本是 `src/types/src/params.rs` 的 `ModelId`：`as_str` 给正本名，`LEGACY` 给旧写法。本规则从那个文件读出
//! 「有旧写法的模型」的全部写法（正本名 + 旧写法），再查 `src/` 下其他 `.rs` 的字符串字面量里有没有写死它们。
//! 别处要用模型名，从 `ModelId::as_str` / `ModelId::NAMES` / `ModelId::parse` 取。
//!
//! | 查什么 | 不查 | 放行写法 |
//! |---|---|---|
//! | 字符串字面量里出现上述写法（前后不是字母、数字、`_`、`-`） | 正本文件本身；`tests/` 目录；文件里第一个 `#[cfg(test)]` 之后（单测要拿字面量造输入）；注释 | 同一行注释写 `tidy: 不是模型名：<理由>`（如权重文件名） |
//!
//! 只查有过旧写法的模型：其他模型名（如 `gtcrn`）也是路径的一段，逐个查噪声太大；一旦给它登记旧写法，就自动进来。

use std::fs;
use std::path::Path;

use crate::scan::{has_reason, lex_lines, src_files, strip_comments_keep_strings};
use crate::tidy::rs_files;

/// 模型名的正本文件（相对 workspace 根）。
pub const NAMES_FILE: &str = "src/types/src/params.rs";
const ALLOW: &str = "tidy: 不是模型名：";

pub fn check(root: &Path, problems: &mut Vec<String>) {
    let Ok(src) = fs::read_to_string(root.join(NAMES_FILE)) else {
        problems.push(format!("{NAMES_FILE} 不存在：模型名的正本（R-MODEL）"));
        return;
    };
    let names = watched_names(&src);
    if names.is_empty() {
        problems.push(format!(
            "{NAMES_FILE}：读不出 ModelId::LEGACY 里的模型名；解析失败会让 R-MODEL 漏查（R-MODEL）"
        ));
        return;
    }
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    for f in all {
        let rel = f
            .strip_prefix(root)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| f.display().to_string());
        if rel == NAMES_FILE || rel.contains("/tests/") {
            continue;
        }
        if let Ok(s) = fs::read_to_string(&f) {
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
pub fn watched_names(src: &str) -> Vec<String> {
    let lines = strip_comments_keep_strings(src);
    // LEGACY 那一段（从 `const LEGACY` 到 `];`）里的 ("旧写法", ModelId::X)
    let mut legacy: Vec<(String, String)> = Vec::new();
    let mut in_legacy = false;
    let mut buf = String::new();
    for l in &lines {
        if l.contains("const LEGACY") {
            in_legacy = true;
        }
        if in_legacy {
            buf.push_str(l);
            if l.contains("];") {
                break;
            }
        }
    }
    let after_eq = buf.split_once('=').map(|(_, r)| r).unwrap_or("");
    for tuple in after_eq.split('(').skip(1) {
        let Some((old, rest)) = tuple.split_once(',') else {
            continue;
        };
        let old = old.trim().trim_matches('"');
        let Some(variant) = rest.trim().strip_prefix("ModelId::") else {
            continue;
        };
        let variant: String = variant
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !old.is_empty() && !variant.is_empty() {
            legacy.push((old.to_string(), variant));
        }
    }
    let mut out = Vec::new();
    for (old, variant) in &legacy {
        out.push(old.clone());
        // as_str 里的 `ModelId::<变体> => "<正本名>"`
        let arm = format!("ModelId::{variant} =>");
        if let Some(name) = lines
            .iter()
            .find_map(|l| l.split_once(&arm).map(|(_, r)| r.to_string()))
            .and_then(|r| strings_in(&r).into_iter().next())
        {
            out.push(name);
        }
    }
    out.sort();
    out.dedup();
    out
}

fn bounded_in(hay: &str, needle: &str) -> bool {
    let is_name = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    hay.match_indices(needle).any(|(i, _)| {
        let before = hay[..i].chars().next_back();
        let after = hay[i + needle.len()..].chars().next();
        !before.is_some_and(is_name) && !after.is_some_and(is_name)
    })
}

pub fn file_problems(rel: &str, src: &str, names: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let keep = strip_comments_keep_strings(src);
    let lexed = lex_lines(src);
    for (i, line) in keep.iter().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        let comment = lexed.get(i).map(|(_, c)| c.as_str()).unwrap_or("");
        if comment
            .split_once(ALLOW)
            .is_some_and(|(_, why)| !why.trim().is_empty())
        {
            continue;
        }
        for s in strings_in(line) {
            if let Some(n) = names.iter().find(|n| bounded_in(&s, n)) {
                out.push(format!(
                    "{rel}:{}：写死了模型名 `{n}`；正本是 {NAMES_FILE} 的 ModelId，改用 ModelId::as_str / NAMES / parse（R-MODEL）",
                    i + 1
                ));
            }
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
        watched_names(PARAMS)
    }

    #[test]
    fn watched_names_are_read_from_the_source() {
        assert_eq!(names(), ["fastenhancer-m", "fastenhancer_m"]);
        // 没有旧写法 → 读不出（check 据此报错）
        assert!(watched_names("pub const LEGACY: [(&str, ModelId); 0] = [];").is_empty());
    }

    #[test]
    fn real_names_file_parses() {
        let src = fs::read_to_string(crate::root().join(NAMES_FILE)).unwrap();
        let n = watched_names(&src);
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
            // 单测之后不查
            "fn f() {}\n#[cfg(test)]\nmod t { const A: &str = \"fastenhancer-m\"; }\n",
            // 不认的字面量
            "let g = \"gtcrn\";\n",
        ] {
            assert!(file_problems("a.rs", ok, &names()).is_empty(), "{ok}");
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
