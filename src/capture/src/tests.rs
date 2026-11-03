//! Forwarder 单测：假环模拟降噪进程（直通：处理后 = 原声），检查对齐、送原声、淡回、对齐账。

use super::*;
use crate::testing::{FakeBeat, FakeRing};

const BLOCK: usize = 480;
const LEAD: usize = 1440;

fn sig(n: usize, off: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let t = (i + off) as f32 / 48_000.0;
            0.5 * (2.0 * std::f32::consts::PI * 440.0 * t).sin()
                + 0.3 * (2.0 * std::f32::consts::PI * 2000.0 * t).sin()
        })
        .collect()
}

/// 直通「降噪进程」：先推 lead 个零，之后把 Raw 原样搬到 Processed 并心跳。
struct FakeDenoiser {
    alive: bool,
    /// 每次最多搬多少（模拟慢）
    rate_limit: usize,
    /// 处理后 = 原声 × gain（1 = 直通；0.1 = 和原声明显不同，用来查切换是否连续）
    gain: f32,
}

impl FakeDenoiser {
    fn step(&mut self, raw: &mut FakeRing, processed: &mut FakeRing, hb: &FakeBeat) {
        if !self.alive {
            return;
        }
        let mut b = vec![0.0; self.rate_limit];
        let n = raw.pop(&mut b);
        if n > 0 {
            for v in &mut b[..n] {
                *v *= self.gain;
            }
            processed.push(&b[..n]);
            hb.beat();
        }
    }
}

struct Rig {
    fwd: Forwarder,
    raw: FakeRing,
    processed: FakeRing,
    out: FakeRing,
    hb: FakeBeat,
    den: FakeDenoiser,
    t: usize,
}

impl Rig {
    fn new() -> Self {
        let mut processed = FakeRing::new(1 << 16);
        processed.push(&vec![0.0; LEAD]);
        Self {
            fwd: Forwarder::new(ForwarderConfig::new(LEAD, BLOCK)),
            raw: FakeRing::new(1 << 16),
            processed,
            out: FakeRing::new(1 << 20),
            hb: FakeBeat::default(),
            den: FakeDenoiser {
                alive: true,
                rate_limit: BLOCK,
                gain: 1.0,
            },
            t: 0,
        }
    }
    /// 一个回调周期：送音端先跑，降噪进程随后跑（最坏次序）。
    fn tick(&mut self) {
        let x = sig(BLOCK, self.t);
        self.t += BLOCK;
        self.fwd.process(
            &x,
            &mut self.raw,
            &mut self.processed,
            &mut self.out,
            &self.hb,
        );
        self.den.step(&mut self.raw, &mut self.processed, &self.hb);
    }
}

/// out[k] 应等于 sig[k - LEAD]（直通时处理后与原声相同，权重无关）
fn assert_aligned(y: &[f32], from: usize) {
    let x = sig(y.len(), 0);
    for k in from.max(LEAD)..y.len() {
        assert!(
            (y[k] - x[k - LEAD]).abs() < 1e-5,
            "k={k}: {} vs {}",
            y[k],
            x[k - LEAD]
        );
    }
}

#[test]
fn steady_state_aligned_and_fresh() {
    let mut r = Rig::new();
    for _ in 0..200 {
        r.tick();
    }
    let y = r.out.drain_all();
    assert_eq!(y.len(), 200 * BLOCK);
    assert_aligned(&y, 0);
    assert!(r.fwd.is_fresh());
    assert_eq!(r.fwd.gain(), 1.0);
    let s = r.fwd.stats();
    assert_eq!(s.proc_underrun, 0);
    assert_eq!(s.raw_overflow, 0);
    assert_eq!(s.discarded, 0);
    assert_eq!(s.recoveries, 1);
    assert_eq!(s.stale_switches, 0);
}

#[test]
fn denoiser_dies_switches_to_raw_within_timeout_without_gap() {
    let mut r = Rig::new();
    for _ in 0..100 {
        r.tick();
    }
    let kill_at = r.fwd.stats().samples;
    r.den.alive = false;
    for _ in 0..100 {
        r.tick();
    }
    let s = r.fwd.stats();
    assert_eq!(s.stale_switches, 1);
    let switch = s.last_stale_at.unwrap() - kill_at;
    // 超时 4800 + 一块粒度
    assert!(
        switch <= 4800 + 2 * BLOCK as u64,
        "switch after {switch} samples"
    );
    assert!(!r.fwd.is_fresh());
    assert_eq!(r.fwd.gain(), 0.0);
    // 死掉之后照样逐样本对齐（缺的处理后样本由延迟原声补），没有静音
    let y = r.out.drain_all();
    assert_aligned(&y, 0);
}

#[test]
fn recovers_and_fades_back() {
    let mut r = Rig::new();
    for _ in 0..50 {
        r.tick();
    }
    r.den.alive = false;
    for _ in 0..50 {
        r.tick();
    }
    assert!(!r.fwd.is_fresh());
    r.den.alive = true; // 同一进程恢复：积压的 Raw 一次搬完，对齐账把迟到的丢掉
    r.den.rate_limit = 1 << 16;
    for _ in 0..50 {
        r.tick();
    }
    assert!(r.fwd.is_fresh());
    assert_eq!(r.fwd.gain(), 1.0);
    assert_eq!(r.fwd.stats().recoveries, 2);
    let y = r.out.drain_all();
    assert_aligned(&y, 0);
}

