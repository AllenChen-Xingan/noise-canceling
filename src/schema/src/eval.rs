//! 评测清单的 schema：从 `vdn_eval::files` 的类型派生。

use serde_json::Value;
use vdn_eval::files::{BaselineFile, LevelsFile, SuiteFile};

pub const SUITE_FILE: &str = "eval-suite.schema.json";
pub const LEVELS_FILE: &str = "eval-levels.schema.json";
pub const BASELINE_FILE: &str = "eval-baseline.schema.json";

/// 仓里受 schema 管的数据文件：(相对仓库根的路径, 用哪份 schema)。新加清单文件时登记在这里。
pub const DATA_FILES: [(&str, &str); 4] = [
    ("src/eval/suite/ci.json", SUITE_FILE),
    ("src/eval/suite/qa.json", SUITE_FILE),
    ("src/eval/suite/levels.json", LEVELS_FILE),
    ("src/eval/suite/baseline-ci.json", BASELINE_FILE),
];

const SRC: &str = "src/eval/src/files.rs 的";

pub fn suite_schema() -> Value {
    let s = schemars::schema_for!(SuiteFile);
    crate::with_comment(s.to_value(), &format!("{SRC} SuiteFile"))
}

pub fn levels_schema() -> Value {
    let s = schemars::schema_for!(LevelsFile);
    crate::with_comment(s.to_value(), &format!("{SRC} LevelsFile"))
}

pub fn baseline_schema() -> Value {
    let s = schemars::schema_for!(BaselineFile);
    crate::with_comment(s.to_value(), &format!("{SRC} BaselineFile"))
}
