//! `vdn devices` · `vdn run`（单进程开发模式：麦克风 → Pipeline → 扬声器）。
//!
//! `run` 用进程内总线（`Bus::heap`）把三方串起来，走的代码和两个进程时完全相同：
//! 输入回调 = `Forwarder`，降噪线程 = `Worker`，输出回调 = `HalSource` 读 Out。
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)] // NRT: 编排；实时部分在各 crate 内

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use vdn_capture::shm::{OutSource, forward_same_bus};
use vdn_capture::{Forwarder, ForwarderConfig};
use vdn_denoised::{Processor, Step, Worker};
use vdn_hal::HalSource;
use vdn_io_macos::{Direction, IO_RATE, InputStream, OutputStream, devices};
use vdn_shm::{Bus, RingId};

pub fn list_devices() {
    let all = devices();
    if all.is_empty() {
        println!("（没找到音频设备）");
        return;
    }
    for d in all {
        println!(
            "{} {}{} · 默认 {} Hz · {} 声道 · 支持 48 kHz：{}",
            match d.direction {
                Direction::Input => "输入",
                Direction::Output => "输出",
            },
            d.name,
            if d.is_default {
                "（系统默认）"
            } else {
                ""
            },
            d.default_rate,
            d.channels,
            if d.supports_io_rate { "是" } else { "否" }
        );
    }
}

pub struct RunOpts {
    pub seconds: Option<f64>,
    pub block: usize,
    pub lead: usize,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
}

pub fn run<P: Processor + Send + 'static>(pipeline: P, o: &RunOpts) -> Result<(), String> {
    // 块长 / 环余量走 vdn-control 的同一套范围检查
    let cfg = vdn_control::Config {
        block: o.block,
        lead_frames: o.lead,
        ..vdn_control::Config::default()
    };
    cfg.validate().map_err(|e| e.to_string())?;
    let mut worker = Worker::new(pipeline, o.block);
    let warmup = worker.warm_up(64);
    let rtp = vdn_denoised::denoise_rt_params(&worker, &warmup);
    let l_pipe = worker.processor().latency_samples();
    let l_total = worker.l_total(o.lead);
    let bus = Arc::new(
        Bus::heap(vdn_denoised::denoise_bus_config(
            &worker,
            o.lead,
            IO_RATE as usize,
        )?)
        .map_err(|e| e.to_string())?,
    );
    if worker.prime(&bus, o.lead) != o.lead {
        return Err("Processed 环放不下 lead 个零".into());
    }

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed))
            .map_err(|e| format!("装 Ctrl-C 处理：{e}"))?;
    }

    let den = {
        let (bus, stop) = (bus.clone(), stop.clone());
        let block = o.block;
        thread::spawn(move || {
            if let Err(e) = vdn_rt_promote(block) {
                eprintln!("vdn run: 降噪线程实时优先级提升失败（照常运行）：{e}");
            }
            while !stop.load(Ordering::Relaxed) {
                let _ = bus.wait(RingId::Raw, block, Duration::from_millis(20));
                while worker.step(&bus) == Step::Block {}
            }
            worker.stats()
        })
    };

    let mut fwd = Forwarder::new(ForwarderConfig::new(l_total, o.block));
    let input = {
        let bus = bus.clone();
        InputStream::open(
            o.input_device.as_deref(),
            IO_RATE,
            o.block,
            move |m: &[f32]| {
                forward_same_bus(&mut fwd, &bus, m);
            },
        )
        .map_err(|e| format!("开麦克风：{e}"))?
    };
    let output = {
        let mut src = HalSource::new(OutSource::new(bus.clone()));
        OutputStream::open(
            o.output_device.as_deref(),
            IO_RATE,
            o.block,
            move |out: &mut [f32]| {
                let _ = src.render(out);
            },
        )
        .map_err(|e| format!("开扬声器：{e}"))?
    };
    eprintln!(
        "vdn run: 输入 {:?} → 输出 {:?} · L_total {} 样本（{:.2} ms）= Pipeline {} + 环余量 {} · Ctrl-C 停",
        input.info(),
        output.info(),
        l_total,
        l_total as f64 / 48.0,
        l_pipe,
        o.lead
    );

    let until = o
        .seconds
        .map(|s| Instant::now() + Duration::from_secs_f64(s));
    while !stop.load(Ordering::Relaxed) && until.is_none_or(|u| Instant::now() < u) {
        thread::sleep(Duration::from_millis(50));
        if input.health().needs_reopen() || output.health().needs_reopen() {
            eprintln!("vdn run: 设备流停了（拔掉或切换了设备？），退出");
            break;
        }
    }
    stop.store(true, Ordering::Relaxed);
    drop(input);
    drop(output);
    let ws = den.join().map_err(|_| "降噪线程 panic")?;
    eprintln!(
        "vdn run: 结束 · 降噪 {} 块 · Raw 溢出 {} · Out 溢出 {} · 输出欠载 {} 样本",
        ws.blocks,
        bus.ring(RingId::Raw).overflows(),
        bus.ring(RingId::Out).overflows(),
        bus.ring(RingId::Out).underruns()
    );
    Ok(())
}

fn vdn_rt_promote(block: usize) -> Result<(), String> {
    let p = vdn_rt_thread::RtParams::for_block(block as u32, IO_RATE, 0.5);
    vdn_rt_thread::promote_current(p).map_err(|e| format!("{e:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// V5（人判）：真麦克风 → 直通 → 扬声器 5 秒。会开麦、出声、首次弹麦克风权限框，所以默认不跑。
    ///
    /// 人工步骤：
    /// 1. 戴耳机（外放会啸叫）
    /// 2. `cargo test -p vdn-cli -- --ignored v5_live_passthrough --nocapture`
    ///    （或 `./target/debug/vdn run --passthrough --seconds 5`）
    /// 3. 第一次会弹「终端想使用麦克风」，点允许后重跑
    /// 4. 5 秒内说话
    ///
    /// 算过：耳机里听到自己的声音、晚一点点（链路内 L_total ≈ 22 ms，另加设备缓冲）、不断续、没有咔哒声。
    /// 机器只判「跑完不报错」；听感由人判，结果回报大总管
    #[test]
    #[ignore = "开麦克风、扬声器出声：人工跑，戴耳机（步骤见文档注释）"]
    fn v5_live_passthrough() {
        let o = RunOpts {
            seconds: Some(5.0),
            block: vdn_control::Config::DEFAULT_BLOCK,
            lead: vdn_control::Config::DEFAULT_LEAD,
            input_device: None,
            output_device: None,
        };
        run(vdn_denoised::identity_pipeline(None).unwrap(), &o).unwrap();
    }
}
