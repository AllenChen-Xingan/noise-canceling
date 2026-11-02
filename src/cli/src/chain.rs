//! `vdn chain-test`：无设备跑完整链路（docs/plan/010 第十一节 V1–V3）。
//!
//! 进程内三个线程扮三方，中间用**真 POSIX 共享内存**：
//! - 收音：按 48 kHz 实时节拍产生测试信号，`Forwarder` 推 Raw、取 Processed、推 Out（自己 `Bus::open` 一份映射）
//! - 降噪：`Worker` + `Pipeline`（建总线的一方）；`--kill-denoiser-at` 到点就退出（不再心跳、不再推）
//! - 虚拟麦克风：`HalSource` 按同样节拍读 Out（再 `Bus::open` 一份映射）
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)] // NRT: 测试编排；实时部分在各 crate 内

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use vdn_capture::{Forwarder, ForwarderConfig, ForwarderStats};
use vdn_denoised::{Processor, Step, Worker};
use vdn_hal::{HalSource, RenderStatus};
use vdn_shm::{Bus, BusConfig};

use crate::verify::{
    best_lag, longest_silence, ms48, nonfinite, onset, pearson_at_lag, rms, test_tone,
};

const RATE: u32 = 48_000;
/// 判相关前跳过的样本（淡入与重采样起振）
const SKIP: usize = 4800;
/// 静音阈值（−60 dBFS）
const SILENCE: f32 = 1e-3;
/// V2：切原声与最长静音的上限（150 ms）
const LIMIT_150MS: usize = 7200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainMode {
    Passthrough,
    Denoise,
}

pub struct ChainOpts {
    pub mode: ChainMode,
    pub seconds: f64,
    pub block: usize,
    pub lead: usize,
    /// 降噪线程每次处理的块长（定块长 Pipeline）
    pub denoise_block: usize,
    pub kill_at: Option<f64>,
    pub bus_name: String,
    pub heartbeat_timeout: usize,
}

/// 一次运行的原始结果。
pub struct ChainRun {
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub l_total: usize,
    pub l_pipe: usize,
    pub lead: usize,
    /// 降噪线程每次处理的块长（定块长 Pipeline）
    pub denoise_block: usize,
    pub fwd: ForwarderStats,
    pub hal_underrun_blocks: u64,
    pub hal_underrun_samples: u64,
    pub killed_at: Option<u64>,
    pub worker_blocks: u64,
}

/// 名字在退出（含出错）时一定 unlink。
struct Unlink(String);
impl Drop for Unlink {
    fn drop(&mut self) {
        let _ = Bus::unlink_name(&self.0);
    }
}

fn sleep_until(t: Instant) {
    let now = Instant::now();
    if t > now {
        thread::sleep(t - now);
    }
}

