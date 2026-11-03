//! `vdn devices` · `vdn run`（单进程开发模式：麦克风 → Pipeline → 扬声器）。
//!
//! `run` 用进程内总线（`Bus::heap`）把三方串起来，走的代码和两个进程时完全相同：
//! 输入回调 = `Forwarder`，降噪线程 = `Worker`，输出回调 = `HalSource` 读 Out。
//!
//! 不开设备的部分（[`prepare`] · [`input_callback`] · [`output_callback`] · [`denoise_loop`]）单独拆出来，
//! 单测用假输入 / 假输出把同一套回调串起来跑（`run_path_without_devices_*`）；`run` 只多了开设备这一步。
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)] // NRT: 编排；实时部分在下面带 #[deny] 的函数里与各 crate 内

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use vdn_capture::shm::{OutSource, forward_same_bus};
use vdn_capture::{Forwarder, ForwarderConfig};
use vdn_denoised::{Processor, Step, Worker, WorkerStats};
use vdn_hal::HalSource;
use vdn_io_macos::{
    Direction, IO_RATE, InputStream, Level, OutputStream, devices, fmt_dbfs,
};
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

/// `--gain-db` 的上限（dB）：只给 V5 试听用，放大小声的麦克风（人定 2026-11-02，大总管转）
pub const MAX_GAIN_DB: f64 = 20.0;
/// `--gain-db` 的下限（dB）
pub const MIN_GAIN_DB: f64 = -60.0;
/// `--check` 不给 `--seconds` 时跑多久（秒）
pub const CHECK_SECONDS: f64 = 5.0;
/// 录音交接队列长度：2 s（写文件线程每 10 ms 取一次，正常不会满）
const RECORD_QUEUE: usize = 2 * IO_RATE as usize;

pub struct RunOpts {
    pub seconds: Option<f64>,
    pub block: usize,
    pub lead: usize,
    pub input_device: Option<String>,
    pub output_device: Option<String>,
    /// 输出增益（dB，[`MIN_GAIN_DB`]..=[`MAX_GAIN_DB`]）；加在送进耳机之前，录音录的也是加过增益的
    pub gain_db: f64,
    /// 把送进耳机的那一路同时写进这个 WAV（48 kHz 单声道 32 位浮点）
    pub record: Option<PathBuf>,
}

/// 增益的范围检查，dB → 线性倍数。
pub fn gain_linear(db: f64) -> Result<f32, String> {
    if !db.is_finite() || !(MIN_GAIN_DB..=MAX_GAIN_DB).contains(&db) {
        return Err(format!(
            "--gain-db {db} 不在 {MIN_GAIN_DB}..={MAX_GAIN_DB} 内"
        ));
    }
    Ok(10f64.powf(db / 20.0) as f32)
}

/// 不开设备的部分：进程内总线 + 预热过、已推好 lead 个零的降噪 Worker。
pub struct Prepared<P> {
    pub bus: Arc<Bus>,
    pub worker: Worker<P>,
    pub rtp: vdn_rt_thread::RtParams,
    pub l_total: usize,
    pub l_pipe: usize,
}

/// 建总线、预热、算实时参数（正本 `vdn_denoised::denoise_rt_params`）、往 Processed 推 lead 个零。
pub fn prepare<P: Processor>(pipeline: P, block: usize, lead: usize) -> Result<Prepared<P>, String> {
    // 块长 / 环余量走 vdn-control 的同一套范围检查
    let cfg = vdn_control::Config {
        block,
        lead_frames: lead,
        ..vdn_control::Config::default()
    };
    cfg.validate().map_err(|e| e.to_string())?;
    let mut worker = Worker::new(pipeline, block);
    let warmup = worker.warm_up(64);
    let rtp = vdn_denoised::denoise_rt_params(&worker, &warmup);
    let l_pipe = worker.processor().latency_samples();
    let l_total = worker.l_total(lead);
    let bus = Arc::new(
        Bus::heap(vdn_denoised::denoise_bus_config(
            &worker,
            lead,
            IO_RATE as usize,
        )?)
        .map_err(|e| e.to_string())?,
    );
    if worker.prime(&bus, lead) != lead {
        return Err("Processed 环放不下 lead 个零".into());
    }
    Ok(Prepared {
        bus,
        worker,
        rtp,
        l_total,
        l_pipe,
    })
}

