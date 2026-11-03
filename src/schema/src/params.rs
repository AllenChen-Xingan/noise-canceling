//! 参数描述表 → `params.table.json`（`vdn_types::desc` 是正本；核心 crate 不引 serde，这里转 JSON）。
//!
//! 导出时对 `ParamDesc` 穷举解构：描述表加了字段而这里没导出，编译失败。
//! 文件还写出两种文本格式（场景参数表 `vdn-scene-table`、我的配置 `vdn-profile`）里能出现的参数键，由 `ParamField` 派生。

use serde_json::{Number, Value, json};
use vdn_types::desc::{self, ParamDesc};
use vdn_types::{ModelId, ParamField};

pub const TABLE_FILE: &str = "params.table.json";

/// f32 按最短往返写法写出（`0.002` 不写成 `0.0020000000949949026`）。描述表的数都是有限数（desc 的测试保证）。
pub fn num(v: f32) -> Value {
    format!("{v}")
        .parse::<f64>()
        .ok()
        .and_then(Number::from_f64)
        .map_or(Value::Null, Value::Number)
}

fn entry(d: &ParamDesc) -> Value {
    // 穷举解构：ParamDesc 加字段 → 这里编译失败，提醒导出
    let ParamDesc {
        id,
        key,
        unit,
        min,
        max,
        default,
        optional,
        doc,
    } = *d;
    json!({
        "id": id,
        "key": key,
        "unit": unit.as_str(),
        "min": num(min),
        "max": num(max),
        "default": num(default),
        "optional": optional,
        "doc": doc,
    })
}

/// 表里用到的单位（`Unit::as_str`），按第一次出现的顺序。
pub fn units() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for d in desc::ALL {
        let u = d.unit.as_str();
        if !out.contains(&u) {
            out.push(u);
        }
    }
    out
}

pub fn table_doc() -> Value {
    json!({
        "$comment": "由 `cargo run -p vdn-schema` 从 src/types/src/desc/（参数描述表）导出，勿手改。区间是闭区间；optional = 使用方是 Option，出厂缺省",
        "units": units(),
        "params": desc::ALL.iter().map(|d| entry(d)).collect::<Vec<_>>(),
        "text_formats": text_formats(),
    })
}

/// 两种文本格式里每个小节能写的键（`params` = 全部 `param_keys`）。小节 `top` = 文件头之后、第一个小节之前。
/// 这是解析器（`table.rs` · `memory.rs`）行为的第二处表示，由 `tests/text_formats.rs` 逐键核对「列出的都认、别处列出而这里没列的都拒」。
pub fn text_formats() -> Value {
    json!({
        "comment": "场景参数表与我的配置里各小节能写的键。\"params\" 指 param_keys 的全部；参数范围见 params；model 取 models 之一",
        "param_keys": ParamField::ALL.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
        "models": ModelId::NAMES,
        "formats": {
            "vdn-scene-table": {
                "top": ["version"],
                "user": ["model", "tone", "params"],
                "fine": ["name", "user", "model", "tone", "params"],
            },
            "vdn-profile": {
                "top": ["version", "binding"],
                "user": ["model", "tone", "params"],
                "fine": ["model", "tone", "params"],
            },
        },
    })
}
