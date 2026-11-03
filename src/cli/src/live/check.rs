//! `vdn run --check` 的判定（机器辅助 V5；最终仍由人耳判）与「输入全零」提示的节流。

use vdn_io_macos::Level;
/// 「输入全零」提示的节流：进入全零时提示一次；有声音之后再全零满 1 s 才会再提示（所以最多每秒一次，不刷屏）。
#[derive(Debug, Default)]
pub struct SilenceNotice {
    shown: bool,
}

impl SilenceNotice {
    /// 返回 true = 这次该打印提示。
    pub fn update(&mut self, silent_now: bool) -> bool {
        match (silent_now, self.shown) {
            (true, false) => {
                self.shown = true;
                true
            }
            (false, true) => {
                self.shown = false;
                false
            }
            _ => false,
        }
    }
}

/// `--check` 的判定（机器辅助 V5；最终仍由人耳判）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LiveVerdict {
    /// 一个麦克风样本都没收到
    NoInput,
    /// 收到的全是 0：疑似没有麦克风权限或设备静音
    InputAllZero,
    /// 直通时输出电平和「输入 + 增益」差太多（dB）：链路里丢了声音
    OutputMismatch { diff_db: f64 },
    /// 输入有声、（直通时）输出电平对得上。`quiet` = 输入峰值低于 [`QUIET_PEAK_DBFS`]，耳机里可能听不出来
    Pass { quiet: bool, diff_db: Option<f64> },
}

/// 直通时输出均方根允许偏离「输入 + 增益」多少 dB
pub const LEVEL_TOLERANCE_DB: f64 = 3.0;
/// 输入峰值低于这个就提示「很小声」。2026-11-02 实测：MacBook Air 麦克风正常说话峰值约 −28 dBFS，
/// 人戴耳机听不出自己的声音；取 −20 dBFS 让这种情况被提示（经验值，未标定）
pub const QUIET_PEAK_DBFS: f64 = -20.0;

/// 判定。`passthrough` 时才比对输入 / 输出电平（降噪会改电平）。
pub fn judge(input: &Level, output: &Level, passthrough: bool, gain_db: f64) -> LiveVerdict {
    if input.samples == 0 {
        return LiveVerdict::NoInput;
    }
    if input.all_zero() {
        return LiveVerdict::InputAllZero;
    }
    let quiet = input.peak_dbfs() < QUIET_PEAK_DBFS;
    if !passthrough {
        return LiveVerdict::Pass {
            quiet,
            diff_db: None,
        };
    }
    let diff = output.rms_dbfs() - input.rms_dbfs() - gain_db;
    // 输出全零时 diff = −∞，也落在这里
    if !diff.is_finite() || diff.abs() > LEVEL_TOLERANCE_DB {
        return LiveVerdict::OutputMismatch { diff_db: diff };
    }
    LiveVerdict::Pass {
        quiet,
        diff_db: Some(diff),
    }
}

impl LiveVerdict {
    pub fn passed(&self) -> bool {
        matches!(self, LiveVerdict::Pass { .. })
    }
    /// 给人看的一行
    pub fn describe(&self) -> String {
        match self {
            LiveVerdict::NoInput => "不过：没收到麦克风数据（设备没开起来？）".into(),
            LiveVerdict::InputAllZero => {
                "不过：输入全零——疑似没有麦克风权限（系统设置 › 隐私与安全性 › 麦克风）或设备静音"
                    .into()
            }
            LiveVerdict::OutputMismatch { diff_db } => format!(
                "不过：输入有声，但输出电平和输入差 {} dB（允许 ±{LEVEL_TOLERANCE_DB}）——链路里丢了声音",
                if diff_db.is_finite() {
                    format!("{diff_db:+.1}")
                } else {
                    "−∞".into()
                }
            ),
            LiveVerdict::Pass { quiet, diff_db } => {
                let mut s = String::from("过（机器部分）：输入有声");
                if let Some(d) = diff_db {
                    s.push_str(&format!(" · 输出电平与输入一致（差 {d:+.1} dB）"));
                }
                if *quiet {
                    s.push_str(&format!(
                        " · ⚠️ 输入很小声（峰值低于 {QUIET_PEAK_DBFS} dBFS），耳机里可能听不出来：加 --gain-db 12 或回放 --record 的文件"
                    ));
                }
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 输出电平和输入对不上（例如输出全零）→ 判不过
    #[test]
    fn judge_cases() {
        let lvl = |peak: f32, rms: f64| Level {
            samples: u64::from(vdn_io_macos::IO_RATE),
            peak,
            rms,
            ..Level::default()
        };
        let speech = lvl(0.04, 0.011); // 峰值约 −28 dBFS、均方根约 −39 dBFS（2026-11-02 实测量级）
        let zero = lvl(0.0, 0.0);
        assert_eq!(
            judge(&Level::default(), &speech, true, 0.0),
            LiveVerdict::NoInput
        );
        assert_eq!(judge(&zero, &zero, true, 0.0), LiveVerdict::InputAllZero);
        assert!(matches!(
            judge(&speech, &zero, true, 0.0),
            LiveVerdict::OutputMismatch { diff_db } if diff_db == f64::NEG_INFINITY
        ));
        // 输出低 6 dB：超出 ±3 dB
        assert!(matches!(
            judge(&speech, &lvl(0.02, 0.0055), true, 0.0),
            LiveVerdict::OutputMismatch { .. }
        ));
        // 小声但通：过，带「很小声」提示
        assert!(matches!(
            judge(&speech, &speech, true, 0.0),
            LiveVerdict::Pass { quiet: true, .. }
        ));
        // 降噪模式不比电平
        assert_eq!(
            judge(&lvl(0.5, 0.2), &zero, false, 0.0),
            LiveVerdict::Pass {
                quiet: false,
                diff_db: None
            }
        );
        assert!(LiveVerdict::InputAllZero.describe().contains("麦克风权限"));
    }

    /// 「输入全零」提示不刷屏：一段连续全零只提示一次，有声后再全零才再提示
    #[test]
    fn silence_notice_fires_once_per_episode() {
        let mut n = SilenceNotice::default();
        let fired: Vec<bool> = [false, true, true, true, false, false, true, true]
            .into_iter()
            .map(|s| n.update(s))
            .collect();
        assert_eq!(
            fired,
            [false, true, false, false, false, false, true, false]
        );
    }
}
