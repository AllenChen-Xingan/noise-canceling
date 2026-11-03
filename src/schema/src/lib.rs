//! vdn-schema：把 Rust 类型导出成 schema 文件（`specs/schema/`），并检查仓里的文件与正本一致。
//!
//! 机制（specs/schema/README.md）：
//! - **正本是 Rust 类型**。有 serde 的（C 档：`vdn-control`、`vdn-eval`）用 `schemars` 派生 JSON Schema（draft 2020-12）；
//!   核心 crate 不引 serde，字段表写成本 crate 的常量（如 `vdn_shm::desc`），这里转成 JSON
//! - `cargo run -p vdn-schema` 重新导出并写盘；`cargo run -p vdn-schema -- --check` 只比对
//! - 本 crate 的测试（随 `cargo test --workspace` 和 `cargo run -p xtask -- ci` 跑）逐字比对：改了 Rust 不重新导出 → 红
//! - 不在构建时生成代码（规约 ADR 004）：导出物进仓，靠测试保证一致

#![forbid(unsafe_code)]

pub mod eval;
pub mod shm;

use std::path::{Path, PathBuf};

use serde_json::Value;

/// schema 文件所在目录（相对仓库根）。
pub const DIR: &str = "specs/schema";

/// 一份导出物：文件名（在 `DIR` 下）+ 全文。
pub struct Doc {
    pub file: &'static str,
    pub text: String,
}

/// 仓库根（本 crate 在 `src/schema/`）。
pub fn root() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.ancestors().nth(2).map(PathBuf::from).unwrap_or(here)
}

/// 统一的写法：两格缩进 + 末尾换行（键顺序由 serde_json 的有序 Map 决定，稳定）。
pub fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).unwrap_or_default() + "\n"
}

/// 给派生出的 schema 加「从哪导出、勿手改」的说明。
pub fn with_comment(mut schema: Value, source: &str) -> Value {
    if let Value::Object(m) = &mut schema {
        m.insert(
            "$comment".into(),
            Value::String(format!(
                "由 `cargo run -p vdn-schema` 从 {source} 导出，勿手改"
            )),
        );
    }
    schema
}

/// `vdn-control` 的 `Config`。
pub fn config_schema() -> Value {
    let s = schemars::schema_for!(vdn_control::Config);
    with_comment(s.to_value(), "src/control/src/lib.rs 的 Config")
}

/// 全部导出物。
pub fn export() -> Vec<Doc> {
    let body = shm::layout_body();
    vec![
        Doc {
            file: shm::LAYOUT_FILE,
            text: pretty(&shm::layout_doc(vdn_shm::layout::VERSION, &body)),
        },
        Doc {
            file: "config.schema.json",
            text: pretty(&config_schema()),
        },
        Doc {
            file: eval::SUITE_FILE,
            text: pretty(&eval::suite_schema()),
        },
        Doc {
            file: eval::LEVELS_FILE,
            text: pretty(&eval::levels_schema()),
        },
        Doc {
            file: eval::BASELINE_FILE,
            text: pretty(&eval::baseline_schema()),
        },
    ]
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// 比对仓里的文件与重新导出的结果；返回问题清单（空 = 一致）。
pub fn check(root: &Path) -> Vec<String> {
    check_with(root, shm::HISTORY)
}

/// 同 `check`，版本历史由调用方给（测试用它模拟「布局变了」）。
pub fn check_with(root: &Path, history: &[(u32, &str)]) -> Vec<String> {
    let dir = root.join(DIR);
    let mut problems = Vec::new();
    let docs = export();
    for d in &docs {
        match read(&dir.join(d.file)) {
            None => problems.push(format!(
                "{DIR}/{} 不存在：跑 `cargo run -p vdn-schema`",
                d.file
            )),
            Some(t) if t != d.text => problems.push(first_diff(d.file, &t, &d.text)),
            Some(_) => {}
        }
    }
    // 目录里多出来的 json（改名或删了类型后留下的旧文件）也算不一致
    let known: Vec<&str> = docs
        .iter()
        .map(|d| d.file)
        .collect();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.ends_with(".json") && !known.contains(&name.as_str()) {
                problems.push(format!(
                    "{DIR}/{name} 不是任何 Rust 类型导出的：删掉，或在 vdn-schema 里登记"
                ));
            }
        }
    }
    if let Err(e) = shm::guard(
        history,
        vdn_shm::layout::VERSION,
        &shm::fingerprint(&shm::layout_body()),
    ) {
        problems.push(e);
    }
    problems
}

/// 第一处不同的行，方便定位。
pub fn first_diff(file: &str, committed: &str, fresh: &str) -> String {
    let line = committed
        .lines()
        .zip(fresh.lines())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| committed.lines().count().min(fresh.lines().count()));
    format!(
        "{DIR}/{file} 与 Rust 正本不一致（第 {} 行起）：改了类型就跑 `cargo run -p vdn-schema` 重新导出并提交",
        line + 1
    )
}

/// 重新导出并写盘（只写有变化的文件）。返回写了哪些文件。
pub fn write(root: &Path) -> Result<Vec<String>, String> {
    write_with(root, shm::HISTORY)
}

/// 同 `write`，版本历史由调用方给。
pub fn write_with(root: &Path, history: &[(u32, &str)]) -> Result<Vec<String>, String> {
    let dir = root.join(DIR);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // 先过版本守卫：布局变了而版本没升（或新版本没登记），一个文件都不写
    shm::guard(
        history,
        vdn_shm::layout::VERSION,
        &shm::fingerprint(&shm::layout_body()),
    )?;
    let mut wrote = Vec::new();
    for d in export() {
        let p = dir.join(d.file);
        if read(&p).as_deref() != Some(d.text.as_str()) {
            std::fs::write(&p, &d.text).map_err(|e| format!("{}：{e}", p.display()))?;
            wrote.push(d.file.to_string());
        }
    }
    Ok(wrote)
}
