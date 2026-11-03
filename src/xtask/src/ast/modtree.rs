//! 从编译根建模块树，判哪些文件、哪些行是测试代码（012 第三节第 1 条；替代 #65 的文本版 `test_scope.rs`）。
//!
//! - **起点**：每个 crate 的编译根。生产：`src/lib.rs` · `src/main.rs` · `src/bin/*.rs` · `src/bin/*/main.rs` ·
//!   `examples/*.rs` · `examples/*/main.rs`；测试：`tests/*.rs` · `tests/*/main.rs` · `benches/*.rs`
//! - **展开**：跟 `mod` 声明走，按 Rust 的模块路径规则找文件（内联模块拼路径、`#[path]`、`r#name`）
//! - **测试判定**：模块或它的某个祖先带 `#[cfg(test)]` / `#[cfg(all(test, …))]` / `#[test]`，或在测试编译根下。
//!   一个文件被多处声明时，**只要有一条非测试路径能到它，它就是生产代码**
//! - **生产文件里的测试行**：带上述属性的项（含 impl / trait 里的项）、生产函数体里带上述属性的语句所占的行
//! - **孤儿**：crate 里任何根都到不了的 `.rs`，只报注释（012 第九节第 3 条）
//!
//! 判不了的（012 第四节）：`cfg(any(test, …))`、`not(…)`、feature 门控一律按生产算；宏展开出的模块不认。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use quote::ToTokens;
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ImplItem, Item, TraitItem};

use super::{cfg_is_test, parse_file, path_attr};

/// 源码里的位置：（行从 1 起，列从 0 起）。
pub type Pos = (usize, usize);

/// 位置落在某个区间里（终点不含）。按列比，同一行里测试项后面的生产代码不算测试（codex #71 第 1 轮 P1 #1）。
pub fn in_ranges(r: &[(Pos, Pos)], p: Pos) -> bool {
    r.iter().any(|(a, b)| *a <= p && p < *b)
}

/// 一个 span 的起点。
pub fn pos(s: proc_macro2::Span) -> Pos {
    (s.start().line, s.start().column)
}

/// 全仓的测试范围。路径都是相对仓库根的规范写法。
#[derive(Debug, Default)]
pub struct Scope {
    /// 经测试路径到达的文件
    test_files: BTreeSet<String>,
    /// 经生产路径到达的文件
    prod_files: BTreeSet<String>,
    /// 文件里属于测试的区间：（起点，终点），位置是（行从 1 起，列从 0 起）
    test_ranges: BTreeMap<String, Vec<(Pos, Pos)>>,
    /// 任何根都到不了的 `.rs`
    pub orphans: Vec<String>,
    /// 建树时的错误（解析失败、找不到模块文件、`#[path]` 跳出 crate）
    pub problems: Vec<String>,
}

impl Scope {
    /// 整个文件只在测试编译。
    pub fn is_test_file(&self, rel: &str) -> bool {
        self.test_files.contains(rel) && !self.prod_files.contains(rel)
    }

    /// 这一行（从 1 起）是测试代码。
    #[cfg(test)]
    pub fn is_test_line(&self, rel: &str, line: usize) -> bool {
        self.is_test_file(rel)
            || self
                .test_ranges
                .get(rel)
                .is_some_and(|rs| rs.iter().any(|(a, b)| (a.0..=b.0).contains(&line)))
    }

    /// 只在测试编译的文件（排过序）。
    #[cfg(test)]
    pub fn test_files(&self) -> Vec<String> {
        self.test_files
            .iter()
            .filter(|f| !self.prod_files.contains(*f))
            .cloned()
            .collect()
    }
}

/// 去掉 `.`、按 `..` 回退，免得同一文件有两种写法。
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn rel_of(root: &Path, p: &Path) -> String {
    p.strip_prefix(root)
        .map(|r| r.display().to_string())
        .unwrap_or_else(|_| p.display().to_string())
}

fn rs_in(dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    crate::tidy::rs_files(dir, &mut v);
    v
}

