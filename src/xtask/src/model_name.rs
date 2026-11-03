//! tidy 规则 R-MODEL：有过不止一种写法的模型名，只许在正本里写（CLAUDE.md 第五节第 5 条；issue #65）。
//! 按语法树查（issue #71）：不依赖任何 vdn crate。
//!
//! 正本是 `src/types/src/params.rs` 的 `ModelId`：`as_str` 给正本名，`LEGACY` 给旧写法。本规则从那个文件的语法树读出
//! 「有旧写法的模型」的全部写法（正本名 + 旧写法），再查 `src/` 下其他 `.rs` 的字符串字面量里有没有写死它们。
//! 别处要用模型名，从 `ModelId::as_str` / `ModelId::NAMES` / `ModelId::parse` 取。
//!
//! | 查什么 | 不查 | 放行写法 |
//! |---|---|---|
//! | 字符串字面量（含宏参数、属性里的；值按转义解码后）里出现上述写法（前后不是字母、数字、`_`、`-`） | 正本文件本身；模块树判出的测试文件与测试项（`ast::modtree`）；注释与文档注释 | 同一行注释写 `tidy: 不是模型名：<理由>`（如权重文件名） |
//!
//! 只查有过旧写法的模型：其他模型名（如 `gtcrn`）也是路径的一段，逐个查噪声太大；一旦给它登记旧写法，就自动进来。

use std::fs;
use std::path::Path;

use syn::visit::Visit;

use crate::ast::modtree::{self, file_test_ranges, in_ranges};
use crate::ast::str_literals;
use crate::scan::{has_reason, lex_lines, src_files};

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
    let scope = modtree::build(root);
    for (rel, s) in &src_files(root) {
        if rel != NAMES_FILE && !scope.is_test_file(rel) {
            problems.extend(file_problems(rel, s, &names));
        }
    }
}

/// `ModelId::X` 或 `Self::X` 的变体名。
fn variant_of(path: &syn::Path) -> Option<String> {
    let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    match segs.as_slice() {
        [a, v] if a == "ModelId" || a == "Self" => Some(v.clone()),
        _ => None,
    }
}

fn variant(e: &syn::Expr) -> Option<String> {
    match e {
        syn::Expr::Path(p) => variant_of(&p.path),
        _ => None,
    }
}

fn lit_str(e: &syn::Expr) -> Option<String> {
    match e {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) => Some(s.value()),
        syn::Expr::Group(g) => lit_str(&g.expr),
        syn::Expr::Paren(p) => lit_str(&p.expr),
        _ => None,
    }
}

/// `as_str` 里 match 分支：变体 → 正本名。
#[derive(Default)]
struct Arms(Vec<(String, String)>);

impl<'ast> Visit<'ast> for Arms {
    fn visit_arm(&mut self, a: &'ast syn::Arm) {
        if let syn::Pat::Path(p) = &a.pat
            && let Some(v) = variant_of(&p.path)
            && let Some(name) = lit_str(&a.body)
        {
            self.0.push((v, name));
        }
        syn::visit::visit_arm(self, a);
    }
}

