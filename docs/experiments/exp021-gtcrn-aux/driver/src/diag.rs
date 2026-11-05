//! `diag` 子命令：一段上对比「真 GTCRN + 默认保命」（成绩单原路径）与「原始输出回放 + 默认预混」，
//! 报引擎计数与第一处不同的样本。只做排查，不写音频。

use exp021_remix::{ReplayBuf, run_engine};
use vdn_engine::{Engine, Model, SafetyConfig};
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::load_scene_clip;
use vdn_eval::suite::{main_root, resolve};
use vdn_eval::system::external::Kind;
use vdn_models::ModelKind;

fn drive<M: Model>(m: M, x: &[f32], cfg: SafetyConfig) -> (Vec<f32>, vdn_engine::EngineStats) {
    let mut eng = Engine::new(m, cfg);
    let (hop, lat) = (eng.hop(), eng.latency_samples());
    let n = (x.len() + lat).div_ceil(hop) * hop;
    let mut xp = vec![0.0f32; n];
    xp[..x.len()].copy_from_slice(x);
    let mut raw = vec![0.0f32; n];
    for (a, b) in xp.chunks_exact(hop).zip(raw.chunks_exact_mut(hop)) {
        eng.process(a, b);
    }
    (raw[lat..lat + x.len()].to_vec(), eng.stats())
}

pub fn run(clips_rel: &str, id: &str, kind: &str) -> Result<(), String> {
    let cp = resolve(clips_rel).ok_or("找不到片段清单")?;
    let clips = SceneClip::parse_jsonl(&std::fs::read_to_string(&cp).map_err(|e| e.to_string())?)?;
    let c = clips.iter().find(|c| c.id == id).ok_or("没有这段")?;
    let clip = load_scene_clip(c, &main_root())?;
    let x = Kind::parse(kind).ok_or("未知种类")?.input(&clip);
    let mk = ModelKind::Gtcrn;
    let path = mk.default_path().ok_or("GTCRN 权重不在本机")?;
    let open = || mk.open(&path).map_err(|e| e.to_string());
    let lat = open()?.latency_samples();
    let (a, sa) = drive(open()?, &x, SafetyConfig::default());
    let y = run_engine(open()?, &x, SafetyConfig::off());
    let (b, sb) = drive(ReplayBuf::new(&y, lat), &x, SafetyConfig::default());
    // 假设验证：回放时把模型真实的前 lat 个预热样本补回去（外部目录格式只存对齐后的输出，这段补零）
    let full = {
        let mut eng = Engine::new(open()?, SafetyConfig::off());
        let n = (x.len() + lat).div_ceil(160) * 160;
        let mut xp = vec![0.0f32; n];
        xp[..x.len()].copy_from_slice(&x);
        let mut raw = vec![0.0f32; n];
        for (p, q) in xp.chunks_exact(160).zip(raw.chunks_exact_mut(160)) {
            eng.process(p, q);
        }
        raw
    };
    let head = &full[..lat];
    println!("真实预热段最大幅度 {:e}", head.iter().fold(0.0f32, |m, v| m.max(v.abs())));
    let (c2, _) = drive(ReplayBuf::with_head(head, &y), &x, SafetyConfig::default());
    let d2 = a.iter().zip(&c2).filter(|(p, q)| p != q).count();
    println!("补回预热段后与 A 不同的样本数：{d2}");
    println!("A 真模型 + 默认保命：{sa:?}");
    println!("B 回放 + 默认保命：  {sb:?}");
    let first = a.iter().zip(&b).position(|(p, q)| p != q);
    let maxd = a.iter().zip(&b).map(|(p, q)| (p - q).abs()).fold(0.0f32, f32::max);
    println!("第一处不同：{first:?}（hop {:?}）· 最大差 {maxd:e} · 段长 {}", first.map(|i| (i + lat) / 160), x.len());
    Ok(())
}
