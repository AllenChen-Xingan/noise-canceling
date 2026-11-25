//! 只按缓存重算吃字（V-EAT-ASR）：不重跑 DSP、不调识别接口（大总管 2026-11-25）。
//!
//! 用旧运行留下的 `qa/asr-wav/<名字>-<16 位十六进制>.wav`（[`super::QaExtra::dump`] 写的）和 `qa/asr_cache.json`。
//! 打分走 [`super::QaExtra::eat_on`]，和整套重跑同一段代码；识别后端换成只读缓存的
//! [`super::asr::Backend::Offline`]，缓存缺一条就记一次，调用方据此拒绝把结果当整套重跑用。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Audio, QaExtra};
use crate::Score;
use crate::suite::Task;
use crate::verifiers::Params;

/// 旧运行落盘音频的索引：名字 → 文件（一般恰好一个）。
pub struct WavIndex(BTreeMap<String, Vec<PathBuf>>);

impl WavIndex {
    /// 读 `dir`（旧运行的 `qa/asr-wav`）。文件名不是 `<名字>-<16 位十六进制>.wav` 的忽略。
    pub fn read(dir: &Path) -> Result<WavIndex, String> {
        let mut m: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        let it = std::fs::read_dir(dir).map_err(|e| format!("读不了旧运行的 qa/asr-wav：{e}"))?;
        for e in it {
            let e = e.map_err(|e| format!("读不了旧运行的 qa/asr-wav：{e}"))?;
            let f = e.file_name().to_string_lossy().into_owned();
            if let Some((name, hex)) = f.strip_suffix(".wav").and_then(|s| s.rsplit_once('-'))
                && hex.len() == 16
                && hex.bytes().all(|b| b.is_ascii_hexdigit())
            {
                m.entry(name.to_string()).or_default().push(e.path());
            }
        }
        Ok(WavIndex(m))
    }

    /// 找 `name` 的落盘音频：恰好一份才算找到，返回路径与 sha256。
    pub fn find(&self, name: &str) -> Result<(PathBuf, String), String> {
        match self.0.get(name).map(Vec::as_slice) {
            Some([p]) => Ok((p.clone(), crate::wav::sha256(p)?)),
            None | Some([]) => Err(format!("旧运行里没有 {name} 的落盘音频")),
            Some(v) => Err(format!("{name} 有 {} 份落盘音频，认不出是哪一份", v.len())),
        }
    }
}

/// 重算一行的吃字：三条音频（输出 · 干净人声 · 原声）都要找得到，否则 `Err`（不当成「不知道」糊过去）。
pub fn eat_from_files(
    q: &mut QaExtra,
    sys: &str,
    task: &Task,
    p: &Params,
    idx: &WavIndex,
) -> Result<Score, String> {
    let mut audio: Audio = Vec::new();
    for n in [
        format!("{}-{sys}", task.id),
        format!("{}-clean", task.id),
        format!("{}-mix", task.id),
    ] {
        let r = idx.find(&n)?;
        audio.push((n, Ok(r)));
    }
    Ok(q.eat_on(sys, task, p, &audio))
}

/// 本次重算里缓存没命中的次数（> 0 = 结果不能当整套重跑用）。
pub fn misses(q: &QaExtra) -> usize {
    q.asr.as_ref().map_or(0, |a| a.offline_misses)
}