/// 输入回调（麦克风的实时线程）：原声 → Forwarder → Raw / Out。
pub fn input_callback(bus: Arc<Bus>, l_total: usize, block: usize) -> impl FnMut(&[f32]) + Send {
    let mut fwd = Forwarder::new(ForwarderConfig::new(l_total, block));
    move |m: &[f32]| forward_same_bus(&mut fwd, &bus, m)
}

/// 输出端的增益与录音分流（实时线程上用）。
pub struct OutputTap {
    gain: f32,
    rec: Option<rtrb::Producer<f32>>,
    /// 录音队列满了丢掉的样本数
    rec_dropped: Arc<AtomicU64>,
    /// 加增益后超过 ±1、被夹住的样本数
    clipped: Arc<AtomicU64>,
}

impl OutputTap {
    pub fn new(gain: f32, rec: Option<rtrb::Producer<f32>>) -> Self {
        Self {
            gain,
            rec,
            rec_dropped: Arc::new(AtomicU64::new(0)),
            clipped: Arc::new(AtomicU64::new(0)),
        }
    }
    pub fn rec_dropped(&self) -> Arc<AtomicU64> {
        self.rec_dropped.clone()
    }
    pub fn clipped(&self) -> Arc<AtomicU64> {
        self.clipped.clone()
    }
}

#[deny(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]
impl OutputTap {
    /// 加增益（夹在 ±1 内并计数），再把这块送进录音队列（满了丢、计数，不等）。不分配、不加锁。
    pub fn apply(&mut self, out: &mut [f32]) {
        if self.gain != 1.0 {
            let mut clipped = 0u64;
            for s in out.iter_mut() {
                let v = *s * self.gain;
                if v.abs() > 1.0 {
                    clipped += 1;
                }
                *s = v.clamp(-1.0, 1.0);
            }
            if clipped > 0 {
                self.clipped.fetch_add(clipped, Ordering::Relaxed);
            }
        }
        if let Some(p) = self.rec.as_mut() {
            let mut lost = 0u64;
            for &s in out.iter() {
                if p.push(s).is_err() {
                    lost += 1;
                }
            }
            if lost > 0 {
                self.rec_dropped.fetch_add(lost, Ordering::Relaxed);
            }
        }
    }
}

/// 输出回调（扬声器的实时线程）：读 Out → 增益 → 录音分流。拿到的块已清零（OutputPump 保证）。
pub fn output_callback(bus: Arc<Bus>, mut tap: OutputTap) -> impl FnMut(&mut [f32]) + Send {
    let mut src = HalSource::new(OutSource::new(bus));
    move |out: &mut [f32]| {
        let _ = src.render(out);
        tap.apply(out);
    }
}

/// 降噪线程的循环体：等 Raw 够一块就处理，直到 `stop`。
pub fn denoise_loop<P: Processor>(
    worker: &mut Worker<P>,
    bus: &Bus,
    block: usize,
    stop: &AtomicBool,
) -> WorkerStats {
    while !stop.load(Ordering::Relaxed) {
        let _ = bus.wait(RingId::Raw, block, Duration::from_millis(20));
        while worker.step(bus) == Step::Block {}
    }
    worker.stats()
}

