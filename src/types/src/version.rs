//! 版本号与内容 ID（005 第三节 A/B：每个配置都有版本、可回滚；规约 §6 借 Unison：用内容哈希当 ID）。
//!
//! - **版本号**（`TableVersion` / `ProfileVersion`）：单调递增，人读得懂，回滚 = 换回旧版本号那份
//! - **内容 ID**（`ContentId`）：规范文本的 FNV-1a 64 位哈希，决策日志里记它，事后能确认「当时用的就是这份」
//! - **格式号**（各模块的 `FORMAT` 常量）：文本格式本身变了才加，读到更高的格式号一律拒读

use core::fmt;

/// 场景参数表的版本。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TableVersion(pub u32);

/// 「我的配置」的版本。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProfileVersion(pub u32);

impl TableVersion {
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl ProfileVersion {
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// 内容 ID：FNV-1a 64。不是密码学哈希，只用来认「是不是同一份」。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ContentId(pub u64);

impl ContentId {
    pub fn of(bytes: &[u8]) -> Self {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in bytes {
            h ^= u64::from(*b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self(h)
    }

    /// 解析 16 位十六进制。
    pub fn parse(s: &str) -> Option<Self> {
        if s.len() != 16 {
            return None;
        }
        u64::from_str_radix(s, 16).ok().map(Self)
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_known_vectors() {
        // FNV-1a 64 的公开测试向量
        assert_eq!(ContentId::of(b"").0, 0xcbf2_9ce4_8422_2325);
        assert_eq!(ContentId::of(b"a").0, 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn content_id_roundtrips_as_hex() {
        let id = ContentId::of(b"vdn");
        assert_eq!(ContentId::parse(&id.to_string()), Some(id));
        assert_eq!(ContentId::parse("xyz"), None);
    }

    #[test]
    fn versions_saturate() {
        assert_eq!(TableVersion(u32::MAX).next(), TableVersion(u32::MAX));
        assert_eq!(ProfileVersion(1).next(), ProfileVersion(2));
    }
}