/// 一个 crate 的编译根：（文件，是不是测试根）。
pub fn crate_roots(dir: &Path) -> Vec<(PathBuf, bool)> {
    let mut out = Vec::new();
    for f in ["src/lib.rs", "src/main.rs"] {
        if dir.join(f).is_file() {
            out.push((dir.join(f), false));
        }
    }
    for (sub, test) in [
        ("src/bin", false),
        ("examples", false),
        ("tests", true),
        ("benches", true),
    ] {
        let Ok(rd) = std::fs::read_dir(dir.join(sub)) else {
            continue;
        };
        let mut es: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        es.sort();
        for p in es {
            if p.extension().is_some_and(|e| e == "rs") {
                out.push((p, test));
            } else if p.join("main.rs").is_file() {
                out.push((p.join("main.rs"), test));
            }
        }
    }
    out
}

/// 一段 token 开头的外层属性（语句、表达式的属性统一这样取，不必逐个枚举表达式种类）。
fn leading_attrs(ts: proc_macro2::TokenStream) -> Vec<syn::Attribute> {
    let p = |input: syn::parse::ParseStream| {
        let a = input.call(syn::Attribute::parse_outer)?;
        input.parse::<proc_macro2::TokenStream>()?;
        Ok(a)
    };
    syn::parse::Parser::parse2(p, ts).unwrap_or_default()
}

/// 生产函数体里带测试属性的语句（`#[cfg(test)] tests::gap();` 这类）。
#[derive(Default)]
struct TestStmts(Vec<proc_macro2::Span>);

impl<'ast> Visit<'ast> for TestStmts {
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        if cfg_is_test(&leading_attrs(s.to_token_stream())) {
            self.0.push(s.span());
        } else {
            syn::visit::visit_stmt(self, s);
        }
    }
}

/// 展开时的位置。
struct Ctx<'a> {
    rel: &'a str,
    /// 本文件所在目录（顶层 `#[path]` 相对它）
    file_dir: &'a Path,
    /// 这里声明的子模块放在哪（内联模块里的 `#[path]` 也相对它）
    child_dir: PathBuf,
    top_level: bool,
    test: bool,
}

struct Builder<'a> {
    root: &'a Path,
    crate_dir: PathBuf,
    scope: &'a mut Scope,
    queue: Vec<(PathBuf, PathBuf, bool)>,
    seen: BTreeSet<(PathBuf, bool)>,
}

