//! 锁步模拟：`run` 的单线程确定版，只给单元测试用。

use vdn_capture::{Forwarder, ForwarderConfig};
use vdn_denoised::{Processor, Step};
use vdn_hal::{HalSource, RenderStatus};

use super::{ChainOpts, ChainRun, RATE, Setup, setup};
use crate::verify::test_tone;

/// 锁步模拟（单元测试用）：和 `run` **同一套**总线 · Forwarder · Worker · HalSource · 命令通道，
/// 但单线程逐块推进：送音推一块 → 降噪把能处理的都处理完 → 虚拟麦（晚 `hal_offset` 起读）取一块。
/// 不 sleep、不靠真实时钟，所以结果确定，和机器负载、并行跑的别的测试无关。
/// 测不到的：线程调度、唤醒延迟、处理耗时——那些归 `vdn chain-test`（真实节拍）和 V7（release、10 次 × 10 s）
pub fn run_lockstep<P: Processor>(pipeline: P, o: &ChainOpts) -> Result<ChainRun, String> {
    let Setup {
        mut worker,
        warmup,
        l_pipe,
        l_total,
        total,
        den_bus,
        out_bus,
        cap_bus,
        hal_bus,
    } = setup(pipeline, o)?;
    let block = o.block;
    let mut fcfg = ForwarderConfig::new(l_total, block);
    fcfg.timeout = o.heartbeat_timeout;
    let mut fwd = Forwarder::new(fcfg);
    let mut src = HalSource::new(
        vdn_capture::shm::OutSource::new(hal_bus.clone()).with_drift(o.hal_offset, block),
    );
    let (mut cmd_tx, mut cmd_rx) = vdn_denoised::command_channel(8);
    let x = test_tone(RATE, total, 0);
    let mut y = vec![0.0f32; total];
    let kill_samples = o.kill_at.map(|s| (s * f64::from(RATE)) as u64);
    let switch_samples = o.switch_at.map(|s| (s * f64::from(RATE)) as u64);
    // 读方晚几块开始读（真实跑法里是晚 hal_offset 个样本的相位）
    let hal_start = o.hal_offset.div_ceil(block);
    let (mut consumed, mut alive, mut killed_at, mut switched) = (0u64, true, None, false);
    let (mut ub, mut us) = (0u64, 0u64);
    let mut xs = x.chunks(block);
    let mut ys = y.chunks_mut(block);
    for i in 0.. {
        let t = (i * block) as u64;
        let xb = xs.next();
        if xb.is_none() && i >= hal_start + total / block {
            break;
        }
        if let Some(xb) = xb {
            if let Some(s) = switch_samples
                && !switched
                && t >= s
            {
                let _ = cmd_tx.push(vdn_control::Cmd::SetMode(vdn_control::Mode::Passthrough));
                switched = true;
            }
            vdn_capture::shm::forward(&mut fwd, Some(&cap_bus), &out_bus, xb);
            if alive {
                worker.apply_commands(&mut cmd_rx);
                loop {
                    if kill_samples.is_some_and(|k| consumed >= k) {
                        alive = false;
                        killed_at = Some(fwd.stats().samples);
                        break;
                    }
                    if worker.step(&den_bus) != Step::Block {
                        break;
                    }
                    consumed += worker.block() as u64;
                }
            }
        }
        if i >= hal_start
            && let Some(yb) = ys.next()
            && let RenderStatus::Underrun { got } = src.render(yb)
        {
            ub += 1;
            us += (yb.len() - got) as u64;
        }
    }
    drop(src);
    den_bus
        .unlink()
        .map_err(|e| format!("unlink 降噪总线：{e}"))?;
    out_bus
        .unlink()
        .map_err(|e| format!("unlink Out 总线：{e}"))?;
    Ok(ChainRun {
        x,
        y,
        l_total,
        l_pipe,
        lead: o.lead,
        denoise_block: o.denoise_block,
        fwd: fwd.stats(),
        hal_underrun_blocks: ub,
        hal_underrun_samples: us,
        killed_at,
        worker: worker.stats(),
        warmup,
        hal_offset: o.hal_offset,
        rt_failures: 0,
    })
}
