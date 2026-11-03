//! 研究 020：手写 GTCRN（原写法与 v2）逐层计时 + 每层工作量（子命令 `gtcrn-layers`）。
//!
//! 计时用 ARM 通用计时器 `cntvct_el0`（本机 24 MHz，一格约 41.7 ns）。单层只有零点几到几微秒，
//! 一格的量化误差靠几千帧取平均消掉（各层起点与计时器相位无关），所以报**平均值**为主，中位只作参考。
//! 层的顺序、名字、工作量都来自 `GtcrnNative::infer_probed`（正本），这里不另写一份。

use crate::gtcrn_native::{Cost, GtcrnNative, Probe};

#[inline(always)]
pub fn ticks() -> u64 {
    let v: u64;
    // isb：等前面的指令做完再读计时器，避免乱序把层的边界读歪
    unsafe { core::arch::asm!("isb", "mrs {}, cntvct_el0", out(reg) v, options(nostack, nomem)) };
    v
}

pub fn tick_hz() -> u64 {
    let v: u64;
    unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) v, options(nostack, nomem)) };
    v
}

/// 一段的名字与工作量（第 0 帧收集）
pub struct Seg {
    pub blk: &'static str,
    pub op: &'static str,
    pub cost: Cost,
}

/// 计时打点：每帧 `begin` 一次；每个 `mark` 记下距上一点的计时器格数。
pub struct TimerProbe {
    pub segs: Vec<Seg>,
    /// 每帧每段的格数，按帧拼接（帧数 × 段数）
    pub samples: Vec<u32>,
    last: u64,
    idx: usize,
    collect: bool,
}

impl TimerProbe {
    pub fn new(frames: usize) -> Self {
        Self {
            segs: Vec::with_capacity(64),
            samples: Vec::with_capacity(frames * 64),
            last: 0,
            idx: 0,
            collect: true,
        }
    }
    /// 第 0 帧之后调用：段表定型，之后只记时间
    pub fn freeze(&mut self) {
        self.collect = false;
        self.samples.clear();
    }
    #[inline(always)]
    pub fn begin(&mut self) {
        self.idx = 0;
        self.last = ticks();
    }
    pub fn n(&self) -> usize {
        self.segs.len()
    }
}

impl Probe for TimerProbe {
    #[inline(always)]
    fn mark(&mut self, blk: &'static str, op: &'static str, cost: impl FnOnce() -> Cost) {
        let now = ticks();
        if self.collect {
            self.segs.push(Seg { blk, op, cost: cost() });
        } else {
            debug_assert!(self.segs[self.idx].blk == blk && self.segs[self.idx].op == op);
            self.samples.push((now - self.last) as u32);
        }
        self.idx += 1;
        self.last = ticks();
    }
}

fn cost_json(c: &Cost) -> String {
    format!(
        "\"macs\":{},\"nonlin\":{},\"other\":{},\"w_bytes\":{},\"io_bytes\":{},\"chain_fma\":{},\"chain_nl\":{}",
        c.macs, c.nonlin, c.other, c.w_bytes, c.io_bytes, c.chain_fma, c.chain_nl
    )
}

