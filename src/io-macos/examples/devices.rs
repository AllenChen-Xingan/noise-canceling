//! 列本机音频设备（不开流，不弹权限框）：`cargo run -p vdn-io-macos --example devices`
fn main() {
    println!("| 名字 | 方向 | 默认采样率 | 声道 | 默认 | 支持 48 kHz |");
    println!("|---|---|---|---|---|---|");
    for d in vdn_io_macos::devices() {
        println!(
            "| {} | {} | {} | {} | {} | {} |",
            d.name,
            d.direction,
            d.default_rate,
            d.channels,
            if d.is_default { "是" } else { "" },
            if d.supports_io_rate { "是" } else { "否" }
        );
    }
}
