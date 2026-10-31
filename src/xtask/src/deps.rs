//! check-deps（规约 §5.4）：`cargo metadata` 的 `resolve.nodes[].deps[]`，只取普通依赖（`dep_kinds[].kind == null`），
//! 按包 BFS 求闭包，与 `metadata.vdn.forbid` 求交。resolve 是 feature 合并后的图，只会多报、不会漏报。

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::process::Command;

use serde_json::Value;

use crate::root;

pub struct Meta {
    pub json: Value,
}

pub fn metadata() -> Result<Meta, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = Command::new(cargo)
        .args(["metadata", "--format-version", "1"])
        .current_dir(root())
        .output()
        .map_err(|e| format!("调不起 cargo metadata：{e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    let json = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    Ok(Meta { json })
}

impl Meta {
    /// workspace 成员：(包 id, 包名, manifest 路径, metadata.vdn)
    pub fn members(&self) -> Vec<(String, String, String, Option<Value>)> {
        let ids: BTreeSet<&str> = self.json["workspace_members"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        self.json["packages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|p| p["id"].as_str().is_some_and(|id| ids.contains(id)))
            .map(|p| {
                (
                    p["id"].as_str().unwrap_or_default().to_string(),
                    p["name"].as_str().unwrap_or_default().to_string(),
                    p["manifest_path"].as_str().unwrap_or_default().to_string(),
                    p["metadata"].get("vdn").cloned(),
                )
            })
            .collect()
    }

    fn names(&self) -> HashMap<String, String> {
        self.json["packages"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| {
                (
                    p["id"].as_str().unwrap_or_default().to_string(),
                    p["name"].as_str().unwrap_or_default().to_string(),
                )
            })
            .collect()
    }

    /// 包 id → 普通依赖的包 id
    fn normal_edges(&self) -> HashMap<String, Vec<String>> {
        let mut g = HashMap::new();
        for n in self.json["resolve"]["nodes"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let id = n["id"].as_str().unwrap_or_default().to_string();
            let deps = n["deps"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|d| {
                    d["dep_kinds"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|k| k["kind"].is_null())
                })
                .filter_map(|d| d["pkg"].as_str().map(String::from))
                .collect();
            g.insert(id, deps);
        }
        g
    }

    /// 某包普通依赖闭包里的包名（不含自己）。
    pub fn closure(&self, id: &str) -> BTreeSet<String> {
        let g = self.normal_edges();
        let names = self.names();
        let mut seen = BTreeSet::new();
        let mut q = VecDeque::from([id.to_string()]);
        while let Some(cur) = q.pop_front() {
            for d in g.get(&cur).into_iter().flatten() {
                if seen.insert(d.clone()) {
                    q.push_back(d.clone());
                }
            }
        }
        seen.iter().filter_map(|i| names.get(i).cloned()).collect()
    }
}

pub fn check() -> Result<Vec<String>, String> {
    let m = metadata()?;
    let mut problems = Vec::new();
    for (id, name, _, vdn) in m.members() {
        let Some(vdn) = vdn else {
            problems.push(format!("{name}：缺 [package.metadata.vdn]（规约 §5.4）"));
            continue;
        };
        let forbid: BTreeSet<String> = vdn["forbid"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        let closure = m.closure(&id);
        for bad in closure.intersection(&forbid) {
            problems.push(format!("{name}：依赖闭包里出现了禁用的 {bad}（R12）"));
        }
    }
    Ok(problems)
}
