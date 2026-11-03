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

use super::{cfg_attr_paths, cfg_is_test, parse_file, path_attr};

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

/// 走一个文件时所在的位置。
#[derive(Clone)]
struct Frame {
    /// 这里声明的子模块放在哪（内联模块里的 `#[path]` 也相对它）
    child_dir: PathBuf,
    /// 在文件顶层（顶层 `#[path]` 相对文件目录）
    top_level: bool,
    test: bool,
    /// 在函数、常量、impl 这类块里（块里的外部模块只有带 `#[path]` 才合法，基准目录两种都算）
    in_block: bool,
}

struct Builder<'a> {
    root: &'a Path,
    crate_dir: PathBuf,
    scope: &'a mut Scope,
    queue: Vec<(PathBuf, PathBuf, bool)>,
    /// （文件，是否测试，子模块目录）：同一文件以不同目录上下文加载要各展开一次（codex #71 第 3 轮 P1 #2）
    seen: BTreeSet<(PathBuf, bool, PathBuf)>,
}

/// 一个文件的语法树访问器：走遍**每个**节点（条目、impl / trait 的项、函数与常量的块、语句），
/// 不再按位置一处处枚举（codex #71 第 2、3 轮 P1 #1）。
struct Walker<'b, 'a> {
    b: &'b mut Builder<'a>,
    rel: String,
    file_dir: PathBuf,
    stack: Vec<Frame>,
}

fn item_attrs(it: &Item) -> &[syn::Attribute] {
    match it {
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
    }
}

impl Walker<'_, '_> {
    fn cur(&self) -> Frame {
        self.stack.last().cloned().unwrap_or(Frame {
            child_dir: PathBuf::new(),
            top_level: true,
            test: false,
            in_block: false,
        })
    }

    /// 进入一个节点：带测试属性就记区间；返回进去后的位置。
    fn enter(&mut self, test_attr: bool, span: proc_macro2::Span, block: bool) -> Frame {
        let cur = self.cur();
        if test_attr && !cur.test {
            self.b.range(&self.rel, span);
        }
        Frame {
            test: cur.test || test_attr,
            in_block: cur.in_block || block,
            ..cur
        }
    }

    fn with<F: FnOnce(&mut Self)>(&mut self, f: Frame, go: F) {
        self.stack.push(f);
        go(self);
        self.stack.pop();
    }

    fn module(&mut self, m: &syn::ItemMod, fr: Frame) {
        let name = m.ident.unraw().to_string();
        let line = m.span().start().line;
        let plain = path_attr(&m.attrs);
        if let Some((_, items)) = &m.content {
            // 内联模块自己的 `#[path]` 决定它里面子模块的目录（codex #71 第 1 轮 P1 #2）
            let dir = match &plain {
                Some(p) => normalize(&fr.child_dir.join(p)),
                None => fr.child_dir.join(&name),
            };
            let inner = Frame {
                child_dir: dir,
                top_level: false,
                ..fr
            };
            self.with(inner, |w| {
                for it in items {
                    w.visit_item(it);
                }
            });
            return;
        }
        // 外部模块：所有可能指到的文件都当引用（`#[path]`、`cfg_attr` 里的 path、默认位置），宁可多查
        let bases: Vec<PathBuf> = if fr.in_block {
            vec![self.file_dir.clone(), fr.child_dir.clone()]
        } else if fr.top_level {
            vec![self.file_dir.clone()]
        } else {
            vec![fr.child_dir.clone()]
        };
        let mut cands: Vec<(PathBuf, PathBuf)> = Vec::new();
        let paths: Vec<String> = plain
            .iter()
            .cloned()
            .chain(cfg_attr_paths(&m.attrs))
            .collect();
        for p in &paths {
            for base in &bases {
                let t = normalize(&base.join(p));
                let c = t.parent().map(Path::to_path_buf).unwrap_or_default();
                cands.push((t, c));
            }
        }
        if plain.is_none() && !fr.in_block {
            let flat = fr.child_dir.join(format!("{name}.rs"));
            let nested = fr.child_dir.join(&name).join("mod.rs");
            if flat.is_file() && nested.is_file() {
                self.b.scope.problems.push(format!(
                    "{}:{line}：模块 {name} 同时有 {name}.rs 与 {name}/mod.rs（编译器也会拒绝）",
                    self.rel
                ));
                return;
            }
            let t = if nested.is_file() { nested } else { flat };
            cands.push((t, fr.child_dir.join(&name)));
        }
        let (mut hit, mut escaped) = (false, false);
        for (t, c) in cands {
            if !t.starts_with(&self.b.crate_dir) {
                self.b.scope.problems.push(format!(
                    "{}:{line}：模块 {name} 的文件 {} 跳出了 crate 目录",
                    self.rel,
                    t.display()
                ));
                escaped = true;
                continue;
            }
            if t.is_file() {
                self.b.queue.push((t, c, fr.test));
                hit = true;
            }
        }
        // 带别的 cfg（平台、feature）的声明，文件可能本就不存在于这次构建：只在没有任何 cfg 时报红
        if !hit
            && !escaped
            && !m.attrs.iter().any(|a| a.path().is_ident("cfg"))
            && (!fr.in_block || plain.is_some())
        {
            self.b.scope.problems.push(format!(
                "{}:{line}：找不到模块 {name} 的文件（{}）",
                self.rel,
                if paths.is_empty() {
                    format!("{name}.rs 或 {name}/mod.rs")
                } else {
                    paths.join(" / ")
                }
            ));
        }
    }
}

