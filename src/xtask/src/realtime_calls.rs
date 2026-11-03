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
use syn::ext::IdentExt;
use syn::visit::Visit;
use syn::{UseTree, spanned::Spanned};

use crate::ast::modtree::{self, Pos, file_test_ranges, in_ranges, pos};

/// 禁用的路径前缀与报错里的叫法。`std::thread::sleep` 与根 `clippy.toml` 的 `disallowed-methods` 一致（测试比对）。
pub const BANNED: [(&[&str], &str); 3] = [
    (&["std", "fs"], "std::fs"),
    (&["log"], "log::"),
    (&["std", "thread", "sleep"], "thread::sleep"),
];

/// `use` 展开：（别名，全路径）；glob 的全路径就是前缀，别名为空。
fn use_paths(t: &UseTree, prefix: &mut Vec<String>, out: &mut Vec<(String, Vec<String>, Pos)>) {
    let line = pos(t.span());
    match t {
        UseTree::Path(p) => {
            prefix.push(p.ident.unraw().to_string());
            use_paths(&p.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.unraw().to_string();
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
                full.push(r.ident.unraw().to_string());
            }
            out.push((r.rename.unraw().to_string(), full, line));
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
struct Uses(Vec<(String, Vec<String>, Pos)>);

impl<'ast> Visit<'ast> for Uses {
    fn visit_item_use(&mut self, u: &'ast syn::ItemUse) {
        use_paths(&u.tree, &mut Vec::new(), &mut self.0);
    }
}

/// token 里的 `a::b::c` 序列：（起点，各段）。前导 `::` 去掉。
fn path_seqs(ts: TokenStream, out: &mut Vec<(Pos, Vec<String>)>) {
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
                let line = pos(id.span());
                // 各段去掉 `r#`：`r#std::fs` 与 `std::fs` 等价（codex #71 第 2 轮 P2 #6）
                let mut segs = vec![id.unraw().to_string()];
                let mut j = i + 1;
                while is_colon2(j) {
                    let Some(TokenTree::Ident(next)) = toks.get(j + 2) else {
                        break;
                    };
                    segs.push(next.unraw().to_string());
                    j += 3;
                }
                out.push((line, segs));
                i = j;
            }
            _ => i += 1,
        }
    }
}

fn banned(segs: &[String]) -> Option<&'static str> {
    BANNED
        .iter()
        .find(|(p, _)| segs.len() >= p.len() && segs.iter().zip(p.iter()).all(|(a, b)| a == b))
        .map(|(_, name)| *name)
}

/// 一个文件里的 R3 命中：（行，禁用项叫法）。测试代码不算。文件解析不了 → `Err`。
///
/// 别名还原只用**非测试代码**里的 `use`；路径不管还原前还是还原后，只要有一个以禁用前缀开头就算
///（测试模块里 `use crate::fake as std;` 不能把生产代码的 `std::fs` 改写掉，codex #71 第 1 轮 P1 #4）。
pub fn hits(rel: &str, src: &str) -> Result<Vec<(usize, &'static str)>, String> {
    let file = crate::ast::parse_src(src)
        .map_err(|e| format!("{rel}:{}：语法树解析失败：{}（R3）", e.line, e.msg))?;
    let tests = file_test_ranges(rel, &file);
    let mut uses = Uses::default();
    uses.visit_file(&file);
    let aliases: Vec<&(String, Vec<String>, Pos)> = uses
        .0
        .iter()
        .filter(|(a, _, p)| !a.is_empty() && !in_ranges(&tests, *p))
        .collect();
    let mut out: Vec<(Pos, &'static str)> = Vec::new();
    for (_, full, p) in &uses.0 {
        if let Some(b) = banned(full) {
            out.push((*p, b));
        }
    }
    let mut seqs = Vec::new();
    path_seqs(file.to_token_stream(), &mut seqs);
    for (p, segs) in seqs {
        if segs.len() > 1
            && let Some(b) = banned(&segs)
        {
            out.push((p, b));
            continue;
        }
        // 没经过别名还原的单个标识符不是路径（局部变量 `log`、字段 `fs`……）。
        // 同名别名可能在不同作用域指向不同路径：不解析作用域，所有候选里有一个命中就报（codex #71 第 2 轮 P1 #2）
        for (_, f, _) in aliases.iter().filter(|(a, ..)| *a == segs[0]) {
            let full: Vec<String> = f.iter().cloned().chain(segs[1..].iter().cloned()).collect();
            if let Some(b) = banned(&full) {
                out.push((p, b));
                break;
            }
        }
    }
    let mut lines: Vec<(usize, &'static str)> = out
        .into_iter()
        .filter(|(p, _)| !in_ranges(&tests, *p))
        .map(|(p, b)| (p.0, b))
        .collect();
    lines.sort_unstable();
    lines.dedup();
    Ok(lines)
}

/// tidy 的入口：整份测试文件（模块树判出的）跳过，其余按 [`hits`] 报（codex #71 第 1 轮 P2 #5）。
pub fn check_file(root: &std::path::Path, rel: &str, src: &str) -> Vec<String> {
    if modtree::build(root).is_test_file(rel) {
        return Vec::new();
    }
    match hits(rel, src) {
        Ok(h) => h
            .into_iter()
            .map(|(l, bad)| format!("{rel}:{l}：实时 crate 里出现 {bad}（R3）"))
            .collect(),
        Err(e) => vec![e],
    }
}

#[cfg(test)]
#[path = "realtime_calls_tests.rs"]
mod tests;
