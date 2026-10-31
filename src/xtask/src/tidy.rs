//! tidy（规约 §5.1「静态·项目约定」、§5.3、§5.4）：文本 / 轻语法检查，不做语义。
//!
//! | 检查 | 规则 |
//! |---|---|
//! | `.gitignore` 含 `/target/` | R14 |
//! | 每个成员有 `[package.metadata.vdn]`，`tier` ∈ A/B/C/tool | §5.4 |
//! | `unsafe = false` 的 crate 根有 `#![forbid(unsafe_code)]` | R10 |
//! | A/B 档 crate 根有 panic 类 lint（unwrap/expect/panic/indexing），且至少一个实时块升了禁用清单 | R4 · §5.3 |
//! | 放行禁用清单 / panic 类 lint 的 `allow` 必须带 `// NRT: <理由>` | §5.3 |
//! | A/B 档 crate 的源码（整模块 NRT 除外）不出现 `std::fs`、`log::`、`thread::sleep` | R3 |
//! | 实时块（带 `deny(clippy::disallowed_macros)` 的 impl / mod）里没有非 debug 的 `assert*!` | R4（clippy 禁不了，见根 clippy.toml） |

use std::fs;
use std::path::{Path, PathBuf};

use crate::deps::metadata;
use crate::root;

const GUARDED_LINTS: [&str; 7] = [
    "disallowed_methods",
    "disallowed_types",
    "disallowed_macros",
    "indexing_slicing",
    "unwrap_used",
    "expect_used",
    "clippy::panic",
];
const PANIC_LINTS: [&str; 4] = [
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::indexing_slicing",
];

pub struct Tidy {
    pub problems: Vec<String>,
    pub nrt_allows: usize,
    pub notes: Vec<String>,
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// 去掉行尾 `//` 注释（粗糙：不认字符串里的 `//`，对本仓够用）。
fn code_of(line: &str) -> &str {
    line.split_once("//").map_or(line, |(c, _)| c)
}

fn has_nrt_reason(line: &str) -> bool {
    line.split_once("// NRT:")
        .is_some_and(|(_, why)| !why.trim().is_empty())
}

/// 一个属性从 `start` 行起，到含 `]` 的行止（rustfmt 会把长属性拆成多行）。
fn attr_span(lines: &[&str], start: usize) -> usize {
    let mut end = start;
    while end < lines.len() && !code_of(lines[end]).contains(']') {
        end += 1;
    }
    end.min(lines.len().saturating_sub(1))
}

/// 整个文件是 NRT 模块：头部有 `#![allow(…)]` 且带 NRT 理由。
fn is_nrt_module(lines: &[&str]) -> bool {
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if t.starts_with("//") || t.is_empty() {
            continue;
        }
        if t.starts_with("#![allow(") {
            let end = attr_span(lines, i);
            return lines[i..=end].iter().any(|l| has_nrt_reason(l));
        }
        if !t.starts_with("#!") {
            return false;
        }
    }
    false
}

