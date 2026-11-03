//! exp010 推理后端基准：同口径比 ORT CPU（基线）、手写 Rust、tract（纯 Rust ONNX 推理）。
//!
//! 子命令：
//!   gtcrn-parity <权重前缀> <onnx> <wav> [--vs native|tract] [--frames N] [--reset-at K]
//!                逐帧对拍增强后频谱（ORT 对 --vs，默认 native；两边吃同一份 STFT 频谱）；--frames 循环音频跑长；
//!                --reset-at：被测一侧跑 K 帧后 reset，对比新开实例（只 native 实现了 reset）
//!   gtcrn-bench  <ort|native|tract> <权重前缀|onnx> <wav> [--frames N] [--warmup N]
//!   fem-parity   <onnx> <wav> [--vs tract|ort] [--b-onnx <被测侧的 onnx>] [--frames N]
//!                FastEnhancer-M：ORT 跑 <onnx> 对 --vs 跑 --b-onnx（默认同一份），逐帧对拍输出波形
//!                （--vs ort --b-onnx 改写图 = 单独证明改写等价；旧参数 --tract-onnx 等同 --b-onnx）
//!   fem-bench    <ort|tract> <onnx> <wav> [--frames N] [--warmup N]
//!   常驻内存：footprint_before_mb（加载前）/ footprint_warm_mb（预热完）/ footprint_end_mb（跑完），读 ri_phys_footprint。
//!   两个 bench 都可加 `--count-malloc`：挂 malloc_logger 数全进程 malloc（process_allocs_per_frame；不加时为 null）。
//!   这个钩子会拖慢分配，所以计数和计时分开跑。
//!   两个 bench 都可加 `--qos ui|rt|default`：计时线程的调度等级，默认 ui（QOS_CLASS_USER_INTERACTIVE）；
//!   rt = THREAD_TIME_CONSTRAINT_POLICY（period = 帧长、computation 1 ms、constraint 3 ms）。
//!   `--workgroup`：计时线程加入默认输出设备的 IO 工作组（字段 workgroup:true/false）。
//!   frames_over_3ms：计时帧里纯推理 > 3.0 ms 的个数。实时节奏下另报 late_*（醒来迟到）、release_*（计划时刻到完成，
//!   含迟到）与 release_over_3ms——BE-DEADLINE 应看 release_over_3ms（产品里帧是否赶上截止）。
//!   open_to_warm_ms：从开始打开到预热完（BE-COLD 直接量）。`--ready-file <路径>`：预热完写这个文件（耗电脚本用）。
//!   对拍输出 be_parity_pass；不过时退出码非零（--allow-fail 例外，给已知错误的对照用）；非有限指标写 null。
//!   两个 bench 都可加 `--pace-ms <毫秒>`：按实时节奏每隔这么久喂一帧（测耗电、测睡醒后的帧耗时用；默认 0 = 连续跑）。
//!   两个 bench 都可加 `--dump-io <前缀>`（此时第 2 个参数须是 onnx）：计时之后另起一个 ORT 会话，从零状态把
//!   全部 1 + warmup + frames 帧再跑一遍，写 `<前缀>.in.f32` / `<前缀>.ort.f32`（小端 f32，按帧拼接；
//!   GTCRN 每帧 re[257] 再 im[257]，FE-M 每帧 160 个样本）和 `<前缀>.json`（帧数、每帧长度、wav 路径与 sha256）。
//!   给非 Rust 的测量程序（applebe 的 Swift / Python）吃同一份输入、对同一份 ORT 输出。
//!   gtcrn-layers <权重前缀> <wav> [--frames N] [--warmup N] [--qos rt|ui|bg] [--pace-ms X] [--variant native|native-fast]
//!                研究 020：手写版逐层计时（cntvct 打点）+ 每层工作量（乘加、非线性、字节、关键路径），见 src/layers.rs
//!   hw-peak [--qos rt|ui|bg] [--reps N]  研究 020：本机 NEON 乘加吞吐 / 延迟、非线性延迟、各级缓存读写带宽，见 src/hwpeak.rs
//!   dsp-bench <wav16k> [--frames N] [--qos rt|ui|bg] [--pace-ms X]
//!                研究 020：前后处理（vdn-dsp 的 48↔16 kHz 重采样、STFT / iSTFT）每帧计时，对照「只改点积求和方式」的 NEON 版，见 src/dsp.rs
//!   gtcrn-bench 的后端另有 native-v3（研究 020 第二批，按能效核写，见 src/gtcrn_native/v3.rs）；后缀 -pf = 每帧预取权重；以及 native-v2 / native-v2-fast（研究 020：同一结构与权重，算子换 NEON 重写，见 src/gtcrn_native/v2.rs）
//!   amx-probe                                                   小矩阵上 Accelerate（AMX / SME）对手写循环，形状取自 GTCRN
//! 输出一行 JSON（stdout）。一帧 = 模型的一个 hop（GTCRN 256 样本 / 16 ms，FE-M 160 样本 / 10 ms）。