/// 录音线程：从队列取样本写 WAV，`stop` 后把队列取空再收尾。返回写了多少样本。
fn spawn_recorder(
    path: &Path,
    mut rx: rtrb::Consumer<f32>,
    stop: Arc<AtomicBool>,
) -> Result<thread::JoinHandle<Result<u64, String>>, String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: IO_RATE,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)
        .map_err(|e| format!("建录音文件 {}：{e}", path.display()))?;
    Ok(thread::spawn(move || {
        let mut n = 0u64;
        loop {
            let done = stop.load(Ordering::Acquire);
            while let Ok(s) = rx.pop() {
                w.write_sample(s).map_err(|e| e.to_string())?;
                n += 1;
            }
            if done {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        w.finalize().map_err(|e| e.to_string())?;
        Ok(n)
    }))
}

/// 一次 `vdn run` 的结果（结束时打印，`--check` 据此判定）。
#[derive(Clone, Copy, Debug, Default)]
pub struct RunReport {
    /// 麦克风进来的电平（取第 1 声道之后）
    pub input: Level,
    /// 送进耳机的电平（加增益之后、复制到各声道之前）
    pub output: Level,
    pub worker: WorkerStats,
    pub raw_overflow: u64,
    pub out_overflow: u64,
    pub out_underrun: u64,
    pub clipped: u64,
    pub rec_written: Option<u64>,
    pub rec_dropped: u64,
}

pub fn run<P: Processor + Send + 'static>(pipeline: P, o: &RunOpts) -> Result<RunReport, String> {
    let gain = gain_linear(o.gain_db)?;
    let Prepared {
        bus,
        mut worker,
        rtp,
        l_total,
        l_pipe,
    } = prepare(pipeline, o.block, o.lead)?;

    let stop = Arc::new(AtomicBool::new(false));
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || stop.store(true, Ordering::Relaxed))
            .map_err(|e| format!("装 Ctrl-C 处理：{e}"))?;
    }

    let rec_stop = Arc::new(AtomicBool::new(false));
    let (tap, recorder) = match &o.record {
        Some(path) => {
            let (tx, rx) = rtrb::RingBuffer::new(RECORD_QUEUE);
            let h = spawn_recorder(path, rx, rec_stop.clone())?;
            (OutputTap::new(gain, Some(tx)), Some(h))
        }
        None => (OutputTap::new(gain, None), None),
    };
    let (rec_dropped, clipped) = (tap.rec_dropped(), tap.clipped());

    let den = {
        let (bus, stop) = (bus.clone(), stop.clone());
        let block = o.block;
        thread::spawn(move || {
            crate::chain::promote_or_report("vdn run 降噪线程", rtp);
            denoise_loop(&mut worker, &bus, block, &stop)
        })
    };

    let input = InputStream::open(
        o.input_device.as_deref(),
        IO_RATE,
        o.block,
        input_callback(bus.clone(), l_total, o.block),
    )
    .map_err(|e| format!("开麦克风：{e}"))?;
    let output = OutputStream::open(
        o.output_device.as_deref(),
        IO_RATE,
        o.block,
        output_callback(bus.clone(), tap),
    )
    .map_err(|e| format!("开扬声器：{e}"))?;
    eprintln!(
        "vdn run: 输入 {:?} → 输出 {:?} · L_total {} 样本（{:.2} ms）= Pipeline {} + 环余量 {} · 增益 {:+.1} dB{} · Ctrl-C 停",
        input.info(),
        output.info(),
        l_total,
        l_total as f64 / 48.0,
        l_pipe,
        o.lead,
        o.gain_db,
        o.record
            .as_ref()
            .map(|p| format!(" · 录音 {}", p.display()))
            .unwrap_or_default()
    );

    let until = o
        .seconds
        .map(|s| Instant::now() + Duration::from_secs_f64(s));
    let mut silence = SilenceNotice::default();
    while !stop.load(Ordering::Relaxed) && until.is_none_or(|u| Instant::now() < u) {
        thread::sleep(Duration::from_millis(50));
        if silence.update(input.health().silent_now()) {
            eprintln!(
                "vdn run: 麦克风连续 1 s 全是 0：可能没有麦克风权限（系统设置 › 隐私与安全性 › 麦克风，给运行本命令的终端打开），或设备被静音"
            );
        }
        if input.health().needs_reopen() || output.health().needs_reopen() {
            eprintln!("vdn run: 设备流停了（拔掉或切换了设备？），退出");
            break;
        }
    }
    stop.store(true, Ordering::Relaxed);
    let (in_level, out_level) = (input.health().level(), output.health().level());
    drop(input);
    drop(output);
    let ws = den.join().map_err(|_| "降噪线程 panic")?;
    rec_stop.store(true, Ordering::Release);
    let rec_written = match recorder {
        Some(h) => Some(h.join().map_err(|_| "录音线程 panic")??),
        None => None,
    };
    let r = RunReport {
        input: in_level,
        output: out_level,
        worker: ws,
        raw_overflow: bus.ring(RingId::Raw).overflows(),
        out_overflow: bus.ring(RingId::Out).overflows(),
        out_underrun: bus.ring(RingId::Out).underruns(),
        clipped: clipped.load(Ordering::Relaxed),
        rec_written,
        rec_dropped: rec_dropped.load(Ordering::Relaxed),
    };
    eprintln!("{}", summary(&r));
    Ok(r)
}

/// 「输入全零」提示的节流：进入全零时提示一次；有声音之后再全零满 1 s 才会再提示（所以最多每秒一次，不刷屏）。
#[derive(Debug, Default)]
pub struct SilenceNotice {
    shown: bool,
}

impl SilenceNotice {
    /// 返回 true = 这次该打印提示。
    pub fn update(&mut self, silent_now: bool) -> bool {
        match (silent_now, self.shown) {
            (true, false) => {
                self.shown = true;
                true
            }
            (false, true) => {
                self.shown = false;
                false
            }
            _ => false,
        }
    }
}

