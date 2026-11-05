use super::*;

/// 合成一段「说话 + 停顿」的原声和一份假的模型输出（原声减半 + 小抖动）。
fn fake(n: usize) -> (Vec<f32>, Vec<f32>) {
    let x: Vec<f32> = (0..n)
        .map(|i| {
            let t = i as f32 / SR as f32;
            let speech = if (t % 1.0) < 0.5 { 0.3 } else { 0.0 };
            speech * (2.0 * std::f32::consts::PI * 220.0 * t).sin()
                + 0.01 * (2.0 * std::f32::consts::PI * 3000.0 * t).sin()
        })
        .collect();
    let y: Vec<f32> = x.iter().map(|v| 0.5 * v).collect();
    (x, y)
}

/// 核心对拍：默认参数下「先重混、再关保命过一遍」= 「原始输出直接开保命过一遍」（成绩单原路径）。
#[test]
fn default_remix_then_safety_off_equals_engine_path() {
    let (x, y) = fake(SR as usize * 3 + 77);
    for lat in [0usize, 160, 284] {
        let direct = remix_one(&x, &y, lat, SafetyConfig::default());
        let pre = remix_one(&x, &y, lat, MixParams::default().config());
        let twice = remix_one(&x, &pre, lat, SafetyConfig::off());
        assert_eq!(direct.len(), x.len());
        assert_eq!(direct, twice, "延迟 {lat}");
    }
}

/// 关保命时引擎原样吐出模型输出（「不混回」那一档就是原始输出）。
#[test]
fn off_is_identity() {
    let (x, y) = fake(SR as usize + 5);
    let out = remix_one(&x, &y, 160, MixParams { off: true, ..Default::default() }.config());
    assert_eq!(out, y);
}

/// 混回越多，输出越靠近原声：停顿处（只有底噪）残留随 floor_db 单调变大。
#[test]
fn more_floor_keeps_more_noise() {
    let (x, y) = fake(SR as usize * 2);
    // 停顿段：0.6 s–0.95 s
    let a = (0.6 * SR as f32) as usize;
    let b = (0.95 * SR as f32) as usize;
    let err = |fl: f32| {
        let p = MixParams { floor_db: Some(fl), ..Default::default() };
        let o = remix_one(&x, &y, 0, p.config());
        (a..b).map(|i| (o[i] - x[i]).powi(2)).sum::<f32>()
    };
    let (e18, e12, e6) = (err(-18.0), err(-12.0), err(-6.0));
    assert!(e18 > e12 && e12 > e6, "{e18} {e12} {e6}");
}

#[test]
fn suite_off_sets_every_task_and_renames() {
    let s = serde_json::json!({"name": "exp013-scenes", "tasks": [{"id": "a"}, {"id": "b", "safety": true}]});
    let o = suite_safety_off(&s).unwrap();
    assert_eq!(o["name"], "exp013-scenes-safety-off");
    for t in o["tasks"].as_array().unwrap() {
        assert_eq!(t["safety"], false);
    }
    assert_eq!(o["tasks"][0]["id"], "a");
    assert!(suite_safety_off(&serde_json::json!({})).is_err());
}

#[test]
fn names_follow_rule() {
    let n = system_name("dfn3", &MixParams::default());
    assert_eq!(n, "dfn3-om6-fm12");
    let n = system_name("g", &MixParams { open_db: Some(-9.0), floor_db: Some(-18.0), off: false });
    assert_eq!(n, "g-om9-fm18");
    assert_eq!(system_name("g", &MixParams { off: true, ..Default::default() }), "g-nomix");
    assert!(n.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'));
}

/// 坏值与超幅保护（出事送原声）在预混那一遍按默认打开、作用于原始输出，和成绩单原路径同一处；
/// 第二遍关保命只是不再查已混好的结果。原始输出里塞 NaN 与超幅帧，两条路仍逐样本相同、输出全有限。
#[test]
fn faults_handled_in_premix_same_as_engine_path() {
    let (x, mut y) = fake(SR as usize * 3);
    for v in &mut y[4000..4100] {
        *v = f32::NAN;
    }
    for v in &mut y[20000..20400] {
        *v = 50.0;
    }
    let direct = remix_one(&x, &y, 160, SafetyConfig::default());
    let pre = remix_one(&x, &y, 160, MixParams::default().config());
    let twice = remix_one(&x, &pre, 160, SafetyConfig::off());
    assert!(direct.iter().all(|v| v.is_finite()));
    assert_eq!(direct, twice);
}
