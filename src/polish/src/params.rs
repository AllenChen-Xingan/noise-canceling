//! 修音链的一组参数。`Copy`：判断回路（约 0.5 s）整包送进实时侧（规约 §3.4「必须同时生效的一组参数 → rtrb 送 Copy 系数包」），
//! 实时侧用 [`PolishParams::lerp`] 在几百毫秒到数秒内慢慢过渡（D40「慢慢切」）。

use vdn_types::desc;

/// 每个字段的范围与默认值的正本在 `vdn_types::desc`（`polish.*` 那几条）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolishParams {
    /// 频谱平滑强度 0..=1（0 = 不平滑）。治金属声 / 音乐噪声
    pub smooth: f32,
    /// 增益下限（dB，≤ 0）。-100 视为不设下限。治真空感：保留一点真实环境声
    pub floor_db: f32,
    /// 舒适噪声电平（dB，相对稳态底噪估计，≤ 0）。-100 视为关。治真空感：不带回人声的合成底噪
    pub comfort_db: f32,
    /// 去发闷的最大补偿（dB，≥ 0）。0 = 关
    pub demuffle_max_db: f32,
    /// AGC 开关（0 / 1；用 f32 便于插值，≥ 0.5 视为开）
    pub agc: f32,
    /// AGC 目标语音电平（dBFS，按语音帧 RMS 计）
    pub agc_target_dbfs: f32,
    /// AGC 最大增益 / 最大衰减（dB，对称）
    pub agc_range_db: f32,
}

impl PolishParams {
    /// 全关：修音链原样透传（离线对照组「修音链关」用它）。各开关量取描述表的下限（= 关），AGC 目标电平取默认。
    pub const OFF: Self = Self {
        smooth: desc::POLISH_SMOOTH.min,
        floor_db: desc::POLISH_FLOOR_DB.min,
        comfort_db: desc::POLISH_COMFORT_DB.min,
        demuffle_max_db: desc::POLISH_DEMUFFLE_MAX_DB.min,
        agc: desc::POLISH_AGC.min,
        agc_target_dbfs: desc::POLISH_AGC_TARGET_DBFS.default,
        agc_range_db: desc::POLISH_AGC_RANGE_DB.min,
    };

    /// 默认起步值（没有场景信息时），取自描述表 `vdn_types::desc`。数值来源：exp006 模拟数据上的初调，待人耳定。
    pub const DEFAULT: Self = Self {
        smooth: desc::POLISH_SMOOTH.default,
        floor_db: desc::POLISH_FLOOR_DB.default,
        comfort_db: desc::POLISH_COMFORT_DB.default,
        demuffle_max_db: desc::POLISH_DEMUFFLE_MAX_DB.default,
        agc: desc::POLISH_AGC.default,
        agc_target_dbfs: desc::POLISH_AGC_TARGET_DBFS.default,
        agc_range_db: desc::POLISH_AGC_RANGE_DB.default,
    };

    /// 每个字段都在描述表范围内（解析命令行 / 配置时调；越界拒收）。
    /// 穷举解构：结构加了字段而这里没登记描述表，编译失败。
    pub fn validate(&self) -> Result<(), vdn_types::RangeError> {
        let PolishParams {
            smooth,
            floor_db,
            comfort_db,
            demuffle_max_db,
            agc,
            agc_target_dbfs,
            agc_range_db,
        } = *self;
        desc::POLISH_SMOOTH.check(smooth)?;
        desc::POLISH_FLOOR_DB.check(floor_db)?;
        desc::POLISH_COMFORT_DB.check(comfort_db)?;
        desc::POLISH_DEMUFFLE_MAX_DB.check(demuffle_max_db)?;
        desc::POLISH_AGC.check(agc)?;
        desc::POLISH_AGC_TARGET_DBFS.check(agc_target_dbfs)?;
        desc::POLISH_AGC_RANGE_DB.check(agc_range_db)?;
        Ok(())
    }

    /// 夹进描述表范围（NaN 当默认值）。[`crate::PolishChain`] 收参数时先夹，实时侧不会见到越界值。不分配。
    pub fn clamped(self) -> Self {
        Self {
            smooth: desc::POLISH_SMOOTH.clamp(self.smooth),
            floor_db: desc::POLISH_FLOOR_DB.clamp(self.floor_db),
            comfort_db: desc::POLISH_COMFORT_DB.clamp(self.comfort_db),
            demuffle_max_db: desc::POLISH_DEMUFFLE_MAX_DB.clamp(self.demuffle_max_db),
            agc: desc::POLISH_AGC.clamp(self.agc),
            agc_target_dbfs: desc::POLISH_AGC_TARGET_DBFS.clamp(self.agc_target_dbfs),
            agc_range_db: desc::POLISH_AGC_RANGE_DB.clamp(self.agc_range_db),
        }
    }

