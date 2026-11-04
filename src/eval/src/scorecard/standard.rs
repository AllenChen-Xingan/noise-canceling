//! 成绩单标准的机读正本：每个场景必看哪几项、按什么顺序判。人读的说明在 `specs/rubric/scorecard.md`，
//! 那里的「场景 → 场景类」表与本文件 [`group_of`] 由 `tests/scorecard_standard.rs` 逐行比对。
//!
//! 过关线一律取 `thresholds` 表（正本）；这里只在表的默认值为空、必须由任务给的参数上写数，
//! 这些数必须是表里已登记的取值（同一个测试核对），不许在这里新造过关线。

use crate::scene::MeetingPlace;

/// 一项：verifier + 必须由任务给的参数。
#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub v: &'static str,
    pub params: &'static [(&'static str, f64)],
    /// 只报不判：照常打分、列在表里，但不进该场景的过不过（检测器自己还不可信时用）
    pub report_only: bool,
}

const fn it(v: &'static str) -> Item {
    Item {
        v,
        params: &[],
        report_only: false,
    }
}

/// 一票否决项（排在听感前）：吃字 · 人声被压 · 延迟 · 坏值。任一不过 → 该场景不过（018 结论 5）。
/// 静音全零（V-MUTE）是系统层的事实，与场景无关，取 CI 层结果，不在每个场景里重算。
pub const VETO: [Item; 4] = [
    it("V-NAN"),
    it("V-LAT"),
    it("V-SPLVL"),
    Item {
        v: "V-EAT-ASR",
        params: &[("max", 1.0)],
        report_only: false,
    },
];

/// 所有开会场景都看的项。
pub const COMMON: [Item; 9] = [
    it("V-ONSET"),
    Item {
        v: "V-SIM-GATE",
        params: &[("max", 0.05), ("onset_max", 0.2)],
        report_only: false,
    },
    it("V-WEAK"),
    it("V-PUMPV"),
    // 只报不判（大总管 2026-11-03）：真实降噪输出上分不清咔嗒和残留噪声的短促起伏（scorecard.md 第九节第 2 条）
    Item {
        v: "V-CLICK",
        params: &[],
        report_only: true,
    },
    it("V-ESTOI"),
    it("V-TIMBRE"),
    it("V-NRMOS-SCOREQ"),
    it("V-NRMOS-DISTILL"),
];

const NR6: Item = Item {
    v: "V-NR",
    params: &[("max", -6.0)],
    report_only: false,
};
const INTF10: Item = Item {
    v: "V-INTF",
    params: &[("max", -10.0)],
    report_only: false,
};

const QUIET: [Item; 2] = [it("V-MUSIC"), NR6];
const TALKERS: [Item; 4] = [
    NR6,
    INTF10,
    Item {
        v: "V-SIM-FA",
        params: &[("max", 0.1)],
        report_only: false,
    },
    it("V-BGSTAB"),
];
const TRANSIT: [Item; 4] = [NR6, INTF10, it("V-BGSTAB"), it("V-MUSIC")];
const MEETROOM: [Item; 2] = [NR6, it("V-BGSTAB")];

/// 场景类：决定在 [`COMMON`] 之外再看哪几项。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    /// 本来安静：难在不做坏事（021 主表「难点」列）
    Quiet,
    /// 有旁人说话：要压旁人
    Talkers,
    /// 交通：强稳态低频 + 广播人声
    Transit,
    /// 会议室：近处几人都要留（`preserve_all`，待人定），不查「压旁人」
    MeetingRoom,
}

impl Group {
    pub const ALL: [Group; 4] = [
        Group::Quiet,
        Group::Talkers,
        Group::Transit,
        Group::MeetingRoom,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Group::Quiet => "quiet",
            Group::Talkers => "talkers",
            Group::Transit => "transit",
            Group::MeetingRoom => "meeting-room",
        }
    }

    /// 本类在 [`COMMON`] 之外加看的项。
    pub fn extra(self) -> &'static [Item] {
        match self {
            Group::Quiet => &QUIET,
            Group::Talkers => &TALKERS,
            Group::Transit => &TRANSIT,
            Group::MeetingRoom => &MEETROOM,
        }
    }
}

/// 场景 → 场景类。`other` 不是开会场景，不进成绩单。
pub fn group_of(s: MeetingPlace) -> Option<Group> {
    use MeetingPlace::*;
    Some(match s {
        HomeQuiet | PhoneBooth | HotelRoom | LibraryClassroom => Group::Quiet,
        HomeFamily | OpenOffice | Cowork | Cafe | Canteen => Group::Talkers,
        TransitHub | CarPassenger | BusMetroCabin | RailAirCabin | StreetOutdoor => Group::Transit,
        MeetingRoom => Group::MeetingRoom,
        Other => return None,
    })
}

/// 一个场景要跑的全部项：否决项在前，再是共同项，再是本类加看的项。
pub fn items(s: MeetingPlace) -> Vec<(Item, bool)> {
    let Some(g) = group_of(s) else {
        return Vec::new();
    };
    VETO.iter()
        .map(|i| (*i, true))
        .chain(COMMON.iter().map(|i| (*i, false)))
        .chain(g.extra().iter().map(|i| (*i, false)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_meeting_scene_has_a_group_and_veto_first() {
        for s in MeetingPlace::ALL {
            let v = items(s);
            if s.is_meeting() {
                assert!(v.len() > VETO.len() + COMMON.len(), "{s:?}");
                assert!(v[..VETO.len()].iter().all(|(_, veto)| *veto));
                assert!(v[VETO.len()..].iter().all(|(_, veto)| !*veto));
            } else {
                assert!(v.is_empty());
            }
        }
    }

    #[test]
    fn items_are_registered_and_unique_per_scene() {
        for s in MeetingPlace::ALL {
            let mut ids: Vec<&str> = items(s).iter().map(|(i, _)| i.v).collect();
            for id in &ids {
                assert!(crate::verifiers::info(id).is_some(), "{id} 没登记");
            }
            ids.sort_unstable();
            let n = ids.len();
            ids.dedup();
            assert_eq!(ids.len(), n, "{s:?} 有重复项");
        }
    }

    /// 写在这里的参数必须是 thresholds 表里已登记的取值（不许在成绩单里新造过关线）。
    #[test]
    fn params_are_registered_threshold_values() {
        let all = VETO
            .iter()
            .chain(&COMMON)
            .chain(Group::ALL.iter().flat_map(|g| g.extra()));
        for i in all {
            for (k, v) in i.params {
                let l = crate::thresholds::ALL
                    .iter()
                    .find(|l| l.id == i.v)
                    .unwrap_or_else(|| panic!("{} 不在 thresholds 表里", i.v));
                let p = l
                    .params
                    .iter()
                    .find(|p| p.key == *k)
                    .unwrap_or_else(|| panic!("{} 没有参数 {k}", i.v));
                assert!(
                    p.values.iter().any(|(x, _)| (x - v).abs() < 1e-12),
                    "{} · {k} = {v} 没登记在 thresholds 表里",
                    i.v
                );
            }
        }
    }
}