impl<'ast> Visit<'ast> for Walker<'_, '_> {
    fn visit_item(&mut self, it: &'ast Item) {
        let test = cfg_is_test(item_attrs(it));
        let block = !matches!(it, Item::Mod(_));
        let fr = self.enter(test, it.span(), block);
        if let Item::Mod(m) = it {
            self.module(m, fr);
            return;
        }
        self.with(fr, |w| syn::visit::visit_item(w, it));
    }
    fn visit_impl_item(&mut self, ii: &'ast ImplItem) {
        let a = match ii {
            ImplItem::Const(x) => &x.attrs[..],
            ImplItem::Fn(x) => &x.attrs[..],
            ImplItem::Type(x) => &x.attrs[..],
            ImplItem::Macro(x) => &x.attrs[..],
            _ => &[],
        };
        let fr = self.enter(cfg_is_test(a), ii.span(), true);
        self.with(fr, |w| syn::visit::visit_impl_item(w, ii));
    }
    fn visit_trait_item(&mut self, ti: &'ast TraitItem) {
        let a = match ti {
            TraitItem::Const(x) => &x.attrs[..],
            TraitItem::Fn(x) => &x.attrs[..],
            TraitItem::Type(x) => &x.attrs[..],
            TraitItem::Macro(x) => &x.attrs[..],
            _ => &[],
        };
        let fr = self.enter(cfg_is_test(a), ti.span(), true);
        self.with(fr, |w| syn::visit::visit_trait_item(w, ti));
    }
    fn visit_stmt(&mut self, s: &'ast syn::Stmt) {
        let test = cfg_is_test(&leading_attrs(s.to_token_stream()));
        let fr = self.enter(test, s.span(), true);
        self.with(fr, |w| syn::visit::visit_stmt(w, s));
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if m.path.is_ident("include") && !self.cur().test {
            self.b.scope.problems.push(format!(
                "{}:{}：生产代码里用了 include!，引入的代码不在模块树里，tidy 查不到（012 第四节）",
                self.rel,
                m.span().start().line
            ));
        }
    }
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

    /// 走一个文件的全部条目。
    fn walk(&mut self, rel: &str, file_dir: &Path, child_dir: PathBuf, test: bool, f: &syn::File) {
        let mut w = Walker {
            b: self,
            rel: rel.to_string(),
            file_dir: file_dir.to_path_buf(),
            stack: vec![Frame {
                child_dir,
                top_level: true,
                test,
                in_block: false,
            }],
        };
        for it in &f.items {
            w.visit_item(it);
        }
    }

    fn run(&mut self) {
        while let Some((file, child_dir, test)) = self.queue.pop() {
            if !self.seen.insert((file.clone(), test, child_dir.clone())) {
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
            self.walk(&rel, &file_dir, child_dir, test, &parsed);
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
    b.walk(rel, Path::new(""), PathBuf::new(), false, file);
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
