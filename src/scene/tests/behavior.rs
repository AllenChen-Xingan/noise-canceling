//! 场景分析行为测试：滞回按确认时长才切；静音不出 NaN；手动选用户层场景后只在其下属大类里判。
use vdn_scene::SceneAnalyzer;
use vdn_scene::mirror::{
    Confidence, FineSceneId, Hysteresis, HysteresisGate, SampleClock, SceneLabel, UserScene,
};

fn label(fine: u16, user: UserScene, conf: f32, t_s: f64) -> SceneLabel {
    SceneLabel {
        user,
        fine: FineSceneId(fine),
        confidence: Confidence::new(conf),
        at: SampleClock((t_s * 16000.0) as u64),
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
        g.step(&label(4, UserScene::Cafe, 0.9, 0.0), 16000)
            .is_none()
    );
    assert!(
        g.step(&label(4, UserScene::Cafe, 0.9, 0.5), 16000)
            .is_none()
    );
    assert!(
        g.step(&label(4, UserScene::Cafe, 0.9, 1.0), 16000)
            .is_some()
    );
    // 新候选：中间一次低置信就从头算
    g.step(&label(16, UserScene::Commute, 0.9, 4.0), 16000);
    g.step(&label(16, UserScene::Commute, 0.1, 4.5), 16000);
    assert!(
        g.step(&label(16, UserScene::Commute, 0.9, 5.0), 16000)
            .is_none()
    );
    assert!(
        g.step(&label(16, UserScene::Commute, 0.9, 6.0), 16000)
            .is_some()
    );
    assert_eq!(g.committed(), (UserScene::Commute, FineSceneId(16)));
}

#[test]
fn silence_is_finite_and_manual_restricts() {
    let mut sa = SceneAnalyzer::new(16000, 256);
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
    let mut sa = SceneAnalyzer::new(16000, 256);
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