#[test]
fn slow_denoiser_keeps_alignment() {
    let mut r = Rig::new();
    for i in 0..300 {
        // 每 7 块卡一下：这一块不搬，下一块一起搬
        r.den.alive = i % 7 != 3;
        r.den.rate_limit = 4 * BLOCK;
        r.tick();
    }
    let y = r.out.drain_all();
    assert_aligned(&y, 0);
}

#[test]
fn raw_overflow_keeps_alignment() {
    let mut r = Rig::new();
    r.raw = FakeRing::new(2 * BLOCK);
    for _ in 0..20 {
        r.tick();
    }
    r.den.alive = false;
    for _ in 0..5 {
        r.tick(); // Raw 满了，后几块推不进
    }
    assert!(r.fwd.stats().raw_overflow > 0);
    r.den.alive = true;
    r.den.rate_limit = 1 << 16;
    for _ in 0..40 {
        r.tick();
    }
    let y = r.out.drain_all();
    assert_aligned(&y, 0);
    assert!(r.fwd.is_fresh());
}

#[test]
fn backlog_is_capped() {
    let mut r = Rig::new();
    for _ in 0..10 {
        r.tick();
    }
    // 一个错位的新降噪进程塞进一大段积压
    r.processed.push(&vec![0.25; 20 * BLOCK]);
    r.tick();
    assert!(r.fwd.stats().discarded > 0);
    assert_eq!(r.fwd.stats().resyncs, 1);
    assert!(r.processed.available() <= LEAD + 2 * BLOCK);
    // 重新对账后照常出声、不再欠载
    let before = r.fwd.stats().proc_underrun;
    for _ in 0..20 {
        r.tick();
    }
    assert_eq!(r.fwd.stats().proc_underrun, before);
    assert_eq!(r.fwd.stats().resyncs, 1);
}

#[test]
fn process_into_matches_process_and_splits_long_input() {
    let mut a = Rig::new();
    let mut b = Rig::new();
    let x = sig(5 * BLOCK + 17, 0);
    a.fwd
        .process(&x, &mut a.raw, &mut a.processed, &mut a.out, &a.hb);
    let mut y = vec![9.0; x.len()];
    b.fwd
        .process_into(&x, &mut y, &mut b.raw, &mut b.processed, &b.hb);
    assert_eq!(a.out.drain_all(), y);
    assert_eq!(b.raw.len(), x.len());
}

/// 相邻样本最大跳变。测试信号最大斜率 2π(440·0.5 + 2000·0.3)/48000 ≈ 0.107；淡化 10 ms 时
/// 处理后 ↔ 原声 的权重每样本只变 1/480，带来的额外跳变 ≤ 0.8/480。硬切若不抹平，落差可达 0.9 × 0.8 = 0.72。
fn max_jump(y: &[f32]) -> f32 {
    y.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .fold(0.0, f32::max)
}

const JUMP_LIMIT: f32 = 0.13;

fn rig_01() -> Rig {
    let mut r = Rig::new();
    r.den.gain = 0.1;
    r
}

#[test]
fn switching_is_continuous_when_denoiser_dies() {
    let mut r = rig_01();
    for i in 0..300 {
        r.den.alive = i < 100;
        r.tick();
    }
    let y = r.out.drain_all();
    let j = max_jump(&y[LEAD..]);
    assert!(j < JUMP_LIMIT, "最大跳变 {j}");
    let s = r.fwd.stats();
    assert_eq!(s.stale_switches, 1);
    assert!(s.cuts >= 1);
    // 死了以后输出 = 延迟原声（接缝偏移衰减完以后）
    let x = sig(y.len(), 0);
    for k in 250 * BLOCK..y.len() {
        assert!((y[k] - x[k - LEAD]).abs() < 1e-4, "k={k}");
    }
}

#[test]
fn switching_is_continuous_on_stalls_and_recovery() {
    let mut r = rig_01();
    for i in 0..600 {
        // 卡 50 ms（5 块）后恢复；之后每 50 块停 4 块
        r.den.alive = !(100..105).contains(&i) && !(i > 200 && i % 50 < 4);
        r.den.rate_limit = 8 * BLOCK;
        r.tick();
    }
    let y = r.out.drain_all();
    let j = max_jump(&y[LEAD..]);
    assert!(j < JUMP_LIMIT, "最大跳变 {j}");
    assert!(r.fwd.stats().cuts > 1);
    // 稳定期间是处理后的（0.1 倍）：回滞之后权重回到 1
    assert!(r.fwd.is_fresh());
}