/// 结束时的一段汇总（电平 + 计数）。
pub fn summary(r: &RunReport) -> String {
    let lv = |l: &Level| {
        format!(
            "峰值 {} · 均方根 {}（{} 样本，最长连续全零 {:.2} s）",
            fmt_dbfs(l.peak_dbfs()),
            fmt_dbfs(l.rms_dbfs()),
            l.samples,
            l.max_zero_run as f64 / f64::from(IO_RATE)
        )
    };
    let mut s = format!(
        "vdn run: 结束 · 降噪 {} 块 · Raw 溢出 {} · Out 溢出 {} · 输出欠载 {} 样本\n  输入（麦克风）：{}\n  输出（耳机）：{}",
        r.worker.blocks,
        r.raw_overflow,
        r.out_overflow,
        r.out_underrun,
        lv(&r.input),
        lv(&r.output)
    );
    if r.clipped > 0 {
        s.push_str(&format!("\n  加增益后被夹住 {} 样本（增益太大）", r.clipped));
    }
    if let Some(n) = r.rec_written {
        s.push_str(&format!(
            "\n  录音写了 {n} 样本（{:.2} s）· 队列满丢掉 {} 样本",
            n as f64 / f64::from(IO_RATE),
            r.rec_dropped
        ));
    }
    s
}

/// `--check` 的判定（机器辅助 V5；最终仍由人耳判）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Verdict {
    /// 一个麦克风样本都没收到
    NoInput,
    /// 收到的全是 0：疑似没有麦克风权限或设备静音
    InputAllZero,
    /// 直通时输出电平和「输入 + 增益」差太多（dB）：链路里丢了声音
    OutputMismatch { diff_db: f64 },
    /// 输入有声、（直通时）输出电平对得上。`quiet` = 输入峰值低于 [`QUIET_PEAK_DBFS`]，耳机里可能听不出来
    Pass { quiet: bool, diff_db: Option<f64> },
}

/// 直通时输出均方根允许偏离「输入 + 增益」多少 dB
pub const LEVEL_TOLERANCE_DB: f64 = 3.0;
/// 输入峰值低于这个就提示「很小声」（2026-11-02 实测：MacBook Air 麦克风说话峰值约 −28 dBFS）
pub const QUIET_PEAK_DBFS: f64 = -30.0;

/// 判定。`passthrough` 时才比对输入 / 输出电平（降噪会改电平）。
pub fn judge(input: &Level, output: &Level, passthrough: bool, gain_db: f64) -> Verdict {
    if input.samples == 0 {
        return Verdict::NoInput;
    }
    if input.all_zero() {
        return Verdict::InputAllZero;
    }
    let quiet = input.peak_dbfs() < QUIET_PEAK_DBFS;
    if !passthrough {
        return Verdict::Pass {
            quiet,
            diff_db: None,
        };
    }
    let diff = output.rms_dbfs() - input.rms_dbfs() - gain_db;
    // 输出全零时 diff = −∞，也落在这里
    if !diff.is_finite() || diff.abs() > LEVEL_TOLERANCE_DB {
        return Verdict::OutputMismatch { diff_db: diff };
    }
    Verdict::Pass {
        quiet,
        diff_db: Some(diff),
    }
}