    /// 预设表：`vdn-types` 的 `ParamSet.tone: ToneId(u8)` 就是这里的编号（0 = 不加工）。
    /// 1–6 按细场景大类给起步值；9 = 还没判出场景时用的 `DEFAULT`；+10 是同一预设的「窄带蓝牙」版（4 kHz 以上本来就没有，关掉去发闷）；
    /// **20 及以上 = 只开 AGC**：给拿不到模型频谱的模型（FastEnhancer-M 的 wav2wav 图）用——频谱节点要另做 STFT 才能跑，
    /// 会多 16 ms，违反「修音链零前瞻」，所以那一路只留时域的 AGC（自审 P1）。
    /// 数值是 exp006 模拟数据上的初调，**待人耳定**；未知编号退回 `DEFAULT`。
    pub fn preset(tone: u8) -> Self {
        let base = |smooth, floor_db, comfort_db, demuffle_max_db| Self {
            smooth,
            floor_db,
            comfort_db,
            demuffle_max_db,
            agc: 1.0,
            agc_target_dbfs: -26.0,
            agc_range_db: 10.0,
        };
        if tone >= 20 {
            return Self {
                agc: 1.0,
                ..Self::OFF
            };
        }
        let (b, narrow) = if tone > 10 {
            (tone - 10, true)
        } else {
            (tone, false)
        };
        let mut p = match b {
            0 => return Self::OFF,
            1 => base(0.4, -30.0, -12.0, 3.0), // 安静 / 空调：残留少，轻处理
            2 => base(0.8, -22.0, -8.0, 4.0),  // 咖啡馆 babble / 杯碟：音乐噪声最多，平滑最强
            3 => base(0.6, -30.0, -6.0, 4.0), // 邻座说话：下限压低（别把旁人的话带回来），用合成舒适噪声补真空
            4 => base(0.7, -22.0, -8.0, 3.0), // 交通：低频轰鸣，保留一点环境感
            5 => base(0.8, -20.0, -10.0, 2.0), // 音乐：残留是有音高的，平滑强
            6 => base(0.3, -16.0, -100.0, 3.0), // 多人会议（占位）：近处人声都留，处理最轻
            9 => Self::DEFAULT,
            _ => Self::DEFAULT,
        };
        if narrow {
            p.demuffle_max_db = 0.0;
        }
        p
    }

    /// 线性插值：`t` = 0 得 `a`，1 得 `b`。
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let l = |x: f32, y: f32| x + (y - x) * t;
        Self {
            smooth: l(a.smooth, b.smooth),
            floor_db: l(a.floor_db, b.floor_db),
            comfort_db: l(a.comfort_db, b.comfort_db),
            demuffle_max_db: l(a.demuffle_max_db, b.demuffle_max_db),
            agc: l(a.agc, b.agc),
            agc_target_dbfs: l(a.agc_target_dbfs, b.agc_target_dbfs),
            agc_range_db: l(a.agc_range_db, b.agc_range_db),
        }
    }
}

impl Default for PolishParams {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_types::desc::ParamDesc;

    type Set = fn(&mut PolishParams, f32);
    type Get = fn(&PolishParams) -> f32;
    const FIELDS: [(&ParamDesc, Get, Set); 7] = [
        (&desc::POLISH_SMOOTH, |p| p.smooth, |p, v| p.smooth = v),
        (
            &desc::POLISH_FLOOR_DB,
            |p| p.floor_db,
            |p, v| p.floor_db = v,
        ),
        (
            &desc::POLISH_COMFORT_DB,
            |p| p.comfort_db,
            |p, v| p.comfort_db = v,
        ),
        (
            &desc::POLISH_DEMUFFLE_MAX_DB,
            |p| p.demuffle_max_db,
            |p, v| p.demuffle_max_db = v,
        ),
        (&desc::POLISH_AGC, |p| p.agc, |p, v| p.agc = v),
        (
            &desc::POLISH_AGC_TARGET_DBFS,
            |p| p.agc_target_dbfs,
            |p, v| p.agc_target_dbfs = v,
        ),
        (
            &desc::POLISH_AGC_RANGE_DB,
            |p| p.agc_range_db,
            |p, v| p.agc_range_db = v,
        ),
    ];

    /// 描述表是正本：默认值逐条同表；越界 validate 拒收（报的是那一条）、clamped 夹到边界。
    #[test]
    fn params_follow_desc_table() {
        for (d, get, set) in FIELDS {
            assert_eq!(get(&PolishParams::DEFAULT), d.default, "{} 默认值", d.key);
            for bad in [d.min - 0.5, d.max + 0.5] {
                let mut p = PolishParams::DEFAULT;
                set(&mut p, bad);
                assert_eq!(p.validate().unwrap_err().desc.id, d.id, "{}", d.key);
                assert_eq!(get(&p.clamped()), d.clamp(bad), "{}", d.key);
            }
        }
    }

    /// 处理图里修音链两个挂点的参数清单（`vdn_types::graph`）与本表 + 过渡时长一致（交叉校验，D72）。
    #[test]
    fn graph_kinds_list_the_same_params() {
        use vdn_types::graph::{POLISH_AGC, POLISH_SPECTRAL};
        let mut want: Vec<u16> = FIELDS.iter().map(|f| f.0.id).collect();
        want.push(desc::POLISH_RAMP_S.id);
        want.sort_unstable();
        let mut got: Vec<u16> = POLISH_AGC.param_ids();
        got.extend(POLISH_SPECTRAL.param_ids());
        got.sort_unstable();
        got.dedup();
        assert_eq!(got, want);
    }

    /// 预设表是数据（不是第二份范围），但每个预设都必须落在描述表范围内。
    #[test]
    fn every_preset_is_in_range() {
        PolishParams::OFF.validate().unwrap();
        for tone in 0..=u8::MAX {
            let p = PolishParams::preset(tone);
            p.validate().unwrap_or_else(|e| panic!("预设 {tone}：{e}"));
        }
    }
}
