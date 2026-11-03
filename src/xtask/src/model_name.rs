//! tidy 规则 R-MODEL：有过不止一种写法的模型名，只许在正本里写（CLAUDE.md 第五节第 5 条；issue #65）。纯文本检查，不依赖任何 vdn crate。
//!
//! 正本是 `src/types/src/params.rs` 的 `ModelId`：`as_str` 给正本名，`LEGACY` 给旧写法。本规则从那个文件读出
//! 「有旧写法的模型」的全部写法（正本名 + 旧写法），再查 `src/` 下其他 `.rs` 的字符串字面量里有没有写死它们。
//! 别处要用模型名，从 `ModelId::as_str` / `ModelId::NAMES` / `ModelId::parse` 取。
//!
//! | 查什么 | 不查 | 放行写法 |
//! |---|---|---|
//! | 字符串字面量里出现上述写法（前后不是字母、数字、`_`、`-`） | 正本文件本身；`tests/` 目录；`#[cfg(test)]` 管的那一项（单测要拿字面量造输入；它后面的代码照查），以及经 `#[cfg(test)] mod x;` 引入的测试文件；注释 | 同一行注释写 `tidy: 不是模型名：<理由>`（如权重文件名） |
//!
//! 只查有过旧写法的模型：其他模型名（如 `gtcrn`）也是路径的一段，逐个查噪声太大；一旦给它登记旧写法，就自动进来。

use std::fs;
use std::path::Path;

use crate::scan::{
    code_lines, has_reason, lex_lines, src_files, string_literals, strip_comments_keep_strings,
};
use crate::test_scope::{strings_in, test_lines, test_module_files};

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
    let files = src_files(root);
    let test_files = test_module_files(&files);
    for (rel, s) in &files {
        if rel != NAMES_FILE && !rel.contains("/tests/") && !test_files.contains(rel) {
            problems.extend(file_problems(rel, s, &names));
        }
    }
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
            "let s = \"\\u{6_6}astenhancer-m\";\n",
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
