//! 参数组：策略的输出、场景参数表的一格、「我的配置」覆盖的对象（005「场景路由」：
//! 模型、衰减下限、背景保留、门控灵敏度、音色目标）。
//!
//! `ParamSet` 是 `Copy` 的定长结构：控制线程算好，经 rtrb 整包送进实时线程（规约 §3.4「必须同时生效的一组参数」），
//! 实时线程只做逐帧平滑（[`ParamSet::glide_toward`]），不分配。

use core::fmt;

/// 用哪个降噪模型。路由池里只放同一延迟档的模型（本批硬约束 1）；当前两档都是 32 ms（见 `vdn-models`）。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ModelId {
    #[default]
    Gtcrn,
    FastEnhancerM,
}

impl ModelId {
    pub const ALL: [ModelId; 2] = [ModelId::Gtcrn, ModelId::FastEnhancerM];

    /// 模型名的**正本**（命令行、配置 JSON、场景参数表、会话记录、评测结果文件都用它）。
    /// 别处一律从这里取，不手写；`xtask tidy` 的 R-MODEL 查别处有没有写死。
    pub const fn as_str(self) -> &'static str {
        match self {
            ModelId::Gtcrn => "gtcrn",
            ModelId::FastEnhancerM => "fastenhancer-m",
        }
    }

    /// 全部正本名（`ALL` 的顺序）：命令行的可选值、schema 的枚举从这里取。
    pub const NAMES: [&'static str; 2] = [ModelId::Gtcrn.as_str(), ModelId::FastEnhancerM.as_str()];

    /// 旧写法 → 模型。只为读 2026-11-02 之前写出的文本（场景参数表 / 我的配置 / 会话记录）；写出一律用正本。
    /// 兼容只在这一处（[`ModelId::parse`]），别处不许认旧写法。
    pub const LEGACY: [(&'static str, ModelId); 1] = [("fastenhancer_m", ModelId::FastEnhancerM)];

    /// 正本名或旧写法 → 模型。
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == s).or_else(|| {
            Self::LEGACY
                .into_iter()
                .find(|(old, _)| *old == s)
                .map(|(_, m)| m)
        })
    }
}

/// 修音链预设编号（`0` = 不加工；`1..=N` 是 `vdn-polish` 里的预设：频谱平滑强度、舒适噪声、去发闷上限、AGC 等）。
/// 路由表只给编号，预设内容归 polish。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ToneId(pub u8);

/// 连续可调的参数字段。旋钮、覆盖、文本键都按它寻址。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParamField {
    /// 降噪强度 0..=1（D36 给用户的旋钮）：输出 = 原声 + strength·(降噪结果 − 原声)，1 = 全量。
    /// 只管模型的湿干比，不缩放修音链（修音链走 `tone` 预设）——一个旋钮一个含义，日志与学习才读得懂
    Strength,
    /// 衰减下限（dB，≤ 0）：停顿时背景最多压到这里，即「背景保留」
    FloorDb,
    /// 有声时原声占比（dB，≤ 0）：快开
    OpenDb,
    /// 门控灵敏度：高出底噪多少 dB 判为有声
    DetectDb,
    /// 人声停后回到下限的时间常数（秒）：慢关
    ReleaseS,
    /// 人声停后先保持多久（秒）
    HangoverS,
}

impl ParamField {
    pub const ALL: [ParamField; 6] = [
        ParamField::Strength,
        ParamField::FloorDb,
        ParamField::OpenDb,
        ParamField::DetectDb,
        ParamField::ReleaseS,
        ParamField::HangoverS,
    ];

    /// 这个字段在参数描述表里的那一条（范围、默认值、键名的正本，[`crate::desc`]）。
    pub const fn desc(self) -> &'static crate::desc::ParamDesc {
        match self {
            ParamField::Strength => &crate::desc::STRENGTH,
            ParamField::FloorDb => &crate::desc::FLOOR_DB,
            ParamField::OpenDb => &crate::desc::OPEN_DB,
            ParamField::DetectDb => &crate::desc::DETECT_DB,
            ParamField::ReleaseS => &crate::desc::RELEASE_S,
            ParamField::HangoverS => &crate::desc::HANGOVER_S,
        }
    }

    /// 文本键名 = 描述表的 `key`。
    pub fn as_str(self) -> &'static str {
        self.desc().key
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.as_str() == s)
    }

    /// 合法范围（闭区间），取自描述表。越界的参数表 / 配置在加载时拒收，不在实时线程夹值。
    pub fn range(self) -> (f32, f32) {
        let d = self.desc();
        (d.min, d.max)
    }
}

impl fmt::Display for ParamField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 参数越界。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamError {
    pub field: ParamField,
    pub value: f32,
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lo, hi) = self.field.range();
        write!(f, "{} = {} 不在 [{lo}, {hi}]", self.field, self.value)
    }
}

