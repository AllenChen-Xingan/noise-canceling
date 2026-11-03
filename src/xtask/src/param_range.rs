//! tidy 规则 R-PARAM：参数范围（与默认值）只在参数描述表里写（CLAUDE.md 第五节第 5 条；issue #56；
//! #71 起按语法树查，012 第三节第 3 条）。不依赖任何 vdn crate。
//!
//! | 查什么 | 放行写法 |
//! |---|---|
//! | 参数描述表（[`DESC_DIR`]）之外，**同一条语句**（带块的 `for` / `if` / `while` / `match` 只算头部；`match` 分支、常量初始值各自算一条）里既出现某个参数名（键名末段如 `floor_db`，或驼峰形 `FloorDb`），又出现数字写的范围：`a..b` / `a..=b` · `x.clamp(a, b)` · `.max(a).min(b)` / `.min(a).max(b)` · 分支值 `=> (a, b)`。rustfmt 拆成几行也查得到 | 范围起止行的行尾写 `// tidy: 非参数范围：<理由>` |
//! | **非测试代码**里结构体字面量的参数字段写的数**等于该参数在描述表里的默认值**（`Gate { floor_db: -12.0, .. }`，应写 `FLOOR_DB.default`）：默认值的正本也是描述表。和默认值不同的数是有意的预设（修音预设、实验的激进版），不报 | 本意是预设、碰巧等于默认：行尾写 `// tidy: 预设值，不随默认变：<理由和出处>` |
//!
//! 边界（写明，免得以为查全了）：范围先赋给一个不带参数名的变量、在别的语句里再用，查不到；宏参数只在能解析成
//! 逗号分隔的表达式时查（`assert!`、`vec!` 这类；`println!("{}", …)` 的格式串也算一个表达式）；参数名按描述表的键名末段认，同名不同义的标识符也会被查（宁可多报，用放行写法说明）。

use std::collections::BTreeSet;
use std::path::Path;
use std::rc::Rc;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, Lit, RangeLimits, UnOp};

use crate::ast::modtree::{self, Pos, file_test_ranges, in_ranges, pos};
use crate::scan::{has_reason, lex_lines, src_files};
use crate::tidy::rs_files;

/// 描述表正本所在目录（相对 workspace 根）：`mod.rs` 加按参数所属分的几个文件。
pub const DESC_DIR: &str = "src/types/src/desc/";
const RANGE_ALLOW: &str = "tidy: 非参数范围：";
/// 默认值字面量的放行：本意是预设值、只是碰巧等于默认（大总管 8d 定，2026-11-03）
const PRESET_ALLOW: &str = "tidy: 预设值，不随默认变：";

pub fn check(root: &Path, problems: &mut Vec<String>) {
    let srcs = match read_desc(root) {
        Ok(s) => s,
        Err(e) => {
            problems.push(format!("{DESC_DIR}：{e}（R-PARAM）"));
            return;
        }
    };
    let (keys, consts) = desc_keys(&srcs);
    if keys.is_empty() || keys.len() != consts {
        problems.push(format!(
            "{DESC_DIR}：解析出 {} 个键名，但有 {consts} 个 ParamDesc 常量；解析失败的条目会让 R-PARAM 漏查（R-PARAM）",
            keys.len()
        ));
    }
    let tokens = param_tokens(&keys);
    let defaults = desc_defaults(&srcs);
    let scope = modtree::build(root);
    for (rel, src) in src_files(root) {
        if !rel.starts_with(DESC_DIR) {
            let defs = if scope.is_test_file(&rel) {
                &[][..]
            } else {
                &defaults[..]
            };
            problems.extend(range_problems(&rel, &src, &tokens, defs));
        }
    }
}

/// 描述表目录下全部 `.rs` 的语法树；目录不存在、没有文件、解析失败 → `Err`。
pub fn read_desc(root: &Path) -> Result<Vec<Rc<syn::File>>, String> {
    let mut files = Vec::new();
    rs_files(&root.join(DESC_DIR), &mut files);
    files.sort();
    if files.is_empty() {
        return Err("不存在或没有 .rs：参数描述表是参数范围的正本".into());
    }
    files.iter().map(|p| crate::ast::parse_file(p)).collect()
}

/// 描述表里每条的键名（`p(<编号>, "<键名>", …)` 的第二个参数）与 `ParamDesc` 常量的个数（用来核对一条不漏）。
pub fn desc_keys(files: &[Rc<syn::File>]) -> (Vec<String>, usize) {
    #[derive(Default)]
    struct V(Vec<String>, usize);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
            if let Expr::Path(f) = &*c.func
                && f.path.segments.last().is_some_and(|s| s.ident == "p")
                && matches!(c.args.first(), Some(Expr::Lit(l)) if matches!(l.lit, Lit::Int(_)))
                && let Some(Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(k), ..
                })) = c.args.iter().nth(1)
            {
                self.0.push(k.value());
            }
            syn::visit::visit_expr_call(self, c);
        }
        fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
            if matches!(&*c.ty, syn::Type::Path(t) if t.path.segments.last().is_some_and(|s| s.ident == "ParamDesc"))
            {
                self.1 += 1;
            }
            syn::visit::visit_item_const(self, c);
        }
    }
    let mut v = V::default();
    for f in files {
        v.visit_file(f);
    }
    (v.0, v.1)
}

