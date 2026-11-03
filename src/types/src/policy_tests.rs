//! `policy.rs` 的测试（滞回、叠参数、用户优先、决策日志），分出来让每个文件 ≤ 600 行。

use super::*;

/// 滞回与平滑的默认值、范围取自描述表；取表的边界外一点造输入，validate 拒收、clamped 夹住。
#[test]
fn hysteresis_and_transition_follow_desc_table() {
    use crate::desc::*;
    let h = Hysteresis::default();
    assert_eq!(
        (h.confirm_s, h.min_confidence, h.min_dwell_s),
        (
            HYST_CONFIRM_S.default,
            HYST_MIN_CONFIDENCE.default,
            HYST_MIN_DWELL_S.default
        )
    );
    h.validate().unwrap();
    let bad = Hysteresis {
        min_confidence: HYST_MIN_CONFIDENCE.max + 0.1,
        ..h
    };
    assert_eq!(bad.validate().unwrap_err().desc.id, HYST_MIN_CONFIDENCE.id);
    assert_eq!(bad.clamped().min_confidence, HYST_MIN_CONFIDENCE.max);
    let bad = Hysteresis {
        confirm_s: HYST_CONFIRM_S.min - 1.0,
        ..h
    };
    assert_eq!(bad.validate().unwrap_err().desc.id, HYST_CONFIRM_S.id);
    let bad = Hysteresis {
        min_dwell_s: HYST_MIN_DWELL_S.max + 1.0,
        ..h
    };
    assert_eq!(bad.validate().unwrap_err().desc.id, HYST_MIN_DWELL_S.id);

    let t = Transition::default();
    assert_eq!(
        (
            t.glide_s,
            t.model_crossfade_s,
            t.max_db_per_s,
            t.manual_ramp_s
        ),
        (
            TRANS_GLIDE_S.default,
            TRANS_MODEL_CROSSFADE_S.default,
            TRANS_MAX_DB_PER_S.default,
            TRANS_MANUAL_RAMP_S.default
        )
    );
    t.validate().unwrap();
    for (d, bad) in [
        (
            &TRANS_GLIDE_S,
            Transition {
                glide_s: TRANS_GLIDE_S.max + 1.0,
                ..t
            },
        ),
        (
            &TRANS_MODEL_CROSSFADE_S,
            Transition {
                model_crossfade_s: -0.1,
                ..t
            },
        ),
        (
            &TRANS_MAX_DB_PER_S,
            Transition {
                max_db_per_s: TRANS_MAX_DB_PER_S.max + 1.0,
                ..t
            },
        ),
        (
            &TRANS_MANUAL_RAMP_S,
            Transition {
                manual_ramp_s: f32::NAN,
                ..t
            },
        ),
    ] {
        assert_eq!(bad.validate().unwrap_err().desc.id, d.id);
    }
}
use crate::frame::Confidence;
use crate::params::{ModelId, ParamField};
use crate::scene::SceneName;
use crate::table::SceneEntry;

const SR: u32 = crate::IO_RATE;
// 测试用固定参数（不跟默认值走，默认值会随标定变）
const H: Hysteresis = Hysteresis {
    confirm_s: 2.0,
    min_confidence: 0.6,
    min_dwell_s: 5.0,
};

fn label(fine: u16, user: UserScene, sec: f64, conf: f32) -> SceneLabel {
    SceneLabel {
        user,
        fine: FineSceneId(fine),
        confidence: Confidence::new(conf),
        at: SampleClock((sec * f64::from(SR)) as u64),
    }
}

fn table() -> SceneTable {
    let mut t = SceneTable::default();
    let p = ParamSet {
        floor_db: -20.0,
        model: ModelId::FastEnhancerM,
        ..ParamSet::default()
    };
    t.insert(SceneEntry {
        id: FineSceneId(7),
        name: SceneName::new("cafe+bt").unwrap(),
        user: UserScene::Cafe,
        params: p,
    })
    .unwrap();
    t
}

#[test]
fn needs_continuous_confirmation() {
    let mut g = HysteresisGate::new(H);
    assert_eq!(g.step(&label(7, UserScene::Cafe, 0.0, 0.9), SR), None);
    assert_eq!(g.step(&label(7, UserScene::Cafe, 1.0, 0.9), SR), None);
    assert_eq!(
        g.step(&label(7, UserScene::Cafe, 2.0, 0.9), SR),
        Some((FineSceneId(0), FineSceneId(7)))
    );
    assert_eq!(g.committed(), (UserScene::Cafe, FineSceneId(7)));
}

#[test]
fn flapping_never_switches() {
    let mut g = HysteresisGate::new(H);
    for k in 0..40 {
        let fine = if k % 2 == 0 { 7 } else { 8 };
        assert_eq!(
            g.step(&label(fine, UserScene::Cafe, k as f64 * 0.5, 0.9), SR),
            None
        );
    }
}