mod alloc;
mod amx_probe;
mod dsp;
mod gtcrn_native;
mod hwpeak;
mod layers;
mod nets;
mod parity;
mod tract_be;
mod util;
mod workgroup;

use alloc::*;
use nets::*;
use parity::*;
use util::*;

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use gtcrn_native::GtcrnNative;
use ort::session::builder::{AutoDevicePolicy, GraphOptimizationLevel};
use ort::session::{HasSelectedOutputs, OutputSelector, RunOptions, Session};
use ort::value::Tensor;
use tract_be::TractNet;
use vdn_dsp::{StreamingStft, window};

type Err = Box<dyn std::error::Error + Send + Sync>;

fn bench(model: ModelId, args: &[String]) -> Result<(), Err> {
    let (kind, path, wav) = (&args[0], Path::new(&args[1]), Path::new(&args[2]));
    let frames = arg_usize(args, "--frames", 6000);
    let warm = arg_usize(args, "--warmup", 200);
    let pace: f64 = arg_str(args, "--pace-ms", "0").parse()?;
    // 入口校验（codex 第 5 轮 #1 #2）：计时帧数 > 0；节奏必须有限、非负，且不超过 10 秒一帧
    if frames == 0 {
        return Err("--frames 必须 > 0".into());
    }
    let total = frames
        .checked_add(warm)
        .and_then(|v| v.checked_add(1))
        .ok_or("--frames + --warmup 溢出")?;
    if !pace.is_finite() || !(0.0..=10_000.0).contains(&pace) {
        return Err(format!("--pace-ms {pace} 无效：要有限、非负、≤ 10000").into());
    }
    let qos = arg_str(args, "--qos", "ui");
    set_thread_qos(qos, if model == ModelId::Gtcrn { 16.0 } else { 10.0 })?;
    // 加入默认输出设备的 IO 工作组（在调度等级之后、计时之前；令牌留到函数结束）
    let wg_on = args.iter().any(|a| a == "--workgroup");
    let _wg_token = if wg_on {
        Some(workgroup::join_output()?)
    } else {
        None
    };
    let count_malloc = args.iter().any(|a| a == "--count-malloc");
    if count_malloc {
        install_malloc_counter();
    }
    let audio = read_wav(wav)?;
    // 计时前：ORT 环境初始化 + 读输入形状。native 吃的是权重前缀、不碰 ORT，跳过（它的「打开」本来就不含 ORT）
    let (shapes, ort_env_ms) = if kind.starts_with("native") {
        (Vec::new(), 0.0)
    } else {
        prepare(path)?
    };
    let inputs = frames_for(model, &audio, total, &shapes)?;
    let mut x = inputs[0].clone();
    let fp_before = footprint_mb(); // 加载前（输入帧、ORT 环境已在内存里）
    // 冷启动：打开 + 第一帧
    let t0 = Instant::now();
    let mut m = open_net_with(model, kind, path, Some(&shapes))?;
    let t_open = t0.elapsed().as_secs_f64() * 1e3;
    let t1 = Instant::now();
    m.infer(&mut x)?;
    let t_first = t1.elapsed().as_secs_f64() * 1e3;
    for f in &inputs[1..=warm] {
        x.copy_from_slice(f);
        m.infer(&mut x)?;
    }
    // BE-COLD 直接量：从开始打开到预热完（codex 第 1 轮 #6）
    let open_to_warm = t0.elapsed().as_secs_f64() * 1e3;
    let fp_warm = footprint_mb();
    // 预热完发就绪信号（耗电脚本据此只采稳态窗口，codex 第 4 轮 #4）
    if let Some(i) = args.iter().position(|a| a == "--ready-file") {
        std::fs::write(args.get(i + 1).ok_or("--ready-file 缺路径")?, b"ready\n")?;
    }
    let mut times = Vec::with_capacity(frames);
    let mut nan = 0usize;
    // 实时节奏下另记：醒来迟到 = 实际开始 − 计划时刻；释放到完成 = 完成 − 计划时刻（codex 第 1 轮 #4）
    // 所有测量缓冲都在计数起点之前分配好（codex 第 2 轮 #1）
    let mut late = Vec::with_capacity(if pace > 0.0 { frames } else { 0 });
    let mut release = Vec::with_capacity(if pace > 0.0 { frames } else { 0 });
    let a0 = ALLOCS.load(Relaxed);
    let pa0 = PROC_ALLOCS.load(Relaxed);
    let c0 = cpu_seconds();
    let win_start = unix_now();
    let slept = SleepGuard::start();
    let w0 = Instant::now();
    for (n, f) in inputs[warm + 1..].iter().enumerate() {
        // 实时节奏：第 n 帧在 w0 + n·pace 时刻开始（睡眠不计入帧耗时）；连续跑时计划时刻 = w0，不睡
        let due = wait_due(w0, n, pace);
        x.copy_from_slice(f);
        let t = Instant::now();
        m.infer(&mut x)?;
        times.push(t.elapsed().as_secs_f64() * 1e3);
        if pace > 0.0 {
            late.push(t.saturating_duration_since(due).as_secs_f64() * 1e3);
            release.push(Instant::now().saturating_duration_since(due).as_secs_f64() * 1e3);
        }
        if !x.iter().all(|v| v.is_finite()) {
            nan += 1;
        }
    }
    // 循环一结束就截取两种分配计数，之后的排序、格式化不算进去（codex 第 2 轮 #1）
    let allocs = ALLOCS.load(Relaxed) - a0;
    let proc_allocs = PROC_ALLOCS.load(Relaxed) - pa0;
    let wall = w0.elapsed().as_secs_f64();
    let win_end = unix_now();
    let cpu = cpu_seconds() - c0;
    let over_3ms = times.iter().filter(|t| **t > 3.0).count(); // 纯推理 > 3 ms 的帧（同 applebe 字段）
    // 释放到完成 > 3 ms 的帧（含醒来迟到），只在实时节奏下有意义；不按节奏跑时为 null
    let release_over_3ms = if pace > 0.0 {
        release.iter().filter(|t| **t > 3.0).count().to_string()
    } else {
        "null".into()
    };
    late.sort_by(|a, b| a.total_cmp(b));
    release.sort_by(|a, b| a.total_cmp(b));
    let q = |v: &Vec<f64>, p: f64| {
        if v.is_empty() {
            "null".to_string()
        } else {
            format!("{:.4}", pct(v, p))
        }
    };
    let pace_extra = format!(
        "\"open_to_warm_ms\":{open_to_warm:.2},\"late_p50_ms\":{},\"late_p999_ms\":{},\"late_max_ms\":{},\"release_p999_ms\":{},\"release_max_ms\":{},\"release_over_3ms\":{release_over_3ms},",
        q(&late, 0.5),
        q(&late, 0.999),
        q(&late, 1.0),
        q(&release, 0.999),
        q(&release, 1.0)
    );
    let fp_end = footprint_mb();
    times.sort_by(|a, b| a.total_cmp(b));
    if let Some(i) = args.iter().position(|a| a == "--dump-io") {
        dump_io(
            model,
            path,
            wav,
            &inputs,
            args.get(i + 1).ok_or("--dump-io 缺前缀")?,
        )?;
    }
    println!(
        "{{\"model\":\"{}\",\"backend\":\"{kind}\",\"frames\":{frames},\"pace_ms\":{pace},\"qos\":\"{qos}\",\"workgroup\":{wg_on},\"p10_ms\":{:.4},\"p50_ms\":{:.4},\"p90_ms\":{:.4},\"p99_ms\":{:.4},\"p999_ms\":{:.4},\"max_ms\":{:.4},\"mean_ms\":{:.4},\"ort_env_ms\":{ort_env_ms:.2},\"open_ms\":{t_open:.2},\"first_frame_ms\":{t_first:.3},\"cpu_over_wall\":{:.3},\"rust_allocs_per_frame\":{:.3},\"rust_allocs_total\":{allocs},\"process_allocs_total\":{},\"nonfinite_frames\":{nan},\"frames_over_3ms\":{over_3ms},{pace_extra}\"process_allocs_per_frame\":{},\"footprint_before_mb\":{fp_before:.2},\"footprint_warm_mb\":{fp_warm:.2},\"footprint_end_mb\":{fp_end:.2},\"win_start_unix\":{win_start:.3},\"win_end_unix\":{win_end:.3},\"wall_s\":{wall:.3}}}",
        model.name(),
        pct(&times, 0.1),
        pct(&times, 0.5),
        pct(&times, 0.9),
        pct(&times, 0.99),
        pct(&times, 0.999),
        times[times.len() - 1],
        times.iter().sum::<f64>() / times.len() as f64,
        cpu / wall,
        allocs as f64 / frames as f64,
        if count_malloc {
            proc_allocs.to_string()
        } else {
            "null".into()
        },
        if count_malloc {
            format!("{:.3}", proc_allocs as f64 / frames as f64)
        } else {
            "null".into()
        },
    );
    // 有非有限输出：结果已打印（nonfinite_frames 字段可见），但这一轮性能作废，返回错误让脚本停下（codex 第 3 轮 #5）
    if nan > 0 {
        return Err(format!("{nan} 帧输出含非有限值，本轮性能结果无效").into());
    }
    slept.check()
}