/// 正本文件里「有旧写法的模型」的全部写法：`impl ModelId` 里 `LEGACY` 的旧写法，加上同一变体在 `as_str` 里的正本名。
/// 读不出旧写法、或某个旧写法找不到正本名 → `Err`（解析失败不能静默漏查）。
pub fn watched_names(src: &str) -> Result<Vec<String>, String> {
    let file = crate::ast::parse_src(src).map_err(|e| format!("语法树解析失败：{}", e.msg))?;
    let (mut legacy, mut arms) = (None, Arms::default());
    for it in &file.items {
        let syn::Item::Impl(i) = it else { continue };
        if !matches!(&*i.self_ty, syn::Type::Path(t) if t.path.is_ident("ModelId")) {
            continue;
        }
        for ii in &i.items {
            match ii {
                syn::ImplItem::Const(c) if c.ident == "LEGACY" => legacy = Some(&c.expr),
                syn::ImplItem::Fn(f) if f.sig.ident == "as_str" => arms.visit_block(&f.block),
                _ => {}
            }
        }
    }
    let Some(syn::Expr::Array(arr)) = legacy else {
        return Err("找不到 ModelId::LEGACY 数组".into());
    };
    let mut out = Vec::new();
    for el in &arr.elems {
        let syn::Expr::Tuple(t) = el else {
            return Err("ModelId::LEGACY 的元素不是 (旧写法, 变体)".into());
        };
        let (Some(old), Some(v)) = (
            t.elems.first().and_then(lit_str),
            t.elems.iter().nth(1).and_then(variant),
        ) else {
            return Err("ModelId::LEGACY 的元素不是 (旧写法, 变体)".into());
        };
        let canon = arms
            .0
            .iter()
            .find(|(av, _)| *av == v)
            .map(|(_, n)| n.clone())
            .ok_or(format!(
                "旧写法 `{old}` 的模型 {v} 在 as_str 里找不到正本名"
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

/// 一个文件的问题。文件解析不了也报（不静默跳过）。
pub fn file_problems(rel: &str, src: &str, names: &[String]) -> Vec<String> {
    let file = match crate::ast::parse_src(src) {
        Ok(f) => f,
        Err(e) => {
            return vec![format!(
                "{rel}:{}：语法树解析失败：{}（R-MODEL）",
                e.line, e.msg
            )];
        }
    };
    let tests = file_test_ranges(rel, &file);
    let lexed = lex_lines(src);
    let mut out = Vec::new();
    for (p, s) in str_literals(&file) {
        if in_ranges(&tests, p) {
            continue;
        }
        let line = p.0;
        let comment = lexed.get(line - 1).map(|(_, c)| c.as_str()).unwrap_or("");
        if has_reason(comment, ALLOW) {
            continue;
        }
        if let Some(n) = names.iter().find(|n| bounded_in(&s, n)) {
            out.push(format!(
                "{rel}:{line}：写死了模型名 `{n}`；正本是 {NAMES_FILE} 的 ModelId，改用 ModelId::as_str / NAMES / parse（R-MODEL）"
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
            "struct A {\n    #[value(name = \"fastenhancer-m\")]\n    x: u8,\n}\n",
            "fn f() {\n    let x = [\"gtcrn\", \"fastenhancer_m\"];\n}\n",
            "fn f() {\n    writeln!(s, \"`gtcrn` / `fastenhancer-m` = 真模型\");\n}\n",
        ] {
            assert_eq!(file_problems("a.rs", bad, &names()).len(), 1, "{bad}");
        }
    }

    #[test]
    fn allowed_forms_pass() {
        for ok in [
            // 注释里提到
            "// 命令行写 fastenhancer-m\n/// 文档注释里提到 fastenhancer-m\nconst A: u8 = 1;\n",
            // 更长名字的一部分（另一个模型 / 另一个档）
            "const P: &str = \"fastenhancer-m2\";\nconst Q: &str = \"x_fastenhancer_m\";\n",
            // 放行
            "const P: &str = \"w/fastenhancer_m.onnx\"; // tidy: 不是模型名：权重文件名\n",
            // 单测模块不查
            "fn f() {}\n#[cfg(test)]\nmod t {\n    const A: &str = \"fastenhancer-m\";\n}\n",
            "#[cfg(test)]\n#[allow(dead_code)]\nconst A: &str = \"fastenhancer-m\";\n",
            // 不认的字面量
            "const G: &str = \"gtcrn\";\n",
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
            "const S: &str = \"模型：\\\n    fastenhancer-m\";\n",
            "const S: &str = \"第一行\nfastenhancer-m 第二行\";\n",
            "const S: &str = r#\"\n\"q\" fastenhancer_m\n\"#;\n",
            "const Q: char = '\"'; const S: &str = \"fastenhancer-m\";\n",
            // 转义紧挨着名字：\n \t \u{…} 还原成真字符后再判边界（codex 第 3 轮 #1）
            "const S: &str = \"\\nfastenhancer-m\";\n",
            "const S: &str = \"\\tfastenhancer_m\\t\";\n",
            "const S: &str = \"\\u{66}astenhancer-m\";\n",
            "const S: &str = \"\\u{6_6}astenhancer-m\";\n",
            "#[cfg(test)]\n#[path = \"t.rs\"]\nmod tests;\nfn prod() -> &'static str { \"fastenhancer-m\" }\n",
            "#[cfg(test)]\nmod t {\n    fn x() { if true { } }\n}\nconst B: &str = \"fastenhancer_m\";\n",
            "#[cfg(test)]\nfn helper(\n    a: u8,\n) {\n}\nconst C: &str = \"fastenhancer-m\";\n",
            // 字节字符串（codex #71 第 2 轮 P2 #5）
            "const B: &[u8] = b\"fastenhancer-m\";\n",
        ] {
            assert_eq!(file_problems("a.rs", bad, &names()).len(), 1, "{bad}");
        }
        // 放行要写理由
        let no_reason = "const P: &str = \"fastenhancer_m\"; // tidy: 不是模型名：\n";
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