#[test]
fn resync_does_not_jump() {
    let mut r = rig_01();
    for _ in 0..50 {
        r.tick();
    }
    r.processed.push(&vec![0.7; 20 * BLOCK]);
    for _ in 0..50 {
        r.tick();
    }
    assert_eq!(r.fwd.stats().resyncs, 1);
    let y = r.out.drain_all();
    let j = max_jump(&y[LEAD..]);
    assert!(j < JUMP_LIMIT, "最大跳变 {j}");
}

#[test]
fn slow_denoiser_is_reported() {
    let mut r = rig_01();
    for i in 0..400 {
        // 吞吐只有实时的 2/3：每 3 块里搬 2 块
        r.den.alive = i % 3 != 2;
        r.den.rate_limit = BLOCK;
        r.tick();
    }
    let s = r.fwd.stats();
    assert_eq!(s.stale_switches, 0, "心跳一直在动");
    assert!(s.slow_episodes >= 1, "{s:?}");
}

#[test]
fn rebind_to_new_bus_keeps_raw_continuous_and_realigns() {
    let mut r = rig_01();
    for _ in 0..100 {
        r.tick();
    }
    // 降噪进程被杀；过一阵新进程建了新总线（新环、开头 lead 个零、心跳从 0 起）
    r.den.alive = false;
    for _ in 0..50 {
        r.tick();
    }
    r.fwd.rebind();
    r.raw = FakeRing::new(1 << 16);
    r.processed = FakeRing::new(1 << 16);
    r.processed.push(&vec![0.0; LEAD]);
    r.hb = FakeBeat::default();
    r.den.alive = true;
    for _ in 0..100 {
        r.tick();
    }
    let y = r.out.drain_all();
    assert!(max_jump(&y[LEAD..]) < JUMP_LIMIT);
    // 换总线前后都没有静音段（新总线开头的占位零不播）
    let longest_quiet = y[LEAD..]
        .split(|v| v.abs() >= 1e-3)
        .map(<[f32]>::len)
        .max()
        .unwrap_or(0);
    assert!(longest_quiet < 50, "最长静音 {longest_quiet} 样本");
    // 最后一段是处理后的（0.1 倍）且对齐
    let x = sig(y.len(), 0);
    for k in y.len() - 20 * BLOCK..y.len() {
        assert!((y[k] - 0.1 * x[k - LEAD]).abs() < 1e-4, "k={k}");
    }
    assert!(r.fwd.is_fresh());
}

/// 缺口计数涨了（降噪进程跳过积压，带外告知）：接下来 L_total + 2 块送原声（淡出），之后淡回；不跳变
#[test]
fn gap_counter_holds_raw_for_one_window() {
    let mut r = rig_01();
    for _ in 0..100 {
        r.tick();
    }
    assert_eq!(r.fwd.gain(), 1.0);
    r.hb.add_gap(480);
    let mut gains = Vec::new();
    for _ in 0..60 {
        r.tick();
        gains.push(r.fwd.gain());
    }
    let s = r.fwd.stats();
    assert_eq!(s.gap_holds, 1);
    // 窗口里降到 0（原声），窗口过后（L_total + 2 块 + 回滞）回到 1
    assert!(gains.iter().take(30).any(|g| *g == 0.0), "{gains:?}");
    assert_eq!(*gains.last().unwrap(), 1.0);
    let y = r.out.drain_all();
    assert!(max_jump(&y[LEAD..]) < JUMP_LIMIT);
}

/// 防御：处理后的样本不是有限值，输出里也不会有，计数
#[test]
fn nonfinite_processed_never_reaches_output() {
    let mut r = rig_01();
    for _ in 0..50 {
        r.tick();
    }
    r.processed.push(&[f32::NAN, f32::INFINITY, f32::NEG_INFINITY]);
    for _ in 0..50 {
        r.tick();
    }
    let y = r.out.drain_all();
    assert!(y.iter().all(|v| v.is_finite()));
    assert!(r.fwd.stats().nonfinite > 0 || r.fwd.stats().discarded > 0);
}

#[test]
fn process_into_does_not_allocate() {
    let mut r = Rig::new();
    let x = sig(BLOCK, 0);
    let mut y = vec![0.0; BLOCK];
    r.fwd
        .process_into(&x, &mut y, &mut r.raw, &mut r.processed, &r.hb); // 热身
    // FakeRing::push 会分配，所以这里用不分配的「空环」验 Forwarder 自身
    struct Sink;
    impl Push for Sink {
        fn push(&mut self, x: &[f32]) -> usize {
            x.len()
        }
    }
    struct Empty;
    impl Pop for Empty {
        fn pop(&mut self, _: &mut [f32]) -> usize {
            0
        }
        fn available(&self) -> usize {
            0
        }
    }
    let hb = FakeBeat::default();
    assert_no_alloc::assert_no_alloc(|| {
        for _ in 0..10 {
            r.fwd.process_into(&x, &mut y, &mut Sink, &mut Empty, &hb);
            r.fwd.process(&x, &mut Sink, &mut Empty, &mut Sink, &hb);
        }
    });
}

#[cfg(debug_assertions)]
#[global_allocator]
static A: assert_no_alloc::AllocDisabler = assert_no_alloc::AllocDisabler;
