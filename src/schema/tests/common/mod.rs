//! 各交叉校验测试文件共用的小工具（每个 `tests/*.rs` 是一个测试程序，各用其中一部分）。
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use serde_json::Value;
use vdn_schema::{DIR, check, root, write};

pub fn committed(file: &str) -> Value {
    let p = root().join(DIR).join(file);
    let t = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}：{e}", p.display()));
    serde_json::from_str(&t).unwrap()
}

/// 用仓里的 schema 校验一份 JSON；返回 Err(原因)。
pub fn validate(schema: &Value, inst: &Value) -> Result<(), String> {
    let mut schemas = boon::Schemas::new();
    let mut c = boon::Compiler::new();
    c.add_resource("file:///schema.json", schema.clone())
        .map_err(|e| e.to_string())?;
    let idx = c
        .compile("file:///schema.json", &mut schemas)
        .map_err(|e| e.to_string())?;
    schemas.validate(inst, idx).map_err(|e| e.to_string())
}

/// 临时仓库根：只放 `specs/schema/`，由当前 Rust 正本导出。
pub fn temp_root(tag: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("schema-{tag}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    write(&d).unwrap();
    assert!(check(&d).is_empty(), "{:#?}", check(&d));
    d
}

pub fn put(root: &Path, file: &str, v: &Value) {
    std::fs::write(root.join(DIR).join(file), vdn_schema::pretty(v)).unwrap();
}

/// 模拟「Rust 加了一个字段、没重新导出」：把当前导出删掉这一个字段当作仓里的旧文件，
/// 其余（标题、注释、别的字段）都不变，所以报红只能来自这一个字段（第 1 轮 codex #4）。
pub fn assert_missing_field_is_red(tag: &str, file: &str, pointer: &str, field: &str) {
    let d = temp_root(tag);
    let fresh: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join(DIR).join(file)).unwrap()).unwrap();
    let mut old = fresh.clone();
    let props = old
        .pointer_mut(pointer)
        .and_then(Value::as_object_mut)
        .unwrap_or_else(|| panic!("{file} 里没有 {pointer}"));
    assert!(
        props.remove(field).is_some(),
        "{file}{pointer} 里没有 {field}"
    );
    if let Some(Value::Array(req)) = old
        .pointer_mut(pointer.trim_end_matches("/properties"))
        .and_then(|o| o.get_mut("required"))
    {
        req.retain(|r| r != field);
    }
    put(&d, file, &old);
    let p = check(&d);
    assert_eq!(p.len(), 1, "只该报这一份：{p:#?}");
    assert!(p[0].contains(file) && p[0].contains("不一致"), "{p:#?}");
    // 放回去就绿
    put(&d, file, &fresh);
    assert!(check(&d).is_empty());
}

/// 往返的比较口径：null 与缺省等价、空数组与缺省等价（serde 写出 `None` 为 null、空 Vec 为 []）。
pub fn norm(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(_, x)| !x.is_null() && x.as_array().is_none_or(|a| !a.is_empty()))
                .map(|(k, x)| (k.clone(), norm(x)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(norm).collect()),
        x => x.clone(),
    }
}

/// 数字的口径：两边都是整数就精确比（种子是 u64，不能按 f64 比，codex 第 2 轮 #3）；
/// 有一边是小数（文件写 3、f64 字段写出 3.0）才按 f64 比。
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            if x.is_f64() || y.is_f64() {
                x.as_f64() == y.as_f64()
            } else {
                x == y
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| same(v, w)))
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(v, w)| same(v, w))
        }
        _ => a == b,
    }
}

pub fn roundtrip<T: serde::de::DeserializeOwned + serde::Serialize>(path: &str, v: &Value) {
    let t: T = serde_json::from_value(v.clone()).unwrap_or_else(|e| panic!("{path}：{e}"));
    let out = serde_json::to_value(&t).unwrap();
    assert!(same(&norm(v), &norm(&out)), "{path}：往返后语义不同");
    // 写出的结果还能读回，再写一次不变
    let t2: T = serde_json::from_value(out.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(&t2).unwrap(),
        out,
        "{path}：第二次往返变了"
    );
}
