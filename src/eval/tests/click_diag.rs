//! V-CLICK 诊断（人工跑，不进 CI）：检出的咔嗒落在哪里——参考降噪器的门控开关附近？STFT 帧边界上？有没有人声？
//! 用法：`VDN_CLICK_DIAG=<clips.jsonl 绝对路径> cargo test -p vdn-eval --release --test click_diag -- --ignored --nocapture`
//! （音频按主目录解析；只看 cafe · common · mba-builtin）
//! 另设 `VDN_CLICK_DUMP=<目录>`：把跳变最大的 5 处各剪 ±150 ms（输出、带噪输入、干净参考）写成 wav，供人耳核对

use std::path::PathBuf;

use vdn_eval::analysis::masks;
use vdn_eval::scene::{DeviceKey, MeetingPlace, SceneClip, Tier};
use vdn_eval::scorecard::load_scene_clip;
use vdn_eval::system::{SystemKind, SystemSpec, run};
use vdn_eval::verifiers::{Ctx, Params, click_positions};

#[test]
#[ignore = "诊断，要场景测试集音频，人工跑：见文件头"]
fn where_do_clicks_land() {
    let Ok(p) = std::env::var("VDN_CLICK_DIAG") else {
        panic!("设 VDN_CLICK_DIAG=<clips.jsonl>");
    };
    let all = SceneClip::parse_jsonl(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let root = vdn_eval::suite::main_root();
    let (mut n, mut near_gate, mut voiced, mut lag_hist) = (0usize, 0usize, 0usize, [0usize; 8]);
    let mut steps = Vec::new();
    let mut picks: Vec<(f64, String, usize, Vec<f32>, Vec<f32>, Vec<f32>)> = Vec::new();
    for c in all.iter().filter(|c| {
        c.scene == MeetingPlace::Cafe && c.tier == Tier::Common && c.device == DeviceKey::MbaBuiltin
    }) {
        let clip = load_scene_clip(c, &root).unwrap();
        let spec = SystemSpec { kind: SystemKind::Ref, safety: true, faults: Vec::new() };
        let r = run(&spec, &clip.mix).unwrap();
        let m = masks(&clip.truth, &clip.mix, 0.03);
        let ctx = Ctx { clip: &clip, out: &r.out, run: Some(&r), masks: &m, noise_out: None, ci: true };
        let pos = click_positions(&ctx, &Params::default());
        let switches: Vec<usize> = (1..r.out.len())
            .filter(|&t| r.gate_at(t) != r.gate_at(t - 1))
            .collect();
        for &t in &pos {
            n += 1;
            if switches.iter().any(|&s| s.abs_diff(t) <= r.hop) {
                near_gate += 1;
            }
            if m.owner.get(t / 160).copied().unwrap_or(false) {
                voiced += 1;
            }
            lag_hist[((t + r.latency) % r.hop) * 8 / r.hop] += 1;
            let db = 20.0 * ((r.out[t + 1] - r.out[t]).abs().max(1e-9) as f64).log10();
            steps.push(db);
            let w = 2400; // ±150 ms
            let (a, b) = (t.saturating_sub(w), (t + w).min(r.out.len()));
            picks.push((db, format!("{}-{t}", c.id), t - a, r.out[a..b].to_vec(), clip.mix[a..b].to_vec(), clip.clean[a..b].to_vec()));
        }
        eprintln!("{}：{} 个，门控切换 {} 次", c.id, pos.len(), switches.len());
    }
    steps.sort_by(f64::total_cmp);
    eprintln!("合计 {n} 个；门控切换 ±1 hop 内 {near_gate}；主讲人有声帧里 {voiced}");
    eprintln!("在 hop 内的相位分布（8 格）：{lag_hist:?}");
    if !steps.is_empty() {
        eprintln!("跳变大小 dBFS：最小 {:.1} · 中位 {:.1} · 最大 {:.1}", steps[0], steps[steps.len() / 2], steps[steps.len() - 1]);
    }
    if let Ok(d) = std::env::var("VDN_CLICK_DUMP") {
        let d = PathBuf::from(d);
        std::fs::create_dir_all(&d).unwrap();
        picks.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut list = Vec::new();
        for (db, name, at, o, m, c) in picks.into_iter().take(5) {
            for (k, x) in [("out", &o), ("mix", &m), ("clean", &c)] {
                vdn_eval::wav::write(&d.join(format!("{name}-{k}.wav")), x, vdn_types::MODEL_RATE).unwrap();
            }
            list.push(serde_json::json!({"name": name, "step_dbfs": db, "click_at_s": at as f64 / vdn_types::MODEL_RATE as f64}));
        }
        std::fs::write(d.join("list.json"), serde_json::to_string_pretty(&list).unwrap()).unwrap();
        eprintln!("已剪 5 处到 {}", d.display());
    }
}
