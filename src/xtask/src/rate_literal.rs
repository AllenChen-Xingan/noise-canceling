//! tidy 的「采样率只写一处」（R-RATE；CLAUDE.md 第五节第 5 条；issue #62；#71 起按语法树查）。
//!
//! | 规则 | 查什么 | 放行 |
//! |---|---|---|
//! | R-RATE | `src/**/*.rs` 的代码里数值等于 16000 或 48000 的字面量（任何写法：`48_000u16`、`0xBB80`、`48000.0_f32`……，宏参数、属性里的也算），以及**两边都是字面量**、算出来等于它们的算式（`3 * 16_000`、`48 * 1000`、`144_000 / 3`）；**测试与示例也一样**，一律引用 `vdn_types::{IO_RATE, MODEL_RATE}` | 见下 |
//!
//! 放行只有两种：
//! - [`RATE_FILE`]：正本本身
//! - 单行：字面量所在行的行尾注释写 `tidy: 非采样率：<理由>`（数值恰好相同、含义不是采样率时；理由不能空）
//!
//! 为什么测试代码也查（大总管 8d 定，2026-11-02）：测试改用常量没有坏处；不依赖 `vdn-types` 的底层 crate
//!（shm、rt-thread、dsp）把它加成 dev 依赖即可。
//!
//! 边界（写明，免得以为查全了）：算式只折叠字面量（`+ - * /`、括号、负号），不追变量与常量；
//! 宏参数里的算式不折叠（宏没展开，只认里面的单个字面量）；注释、字符串、文档注释不算。

use std::fs;
use std::path::Path;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{BinOp, Expr, ExprBinary, Lit, UnOp};

use crate::scan::lex_lines;
use crate::tidy::rs_files;

/// 采样率正本。
pub const RATE_FILE: &str = "src/types/src/rate.rs";
const ALLOW: &str = "tidy: 非采样率：";
const RATES: [f64; 2] = [16000.0, 48000.0]; // tidy: 非采样率：本规则要认的两个数本身

fn is_rate(v: f64) -> bool {
    RATES.iter().any(|r| (v - r).abs() < 1e-9)
}

/// 字面量的数值（整数、浮点；别的字面量 → None）。
fn lit_value(l: &Lit) -> Option<f64> {
    match l {
        Lit::Int(i) => i.base10_parse::<u128>().ok().map(|v| v as f64),
        Lit::Float(f) => f.base10_parse::<f64>().ok(),
        _ => None,
    }
}

/// token 里所有数值字面量（含宏参数、属性里的）；文档注释跳过。
fn token_hits(ts: TokenStream, out: &mut Vec<Span>) {
    let toks: Vec<TokenTree> = ts.into_iter().collect();
    let mut i = 0;
    while i < toks.len() {
        if let TokenTree::Punct(p) = &toks[i]
            && p.as_char() == '#'
        {
            let bang = matches!(toks.get(i + 1), Some(TokenTree::Punct(q)) if q.as_char() == '!');
            let j = i + 1 + usize::from(bang);
            if let Some(TokenTree::Group(g)) = toks.get(j)
                && matches!(g.stream().into_iter().next(), Some(TokenTree::Ident(id)) if id == "doc")
            {
                i = j + 1;
                continue;
            }
        }
        match &toks[i] {
            TokenTree::Group(g) => token_hits(g.stream(), out),
            TokenTree::Literal(l) if lit_value(&Lit::new(l.clone())).is_some_and(is_rate) => {
                out.push(l.span());
            }
            _ => {}
        }
        i += 1;
    }
}

fn fold_binary(b: &ExprBinary) -> Option<f64> {
    let (l, r) = (fold(&b.left)?, fold(&b.right)?);
    match b.op {
        BinOp::Add(_) => Some(l + r),
        BinOp::Sub(_) => Some(l - r),
        BinOp::Mul(_) => Some(l * r),
        BinOp::Div(_) if r != 0.0 => Some(l / r),
        _ => None,
    }
}

/// 只由字面量组成的算式的值。
fn fold(e: &Expr) -> Option<f64> {
    match e {
        Expr::Lit(l) => lit_value(&l.lit),
        Expr::Paren(p) => fold(&p.expr),
        Expr::Group(g) => fold(&g.expr),
        Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => fold(&u.expr).map(|v| -v),
        Expr::Binary(b) => fold_binary(b),
        _ => None,
    }
}

/// 算出来是采样率的算式。
#[derive(Default)]
struct Folded(Vec<Span>);

impl<'ast> Visit<'ast> for Folded {
    fn visit_expr_binary(&mut self, b: &'ast ExprBinary) {
        if fold_binary(b).is_some_and(is_rate) {
            self.0.push(b.span());
        }
        syn::visit::visit_expr_binary(self, b);
    }
}

/// 一个文件里命中的行（从 1 起，去重排序）。
pub fn rate_lines(file: &syn::File) -> Vec<usize> {
    let mut spans = Vec::new();
    token_hits(file.to_token_stream(), &mut spans);
    let mut f = Folded::default();
    f.visit_file(file);
    spans.extend(f.0);
    let mut lines: Vec<usize> = spans.iter().map(|s| s.start().line).collect();
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// 放行标记后面有没有写理由。
fn has_allow_reason(comment: &str) -> bool {
    comment
        .split_once(ALLOW)
        .is_some_and(|(_, r)| !r.trim().is_empty())
}

/// 一个文件的问题。文件解析不了也报（不静默跳过）。
pub fn rate_problems(rel: &str, src: &str) -> Vec<String> {
    if rel == RATE_FILE {
        return Vec::new();
    }
    let file = match crate::ast::parse_src(src) {
        Ok(f) => f,
        Err(e) => {
            return vec![format!(
                "{rel}:{}：语法树解析失败：{}（R-RATE）",
                e.line, e.msg
            )];
        }
    };
    let raw: Vec<&str> = src.lines().collect();
    let lexed = lex_lines(src);
    rate_lines(&file)
        .into_iter()
        .filter(|l| !lexed.get(l - 1).is_some_and(|(_, c)| has_allow_reason(c)))
        .map(|l| {
            let text = raw.get(l - 1).map_or("", |s| s.trim());
            format!(
                "{rel}:{l}：采样率字面量（R-RATE）——改用 vdn_types::IO_RATE / MODEL_RATE（正本 {RATE_FILE}）；不是采样率就在行尾写 `// {ALLOW}<理由>`：{text}"
            )
        })
        .collect()
}

pub fn check(root: &Path, problems: &mut Vec<String>) {
    if !root.join(RATE_FILE).exists() {
        problems.push(format!("{RATE_FILE} 不存在：采样率正本（R-RATE）"));
    }
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    for f in &all {
        let Ok(rel) = f.strip_prefix(root).map(|p| p.display().to_string()) else {
            continue;
        };
        match fs::read_to_string(f) {
            Ok(src) => problems.extend(rate_problems(&rel, &src)),
            Err(e) => problems.push(format!("读 {rel}：{e}（R-RATE）")),
        }
    }
}

#[cfg(test)]
#[path = "rate_literal_tests.rs"]
mod tests;
