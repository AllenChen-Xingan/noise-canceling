//! 交叉校验 · 参数描述表与两种文本格式（场景参数表 `vdn-scene-table`、我的配置 `vdn-profile`）。
//!
//! - `params.table.json` 的每一条与 `vdn_types::desc` 逐字段相同（数值按 f32 比）；改表不重新导出 → 红
//! - 文件里写出的「文本格式能用的键」与两个解析器的实际行为一致：列出的都认，没列出的都拒
//! - 每个参数取描述表的边界外一点，两种文本都拒收；边界值收下
//! - 仓里已跟踪的所有这两种文本文件（含 `specs/schema/examples/`）逐个解析通过

use serde_json::Value;
use vdn_schema::{DIR, check, params, root};
use vdn_types::{ModelId, ParamField, SceneTable, UserProfile, desc};

mod common;
use common::*;

fn table() -> Value {
    committed(params::TABLE_FILE)
}

fn strs(v: &Value) -> Vec<&str> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect()
}

/// 每一条都和描述表相同，条数、顺序一致。
#[test]
fn params_table_matches_desc() {
    let t = table();
    let rows = t["params"].as_array().unwrap();
    assert_eq!(rows.len(), desc::ALL.len());
    for (r, d) in rows.iter().zip(desc::ALL) {
        assert_eq!(r["id"].as_u64(), Some(u64::from(d.id)), "{}", d.key);
        assert_eq!(r["key"].as_str(), Some(d.key));
        assert_eq!(r["unit"].as_str(), Some(d.unit.as_str()), "{}", d.key);
        for (k, want) in [("min", d.min), ("max", d.max), ("default", d.default)] {
            let got = r[k].as_f64().unwrap() as f32;
            assert_eq!(got, want, "{}.{k}", d.key);
        }
        assert_eq!(r["optional"].as_bool(), Some(d.optional));
        assert_eq!(r["doc"].as_str(), Some(d.doc));
    }
    for u in strs(&t["units"]) {
        assert!(desc::ALL.iter().any(|d| d.unit.as_str() == u), "{u}");
    }
}

/// 负例：描述表改了（这里模拟 floor_db 的上限变了）而文件没重新导出 → 检查报 params.table.json。
#[test]
fn params_change_without_export_is_red() {
    let d = temp_root("params");
    let mut v: Value = serde_json::from_str(
        &std::fs::read_to_string(d.join(DIR).join(params::TABLE_FILE)).unwrap(),
    )
    .unwrap();
    let row = v["params"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["key"] == "floor_db")
        .unwrap();
    row["max"] = serde_json::json!(6.0);
    put(&d, params::TABLE_FILE, &v);
    let p = check(&d);
    assert!(p.len() == 1 && p[0].contains(params::TABLE_FILE), "{p:#?}");
}

fn profile(key: &str, value: &str) -> String {
    format!("vdn-profile = 1\n[user cafe]\n{key} = {value}\n")
}

fn scene_table(key: &str, value: &str) -> String {
    format!("vdn-scene-table = 1\n[fine 3]\nname = a\nuser = cafe\n{key} = {value}\n")
}

/// 一个键的合法取值（造输入用）。
fn sample(key: &str) -> String {
    match key {
        "version" | "tone" => "1".into(),
        "binding" => vdn_types::ConfigBinding::default().to_value(),
        "name" => "a".into(),
        "user" => "cafe".into(),
        "model" => ModelId::NAMES[0].into(),
        k => desc::by_key(k).map_or("0".into(), |d| d.default.to_string()),
    }
}

/// 某格式某小节里放一行 `key = value` 的完整文本（[fine] 在场景表里必须有 name / user，这两行先写上）。
fn doc(format: &str, section: &str, key: &str, value: &str) -> String {
    let body = match (format, section) {
        (_, "top") => String::new(),
        (_, "user") => "[user cafe]\n".into(),
        ("vdn-scene-table", "fine") if key != "name" && key != "user" => {
            "[fine 3]\nname = a\nuser = cafe\n".into()
        }
        ("vdn-scene-table", "fine") => {
            let other = if key == "name" { "user = cafe" } else { "name = a" };
            format!("[fine 3]\n{other}\n")
        }
        _ => "[fine 3]\n".into(),
    };
    format!("{format} = 1\n{body}{key} = {value}\n")
}

fn parses(format: &str, src: &str) -> bool {
    match format {
        "vdn-scene-table" => SceneTable::from_text(src).is_ok(),
        "vdn-profile" => UserProfile::from_text(src).is_ok(),
        f => panic!("未知格式 {f}"),
    }
}

