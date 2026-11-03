//! 场景分析行为测试：滞回按确认时长才切；静音不出 NaN；手动选用户层场景后只在其下属大类里判。
use vdn_scene::SceneAnalyzer;
use vdn_types::{
    Confidence, FineSceneId, Hysteresis, HysteresisGate, MODEL_RATE, SampleClock, SceneLabel,
    UserScene,
};

fn label(fine: u16, user: UserScene, conf: f32, t_s: f64) -> SceneLabel {
    SceneLabel {
        user,
        fine: FineSceneId(fine),
        confidence: Confidence::new(conf),
        at: SampleClock((t_s * f64::from(MODEL_RATE)) as u64),
    }
}

#[test]
fn gate_needs_confirm_and_low_conf_interrupts() {
    let mut g = HysteresisGate::new(Hysteresis {
        confirm_s: 1.0,
        min_confidence: 0.4,
        min_dwell_s: 3.0,
    });
    assert!(
        g.step(&label(4, UserScene::Cafe, 0.9, 0.0), MODEL_RATE)
            .is_none()
    );
    assert!(
        g.step(&label(4, UserScene::Cafe, 0.9, 0.5), MODEL_RATE)
            .is_none()
    );
    assert!(
        g.step(&label(4, UserScene::Cafe, 0.9, 1.0), MODEL_RATE)
            .is_some()
    );
    // 新候选：中间一次低置信就从头算
    g.step(&label(16, UserScene::Commute, 0.9, 4.0), MODEL_RATE);
    g.step(&label(16, UserScene::Commute, 0.1, 4.5), MODEL_RATE);
    assert!(
        g.step(&label(16, UserScene::Commute, 0.9, 5.0), MODEL_RATE)
            .is_none()
    );
    assert!(
        g.step(&label(16, UserScene::Commute, 0.9, 6.0), MODEL_RATE)
            .is_some()
    );
    assert_eq!(g.committed(), (UserScene::Commute, FineSceneId(16)));
}

#[test]
fn silence_is_finite_and_manual_restricts() {
    let mut sa = SceneAnalyzer::new(MODEL_RATE, 256);
    sa.set_manual(Some(UserScene::Music));
    let z = vec![0.0f32; 256];
    let mut last = None;
    for k in 0..31 * 10 {
        sa.push_hop(&z);
        if k % 31 == 30 {
            let j = sa.judge();
            assert!(j.raw.iter().chain(j.features.iter()).all(|v| v.is_finite()));
            last = Some(j);
        }
    }
    let j = last.unwrap();
    assert_eq!(j.label.user, UserScene::Music);
}

#[test]
fn manual_meeting_commits() {
    // 自审 P1：手动选「多人会议」第一版永远提交不了
    let mut sa = SceneAnalyzer::new(MODEL_RATE, 256);
    sa.set_manual(Some(UserScene::Meeting));
    let x: Vec<f32> = (0..256).map(|i| (i as f32 * 0.3).sin() * 0.05).collect();
    let mut last = None;
    for k in 0..31 * 10 {
        sa.push_hop(&x);
        if k % 31 == 30 {
            last = Some(sa.judge());
        }
    }
    let j = last.unwrap();
    assert_eq!(j.committed.0, UserScene::Meeting);
    assert_eq!(j.params.tone.0, 6);
}

/// 标定产物（`weights.rs`，train.py 生成）与正本（`vdn_types::Hysteresis::default()`）必须一致：
/// 重新标定后只改了一边就红（CLAUDE.md 第五节第 5 条：第二处表示要有机器比对）。
#[test]
fn calibrated_hysteresis_matches_vdn_types_default() {
    assert_eq!(vdn_scene::weights::HYSTERESIS, Hysteresis::default());
}

/// 后验平滑时间常数同理：训练脚本写出的值 = 描述表默认值。
#[test]
fn calibrated_ema_matches_desc_default() {
    assert_eq!(
        vdn_scene::weights::EMA_S,
        vdn_types::desc::SCENE_EMA_S.default
    );
}

/// 调参入口按描述表夹值：越界的滞回与平滑常数进不了状态。
#[test]
fn set_tuning_clamps_by_desc_table() {
    use vdn_types::desc::{HYST_MIN_CONFIDENCE, SCENE_EMA_S};
    let mut sa = SceneAnalyzer::new(MODEL_RATE, 256);
    let bad = Hysteresis {
        min_confidence: HYST_MIN_CONFIDENCE.max + 1.0,
        ..Hysteresis::default()
    };
    assert!(bad.validate().is_err());
    sa.set_tuning(bad, SCENE_EMA_S.min - 1.0);
    let (h, ema) = sa.tuning();
    assert_eq!(h.min_confidence, HYST_MIN_CONFIDENCE.max);
    assert_eq!(ema, SCENE_EMA_S.min);
}