/// 有符号的数值（字面量，可带负号、括号）。
fn num_value(e: &Expr) -> Option<f64> {
    match e {
        Expr::Lit(l) => match &l.lit {
            Lit::Int(i) => i.base10_parse::<f64>().ok(),
            Lit::Float(f) => f.base10_parse::<f64>().ok(),
            _ => None,
        },
        Expr::Paren(p) => num_value(&p.expr),
        Expr::Group(g) => num_value(&g.expr),
        Expr::Unary(u) if matches!(u.op, UnOp::Neg(_)) => num_value(&u.expr).map(|v| -v),
        _ => None,
    }
}

/// 描述表里每条的（键名末段，默认值）：`p(编号, 键名, 单位, 最小, 最大, 默认, 说明)` 的第 6 个参数。
pub fn desc_defaults(files: &[Rc<syn::File>]) -> Vec<(String, f64)> {
    #[derive(Default)]
    struct V(Vec<(String, f64)>);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
            if let Expr::Path(f) = &*c.func
                && f.path.segments.last().is_some_and(|s| s.ident == "p")
                && let Some(Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(k), ..
                })) = c.args.iter().nth(1)
                && let Some(d) = c.args.iter().nth(5).and_then(num_value)
            {
                let key = k.value();
                self.0
                    .push((key.rsplit('.').next().unwrap_or(&key).to_string(), d));
            }
            syn::visit::visit_expr_call(self, c);
        }
    }
    let mut v = V::default();
    for f in files {
        v.visit_file(f);
    }
    v.0
}

/// 参数名在代码里的写法：键名末段（`floor_db`）与驼峰形（`FloorDb`，`ParamField` 的变体名）。
pub fn param_tokens(keys: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for k in keys {
        let short = k.rsplit('.').next().unwrap_or(k).to_string();
        let camel: String = short
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                    .unwrap_or_default()
            })
            .collect();
        for t in [short, camel] {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    out
}

/// 字面量（可带负号、括号）是不是数字。
fn is_num(e: &Expr) -> bool {
    match e {
        Expr::Lit(l) => matches!(l.lit, Lit::Int(_) | Lit::Float(_)),
        Expr::Paren(p) => is_num(&p.expr),
        Expr::Group(g) => is_num(&g.expr),
        Expr::Unary(u) => matches!(u.op, UnOp::Neg(_)) && is_num(&u.expr),
        _ => false,
    }
}

/// 数字写的范围（四种写法之一）。
pub fn is_literal_range(e: &Expr) -> bool {
    match e {
        Expr::Range(r) => {
            r.start.as_deref().is_some_and(is_num)
                && r.end.as_deref().is_some_and(is_num)
                && matches!(r.limits, RangeLimits::HalfOpen(_) | RangeLimits::Closed(_))
        }
        Expr::MethodCall(m) => {
            let name = m.method.to_string();
            if name == "clamp" {
                return m.args.len() == 2 && m.args.iter().all(is_num);
            }
            let other = match name.as_str() {
                "min" => "max",
                "max" => "min",
                _ => return false,
            };
            m.args.len() == 1
                && m.args.iter().all(is_num)
                && matches!(&*m.receiver, Expr::MethodCall(r)
                    if r.method == other && r.args.len() == 1 && r.args.iter().all(is_num))
        }
        _ => false,
    }
}

fn idents(ts: TokenStream, out: &mut BTreeSet<String>) {
    for t in ts {
        match t {
            TokenTree::Ident(i) => {
                out.insert(i.to_string());
            }
            TokenTree::Group(g) => idents(g.stream(), out),
            _ => {}
        }
    }
}

fn ids_of<T: ToTokens>(x: &T) -> BTreeSet<String> {
    let mut s = BTreeSet::new();
    idents(x.to_token_stream(), &mut s);
    s
}

/// 一条语句里算「同一条」的部分：带块的表达式只取头部。
fn stmt_context(s: &syn::Stmt) -> BTreeSet<String> {
    let head = |e: &Expr| -> Option<BTreeSet<String>> {
        match e {
            Expr::ForLoop(f) => {
                let mut a = ids_of(&f.pat);
                a.extend(ids_of(&f.expr));
                Some(a)
            }
            Expr::If(i) => Some(ids_of(&i.cond)),
            Expr::While(w) => Some(ids_of(&w.cond)),
            Expr::Match(m) => Some(ids_of(&m.expr)),
            Expr::Block(_) | Expr::Loop(_) | Expr::Unsafe(_) => Some(BTreeSet::new()),
            _ => None,
        }
    };
    match s {
        syn::Stmt::Expr(e, _) => head(e).unwrap_or_else(|| ids_of(e)),
        _ => ids_of(s),
    }
}

