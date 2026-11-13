//! tidy 的「频谱只算在一处」（R-FFT；D82「所有频谱计算都放在一个地方」；CLAUDE.md 第五节第 5 条；#102）。
//!
//! | 规则 | 查什么 | 放行 |
//! |---|---|---|
//! | R-FFT | `src/**/*.rs` 的代码里出现 FFT 计划的标识符（[`BANNED`]：`RealFftPlanner`、rustfft 的各种 `FftPlanner*`、`plan_fft_forward` / `plan_fft_inverse` / `plan_fft`），**测试与示例也一样** | 统一模块 [`HOME`] 之内；[`ALLOW`] 白名单（每条写原因，并限定只许哪几个标识符） |
//!
//! 白名单只许缩短：条目指向的文件不在了，或文件里已经没有命中，也报红（提醒删掉这一条）。
//!
//! 边界（写明，免得以为查全了）：
//! - 只认 FFT 计划的**标识符**（按语法树取 token，含宏参数；注释、字符串、文档注释不算）。用 `use … as` 改名后
//!   再用新名字，`use` 那一行仍会命中，所以改名绕不过去
//! - **认不出手写的窗公式**（`0.5 - 0.5 * cos(…)`）：`cos` 在合成信号、包络、滤波器设计里都有正当用途，按文本认误报太多。
//!   窗公式只写在 `src/dsp/src/window.rs` 靠评审守，不靠本规则
//! - 认不出手写的 DFT（不经 FFT 库自己写求和）；只扫 `src/`，不扫 `docs/experiments/`（那里没有 FFT 依赖）

use std::fs;
use std::path::Path;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

use crate::tidy::rs_files;

/// 统一模块：频谱计算只许在这里（`vdn-dsp` 的 `fft.rs` · `stft.rs` 及其单元测试）。
pub const HOME: &str = "src/dsp/src/";

/// FFT 计划的标识符。
pub const BANNED: [&str; 10] = [
    "RealFftPlanner",
    "FftPlanner",
    "FftPlannerScalar",
    "FftPlannerSse",
    "FftPlannerAvx",
    "FftPlannerNeon",
    "FftPlannerWasmSimd",
    "plan_fft_forward",
    "plan_fft_inverse",
    "plan_fft",
];

/// 白名单：(文件, 这个文件只许出现的标识符, 原因)。不在第二列的标识符照样报红（例外只放行声明过的那一种计划）。
pub const ALLOW: [(&str, &[&str], &str); 1] = [(
    "src/scene/src/features.rs",
    &["RealFftPlanner", "plan_fft_inverse"],
    "逆 FFT：功率谱 → 自相关求谐波度，不是分析频谱（013 第 5 节：留着）",
)];

fn walk(ts: TokenStream, out: &mut Vec<(usize, String)>) {
    for t in ts {
        match t {
            TokenTree::Ident(id) => {
                let s = id.to_string();
                if BANNED.contains(&s.as_str()) {
                    out.push((id.span().start().line, s));
                }
            }
            TokenTree::Group(g) => walk(g.stream(), out),
            _ => {}
        }
    }
}

/// 一个文件里命中的 (行, 标识符)，按行排序。
pub fn hits(file: &syn::File) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    walk(file.to_token_stream(), &mut out);
    out.sort();
    out
}

/// 一个文件的问题（含白名单判断）。文件解析不了一律报（白名单文件也报，不静默跳过）。
pub fn fft_problems(rel: &str, src: &str) -> Vec<String> {
    if rel.starts_with(HOME) {
        return Vec::new();
    }
    let file = match crate::ast::parse_src(src) {
        Ok(f) => f,
        Err(e) => {
            return vec![format!(
                "{rel}:{}：语法树解析失败：{}（R-FFT）",
                e.line, e.msg
            )];
        }
    };
    let found = hits(&file);
    let entry = ALLOW.iter().find(|(p, _, _)| *p == rel);
    if entry.is_some() && found.is_empty() {
        return vec![format!(
            "{rel}：在 R-FFT 白名单里但已没有 FFT 计划，删掉 fft_site.rs 里这一条"
        )];
    }
    let ok: &[&str] = entry.map_or(&[], |(_, ids, _)| ids);
    found
        .into_iter()
        .filter(|(_, id)| !ok.contains(&id.as_str()))
        .map(|(l, id)| {
            format!(
                "{rel}:{l}：自建 FFT 计划 `{id}`（R-FFT）——频谱计算只在 {HOME}，改调 vdn_dsp::FrameFft / StreamingStft / power_into"
            )
        })
        .collect()
}

pub fn check(root: &Path, problems: &mut Vec<String>) {
    if !root.join(HOME).join("fft.rs").exists() {
        problems.push(format!("{HOME}fft.rs 不存在：频谱统一模块（R-FFT）"));
    }
    let mut all = Vec::new();
    rs_files(&root.join("src"), &mut all);
    for f in &all {
        let Ok(rel) = f.strip_prefix(root).map(|p| p.display().to_string()) else {
            continue;
        };
        match fs::read_to_string(f) {
            Ok(src) => problems.extend(fft_problems(&rel, &src)),
            Err(e) => problems.push(format!("读 {rel}：{e}（R-FFT）")),
        }
    }
    for (p, _, _) in ALLOW {
        if !root.join(p).exists() {
            problems.push(format!("{p}：R-FFT 白名单里的文件不存在，删掉这一条"));
        }
    }
}

#[cfg(test)]
#[path = "fft_site_tests.rs"]
mod tests;
