//! 判决与 JSON 互转（日志里的 `verdict_events`，重放时读回）。

use serde_json::json;
use vdn_types::{Confidence, SampleClock, SpeakerClass, SpeakerVerdict};

pub(crate) fn class_str(c: SpeakerClass) -> &'static str {
    match c {
        SpeakerClass::Unknown => "unknown",
        SpeakerClass::NoSpeech => "nospeech",
        SpeakerClass::Target => "owner",
        SpeakerClass::Other => "others",
    }
}

pub(crate) fn verdict_to_json(v: &SpeakerVerdict, at_hop: usize) -> serde_json::Value {
    json!({"at_hop": at_hop, "class": class_str(v.class), "conf": v.confidence.get(),
           "ws": v.window_start.0, "we": v.window_end.0, "owner_level_db": v.owner_level_db})
}

pub(crate) fn verdict_from_json(e: &serde_json::Value) -> SpeakerVerdict {
    let class = match e["class"].as_str().unwrap_or("") {
        "nospeech" => SpeakerClass::NoSpeech,
        "owner" => SpeakerClass::Target,
        "others" => SpeakerClass::Other,
        _ => SpeakerClass::Unknown,
    };
    SpeakerVerdict {
        class,
        confidence: Confidence::new(e["conf"].as_f64().unwrap_or(0.0) as f32),
        window_start: SampleClock(e["ws"].as_u64().unwrap_or(0)),
        window_end: SampleClock(e["we"].as_u64().unwrap_or(0)),
        owner_level_db: e["owner_level_db"].as_f64().map(|x| x as f32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn verdict_json_round_trips_every_class() {
        for class in [
            SpeakerClass::Unknown,
            SpeakerClass::NoSpeech,
            SpeakerClass::Target,
            SpeakerClass::Other,
        ] {
            let v = SpeakerVerdict {
                class,
                confidence: Confidence::new(0.75),
                window_start: SampleClock(1024),
                window_end: SampleClock(9216),
                owner_level_db: Some(-23.5),
            };
            let j = verdict_to_json(&v, 42);
            assert_eq!(j["at_hop"], 42);
            assert_eq!(verdict_from_json(&j), v, "{class:?} 往返后应不变");
        }
        // owner_level_db 为空时也能往返
        let v = SpeakerVerdict {
            owner_level_db: None,
            ..SpeakerVerdict::default()
        };
        assert_eq!(verdict_from_json(&verdict_to_json(&v, 0)), v);
    }
}
