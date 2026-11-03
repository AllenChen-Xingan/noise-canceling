//! tidy 规则 R-MIRROR：不许有镜像副本（CLAUDE.md 第五节第 5 条「唯一真相源」；issue #56；#71 起按语法树查，
//! 012 第三节第 4 条）。不依赖任何 vdn crate。
//!
//! | 查什么 | 放行写法 |
//! |---|---|
//! | ① `src/` 下不许有名叫 `mirror.rs` 的文件 | — |
//! | ② `vdn-types` 之外不许定义与 `vdn-types` 公开类型**同名**的 `struct` / `enum` / `trait` / `type`（宏展开出的不认） | 定义行或紧邻上方连续的注释 / 属性行的注释里写 `tidy: 同名不同物：<理由>` |
//! | ③ **改了名但结构相同**：字段（或枚举变体、trait 方法）≥ [`MIN_FIELDS`] 个，且名字与类型（去掉路径前缀）和 `vdn-types` 某个公开类型完全一样 | 同上 |
//!
//! 结构指纹：结构体按「字段名: 类型」（元组结构体按位置）排序后拼接；枚举按变体名与字段；trait 按方法名。
//! 类型去掉路径前缀（`vdn_types::Confidence` 与 `Confidence` 算一样）。门槛 3 由大总管 8d 定（012 第九节）：
//! 再小会把 `struct Pair(f32, f32)` 这类通用小结构误报。

use std::path::Path;

use quote::ToTokens;
use syn::visit::Visit;
use syn::{Fields, Item, Visibility};

use crate::scan::{has_reason, lex_lines, src_files};
use crate::tidy::rs_files;

const TYPES_SRC: &str = "src/types/src";
const MIRROR_ALLOW: &str = "tidy: 同名不同物：";
/// 改名副本的门槛：字段 / 变体 / 方法数不少于它才比结构。
pub const MIN_FIELDS: usize = 3;

/// 一个类型的（名字，结构指纹，字段数）。指纹为空 = 不比结构（类型别名）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub name: String,
    pub print: String,
    pub fields: usize,
}

/// 类型的文本，去掉路径前缀：`vdn_types::frame::Confidence` → `Confidence`，`Vec<crate::X>` → `Vec < X >`。
fn norm_ty<T: ToTokens>(t: &T) -> String {
    let s = t.to_token_stream().to_string();
    let toks: Vec<&str> = s.split_whitespace().collect();
    toks.iter()
        .enumerate()
        .filter(|(i, tk)| **tk != "::" && toks.get(i + 1) != Some(&"::"))
        .map(|(_, tk)| *tk)
        .collect::<Vec<_>>()
        .join(" ")
}

fn fields_print(f: &Fields) -> (Vec<String>, usize) {
    let mut v: Vec<String> = f
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let n = x.ident.as_ref().map_or(i.to_string(), ToString::to_string);
            format!("{n}:{}", norm_ty(&x.ty))
        })
        .collect();
    if matches!(f, Fields::Named(_)) {
        v.sort();
    }
    let n = v.len();
    (v, n)
}

/// 一个条目的结构；不是 struct / enum / trait / type 的 → None。
pub fn shape(it: &Item) -> Option<(Shape, &syn::Ident, &Visibility)> {
    match it {
        Item::Struct(s) => {
            let (v, n) = fields_print(&s.fields);
            let print = format!("struct {}", v.join(","));
            Some((
                Shape {
                    name: s.ident.to_string(),
                    print,
                    fields: n,
                },
                &s.ident,
                &s.vis,
            ))
        }
        Item::Enum(e) => {
            let mut v: Vec<String> = e
                .variants
                .iter()
                .map(|x| format!("{}({})", x.ident, fields_print(&x.fields).0.join(",")))
                .collect();
            v.sort();
            let n = v.len();
            Some((
                Shape {
                    name: e.ident.to_string(),
                    print: format!("enum {}", v.join(",")),
                    fields: n,
                },
                &e.ident,
                &e.vis,
            ))
        }
        Item::Trait(t) => {
            let mut v: Vec<String> = t
                .items
                .iter()
                .filter_map(|x| match x {
                    syn::TraitItem::Fn(f) => Some(f.sig.ident.to_string()),
                    _ => None,
                })
                .collect();
            v.sort();
            let n = v.len();
            Some((
                Shape {
                    name: t.ident.to_string(),
                    print: format!("trait {}", v.join(",")),
                    fields: n,
                },
                &t.ident,
                &t.vis,
            ))
        }
        Item::Type(t) => Some((
            Shape {
                name: t.ident.to_string(),
                print: String::new(),
                fields: 0,
            },
            &t.ident,
            &t.vis,
        )),
        _ => None,
    }
}

