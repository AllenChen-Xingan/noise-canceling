//! 早期回声参考（D99 第 2 条）：exp013 新 build 多一条 `owner_early.wav`——主说话人只留直达声 + 前
//! `EARLY_REVERB_MS`（正本 exp013 `rooms.py`）混响、过同样的设备链。成绩单里「拿输出和干净参考比」的项
//! （[`standard::EARLY_REF`]）用它再算一遍，记成分数的旁证 [`EARLY_MARK`]；主列仍是全混响参考 `owner.wav`，
//! 判定规则不变（只看主列）。早于它的 build 没有这条轨：不算旁列，其余照旧。
//!
//! QA 层项（吃字、两个评分模型）本批不算旁列：它们的参考转写 / 参考分按任务缓存，换参考要另跑一遍模型（上云再做）。

use std::path::Path;

use super::standard;
use crate::Score;
use crate::runner::Prepared;
use crate::scene::SceneClip;
use crate::synth::SR;
use crate::wav;

/// 分数旁证名：同一项用早期回声参考算出的值（算不出为 NaN，写进 clips.jsonl 为 null）。
pub const EARLY_MARK: &str = "early_ref";

/// 读早期回声参考：清单没有这条轨 → `Ok(None)`；有就核 wav 哈希、采样率、长度（同 [`super::load_scene_clip`] 的规矩）。
pub fn load_early(c: &SceneClip, root: &Path) -> Result<Option<Vec<f32>>, String> {
    let (Some(rel), want) = (&c.audio.owner_early, &c.hash.wav_sha256.owner_early) else {
        return Ok(None);
    };
    let want = want
        .as_ref()
        .ok_or_else(|| "清单有 audio.owner_early 但没有它的 wav_sha256".to_string())?;
    let p = root.join(rel);
    let got = wav::sha256(&p)?;
    if &got != want {
        return Err(format!("{rel} 的 sha256 对不上（清单 {want}，实际 {got}）"));
    }
    let (x, sr) = wav::read(&p)?;
    if sr != SR {
        return Err(format!("{rel} 采样率 {sr}，只支持 16 kHz"));
    }
    if x.len() as u64 != c.dur_samples {
        return Err(format!(
            "owner_early 长 {} 样本 ≠ 清单 dur_samples {}",
            x.len(),
            c.dur_samples
        ));
    }
    Ok(Some(x))
}

/// 同一次系统运行、只把干净参考换成早期回声参考的评分上下文（掩码来自真值与混合轨，不随参考变）。
pub(crate) fn swap_ref(p: &Prepared, early: &[f32]) -> Prepared {
    let mut clip = p.clip.clone();
    clip.clean = early.to_vec();
    Prepared {
        clip,
        run: p.run.clone(),
        noise_out: p.noise_out.clone(),
        clean_out: p.clean_out.clone(),
        masks: p.masks.clone(),
    }
}

/// 这一项要不要算早期参考旁列。
pub fn wants(v: &str) -> bool {
    standard::EARLY_REF.contains(&v)
}

/// 把早期参考算出的值挂到主分数上。
pub fn attach(main: Score, early: &Score) -> Score {
    main.with_extra(EARLY_MARK, early.value.unwrap_or(f64::NAN))
}

/// 主分数上挂着的早期参考值（没有或算不出为 `None`）。
pub fn value_of(s: &Score) -> Option<f64> {
    s.extra
        .iter()
        .find(|(k, _)| *k == EARLY_MARK)
        .map(|(_, v)| *v)
        .filter(|v| v.is_finite())
}

#[cfg(test)]
#[path = "early_tests.rs"]
mod tests;
