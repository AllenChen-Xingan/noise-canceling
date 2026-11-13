//! 运行时的电源状态（调研 025 A2）：计时类读数（最坏一帧等）受供电影响，电池时可能降频（项目经验：电池降半频不可混比）。
//! 读 `pmset -g batt` 的第一行；读不到或认不出一律记「不知道」，不猜。

/// 电源状态：`"ac"` 接电源 · `"battery"` 用电池 · `"unknown"` 不知道。
pub fn source() -> &'static str {
    std::process::Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map_or("unknown", |o| parse(&String::from_utf8_lossy(&o.stdout)))
}

/// 解析 `pmset -g batt` 的输出（第一行形如 `Now drawing from 'AC Power'`）。
pub fn parse(text: &str) -> &'static str {
    let first = text.lines().next().unwrap_or_default();
    if first.contains("'AC Power'") {
        "ac"
    } else if first.contains("'Battery Power'") {
        "battery"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn parses_pmset_first_line() {
        assert_eq!(
            parse("Now drawing from 'AC Power'\n -InternalBattery-0 100%"),
            "ac"
        );
        assert_eq!(parse("Now drawing from 'Battery Power'\n"), "battery");
        assert_eq!(parse(""), "unknown");
        assert_eq!(parse("Now drawing from 'UPS Power'"), "unknown");
    }
}