pub fn run<P: Processor + Send + 'static>(pipeline: P, o: &ChainOpts) -> Result<ChainRun, String> {
    let block = o.block;
    let total = ((o.seconds * f64::from(RATE)) as usize).div_ceil(block) * block;
    let mut worker = Worker::new(pipeline, o.denoise_block);
    let l_pipe = worker.processor().latency_samples();
    let l_total = l_pipe + o.lead;

    // 降噪方建总线
    let _ = Bus::unlink_name(&o.bus_name); // 上次异常退出的残留
    let cfg = BusConfig {
        rate: RATE,
        capacity_frames: RATE, // 1 s
        l_total: u32::try_from(l_total).map_err(|_| "L_total 太大")?,
    };
    let den_bus =
        Arc::new(Bus::create(&o.bus_name, cfg).map_err(|e| format!("建总线 {}：{e}", o.bus_name))?);
    let _guard = Unlink(o.bus_name.clone());
    // 收音方与虚拟麦克风方各自 open（走真跨进程那条路）
    let cap_bus = Arc::new(Bus::open(&o.bus_name).map_err(|e| format!("收音端打开总线：{e}"))?);
    let hal_bus = Arc::new(Bus::open(&o.bus_name).map_err(|e| format!("虚拟麦端打开总线：{e}"))?);
    if cap_bus.l_total() as usize != l_total {
        return Err(format!(
            "总线头 L_total = {}，应为 {l_total}",
            cap_bus.l_total()
        ));
    }

    if worker.prime(&den_bus, o.lead) != o.lead {
        return Err("Processed 环放不下 lead 个零".into());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let cap_pos = Arc::new(AtomicU64::new(0));
    let killed_at = Arc::new(AtomicU64::new(u64::MAX));
    let start = Instant::now() + Duration::from_millis(20);
    let block_dur = Duration::from_secs_f64(block as f64 / f64::from(RATE));

    // 降噪
    let den = {
        let (bus, stop, cap_pos, killed_at) = (
            den_bus.clone(),
            stop.clone(),
            cap_pos.clone(),
            killed_at.clone(),
        );
        let kill_samples = o.kill_at.map(|s| (s * f64::from(RATE)) as u64);
        thread::spawn(move || {
            let mut consumed = 0u64;
            while !stop.load(Ordering::Relaxed) {
                if kill_samples.is_some_and(|k| consumed >= k) {
                    killed_at.store(cap_pos.load(Ordering::Acquire), Ordering::Release);
                    break;
                }
                match worker.step(&bus) {
                    Step::Block => consumed += worker.block() as u64,
                    Step::Idle => thread::sleep(Duration::from_micros(500)),
                }
            }
            worker.stats().blocks
        })
    };

    // 收音
    let cap = {
        let (bus, cap_pos) = (cap_bus.clone(), cap_pos.clone());
        let mut fcfg = ForwarderConfig::new(l_total, block);
        fcfg.timeout = o.heartbeat_timeout;
        let mut fwd = Forwarder::new(fcfg);
        let x = test_tone(RATE, total, 0);
        thread::spawn(move || {
            for (i, xb) in x.chunks(block).enumerate() {
                sleep_until(start + block_dur * i as u32);
                vdn_capture::shm::forward(&mut fwd, &bus, xb);
                cap_pos.store(fwd.stats().samples, Ordering::Release);
            }
            (x, fwd.stats())
        })
    };

    // 虚拟麦克风：晚 1.5 块开始读
    let hal = {
        let bus = hal_bus.clone();
        thread::spawn(move || {
            let mut src = HalSource::new(vdn_capture::shm::OutSource::new(bus));
            let mut y = vec![0.0f32; total];
            let (mut ub, mut us) = (0u64, 0u64);
            for (i, yb) in y.chunks_mut(block).enumerate() {
                sleep_until(start + block_dur * i as u32 + block_dur * 3 / 2);
                if let RenderStatus::Underrun { got } = src.render(yb) {
                    ub += 1;
                    us += (yb.len() - got) as u64;
                }
            }
            (y, ub, us)
        })
    };

    let (x, fwd) = cap.join().map_err(|_| "收音线程 panic")?;
    let (y, ub, us) = hal.join().map_err(|_| "虚拟麦线程 panic")?;
    stop.store(true, Ordering::Relaxed);
    let worker_blocks = den.join().map_err(|_| "降噪线程 panic")?;
    let k = killed_at.load(Ordering::Acquire);
    den_bus.unlink().map_err(|e| format!("unlink：{e}"))?;
    Ok(ChainRun {
        x,
        y,
        l_total,
        l_pipe,
        lead: o.lead,
        denoise_block: o.denoise_block,
        fwd,
        hal_underrun_blocks: ub,
        hal_underrun_samples: us,
        killed_at: (k != u64::MAX).then_some(k),
        worker_blocks,
    })
}

/// 一条判据的结论。
pub struct Verdict {
    pub id: &'static str,
    pub what: &'static str,
    pub pass: bool,
    pub detail: String,
}

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}：{} · {}",
            self.id,
            self.what,
            if self.pass { "过" } else { "不过" },
            self.detail
        )
    }
}

/// 延迟分项一行。
pub fn latency_line(r: &ChainRun) -> String {
    format!(
        "L_total = {} 样本（{:.2} ms）= Pipeline {}（{:.2} ms，降噪块长 {}）+ 环余量 {}（{:.2} ms）",
        r.l_total,
        ms48(r.l_total),
        r.l_pipe,
        ms48(r.l_pipe),
        r.denoise_block,
        r.lead,
        ms48(r.lead)
    )
}

/// 计数一行。
pub fn counters_line(r: &ChainRun) -> String {
    let f = &r.fwd;
    format!(
        "计数：虚拟麦欠载 {} 块 / {} 样本 · 处理后欠载 {} 样本 · Raw 溢出 {} · Out 溢出 {} · 丢弃 {} · 重新对账 {} · 切原声 {} 次 · 降噪块数 {}",
        r.hal_underrun_blocks,
        r.hal_underrun_samples,
        f.proc_underrun,
        f.raw_overflow,
        f.out_overflow,
        f.discarded,
        f.resyncs,
        f.stale_switches,
        r.worker_blocks
    )
}

/// V1：输出与「输入延迟 L_total」相关 ≥ 0.999，虚拟麦欠载 0。
pub fn judge_v1(r: &ChainRun) -> Verdict {
    let corr = pearson_at_lag(&r.x, &r.y, r.l_total, SKIP).unwrap_or(0.0);
    let found = onset(&r.y, 0.05).and_then(|on| {
        best_lag(&r.x, &r.y, on.saturating_sub(600), on + 600, SKIP, 9600).map(|(lag, _)| lag)
    });
    let pass = corr >= 0.999 && r.hal_underrun_samples == 0;
    Verdict {
        id: "V1",
        what: "直通链路逐样本对齐",
        pass,
        detail: format!(
            "相关 {corr:.6}（线 ≥ 0.999）· 实测延迟 {}（应为 {}）· 虚拟麦欠载 {} 样本（线 = 0）",
            found.map_or("未找到".to_owned(), |l| l.to_string()),
            r.l_total,
            r.hal_underrun_samples
        ),
    }
}

