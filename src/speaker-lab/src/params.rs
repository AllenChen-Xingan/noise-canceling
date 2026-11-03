//! 命令行参数解析：取旗标值、声纹模型路径、判断回路与门控参数（越界按 `vdn_types::desc` 报错）。

use std::path::PathBuf;

use vdn_speaker::embed::default_model_path;
use vdn_speaker::gate::GateParams;
use vdn_speaker::judge::JudgeParams;

use crate::R;

pub(crate) fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

pub(crate) fn speaker_path(args: &[String]) -> R<PathBuf> {
    arg(args, "--speaker-onnx")
        .map(PathBuf::from)
        .or_else(default_model_path)
        .ok_or_else(|| "找不到声纹模型（--speaker-onnx 或 VDN_SPEAKER_ONNX）".into())
}

/// 判断回路参数：命令行覆盖默认值，再按描述表（`vdn_types::desc` 的 `judge.*`）校验，越界报错。
pub(crate) fn judge_params(args: &[String]) -> R<JudgeParams> {
    let mut jp = JudgeParams::default();
    if let Some(v) = arg(args, "--win") {
        jp.window_s = v.parse()?;
    }
    if let Some(v) = arg(args, "--judge-hop") {
        jp.hop_s = v.parse()?;
    }
    if let Some(v) = arg(args, "--owner-hi") {
        jp.owner_hi = v.parse()?;
    }
    if let Some(v) = arg(args, "--others-lo") {
        jp.others_lo = v.parse()?;
    }
    jp.validate()?;
    Ok(jp)
}

pub(crate) fn gate_params(args: &[String]) -> R<GateParams> {
    let mut p = GateParams::default();
    let f = |name: &str| -> R<Option<f32>> {
        Ok(match arg(args, name) {
            Some(v) => Some(v.parse()?),
            None => None,
        })
    };
    if let Some(v) = f("--onset-margin")? {
        p.onset_level_margin_db = v;
    }
    if let Some(v) = f("--onset-gap")? {
        p.onset_gap_s = v;
    }
    if let Some(v) = f("--jump-db")? {
        p.onset_jump_db = v;
    }
    if let Some(v) = f("--gate-floor")? {
        p.floor_db = v;
    }
    if let Some(v) = f("--release")? {
        p.release_s = v;
    }
    if let Some(v) = f("--level-close")? {
        p.level_close_db = Some(v);
    }
    // 整数参数直接按 u32 解析：`--confirm 20.9` 报错，不截断成 20 再过范围检查
    if let Some(v) = arg(args, "--confirm") {
        p.confirm_n = v.parse()?;
    }
    // 范围的正本是 vdn_types::desc：越界直接报错，不悄悄夹值
    p.validate()?;
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sv(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn arg_returns_value_after_flag_and_none_when_missing_or_dangling() {
        let a = sv(&["in.wav", "--tag", "_v2", "--wet"]);
        assert_eq!(arg(&a, "--tag").as_deref(), Some("_v2"));
        assert_eq!(arg(&a, "--replay"), None);
        // 旗标在末尾、后面没有值：返回 None，不越界
        assert_eq!(arg(&a, "--wet"), None);
    }

    #[test]
    fn gate_params_overrides_only_given_flags() {
        let d = GateParams::default();
        let p = gate_params(&sv(&[
            "--gate-floor",
            "-18.5",
            "--confirm",
            "3",
            "--level-close",
            "12",
        ]))
        .expect("合法参数");
        assert_eq!(p.floor_db, -18.5);
        assert_eq!(p.confirm_n, 3);
        assert_eq!(p.level_close_db, Some(12.0));
        // 没给的保持默认
        assert_eq!(p.release_s, d.release_s);
        assert_eq!(p.onset_gap_s, d.onset_gap_s);
        assert_eq!(p.onset_jump_db, d.onset_jump_db);
        assert_eq!(p.onset_level_margin_db, d.onset_level_margin_db);


    /// 越界（按 vdn_types::desc 的范围）报错；报错里带键名。
    #[test]
    fn gate_params_rejects_out_of_range_by_desc_table() {
        let floor_below = format!("{}", vdn_types::desc::FLOOR_DB.min - 1.0);
        let e = gate_params(&sv(&["--gate-floor", &floor_below])).unwrap_err();
        assert!(e.to_string().contains("floor_db"), "{e}");
        let e = gate_params(&sv(&["--level-close", "-40"])).unwrap_err();
        assert!(e.to_string().contains("gate.level_close_db"), "{e}");
        assert!(gate_params(&sv(&["--confirm", "0"])).is_err());
        // 小数不截断：20.9 不能悄悄变成 20 再通过
        assert!(gate_params(&sv(&["--confirm", "20.9"])).is_err());
        assert!(gate_params(&sv(&["--confirm", "21"])).is_err());


    /// 判断回路参数：合法值照用；越界（如 `--win 20`，表上限 10 s）报错，不会一边按 20 s 建模型、一边被夹成 10 s。
    #[test]
    fn judge_params_parse_and_reject_by_desc_table() {
        let p = judge_params(&sv(&["--win", "2", "--owner-hi", "0.5"])).expect("合法参数");
        assert_eq!((p.window_s, p.owner_hi), (2.0, 0.5));
        assert_eq!(p.hop_s, JudgeParams::default().hop_s);
        let e = judge_params(&sv(&["--win", "20"])).unwrap_err();
        assert!(e.to_string().contains("judge.window_s"), "{e}");
        assert!(judge_params(&sv(&["--others-lo", "1.5"])).is_err());


    #[test]
    fn gate_params_rejects_non_numeric_value() {
        assert!(gate_params(&sv(&["--release", "slow"])).is_err());
}
