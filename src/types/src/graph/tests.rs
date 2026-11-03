use super::*;
use crate::frame::FrameCtx;
use crate::stage::Stage;

/// 只报种类与延迟的假级。
struct Fake(&'static NodeKind, usize);
impl Stage for Fake {
    fn kind(&self) -> &'static NodeKind {
        self.0
    }
    fn reset(&mut self) {}
    fn process(&mut self, _wet: &mut [f32], _dry: &[f32], _ctx: FrameCtx) {}
    fn latency_samples(&self) -> usize {
        self.1
    }
}

fn fake(k: &'static NodeKind) -> Fake {
    Fake(k, k.latency_samples)
}

const TWO: [&NodeKind; 2] = [&FLOOR, &ASYM_GATE];

#[test]
fn registry_is_well_formed() {
    for (i, a) in KINDS.iter().enumerate() {
        assert!(a.id > 0 && a.id < TEST_ID_MIN, "{} 编号越界", a.name);
        assert!(a.version > 0, "{} 版本从 1 起", a.name);
        // 规约 §3.3：「每级 dry 对齐」做出来之前，级延迟必须为 0
        assert_eq!(a.latency_samples, 0, "{}", a.name);
        assert!(!a.name.is_empty() && !a.name.contains(char::is_whitespace));
        for b in KINDS.iter().skip(i + 1) {
            assert_ne!(a.id, b.id, "编号重复：{} / {}", a.name, b.name);
            assert_ne!(a.name, b.name, "名字重复");
        }
        assert_eq!(kind_by_id(a.id), Some(*a));
        // 参数只引用 desc 的正本：同编号的条目必须就是 desc 表里那一条（不许另抄一份改了范围）
        for p in a.params {
            assert_eq!(desc::by_id(p.id), Some(*p), "{} 引用了不在 desc::ALL 的参数", a.name);
        }
        let mut ids: Vec<u16> = a.params.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), a.params.len(), "{} 参数重复", a.name);
    }
    assert_eq!(kind_by_id(TEST_ID_MIN), None);
}

#[test]
fn matching_chain_passes() {
    let c = crate::stages![fake(&FLOOR), fake(&ASYM_GATE)];
    assert_eq!(match_chain(&TWO, &c), Ok(()));
    assert_eq!(match_chain(&[], &()), Ok(()));
}

#[test]
fn missing_stage_is_rejected() {
    let c = crate::stages![fake(&FLOOR)];
    assert_eq!(
        match_chain(&TWO, &c),
        Err(ChainMismatch::Missing {
            index: 1,
            expected: "asym-gate"
        })
    );
}

#[test]
fn extra_stage_is_rejected() {
    let c = crate::stages![fake(&FLOOR), fake(&ASYM_GATE), fake(&FLOOR)];
    assert_eq!(
        match_chain(&TWO, &c),
        Err(ChainMismatch::Extra {
            index: 2,
            actual: "floor"
        })
    );
}

#[test]
fn reversed_order_is_rejected() {
    let c = crate::stages![fake(&ASYM_GATE), fake(&FLOOR)];
    assert_eq!(
        match_chain(&TWO, &c),
        Err(ChainMismatch::Kind {
            index: 0,
            expected: "floor",
            actual: "asym-gate"
        })
    );
}

/// 编号相同、名字不同（有人在实现里手写了一份种类）也算不一致。
#[test]
fn wrong_name_is_rejected() {
    static RENAMED: NodeKind = NodeKind {
        name: "floor2",
        ..FLOOR
    };
    let c = crate::stages![Fake(&RENAMED, 0), fake(&ASYM_GATE)];
    assert_eq!(
        match_chain(&TWO, &c),
        Err(ChainMismatch::Kind {
            index: 0,
            expected: "floor",
            actual: "floor2"
        })
    );
    // 版本不同同理
    static BUMPED: NodeKind = NodeKind {
        version: 2,
        ..FLOOR
    };
    let c = crate::stages![Fake(&BUMPED, 0), fake(&ASYM_GATE)];
    assert!(matches!(
        match_chain(&TWO, &c),
        Err(ChainMismatch::Kind { index: 0, .. })
    ));
}