impl std::error::Error for ParamError {}

/// 一组参数。默认值取自参数描述表（[`crate::desc`]，= p1 `SafetyConfig::default()` 的初值，待 AB 听测标定）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamSet {
    pub model: ModelId,
    pub tone: ToneId,
    pub strength: f32,
    pub floor_db: f32,
    pub open_db: f32,
    pub detect_db: f32,
    pub release_s: f32,
    pub hangover_s: f32,
}

impl Default for ParamSet {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// 实时可调（glide_toward 在实时线程逐帧平滑）：不分配、不 panic（specs/rust/architecture.md §5.3）
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl ParamSet {
    pub const DEFAULT: ParamSet = ParamSet {
        model: ModelId::Gtcrn,
        tone: ToneId(0),
        strength: crate::desc::STRENGTH.default,
        floor_db: crate::desc::FLOOR_DB.default,
        open_db: crate::desc::OPEN_DB.default,
        detect_db: crate::desc::DETECT_DB.default,
        release_s: crate::desc::RELEASE_S.default,
        hangover_s: crate::desc::HANGOVER_S.default,
    };

    pub fn get(&self, f: ParamField) -> f32 {
        match f {
            ParamField::Strength => self.strength,
            ParamField::FloorDb => self.floor_db,
            ParamField::OpenDb => self.open_db,
            ParamField::DetectDb => self.detect_db,
            ParamField::ReleaseS => self.release_s,
            ParamField::HangoverS => self.hangover_s,
        }
    }

    pub fn set(&mut self, f: ParamField, v: f32) {
        let slot = match f {
            ParamField::Strength => &mut self.strength,
            ParamField::FloorDb => &mut self.floor_db,
            ParamField::OpenDb => &mut self.open_db,
            ParamField::DetectDb => &mut self.detect_db,
            ParamField::ReleaseS => &mut self.release_s,
            ParamField::HangoverS => &mut self.hangover_s,
        };
        *slot = v;
    }

    /// 每个连续字段都有限且在范围内。
    pub fn validate(&self) -> Result<(), ParamError> {
        // 穷举解构：ParamSet 加了连续字段，必须同时加进 ParamField（及描述表），否则这里编译失败提醒
        let ParamSet {
            model: _,
            tone: _,
            strength: _,
            floor_db: _,
            open_db: _,
            detect_db: _,
            release_s: _,
            hangover_s: _,
        } = self;
        for f in ParamField::ALL {
            let v = self.get(f);
            let (lo, hi) = f.range();
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                return Err(ParamError { field: f, value: v });
            }
        }
        Ok(())
    }

    /// 连续字段向 `target` 走一步：`x += alpha·(target − x)`（`alpha` 来自 `vdn_dsp::one_pole_coef`）。
    /// 离散字段（模型、音色）直接取目标——换模型的交叉淡化由 engine 做（规约 §3.5）。实时线程可调，不分配。
    pub fn glide_toward(&mut self, target: &ParamSet, alpha: f32) {
        let a = alpha.clamp(0.0, 1.0);
        for f in ParamField::ALL {
            let x = self.get(f);
            self.set(f, x + a * (target.get(f) - x));
        }
        self.model = target.model;
        self.tone = target.tone;
    }
}

/// 进实时线程的参数块（ondevice 需求 ①）：`Copy` 纯数据，经 rtrb 送一个包（规约 §3.4「必须同时生效的一组参数」）。
/// 实时线程在帧边界取最新一份，没有新的就沿用旧的，**绝不等**。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ParamBlock {
    /// 单调递增；实时侧只认比手上新的
    pub seq: u32,
    /// 生成它的配置（场景参数表 + 我的配置）的内容 ID，进决策日志
    pub config: crate::version::ContentId,
    /// 当时选的模型文件的内容哈希；与实时侧正在跑的不一致时，离散切换走交叉淡化流程（规约 §3.5）
    pub model_hash: crate::version::ContentId,
    pub params: ParamSet,
    pub transition: crate::policy::Transition,
}

