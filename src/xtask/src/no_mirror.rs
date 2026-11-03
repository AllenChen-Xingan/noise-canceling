//! tidy 规则 R-MIRROR：不许有镜像副本（CLAUDE.md 第五节第 5 条「唯一真相源」；issue #56）。纯文本检查，不依赖任何 vdn crate。
//!
//! | 查什么 | 放行写法 |
//! |---|---|
//! | ① `src/` 下不许有名叫 `mirror.rs` 的文件；② `vdn-types` 之外的代码不许定义与 `vdn-types` 公开类型**同名**的 `struct` / `enum` / `trait` / `type`（镜像副本的特征） | 定义行或紧邻上方的注释里写 `tidy: 同名不同物：<理由>` |
//!
//! 边界：同名类型只比名字不比字段；改名后的手抄副本查不到。

use std::fs;
use std::path::Path;

use crate::scan::{code_lines, has_reason, lex_lines, src_files};
use crate::tidy::rs_files;

const TYPES_SRC: &str = "src/types/src";
const MIRROR_ALLOW: &str = "tidy: 同名不同物：";

pub fn check(root: &Path, problems: &mut Vec<String>) {
    let mut type_files = Vec::new();
    rs_files(&root.join(TYPES_SRC), &mut type_files);
    let type_srcs: Vec<String> = type_files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .collect();
    let names = types_names(type_srcs.iter().map(String::as_str));
    for (rel, src) in src_files(root) {
        if !rel.starts_with(TYPES_SRC) {
            problems.extend(mirror_problems(&rel, &src, &names));
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── R-MIRROR ──
    #[test]
    fn mirror_file_and_same_name_types_are_flagged() {
        let names = types_names(
            ["pub struct FrameCtx {\n}\npub trait Stage: Send {}\npub enum UserScene {}\n"]
                .into_iter(),
        );
        assert_eq!(names, ["FrameCtx", "Stage", "UserScene"]);
        assert_eq!(
            mirror_problems("src/polish/src/mirror.rs", "", &names).len(),
            1
        );
        for bad in [
            "pub struct FrameCtx {\n",
            "pub(crate) trait Stage: Send {}\n",
            "#[derive(Clone)]\nenum UserScene { A }\n",
            "pub type FrameCtx = u8;\n",
        ] {
            assert_eq!(
                mirror_problems("src/x/src/lib.rs", bad, &names).len(),
                1,
                "{bad}"
            );
        }
    }

    #[test]
    fn distinct_names_and_reasoned_same_names_pass() {
        let names = vec!["Hysteresis".to_string(), "Stage".to_string()];
        for good in [
            "pub struct VadHysteresis {}\n",
            "use vdn_types::Stage;\nimpl Stage for X {}\n",
            "/// VAD 概率的滞回\n// tidy: 同名不同物：这是 VAD 概率滞回，vdn-types 的是场景滞回\n#[derive(Clone, Copy)]\npub struct Hysteresis {}\n",
            "pub struct Hysteresis {} // tidy: 同名不同物：VAD 概率滞回\n",
            "// pub struct Stage {}\n",
            "/* pub struct Stage {} */\n",
            "let s = \"pub struct Stage {}\";\n",
            "/* tidy: 同名不同物：块注释里写理由也算 */\npub struct Hysteresis {}\n",
        ] {
            assert!(
                mirror_problems("src/x/src/lib.rs", good, &names).is_empty(),
                "{good}"
            );
        }
        // 只写标记、不写理由不算；写在字符串里也不算
        for bad in [
            "// tidy: 同名不同物：\npub struct Hysteresis {}\n",
            "pub struct Hysteresis {} const S: &str = \"tidy: 同名不同物：假的\";\n",
        ] {
            assert_eq!(
                mirror_problems("src/x/src/lib.rs", bad, &names).len(),
                1,
                "{bad}"
            );
        }
    }

    /// 整个仓库当前过这条规则（新加的违规会在这里和 `tidy` 里同时红）。
    #[test]
    fn repo_passes() {
        let mut p = Vec::new();
        check(&crate::root(), &mut p);
        assert!(p.is_empty(), "{p:#?}");
    }
}
