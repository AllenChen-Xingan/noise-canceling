//! `vdn` 命令行（P1 离线原型）。
//!
//! ```text
//! vdn denoise in.wav out.wav [--model gtcrn|fastenhancer-m] [--floor-db -12] [--no-safety] …
//! vdn bench [--model …] --input mix.wav [--warmup 200] [--steps 6000] [--json out.json]
//! vdn devices                                         # 列设备（不开流）
//! vdn run --passthrough|--denoise [--seconds N] [--check] [--record out.wav] [--gain-db 12]   # 单进程实时：麦克风 → Pipeline → 扬声器（开麦、出声）
//! vdn chain-test [--mode passthrough|denoise] [--kill-denoiser-at 2]   # 无设备链路测试，按判据给退出码
//! ```
//! denoise / bench 只读写 16 kHz 单声道 WAV、不碰音频设备。

#![forbid(unsafe_code)]

mod args;
mod chain;
mod live;
mod offline_cmd;
mod stop;
mod verify;

use std::path::Path;

use clap::Parser;
use serde_json::json;
use vdn_models::{BoxError, ModelKind};

use args::{ChainArgs, ChainCheckArgs, ChainModeArg, Cli, Cmd, RunArgs, V7Args};
use offline_cmd::{bench, denoise};

/// 计数分配器：bench 里量「每 hop 在 Rust 侧分配几次」（ORT C++ 内部的分配绕过它，量不到）。
#[global_allocator]
pub(crate) static GLOBAL: &stats_alloc::StatsAlloc<std::alloc::System> =
    &stats_alloc::INSTRUMENTED_SYSTEM;

fn main() {
    let cli = Cli::parse();
    // flush-to-zero 暂不开：P1 要和 Python 逐值对拍，且 no_denormals 是 unsafe；留到 P3 实时线程入口（exp004 README「已知局限」）
    let res = match cli.cmd {
        Cmd::Denoise(a) => denoise(a),
        Cmd::Bench(a) => bench(a),
        Cmd::Devices => {
            live::list_devices();
            Ok(())
        }
        Cmd::Run(a) => match run(a) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
        Cmd::V7(a) => match v7(a) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
        Cmd::ChainCheck(a) => match chain_check(a) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
        Cmd::ChainTest(a) => match chain_test(a) {
            Ok(true) => Ok(()),
            Ok(false) => std::process::exit(1),
            Err(e) => Err(e),
        },
    };
    if let Err(e) = res {
        eprintln!("vdn: {e}");
        std::process::exit(1);
    }
}

/// 返回 Ok(false) = `--check` 判不过。
fn run(a: RunArgs) -> Result<bool, BoxError> {
    let o = live::RunOpts {
        seconds: a.seconds.or(a.check.then_some(live::CHECK_SECONDS)),
        block: a.block,
        lead: a.lead,
        input_device: a.input_device,
        output_device: a.output_device,
        gain_db: a.gain_db,
        record: a.record,
    };
    let report = if a.mode.passthrough {
        // 单进程：cpal 回调块长不固定，用任意块长的 Pipeline::new
        live::run(vdn_denoised::identity_pipeline(None)?, &o)?
    } else {
        let kind = ModelKind::from(a.m.model);
        live::run(
            vdn_denoised::model_pipeline(kind, a.m.model_path.as_deref(), None)?,
            &o,
        )?
    };
    if !a.check {
        return Ok(true);
    }
    let v = live::judge(&report.input, &report.output, a.mode.passthrough, a.gain_db);
    eprintln!("vdn run --check: {}", v.describe());
    if v.passed() {
        eprintln!(
            "  机器部分只判电平；听不听得到、断不断续、有没有咔哒声仍由人耳判（有 --record 就回放那个文件）"
        );
    }
    Ok(v.passed())
}