/// 找到的一处：（范围或字面量的 span，命中的参数名）。
struct Finder<'a> {
    tokens: &'a [String],
    /// 当前「同一条语句」的标识符
    ctx: Vec<BTreeSet<String>>,
    /// 本文件的测试行（默认值字面量在这里不查）
    test_ranges: &'a [(Pos, Pos)],
    /// （键名末段，描述表默认值）；空 = 不查默认值（整份测试文件）
    defaults: &'a [(String, f64)],
    /// （位置，参数名，是不是默认值字面量）
    hits: Vec<(Span, String, bool)>,
}

impl Finder<'_> {
    fn tok_in(&self, extra: &BTreeSet<String>) -> Option<String> {
        self.tokens
            .iter()
            .find(|t| extra.contains(*t) || self.ctx.last().is_some_and(|c| c.contains(*t)))
            .cloned()
    }
}

impl<'ast> Visit<'ast> for Finder<'_> {
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        self.ctx.push(stmt_context(s));
        syn::visit::visit_stmt(self, s);
        self.ctx.pop();
    }
    fn visit_arm(&mut self, a: &'ast syn::Arm) {
        let mut c = ids_of(&a.pat);
        if let Some((_, g)) = &a.guard {
            c.extend(ids_of(g));
        }
        // 分支值 `=> (a, b)`
        if let Expr::Tuple(t) = &*a.body
            && t.elems.len() == 2
            && t.elems.iter().all(is_num)
            && let Some(tok) = self.tokens.iter().find(|x| c.contains(*x))
        {
            self.hits.push((a.body.span(), tok.clone(), false));
        }
        c.extend(ids_of(&a.body));
        self.ctx.push(c);
        syn::visit::visit_arm(self, a);
        self.ctx.pop();
    }
    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        self.ctx.push(ids_of(c));
        syn::visit::visit_item_const(self, c);
        self.ctx.pop();
    }
    fn visit_item_static(&mut self, c: &'ast syn::ItemStatic) {
        self.ctx.push(ids_of(c));
        syn::visit::visit_item_static(self, c);
        self.ctx.pop();
    }
    fn visit_expr(&mut self, e: &'ast Expr) {
        if is_literal_range(e)
            && let Some(tok) = self.tok_in(&ids_of(e))
        {
            self.hits.push((e.span(), tok, false));
        }
        syn::visit::visit_expr(self, e);
    }
    /// 宏参数能解析成逗号分隔的表达式（`assert!(…)`、`vec![…]` 这类）就按表达式照查
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let p = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
        if let Ok(args) = syn::parse::Parser::parse2(p, m.tokens.clone()) {
            for e in &args {
                self.visit_expr(e);
            }
        }
    }
    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        let in_test = in_ranges(self.test_ranges, pos(s.span()));
        if !in_test {
            for f in &s.fields {
                if let syn::Member::Named(n) = &f.member
                    && let Some(v) = num_value(&f.expr)
                    && let Some((k, _)) = self
                        .defaults
                        .iter()
                        .find(|(k, d)| n == k && (d - v).abs() < 1e-6)
                {
                    self.hits.push((f.span(), k.clone(), true));
                }
            }
        }
        syn::visit::visit_expr_struct(self, s);
    }
}

/// 一个文件的问题。`defaults` 为空时不查默认值字面量（整份测试文件这样传）。解析不了也报。
pub fn range_problems(
    rel: &str,
    src: &str,
    tokens: &[String],
    defaults: &[(String, f64)],
) -> Vec<String> {
    let file = match crate::ast::parse_src(src) {
        Ok(f) => f,
        Err(e) => {
            return vec![format!(
                "{rel}:{}：语法树解析失败：{}（R-PARAM）",
                e.line, e.msg
            )];
        }
    };
    let tests = file_test_ranges(rel, &file);
    let mut f = Finder {
        tokens,
        ctx: Vec::new(),
        test_ranges: &tests,
        defaults,
        hits: Vec::new(),
    };
    f.visit_file(&file);
    let lexed = lex_lines(src);
    let allowed =
        |l: usize, marker: &str| lexed.get(l - 1).is_some_and(|(_, c)| has_reason(c, marker));
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (sp, tok, default) in f.hits {
        let (a, b) = (sp.start().line, sp.end().line);
        let marker = if default { PRESET_ALLOW } else { RANGE_ALLOW };
        if allowed(a, marker) || allowed(b, marker) || !seen.insert(a) {
            continue;
        }
        let msg = if default {
            format!(
                "{rel}:{a}：参数 `{tok}` 的值和描述表默认值相同却写成了数字；正本是 {DESC_DIR}，改用 vdn_types::desc 的 .default；本意是预设就在行尾写 `// {PRESET_ALLOW}<理由和出处>`（R-PARAM）"
            )
        } else {
            format!(
                "{rel}:{a}：参数 `{tok}` 的范围写成了数字字面量；正本是 {DESC_DIR}，改用 vdn_types::desc 的 check / clamp / min / max（R-PARAM）"
            )
        };
        out.push(msg);
    }
    out
}

#[cfg(test)]
#[path = "param_range_tests.rs"]
mod tests;
