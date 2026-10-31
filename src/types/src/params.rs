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

    pub fn as_str(self) -> &'static str {
        match self {
            ModelId::Gtcrn => "gtcrn",
            ModelId::FastEnhancerM => "fastenhancer_m",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == s)
    }
}

/// 音色目标预设编号（修音链的目标曲线；`0` = 不加工）。曲线本身由 scene/polish 线定义。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ToneId(pub u8);

/// 连续可调的参数字段。旋钮、覆盖、文本键都按它寻址。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ParamField {
    /// 降噪强度 0..=1（D36 给用户的旋钮）：输出 = 原声 + strength·(降噪结果 − 原声)，1 = 全量
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

    pub fn as_str(self) -> &'static str {
        match self {
            ParamField::Strength => "strength",
            ParamField::FloorDb => "floor_db",
            ParamField::OpenDb => "open_db",
            ParamField::DetectDb => "detect_db",
            ParamField::ReleaseS => "release_s",
            ParamField::HangoverS => "hangover_s",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|f| f.as_str() == s)
    }

    /// 合法范围（闭区间）。越界的参数表 / 配置在加载时拒收，不在实时线程夹值。
    pub fn range(self) -> (f32, f32) {
        match self {
            ParamField::Strength => (0.0, 1.0),
            ParamField::FloorDb => (-60.0, 0.0),
            ParamField::OpenDb => (-60.0, 0.0),
            ParamField::DetectDb => (0.0, 40.0),
            ParamField::ReleaseS => (0.0, 5.0),
            ParamField::HangoverS => (0.0, 5.0),
        }
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

/// 一组参数。默认值 = p1 `SafetyConfig::default()` 的初值（待 AB 听测标定）。
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

impl ParamSet {
    pub const DEFAULT: ParamSet = ParamSet {
            model: ModelId::Gtcrn,
            tone: ToneId(0),
            strength: 1.0,
            floor_db: -12.0,
            open_db: -6.0,
            detect_db: 9.0,
            release_s: 0.3,
            hangover_s: 0.15,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_valid() {
        ParamSet::default().validate().unwrap();
    }

    #[test]
    fn validate_rejects_out_of_range_and_nan() {
        let mut p = ParamSet::default();
        p.floor_db = 3.0;
        assert_eq!(p.validate().unwrap_err().field, ParamField::FloorDb);
        p.floor_db = -12.0;
        p.strength = f32::NAN;
        assert_eq!(p.validate().unwrap_err().field, ParamField::Strength);
    }

    #[test]
    fn glide_converges_and_switches_discrete_at_once() {
        let mut cur = ParamSet::default();
        let mut tgt = ParamSet::default();
        tgt.floor_db = -24.0;
        tgt.model = ModelId::FastEnhancerM;
        cur.glide_toward(&tgt, 0.5);
        assert_eq!(cur.floor_db, -18.0);
        assert_eq!(cur.model, ModelId::FastEnhancerM);
        for _ in 0..60 {
            cur.glide_toward(&tgt, 0.5);
        }
        assert!((cur.floor_db - tgt.floor_db).abs() < 1e-5);
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
}
