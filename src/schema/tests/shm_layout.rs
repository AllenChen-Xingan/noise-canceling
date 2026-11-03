//! 交叉校验 · 共享内存头：布局文件 ↔ `offset_of!` / `size_of`；改布局必须升版本，历史只许追加。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vdn_control::Config;
use vdn_schema::{DIR, check, eval, export, root, shm, write};

mod common;
use common::*;

// ───────────── 共享内存头 ─────────────

/// 逐项比对：仓里布局文件的每个结构、每个字段（名 · 偏移 · 大小）都等于 `offset_of!` / `size_of` 算出的值。
#[test]
fn shm_layout_fields_match_offset_of() {
    let doc = committed(shm::LAYOUT_FILE);
    assert_eq!(
        doc["version"],
        json!(vdn_shm::layout::VERSION),
        "版本号与 layout::VERSION 不同"
    );
    assert_eq!(doc["header_bytes"], json!(vdn_shm::layout::HEADER_BYTES));
    assert_eq!(doc["slot_bytes"], json!(vdn_shm::layout::SLOT_BYTES));
    let structs = doc["structs"].as_array().unwrap();
    assert_eq!(structs.len(), vdn_shm::desc::STRUCTS.len());
    for (s, j) in vdn_shm::desc::STRUCTS.iter().zip(structs) {
        assert_eq!(j["name"], json!(s.name));
        assert_eq!(j["size"], json!(s.size), "{} 大小", s.name);
        assert_eq!(j["align"], json!(s.align), "{} 对齐", s.name);
        let fs = j["fields"].as_array().unwrap();
        assert_eq!(fs.len(), s.fields.len(), "{} 字段数", s.name);
        for (f, jf) in s.fields.iter().zip(fs) {
            assert_eq!(jf["name"], json!(f.name), "{} 字段顺序", s.name);
            assert_eq!(jf["offset"], json!(f.offset), "{}.{} 偏移", s.name, f.name);
            assert_eq!(jf["size"], json!(f.size), "{}.{} 大小", s.name, f.name);
        }
    }
    // 展开表的几个锚点直接用 offset_of! 再算一遍（不经 desc）
    use core::mem::{offset_of, size_of};
    use vdn_shm::layout::{ConfigLine, ConsLine, Header, ProdLine, RingHdr};
    let flat = doc["flat"].as_array().unwrap();
    let at = |path: &str| {
        flat.iter()
            .find(|f| f["path"] == path)
            .unwrap_or_else(|| panic!("flat 里没有 {path}"))["offset"]
            .as_u64()
            .unwrap() as usize
    };
    assert_eq!(
        at("config.session"),
        offset_of!(Header, config) + offset_of!(ConfigLine, session)
    );
    assert_eq!(
        at("rings[2].cons.gaps"),
        offset_of!(Header, rings)
            + 2 * size_of::<RingHdr>()
            + offset_of!(RingHdr, cons)
            + offset_of!(ConsLine, gaps)
    );
    assert_eq!(
        at("rings[1].prod.wake"),
        offset_of!(Header, rings)
            + size_of::<RingHdr>()
            + offset_of!(RingHdr, prod)
            + offset_of!(ProdLine, wake)
    );
}

