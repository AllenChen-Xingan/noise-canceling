//! exp017 对拍：`docs/experiments/exp017-timbre-measure/measures.py` 的 Python 量法与 Rust `V-TIMBRE`
//! 在同一批劣化音频上读数一致（差 ≤ 0.05 dB）。
//! - `old_code_port_matches_rust`：「代码版」移植（`code_timbre`，列 `code_max`）对 main 90ed08f2 的旧 `v_timbre`。
//!   只能在改量法之前跑；结果记在 `docs/experiments/exp017-timbre-measure/parity/`
//! - `erb_port_matches_rust`：调研 026 推荐量法（`erb_shape(sym=True)`，列 `erb_std_sym`）对改后的 `v_timbre`
//!
//! 音频不进仓，所以这条测试默认不跑。人工跑法：
//! 1. 主目录下 `python -I docs/experiments/exp017-timbre-measure/run.py <exp013 build 目录> _tmp/026/<run-id>`
//! 2. `VDN_EXP013=<exp013 build 目录> VDN_EXP017=_tmp/026/<run-id> cargo test -p vdn-eval --test timbre_parity -- --ignored --nocapture`

use std::path::Path;

use vdn_eval::analysis::{Masks, truth_from_clean};
use vdn_eval::synth::{Clip, Truth};
use vdn_eval::verifiers::{Ctx, Params, eval};
use vdn_eval::wav;

/// 干净人声那一路的评分上下文（同 `runner::Prepared::clean_view`：旁人与停顿掩码全空）。
fn clean_pass_timbre(clean: &[f32], out: &[f32], sr: u32) -> f64 {
    let owner_iv = truth_from_clean(clean);
    let nf = clean.len() / vdn_eval::analysis::MASK_FRAME;
    let mut owner = vec![false; nf];
    for &(a, b) in &owner_iv {
        let fb = b.div_ceil(vdn_eval::analysis::MASK_FRAME).min(nf);
        for v in owner.iter_mut().take(fb).skip(a / vdn_eval::analysis::MASK_FRAME) {
            *v = true;
        }
    }
    let masks = Masks {
        owner,
        interf: vec![false; nf],
        noise_only: vec![false; nf],
        interf_only: vec![false; nf],
    };
    let clip = Clip {
        sr,
        clean: clean.to_vec(),
        interf: vec![0.0; clean.len()],
        noise: vec![0.0; clean.len()],
        mix: clean.to_vec(),
        truth: Truth {
            owner: owner_iv,
            interf: Vec::new(),
        },
        tracks: true,
    };
    let ctx = Ctx {
        clip: &clip,
        out,
        run: None,
        masks: &masks,
        noise_out: None,
        ci: true,
    };
    eval("V-TIMBRE", &ctx, &Params::default())
        .value
        .expect("V-TIMBRE 应有读数")
}

/// readings.csv 里 (clip, deg) → 某一列。
fn python_readings(csv: &str, column: &str) -> Vec<(String, String, f64)> {
    let mut lines = csv.lines();
    let head: Vec<&str> = lines.next().unwrap().split(',').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).unwrap();
    let (ic, id, iv) = (col("clip"), col("deg"), col(column));
    lines
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            (f[ic].to_string(), f[id].to_string(), f[iv].parse().unwrap())
        })
        .collect()
}

fn parity(column: &str) {
    let e13 = std::env::var("VDN_EXP013").expect("设 VDN_EXP013");
    let e17 = std::env::var("VDN_EXP017").expect("设 VDN_EXP017");
    let csv = std::fs::read_to_string(Path::new(&e17).join("readings.csv")).unwrap();
    let mut n = 0;
    let mut worst = 0.0f64;
    for (clip, deg, py) in python_readings(&csv, column) {
        let p = Path::new(&e17).join(format!("{clip}__{deg}.wav"));
        if !p.exists() {
            continue;
        }
        let (clean, sr) = wav::read(&Path::new(&e13).join(&clip).join("owner.wav")).unwrap();
        let (out, _) = wav::read(&p).unwrap();
        let rs = clean_pass_timbre(&clean, &out, sr);
        let d = (rs - py).abs();
        println!("{column}\t{clip}\t{deg}\trust={rs:.3}\tpy={py:.3}\tdiff={d:.3}");
        worst = worst.max(d);
        n += 1;
    }
    assert!(n >= 30, "只对上 {n} 个文件");
    assert!(worst <= 0.05, "{column}：Python 与 Rust 最大差 {worst:.3} dB（{n} 个文件）");
}

#[test]
#[ignore = "要 exp017 劣化音频与 exp013 正式 build（音频不进仓），且只对改量法之前的代码有效，人工跑，步骤见文件头"]
fn old_code_port_matches_rust() {
    parity("code_max");
}

#[test]
#[ignore = "要 exp017 劣化音频与 exp013 正式 build（音频不进仓），人工跑，步骤见文件头"]
fn erb_port_matches_rust() {
    parity("erb_std_sym");
}