// 实时侧收块（ondevice #1）：完整格式校验在判断线程写入前做；实时线程只做有限值检查、钳位、核对模型
#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl ParamBlock {
    /// 实时线程拿到新块时调用：`seq` 不比手上新、`model_hash` 与正在跑的模型不符、或有非有限值 → `None`（沿用旧块）；
    /// 否则返回钳位进范围的参数。换模型不走参数块，走规约 §3.5 的换 Pipeline 流程。不分配。
    pub fn accept_rt(
        &self,
        held_seq: u32,
        running_model: crate::version::ContentId,
    ) -> Option<ParamSet> {
        if self.seq <= held_seq || self.model_hash != running_model {
            return None;
        }
        let mut p = self.params;
        for f in ParamField::ALL {
            let v = p.get(f);
            if !v.is_finite() {
                return None;
            }
            let (lo, hi) = f.range();
            p.set(f, v.clamp(lo, hi));
        }
        Some(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_valid() {
        ParamSet::default().validate().unwrap();
    }

    #[test]
    fn validate_rejects_out_of_range_and_nan() {
        let mut p = ParamSet {
            floor_db: 3.0,
            ..ParamSet::default()
        };
        assert_eq!(p.validate().unwrap_err().field, ParamField::FloorDb);
        p.floor_db = -12.0;
        p.strength = f32::NAN;
        assert_eq!(p.validate().unwrap_err().field, ParamField::Strength);
    }

    #[test]
    fn glide_converges_and_switches_discrete_at_once() {
        let mut cur = ParamSet::default();
        let tgt = ParamSet {
            floor_db: -24.0,
            model: ModelId::FastEnhancerM,
            ..ParamSet::default()
        };
        cur.glide_toward(&tgt, 0.5);
        assert_eq!(cur.floor_db, -18.0);
        assert_eq!(cur.model, ModelId::FastEnhancerM);
        for _ in 0..60 {
            cur.glide_toward(&tgt, 0.5);
        }
        assert!((cur.floor_db - tgt.floor_db).abs() < 1e-5);
    }

    #[test]
    fn param_block_is_small_copy() {
        fn is_copy<T: Copy>() {}
        is_copy::<ParamBlock>();
        // 经 rtrb 按值送进实时线程：守住体积
        assert!(core::mem::size_of::<ParamBlock>() <= 96);
    }

    #[test]
    fn accept_rt_rules() {
        use crate::version::ContentId;
        let m = ContentId::of(b"gtcrn");
        let mut b = ParamBlock {
            seq: 5,
            model_hash: m,
            ..Default::default()
        };
        b.params.floor_db = 9.0;
        assert_eq!(b.accept_rt(4, m).unwrap().floor_db, 0.0, "越界钳位");
        assert!(b.accept_rt(5, m).is_none(), "不比手上新");
        assert!(b.accept_rt(4, ContentId::of(b"fe")).is_none(), "模型不符");
        b.params.strength = f32::NAN;
        assert!(b.accept_rt(4, m).is_none(), "非有限值");
    }

    /// 描述表是正本：每个字段的范围、默认值、键名都和表里那一条相同；取表里的边界外一点造输入，validate 拒收、accept_rt 夹住。
    #[test]
    fn fields_follow_desc_table() {
        use crate::version::ContentId;
        let m = ContentId::of(b"gtcrn");
        for f in ParamField::ALL {
            let d = f.desc();
            assert_eq!(f.range(), (d.min, d.max));
            assert_eq!(f.as_str(), d.key);
            assert_eq!(ParamSet::DEFAULT.get(f), d.default, "{f} 默认值");
            assert_eq!(crate::desc::by_key(d.key), Some(d));
            for bad in [d.min - 0.5, d.max + 0.5] {
                let mut p = ParamSet::DEFAULT;
                p.set(f, bad);
                assert_eq!(
                    p.validate().unwrap_err(),
                    ParamError {
                        field: f,
                        value: bad
                    }
                );
                let b = ParamBlock {
                    seq: 1,
                    model_hash: m,
                    params: p,
                    ..Default::default()
                };
                assert_eq!(b.accept_rt(0, m).unwrap().get(f), d.clamp(bad));
            }
        }
    }

    #[test]
    fn names_roundtrip() {
        for f in ParamField::ALL {
            assert_eq!(ParamField::parse(f.as_str()), Some(f));
        }
        for m in ModelId::ALL {
            assert_eq!(ModelId::parse(m.as_str()), Some(m));
        }
    }

    /// FE-M 的两种写法解析到同一个模型；写出只用正本；`NAMES` 与 `ALL` 一一对应；旧写法不与正本撞名。
    #[test]
    fn model_names_single_source() {
        assert_eq!(
            ModelId::parse("fastenhancer-m"),
            Some(ModelId::FastEnhancerM)
        );
        assert_eq!(
            ModelId::parse("fastenhancer_m"),
            Some(ModelId::FastEnhancerM)
        );
        assert_eq!(ModelId::FastEnhancerM.as_str(), "fastenhancer-m");
        assert_eq!(ModelId::parse("FastEnhancer-M"), None);
        for (m, n) in ModelId::ALL.into_iter().zip(ModelId::NAMES) {
            assert_eq!(m.as_str(), n);
        }
        for (old, m) in ModelId::LEGACY {
            assert!(!ModelId::NAMES.contains(&old), "{old} 既是正本又是旧写法");
            assert_ne!(old, m.as_str());
        }
    }
}