/// 负例：布局变了（这里把一个偏移挪 8 字节模拟）而版本号没升 → 守卫报错，`write` 一个文件都不写。
#[test]
fn shm_layout_change_without_version_bump_is_red() {
    let body = shm::layout_body();
    let fp = shm::fingerprint(&body);
    let mut moved = body.clone();
    moved["flat"][1]["offset"] = json!(16);
    let fp2 = shm::fingerprint(&moved);
    assert_ne!(fp, fp2);
    let v = vdn_shm::layout::VERSION;
    let hist = [(v, fp.as_str())];
    // 布局变了、版本没升
    let e = shm::guard(&hist, v, &fp2).unwrap_err();
    assert!(e.contains("必须升版本"), "{e}");
    // 升了版本、但没在 HISTORY 登记：也红（导出工具不会替你登记）
    let e = shm::guard(&hist, v + 1, &fp2).unwrap_err();
    assert!(e.contains("还没登记"), "{e}");
    // 升了版本且登记了：放行
    assert!(shm::guard(&[(v, fp.as_str()), (v + 1, fp2.as_str())], v + 1, &fp2).is_ok());
    // 版本往回降、历史乱序、空历史：红
    assert!(shm::guard(&hist, v - 1, &fp).is_err());
    assert!(shm::guard(&[(v + 1, "a"), (v, fp.as_str())], v, &fp).is_err());
    assert!(shm::guard(&[], v, &fp).is_err());

    // 端到端：历史与当前布局不符 → write 一个文件都不写、check 报错
    let d = temp_root("bump");
    let bogus = [(v, "fnv1a64:0000000000000000")];
    std::fs::remove_file(d.join(DIR).join(shm::LAYOUT_FILE)).unwrap();
    let e = vdn_schema::write_with(&d, &bogus).unwrap_err();
    assert!(e.contains("必须升版本"), "{e}");
    assert!(
        !d.join(DIR).join(shm::LAYOUT_FILE).exists(),
        "守卫没过还写了文件"
    );
    write(&d).unwrap();
    assert!(
        vdn_schema::check_with(&d, &bogus)
            .iter()
            .any(|m| m.contains("必须升版本"))
    );
}

/// 历史只增不改：main 上已发布的 (版本, 指纹) 必须原样留在 HISTORY 里（codex 第 2 轮 #1）。
/// 拿不到 main 时：CI 里算红（要 fetch-depth: 0），本机只提示。
#[test]
fn shm_history_append_only_against_main() {
    match vdn_schema::shm_baseline(&root()) {
        vdn_schema::Baseline::Found(base, text) => {
            let b = shm::baseline_entry(&text).unwrap_or_else(|e| panic!("对照 {base}：{e}"));
            shm::guard_baseline(shm::HISTORY, &b).unwrap_or_else(|e| panic!("对照 {base}：{e}"));
        }
        vdn_schema::Baseline::Absent(base) => {
            eprintln!("注：{base} 上还没有 {}，本次是第一次引入", shm::LAYOUT_FILE);
        }
        vdn_schema::Baseline::Unavailable(why) => {
            assert!(std::env::var_os("CI").is_none(), "CI 里拿不到基线：{why}");
            eprintln!("注：跳过对照 main（{why}）");
        }
    }
}

/// 负例：把旧版本的指纹改写成新布局（不升版本）、或删掉旧行 → 红；在末尾追加新版本 → 绿。
#[test]
fn shm_history_rewrite_is_red() {
    let fresh = std::fs::read_to_string(root().join(DIR).join(shm::LAYOUT_FILE)).unwrap();
    let b = shm::baseline_entry(&fresh).unwrap();
    assert_eq!(
        b.fingerprint,
        shm::fingerprint(&shm::layout_body()),
        "从文件算的指纹应与从 Rust 算的相同"
    );
    let own: Vec<(u32, String)> = shm::HISTORY
        .iter()
        .map(|(v, f)| (*v, f.to_string()))
        .collect();
    assert_eq!(b.history, own, "文件里的历史应与 HISTORY 相同");
    // 改写、删除、追加都从完整历史出发改一行：以后正常追加了版本 3、4…，这条测试照样成立（codex 第 4 轮 #1）
    let full: Vec<(u32, &str)> = shm::HISTORY.to_vec();
    let last = full.len() - 1;
    let mut rewritten = full.clone();
    rewritten[last].1 = "fnv1a64:1111111111111111";
    let e = shm::guard_baseline(&rewritten, &b).unwrap_err();
    assert!(e.contains("被改写"), "{e}");
    let mut deleted = full.clone();
    deleted.remove(0);
    deleted.push((full[last].0 + 1, "fnv1a64:2222222222222222"));
    let e = shm::guard_baseline(&deleted, &b).unwrap_err();
    assert!(e.contains("删掉"), "{e}");
    let mut appended = full.clone();
    appended.push((full[last].0 + 1, "fnv1a64:2222222222222222"));
    assert!(shm::guard_baseline(&appended, &b).is_ok());
    assert!(shm::guard_baseline(&full, &b).is_ok());
    // 已发布两个版本的基线：原样保留并追加 → 绿
    let two = shm::BaseLayout {
        version: 3,
        fingerprint: "fnv1a64:b3".into(),
        history: vec![(2, "fnv1a64:a2".into()), (3, "fnv1a64:b3".into())],
    };
    let ok = [(2, "fnv1a64:a2"), (3, "fnv1a64:b3"), (4, "fnv1a64:c4")];
    assert!(shm::guard_baseline(&ok, &two).is_ok());
}

