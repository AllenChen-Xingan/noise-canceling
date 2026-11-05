//! 外部离线输出入口（exp016 · #112）端到端：自造合成片段 → 写外部输出目录 → `External::load` 自检 → 成绩单。
//! 主测试：内置 `passthrough` 的模型输出写成文件，走外部入口打分，与内置成绩单逐位一致。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vdn_eval::runner::{Baseline, Opts};
use vdn_eval::scene::SceneClip;
use vdn_eval::scorecard::load_scene_clip;
use vdn_eval::suite::{LEVELS, parse_levels, parse_suite};
use vdn_eval::synth::{NoiseKind, SceneSpec, TalkerSpec, render};
use vdn_eval::system::external::{External, Kind, trunc_picks};
use vdn_eval::system::{SystemKind, SystemSpec, run};
use vdn_eval::wav;

const SR: u32 = vdn_types::MODEL_RATE;
const SCENES: [&str; 2] = ["cafe", "open-office"];

fn tracks(v: impl Fn(&str) -> String) -> Value {
    json!({"mix": v("mix"), "owner": v("owner"), "owner_dry": v("owner"), "floor": v("noise"),
           "noise": v("noise"), "interf": v("interf"), "stems": {"bed0": v("noise")}})
}

/// 一段合成片段（4 秒）：返回 clips.jsonl 的一行。
fn write_clip(root: &Path, scene: &str, i: usize) -> Value {
    let c = render(&SceneSpec {
        dur_s: 4.0,
        seed: 300 + i as u64,
        owner: Some(TalkerSpec {
            level_dbov: -26.0,
            f0_lo: 100.0,
            f0_hi: 140.0,
            tonal: false,
        }),
        owner_start_s: 0.5,
        pause_s: (0.4, 1.2),
        interferers: Vec::new(),
        noise: NoiseKind::Cafe,
        snr_db: 5.0,
        tail_silence_s: 0.0,
    });
    let id = format!("{scene}-c{i:02}");
    let rel = |k: &str| format!("clips/{id}-{k}.wav");
    for (k, x) in [
        ("mix", &c.mix),
        ("owner", &c.clean),
        ("noise", &c.noise),
        ("interf", &c.interf),
    ] {
        wav::write(&root.join(rel(k)), x, SR).unwrap();
    }
    let sha = |k: &str| wav::sha256(&root.join(rel(k))).unwrap();
    let n = c.mix.len() as u64;
    let iv: Vec<[u64; 2]> = c
        .truth
        .owner
        .iter()
        .map(|&(a, b)| [a as u64, b as u64])
        .collect();
    json!({
        "id": id, "scene": scene, "tier": "common", "device": "mba-builtin", "mic_mode": "standard",
        "pair": format!("{id}-x"), "lang": "en", "polarity": "suppress", "dur_samples": n, "sr": SR,
        "audio": tracks(rel), "has_clean_ref": true, "additive": true,
        "targets": [{"spk": "s1", "dist_m": 0.5, "azimuth_deg": 0.0, "level_dbfs": -26.0, "iv": iv, "src": {"kind": "synth", "spk": "s1"}}],
        "sounds": [{"inst": "bed0", "cls": "bed-cafe", "group": "bed", "is_speech": false, "iv": [[0, n]],
                    "level_dbfs": -40.0, "rel_db": -10.0, "dist_m": null, "azimuth_deg": null,
                    "src": {"kind": "synth", "fn": "bed-cafe"}, "license": "own"}],
        "nominal": {"snr_db": 5.0, "sir_db": null, "t60_s": 0.3, "owner_dist_m": 0.5, "owner_level_dbfs": -26.0},
        "measured": {"snr_db": 5.0, "snr_all_db": 5.0, "owner_level_dbfs": -26.0, "sir_db": null, "t60_s": 0.3, "drr_db": 5.0, "overlap_ratio": 0.0},
        "owner_iv": iv, "interf_iv": [], "chain": {"flat": true}, "seed": 300 + i,
        "hash": {"samples": tracks(|_| "0".repeat(64)), "wav_sha256": tracks(sha)},
    })
}

struct Fx {
    root: PathBuf,
    clips: Vec<SceneClip>,
    suite: String,
}