fn check_file(path: &Path, rel: &str, realtime_crate: bool, t: &mut Tidy) -> usize {
    let Ok(src) = fs::read_to_string(path) else {
        t.problems.push(format!("{rel}：读不了"));
        return 0;
    };
    let lines: Vec<&str> = src.lines().collect();
    let mut rt_blocks = 0;

    // 1. allow 例外必须带 NRT 理由
    for (i, l) in lines.iter().enumerate() {
        let c = code_of(l);
        if !(c.contains("#[allow(") || c.contains("#![allow(")) {
            continue;
        }
        let end = attr_span(&lines, i);
        let body: String = lines[i..=end].iter().map(|l| code_of(l)).collect();
        if !GUARDED_LINTS.iter().any(|g| body.contains(g)) {
            continue;
        }
        t.nrt_allows += 1;
        let prev_ok = i > 0 && has_nrt_reason(lines[i - 1]);
        if !(prev_ok || lines[i..=end].iter().any(|l| has_nrt_reason(l))) {
            t.problems
                .push(format!("{rel}:{}：放行实时 lint 的 allow 缺「// NRT: 理由」（§5.3）", i + 1));
        }
    }

    if !realtime_crate || rel.contains("/tests/") || rel.contains("/benches/") {
        return 0;
    }
    if is_nrt_module(&lines) {
        return 0;
    }

    // 2. R3：实时 crate 不做 IO、日志、sleep（#[cfg(test)] 之后的部分不查）
    let test_from = lines
        .iter()
        .position(|l| l.trim() == "#[cfg(test)]")
        .unwrap_or(lines.len());
    for (i, l) in lines.iter().enumerate().take(test_from) {
        let c = code_of(l);
        for bad in ["std::fs", "log::", "thread::sleep"] {
            if c.contains(bad) && !c.contains("crate::record") {
                t.problems.push(format!("{rel}:{}：实时 crate 里出现 {bad}（R3）", i + 1));
            }
        }
    }

    // 3. R4：实时块里只许 debug_assert*
    let mut i = 0;
    while i < test_from {
        let c = code_of(lines[i]);
        if c.contains("#[deny(") || c.contains("#![deny(") {
            let end = attr_span(&lines, i);
            let body: String = lines[i..=end].iter().map(|l| code_of(l)).collect();
            if body.contains("clippy::disallowed_macros") {
                rt_blocks += 1;
                // 找块的起止：从属性后第一个 `{` 开始数括号
                let mut depth = 0i32;
                let mut opened = false;
                let mut j = end + 1;
                while j < lines.len() {
                    let cj = code_of(lines[j]);
                    for (k, _) in cj.match_indices("assert") {
                        let before = &cj[..k];
                        let after = &cj[k..];
                        let is_macro = after.starts_with("assert!(")
                            || after.starts_with("assert_eq!(")
                            || after.starts_with("assert_ne!(");
                        if opened && is_macro && !before.ends_with("debug_") {
                            t.problems.push(format!(
                                "{rel}:{}：实时块里有会在 release panic 的 assert，改 debug_assert*（R4）",
                                j + 1
                            ));
                        }
                    }
                    for ch in cj.chars() {
                        match ch {
                            '{' => {
                                depth += 1;
                                opened = true;
                            }
                            '}' => depth -= 1,
                            _ => {}
                        }
                    }
                    if opened && depth <= 0 {
                        break;
                    }
                    j += 1;
                }
                i = j;
            }
        }
        i += 1;
    }
    rt_blocks
}