/// 整份历史只增不改（codex 第 3 轮 #3）：main 已发布到版本 3 时，改写或删掉更早的版本 2 也红。
#[test]
fn shm_history_whole_prefix_is_checked() {
    let s = |x: &str| x.to_string();
    let base = shm::BaseLayout {
        version: 3,
        fingerprint: s("fnv1a64:b3"),
        history: vec![(2, s("fnv1a64:a2")), (3, s("fnv1a64:b3"))],
    };
    let ok = [(2, "fnv1a64:a2"), (3, "fnv1a64:b3"), (4, "fnv1a64:c4")];
    assert!(shm::guard_baseline(&ok, &base).is_ok());
    assert!(shm::guard_baseline(&ok[..2], &base).is_ok());
    let old_rewritten = [(2, "fnv1a64:xx"), (3, "fnv1a64:b3"), (4, "fnv1a64:c4")];
    let e = shm::guard_baseline(&old_rewritten, &base).unwrap_err();
    assert!(e.contains("版本 2") && e.contains("被改写"), "{e}");
    let old_deleted = [(3, "fnv1a64:b3"), (4, "fnv1a64:c4")];
    let e = shm::guard_baseline(&old_deleted, &base).unwrap_err();
    assert!(e.contains("删掉"), "{e}");
    // 在已发布的行之间插一行：版本号仍递增，但位置变了
    let base1 = shm::BaseLayout {
        version: 3,
        fingerprint: s("fnv1a64:b3"),
        history: vec![(1, s("fnv1a64:z1")), (3, s("fnv1a64:b3"))],
    };
    let inserted = [(1, "fnv1a64:z1"), (2, "fnv1a64:a2"), (3, "fnv1a64:b3")];
    let e = shm::guard_baseline(&inserted, &base1).unwrap_err();
    assert!(e.contains("插进"), "{e}");
    // 基线文件自相矛盾（布局指纹不在它自己的历史里）：红
    let broken = shm::BaseLayout {
        fingerprint: s("fnv1a64:zz"),
        ..base.clone()
    };
    assert!(
        shm::guard_baseline(&ok, &broken)
            .unwrap_err()
            .contains("自相矛盾")
    );
    // 旧格式文件（没有 history）：按只有当前一行算
    let mut old: Value = serde_json::from_str(
        &std::fs::read_to_string(root().join(DIR).join(shm::LAYOUT_FILE)).unwrap(),
    )
    .unwrap();
    old.as_object_mut().unwrap().remove("history");
    let b = shm::baseline_entry(&old.to_string()).unwrap();
    assert_eq!(b.history, vec![(b.version, b.fingerprint.clone())]);
}

