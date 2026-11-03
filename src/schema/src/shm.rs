//! 共享内存头 → `shm-header.layout.json`，以及「改布局必须升版本」的守卫（`HISTORY`）。

use serde_json::{Value, json};
use vdn_shm::RingId;
use vdn_shm::desc::{self, FieldDesc, Kind, StructDesc};
use vdn_shm::layout;

pub const LAYOUT_FILE: &str = "shm-header.layout.json";

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

/// 每个布局版本的指纹（只增不改；版本 1 早于本机制，没有记录）。
///
/// 故意写在代码里、不放导出文件：导出工具从不改它，改布局时必须手动在末尾加一行，
/// 改动在评审里看得见；改写或删掉旧行同样是一处显眼的代码改动（第 1 轮 codex #1）。
pub const HISTORY: &[(u32, &str)] = &[(2, "fnv1a64:fbd400fe113ede86")];

/// 守卫：`history` 版本号严格递增；当前 (版本, 指纹) 必须正是最后一行。
pub fn guard(history: &[(u32, &str)], version: u32, fp: &str) -> Result<(), String> {
    if history.windows(2).any(|w| w[0].0 >= w[1].0) {
        return Err("shm::HISTORY 的版本号必须严格递增".into());
    }
    let next = |v: u32| format!("在 src/schema/src/shm.rs 的 HISTORY 末尾加 ({v}, \"{fp}\")");
    match history.last() {
        None => Err(format!("shm::HISTORY 是空的：{}", next(version))),
        Some(&(v, f)) if v == version && f == fp => Ok(()),
        Some(&(v, f)) if v == version => Err(format!(
            "共享内存头布局变了（指纹 {f} → {fp}），但 layout::VERSION 还是 {v}：改布局必须升版本——\
             把 src/shm/src/layout.rs 的 VERSION 升到 {}，并{}",
            v + 1,
            next(v + 1)
        )),
        Some(&(v, _)) if version > v => Err(format!(
            "layout::VERSION = {version} 还没登记：{}",
            next(version)
        )),
        Some(&(v, _)) => Err(format!(
            "layout::VERSION = {version} 比已登记的 {v} 小：版本只能往上升"
        )),
    }
}

/// 从一份已提交的布局文件（可信基线：main 上的版本）算出它的 (版本, 指纹)。
pub fn baseline_entry(layout_text: &str) -> Result<(u32, String), String> {
    let mut v: Value =
        serde_json::from_str(layout_text).map_err(|e| format!("基线 {LAYOUT_FILE}：{e}"))?;
    let version = v
        .get("version")
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| format!("基线 {LAYOUT_FILE} 缺 version"))?;
    if let Value::Object(m) = &mut v {
        m.remove("$comment");
        m.remove("version");
    }
    Ok((version, fingerprint(&v)))
}

/// 历史只增不改（codex 第 2 轮 #1）：main 上已发布的 (版本, 指纹) 必须原样留在 `history` 里。
pub fn guard_baseline(history: &[(u32, &str)], base: (u32, &str)) -> Result<(), String> {
    match history.iter().find(|(v, _)| *v == base.0) {
        Some((_, f)) if *f == base.1 => Ok(()),
        Some((v, f)) => Err(format!(
            "shm::HISTORY 里版本 {v} 的指纹被改写了（main 上的布局是 {}，现在写的是 {f}）：\
             已发布的版本只能在末尾追加新版本，不能改旧行——改布局请升 layout::VERSION",
            base.1
        )),
        None => Err(format!(
            "shm::HISTORY 删掉了 main 上已发布的版本 {}：旧行不能删",
            base.0
        )),
    }
}