pub fn check() -> Result<Tidy, String> {
    let root = root();
    let mut t = Tidy {
        problems: Vec::new(),
        nrt_allows: 0,
        notes: Vec::new(),
    };

    // R14
    let gi = fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    if !gi.lines().any(|l| l.trim() == "/target/") {
        t.problems.push(".gitignore 缺 /target/（R14）".into());
    }

    // 根 clippy.toml 只许一份
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    for d in fs::read_dir(root.join("src")).map_err(|e| e.to_string())?.flatten() {
        if d.path().join("clippy.toml").exists() || d.path().join(".clippy.toml").exists() {
            t.problems.push(format!(
                "{}：crate 目录下不许放 clippy.toml（会整份替换根配置，§5.3）",
                d.path().display()
            ));
        }
    }

    if !root.join("deny.toml").exists() {
        t.notes.push("deny.toml 尚未建立：cargo-deny 未装，直接边检查待 E-deny（decision.md「仍未补齐」）".into());
    }

    let meta = metadata()?;
    for (_, name, manifest, vdn) in meta.members() {
        let dir = Path::new(&manifest)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let Some(vdn) = vdn else {
            t.problems.push(format!("{name}：缺 [package.metadata.vdn]（§5.4）"));
            continue;
        };
        let tier = vdn["tier"].as_str().unwrap_or("");
        if !["A", "B", "C", "tool"].contains(&tier) {
            t.problems.push(format!("{name}：metadata.vdn.tier 必须是 A/B/C/tool，现在是 {tier:?}"));
        }
        if !vdn["forbid"].is_array() {
            t.problems.push(format!("{name}：metadata.vdn.forbid 必须是数组"));
        }
        let crate_root = ["src/lib.rs", "src/main.rs"]
            .iter()
            .map(|f| dir.join(f))
            .find(|p| p.exists());
        let root_src = crate_root
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .unwrap_or_default();
        match vdn["unsafe"].as_bool() {
            Some(false) if !root_src.contains("#![forbid(unsafe_code)]") => t
                .problems
                .push(format!("{name}：unsafe = false 但 crate 根没有 #![forbid(unsafe_code)]（R10）")),
            Some(_) => {}
            None => t.problems.push(format!("{name}：metadata.vdn.unsafe 必须是 true/false")),
        }
        let realtime = tier == "A" || tier == "B";
        if realtime && !PANIC_LINTS.iter().all(|l| root_src.contains(l)) {
            t.problems.push(format!(
                "{name}：A/B 档 crate 根缺 panic 类 lint（unwrap/expect/panic/indexing，§5.3）"
            ));
        }
        let mut files = Vec::new();
        rs_files(&dir.join("src"), &mut files);
        let mut blocks = 0;
        for f in files {
            let rel = f
                .strip_prefix(&root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| f.display().to_string());
            blocks += check_file(&f, &rel, realtime, &mut t);
        }
        if realtime && blocks == 0 {
            t.problems.push(format!(
                "{name}：A/B 档 crate 没有任何实时块升禁用清单（deny(clippy::disallowed_*)，§5.3）"
            ));
        }
    }
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str, realtime: bool) -> Tidy {
        let dir = std::env::temp_dir().join(format!("vdn-tidy-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let p = dir.join(format!("{}.rs", src.len()));
        fs::write(&p, src).unwrap();
        let mut t = Tidy {
            problems: Vec::new(),
            nrt_allows: 0,
            notes: Vec::new(),
        };
        check_file(&p, "x/src/a.rs", realtime, &mut t);
        let _ = fs::remove_file(&p);
        t
    }

    #[test]
    fn allow_without_reason_is_flagged() {
        let t = run("#[allow(clippy::indexing_slicing)]\nfn f() {}\n", false);
        assert_eq!(t.problems.len(), 1);
        let t = run("#[allow(clippy::indexing_slicing)] // NRT: 构造期\nfn f() {}\n", false);
        assert!(t.problems.is_empty());
        assert_eq!(t.nrt_allows, 1);
        let t = run("#[allow(clippy::indexing_slicing)] // NRT:\nfn f() {}\n", false);
        assert_eq!(t.problems.len(), 1, "理由不许为空");
        let t = run("#[allow(clippy::large_enum_variant)]\nenum E {}\n", false);
        assert!(t.problems.is_empty(), "不相关的 allow 不管");
    }

    #[test]
    fn release_assert_in_realtime_block_is_flagged() {
        let src = "#[deny(\n    clippy::disallowed_macros\n)]\nimpl X {\n    fn p(&self) {\n        debug_assert_eq!(1, 1);\n        assert!(true);\n    }\n}\nfn new() { assert!(true); }\n";
        let t = run(src, true);
        assert_eq!(t.problems.len(), 1, "{:?}", t.problems);
        assert!(t.problems[0].contains(":7："));
    }

    #[test]
    fn io_in_realtime_crate_is_flagged_but_not_in_nrt_module() {
        let t = run("fn f() { let _ = std::fs::read(\"a\"); }\n", true);
        assert_eq!(t.problems.len(), 1);
        let t = run(
            "//! 离线\n\n#![allow(clippy::indexing_slicing)] // NRT: 离线驱动\nfn f() { let _ = std::fs::read(\"a\"); }\n",
            true,
        );
        assert!(t.problems.is_empty(), "{:?}", t.problems);
    }
}
