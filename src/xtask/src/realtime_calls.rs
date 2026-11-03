//! tidy 的 R3（实时 crate 不做 IO、日志、sleep；规约 §5.1；#71 起按语法树查，012 第三节第 5 条）。
//!
//! | 查什么 | 不查 | 放行 |
//! |---|---|---|
//! | A/B 档 crate 的 `src/` 里，路径（表达式、类型、宏名与宏参数里的 `a::b::c`、`use` 声明）还原 `use` 别名后以 [`BANNED`] 开头 | 模块树判出的测试代码（`ast::modtree`）；注释、字符串 | 整模块 NRT 标注（`#![allow(…)] // NRT: 理由`，见 `tidy.rs`） |
//!
//! 别名还原只看本文件的 `use`：`use std::thread::sleep as nap; nap(d)`、`use std::thread; thread::sleep(d)`、
//! `let f = std::thread::sleep;` 都认。跨文件的 `pub use` 再导出不追（012 第四节）；方法调用按类型判的那部分
//! 交给 clippy 的 `disallowed-methods`（根 `clippy.toml`）。

use proc_macro2::{Spacing, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit::Visit;
use syn::{UseTree, spanned::Spanned};

use crate::ast::modtree::file_test_ranges;

/// 禁用的路径前缀与报错里的叫法。`std::thread::sleep` 与根 `clippy.toml` 的 `disallowed-methods` 一致（测试比对）。
pub const BANNED: [(&[&str], &str); 3] = [
    (&["std", "fs"], "std::fs"),
    (&["log"], "log::"),
    (&["std", "thread", "sleep"], "thread::sleep"),
];

/// `use` 展开：（别名，全路径）；glob 的全路径就是前缀，别名为空。
fn use_paths(t: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<(String, Vec<String>, usize)>) {
    let line = t.span().start().line;
    match t {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            use_paths(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.to_string();
            let mut full = prefix.clone();
            if name != "self" {
                full.push(name.clone());
            }
            let alias = if name == "self" {
                prefix.last().cloned().unwrap_or_default()
            } else {
                name
            };
            out.push((alias, full, line));
        }
        UseTree::Rename(r) => {
            let mut full = prefix.clone();
            if r.ident != "self" {
                full.push(r.ident.to_string());
            }
            out.push((r.rename.to_string(), full, line));
        }
        UseTree::Glob(_) => out.push((String::new(), prefix.clone(), line)),
        UseTree::Group(g) => {
            for it in &g.items {
                use_paths(it, prefix, out);
            }
        }
    }
}

#[derive(Default)]
struct Uses(Vec<(String, Vec<String>, usize)>);

impl<'ast> Visit<'ast> for Uses {
    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        use_paths(&u.tree, &mut Vec::new(), &mut self.0);
    }
}

/// token 里的 `a::b::c` 序列：（起始行，各段）。前导 `::` 去掉。
fn path_seqs(ts: TokenStream, out: &mut Vec<(usize, Vec<String>)>) {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let is_colon2 = |i: usize| {
        matches!((toks.get(i), toks.get(i + 1)),
            (Some(TokenTree::Punct(a)), Some(TokenTree::Punct(b)))
                if a.as_char() == ':' && a.spacing() == Spacing::Joint && b.as_char() == ':')
    };
    let mut i = 0;
    while i < toks.len() {
        match &toks[i] {
            TokenTree::Group(g) => {
                path_seqs(g.stream(), out);
                i += 1;
            }
            TokenTree::Ident(id) => {
                let line = id.span().start().line;
                let mut segs = vec![id.to_string()];
                let mut j = i + 1;
                while is_colon2(j) {
                    let Some(TokenTree::Ident(next)) = toks.get(j + 2) else {
                        break;
                    };
                    segs.push(next.to_string());
                    j += 3;
                }
                out.push((line, segs));
                i = j;
            }
            _ => i += 1,
        }
    }
}

/// 一个文件里的 R3 命中：（行，禁用项叫法）。测试代码不算。文件解析不了 → `Err`。
pub fn hits(rel: &str, src: &str) -> Result<Vec<(usize, &'static str)>, String> {
    let file = crate::ast::parse_src(src)
        .map_err(|e| format!("{rel}:{}：语法树解析失败：{}（R3）", e.line, e.msg))?;
    let tests = file_test_ranges(rel, &file);
    let in_test = |l: usize| tests.iter().any(|(a, b)| (*a..=*b).contains(&l));
    let mut uses = Uses::default();
    uses.visit_file(&file);
    let banned = |segs: &[String]| {
        BANNED
            .iter()
            .find(|(p, _)| segs.len() >= p.len() && segs.iter().zip(p.iter()).all(|(a, b)| a == b))
            .map(|(_, name)| *name)
    };
    let mut out = Vec::new();
    for (_, full, line) in &uses.0 {
        if let Some(b) = banned(full) {
            out.push((*line, b));
        }
    }
    let mut seqs = Vec::new();
    path_seqs(file.to_token_stream(), &mut seqs);
    for (line, segs) in seqs {
        let full: Vec<String> = match uses.0.iter().find(|(a, ..)| !a.is_empty() && *a == segs[0]) {
            Some((_, f, _)) => f.iter().cloned().chain(segs[1..].iter().cloned()).collect(),
            // 没经过别名还原的单个标识符不是路径（局部变量 `log`、字段 `fs`……）
            None if segs.len() == 1 => continue,
            None => segs,
        };
        if let Some(b) = banned(&full) {
            out.push((line, b));
        }
    }
    out.retain(|(l, _)| !in_test(*l));
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

#[cfg(test)]
#[path = "realtime_calls_tests.rs"]
mod tests;
