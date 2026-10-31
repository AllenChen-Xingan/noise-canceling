//! 修音链的一组参数。`Copy`：判断回路（约 0.5 s）整包送进实时侧（规约 §3.4「必须同时生效的一组参数 → rtrb 送 Copy 系数包」），
//! 实时侧用 [`PolishParams::lerp`] 在几百毫秒到数秒内慢慢过渡（D40「慢慢切」）。

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
    /// 全关：修音链原样透传（离线对照组「修音链关」用它）。
    pub const OFF: Self = Self {
        smooth: 0.0,
        floor_db: -100.0,
        comfort_db: -100.0,
        demuffle_max_db: 0.0,
        agc: 0.0,
        agc_target_dbfs: -26.0,
        agc_range_db: 0.0,
    };

    /// 默认起步值（没有场景信息时）。数值来源：exp006 模拟数据上的初调，待人耳定。
    pub const DEFAULT: Self = Self {
        smooth: 0.6,
        floor_db: -24.0,
        comfort_db: -10.0,
        demuffle_max_db: 4.0,
        agc: 1.0,
        agc_target_dbfs: -26.0,
        agc_range_db: 10.0,
    };

    /// 预设表：`vdn-types` 的 `ParamSet.tone: ToneId(u8)` 就是这里的编号（0 = 不加工）。
    /// 1–6 按细场景大类给起步值；9 = 还没判出场景时用的 `DEFAULT`；+10 是同一预设的「窄带蓝牙」版（4 kHz 以上本来就没有，关掉去发闷）。
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