/// `gtcrn-layers <权重前缀> <wav> [--frames N] [--warmup N] [--qos rt|ui|bg|default] [--pace-ms X] [--variant native|native-fast]`
/// 输出一行 JSON：每段平均 / 中位纳秒 + 工作量，以及整帧（各段之和）。
pub fn run(args: &[String], inputs_for: impl Fn(usize) -> Result<Vec<Vec<f32>>, crate::Err>) -> Result<(), crate::Err> {
    use crate::{arg_str, arg_usize};
    let frames = arg_usize(args, "--frames", 6000);
    let warm = arg_usize(args, "--warmup", 200);
    let pace: f64 = arg_str(args, "--pace-ms", "0").parse()?;
    let qos = arg_str(args, "--qos", "rt");
    let variant = arg_str(args, "--variant", "native");
    if frames == 0 || !pace.is_finite() || !(0.0..=10_000.0).contains(&pace) {
        return Err("--frames 要 > 0，--pace-ms 要在 0..=10000".into());
    }
    let kind = match variant {
        "native" | "native-fast" => crate::gtcrn_native::Kind::Scalar,
        "native-v2" => crate::gtcrn_native::Kind::V2,
        "native-v2-fast" => crate::gtcrn_native::Kind::V2Fast,
        _ => return Err("--variant 只收 native / native-fast / native-v2 / native-v2-fast（BLAS 版每帧会分配，不在下界分析里）".into()),
    };
    crate::set_thread_qos(qos, 16.0)?;
    crate::gtcrn_native::BLAS.store(false, std::sync::atomic::Ordering::Relaxed);
    crate::gtcrn_native::FAST.store(variant == "native-fast", std::sync::atomic::Ordering::Relaxed);
    let mut net = GtcrnNative::load(std::path::Path::new(&args[0]))?;
    net.set_kind(kind);
    let inputs = inputs_for(1 + warm + frames)?;
    let mut p = TimerProbe::new(frames);
    let mut x = inputs[0].clone();
    {
        let (re, im) = x.split_at_mut(257);
        p.begin();
        net.infer_probed(re, im, &mut p);
    }
    p.freeze();
    let n = p.n();
    let mut nop = crate::gtcrn_native::NoProbe;
    for f in &inputs[1..=warm] {
        x.copy_from_slice(f);
        let (re, im) = x.split_at_mut(257);
        net.infer_probed(re, im, &mut nop);
    }
    let w0 = std::time::Instant::now();
    for (k, f) in inputs[warm + 1..].iter().enumerate() {
        if pace > 0.0 {
            let due = w0 + std::time::Duration::from_secs_f64(k as f64 * pace * 1e-3);
            if let Some(d) = due.checked_duration_since(std::time::Instant::now()) {
                std::thread::sleep(d);
            }
        }
        x.copy_from_slice(f);
        let (re, im) = x.split_at_mut(257);
        p.begin();
        net.infer_probed(re, im, &mut p);
    }
    let ns_per_tick = 1e9 / tick_hz() as f64;
    let mut per_seg = Vec::with_capacity(n);
    let mut frame_ticks = vec![0u64; frames];
    for i in 0..n {
        let mut v: Vec<u32> = (0..frames).map(|f| p.samples[f * n + i]).collect();
        for (f, t) in v.iter().enumerate() {
            frame_ticks[f] += *t as u64;
        }
        let mean = v.iter().map(|t| *t as f64).sum::<f64>() / frames as f64 * ns_per_tick;
        v.sort_unstable();
        let p50 = v[frames / 2] as f64 * ns_per_tick;
        let s = &p.segs[i];
        per_seg.push(format!(
            "{{\"i\":{i},\"blk\":\"{}\",\"op\":\"{}\",\"mean_ns\":{mean:.1},\"p50_ns\":{p50:.1},{}}}",
            s.blk,
            s.op,
            cost_json(&s.cost)
        ));
    }
    frame_ticks.sort_unstable();
    let fmean = frame_ticks.iter().sum::<u64>() as f64 / frames as f64 * ns_per_tick;
    let total = p.segs.iter().fold(Cost::default(), |a, s| a + s.cost);
    println!(
        "{{\"model\":\"gtcrn\",\"backend\":\"{variant}\",\"frames\":{frames},\"warmup\":{warm},\"pace_ms\":{pace},\"qos\":\"{qos}\",\"tick_hz\":{},\"frame_mean_ns\":{fmean:.1},\"frame_p50_ns\":{:.1},\"frame_p99_ns\":{:.1},\"total\":{{{}}},\"segs\":[{}]}}",
        tick_hz(),
        frame_ticks[frames / 2] as f64 * ns_per_tick,
        frame_ticks[(frames * 99) / 100] as f64 * ns_per_tick,
        cost_json(&total),
        per_seg.join(",")
    );
    Ok(())
}
