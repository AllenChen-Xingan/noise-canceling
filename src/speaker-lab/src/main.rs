//! vdn-speaker-lab：P2 离线实验命令行（exp005）。从 vdn-speaker 的 offline feature 挪出（2026-10-31 p2fix，规约 §2.2 规则 1）。
//!
//! 子命令与参数以 `src/speaker-lab/README.md` 为准（denoise · run · envelope · fbank · embed），这里不另抄一份。
//!
//! `run` 一次出 6 个版本（共用同一份降噪输出）：
//! - `raw`：模型原样输出 · `floor`：只开衰减下限 −12 dB · `energy`：p1 能量版快开慢关
//! - `vad`：VAD 版快开慢关（= 个性化关）· `personal`：VAD 版 + 判断回路 + 不对称门控（= 保守版个性化）
//! - `aggressive`：模型原样输出（不混回原声）+ 门控（下限 −20 dB、确认 1 次、慢收 0.3 s）（= 激进版个性化）
//!
//! 判断回路的判决按**实测计算时间**延后生效（向上取整到 hop），模拟它在另一个线程上跑。

#![forbid(unsafe_code)]

mod cmds;
mod params;
mod run;
mod verdict;
mod wav;

type R<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// 本 crate 只收 16 kHz（模型采样率，正本 `vdn_types::MODEL_RATE`）
const SR: u32 = vdn_types::MODEL_RATE;
const HOP: usize = 256;

/// 本进程唯一的 ORT Environment（离线命令行自己建；接入时由宿主建好传给 Embedder）。
fn ort_env() -> R<ort::environment::Environment> {
    Ok(ort::init().with_name("vdn-speaker").build()?)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.first().map(String::as_str) {
        Some("run") => run::cmd_run(&args[1..]),
        Some("denoise") => cmds::cmd_denoise(&args[1..]),
        Some("envelope") => cmds::cmd_envelope(&args[1..]),
        Some("fbank") => cmds::cmd_fbank(&args[1..]),
        Some("embed") => cmds::cmd_embed(&args[1..]),
        _ => Err("用法：vdn-speaker-lab denoise|run|envelope|fbank|embed …（见 src/speaker-lab/README.md）".into()),
    };
    if let Err(e) = r {
        eprintln!("vdn-speaker-lab: {e}");
        std::process::exit(1);
    }
}