/// 两个场景 × 2 段。
fn setup(tag: &str) -> Fx {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("external-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("clips")).unwrap();
    let mut lines = Vec::new();
    for (si, s) in SCENES.iter().enumerate() {
        for i in 0..2 {
            lines.push(write_clip(&root, s, si * 10 + i).to_string());
        }
    }
    let jl = root.join("clips.jsonl");
    std::fs::write(&jl, lines.join("\n") + "\n").unwrap();
    let clips = SceneClip::parse_jsonl(&lines.join("\n")).unwrap();
    let tasks: Vec<Value> = SCENES
        .iter()
        .map(|s| json!({"id": format!("S-{s}"), "set": "capability", "polarity": "suppress",
            "origin": "外部入口测试", "desc": s, "graders": [],
            "real": {"kind": "scene", "clips": jl.to_string_lossy(), "scene": s, "tier": "common", "device": "mba-builtin"}}))
        .collect();
    let suite = json!({"name": "external-e2e", "tasks": tasks}).to_string();
    Fx { root, clips, suite }
}

/// 写外部输出目录：每种输入过一遍 `f`。`inputs` 决定写哪几种子目录。
fn write_ext(
    fx: &Fx,
    tag: &str,
    name: &str,
    lat: usize,
    inputs: &[Kind],
    f: &dyn Fn(&[f32]) -> Vec<f32>,
) -> PathBuf {
    let dir = fx.root.join("ext").join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    let picks = trunc_picks(&fx.clips);
    for c in &fx.clips {
        let clip = load_scene_clip(c, &fx.root).unwrap();
        for &k in inputs {
            if k == Kind::Trunc && !picks.contains(&c.id) {
                continue;
            }
            let p = dir.join(k.as_str()).join(format!("{}.wav", c.id));
            wav::write(&p, &f(&k.input(&clip)), SR).unwrap();
        }
    }
    let rj = json!({"schema": "vdn-external-output/1", "name": name,
        "clips": fx.root.join("clips.jsonl").to_string_lossy(), "sr": SR, "latency_samples": lat,
        "inputs": inputs.iter().map(|k| k.as_str()).collect::<Vec<_>>(), "model": {"kind": "test"}});
    std::fs::write(dir.join("run.json"), rj.to_string()).unwrap();
    dir
}

fn load(fx: &Fx, dir: &Path) -> Result<External, String> {
    External::load(
        dir,
        &fx.root,
        |s| Some(PathBuf::from(s)),
        |n| SystemKind::parse(n).is_some(),
    )
}

const ALL: [Kind; 4] = Kind::ALL;

fn ident(x: &[f32]) -> Vec<f32> {
    x.to_vec()
}

/// 输出 = 输入提前 `k` 个样本（看了 k 样本的未来），尾部补零。
fn ahead(k: usize) -> impl Fn(&[f32]) -> Vec<f32> {
    move |x| {
        let mut y = x[k.min(x.len())..].to_vec();
        y.resize(x.len(), 0.0);
        y
    }
}

fn opts() -> Opts {
    Opts {
        ci: true,
        audio_dir: None,
        baseline: Baseline::new(),
        levels: parse_levels(LEVELS).unwrap(),
    }
}

/// 主测试：外部入口 = 内置 passthrough，逐段逐项逐位一致；外部系统零「没跑成」（即索引用的就是 system::run 收到的 x）。
#[test]
fn external_passthrough_matches_builtin_bit_for_bit() {
    let fx = setup("bitwise");
    let dir = write_ext(&fx, "pt", "pt-ext", 0, &ALL, &ident);
    let ext = SystemKind::external(load(&fx, &dir).unwrap()).unwrap();
    let suite = parse_suite(&fx.suite).unwrap();
    let (rows, missing) = vdn_eval::scorecard::run(
        &suite,
        &[SystemKind::Passthrough, ext],
        &opts(),
        None,
        Some(&fx.root),
    );
    assert!(missing.is_empty(), "{missing:?}");
    assert_eq!(rows.len(), 2 * fx.clips.len());
    for pair in rows.chunks(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert_eq!(
            (a.system.as_str(), b.system.as_str()),
            ("passthrough", "pt-ext")
        );
        assert_eq!(a.clip, b.clip);
        assert_eq!(a.scores.len(), b.scores.len());
        for ((va, _, sa), (vb, _, sb)) in a.scores.iter().zip(&b.scores) {
            assert_eq!(va, vb);
            assert_eq!(format!("{sa:?}"), format!("{sb:?}"), "{} {va}", a.clip);
        }
    }
    let p = ext.provenance().unwrap();
    assert_eq!(p["latency_declared_only"], json!(true));
    assert_eq!(p["dir"], json!("ext/pt"));
    assert_eq!(p["run_json_sha256"].as_str().unwrap().len(), 64);
}

/// 声明延迟 L：离线输出已对齐（≈ 输入），回放把延迟补回去——原始流晚 L 样本、Engine 记账 = L；对齐后的输出 = 文件；计时为空。
#[test]
fn declared_latency_is_reinserted_and_timing_is_empty() {
    let fx = setup("lat");
    let lat = 320;
    let dir = write_ext(&fx, "lat", "lat-ext", lat, &ALL, &ident);
    let ext = SystemKind::external(load(&fx, &dir).unwrap()).unwrap();
    let clip = load_scene_clip(&fx.clips[0], &fx.root).unwrap();
    let o = run(
        &SystemSpec {
            kind: ext,
            safety: false,
            faults: vec![],
        },
        &clip.mix,
    )
    .unwrap();
    assert_eq!(o.latency, lat);
    assert!(o.engine_times.is_empty());
    assert_eq!(o.out, clip.mix);
    assert!(o.raw[..lat].iter().all(|v| *v == 0.0));
    assert_eq!(&o.raw[lat..lat + clip.mix.len()], &clip.mix[..]);
}

/// 偷看未来：输出看了 L + 100 样本的未来，只声明 L → 整个系统不打分。
#[test]
fn peeking_past_declared_latency_is_rejected() {
    let fx = setup("peek");
    let lat = 256;
    let dir = write_ext(&fx, "peek", "peek-ext", lat, &ALL, &ahead(lat + 100));
    let e = load(&fx, &dir).unwrap_err();
    assert!(e.contains("输出依赖声明延迟之后的输入"), "{e}");
    // 同一个候选如实声明 L + 100：通过
    let dir = write_ext(
        &fx,
        "honest",
        "honest-ext",
        lat + 100,
        &ALL,
        &ahead(lat + 100),
    );
    load(&fx, &dir).unwrap();
}

/// 没提供的输入种类：对应的那一路报错，不回退。
#[test]
fn missing_input_kind_is_reported() {
    let fx = setup("nobg");
    let dir = write_ext(
        &fx,
        "nobg",
        "nobg-ext",
        0,
        &[Kind::Mix, Kind::Trunc],
        &ident,
    );
    let ext = SystemKind::external(load(&fx, &dir).unwrap()).unwrap();
    let clip = load_scene_clip(&fx.clips[0], &fx.root).unwrap();
    let spec = SystemSpec {
        kind: ext,
        safety: true,
        faults: vec![],
    };
    let e = run(&spec, &Kind::Bg.input(&clip)).err().unwrap();
    assert!(e.contains("外部输出没提供 bg"), "{e}");
    // clean 同理：没列就报错、不回退成带噪那路；runner 凭 provides 事先知道
    let e = run(&spec, &Kind::Clean.input(&clip)).err().unwrap();
    assert!(e.contains("外部输出没提供 clean"), "{e}");
    let SystemKind::External(x) = ext else { unreachable!() };
    assert!(x.provides(Kind::Mix) && x.provides(Kind::Trunc));
    assert!(!x.provides(Kind::Bg) && !x.provides(Kind::Clean));
    assert!(run(&spec, &clip.mix).is_ok());
    let mut other = clip.mix.clone();
    other[7] += 1e-3;
    assert!(run(&spec, &other).err().unwrap().contains("没有这段输入"));
}

/// 自检每条：对不上就报错不打分。
#[test]
fn self_checks_reject_bad_dirs() {
    let fx = setup("bad");
    let good = || write_ext(&fx, "bad", "bad-ext", 0, &ALL, &ident);
    let id = &fx.clips[1].id;
    let expect = |dir: &Path, want: &str| {
        let e = load(&fx, dir).unwrap_err();
        assert!(e.contains(want), "想要「{want}」，得到 {e}");
    };
    // 缺片段
    let d = good();
    std::fs::remove_file(d.join(format!("mix/{id}.wav"))).unwrap();
    expect(&d, "缺 1 段");
    // clean/ 列了就必须段段齐、时长对
    let d = good();
    std::fs::remove_file(d.join(format!("clean/{id}.wav"))).unwrap();
    expect(&d, "clean/ 缺 1 段");
    let d = good();
    let pc = d.join(format!("clean/{id}.wav"));
    let (xc, _) = wav::read(&pc).unwrap();
    wav::write(&pc, &xc[..xc.len() - 1], SR).unwrap();
    expect(&d, "dur_samples");
    // 多片段
    let d = good();
    wav::write(&d.join("bg/nope-c99.wav"), &[0.0; 4], SR).unwrap();
    expect(&d, "清单没有的片段");
    // 采样率
    let d = good();
    let p = d.join(format!("mix/{id}.wav"));
    let (x, _) = wav::read(&p).unwrap();
    wav::write(&p, &x, vdn_types::IO_RATE).unwrap();
    expect(&d, "采样率");
    // 时长
    let d = good();
    wav::write(&p, &x[..x.len() - 1], SR).unwrap();
    expect(&d, "dur_samples");
    // NaN
    let d = good();
    let mut y = x.clone();
    y[10] = f32::NAN;
    wav::write(&p, &y, SR).unwrap();
    expect(&d, "不是有限数");
    // 名字撞内置
    let d = write_ext(&fx, "bad", "gtcrn", 0, &ALL, &ident);
    expect(&d, "撞名");
    // 没列的子目录（旧名 out/）
    let d = good();
    std::fs::create_dir_all(d.join("out")).unwrap();
    expect(&d, "没列的子目录 out/");
    // inputs 缺 trunc
    let d = write_ext(&fx, "bad", "bad-ext", 0, &[Kind::Mix, Kind::Bg], &ident);
    expect(&d, "必须含 trunc");
}

/// 命令行写法：`external:<目录>` 走自检，不过就解析不出系统（原因在 stderr）。
#[test]
fn parse_rejects_missing_dir() {
    assert_eq!(SystemKind::parse("external:no/such/dir"), None);
    assert_eq!(
        SystemKind::parse("passthrough"),
        Some(SystemKind::Passthrough)
    );
}

/// 两个目录同名：第二个载入报错（成绩单行分不清来源，codex 016 入口第 1 轮 P1 #1）。
#[test]
fn duplicate_external_name_is_rejected() {
    let fx = setup("dup");
    let a = write_ext(&fx, "dup-a", "dup-ext", 0, &ALL, &ident);
    let b = write_ext(&fx, "dup-b", "dup-ext", 0, &ALL, &ident);
    SystemKind::external(load(&fx, &a).unwrap()).unwrap();
    let e = SystemKind::external(load(&fx, &b).unwrap()).unwrap_err();
    assert!(e.contains("已被另一个目录用过"), "{e}");
}

/// 声明延迟 ≥ 段长 1/4：截半比较区间太短，报「无法验证」而不是空比较通过（codex 016 入口第 1 轮 P2 #2）。
#[test]
fn huge_declared_latency_cannot_be_verified() {
    let fx = setup("hugelat");
    let n = fx.clips[0].dur_samples as usize;
    let dir = write_ext(&fx, "hugelat", "hugelat-ext", n / 2, &ALL, &ident);
    let e = load(&fx, &dir).unwrap_err();
    assert!(e.contains("截半自检无法验证"), "{e}");
}

/// 保命开、注入故障：Engine 复位模型后，回放位置不回卷——输出、送原声、门控与内置 passthrough 逐位一致
/// （内置 passthrough 延迟 0、复位不改输出；回放若回卷，故障后的输出会变成段首的样本）。codex 016 入口第 2 轮 P2 #1。
/// 成绩单标准（`scorecard::standard`）不含计时项，所以「计时给空」对成绩单无影响；这里只断言计时为空。
#[test]
fn faults_and_resets_keep_replay_on_the_same_time_axis() {
    use vdn_engine::FaultKind;
    let fx = setup("fault");
    let dir = write_ext(&fx, "fault", "fault-ext", 0, &ALL, &ident);
    let ext = SystemKind::external(load(&fx, &dir).unwrap()).unwrap();
    let clip = load_scene_clip(&fx.clips[0], &fx.root).unwrap();
    let faults = vec![
        (20, FaultKind::Nan),
        (60, FaultKind::Error),
        (100, FaultKind::Explode),
    ];
    let spec = |kind| SystemSpec {
        kind,
        safety: true,
        faults: faults.clone(),
    };
    let a = run(&spec(SystemKind::Passthrough), &clip.mix).unwrap();
    let b = run(&spec(ext), &clip.mix).unwrap();
    assert!(a.stats.model_resets >= 2, "故障没触发复位：{:?}", a.stats);
    assert_eq!(a.stats.model_resets, b.stats.model_resets);
    assert_eq!(a.out, b.out);
    assert_eq!(a.bypass, b.bypass);
    assert_eq!(a.voiced, b.voiced);
    assert!(b.engine_times.is_empty());
    assert!(
        !vdn_eval::scorecard::standard::items(fx.clips[0].scene)
            .iter()
            .any(|(i, _)| i.v == "V-FRAME" || i.v == "V-RTF")
    );
}

/// `native_rate`：不写 = MODEL_RATE；写 IO_RATE 可以；别的值自检报错（evalfix 定，#111）。
#[test]
fn native_rate_is_read_and_checked() {
    let fx = setup("native");
    let dir = write_ext(&fx, "native", "native-ext", 0, &ALL, &ident);
    assert_eq!(
        load(&fx, &dir).unwrap().native_rate(),
        vdn_types::MODEL_RATE
    );
    let set = |v: Value| {
        let p = dir.join("run.json");
        let mut rj: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
        rj["native_rate"] = v;
        std::fs::write(&p, rj.to_string()).unwrap();
    };
    set(json!(vdn_types::IO_RATE));
    let e = load(&fx, &dir).unwrap();
    assert_eq!(e.native_rate(), vdn_types::IO_RATE);
    assert_eq!(e.provenance()["native_rate"], json!(vdn_types::IO_RATE));
    set(json!(44_100)); // tidy: 非采样率：故意给一个不允许的值
    assert!(load(&fx, &dir).unwrap_err().contains("native_rate"));
}