fn main() -> Result<(), Err> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    ALLOW_FAIL.store(rest.iter().any(|a| a == "--allow-fail"), Relaxed);
    match args.first().map(String::as_str) {
        Some("amx-probe") => {
            amx_probe::run();
            Ok(())
        }
        Some("gtcrn-bench") => bench(ModelId::Gtcrn, rest),
        Some("hw-peak") => hwpeak::run(rest),
        Some("dsp-bench") => {
            let audio = read_wav(Path::new(rest.first().ok_or("缺 wav")?))?;
            dsp::run(rest, &audio)
        }
        Some("gtcrn-layers") => {
            let audio = read_wav(Path::new(rest.get(1).ok_or("缺 wav")?))?;
            layers::run(rest, |n| frames_for(ModelId::Gtcrn, &audio, n, &[]))
        }
        Some("fem-bench") => bench(ModelId::Fem, rest),
        Some("gtcrn-parity") => {
            // 帧数默认 = 音频长度 / hop（不循环）；--frames N 时循环音频（STFT 不重置），查长时间跑会不会漂
            let (wp, onnx, wav) = (
                Path::new(&rest[0]),
                Path::new(&rest[1]),
                Path::new(&rest[2]),
            );
            let vs = arg_str(rest, "--vs", "native");
            let audio = read_wav(wav)?;
            let n = arg_usize(rest, "--frames", audio.len() / HOP);
            let inputs = frames_for(ModelId::Gtcrn, &audio, n, &[])?;
            let pb = if vs.starts_with("native") { wp } else { onnx };
            parity(
                ModelId::Gtcrn,
                vs,
                open_net(ModelId::Gtcrn, "ort", onnx)?,
                open_net(ModelId::Gtcrn, vs, pb)?,
                &inputs,
            )?;
            if let Some(i) = rest.iter().position(|a| a == "--reset-at") {
                let k: usize = rest.get(i + 1).ok_or("--reset-at 缺帧号")?.parse()?;
                let (mut b, mut b2) = (
                    open_net(ModelId::Gtcrn, vs, pb)?,
                    open_net(ModelId::Gtcrn, vs, pb)?,
                );
                reset_check(ModelId::Gtcrn, vs, b.as_mut(), b2.as_mut(), &inputs, k)?;
            }
            Ok(())
        }
        Some("fem-parity") => {
            // --vs tract（默认）| ort；--b-onnx 给被测一侧用的图（如改写图），默认同 <onnx>
            let (onnx, wav) = (Path::new(&rest[0]), Path::new(&rest[1]));
            let vs = arg_str(rest, "--vs", "tract");
            let bo = Path::new(arg_str(
                rest,
                "--b-onnx",
                arg_str(rest, "--tract-onnx", &rest[0]),
            ));
            let audio = read_wav(wav)?;
            let shapes = input_shapes(&ort_session(onnx)?)?;
            let hop = *shapes[0].last().ok_or("wav_in 形状空")?;
            let n = arg_usize(rest, "--frames", audio.len() / hop);
            let inputs = frames_for(ModelId::Fem, &audio, n, &shapes)?;
            let label = format!(
                "{vs}:{}",
                bo.file_name().and_then(|s| s.to_str()).unwrap_or("?")
            );
            parity(
                ModelId::Fem,
                &label,
                open_net(ModelId::Fem, "ort", onnx)?,
                open_net_with(ModelId::Fem, vs, bo, Some(&shapes))?,
                &inputs,
            )
        }
        _ => Err("用法见 src/main.rs 顶部".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 恒等或输出 NaN 的假模型，用来测对拍和 reset 检查本身
    struct Fake {
        nan: bool,
    }
    impl Net for Fake {
        fn infer(&mut self, x: &mut [f32]) -> Result<(), Err> {
            if self.nan {
                x.fill(f32::NAN);
            }
            Ok(())
        }
    }

    /// 计时窗字段：applebe 的耗电解析（power_parse.py v2）与 apple-rs/tests/fields.rs 依赖这三个键（#58）
    #[test]
    fn bench_emits_window_fields() {
        let src = include_str!("main.rs");
        for k in ["win_start_unix", "win_end_unix", "wall_s"] {
            assert!(src.contains(&format!("\\\"{k}\\\":{{")), "bench 输出缺 {k}");
        }
        let t = unix_now();
        assert!(t > 1.7e9 && t < 1e10, "unix_now 应是 Unix 秒：{t}");
    }

    /// 节奏等待：按 2 ms 节奏跑 20 帧，总时长在 [38 ms, 20 × 2 × 1.2 + 10 ms] 之内（大总管要的回归测试）
    #[test]
    fn wait_due_keeps_pace() {
        let w0 = Instant::now();
        for n in 0..20 {
            wait_due(w0, n, 2.0);
        }
        let ms = w0.elapsed().as_secs_f64() * 1e3;
        assert!((38.0..=58.0).contains(&ms), "20 帧 × 2 ms 用了 {ms:.1} ms");
        // 连续跑（pace 0）不睡
        let t = Instant::now();
        for n in 0..1000 {
            wait_due(t, n, 0.0);
        }
        assert!(t.elapsed().as_millis() < 5);
    }

    #[test]
    fn sleep_guard_flags_wall_clock_jump() {
        let g = SleepGuard::start();
        assert!(g.check().is_ok() && g.gap_s().abs() < 0.5);
        let jumped = SleepGuard {
            unix0: g.unix0 - 5.0, // 假装墙钟多走了 5 秒
            mono0: g.mono0,
        };
        assert!(jumped.check().is_err());
    }

    #[test]
    fn jnum_and_pct() {
        assert_eq!(jnum(f64::NAN, 2), "null");
        assert_eq!(jnum(f64::NEG_INFINITY, 2), "null");
        assert_eq!(jnum(1.234, 2), "1.23");
        let v: Vec<f64> = (0..11).map(|i| i as f64).collect();
        assert_eq!((pct(&v, 0.0), pct(&v, 0.5), pct(&v, 1.0)), (0.0, 5.0, 10.0));
    }

    #[test]
    fn frames_for_rejects_empty_and_short() {
        let audio = vec![0.1f32; HOP * 3];
        assert!(frames_for(ModelId::Gtcrn, &audio, 0, &[]).is_err());
        assert!(frames_for(ModelId::Gtcrn, &audio[..HOP - 1], 5, &[]).is_err());
        assert_eq!(frames_for(ModelId::Gtcrn, &audio, 5, &[]).unwrap().len(), 5);
        assert!(
            frames_for(ModelId::Fem, &audio, 5, &[]).is_err(),
            "FE-M 没给形状要报错"
        );
    }

    #[test]
    fn parity_identical_passes_nan_fails_silence_passes() {
        let inputs: Vec<Vec<f32>> = (0..4).map(|i| vec![i as f32 + 1.0; 8]).collect();
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: false }),
                &inputs
            )
            .is_ok()
        );
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: true }),
                &inputs
            )
            .is_err()
        );
        let silent = vec![vec![0.0f32; 8]; 4];
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: false }),
                &silent
            )
            .is_ok(),
            "全静音且两边一致要通过"
        );
        assert!(
            parity(
                ModelId::Gtcrn,
                "t",
                Box::new(Fake { nan: false }),
                Box::new(Fake { nan: false }),
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn reset_check_bounds_and_nan() {
        let inputs: Vec<Vec<f32>> = (0..4).map(|i| vec![i as f32; 8]).collect();
        let (mut a, mut b) = (Fake { nan: false }, Fake { nan: false });
        assert!(reset_check(ModelId::Gtcrn, "t", &mut a, &mut b, &inputs, 2).is_ok());
        assert!(
            reset_check(ModelId::Gtcrn, "t", &mut a, &mut b, &inputs, 4).is_err(),
            "区间为空要报错"
        );
        assert!(reset_check(ModelId::Gtcrn, "t", &mut a, &mut b, &inputs, 0).is_err());
        let (mut c, mut d) = (Fake { nan: true }, Fake { nan: true });
        assert!(
            reset_check(ModelId::Gtcrn, "t", &mut c, &mut d, &inputs, 2).is_err(),
            "两边一样的 NaN 也不能过"
        );
    }

    #[test]
    fn gtcrn_shape_check() {
        let mut ok = vec![vec![1, BINS, 1, 2]];
        ok.extend(CACHE_SHAPES.iter().map(|s| s.to_vec()));
        assert!(check_gtcrn_shapes(&ok).is_ok());
        ok[1][0] = 3;
        assert!(check_gtcrn_shapes(&ok).is_err());
    }

    /// 端到端：手写版（含三个变体）对 ORT 逐值一致 + reset。需要仓外的权重与模型，CI 上没有，人工跑：
    ///   EXP010_WEIGHTS=<主目录>/_tmp/20261031-exp010/gtcrn_dns3 \
    ///   EXP010_GTCRN_ONNX=<主目录>/resources/gtcrn/stream/onnx_models/gtcrn_simple.onnx \
    ///   EXP010_WAV=<主目录>/resources/gtcrn/test_wavs/mix.wav \
    ///   cargo test --release -- --ignored --test-threads 1
    /// （--test-threads 1：native 变体靠全局开关切换，不能并行）
    #[test]
    #[ignore = "需要仓外的 GTCRN 权重、ONNX 和 wav，人工跑（见注释）"]
    fn native_variants_match_ort() {
        let v = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("要设 {k}"));
        let (w, onnx, wav) = (v("EXP010_WEIGHTS"), v("EXP010_GTCRN_ONNX"), v("EXP010_WAV"));
        let audio = read_wav(Path::new(&wav)).unwrap();
        let inputs = frames_for(ModelId::Gtcrn, &audio, audio.len() / HOP, &[]).unwrap();
        for k in ["native", "native-fast", "native-blas", "native-v2", "native-v2-fast", "native-v2-fast-pf", "native-v3", "native-v3-pf", "native-v3i", "native-v4", "native-v4i"] {
            let ort = open_net(ModelId::Gtcrn, "ort", Path::new(&onnx)).unwrap();
            let nat = open_net(ModelId::Gtcrn, k, Path::new(&w)).unwrap();
            parity(ModelId::Gtcrn, k, ort, nat, &inputs).unwrap_or_else(|e| panic!("{k}: {e}"));
            let (mut a, mut b) = (
                open_net(ModelId::Gtcrn, k, Path::new(&w)).unwrap(),
                open_net(ModelId::Gtcrn, k, Path::new(&w)).unwrap(),
            );
            reset_check(
                ModelId::Gtcrn,
                k,
                a.as_mut(),
                b.as_mut(),
                &inputs,
                inputs.len() / 2,
            )
            .unwrap_or_else(|e| panic!("{k} reset: {e}"));
        }
        // 手抄形状与 ONNX 一致（OrtGtcrn::open 里也会查，这里单独断言一次）
        check_gtcrn_shapes(&input_shapes(&ort_session(Path::new(&onnx)).unwrap()).unwrap())
            .unwrap();
    }
}