/// 返回 Ok(true) = 全部判据过。
fn chain_test(a: ChainArgs) -> Result<bool, BoxError> {
    let o = chain::ChainOpts {
        mode: match a.mode {
            ChainModeArg::Passthrough => chain::ChainMode::Passthrough,
            ChainModeArg::Denoise => chain::ChainMode::Denoise,
        },
        seconds: a.seconds,
        block: a.block,
        lead: a.lead,
        denoise_block: a.denoise_block,
        kill_at: a.kill_denoiser_at,
        bus_name: a
            .bus
            // 固定默认名：上次被 Ctrl-C 打断留下的残留，下次拿到锁时会清掉；同时跑两个会报「正被占用」
            .unwrap_or_else(|| "/vdn-chaintest".to_owned()),
        heartbeat_timeout: a.heartbeat_ms as usize * 48,
        rt: !a.no_rt,
        stress: a.stress,
        warmup_blocks: a.warmup_blocks,
        hal_offset: a.hal_offset.unwrap_or(a.block * 3 / 2),
        switch_at: a.switch_mode_at,
    };
    vdn_control::validate_bus_name(&o.bus_name)?;
    let r = match o.mode {
        chain::ChainMode::Passthrough => {
            chain::run(vdn_denoised::identity_pipeline(Some(o.denoise_block))?, &o)?
        }
        chain::ChainMode::Denoise => {
            let kind = ModelKind::from(a.m.model);
            chain::run(
                vdn_denoised::model_pipeline(
                    kind,
                    a.m.model_path.as_deref(),
                    Some(o.denoise_block),
                )?,
                &o,
            )?
        }
    };
    if let Some(prefix) = &a.dump {
        let p = prefix.display().to_string();
        write_wav48(Path::new(&format!("{p}-in.wav")), &r.x)?;
        write_wav48(Path::new(&format!("{p}-out.wav")), &r.y)?;
    }
    println!(
        "chain-test：模式 {} · {} s · IO 块长 {} · 降噪块长 {} · 心跳超时 {} ms{}",
        match o.mode {
            chain::ChainMode::Passthrough => "直通",
            chain::ChainMode::Denoise => "降噪",
        },
        o.seconds,
        o.block,
        o.denoise_block,
        a.heartbeat_ms,
        o.kill_at
            .map_or(String::new(), |k| format!(" · 第 {k} s 让降噪线程退出"))
    );
    println!("{}", chain::latency_line(&r));
    println!("{}", chain::counters_line(&r));
    println!("{}", chain::timing_line(&r));
    let mut verdicts = Vec::new();
    match (o.mode, o.kill_at) {
        (chain::ChainMode::Passthrough, None) => verdicts.push(chain::judge_v1(&r)),
        (_, Some(_)) => verdicts.push(chain::judge_v2(&r)),
        (chain::ChainMode::Denoise, None) => verdicts.push(chain::judge_v3(&r)),
    }
    let mut ok = true;
    for v in &verdicts {
        println!("{v}");
        ok &= v.pass;
    }
    Ok(ok)
}

fn write_wav48(path: &Path, y: &[f32]) -> Result<(), BoxError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: vdn_types::IO_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for &v in y {
        w.write_sample(v)?;
    }
    w.finalize()?;
    Ok(())
}

fn chain_check(a: ChainCheckArgs) -> Result<bool, BoxError> {
    let mut r = hound::WavReader::open(&a.wav)?;
    let spec = r.spec();
    if spec.channels != 1
        || spec.sample_rate != vdn_types::IO_RATE
        || spec.sample_format != hound::SampleFormat::Float
    {
        return Err(format!("{}：只收 48 kHz 单声道 32 位浮点", a.wav.display()).into());
    }
    let y = r.samples::<f32>().collect::<Result<Vec<_>, _>>()?;
    let v = chain::judge_file(&y, a.l_total, (a.from * f64::from(vdn_types::IO_RATE)) as usize);
    println!("{} · 文件 {}（{} 样本）", v, a.wav.display(), y.len());
    Ok(v.pass)
}

/// V7 线（docs/plan/010 延迟修复，latency-fix.md）
const V7_LEAD_MAX: usize = 192;
const V7_LTOTAL_MAX: usize = 36 * 48;