impl Verdict {
    pub fn passed(&self) -> bool {
        matches!(self, Verdict::Pass { .. })
    }
    /// 给人看的一行
    pub fn describe(&self) -> String {
        match self {
            Verdict::NoInput => "不过：没收到麦克风数据（设备没开起来？）".into(),
            Verdict::InputAllZero => "不过：输入全零——疑似没有麦克风权限（系统设置 › 隐私与安全性 › 麦克风）或设备静音".into(),
            Verdict::OutputMismatch { diff_db } => format!(
                "不过：输入有声，但输出电平和输入差 {} dB（允许 ±{LEVEL_TOLERANCE_DB}）——链路里丢了声音",
                if diff_db.is_finite() { format!("{diff_db:+.1}") } else { "−∞".into() }
            ),
            Verdict::Pass { quiet, diff_db } => {
                let mut s = String::from("过（机器部分）：输入有声");
                if let Some(d) = diff_db {
                    s.push_str(&format!(" · 输出电平与输入一致（差 {d:+.1} dB）"));
                }
                if *quiet {
                    s.push_str(&format!(
                        " · ⚠️ 输入很小声（峰值低于 {QUIET_PEAK_DBFS} dBFS），耳机里可能听不出来：加 --gain-db 12 或回放 --record 的文件"
                    ));
                }
                s
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdn_io_macos::pump::{InputPump, OutputPump};

    /// 直通管线 + 定块长 96，块数够跑 `secs` 秒
    fn drive(gain_db: f64, amp: f32, secs: f64) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Level, Level, usize, u64) {
        let block = vdn_control::Config::DEFAULT_BLOCK;
        let lead = vdn_control::Config::DEFAULT_LEAD;
        let mut p = prepare(vdn_denoised::identity_pipeline(None).unwrap(), block, lead).unwrap();
        let (tx, mut rx) = rtrb::RingBuffer::new(4 * IO_RATE as usize);
        let tap = OutputTap::new(gain_linear(gain_db).unwrap(), Some(tx));
        let clipped = tap.clipped();
        // 真跑时的两个泵：输入单声道，输出 2 声道（V5 那副耳机是 2 声道）
        let mut inp = InputPump::new(block, 1, input_callback(p.bus.clone(), p.l_total, block));
        let mut outp = OutputPump::new(block, 2, output_callback(p.bus.clone(), tap));
        let (ih, oh) = (inp.health(), outp.health());
        let n = (secs * f64::from(IO_RATE)) as usize;
        let x: Vec<f32> = vdn_capture::testing::test_tone(IO_RATE, n, 0)
            .into_iter()
            .map(|s| s * amp)
            .collect();
        let (mut left, mut right) = (Vec::new(), Vec::new());
        let mut buf = vec![0.0f32; 2 * block];
        for c in x.chunks(block) {
            inp.on_data(c);
            while p.worker.step(&p.bus) == Step::Block {}
            let o = &mut buf[..2 * c.len()];
            outp.on_data(o);
            for f in o.chunks_exact(2) {
                left.push(f[0]);
                right.push(f[1]);
            }
        }
        let mut rec = Vec::new();
        while let Ok(s) = rx.pop() {
            rec.push(s);
        }
        (x, left, right, rec, ih.level(), oh.level(), p.l_total, clipped.load(Ordering::Relaxed))
    }

    /// V5 的不开设备版：vdn run 的同一套回调（输入泵 → Forwarder → Worker(直通) → Out → 输出泵 2 声道），
    /// 喂正弦，判「两个声道都有声且相同 · 输出 = 输入延迟 L_total · 电平一致 · 录音 = 送进耳机的」
    #[test]
    fn run_path_without_devices_passes_sine_through_at_same_level() {
        let (x, left, right, rec, li, lo, l_total, clipped) = drive(0.0, 0.05, 2.0);
        assert_eq!(left, right, "两个声道应完全相同");
        assert_eq!(rec, left, "录音应等于送进耳机的那一路");
        assert_eq!(clipped, 0);
        let r = crate::verify::pearson_at_lag(&x, &left, l_total, 4800).unwrap();
        assert!(r > 0.999, "输出与输入延迟 L_total 的相关 {r}");
        // 电平：开头 L_total 个零占全长约 1%，均方根差 < 0.2 dB
        let d = lo.rms_dbfs() - li.rms_dbfs();
        assert!(d.abs() < 0.2, "输出 {lo:?} vs 输入 {li:?}（差 {d:.2} dB）");
        assert!(!lo.all_zero());
        assert_eq!(judge(&li, &lo, true, 0.0), Verdict::Pass { quiet: false, diff_db: Some(d) });
    }

    /// 增益 +12 dB：输出均方根比输入高 12 dB；V5 那样小声的输入（峰值约 −28 dBFS）不会被夹
    #[test]
    fn run_path_gain_raises_output_level() {
        let (_x, left, _r, _rec, li, lo, _l, clipped) = drive(12.0, 0.05, 1.0);
        let d = lo.rms_dbfs() - li.rms_dbfs();
        assert!((d - 12.0).abs() < 0.2, "差 {d:.2} dB");
        assert_eq!(clipped, 0);
        assert!(left.iter().all(|s| s.abs() <= 1.0));
        assert!(judge(&li, &lo, true, 12.0).passed());
    }

    /// 增益太大会被夹在 ±1，并计数（不会送出超过满刻度的样本）
    #[test]
    fn run_path_gain_clips_and_counts() {
        let (_x, left, ..) = drive(20.0, 1.0, 0.3);
        assert!(left.iter().all(|s| s.abs() <= 1.0));
        let (.., clipped) = drive(20.0, 1.0, 0.3);
        assert!(clipped > 0);
    }

    #[test]
    fn gain_range_is_checked() {
        assert_eq!(gain_linear(0.0), Ok(1.0));
        assert!((gain_linear(20.0).unwrap() - 10.0).abs() < 1e-5);
        assert!(gain_linear(20.1).is_err());
        assert!(gain_linear(-61.0).is_err());
        assert!(gain_linear(f64::NAN).is_err());
    }

    /// 录音队列满了：不阻塞、丢掉并计数
    #[test]
    fn record_queue_full_drops_and_counts() {
        let (tx, _rx) = rtrb::RingBuffer::new(10);
        let mut tap = OutputTap::new(1.0, Some(tx));
        let d = tap.rec_dropped();
        tap.apply(&mut [0.1; 25]);
        assert_eq!(d.load(Ordering::Relaxed), 15);
    }

    /// 输出电平和输入对不上（例如输出全零）→ 判不过
    #[test]
    fn judge_cases() {
        let lvl = |peak: f32, rms: f64| Level {
            samples: 48_000,
            peak,
            rms,
            ..Level::default()
        };
        let speech = lvl(0.04, 0.011); // 峰值约 −28 dBFS、均方根约 −39 dBFS（2026-11-02 实测量级）
        let zero = lvl(0.0, 0.0);
        assert_eq!(judge(&Level::default(), &speech, true, 0.0), Verdict::NoInput);
        assert_eq!(judge(&zero, &zero, true, 0.0), Verdict::InputAllZero);
        assert!(matches!(
            judge(&speech, &zero, true, 0.0),
            Verdict::OutputMismatch { diff_db } if diff_db == f64::NEG_INFINITY
        ));
        // 输出低 6 dB：超出 ±3 dB
        assert!(matches!(
            judge(&speech, &lvl(0.02, 0.0055), true, 0.0),
            Verdict::OutputMismatch { .. }
        ));
        // 小声但通：过，带「很小声」提示
        assert!(matches!(
            judge(&speech, &speech, true, 0.0),
            Verdict::Pass { quiet: true, .. }
        ));
        // 降噪模式不比电平
        assert_eq!(
            judge(&lvl(0.5, 0.2), &zero, false, 0.0),
            Verdict::Pass { quiet: false, diff_db: None }
        );
        assert!(Verdict::InputAllZero.describe().contains("麦克风权限"));
    }

    /// 「输入全零」提示不刷屏：一段连续全零只提示一次，有声后再全零才再提示
    #[test]
    fn silence_notice_fires_once_per_episode() {
        let mut n = SilenceNotice::default();
        let fired: Vec<bool> = [false, true, true, true, false, false, true, true]
            .into_iter()
            .map(|s| n.update(s))
            .collect();
        assert_eq!(fired, [false, true, false, false, false, false, true, false]);
    }

    /// V5（人判）：真麦克风 → 直通 → 耳机 5 秒，同时录下送进耳机的那一路。会开麦、出声，所以默认不跑。
    ///
    /// 人工步骤：
    /// 1. 戴耳机（外放会啸叫）
    /// 2. `./target/debug/vdn run --passthrough --check --record _tmp/v5.wav`
    /// 3. 5 秒内正常说话
    /// 4. 看结束时打印的判定行（机器部分）；再 `afplay _tmp/v5.wav` 用耳机回放
    ///
    /// 算过：判定行是「过」；回放里听得到自己的声音、不断续、没有咔哒声（自己的声音靠骨传导先到，
    /// 跑的时候听不出 22 ms 后的小声副本是正常的，以回放为准）
    #[test]
    #[ignore = "开麦克风、扬声器出声：人工跑，戴耳机（步骤见文档注释）"]
    fn v5_live_passthrough() {
        let path = std::env::temp_dir().join("vdn-v5-test.wav");
        let o = RunOpts {
            seconds: Some(CHECK_SECONDS),
            block: vdn_control::Config::DEFAULT_BLOCK,
            lead: vdn_control::Config::DEFAULT_LEAD,
            input_device: None,
            output_device: None,
            gain_db: 0.0,
            record: Some(path.clone()),
        };
        let r = run(vdn_denoised::identity_pipeline(None).unwrap(), &o).unwrap();
        let v = judge(&r.input, &r.output, true, 0.0);
        println!("{} · 录音 {}", v.describe(), path.display());
        assert!(v.passed(), "{v:?}");
    }
}
