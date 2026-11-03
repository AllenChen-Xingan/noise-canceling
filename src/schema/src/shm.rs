//! 共享内存头 → `shm-header.layout.json`，以及「改布局必须升版本」的守卫（`shm-header.versions.json`）。

use std::collections::BTreeMap;

use serde_json::{Value, json};
use vdn_shm::RingId;
use vdn_shm::desc::{self, FieldDesc, Kind, StructDesc};
use vdn_shm::layout;

pub const LAYOUT_FILE: &str = "shm-header.layout.json";
pub const VERSIONS_FILE: &str = "shm-header.versions.json";

fn kind_name(k: Kind) -> String {
    match k {
        Kind::U32 => "u32".into(),
        Kind::U64 => "u64".into(),
        Kind::Struct(s) => s.name.into(),
        Kind::Array(s, n) => format!("{}[{n}]", s.name),
    }
}

fn struct_json(s: &StructDesc) -> Value {
    json!({
        "name": s.name,
        "size": s.size,
        "align": s.align,
        "fields": s.fields.iter().map(|f| json!({
            "name": f.name,
            "offset": f.offset,
            "size": f.size,
            "type": kind_name(f.kind),
        })).collect::<Vec<_>>(),
    })
}

/// 展开成从头部起点算的原始字段（给 C / 别的语言直接对表）。
fn flatten(prefix: &str, base: usize, fields: &[FieldDesc], out: &mut Vec<Value>) {
    for f in fields {
        let path = if prefix.is_empty() {
            f.name.to_string()
        } else {
            format!("{prefix}.{}", f.name)
        };
        let at = base + f.offset;
        match f.kind {
            Kind::U32 | Kind::U64 => out.push(json!({
                "path": path, "offset": at, "size": f.size, "type": kind_name(f.kind),
            })),
            Kind::Struct(s) => flatten(&path, at, s.fields, out),
            Kind::Array(s, n) => {
                for i in 0..n {
                    flatten(&format!("{path}[{i}]"), at + i * s.size, s.fields, out);
                }
            }
        }
    }
}

/// 布局本身（不含版本号）：指纹只算这一部分。
pub fn layout_body() -> Value {
    let mut flat = Vec::new();
    flatten("", 0, desc::HEADER.fields, &mut flat);
    json!({
        "magic": format!("{:#018x}", layout::MAGIC),
        "endian": "little",
        "header_bytes": layout::HEADER_BYTES,
        "slot_bytes": layout::SLOT_BYTES,
        "slot_type": "u32（f32::to_bits）",
        "rings": RingId::ALL.iter().map(|r| format!("{r:?}")).collect::<Vec<_>>(),
        "max_capacity_frames": layout::MAX_CAPACITY_FRAMES,
        "structs": desc::STRUCTS.iter().map(|s| struct_json(s)).collect::<Vec<_>>(),
        "flat": flat,
    })
}

/// 导出的完整文件：版本号 + 布局。
pub fn layout_doc(version: u32, body: &Value) -> Value {
    let mut doc = json!({
        "$comment": "由 `cargo run -p vdn-schema` 从 src/shm/src/layout.rs（经 src/shm/src/desc.rs）导出，勿手改。改布局必须升 layout::VERSION",
        "version": version,
    });
    if let (Value::Object(d), Value::Object(b)) = (&mut doc, body) {
        d.extend(b.clone());
    }
    doc
}

/// 布局指纹：FNV-1a 64 位，算在紧凑 JSON（键已排序）上。
pub fn fingerprint(body: &Value) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in body.to_string().bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("fnv1a64:{h:016x}")
}

/// 版本 → 指纹 的登记表（只增不改）。
pub type Versions = BTreeMap<u32, String>;

pub fn parse_versions(text: &str) -> Result<Versions, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("{VERSIONS_FILE}：{e}"))?;
    let mut out = Versions::new();
    for (k, f) in v
        .get("versions")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{VERSIONS_FILE} 缺 versions"))?
    {
        let n: u32 = k.parse().map_err(|_| format!("{VERSIONS_FILE}：版本号 {k:?} 不是整数"))?;
        let f = f.as_str().ok_or_else(|| format!("{VERSIONS_FILE}：{k} 的指纹不是字符串"))?;
        out.insert(n, f.to_string());
    }
    Ok(out)
}

pub fn versions_text(v: &Versions) -> String {
    let doc = json!({
        "$comment": "共享内存头：每个布局版本的指纹（只增不改）。由 `cargo run -p vdn-schema` 追加；同一版本号对应的布局变了，导出与测试都报错，提示升 layout::VERSION",
        "versions": v.iter().map(|(k, f)| (k.to_string(), json!(f))).collect::<serde_json::Map<_, _>>(),
    });
    crate::pretty(&doc)
}

/// 守卫：当前版本要么没登记过（新版本，导出时追加），要么登记的指纹与当前布局一致。
/// `require_registered` = true 时（检查模式）没登记也算错。
pub fn guard(
    versions: &Versions,
    version: u32,
    fp: &str,
    require_registered: bool,
) -> Result<(), String> {
    match versions.get(&version) {
        Some(f) if f == fp => Ok(()),
        Some(f) => Err(format!(
            "共享内存头布局变了（指纹 {f} → {fp}），但 layout::VERSION 还是 {version}：改布局必须升版本（src/shm/src/layout.rs）"
        )),
        None if require_registered => Err(format!(
            "{VERSIONS_FILE} 里没有版本 {version}：跑 `cargo run -p vdn-schema` 登记"
        )),
        None => Ok(()),
    }
}
