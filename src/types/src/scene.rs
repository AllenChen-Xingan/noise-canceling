//! 感知的输出：两层场景标签（D41）。
//!
//! - **用户层** [`UserScene`]：少数基本场景，用户看得到、可手动切、可锁定
//! - **内部细场景** [`FineSceneId`]：如「咖啡馆 + 邻座一人 + 蓝牙」，自动判断、用户无感；
//!   类目由场景参数表定义（[`crate::table`]），这里只给编号，加类目不改代码
//!
//! 感知跑在判断回路（约 0.5 s 一次，C 档线程），只产出标签，不碰声音、不加延迟（本批硬约束 1）。

use core::fmt;

use crate::frame::{Confidence, SampleClock};

/// 用户层基本场景。类目由人定（scene 线提议，写进待人拍板）；`Meeting` = 一屋子人共用一台电脑，
/// 保留近处所有人声、只压远处噪音，暂时只能手动选。不设 `Other`：判不准时保持当前场景。
/// 加一个变体 = 改这里 + `ALL` + 名字表，文本格式用名字存，不受顺序影响。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UserScene {
    #[default]
    Quiet,
    Cafe,
    Commute,
    Music,
    Meeting,
}

impl UserScene {
    pub const ALL: [UserScene; 5] = [
        UserScene::Quiet,
        UserScene::Cafe,
        UserScene::Commute,
        UserScene::Music,
        UserScene::Meeting,
    ];

    /// 文本格式里的名字（稳定，不随显示文案变）。
    pub fn as_str(self) -> &'static str {
        match self {
            UserScene::Quiet => "quiet",
            UserScene::Cafe => "cafe",
            UserScene::Commute => "commute",
            UserScene::Music => "music",
            UserScene::Meeting => "meeting",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|u| u.as_str() == s)
    }
}

impl fmt::Display for UserScene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 内部细场景编号。含义查场景参数表；`0` 保留给「未知 / 还没判出来」。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FineSceneId(pub u16);

impl FineSceneId {
    pub const UNKNOWN: FineSceneId = FineSceneId(0);
}

/// 细场景的短名，定长存放（`Copy`，可整表按值热更新）。最多 [`SceneName::CAP`] 字节 UTF-8。
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneName {
    len: u8,
    buf: [u8; SceneName::CAP],
}

impl SceneName {
    pub const CAP: usize = 32;
    /// 空名（只用作定长表里未用槽位的占位）。
    pub const EMPTY: SceneName = SceneName {
        len: 0,
        buf: [0; SceneName::CAP],
    };

    /// 超长、含空白或 `#[]=` 时返回 `None`（这些字符会破坏文本格式）。
    pub fn new(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.is_empty()
            || bytes.len() > Self::CAP
            || s.chars().any(|c| c.is_whitespace() || "#[]=".contains(c))
        {
            return None;
        }
        let mut buf = [0u8; Self::CAP];
        buf.get_mut(..bytes.len())?.copy_from_slice(bytes);
        Some(Self {
            len: bytes.len() as u8,
            buf,
        })
    }

    pub fn as_str(&self) -> &str {
        // new() 只收合法 UTF-8 的完整字符串，切片不会落在字符中间
        self.buf
            .get(..usize::from(self.len))
            .and_then(|b| core::str::from_utf8(b).ok())
            .unwrap_or("")
    }
}

impl fmt::Debug for SceneName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SceneName({:?})", self.as_str())
    }
}

/// 感知一次的输出。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneLabel {
    /// 用户层场景（由细场景查表得到，或分类器直接给）
    pub user: UserScene,
    /// 内部细场景
    pub fine: FineSceneId,
    /// 对 `fine` 的置信度
    pub confidence: Confidence,
    /// 这次判断所用音频窗的**末尾**时刻
    pub at: SampleClock,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_scene_names_roundtrip() {
        for u in UserScene::ALL {
            assert_eq!(UserScene::parse(u.as_str()), Some(u));
        }
        assert_eq!(UserScene::parse("nope"), None);
    }

    #[test]
    fn scene_name_rejects_bad_text() {
        assert_eq!(SceneName::new("cafe+bt").unwrap().as_str(), "cafe+bt");
        assert_eq!(SceneName::new("咖啡馆").unwrap().as_str(), "咖啡馆");
        assert!(SceneName::new("").is_none());
        assert!(SceneName::new("a b").is_none());
        assert!(SceneName::new("a=b").is_none());
        assert!(SceneName::new(&"x".repeat(33)).is_none());
    }
}