fn git_out(args: &[&str]) -> Option<String> {
    let o = std::process::Command::new("git")
        .arg("-C")
        .arg(root())
        .args(args)
        .output()
        .ok()?;
    o.status
        .success()
        .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// 读基线：提交在、但读树或读文件失败 → Unavailable，不算 Absent（codex 第 4 轮 #2）。
/// 用假的 git 逐步注入结果，不依赖本机仓库。
#[test]
fn shm_baseline_read_errors_are_unavailable() {
    use vdn_schema::Baseline;
    let sha = "1111111111111111111111111111111111111111";
    // ls_tree / show：None = 命令失败
    let fake = |ls_tree: Option<&'static str>, show: Option<&'static str>| {
        move |args: &[&str]| -> Option<String> {
            match args[0] {
                "rev-parse" => Some(sha.to_string()),
                "ls-tree" => ls_tree.map(str::to_string),
                "show" => show.map(str::to_string),
                other => panic!("没料到的 git 子命令 {other}"),
            }
        }
    };
    let path = format!("{DIR}/{}", shm::LAYOUT_FILE);
    let listed: &'static str = Box::leak(path.into_boxed_str());
    let cases: [(Option<&'static str>, Option<&'static str>, &str); 4] = [
        (None, None, "Unavailable"),         // 读树失败
        (Some(listed), None, "Unavailable"), // 树里有文件，读文件失败
        (Some(""), None, "Absent"),          // 树里确实没有
        (Some(listed), Some("{}"), "Found"), // 读到了
    ];
    for (ls, show, want) in cases {
        let got = match vdn_schema::shm_baseline_with("base", &fake(ls, show)) {
            Baseline::Found(c, _) => {
                assert_eq!(c, sha);
                "Found"
            }
            Baseline::Absent(c) => {
                assert_eq!(c, sha);
                "Absent"
            }
            Baseline::Unavailable(_) => "Unavailable",
        };
        assert_eq!(got, want, "ls-tree = {ls:?}，show = {show:?}");
    }
    // 提交不存在
    let none = |_: &[&str]| -> Option<String> { None };
    assert!(matches!(
        vdn_schema::shm_baseline_with("base", &none),
        Baseline::Unavailable(_)
    ));
}

/// 读基线：提交不存在 → Unavailable（不当成「首次引入」放行）；提交在、树里没有文件 → Absent；
/// 有文件 → Found。环境变量给的全零提交 → Unavailable（codex 第 3 轮 #1、#2）。
#[test]
fn shm_baseline_failures_are_not_absent() {
    let r = root();
    let Some(first) = git_out(&["rev-list", "--max-parents=0", "HEAD"]) else {
        assert!(std::env::var_os("CI").is_none(), "CI 里不是 git 仓库");
        eprintln!("注：不是 git 仓库，跳过");
        return;
    };
    let first = first.lines().next().unwrap().to_string();
    let missing = "0123456789abcdef0123456789abcdef01234567";
    assert!(matches!(
        vdn_schema::shm_baseline_at(&r, missing),
        vdn_schema::Baseline::Unavailable(_)
    ));
    assert!(matches!(
        vdn_schema::shm_baseline_from(&r, Some(missing)),
        vdn_schema::Baseline::Unavailable(_)
    ));
    assert!(matches!(
        vdn_schema::shm_baseline_from(&r, Some("0000000000000000000000000000000000000000")),
        vdn_schema::Baseline::Unavailable(_)
    ));
    // 仓库的第一个提交早于本机制，树里没有这份文件
    assert!(matches!(
        vdn_schema::shm_baseline_at(&r, &first),
        vdn_schema::Baseline::Absent(_)
    ));
    // 环境变量指定的基线优先于本机规则（多提交推送时用推送前的提交）
    assert!(matches!(
        vdn_schema::shm_baseline_from(&r, Some(&first)),
        vdn_schema::Baseline::Absent(_)
    ));
    // 本分支已提交了这份文件
    if git_out(&[
        "ls-files",
        "--error-unmatch",
        &format!("{DIR}/{}", shm::LAYOUT_FILE),
    ])
    .is_some()
    {
        assert!(matches!(
            vdn_schema::shm_baseline_at(&r, "HEAD"),
            vdn_schema::Baseline::Found(..)
        ));
    }
}

/// 当前版本正是 HISTORY 的最后一行、指纹相同。
#[test]
fn shm_version_registered() {
    assert_eq!(
        shm::HISTORY.last().copied(),
        Some((
            vdn_shm::layout::VERSION,
            shm::fingerprint(&shm::layout_body()).as_str()
        ))
    );
}
