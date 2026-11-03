use super::*;
use crate::graph::{ASYM_GATE, FLOOR};
use crate::record::tests::every_kind;

const G: ProcessingGraph = ProcessingGraph {
    model: ModelId::FastEnhancerM,
    spectral: &[],
    stages: &[&FLOOR, &ASYM_GATE],
};

fn full() -> SessionRecord {
    let mut header = SessionHeader {
        rate: 48_000,
        table: (TableVersion(3), ContentId::of(b"t")),
        profile: (ProfileVersion(1), ContentId::of(b"p")),
        ..SessionHeader::default()
    };
    header.set_graph(&G);
    SessionRecord {
        header,
        events: every_kind(),
    }
}

#[test]
fn session_roundtrips_with_graph_and_model() {
    let r = full();
    assert_eq!(r.header.graph, Some(G.content_id()));
    assert_eq!(r.header.model, Some(ModelId::FastEnhancerM));
    let text = r.to_text();
    assert!(text.starts_with("vdn-session = 2\n"), "{text}");
    assert!(text.contains(&format!("graph = {}\n", G.content_id())));
    let fem = ModelId::FastEnhancerM.as_str();
    assert!(text.contains(&format!("model = {fem}\n")), "{text}");
    assert_eq!(SessionRecord::from_text(&text).unwrap(), r);
}

#[test]
fn session_roundtrips_without_graph() {
    let mut r = full();
    r.header.graph = None;
    r.header.model = None;
    let text = r.to_text();
    assert!(!text.contains("graph =") && !text.contains("model ="));
    assert_eq!(SessionRecord::from_text(&text).unwrap(), r);
}

/// 格式 1（加 graph / model 之前写的文件）照常能读，新字段为 None。
#[test]
fn format_1_still_reads() {
    let e = every_kind()[0];
    let v1 = format!(
        "vdn-session = 1\nrate = 48000\ntable = 3 0000000000000001\nprofile = 1 0000000000000002\n\
         [events]\ne = {}\n",
        e.to_line()
    );
    let r = SessionRecord::from_text(&v1).unwrap();
    assert_eq!(r.header.rate, 48_000);
    assert_eq!(r.header.graph, None);
    assert_eq!(r.header.model, None);
    assert_eq!(r.events, [e]);
    // 再写出就是格式 2
    assert!(r.to_text().starts_with("vdn-session = 2\n"));
}

#[test]
fn format_1_with_new_keys_is_rejected() {
    for extra in ["graph = 0000000000000001", "model = gtcrn"] {
        let v1 = format!("vdn-session = 1\nrate = 48000\n{extra}\n[events]\n");
        assert!(SessionRecord::from_text(&v1).is_err(), "{extra}");
    }
}

/// 旧写法的模型名（`fastenhancer_m`，2026-11-02 之前写出的会话记录）照常能读，再写出是正本名。
#[test]
fn legacy_model_name_still_reads() {
    let fem = ModelId::FastEnhancerM.as_str();
    let text = full().to_text().replace(&format!("model = {fem}"), "model = fastenhancer_m");
    let r = SessionRecord::from_text(&text).unwrap();
    assert_eq!(r.header.model, Some(ModelId::FastEnhancerM));
    assert!(r.to_text().contains(&format!("model = {fem}\n")));
}

#[test]
fn bad_headers_are_rejected() {
    let ok = full().to_text();
    let model_line = format!("model = {}", ModelId::FastEnhancerM.as_str());
    let bad = [
        ok.replace("vdn-session = 2", "vdn-session = 3"),
        ok.replace(&model_line, "model = nope"),
        ok.replace(&model_line, "model = gtcrn extra"),
        ok.replace(&format!("graph = {}", G.content_id()), "graph = xyz"),
        ok.replace("rate = 48000", "rate = 48000 1"),
        ok.replace("rate = 48000", "speed = 1"),
        String::new(),
    ];
    for b in bad {
        assert!(SessionRecord::from_text(&b).is_err(), "{b}");
    }
}

/// 头部键的正本 `HEADER_KEYS` 与 `to_text` 实际写出的键逐个一致（顺序也一致）。
#[test]
fn header_keys_match_text() {
    let text = full().to_text();
    let written: Vec<&str> = text
        .lines()
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter_map(|l| l.split(" = ").next())
        .collect();
    let keys: Vec<&str> = HEADER_KEYS.iter().map(|k| k.key).collect();
    assert_eq!(written, keys);
    for k in HEADER_KEYS {
        assert!(
            k.since >= 1 && k.since <= SessionRecord::FORMAT,
            "{}",
            k.key
        );
        // 后加的键必须可缺省，否则旧文件读不进来
        assert!(k.since == 1 || k.optional, "{}", k.key);
    }
}

#[test]
fn set_graph_tracks_graph_changes() {
    let mut h = SessionHeader::default();
    h.set_graph(&G);
    let a = h.graph;
    h.set_graph(&ProcessingGraph {
        stages: &[&FLOOR],
        ..G
    });
    assert_ne!(h.graph, a);
    h.set_graph(&ProcessingGraph::bare(ModelId::Gtcrn));
    assert_eq!(h.model, Some(ModelId::Gtcrn));
}

/// codex 第 1 轮 #2：逐键删除——删必填键必须拒读，删可缺省键照常能读；同一个键写两次拒读。
#[test]
fn required_keys_must_be_present_once() {
    let text = full().to_text();
    for k in HEADER_KEYS {
        let prefix = format!("{} = ", k.key);
        let without: String = text
            .lines()
            .filter(|l| !l.starts_with(&prefix))
            .map(|l| format!("{l}\n"))
            .collect();
        assert_ne!(without, text, "{} 应在全字段文本里", k.key);
        assert_eq!(
            SessionRecord::from_text(&without).is_ok(),
            k.optional,
            "删掉 {}",
            k.key
        );
        let line = text.lines().find(|l| l.starts_with(&prefix)).unwrap();
        let twice = text.replacen(line, &format!("{line}\n{line}"), 1);
        assert!(
            SessionRecord::from_text(&twice).is_err(),
            "{} 写两次",
            k.key
        );
    }
    // 只有格式行的文件不再「读成默认头」
    assert!(SessionRecord::from_text("vdn-session = 2\n").is_err());
}