/// 各格式各小节列出的键都认；任何别处列出、这里没列的键都拒（codex 第 1 轮 #5）；
/// 参数键 = `ParamField` 的键，模型名 = `ModelId::NAMES`；拼错的键、代码里才用的参数都拒。
#[test]
fn text_keys_agree_with_parsers() {
    let t = table();
    let tf = &t["text_formats"];
    let params = strs(&tf["param_keys"]);
    let want: Vec<&str> = ParamField::ALL.iter().map(|f| f.as_str()).collect();
    assert_eq!(params, want);
    assert_eq!(strs(&tf["models"]), ModelId::NAMES);
    let formats = tf["formats"].as_object().unwrap();
    assert_eq!(
        formats.keys().map(String::as_str).collect::<Vec<_>>(),
        ["vdn-profile", "vdn-scene-table"]
    );
    let expand = |v: &Value| -> Vec<String> {
        strs(v)
            .into_iter()
            .flat_map(|k| {
                if k == "params" {
                    params.iter().map(|p| p.to_string()).collect()
                } else {
                    vec![k.to_string()]
                }
            })
            .collect()
    };
    // 所有格式所有小节出现过的键 + 几个不该认的
    let mut universe: Vec<String> = Vec::new();
    for f in formats.values() {
        for v in f.as_object().unwrap().values() {
            universe.extend(expand(v));
        }
    }
    universe.extend(["gate.attack_s", "floor", "Floor_db", "audio"].map(String::from));
    universe.sort();
    universe.dedup();
    for (fname, secs) in formats {
        let secs = secs.as_object().unwrap();
        assert_eq!(
            secs.keys().map(String::as_str).collect::<Vec<_>>(),
            ["fine", "top", "user"]
        );
        for (sec, keys) in secs {
            let ok = expand(keys);
            for k in &universe {
                let got = parses(fname, &doc(fname, sec, k, &sample(k)));
                assert_eq!(got, ok.contains(k), "{fname} [{sec}] {k}");
            }
        }
    }
    for m in ModelId::NAMES {
        assert!(parses("vdn-profile", &doc("vdn-profile", "user", "model", m)));
        assert!(parses("vdn-scene-table", &doc("vdn-scene-table", "fine", "model", m)));
    }
    assert!(!parses("vdn-profile", &doc("vdn-profile", "user", "model", "nope")));
}

/// 每个文本参数取描述表的边界外一点：两种文本都拒；边界值收下。
#[test]
fn out_of_range_values_are_rejected() {
    for f in ParamField::ALL {
        let d = f.desc();
        for bad in [d.min - 0.5, d.max + 0.5] {
            let v = bad.to_string();
            assert!(
                UserProfile::from_text(&profile(d.key, &v)).is_err(),
                "{} = {v}",
                d.key
            );
            assert!(
                SceneTable::from_text(&scene_table(d.key, &v)).is_err(),
                "{} = {v}",
                d.key
            );
        }
        for ok in [d.min, d.max] {
            let v = ok.to_string();
            UserProfile::from_text(&profile(d.key, &v)).unwrap();
            SceneTable::from_text(&scene_table(d.key, &v)).unwrap();
        }
    }
}

/// 第一行非注释、非空的内容（文件头）。
fn header(src: &str) -> Option<&str> {
    src.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .and_then(|l| l.split_once('=').map(|(k, _)| k.trim()))
}

/// 仓里已跟踪的所有这两种文本文件逐个解析通过（以后谁提交这类文件都会被查）。至少有两份示例。
#[test]
fn committed_text_files_parse() {
    let o = std::process::Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["ls-files", "-z"])
        .output()
        .expect("git ls-files");
    assert!(o.status.success());
    let mut n = [0usize; 2];
    for rel in String::from_utf8_lossy(&o.stdout).split('\0') {
        if rel.is_empty() {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(root().join(rel)) else {
            continue; // 二进制文件
        };
        match header(&src) {
            Some("vdn-scene-table") => {
                SceneTable::from_text(&src).unwrap_or_else(|e| panic!("{rel}：{e}"));
                n[0] += 1;
            }
            Some("vdn-profile") => {
                UserProfile::from_text(&src).unwrap_or_else(|e| panic!("{rel}：{e}"));
                n[1] += 1;
            }
            _ => {}
        }
    }
    assert!(n[0] >= 1 && n[1] >= 1, "示例文件不见了：{n:?}");
}