/// V2：降噪死后 ≤ 150 ms 切原声；全程无 > 150 ms 静音。
pub fn judge_v2(r: &ChainRun) -> Verdict {
    let switch = match (r.killed_at, r.fwd.last_stale_at) {
        (Some(k), Some(s)) if s >= k => Some((s - k) as usize),
        _ => None,
    };
    let (sil_at, sil_len) = longest_silence(&r.y, SILENCE, r.l_total + SKIP);
    let pass = switch.is_some_and(|s| s <= LIMIT_150MS) && sil_len <= LIMIT_150MS;
    Verdict {
        id: "V2",
        what: "降噪进程死掉仍出声",
        pass,
        detail: format!(
            "切原声用时 {}（线 ≤ 150 ms）· 最长静音 {:.1} ms，起于 {:.1} ms（线 ≤ 150 ms）",
            switch.map_or("未切换".to_owned(), |s| format!("{:.1} ms", ms48(s))),
            ms48(sil_len),
            ms48(sil_at)
        ),
    }
}

/// V3：真模型链路出声：非静音、有限值、长度对齐。
pub fn judge_v3(r: &ChainRun) -> Verdict {
    let tail = r.y.get(r.l_total + SKIP..).unwrap_or(&[]);
    let level = rms(tail);
    let bad = nonfinite(&r.y);
    let pass = level > 0.01 && bad == 0 && r.y.len() == r.x.len();
    Verdict {
        id: "V3",
        what: "真模型链路出声",
        pass,
        detail: format!(
            "输出均方根 {level:.4}（线 > 0.01）· 非有限样本 {bad}（线 = 0）· 长度 {} / {}",
            r.y.len(),
            r.x.len()
        ),
    }
}

/// V4：文件里应是测试正弦延迟 `l_total`（同 V1 的相关判据；欠载看不到，由 vdn-capture 的计数报）。
pub fn judge_file(y: &[f32], l_total: usize) -> Verdict {
    let x = test_tone(RATE, y.len(), 0);
    let corr = pearson_at_lag(&x, y, l_total, SKIP).unwrap_or(0.0);
    let found = onset(y, 0.05).and_then(|on| {
        best_lag(&x, y, on.saturating_sub(600), on + 600, SKIP, 9600).map(|(lag, _)| lag)
    });
    let (_, sil) = longest_silence(y, SILENCE, l_total + SKIP);
    let pass = corr >= 0.999;
    Verdict {
        id: "V4",
        what: "两个真进程",
        pass,
        detail: format!(
            "相关 {corr:.6}（线 ≥ 0.999）· 实测延迟 {}（应为 {l_total}）· 最长静音 {:.1} ms",
            found.map_or("未找到".to_owned(), |l| l.to_string()),
            ms48(sil)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(seconds: f64, kill_at: Option<f64>, tag: &str) -> ChainOpts {
        ChainOpts {
            mode: ChainMode::Passthrough,
            seconds,
            block: 480,
            lead: 1440,
            denoise_block: 96,
            kill_at,
            bus_name: format!("/vdn-t{tag}-{}", std::process::id()),
            heartbeat_timeout: 4800,
        }
    }

    /// V1 的短版（1.2 s）：真共享内存、三线程
    #[test]
    fn v1_short() {
        let r = run(
            vdn_denoised::identity_pipeline(Some(96)).unwrap(),
            &opts(1.2, None, "v1"),
        )
        .unwrap();
        let v = judge_v1(&r);
        assert!(v.pass, "{v}");
        assert_eq!(r.l_total, 768 + 1440);
    }

    /// V2 的短版：0.5 s 处让降噪线程退出
    #[test]
    fn v2_short() {
        let r = run(
            vdn_denoised::identity_pipeline(Some(96)).unwrap(),
            &opts(1.2, Some(0.5), "v2"),
        )
        .unwrap();
        let v = judge_v2(&r);
        assert!(v.pass, "{v}");
        assert_eq!(r.fwd.stale_switches, 1);
    }

    /// 退出后名字已删：再 open 应找不到
    #[test]
    fn bus_unlinked_after_run() {
        let o = opts(0.3, None, "ul");
        run(vdn_denoised::identity_pipeline(Some(96)).unwrap(), &o).unwrap();
        assert!(matches!(
            Bus::open(&o.bus_name),
            Err(vdn_shm::ShmError::NotFound)
        ));
    }

    #[test]
    fn judge_file_on_synthetic() {
        let x = test_tone(RATE, 48_000, 0);
        let mut y = vec![0.0; 2303];
        y.extend_from_slice(&x[..48_000 - 2303]);
        assert!(judge_file(&y, 2303).pass);
        assert!(!judge_file(&y, 2303 + 300).pass);
    }
}
