//! 外部离线输出：给「事先离线算好的降噪输出目录」打分（exp016 · #112；形状见
//! `docs/experiments/exp016-candidates/external-entry.md`）。
//!
//! 目录 = `run.json` + 每种输入一个子目录（子目录名 = 输入种类名，见 [`Kind`]），里面是 `<片段 id>.wav`。
//! 载入时把片段清单里每段的每种输入**按 Rust 自己读到的样本**算指纹，建「输入指纹 → 输出文件」索引；
//! `system::run(spec, x)` 只凭 `x` 的指纹找输出，不依赖调用方传片段 id。
//! 输出补回声明延迟后包成 [`Replay`] 模型，放进和内置模型同一个 `Engine`（三件保命照开）。
//!
//! 公平性三条（evalfix 审，2026-11-04）：延迟只是声明值（provenance 标 `latency_declared_only`）；
//! 回放不耗时，`engine_times` 给空（最坏一帧判「不知道」）；截半输入（`trunc`）查「偷看未来」，超差整个系统不打分。

use std::collections::{BTreeSet, HashMap};
use std::hash::{DefaultHasher, Hasher};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use vdn_engine::{Model, ModelError};

use crate::scene::SceneClip;
use crate::scorecard::load_scene_clip;
use crate::synth::{Clip, SR};
use crate::wav;

/// `run.json` 的格式版本（正本在此）。
pub const SCHEMA: &str = "vdn-external-output/1";
/// 回放的 hop（与内置模型同口径）。
const HOP: usize = vdn_types::FRONTEND.hop;
/// 截半自检的容差：浮点 wav / 16 位整型 wav。
const TRUNC_TOL_FLOAT: f32 = 1e-4;
const TRUNC_TOL_INT16: f32 = 2.0 / 32768.0;

