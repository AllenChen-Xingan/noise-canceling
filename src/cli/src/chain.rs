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
use vdn_shm::{Bus, OwnedBus, RingId};

use crate::verify::{
    best_lag, longest_silence, ms48, nonfinite, onset, pearson_at_lag, rms, test_tone,
};

/// 正本在 vdn-control
const RATE: u32 = vdn_control::IO_RATE;
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
    /// 三个线程提 time-constraint 实时优先级（模拟真 IO 线程；不提时会被普通调度拖）
    pub rt: bool,
    /// 后台压力：再起这么多个忙线程（V7 压力组）
    pub stress: usize,
    /// 降噪线程开始服务前预热多少块
    pub warmup_blocks: usize,
    /// 虚拟麦克风比送音端晚多少样本读 Out（模拟插件 IO 相位；不在 L_total 里）
    pub hal_offset: usize,
    /// 第几秒经 rtrb 发「切到直通」命令（测运行中切换）
    pub switch_at: Option<f64>,
}

impl ChainOpts {
    /// 默认值（直通、5 s、IO 块与降噪块按 vdn-control 默认）
    pub fn new(bus_name: &str) -> Self {
        let block = vdn_control::Config::DEFAULT_BLOCK;
        Self {
            mode: ChainMode::Passthrough,
            seconds: 5.0,
            block,
            lead: vdn_control::Config::DEFAULT_LEAD,
            denoise_block: vdn_control::Config::DEFAULT_DENOISE_BLOCK,
            kill_at: None,
            bus_name: bus_name.to_owned(),
            heartbeat_timeout: 4800,
            rt: true,
            stress: 0,
            warmup_blocks: 64,
            hal_offset: block * 3 / 2,
            switch_at: None,
        }
    }
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
    pub worker: vdn_denoised::WorkerStats,
    pub warmup: vdn_denoised::Warmup,
    pub hal_offset: usize,
    /// 实时优先级提升失败的线程数
    pub rt_failures: u32,
}

/// Out 总线名：降噪总线名 + `-out`（如 /vdn-chaintest → /vdn-chaintest-out）
pub fn out_bus_name(bus: &str) -> String {
    format!("{bus}-out")
}

/// 当前线程提 time-constraint 优先级；失败返回 false（照常跑，计数）。
fn promote(block: usize, load: f64) -> bool {
    let p = vdn_rt_thread::RtParams::for_block(block as u32, RATE, load);
    vdn_rt_thread::promote_current(p).is_ok()
}