impl Builder<'_> {
    fn range(&mut self, rel: &str, span: proc_macro2::Span) {
        let r = (pos(span), (span.end().line, span.end().column));
        self.scope
            .test_ranges
            .entry(rel.to_string())
            .or_default()
            .push(r);
    }

    fn body(&mut self, rel: &str, b: &syn::Block) {
        let mut v = TestStmts::default();
        v.visit_block(b);
        for sp in v.0 {
            self.range(rel, sp);
        }
    }

    fn items(&mut self, items: &[Item], cx: &Ctx) {
        for it in items {
            let attrs: &[syn::Attribute] = match it {
                Item::Const(i) => &i.attrs,
                Item::Enum(i) => &i.attrs,
                Item::ExternCrate(i) => &i.attrs,
                Item::Fn(i) => &i.attrs,
                Item::ForeignMod(i) => &i.attrs,
                Item::Impl(i) => &i.attrs,
                Item::Macro(i) => &i.attrs,
                Item::Mod(i) => &i.attrs,
                Item::Static(i) => &i.attrs,
                Item::Struct(i) => &i.attrs,
                Item::Trait(i) => &i.attrs,
                Item::TraitAlias(i) => &i.attrs,
                Item::Type(i) => &i.attrs,
                Item::Union(i) => &i.attrs,
                Item::Use(i) => &i.attrs,
                _ => &[],
            };
            let test = cfg_is_test(attrs);
            if test && !cx.test {
                self.range(cx.rel, it.span());
            }
            match it {
                Item::Mod(m) => self.module(m, cx, cx.test || test),
                Item::Fn(f) if !cx.test && !test => self.body(cx.rel, &f.block),
                Item::Impl(i) if !cx.test && !test => {
                    for ii in &i.items {
                        let a = match ii {
                            ImplItem::Const(x) => &x.attrs,
                            ImplItem::Fn(x) => &x.attrs,
                            ImplItem::Type(x) => &x.attrs,
                            ImplItem::Macro(x) => &x.attrs,
                            _ => continue,
                        };
                        if cfg_is_test(a) {
                            self.range(cx.rel, ii.span());
                        } else if let ImplItem::Fn(f) = ii {
                            self.body(cx.rel, &f.block);
                        }
                    }
                }
                Item::Trait(t) if !cx.test && !test => {
                    for ti in &t.items {
                        let a = match ti {
                            TraitItem::Const(x) => &x.attrs,
                            TraitItem::Fn(x) => &x.attrs,
                            TraitItem::Type(x) => &x.attrs,
                            TraitItem::Macro(x) => &x.attrs,
                            _ => continue,
                        };
                        if cfg_is_test(a) {
                            self.range(cx.rel, ti.span());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn module(&mut self, m: &syn::ItemMod, cx: &Ctx, test: bool) {
        let name = m.ident.unraw().to_string();
        if let Some((_, items)) = &m.content {
            // 内联模块自己的 `#[path]` 决定它里面子模块的目录（相对当前模块目录；codex #71 第 1 轮 P1 #2）
            let dir = match path_attr(&m.attrs) {
                Some(p) => normalize(&cx.child_dir.join(p)),
                None => cx.child_dir.join(&name),
            };
            let inner = Ctx {
                rel: cx.rel,
                file_dir: cx.file_dir,
                child_dir: dir,
                top_level: false,
                test,
            };
            self.items(items, &inner);
            return;
        }
        let line = m.span().start().line;
        let (target, child) = if let Some(p) = path_attr(&m.attrs) {
            let base = if cx.top_level {
                cx.file_dir
            } else {
                &cx.child_dir
            };
            let t = normalize(&base.join(p));
            let c = t.parent().map(Path::to_path_buf).unwrap_or_default();
            (t, c)
        } else {
            let flat = cx.child_dir.join(format!("{name}.rs"));
            let nested = cx.child_dir.join(&name).join("mod.rs");
            match (flat.is_file(), nested.is_file()) {
                (true, true) => {
                    self.scope.problems.push(format!(
                        "{}:{line}：模块 {name} 同时有 {name}.rs 与 {name}/mod.rs（编译器也会拒绝）",
                        cx.rel
                    ));
                    return;
                }
                (false, true) => (nested, cx.child_dir.join(&name)),
                _ => (flat, cx.child_dir.join(&name)),
            }
        };
        if !target.starts_with(&self.crate_dir) {
            self.scope.problems.push(format!(
                "{}:{line}：模块 {name} 的文件 {} 跳出了 crate 目录",
                cx.rel,
                target.display()
            ));
            return;
        }
        if !target.is_file() {
            // 带别的 cfg（平台、feature）的声明，文件可能本就不存在于这次构建：只在没有任何 cfg 时报红
            if !m.attrs.iter().any(|a| a.path().is_ident("cfg")) {
                self.scope.problems.push(format!(
                    "{}:{line}：找不到模块 {name} 的文件 {}",
                    cx.rel,
                    rel_of(self.root, &target)
                ));
            }
            return;
        }
        self.queue.push((target, child, test));
    }

    fn run(&mut self) {
        while let Some((file, child_dir, test)) = self.queue.pop() {
            if !self.seen.insert((file.clone(), test)) {
                continue;
            }
            let rel = rel_of(self.root, &file);
            let parsed = match parse_file(&file) {
                Ok(f) => f,
                Err(e) => {
                    self.scope.problems.push(e);
                    continue;
                }
            };
            let test = test || cfg_is_test(&parsed.attrs);
            if test {
                self.scope.test_files.insert(rel.clone());
            } else {
                self.scope.prod_files.insert(rel.clone());
            }
            let file_dir = file.parent().map(Path::to_path_buf).unwrap_or_default();
            let cx = Ctx {
                rel: &rel,
                file_dir: &file_dir,
                child_dir,
                top_level: true,
                test,
            };
            self.items(&parsed.items, &cx);
        }
    }
}

/// 一个 crate 的模块树并入 `scope`。
pub fn add_crate(root: &Path, crate_dir: &Path, scope: &mut Scope) {
    let mut b = Builder {
        root,
        crate_dir: normalize(crate_dir),
        scope,
        queue: Vec::new(),
        seen: BTreeSet::new(),
    };
    for (f, test) in crate_roots(crate_dir) {
        let dir = f.parent().map(Path::to_path_buf).unwrap_or_default();
        b.queue.push((normalize(&f), normalize(&dir), test));
    }
    b.run();
    let reached: BTreeSet<String> = b
        .scope
        .test_files
        .union(&b.scope.prod_files)
        .cloned()
        .collect();
    for sub in ["src", "tests", "benches", "examples"] {
        for f in rs_in(&crate_dir.join(sub)) {
            let rel = rel_of(root, &normalize(&f));
            if !reached.contains(&rel) {
                b.scope.orphans.push(rel);
            }
        }
    }
}

thread_local! {
    /// 同一次 tidy 里模块树只建一次（R-MODEL、R-PARAM 共用）。
    static BUILT: std::cell::RefCell<Vec<(PathBuf, std::rc::Rc<Scope>)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// 全仓（`src/*/` 下每个 crate）的测试范围（带缓存）。
pub fn build(root: &Path) -> std::rc::Rc<Scope> {
    if let Some(s) = BUILT.with(|b| {
        b.borrow()
            .iter()
            .find(|(r, _)| r == root)
            .map(|(_, s)| s.clone())
    }) {
        return s;
    }
    let s = std::rc::Rc::new(build_uncached(root));
    BUILT.with(|b| b.borrow_mut().push((root.to_path_buf(), s.clone())));
    s
}

fn build_uncached(root: &Path) -> Scope {
    let mut scope = Scope::default();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root.join("src"))
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    dirs.sort();
    for d in dirs.into_iter().filter(|d| d.join("Cargo.toml").is_file()) {
        add_crate(root, &d, &mut scope);
    }
    for rs in scope.test_ranges.values_mut() {
        rs.sort_unstable();
    }
    scope
}

/// 单个文件内的测试行区间（不跟 `mod x;` 去别的文件；外部模块声明本身那几行照样算）。
/// 给只看一个文件的规则用；它们按 [`Scope::is_test_file`] 先跳过整份测试文件。
pub fn file_test_ranges(rel: &str, file: &syn::File) -> Vec<(Pos, Pos)> {
    let mut scope = Scope::default();
    let mut b = Builder {
        root: Path::new(""),
        crate_dir: PathBuf::new(),
        scope: &mut scope,
        queue: Vec::new(),
        seen: BTreeSet::new(),
    };
    let none = PathBuf::new();
    let cx = Ctx {
        rel,
        file_dir: &none,
        child_dir: PathBuf::new(),
        top_level: true,
        test: false,
    };
    b.items(&file.items, &cx);
    let mut r = scope.test_ranges.remove(rel).unwrap_or_default();
    r.sort_unstable();
    r
}

/// 建树的错误进 `problems`（红），孤儿 `.rs` 进 `notes`（012 第九节第 3 条：先只报注释）。tidy 调它（codex #71 第 1 轮 P1 #3）。
pub fn report(root: &Path, problems: &mut Vec<String>, notes: &mut Vec<String>) {
    let s = build(root);
    problems.extend(s.problems.iter().map(|p| format!("{p}（模块树）")));
    notes.extend(
        s.orphans
            .iter()
            .map(|o| format!("{o}：没有任何模块声明指到它（孤儿 .rs，不参与编译；删掉或声明它）")),
    );
}
