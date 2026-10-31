//! 路由表：细场景 → 参数组（D40）。
//!
//! 细场景编号 = 噪音大类 × 设备：`id = 1 + 3·class + device`（device：0 普通 · 1 窄带蓝牙 · 2 削波），共 21 个；
//! 22 = 多人会议（占位，只能手动选）。0 = 未知。一个细场景属于哪个用户层场景只由噪音大类决定。
//!
//! 模型选择的证据（exp003 人耳盲听，`docs/experiments/exp003-offline-ab/results/README.md`；D33）：
//! - 赌场类多人 babble → FastEnhancer-M（人耳赌场段 FE-M 好）
//! - 邻座说话 → GTCRN（人耳邻桌说话段 GTCRN 略好；ASR 吃字最少）
//! - 音乐 → FastEnhancer-M（**只有客观分**：c10 钢琴非语音段压低 FE-M −63 dB、GTCRN −29 dB）
//! - 其余（安静、空调、杯碟键盘、交通）→ GTCRN（D33 主模型；c09 杯碟两者客观分打平；交通没有证据）
//!
//! 延迟账：池里只有 GTCRN（模型延迟 256 样本 @16 kHz = 16 ms）与 FastEnhancer-M（352 样本 = 22 ms），都在 ≤ 45 ms 档；
//! 引擎按池内最大补齐（规约 §3.5），所以开了路由，GTCRN 场景也要多付 6 ms。路由本身在判断回路，不加延迟。

use crate::classify::NoiseClass;
use crate::mirror::{FineSceneId, ModelId, ParamSet, ToneId, UserScene};

pub const N_DEVICE: u16 = 3;
pub const MEETING: FineSceneId = FineSceneId(22);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Normal,
    BtNarrow,
    Clipping,
}

impl Device {
    pub fn index(self) -> u16 {
        self as u16
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Device::Normal => "normal",
            Device::BtNarrow => "bt_narrow",
            Device::Clipping => "clip",
        }
    }
}

pub fn fine_id(c: NoiseClass, d: Device) -> FineSceneId {
    FineSceneId(1 + 3 * c.index() as u16 + d.index())
}

/// 反查：细场景 → (大类, 设备)。会议与未知返回 `None`。
pub fn decode(id: FineSceneId) -> Option<(NoiseClass, Device)> {
    if id.0 == 0 || id.0 > 21 {
        return None;
    }
    let k = id.0 - 1;
    let d = match k % N_DEVICE {
        0 => Device::Normal,
        1 => Device::BtNarrow,
        _ => Device::Clipping,
    };
    Some((NoiseClass::ALL[(k / N_DEVICE) as usize], d))
}

pub fn user_of(c: NoiseClass) -> UserScene {
    match c {
        NoiseClass::Quiet | NoiseClass::Hvac => UserScene::Quiet,
        NoiseClass::Babble | NoiseClass::Neighbor | NoiseClass::Clatter => UserScene::Cafe,
        NoiseClass::Traffic => UserScene::Commute,
        NoiseClass::Music => UserScene::Music,
    }
}

pub fn user_of_id(id: FineSceneId) -> UserScene {
    if id == MEETING {
        return UserScene::Meeting;
    }
    decode(id).map(|(c, _)| user_of(c)).unwrap_or_default()
}

/// 细场景名（日志、文本表用）。
pub fn name(id: FineSceneId, buf: &mut String) {
    buf.clear();
    match decode(id) {
        Some((c, d)) => {
            buf.push_str(c.as_str());
            if d != Device::Normal {
                buf.push('+');
                buf.push_str(d.as_str());
            }
        }
        None if id == MEETING => buf.push_str("meeting"),
        None => buf.push_str("unknown"),
    }
}

/// 这一组参数是路由表的默认行。
pub fn params_for(id: FineSceneId) -> ParamSet {
    let mut p = ParamSet::DEFAULT;
    if id == MEETING {
        p.tone = ToneId(6);
        return p;
    }
    let Some((c, d)) = decode(id) else {
        p.tone = ToneId(1);
        return p;
    };
    let (model, tone) = match c {
        NoiseClass::Quiet | NoiseClass::Hvac => (ModelId::Gtcrn, 1),
        NoiseClass::Babble => (ModelId::FastEnhancerM, 2),
        NoiseClass::Clatter => (ModelId::Gtcrn, 2),
        NoiseClass::Neighbor => (ModelId::Gtcrn, 3),
        NoiseClass::Traffic => (ModelId::Gtcrn, 4),
        NoiseClass::Music => (ModelId::FastEnhancerM, 5),
    };
    p.model = model;
    p.tone = ToneId(if d == Device::BtNarrow { tone + 10 } else { tone });
    p
}

/// 整张路由表的文本（行格式照 vdn-types `SceneTable` 的思路：一行一个细场景），给 README 与接入时导入用。
pub fn table_text() -> String {
    let mut s = String::from("# id name user model tone\n");
    let mut nm = String::new();
    for i in 1..=22u16 {
        let id = FineSceneId(i);
        let p = params_for(id);
        name(id, &mut nm);
        s.push_str(&format!("{i} {nm} {} {} {}\n", user_of_id(id).as_str(), p.model.as_str(), p.tone.0));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_roundtrip() {
        for c in NoiseClass::ALL {
            for d in [Device::Normal, Device::BtNarrow, Device::Clipping] {
                assert_eq!(decode(fine_id(c, d)), Some((c, d)));
            }
        }
        assert_eq!(decode(FineSceneId::UNKNOWN), None);
        assert_eq!(decode(MEETING), None);
        assert_eq!(user_of_id(MEETING), UserScene::Meeting);
    }

    #[test]
    fn evidence_based_models() {
        assert_eq!(params_for(fine_id(NoiseClass::Babble, Device::Normal)).model, ModelId::FastEnhancerM);
        assert_eq!(params_for(fine_id(NoiseClass::Neighbor, Device::Normal)).model, ModelId::Gtcrn);
        assert_eq!(params_for(fine_id(NoiseClass::Neighbor, Device::BtNarrow)).tone, ToneId(13));
    }
}
