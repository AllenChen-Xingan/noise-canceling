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

/// 文件列出的参数键 = `ParamField` 的键 = 两个解析器都认的键；模型名 = `ModelId::NAMES`；没列出的键都拒。
#[test]
fn text_keys_agree_with_parsers() {
    let t = table();
    let tf = &t["text_formats"];
    let keys = strs(&tf["param_keys"]);
    let want: Vec<&str> = ParamField::ALL.iter().map(|f| f.as_str()).collect();
    assert_eq!(keys, want);
    assert_eq!(strs(&tf["models"]), ModelId::NAMES);
    assert_eq!(strs(&tf["headers"]), ["vdn-scene-table", "vdn-profile"]);
    for k in &keys {
        let d = desc::by_key(k).unwrap();
        let v = d.default.to_string();
        UserProfile::from_text(&profile(k, &v)).unwrap_or_else(|e| panic!("{k}：{e}"));
        SceneTable::from_text(&scene_table(k, &v)).unwrap_or_else(|e| panic!("{k}：{e}"));
    }
    for m in ModelId::NAMES {
        UserProfile::from_text(&profile("model", m)).unwrap();
        SceneTable::from_text(&scene_table("model", m)).unwrap();
    }
    UserProfile::from_text(&profile("tone", "1")).unwrap();
    SceneTable::from_text(&scene_table("tone", "1")).unwrap();
    // 不是文本键的参数（只在代码里用的，如门控的 gate.attack_s）和拼错的键都拒
    for bad in ["gate.attack_s", "floor", "Floor_db", "audio"] {
        assert!(UserProfile::from_text(&profile(bad, "0")).is_err(), "{bad}");
        assert!(
            SceneTable::from_text(&scene_table(bad, "0")).is_err(),
            "{bad}"
        );
    }
    assert!(UserProfile::from_text(&profile("model", "nope")).is_err());
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