#[test]
fn different_latency_is_rejected() {
    let c = crate::stages![fake(&FLOOR), Fake(&ASYM_GATE, 3)];
    assert_eq!(
        match_chain(&TWO, &c),
        Err(ChainMismatch::Latency {
            index: 1,
            stage: "asym-gate",
            declared: 0,
            actual: 3
        })
    );
}

#[test]
fn spectral_kind_in_time_chain_is_rejected() {
    let c = crate::stages![fake(&POLISH_SPECTRAL)];
    assert!(matches!(
        match_chain(&[&POLISH_SPECTRAL], &c),
        Err(ChainMismatch::WrongHook { index: 0, .. })
    ));
    let g = ProcessingGraph {
        stages: &[&POLISH_SPECTRAL],
        ..ProcessingGraph::bare(ModelId::Gtcrn)
    };
    assert!(g.check().is_err());
    let g = ProcessingGraph {
        spectral: &[&POLISH_AGC],
        ..ProcessingGraph::bare(ModelId::Gtcrn)
    };
    assert!(g.check().is_err());
    let g = ProcessingGraph {
        spectral: &[&POLISH_SPECTRAL],
        stages: &[&POLISH_AGC],
        model: ModelId::Gtcrn,
    };
    assert_eq!(g.check(), Ok(()));
}

#[test]
fn mismatch_messages_name_the_stage() {
    let e = ChainMismatch::Kind {
        index: 0,
        expected: "floor",
        actual: "asym-gate",
    };
    let s = e.to_string();
    assert!(s.contains("floor") && s.contains("asym-gate"), "{s}");
}

const G: ProcessingGraph = ProcessingGraph {
    model: ModelId::Gtcrn,
    spectral: &[&POLISH_SPECTRAL],
    stages: &[&ASYM_GATE, &POLISH_AGC],
};

#[test]
fn content_id_is_stable() {
    assert_eq!(G.content_id(), G.content_id());
    let copy = ProcessingGraph { ..G };
    assert_eq!(copy.content_id(), G.content_id());
    // 规范文本的写法是格式的一部分：改了它，所有旧 ID 都会变，要升 FORMAT
    assert_eq!(
        G.canonical_text(),
        "vdn-graph = 1\nmodel = gtcrn\n[spectral]\nnode = 6 polish-spectral 1 spectral 0 20,21,22,23,27\n\
         [stages]\nnode = 4 asym-gate 1 time 0 2,10,11,12,13,14,15,16,17,18,19,20,8,9\n\
         node = 5 polish-agc 1 time 0 24,25,26,27\n"
    );
}

#[test]
fn content_id_changes_with_any_part() {
    static V2: NodeKind = NodeKind {
        version: 2,
        ..ASYM_GATE
    };
    static FEWER: NodeKind = NodeKind {
        params: &[&desc::FLOOR_DB],
        ..ASYM_GATE
    };
    static LAT: NodeKind = NodeKind {
        latency_samples: 1,
        ..ASYM_GATE
    };
    static DOC: NodeKind = NodeKind {
        doc: "改了说明",
        ..ASYM_GATE
    };
    let base = G.content_id();
    let variants = [
        ProcessingGraph {
            model: ModelId::FastEnhancerM,
            ..G
        },
        ProcessingGraph {
            stages: &[&V2, &POLISH_AGC],
            ..G
        },
        ProcessingGraph {
            stages: &[&FEWER, &POLISH_AGC],
            ..G
        },
        ProcessingGraph {
            stages: &[&LAT, &POLISH_AGC],
            ..G
        },
        ProcessingGraph {
            stages: &[&POLISH_AGC, &ASYM_GATE],
            ..G
        },
        ProcessingGraph {
            stages: &[&ASYM_GATE],
            ..G
        },
        ProcessingGraph { spectral: &[], ..G },
        // 同一个节点从频域挪到时域段也要变（分段写进了规范文本）
        ProcessingGraph {
            spectral: &[],
            stages: &[&POLISH_SPECTRAL, &ASYM_GATE, &POLISH_AGC],
            ..G
        },
    ];
    for (i, v) in variants.iter().enumerate() {
        assert_ne!(v.content_id(), base, "第 {i} 个变体");
    }
    // 只改说明不算换图
    let doc_only = ProcessingGraph {
        stages: &[&DOC, &POLISH_AGC],
        ..G
    };
    assert_eq!(doc_only.content_id(), base);
}