/// 输入种类 = 子目录名。以后加 `clean`（干净人声单独过一遍，等 evalfix C2 定稿）只在这里加一支。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// 混合轨 `mix.wav`
    Mix,
    /// 噪音 + 干扰逐样本相加（runner 给「背景稳不稳」单独再跑的那一路）
    Bg,
    /// 自检用：每个场景第一段的混合轨，第 floor(n/2) 样本起补零。不进索引
    Trunc,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Mix, Kind::Bg, Kind::Trunc];
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Mix => "mix",
            Kind::Bg => "bg",
            Kind::Trunc => "trunc",
        }
    }
    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.as_str() == s)
    }
    /// 这段需不需要这种输入的输出。
    fn applies(self, c: &SceneClip, trunc_pick: bool) -> bool {
        match self {
            Kind::Mix => true,
            Kind::Bg => c.additive,
            Kind::Trunc => trunc_pick,
        }
    }
    /// 这种输入的样本（与 runner 喂给 `system::run` 的完全同一算法）。
    pub fn input(self, clip: &Clip) -> Vec<f32> {
        match self {
            Kind::Mix => clip.mix.clone(),
            Kind::Bg => clip.noise.iter().zip(&clip.interf).map(|(a, b)| a + b).collect(),
            Kind::Trunc => {
                let mut x = clip.mix.clone();
                let h = x.len() / 2;
                x[h..].fill(0.0);
                x
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunJson {
    schema: String,
    name: String,
    clips: String,
    sr: u32,
    latency_samples: usize,
    inputs: Vec<String>,
    #[serde(default)]
    model: serde_json::Value,
}

/// 输入指纹：(长度, FNV-1a 64, SipHash 64)，都按 `f32::to_bits` 算。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fingerprint(usize, u64, u64);

pub fn fingerprint(x: &[f32]) -> Fingerprint {
    let mut fnv: u64 = 0xcbf2_9ce4_8422_2325;
    let mut sip = DefaultHasher::new();
    for v in x {
        let b = v.to_bits();
        for byte in b.to_le_bytes() {
            fnv = (fnv ^ byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
        sip.write_u32(b);
    }
    Fingerprint(x.len(), fnv, sip.finish())
}

#[derive(Clone, Debug)]
struct Entry {
    id: String,
    kind: Kind,
    /// `None` = run.json 没列这种输入
    path: Option<PathBuf>,
}

/// 一个外部系统（载入并自检过）。
#[derive(Debug)]
pub struct External {
    name: String,
    latency: usize,
    index: HashMap<Fingerprint, Entry>,
    provenance: serde_json::Value,
}

impl PartialEq for External {
    fn eq(&self, o: &Self) -> bool {
        self.name == o.name
    }
}
impl Eq for External {}

/// 名字规则：`[a-z0-9-]+`，不能和内置系统撞（`is_builtin` 由 `system.rs` 给）。
pub fn check_name(name: &str, is_builtin: impl Fn(&str) -> bool) -> Result<(), String> {
    let ok = !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !ok {
        return Err(format!("外部系统名 {name:?} 不合规则 [a-z0-9-]+"));
    }
    if is_builtin(name) {
        return Err(format!("外部系统名 {name} 和内置系统撞名"));
    }
    Ok(())
}

/// 读一个输出文件并自检：采样率 · 时长 · 数值。返回 (样本, 是否 16 位整型)。
fn read_out(p: &Path, n: usize) -> Result<(Vec<f32>, bool), String> {
    let spec = hound::WavReader::open(p)
        .map_err(|e| format!("读 {}：{e}", p.display()))?
        .spec();
    let (x, sr) = wav::read(p)?;
    if sr != SR {
        return Err(format!("{} 采样率 {sr}，应为 {SR}", p.display()));
    }
    if x.len() != n {
        return Err(format!("{} 长 {} 样本 ≠ 清单 dur_samples {n}", p.display(), x.len()));
    }
    if let Some(i) = x.iter().position(|v| !v.is_finite()) {
        return Err(format!("{} 第 {i} 样本不是有限数", p.display()));
    }
    Ok((x, spec.sample_format == hound::SampleFormat::Int && spec.bits_per_sample == 16))
}

/// 每个场景挑清单顺序里第一段有干净参考的（离线脚本按同一规则挑）。
pub fn trunc_picks(clips: &[SceneClip]) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut out = BTreeSet::new();
    for c in clips.iter().filter(|c| c.has_clean_ref) {
        if seen.insert(c.scene) {
            out.insert(c.id.clone());
        }
    }
    out
}

/// 子目录里的文件必须恰好是该种输入适用的片段：缺的、多的都报错。
fn check_listing(dir: &Path, kind: Kind, want: &BTreeSet<String>) -> Result<(), String> {
    let sub = dir.join(kind.as_str());
    let mut got = BTreeSet::new();
    for e in std::fs::read_dir(&sub).map_err(|e| format!("{}：{e}", sub.display()))? {
        let f = e.map_err(|e| e.to_string())?.file_name();
        let f = f.to_string_lossy();
        if f.starts_with('.') {
            continue;
        }
        let id = f
            .strip_suffix(".wav")
            .ok_or_else(|| format!("{}/{f} 不是 wav", kind.as_str()))?;
        got.insert(id.to_string());
    }
    let extra: Vec<_> = got.difference(want).cloned().collect();
    let lack: Vec<_> = want.difference(&got).cloned().collect();
    if !extra.is_empty() {
        return Err(format!("{}/ 里有清单没有的片段：{}", kind.as_str(), extra.join(" · ")));
    }
    if !lack.is_empty() {
        return Err(format!(
            "{}/ 缺 {} 段：{}",
            kind.as_str(),
            lack.len(),
            lack.iter().take(5).cloned().collect::<Vec<_>>().join(" · ")
        ));
    }
    Ok(())
}

impl External {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn latency_samples(&self) -> usize {
        self.latency
    }
    /// 进成绩单元数据：目录 · run.json 哈希 · 声明延迟 · 截半自检结果。
    pub fn provenance(&self) -> &serde_json::Value {
        &self.provenance
    }

    /// 载入并自检。`dir` 为输出目录；`root` 为片段清单里音频相对路径的根（主目录）；
    /// `clips_of` 把 run.json 的 `clips` 字符串解析成文件路径；`is_builtin` 判内置名。
    pub fn load(
        dir: &Path,
        root: &Path,
        clips_of: impl Fn(&str) -> Option<PathBuf>,
        is_builtin: impl Fn(&str) -> bool,
    ) -> Result<External, String> {
        let rj_path = dir.join("run.json");
        let text = std::fs::read_to_string(&rj_path)
            .map_err(|e| format!("{}：{e}", rj_path.display()))?;
        let rj: RunJson =
            serde_json::from_str(&text).map_err(|e| format!("{}：{e}", rj_path.display()))?;
        if rj.schema != SCHEMA {
            return Err(format!("run.json schema {:?} ≠ {SCHEMA}", rj.schema));
        }
        check_name(&rj.name, is_builtin)?;
        if rj.sr != SR {
            return Err(format!("run.json sr {} ≠ {SR}", rj.sr));
        }
        let mut listed = BTreeSet::new();
        for s in &rj.inputs {
            let k = Kind::parse(s).ok_or_else(|| format!("run.json inputs 里有未知种类 {s}"))?;
            if !listed.insert(k) {
                return Err(format!("run.json inputs 重复列了 {s}"));
            }
        }
        for must in [Kind::Mix, Kind::Trunc] {
            if !listed.contains(&must) {
                return Err(format!("run.json inputs 必须含 {}", must.as_str()));
            }
        }
        // 目录里不许有没列的种类子目录（防 out/ 之类的旧名、拼错）
        for e in std::fs::read_dir(dir).map_err(|e| format!("{}：{e}", dir.display()))? {
            let e = e.map_err(|e| e.to_string())?;
            if e.file_type().map_err(|e| e.to_string())?.is_dir() {
                let n = e.file_name().to_string_lossy().to_string();
                if !Kind::parse(&n).is_some_and(|k| listed.contains(&k)) {
                    return Err(format!("目录里有 run.json 没列的子目录 {n}/"));
                }
            }
        }
        let cp = clips_of(&rj.clips).ok_or_else(|| format!("找不到片段清单 {}", rj.clips))?;
        let all = SceneClip::parse_jsonl(
            &std::fs::read_to_string(&cp).map_err(|e| format!("{}：{e}", cp.display()))?,
        )?;
        let clips: Vec<&SceneClip> = all.iter().filter(|c| c.has_clean_ref).collect();
        let picks = trunc_picks(&all);
        for &k in &listed {
            let want = clips
                .iter()
                .filter(|c| k.applies(c, picks.contains(&c.id)))
                .map(|c| c.id.clone())
                .collect();
            check_listing(dir, k, &want)?;
        }
        let lat = rj.latency_samples;
        let mut index = HashMap::new();
        let mut trunc_max = 0.0f32;
        for c in clips {
            let clip = load_scene_clip(c, root)?;
            let n = c.dur_samples as usize;
            let path = |k: Kind| dir.join(k.as_str()).join(format!("{}.wav", c.id));
            for k in [Kind::Mix, Kind::Bg] {
                if !k.applies(c, false) {
                    continue;
                }
                let p = listed.contains(&k).then(|| path(k));
                if let Some(p) = &p {
                    read_out(p, n)?;
                }
                let fp = fingerprint(&k.input(&clip));
                let e = Entry { id: c.id.clone(), kind: k, path: p };
                if let Some(old) = index.insert(fp, e) {
                    return Err(format!(
                        "{} 的 {} 输入与 {} 的 {} 输入样本完全相同，索引会错配",
                        c.id,
                        k.as_str(),
                        old.id,
                        old.kind.as_str()
                    ));
                }
            }
            if picks.contains(&c.id) {
                let (a, ia) = read_out(&path(Kind::Mix), n)?;
                let (b, ib) = read_out(&path(Kind::Trunc), n)?;
                let tol = if ia || ib { TRUNC_TOL_INT16 } else { TRUNC_TOL_FLOAT };
                let upto = (n / 2).saturating_sub(lat);
                let d = a[..upto]
                    .iter()
                    .zip(&b[..upto])
                    .map(|(x, y)| (x - y).abs())
                    .fold(0.0f32, f32::max);
                if d > tol {
                    return Err(format!(
                        "输出依赖声明延迟之后的输入：{} 截半后前 {upto} 样本最大差 {d:.2e} > {tol:.2e}（声明延迟 {lat} 样本）",
                        c.id
                    ));
                }
                trunc_max = trunc_max.max(d);
            }
        }
        let provenance = serde_json::json!({
            "name": rj.name,
            "dir": dir.to_string_lossy(),
            "run_json_sha256": wav::sha256(&rj_path)?,
            "inputs": listed.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            "latency_samples": lat,
            "latency_declared_only": true,
            "trunc_clips": picks.len(),
            "trunc_max_abs_diff": trunc_max,
            "model": rj.model,
        });
        Ok(External { name: rj.name, latency: lat, index, provenance })
    }

    /// 凭输入样本找对应输出，包成回放模型。找不到 / 没提供该种输入 → 报错（进「没跑成」）。
    pub fn replay(&self, x: &[f32], name: &'static str) -> Result<Replay, String> {
        let e = self
            .index
            .get(&fingerprint(x))
            .ok_or("外部输出里没有这段输入（输出目录与成绩单清单不是同一 build，或输入被改过）")?;
        let p = e
            .path
            .as_ref()
            .ok_or_else(|| format!("外部输出没提供 {}", e.kind.as_str()))?;
        let (out, _) = read_out(p, x.len())?;
        let mut raw = vec![0.0f32; self.latency];
        raw.extend_from_slice(&out);
        Ok(Replay { name, raw, pos: 0, latency: self.latency })
    }

    /// 测试用：索引里有几条。
    pub fn index_len(&self) -> usize {
        self.index.len()
    }
}

/// 回放模型：逐 hop 吐出「补回声明延迟的原始流」。
pub struct Replay {
    name: &'static str,
    raw: Vec<f32>,
    pos: usize,
    latency: usize,
}

impl Model for Replay {
    fn name(&self) -> &'static str {
        self.name
    }
    fn sample_rate(&self) -> u32 {
        SR
    }
    fn hop(&self) -> usize {
        HOP
    }
    fn latency_samples(&self) -> usize {
        self.latency
    }
    /// Engine 出故障时会复位模型；时间照样往前走，所以回放位置不回卷。
    fn reset(&mut self) {}
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), ModelError> {
        if input.len() != HOP || output.len() != HOP {
            return Err(ModelError::Shape);
        }
        for (i, o) in output.iter_mut().enumerate() {
            *o = self.raw.get(self.pos + i).copied().unwrap_or(0.0);
        }
        self.pos += HOP;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_rule() {
        let builtin = |s: &str| s == "gtcrn";
        assert!(check_name("dfn3-ll", builtin).is_ok());
        assert!(check_name("gtcrn", builtin).is_err());
        assert!(check_name("DFN3", builtin).is_err());
        assert!(check_name("a_b", builtin).is_err());
        assert!(check_name("", builtin).is_err());
    }

    #[test]
    fn kinds_round_trip_and_trunc_zeroes_second_half() {
        for k in Kind::ALL {
            assert_eq!(Kind::parse(k.as_str()), Some(k));
        }
        assert_eq!(Kind::parse("out"), None);
        let clip = Clip {
            sr: SR,
            truth: crate::synth::Truth { owner: vec![], interf: vec![] },
            clean: vec![0.0; 5],
            interf: vec![0.5; 5],
            noise: vec![0.25; 5],
            mix: vec![1.0; 5],
            tracks: true,
        };
        assert_eq!(Kind::Trunc.input(&clip), vec![1.0, 1.0, 0.0, 0.0, 0.0]);
        assert_eq!(Kind::Bg.input(&clip), vec![0.75; 5]);
    }

    #[test]
    fn fingerprint_sees_one_bit() {
        let a = vec![0.1f32; 1000];
        let mut b = a.clone();
        b[500] = f32::from_bits(b[500].to_bits() ^ 1);
        assert_ne!(fingerprint(&a), fingerprint(&b));
        assert_eq!(fingerprint(&a), fingerprint(&a.clone()));
    }

    #[test]
    fn replay_reinserts_latency_and_ignores_reset() {
        let mut r = Replay { name: "t", raw: (0..600).map(|i| i as f32).collect(), pos: 0, latency: 0 };
        let mut o = vec![0.0; HOP];
        r.process(&vec![0.0; HOP], &mut o).unwrap();
        r.reset();
        r.process(&vec![0.0; HOP], &mut o).unwrap();
        assert_eq!(o[0], HOP as f32);
    }
}
