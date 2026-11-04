//! 场景成绩单（D81）：场景测试集逐段跑 → 每段每项打分 → 按 场景 × 档 × 设备 汇总 → 找拖后腿的声音。
//!
//! - 跑哪些项、按什么顺序判：[`standard`]（正本）；规则说明：`specs/rubric/scorecard.md`
//! - 汇总规则：[`rules`]；输出 Markdown + JSON：[`card`]
//! - 任务清单是 `suite/scenes.json`（scenesuite 写，`real.kind = "scene"`）；一条任务按清单里的片段展开成多段

pub mod card;
pub mod rules;
pub mod standard;

use std::path::Path;

use crate::files::serde_name;
use crate::runner::{Extra, Opts, prepare_clip, score_one};
use crate::scene::{ClipPolarity, MeetingPlace, SceneClip, SoundGroup, Tier};
use crate::suite::{Grader, RealSource, Set, Suite, Task, main_root, resolve};
use crate::synth::{Clip, SR, Truth};
use crate::system::SystemKind;
use crate::verifiers::Params;
use crate::{Score, wav};

/// 一段在一个系统上的全部分数。
#[derive(Clone, Debug)]
pub struct ClipRow {
    pub clip: String,
    pub scene: MeetingPlace,
    pub tier: Tier,
    pub device: String,
    pub system: String,
    /// 段里含的声音：(类别键, 分组)
    pub sounds: Vec<(String, SoundGroup)>,
    /// (verifier, 是否否决项, 分数)，顺序同 [`standard::items`]
    pub scores: Vec<(&'static str, bool, Score)>,
}

/// 没跑成的一段（或一整条任务）：照样进成绩单，让该场景不能判「过」（codex 第 1 轮 P1 #1）。
#[derive(Clone, Debug, serde::Serialize)]
pub struct Missing {
    pub scene: MeetingPlace,
    pub tier: Tier,
    pub device: String,
    pub system: String,
    /// 片段 id，或整条任务读不了时的任务 id
    pub what: String,
    pub why: String,
}

/// 读一段：核 wav 哈希（混合 · 干净参考 · 噪音 · 干扰四条）；四条长度都必须等于清单的 `dur_samples`。
pub fn load_scene_clip(c: &SceneClip, root: &Path) -> Result<Clip, String> {
    let get = |rel: &str, want: &str| -> Result<Vec<f32>, String> {
        let p = root.join(rel);
        let got = wav::sha256(&p)?;
        if got != want {
            return Err(format!("{rel} 的 sha256 对不上（清单 {want}，实际 {got}）"));
        }
        let (x, sr) = wav::read(&p)?;
        if sr != SR {
            return Err(format!("{rel} 采样率 {sr}，只支持 16 kHz"));
        }
        Ok(x)
    };
    let (a, h) = (&c.audio, &c.hash.wav_sha256);
    let mix = get(&a.mix, &h.mix)?;
    let clean = get(&a.owner, &h.owner)?;
    let noise = get(&a.noise, &h.noise)?;
    let interf = get(&a.interf, &h.interf)?;
    // 长度不对就报错，不截短（截短会悄悄删掉尾部内容，codex 第 1 轮 P1 #2）
    let n = c.dur_samples as usize;
    for (k, x) in [("mix", &mix), ("owner", &clean), ("noise", &noise), ("interf", &interf)] {
        if x.len() != n {
            return Err(format!("{k} 长 {} 样本 ≠ 清单 dur_samples {n}", x.len()));
        }
    }
    let iv = |v: &[[u64; 2]]| -> Vec<(usize, usize)> {
        v.iter()
            .map(|[s, e]| (*s as usize, (*e as usize).min(n)))
            .filter(|(s, e)| s < e)
            .collect()
    };
    Ok(Clip {
        sr: SR,
        truth: Truth {
            owner: iv(&c.owner_iv),
            interf: iv(&c.interf_iv),
        },
        clean,
        interf,
        noise,
        mix,
        tracks: true,
    })
}

/// 一段的评分任务：graders 取成绩单标准，`id` = 片段 id（吃字的参考转写按它缓存）。
fn clip_task(base: &Task, c: &SceneClip) -> Task {
    let graders = standard::items(c.scene)
        .into_iter()
        .map(|(i, _)| Grader {
            v: i.v.to_string(),
            params: Params(
                i.params
                    .iter()
                    .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
                    .collect(),
            ),
            info_only: false,
        })
        .collect();
    Task {
        id: c.id.clone(),
        set: Set::Capability,
        // 任务极性的词表沿用 qa.json（suppress / preserve）；清单的 preserve_all 对到 preserve
        polarity: match c.polarity {
            ClipPolarity::Suppress => "suppress",
            ClipPolarity::PreserveAll => "preserve",
        }
        .to_string(),
        origin: base.origin.clone(),
        desc: base.desc.clone(),
        scene: None,
        real: base.real.clone(),
        lang: serde_name(&c.lang),
        safety: base.safety,
        faults: Vec::new(),
        graders,
        trials: 1,
        probe: None,
        xfail: None,
    }
}

/// 读任务对应的片段（按 场景 × 档 × 设备 筛）。
pub fn clips_of(task: &Task) -> Result<Vec<SceneClip>, String> {
    let Some(RealSource::Scene {
        clips,
        scene,
        tier,
        device,
    }) = &task.real
    else {
        return Err(format!("{} 不是场景任务", task.id));
    };
    let p = resolve(clips).ok_or_else(|| format!("找不到 {clips}"))?;
    let text = std::fs::read_to_string(&p).map_err(|e| format!("{clips}：{e}"))?;
    let all = SceneClip::parse_jsonl(&text).map_err(|e| format!("{clips}：{e}"))?;
    Ok(all
        .into_iter()
        .filter(|c| c.scene == *scene && c.tier == *tier && c.device == *device)
        .collect())
}

/// 跑全部场景任务。读不了的段、展开不出片段的任务记进 [`Missing`]（原因），不中断，也不从成绩单里消失。
pub fn run(
    suite: &Suite,
    systems: &[SystemKind],
    opts: &Opts,
    mut extra: Option<&mut dyn Extra>,
    audio_root: Option<&Path>,
) -> (Vec<ClipRow>, Vec<Missing>) {
    let root = audio_root.map_or_else(main_root, Path::to_path_buf);
    let (mut rows, mut missing) = (Vec::new(), Vec::new());
    for task in &suite.tasks {
        let whole = |why: String| -> Vec<Missing> {
            let Some(RealSource::Scene {
                scene, tier, device, ..
            }) = &task.real
            else {
                return Vec::new();
            };
            systems
                .iter()
                .map(|s| Missing {
                    scene: *scene,
                    tier: *tier,
                    device: serde_name(device),
                    system: s.as_str().to_string(),
                    what: task.id.clone(),
                    why: why.clone(),
                })
                .collect()
        };
        let clips = match clips_of(task) {
            Ok(c) if c.is_empty() => {
                missing.extend(whole("清单里没有这个 场景 × 档 × 设备 的片段".into()));
                continue;
            }
            Ok(c) => c,
            Err(e) => {
                missing.extend(whole(e));
                continue;
            }
        };
        for c in &clips {
            let Some(_) = standard::group_of(c.scene) else {
                continue;
            };
            let t = clip_task(task, c);
            let miss = |sys: SystemKind, why: String| Missing {
                scene: c.scene,
                tier: c.tier,
                device: serde_name(&c.device),
                system: sys.as_str().to_string(),
                what: c.id.clone(),
                why,
            };
            let clip = match load_scene_clip(c, &root) {
                Ok(x) => x,
                Err(e) => {
                    missing.extend(systems.iter().map(|&s| miss(s, e.clone())));
                    continue;
                }
            };
            for &sys in systems {
                let p = match prepare_clip(&t, sys, clip.clone(), c.additive) {
                    Ok(p) => p,
                    Err(e) => {
                        missing.push(miss(sys, e));
                        continue;
                    }
                };
                let items = standard::items(c.scene);
                let scores = t
                    .graders
                    .iter()
                    .zip(&items)
                    .map(|(g, (i, veto))| {
                        (
                            i.v,
                            *veto,
                            score_one(g, sys, &t, &p, &p.run.out, opts, &mut extra),
                        )
                    })
                    .collect();
                rows.push(ClipRow {
                    clip: c.id.clone(),
                    scene: c.scene,
                    tier: c.tier,
                    device: serde_name(&c.device),
                    system: sys.as_str().to_string(),
                    sounds: c.sounds.iter().map(|s| (s.cls.clone(), s.group)).collect(),
                    scores,
                });
            }
        }
    }
    (rows, missing)
}
