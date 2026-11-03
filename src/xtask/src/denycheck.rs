//! tidy 的两条规则（规约 §5.4 · 人定「每个功能都带测试」）：
//!
//! 1. `deny.toml` 的直接边规则（`[bans] deny = [{ crate = "X", wrappers = [...] }]`，一条一行）与真实依赖、
//!    各 crate 的 `metadata.vdn.forbid` 一致：
//!    - 名单里每个名字都是本仓 crate
//!    - 名单里的 crate 自己没把 X 列进 forbid（否则两处说法相反）
//!    - 实际**直接**依赖 X 的本仓 crate 都在名单里（与 cargo-deny 同一口径；本机没装 cargo-deny 时也能挡住）
//! 2. 每个 workspace crate 至少有一条测试：`tests/` 下有 `.rs`，或 `src/` 里出现 `#[test]`

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::deps::Meta;

/// 一条直接边规则。
#[derive(Debug, PartialEq, Eq)]
pub struct Ban {
    pub krate: String,
    pub wrappers: Vec<String>,
}

/// 从 `deny.toml` 文本里取 `{ crate = "X", wrappers = ["a", "b"] }` 形式的行（只认一条一行的写法）。
pub fn parse_bans(toml: &str) -> Vec<Ban> {
    let quoted = |s: &str| -> Vec<String> {
        s.split('"')
            .enumerate()
            .filter(|(i, _)| i % 2 == 1)
            .map(|(_, t)| t.to_string())
            .collect()
    };
    toml.lines()
        .map(|l| l.split('#').next().unwrap_or_default())
        .filter(|l| l.contains("crate =") && l.contains("wrappers ="))
        .filter_map(|l| {
            let after_crate = l.split("crate =").nth(1)?;
            let krate = quoted(after_crate).into_iter().next()?;
            let w = l.split("wrappers =").nth(1)?;
            let list = w.split('[').nth(1)?.split(']').next()?;
            Some(Ban { krate, wrappers: quoted(list) })
        })
        .collect()
}

pub fn check_bans(meta: &Meta, deny_toml: &str, problems: &mut Vec<String>) {
    let members = meta.members();
    let names: BTreeSet<&str> = members.iter().map(|(_, n, _, _)| n.as_str()).collect();
    for ban in parse_bans(deny_toml) {
        for w in &ban.wrappers {
            if !names.contains(w.as_str()) {
                problems.push(format!("deny.toml：{} 的名单里有 {w}，但它不是本仓 crate", ban.krate));
            }
        }
        for (id, name, _, vdn) in &members {
            let forbids = vdn
                .as_ref()
                .and_then(|v| v["forbid"].as_array())
                .is_some_and(|a| a.iter().any(|x| x.as_str() == Some(ban.krate.as_str())));
            let listed = ban.wrappers.iter().any(|w| w == name);
            if listed && forbids {
                problems.push(format!(
                    "deny.toml 许 {name} 直接依赖 {}，但 {name} 的 metadata.vdn.forbid 禁了它（两处相反）",
                    ban.krate
                ));
            }
            if !listed && meta.direct_any(id).contains(&ban.krate) {
                problems.push(format!(
                    "{name} 直接依赖 {}，但不在 deny.toml 的名单里（要么改依赖，要么加进名单并写理由）",
                    ban.krate
                ));
            }
        }
    }
}

/// crate 目录里有没有测试。
pub fn has_tests(dir: &Path) -> bool {
    let tests_dir = fs::read_dir(dir.join("tests"))
        .map(|rd| rd.flatten().any(|e| e.path().extension().is_some_and(|x| x == "rs")))
        .unwrap_or(false);
    if tests_dir {
        return true;
    }
    let mut files = Vec::new();
    crate::tidy::rs_files(&dir.join("src"), &mut files);
    files
        .iter()
        .any(|f| fs::read_to_string(f).is_ok_and(|s| s.contains("#[test]")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_line_bans() {
        let t = r#"
[bans]
deny = [
    # 只许后端直接用 ort
    { crate = "ort", wrappers = ["vdn-backend-ort", "vdn-eval"] },
    { crate = "cpal", wrappers = ["vdn-io-macos"], reason = "x" },
    { crate = "openssl" },
]
"#;
        assert_eq!(
            parse_bans(t),
            vec![
                Ban { krate: "ort".into(), wrappers: vec!["vdn-backend-ort".into(), "vdn-eval".into()] },
                Ban { krate: "cpal".into(), wrappers: vec!["vdn-io-macos".into()] },
            ]
        );
    }

    #[test]
    fn crate_without_tests_is_detected() {
        let d = std::env::temp_dir().join(format!("vdn-tidy-tests-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(d.join("src")).unwrap();
        fs::write(d.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        assert!(!has_tests(&d));
        fs::write(d.join("src/lib.rs"), "#[test]\nfn t() {}\n").unwrap();
        assert!(has_tests(&d));
        fs::write(d.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        fs::create_dir_all(d.join("tests")).unwrap();
        fs::write(d.join("tests/a.rs"), "").unwrap();
        assert!(has_tests(&d));
        let _ = fs::remove_dir_all(&d);
    }
}