/// 返回 Ok(true) = 判据过。
fn v7(a: V7Args) -> Result<bool, BoxError> {
    let leads: Vec<usize> = match &a.leads {
        Some(s) => s
            .split(',')
            .map(|t| t.trim().parse::<usize>())
            .collect::<Result<_, _>>()?,
        None => vec![vdn_control::Config::DEFAULT_LEAD],
    };
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    let stress = a.stress.unwrap_or(cores.saturating_sub(3).max(1));
    let kind = ModelKind::from(a.m.model);
    let groups: Vec<(&str, usize)> = if a.idle_only {
        vec![("空闲", 0)]
    } else {
        vec![("空闲", 0), ("压力", stress)]
    };
    println!(
        "V7：{kind:?} · {} 次 × {} s · IO 块 {} · 降噪块 {} · 核数 {cores} · 压力组忙线程 {stress} · release 构建：{}",
        a.runs,
        a.seconds,
        a.block,
        a.denoise_block,
        if cfg!(debug_assertions) {
            "否（debug，数字不作数）"
        } else {
            "是"
        }
    );
    let mut rows = Vec::new();
    let mut clean_leads = Vec::new();
    for &lead in &leads {
        let mut all_clean = true;
        let mut l_total = 0;
        for &(gname, n_stress) in &groups {
            for run in 0..a.runs {
                let mut o = chain::ChainOpts::new("/vdn-v7");
                o.mode = chain::ChainMode::Denoise;
                o.seconds = a.seconds;
                o.block = a.block;
                o.denoise_block = a.denoise_block;
                o.lead = lead;
                o.stress = n_stress;
                o.hal_offset = a.block * 3 / 2;
                let p = vdn_denoised::model_pipeline(
                    kind,
                    a.m.model_path.as_deref(),
                    Some(a.denoise_block),
                )?;
                let r = chain::run(p, &o)?;
                l_total = r.l_total;
                let f = &r.fwd;
                let clean =
                    f.proc_underrun == 0 && r.hal_underrun_samples == 0 && f.stale_switches == 0;
                all_clean &= clean;
                println!(
                    "  lead {lead:>4} · {gname} #{:>2}：处理后欠载 {:>5} · 虚拟麦欠载 {:>5} · 硬切 {:>3} · 切原声 {} · 单块最长 {:.3} ms · 超一块时长 {:>3} 块 · {}",
                    run + 1,
                    f.proc_underrun,
                    r.hal_underrun_samples,
                    f.cuts,
                    f.stale_switches,
                    r.worker.max_process_ns as f64 / 1e6,
                    r.worker.over_budget,
                    if clean { "无欠载" } else { "有欠载" }
                );
                rows.push(json!({
                    "lead": lead, "group": gname, "stress_threads": n_stress, "run": run + 1,
                    "l_total": r.l_total, "proc_underrun": f.proc_underrun, "hal_underrun": r.hal_underrun_samples,
                    "cuts": f.cuts, "stale_switches": f.stale_switches, "slow_episodes": f.slow_episodes,
                    "max_process_ms": r.worker.max_process_ns as f64 / 1e6,
                    "mean_process_ms": r.worker.total_process_ns as f64 / 1e6 / r.worker.blocks.max(1) as f64,
                    "over_budget_blocks": r.worker.over_budget, "rt_failures": r.rt_failures,
                    "max_period_ms": r.worker.max_period_ns as f64 / 1e6, "over_period": r.worker.over_period,
                    "max_raw_backlog": r.worker.max_raw_backlog,
                    "max_raw_backlog_ms": r.worker.max_raw_backlog as f64 / 48.0,
                    "rt_checks": r.worker.rt_checks, "rt_downgrades": r.worker.rt_downgrades,
                    "warmup_p999_period_ms": r.warmup.p999_period_ns as f64 / 1e6,
                    "warmup_first_ms": r.warmup.first_ns as f64 / 1e6,
                }));
            }
        }
        println!(
            "lead {lead}（{:.2} ms）· L_total {l_total}（{:.2} ms）：{}",
            verify::ms48(lead),
            verify::ms48(l_total),
            if all_clean {
                "全部无欠载"
            } else {
                "有欠载"
            }
        );
        if all_clean {
            clean_leads.push((lead, l_total));
        }
    }
    let best = clean_leads.iter().min_by_key(|(l, _)| *l).copied();
    let pass = best.is_some_and(|(l, lt)| l <= V7_LEAD_MAX && lt <= V7_LTOTAL_MAX);
    println!(
        "V7 延迟判据：{} · 最小无欠载 lead {}（线 ≤ {V7_LEAD_MAX}，目标 144）· 对应 L_total {}（线 ≤ {V7_LTOTAL_MAX}）{}",
        if pass { "过" } else { "不过" },
        best.map_or(
            "无（所给的 lead 都有欠载）".to_owned(),
            |(l, _)| format!("{l}（{:.2} ms）", verify::ms48(l))
        ),
        best.map_or("—".to_owned(), |(_, lt)| format!(
            "{lt}（{:.2} ms）",
            verify::ms48(lt)
        )),
        if cfg!(debug_assertions) {
            " · ⚠️ debug 构建，不作数"
        } else {
            ""
        }
    );
    if let Some(j) = &a.json {
        let out = json!({
            "model": format!("{kind:?}"), "runs": a.runs, "seconds": a.seconds, "block": a.block,
            "denoise_block": a.denoise_block, "cores": cores, "stress_threads": stress,
            "release": !cfg!(debug_assertions), "leads": leads, "min_clean_lead": best.map(|b| b.0),
            "pass": pass, "rows": rows,
        });
        std::fs::write(j, serde_json::to_string_pretty(&out)?)?;
    }
    Ok(pass)
}