/// 任何返回路径（含某个线程 panic、`?` 提前返回）都让降噪线程停下。
struct StopOnDrop(Arc<AtomicBool>);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
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
    let mut worker = Worker::new(pipeline, o.denoise_block).skip_backlog_over_lead(o.lead);
    let warmup = worker.warm_up(o.warmup_blocks);
    let l_pipe = worker.processor().latency_samples();
    let l_total = l_pipe + o.lead;

    // 两条总线，和两个真进程时一样：
    // - 降噪总线（Raw / Processed / 心跳）：降噪方建；收音方 open
    // - Out 总线：收音方自己建；虚拟麦克风方 open。降噪进程重启不影响它
    // 都用 OwnedBus：拿到名字的锁才建（不删别人在用的同名总线），任何返回路径上删名字一次
    let den_cfg = vdn_denoised::denoise_bus_config(&worker, o.lead, RATE as usize)?;
    let den_bus = Arc::new(
        OwnedBus::create(&o.bus_name, den_cfg)
            .map_err(|e| format!("建降噪总线 {}：{e}", o.bus_name))?,
    );
    let out_name = out_bus_name(&o.bus_name);
    let out_bus = Arc::new(
        OwnedBus::create(&out_name, vdn_capture::shm::out_bus_config(RATE))
            .map_err(|e| format!("建 Out 总线 {out_name}：{e}"))?,
    );
    let cap_bus = Arc::new(Bus::open(&o.bus_name).map_err(|e| format!("收音端打开降噪总线：{e}"))?);
    let hal_bus =
        Arc::new(Bus::open(&out_name).map_err(|e| format!("虚拟麦端打开 Out 总线：{e}"))?);
    // 交叉校验：收音端从总线头读回的 lead / 降噪块长 = 降噪端的设置（正本是 o.lead 与 Worker）
    let back = cap_bus.config();
    if back.lead as usize != o.lead || back.denoise_block as usize != o.denoise_block {
        return Err(format!(
            "总线头 lead {} / 降噪块长 {} 和设置 {} / {} 不一致",
            back.lead, back.denoise_block, o.lead, o.denoise_block
        ));
    }
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
    let _stop_guard = StopOnDrop(stop.clone());
    let cap_pos = Arc::new(AtomicU64::new(0));
    let killed_at = Arc::new(AtomicU64::new(u64::MAX));
    let rt_failures = Arc::new(AtomicU64::new(0));
    let start = Instant::now() + Duration::from_millis(20);
    let block_dur = Duration::from_secs_f64(block as f64 / f64::from(RATE));
    let hal_delay = Duration::from_secs_f64(o.hal_offset as f64 / f64::from(RATE));

    // 后台压力：忙线程（不提优先级）
    let stressors: Vec<_> = (0..o.stress)
        .map(|_| {
            let stop = stop.clone();
            thread::spawn(move || {
                let mut v = 1u64;
                while !stop.load(Ordering::Relaxed) {
                    for _ in 0..10_000 {
                        v = std::hint::black_box(
                            v.wrapping_mul(6364136223846793005).wrapping_add(1),
                        );
                    }
                }
                v
            })
        })
        .collect();

    // 控制命令（运行中切模式）：本线程 → rtrb → 降噪线程
    let (mut cmd_tx, mut cmd_rx) = vdn_denoised::command_channel(8);

    // 降噪
    let den = {
        let (bus, stop, cap_pos, killed_at) = (
            den_bus.clone(),
            stop.clone(),
            cap_pos.clone(),
            killed_at.clone(),
        );
        let kill_samples = o.kill_at.map(|s| (s * f64::from(RATE)) as u64);
        let (rt, dblock, rtf) = (o.rt, o.denoise_block, rt_failures.clone());
        thread::spawn(move || {
            if rt && !promote(dblock, 0.75) {
                rtf.fetch_add(1, Ordering::Relaxed);
            }
            let mut consumed = 0u64;
            while !stop.load(Ordering::Relaxed) {
                if kill_samples.is_some_and(|k| consumed >= k) {
                    killed_at.store(cap_pos.load(Ordering::Acquire), Ordering::Release);
                    break;
                }
                worker.apply_commands(&mut cmd_rx);
                // 等收音端推够一块再醒（收音端每推一块 notify 一次）
                let _ = bus.wait(RingId::Raw, dblock, Duration::from_millis(20));
                while worker.step(&bus) == Step::Block {
                    consumed += worker.block() as u64;
                    if kill_samples.is_some_and(|k| consumed >= k) {
                        break;
                    }
                }
            }
            worker.stats()
        })
    };

    // 收音
    let cap = {
        let (bus, out, cap_pos) = (cap_bus.clone(), out_bus.clone(), cap_pos.clone());
        let mut fcfg = ForwarderConfig::new(l_total, block);
        fcfg.timeout = o.heartbeat_timeout;
        let mut fwd = Forwarder::new(fcfg);
        let x = test_tone(RATE, total, 0);
        let (rt, rtf) = (o.rt, rt_failures.clone());
        thread::spawn(move || {
            if rt && !promote(block, 0.25) {
                rtf.fetch_add(1, Ordering::Relaxed);
            }
            for (i, xb) in x.chunks(block).enumerate() {
                sleep_until(start + block_dur * i as u32);
                vdn_capture::shm::forward(&mut fwd, Some(&bus), &out, xb);
                cap_pos.store(fwd.stats().samples, Ordering::Release);
            }
            (x, fwd.stats())
        })
    };

    // 虚拟麦克风：晚 hal_offset 个样本开始读（同一时钟，纠漂移不该动手；动了说明调度抖动大）
    let hal_target = o.hal_offset;
    let hal = {
        let bus = hal_bus.clone();
        let (rt, rtf) = (o.rt, rt_failures.clone());
        thread::spawn(move || {
            if rt && !promote(block, 0.25) {
                rtf.fetch_add(1, Ordering::Relaxed);
            }
            let mut src =
                HalSource::new(vdn_capture::shm::OutSource::new(bus).with_drift(hal_target, block));
            let mut y = vec![0.0f32; total];
            let (mut ub, mut us) = (0u64, 0u64);
            for (i, yb) in y.chunks_mut(block).enumerate() {
                sleep_until(start + block_dur * i as u32 + hal_delay);
                if let RenderStatus::Underrun { got } = src.render(yb) {
                    ub += 1;
                    us += (yb.len() - got) as u64;
                }
            }
            (y, ub, us)
        })
    };

    if let Some(t) = o.switch_at {
        sleep_until(start + Duration::from_secs_f64(t));
        let _ = cmd_tx.push(vdn_control::Cmd::SetMode(vdn_control::Mode::Passthrough));
    }
    let (x, fwd) = cap.join().map_err(|_| "收音线程 panic")?;
    let (y, ub, us) = hal.join().map_err(|_| "虚拟麦线程 panic")?;
    stop.store(true, Ordering::Relaxed);
    let worker_stats = den.join().map_err(|_| "降噪线程 panic")?;
    for s in stressors {
        let _ = s.join();
    }
    let k = killed_at.load(Ordering::Acquire);
    den_bus
        .unlink()
        .map_err(|e| format!("unlink 降噪总线：{e}"))?;
    out_bus
        .unlink()
        .map_err(|e| format!("unlink Out 总线：{e}"))?;
    drop(den_bus);
    drop(out_bus);
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
        worker: worker_stats,
        warmup,
        hal_offset: o.hal_offset,
        rt_failures: u32::try_from(rt_failures.load(Ordering::Relaxed)).unwrap_or(u32::MAX),
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
        "计数：虚拟麦欠载 {} 块 / {} 样本 · 处理后欠载 {} 样本 · 硬切 {} · Raw 溢出 {} · Out 溢出 {} · 丢弃 {} · 重新对账 {} · 切原声 {} 次 · 降噪太慢 {} 次 · 跳过积压 {} 样本（送原声窗口 {} 次）· 降噪块数 {}",
        r.hal_underrun_blocks,
        r.hal_underrun_samples,
        f.proc_underrun,
        f.cuts,
        f.raw_overflow,
        f.out_overflow,
        f.discarded,
        f.resyncs,
        f.stale_switches,
        f.slow_episodes,
        r.worker.raw_skipped,
        f.gap_holds,
        r.worker.blocks
    )
}