#[test]
fn low_confidence_does_not_count_and_breaks_the_run() {
    let mut g = HysteresisGate::new(H);
    for k in 0..20 {
        assert_eq!(
            g.step(&label(7, UserScene::Cafe, k as f64 * 0.5, 0.3), SR),
            None
        );
    }
    // 0 s 高置信、中间全是低置信、10 s 再来一次：不算连续，不切
    let mut g = HysteresisGate::new(H);
    g.step(&label(7, UserScene::Cafe, 0.0, 0.9), SR);
    for k in 1..20 {
        g.step(&label(7, UserScene::Cafe, k as f64 * 0.5, 0.3), SR);
    }
    assert_eq!(g.step(&label(7, UserScene::Cafe, 10.0, 0.9), SR), None);
}

#[test]
fn user_layer_only_labels_can_switch() {
    // 分类器只出用户层（fine = 0）
    let mut g = HysteresisGate::new(H);
    g.step(&label(0, UserScene::Cafe, 0.0, 0.9), SR);
    assert!(g.step(&label(0, UserScene::Cafe, 2.0, 0.9), SR).is_some());
    assert_eq!(g.committed(), (UserScene::Cafe, FineSceneId::UNKNOWN));
}

#[test]
fn table_decides_user_layer_of_known_fine_scene() {
    let t = table();
    let prof = UserProfile::default();
    let ctl = UserControl::default();
    // 标签说 Music，表里 7 属于 Cafe：以表为准
    let (u, _, _, _) = resolve_params(&t, &prof, &ctl, UserScene::Music, FineSceneId(7));
    assert_eq!(u, UserScene::Cafe);
}

#[test]
fn dwell_blocks_quick_second_switch() {
    let mut g = HysteresisGate::new(H);
    g.step(&label(7, UserScene::Cafe, 0.0, 0.9), SR);
    assert!(g.step(&label(7, UserScene::Cafe, 2.0, 0.9), SR).is_some());
    // 2 s 后又稳定判成 8，确认够了但停留不够 5 s
    assert_eq!(g.step(&label(8, UserScene::Music, 2.5, 0.9), SR), None);
    assert_eq!(g.step(&label(8, UserScene::Music, 4.5, 0.9), SR), None);
    assert!(g.step(&label(8, UserScene::Music, 7.0, 0.9), SR).is_some());
}

#[test]
fn layering_and_user_priority() {
    let t = table();
    let mut prof = UserProfile::default();
    prof.remember_knob(
        crate::memory::SceneKey::Fine(FineSceneId(7)),
        ParamField::Strength,
        0.8,
    );
    let ctl = UserControl::default();
    let (_, _, p, src) = resolve_params(&t, &prof, &ctl, UserScene::Cafe, FineSceneId(7));
    assert_eq!(
        (p.floor_db, p.strength, src),
        (-20.0, 0.8, ParamSource::Profile)
    );

    let mut ctl = UserControl::default();
    ctl.knobs.set(ParamField::FloorDb, Some(-6.0));
    let (_, _, p, src) = resolve_params(&t, &prof, &ctl, UserScene::Cafe, FineSceneId(7));
    assert_eq!((p.floor_db, src), (-6.0, ParamSource::UserKnob));

    // 手动选了「安静」：咖啡馆下的细场景不用，退回安静的基础参数
    let ctl = UserControl {
        scene: UserSceneChoice::Manual(UserScene::Quiet),
        ..Default::default()
    };
    let (u, f, p, _) = resolve_params(&t, &prof, &ctl, UserScene::Cafe, FineSceneId(7));
    assert_eq!(
        (u, f, p.model),
        (UserScene::Quiet, FineSceneId::UNKNOWN, ModelId::Gtcrn)
    );
}

#[test]
fn table_policy_logs_switch_and_respects_lock() {
    let t = table();
    let prof = UserProfile::default();
    let mut pol = TablePolicy::new(H, Transition::default());
    let mut log = Vec::new();
    let mut ctl = UserControl::default();
    let step = |pol: &mut TablePolicy, ctl: &UserControl, l: SceneLabel, log: &mut Vec<_>| {
        let d = pol.decide(&PolicyInput {
            label: l,
            rate: SR,
            table: &t,
            profile: &prof,
            control: ctl,
        });
        d.log_into(&l, log);
        d
    };
    for k in 0..=4 {
        step(
            &mut pol,
            &ctl,
            label(7, UserScene::Cafe, k as f64 * 0.5, 0.9),
            &mut log,
        );
    }
    // 第一次：默认参数（ParamsApplied）；2 s 时换到 7（SceneCommitted + ParamsApplied）
    assert_eq!(log.len(), 3);
    assert!(matches!(
        log[1],
        DecisionEvent::SceneCommitted {
            to: FineSceneId(7),
            ..
        }
    ));

    ctl.lock = true;
    for k in 5..30 {
        let d = step(
            &mut pol,
            &ctl,
            label(1, UserScene::Quiet, k as f64 * 0.5, 0.9),
            &mut log,
        );
        assert_eq!(d.target.model, ModelId::FastEnhancerM);
        assert_eq!(d.source, ParamSource::Locked);
    }
    // 锁定中旋钮仍然生效
    ctl.knobs.set(ParamField::FloorDb, Some(-3.0));
    let d = step(
        &mut pol,
        &ctl,
        label(1, UserScene::Quiet, 15.0, 0.9),
        &mut log,
    );
    assert_eq!(
        (d.target.floor_db, d.target.model),
        (-3.0, ModelId::FastEnhancerM)
    );
    assert_eq!(d.source, ParamSource::UserKnob);
}
