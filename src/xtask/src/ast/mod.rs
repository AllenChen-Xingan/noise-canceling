//! tidy 的语法树工具（issue #71，设计 `docs/plan/012_tidy-syn.md`）：用 `syn` 解析 Rust 源码，
//! 替代逐行文本匹配。xtask 仍不依赖任何 vdn crate。
//!
//! - [`parse_file`]：读文件 → 语法树；解析失败报错，不静默跳过
//! - [`cfg_is_test`]：属性是不是「只在测试编译」
//! - [`modtree`]：从编译根建模块树，判哪些文件、哪些行是测试代码
//! - [`str_literals`]：文件里全部字符串字面量（含宏与属性里的；文档注释不算）

pub mod modtree;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Attribute, Meta};

/// 解析失败：行号与说明。
#[derive(Clone, Debug)]
pub struct ParseErr {
    pub line: usize,
    pub msg: String,
}

/// 一份源码的解析结果。
type Parsed = Result<Rc<syn::File>, ParseErr>;

thread_local! {
    /// 同一次 tidy 里每份源码只解析一次（各条规则共用；按原文认）。
    static CACHE: RefCell<HashMap<String, Parsed>> =
        RefCell::new(HashMap::new());
}

/// 解析一份源码（带缓存）。
pub fn parse_src(src: &str) -> Result<Rc<syn::File>, ParseErr> {
    // 以原文为键：哈希相等不等于源码相等（codex #71 第 1 轮 P2 #8）
    if let Some(r) = CACHE.with(|c| c.borrow().get(src).cloned()) {
        return r;
    }
    let r = syn::parse_file(src).map(Rc::new).map_err(|e| ParseErr {
        line: e.span().start().line,
        msg: e.to_string(),
    });
    CACHE.with(|c| c.borrow_mut().insert(src.to_string(), r.clone()));
    r
}

/// 读并解析一个文件（带缓存）。
pub fn parse_file(path: &Path) -> Result<Rc<syn::File>, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("读 {}：{e}", path.display()))?;
    parse_src(&src).map_err(|e| format!("{}:{}：语法树解析失败：{}", path.display(), e.line, e.msg))
}

/// 一个 `cfg(...)` 谓词是不是「只在测试时为真」：`test`，或 `all(…)` 里有一项是这样。
/// `any(test, …)`、`not(…)`、feature 门控都按「可能进生产」算（宁可多查，012 第四节）。
fn pred_is_test(m: &Meta) -> bool {
    match m {
        Meta::Path(p) => p.is_ident("test"),
        Meta::List(l) if l.path.is_ident("all") => l
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
            .is_ok_and(|items| items.iter().any(pred_is_test)),
        _ => false,
    }
}

/// 这组属性是不是让该项只在测试编译：`#[cfg(test)]` / `#[cfg(all(test, …))]`，或 `#[test]`。
pub fn cfg_is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        if a.path().is_ident("test") {
            return true;
        }
        a.path().is_ident("cfg") && a.parse_args::<Meta>().is_ok_and(|m| pred_is_test(&m))
    })
}

/// `#[path = "…"]` 的值。
pub fn path_attr(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|a| match &a.meta {
        Meta::NameValue(nv) if nv.path.is_ident("path") => match &nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Some(s.value()),
            _ => None,
        },
        _ => None,
    })
}

/// `#[cfg_attr(谓词, …, path = "…")]` 里的路径：只在某些构建里生效，建树时也当成可能的引用（codex #71 第 3 轮 P1 #3）。
pub fn cfg_attr_paths(attrs: &[Attribute]) -> Vec<String> {
    let mut out = Vec::new();
    for a in attrs.iter().filter(|a| a.path().is_ident("cfg_attr")) {
        let Ok(items) = a
            .parse_args_with(syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated)
        else {
            continue;
        };
        for m in items.iter().skip(1) {
            if let Meta::NameValue(nv) = m
                && nv.path.is_ident("path")
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
            {
                out.push(s.value());
            }
        }
    }
    out
}

/// 文件里全部字符串字面量：（起始行，解码后的值）。按 token 走，宏参数、属性里的也算；
/// 文档注释（`///` 与 `//!`，在语法树里是 `#[doc = "…"]`）不算——它们是注释。
pub fn str_literals(file: &syn::File) -> Vec<(modtree::Pos, String)> {
    fn walk(ts: TokenStream, out: &mut Vec<(modtree::Pos, String)>) {
        let toks: Vec<TokenTree> = ts.into_iter().collect();
        let mut i = 0;
        while i < toks.len() {
            // `#` [`!`] `[doc …]`：整段跳过
            if let TokenTree::Punct(p) = &toks[i]
                && p.as_char() == '#'
            {
                let j = i
                    + 1
                    + usize::from(
                        matches!(toks.get(i + 1), Some(TokenTree::Punct(q)) if q.as_char() == '!'),
                    );
                if let Some(TokenTree::Group(g)) = toks.get(j)
                    && g.delimiter() == proc_macro2::Delimiter::Bracket
                    && matches!(g.stream().into_iter().next(), Some(TokenTree::Ident(id)) if id == "doc")
                {
                    i = j + 1;
                    continue;
                }
            }
            match &toks[i] {
                TokenTree::Group(g) => walk(g.stream(), out),
                TokenTree::Literal(l) => {
                    // 字节字符串也算（`b"fastenhancer-m"`；codex #71 第 2 轮 P2 #5）
                    match syn::Lit::new(l.clone()) {
                        syn::Lit::Str(s) => out.push((modtree::pos(l.span()), s.value())),
                        syn::Lit::ByteStr(s) => out.push((
                            modtree::pos(l.span()),
                            String::from_utf8_lossy(&s.value()).into_owned(),
                        )),
                        _ => {}
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    let mut out = Vec::new();
    walk(file.to_token_stream(), &mut out);
    out
}