/// 耗时一行（降噪线程）。
pub fn timing_line(r: &ChainRun) -> String {
    let w = &r.worker;
    let block_ms = ms48(r.denoise_block);
    format!(
        "耗时：预热 {} 块（第一块 {:.3} ms，之后最长 {:.3} ms）· 服务期单块最长 {:.3} ms、平均 {:.4} ms、超一块时长（{:.3} ms）{} 块 · 虚拟麦读相位 {} 样本（{:.2} ms，不在 L_total 里）· 实时提升失败 {} 个线程",
        r.warmup.blocks,
        r.warmup.first_ns as f64 / 1e6,
        r.warmup.max_after_first_ns as f64 / 1e6,
        w.max_process_ns as f64 / 1e6,
        w.total_process_ns as f64 / 1e6 / w.blocks.max(1) as f64,
        block_ms,
        w.over_budget,
        r.hal_offset,
        ms48(r.hal_offset),
        r.rt_failures
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
pub fn judge_file(y: &[f32], l_total: usize, from: usize) -> Verdict {
    let x = test_tone(RATE, y.len(), 0);
    let skip = SKIP.max(from);
    let corr = pearson_at_lag(&x, y, l_total, skip).unwrap_or(0.0);
    let found = if from == 0 {
        onset(y, 0.05).and_then(|on| {
            best_lag(&x, y, on.saturating_sub(600), on + 600, SKIP, 9600).map(|(lag, _)| lag)
        })
    } else {
        best_lag(
            &x,
            y,
            l_total.saturating_sub(600),
            l_total + 600,
            skip,
            9600,
        )
        .map(|(lag, _)| lag)
    };
    let (_, sil) = longest_silence(y, SILENCE, l_total + skip);
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
        let mut o = ChainOpts::new(&format!("/vdn-t{tag}-{}", std::process::id()));
        o.seconds = seconds;
        o.kill_at = kill_at;
        o.block = 480;
        o.hal_offset = 720;
        o.warmup_blocks = 8;
        o.rt = false;
        o
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
        assert_eq!(r.l_total, 768 + vdn_control::Config::DEFAULT_LEAD);
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
        // 两条总线（降噪 + Out）都删了
        for n in [o.bus_name.clone(), out_bus_name(&o.bus_name)] {
            assert!(matches!(Bus::open(&n), Err(vdn_shm::ShmError::NotFound)), "{n} 还在");
        }
    }

    /// OwnedBus 接入：同名降噪总线正被别人建着时，chain-test 报错退出，不删别人的总线
    #[test]
    fn busy_bus_name_is_refused_without_deleting() {
        let o = opts(0.3, None, "busy");
        let holder = vdn_shm::OwnedBus::create(
            &o.bus_name,
            vdn_capture::shm::out_bus_config(RATE),
        )
        .unwrap();
        let e = run(vdn_denoised::identity_pipeline(Some(96)).unwrap(), &o).err().unwrap();
        assert!(e.contains("建降噪总线"), "{e}");
        assert!(Bus::open(&o.bus_name).is_ok(), "别人的总线被删了");
        drop(holder);
    }

    /// 运行中切模式（rtrb 控制通道）走完整链路：命令执行了，链路照常对齐、不欠载
    #[test]
    fn mode_switch_mid_run() {
        let mut o = opts(1.2, None, "sw");
        o.switch_at = Some(0.5);
        let r = run(vdn_denoised::identity_pipeline(Some(96)).unwrap(), &o).unwrap();
        assert_eq!(r.worker.commands, 1);
        assert_eq!(r.fwd.proc_underrun, 0);
        let v = judge_v1(&r);
        assert!(v.pass, "{v}");
    }

    #[test]
    fn judge_file_on_synthetic() {
        let x = test_tone(RATE, 48_000, 0);
        let mut y = vec![0.0; 2303];
        y.extend_from_slice(&x[..48_000 - 2303]);
        assert!(judge_file(&y, 2303, 0).pass);
        assert!(!judge_file(&y, 2303 + 300, 0).pass);
    }
}