/// 文件里（含内联模块里）全部类型定义：（结构，名字所在行）。
fn defs(file: &syn::File, public_only: bool) -> Vec<(Shape, usize)> {
    #[derive(Default)]
    struct V(Vec<(Shape, usize)>, bool);
    impl<'ast> Visit<'ast> for V {
        fn visit_item(&mut self, it: &'ast Item) {
            if let Some((s, id, vis)) = shape(it)
                && (!self.1 || matches!(vis, Visibility::Public(_)))
                && !(self.1 && matches!(it, Item::Type(_)))
            {
                self.0.push((s, id.span().start().line));
            }
            syn::visit::visit_item(self, it);
        }
    }
    let mut v = V(Vec::new(), public_only);
    v.visit_file(file);
    v.0
}

/// `vdn-types` 的公开 `struct` / `enum` / `trait`。解析失败 → `Err`。
pub fn types_shapes(root: &Path) -> Result<Vec<Shape>, String> {
    let mut files = Vec::new();
    rs_files(&root.join(TYPES_SRC), &mut files);
    let mut out: Vec<Shape> = Vec::new();
    for f in files {
        for (s, _) in defs(&*crate::ast::parse_file(&f)?, true) {
            if !out.iter().any(|o| o.name == s.name) {
                out.push(s);
            }
        }
    }
    Ok(out)
}

pub fn check(root: &Path, problems: &mut Vec<String>) {
    let shapes = match types_shapes(root) {
        Ok(s) => s,
        Err(e) => {
            problems.push(format!("{e}（R-MIRROR）"));
            return;
        }
    };
    for (rel, src) in src_files(root) {
        if !rel.starts_with(TYPES_SRC) {
            problems.extend(mirror_problems(&rel, &src, &shapes));
        }
    }
}

/// 放行：名字所在行，或紧邻上方连续的注释 / 属性行的注释里写了理由。
fn allowed(lexed: &[(String, String)], line: usize) -> bool {
    let mut i = line.saturating_sub(1);
    if lexed
        .get(i)
        .is_some_and(|(_, c)| has_reason(c, MIRROR_ALLOW))
    {
        return true;
    }
    while i > 0 {
        i -= 1;
        let Some((c, cm)) = lexed.get(i) else { break };
        let t = c.trim();
        if !((t.is_empty() && !cm.trim().is_empty()) || t.starts_with("#[")) {
            break;
        }
        if has_reason(cm, MIRROR_ALLOW) {
            return true;
        }
    }
    false
}

pub fn mirror_problems(rel: &str, src: &str, shapes: &[Shape]) -> Vec<String> {
    let mut out = Vec::new();
    if Path::new(rel).file_name().is_some_and(|n| n == "mirror.rs") {
        out.push(format!(
            "{rel}：镜像文件不许存在，直接依赖 vdn-types（R-MIRROR）"
        ));
    }
    let file = match crate::ast::parse_src(src) {
        Ok(f) => f,
        Err(e) => {
            out.push(format!(
                "{rel}:{}：语法树解析失败：{}（R-MIRROR）",
                e.line, e.msg
            ));
            return out;
        }
    };
    let lexed = lex_lines(src);
    for (s, line) in defs(&file, false) {
        if allowed(&lexed, line) {
            continue;
        }
        if shapes.iter().any(|t| t.name == s.name) {
            out.push(format!(
                "{rel}:{line}：定义了与 vdn-types 同名的类型 `{}`，像镜像副本；改用 vdn_types::{}，或注释写「{MIRROR_ALLOW}<理由>」（R-MIRROR）",
                s.name, s.name
            ));
        } else if s.fields >= MIN_FIELDS
            && let Some(t) = shapes
                .iter()
                .find(|t| !t.print.is_empty() && t.print == s.print)
        {
            out.push(format!(
                "{rel}:{line}：类型 `{}` 与 vdn-types 的 `{}` 结构完全相同，像改了名的手抄副本；改用 vdn_types::{}，或注释写「{MIRROR_ALLOW}<理由>」（R-MIRROR）",
                s.name, t.name, t.name
            ));
        }
    }
    out
}

#[cfg(test)]
#[path = "no_mirror_tests.rs"]
mod tests;
