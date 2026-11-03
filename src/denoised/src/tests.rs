//! vdn-denoised 单测：Worker 的对齐、跳过积压、命令、预热、调度参数与不分配。

use super::*;
use crate::worker::quantile;
use vdn_shm::{Bus, BusConfig, RingId};
use vdn_capture::shm::forward_same_bus;
use vdn_capture::{Forwarder, ForwarderConfig};

/// 输出 = 输入、延迟 0
struct Copy0;
impl Processor for Copy0 {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        output.copy_from_slice(input);
    }
    fn latency_samples(&self) -> usize {
        0
    }
}

fn heap(l_total: usize) -> Bus {
    Bus::heap(BusConfig {
        rate: IO_RATE,
        capacity_frames: 1 << 14,
        l_total: l_total as u32,
        lead: 0,
        denoise_block: 96,
    })
    .unwrap()
}

#[test]
fn prime_then_steps_beat_once_per_block() {
    let mut w = Worker::new(Copy0, 480);
    let bus = heap(1440);
    assert_eq!(w.prime(&bus, 1440), 1440);
    assert_eq!(w.step(&bus), Step::Idle);
    bus.ring(RingId::Raw).push(&[1.0; 1000]);
    assert_eq!(w.step(&bus), Step::Block);
    assert_eq!(w.step(&bus), Step::Block);
    assert_eq!(w.step(&bus), Step::Idle);
    assert_eq!(bus.heartbeat(), 2);
    assert_eq!(bus.ring(RingId::Processed).available(), 1440 + 960);
    assert_eq!(w.stats().blocks, 2);
}

/// 交叉校验（唯一真相源，CLAUDE.md 第五节第 5 条）：总线名规则的正本在 vdn-shm（`Bus::open` 报 `InvalidName`），
/// vdn-control 的 `validate_bus_name` 是给配置文件早报错用的第二处表示，两者对每个样例必须一致
#[test]
fn bus_name_rule_matches_vdn_shm() {
    let long = format!("/vdn-{}", "a".repeat(30));
    // 14 个字符、32 字节
    let multi = format!("/vdn-{}", "中".repeat(9));
    for name in [
        "/vdn-bus",
        "/vdn-out",
        "/vdn-chaintest-out",
        "/vdn-",
        "/vdn",
        "/vdnx",
        "vdn-bus",
        "/other",
        "/vdn-a/b",
        "/vdn-a\0b",
        &long,
        "/vdn-ok_1.2",
        // 名字只许可见 ASCII（vdn-shm 收窄，codex 第 2 轮 #2）：空格、中文、多字节超 31 字节、再含 / 都拒；可见符号放行
        "/vdn-a b",
        "/vdn-中",
        &multi,
        "/vdn-x/",
        "/vdn-~!",
    ] {
        let control_ok = vdn_control::validate_bus_name(name).is_ok();
        let shm_ok = !matches!(Bus::open(name), Err(vdn_shm::ShmError::InvalidName));
        assert_eq!(
            control_ok, shm_ok,
            "{name:?}：vdn-control {control_ok} · vdn-shm {shm_ok}"
        );
    }
}

/// 交叉校验：IO 采样率三处（vdn-control 正本 · 总线头 · 降噪总线配置）一致
#[test]
fn io_rate_single_source() {
    let w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
    let cfg = denoise_bus_config(&w, 288, 48_000).unwrap();
    assert_eq!(cfg.rate, vdn_control::IO_RATE);
    assert_eq!(cfg.lead, 288);
    assert_eq!(cfg.denoise_block, 96);
    assert_eq!(cfg.l_total as usize, w.l_total(288));
    let bus = Bus::heap(cfg).unwrap();
    assert_eq!(bus.config(), cfg);
}

/// 输出 = 0.5 × 输入延迟 768（和原声可区分，延迟同直通管线）
struct HalfDelay {
    line: Vec<f32>,
    pos: usize,
}
impl Processor for HalfDelay {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for (x, y) in input.iter().zip(output.iter_mut()) {
            *y = 0.5 * std::mem::replace(&mut self.line[self.pos], *x);
            self.pos = (self.pos + 1) % self.line.len();
        }
    }
    fn latency_samples(&self) -> usize {
        self.line.len()
    }
}

/// 降噪停 100 块后恢复：跳过 Raw 积压（零占位 + 缺口计数，带外），送音端在那个窗口送原声；
/// 全程不静音、不跳变，恢复后立刻回到「现在」并逐样本对齐（不追 1 s 的积压）
#[test]
fn skip_backlog_keeps_alignment_without_silence() {
    let (block, lead) = (96, 288);
    let mut w = Worker::new(
        HalfDelay {
            line: vec![0.0; 768],
            pos: 0,
        },
        block,
    )
    .skip_backlog_over_lead(lead);
    let l_total = w.l_total(lead);
    let bus = heap(l_total);
    w.prime(&bus, lead);
    let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
    let n = 1500;
    let x = vdn_capture::testing::test_tone(IO_RATE, n * block, 0);
    let mut y = vec![0.0f32; x.len()];
    for (i, (c, yc)) in x.chunks(block).zip(y.chunks_mut(block)).enumerate() {
        forward_same_bus(&mut f, &bus, c);
        if !(500..600).contains(&i) {
            while w.step(&bus) == Step::Block {}
        }
        assert_eq!(bus.ring(RingId::Out).pop(yc), c.len());
    }
    let s = w.stats();
    assert!(
        s.raw_skipped > 0 && s.placeholders_pushed == s.raw_skipped,
        "{s:?}"
    );
    assert!(bus.ring(RingId::Raw).gaps() > 0);
    let fs = f.stats();
    assert!(fs.gap_holds >= 1, "{fs:?}");
    assert_eq!(fs.nonfinite, 0);
    // 不静音（零占位没被播出）、不跳变
    let quiet = y[l_total..]
        .split(|v| v.abs() >= 1e-3)
        .map(<[f32]>::len)
        .max()
        .unwrap_or(0);
    assert!(quiet < 50, "最长静音 {quiet} 样本");
    let jump = y
        .windows(2)
        .skip(l_total)
        .map(|p| (p[1] - p[0]).abs())
        .fold(0.0, f32::max);
    assert!(jump < 0.13, "最大跳变 {jump}");
    // 停顿结束后不到 0.2 s 就回到处理后的（0.5 倍）并逐样本对齐
    let back = (600 + 100) * block;
    for k in back..y.len() {
        assert!(
            (y[k] - 0.5 * x[k - l_total]).abs() < 1e-4,
            "k={k}: {} vs {}",
            y[k],
            0.5 * x[k - l_total]
        );
    }
    assert!(f.is_fresh());
}

/// 运行中切模式：命令经 rtrb 送进降噪线程，块之间执行；Pipeline 按一个 hop 淡化，输出连续
#[test]
fn mode_switch_over_rtrb_is_applied_between_blocks() {
    use vdn_control::{Cmd, Mode};
    let mut w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
    let (mut tx, mut rx) = command_channel(8);
    let bus = heap(w.l_total(288));
    w.prime(&bus, 288);
    let x = vdn_capture::testing::test_tone(IO_RATE, 96, 0);
    assert!(tx.push(Cmd::SetMode(Mode::Passthrough)).is_ok());
    assert!(tx.push(Cmd::SetMode(Mode::Denoise)).is_ok());
    assert!(w.apply_commands(&mut rx));
    bus.ring(RingId::Raw).push(&x);
    assert_eq!(w.step(&bus), Step::Block);
    assert!(tx.push(Cmd::Shutdown).is_ok());
    assert!(!w.apply_commands(&mut rx));
    assert_eq!(w.stats().commands, 3);
    assert_eq!(
        engine_mode(Mode::Passthrough),
        vdn_engine::Mode::Passthrough
    );
}

/// 预热量的「每周期总耗时」：窗口 = 一个工作周期的块数（定块长 96 的 Identity 管线：hop 768 → 8 块）
#[test]
fn warmup_measures_per_period_work() {
    let mut w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
    let wu = w.warm_up(64);
    assert_eq!(wu.period_blocks, 8);
    assert!(wu.max_period_ns >= wu.p999_period_ns);
    assert!(
        wu.max_period_ns >= wu.max_after_first_ns,
        "一个周期的总和不小于其中最长的一块"
    );
    // 没有周期概念的处理器按单块算
    let mut w0 = Worker::new(Copy0, 96);
    assert_eq!(w0.warm_up(16).period_blocks, 1);
    assert_eq!(quantile(&[], 0.999), 0);
    assert_eq!(quantile(&[10, 20], 1.0), 20);
    assert_eq!(quantile(&[10, 20], 0.5), 15);
}

/// 直接判据计数：处理慢于周期时长就记「超周期」；Raw 积压越多「醒来迟到」越大
#[test]
fn overrun_and_backlog_counters() {
    /// 每块睡 3 ms（> 一块 2 ms），一个周期（2 块）总耗时 6 ms > 4 ms
    struct Slow;
    impl Processor for Slow {
        fn process(&mut self, i: &[f32], o: &mut [f32]) {
            std::thread::sleep(std::time::Duration::from_millis(3));
            o.copy_from_slice(i);
        }
        fn latency_samples(&self) -> usize {
            0
        }
        fn period_samples(&self) -> usize {
            192
        }
    }
    let mut w = Worker::new(Slow, 96);
    let bus = heap(0);
    bus.ring(RingId::Raw).push(&[0.0; 96 * 5]);
    while w.step(&bus) == Step::Block {}
    let s = w.stats();
    assert_eq!(s.blocks, 5);
    assert_eq!(s.max_raw_backlog, 480, "第一次开始处理时积了 5 块");
    assert!(s.over_period >= 4, "{s:?}");
    assert!(s.max_period_ns >= 6_000_000, "{s:?}");
}

/// D68 调度参数：周期报一个工作周期（GTCRN 定块 96 的管线 hop 768 = 16 ms，不是 2 ms），
/// computation = 每周期总耗时 p99.9 × 1.5，夹在周期内
#[test]
fn denoise_rt_params_use_period_and_measured_window() {
    let mut w = Worker::new(identity_pipeline(Some(96)).unwrap(), 96);
    let wu = w.warm_up(64);
    let p = denoise_rt_params(&w, &wu);
    assert_eq!(p.period_ns, 16_000_000);
    assert_eq!(p.constraint_ns, p.period_ns);
    // 规则只在正本 `RtParams::from_measured` 里：这里比对「同样的输入 → 同样的输出」，不抄夹取规则
    assert_eq!(
        p,
        vdn_rt_thread::RtParams::from_measured(
            p.period_ns,
            wu.p999_period_ns.max(1),
            vdn_rt_thread::MEASURED_MARGIN
        )
    );
    // 直通管线实测极小（2026-11-02 `vdn run` code 4 的起因）：给出的参数内核也要接受
    assert!(p.validate().is_ok(), "{p:?} · 预热 {wu:?}");
    assert!(p.computation_ns >= vdn_rt_thread::MIN_COMPUTATION_NS);
}

#[test]
fn rt_check_counts_downgrades() {
    let mut w = Worker::new(Copy0, 96);
    w.note_rt_check(true);
    w.note_rt_check(false);
    w.note_rt_check(true);
    assert_eq!((w.stats().rt_checks, w.stats().rt_downgrades), (3, 1));
}

#[test]
fn prime_stops_when_full() {
    let mut w = Worker::new(Copy0, 480);
    let bus = heap(0);
    let cap = bus.capacity();
    assert_eq!(w.prime(&bus, cap + 1000), cap);
}

#[test]
fn identity_pipeline_latency() {
    let p = identity_pipeline(None).unwrap();
    assert_eq!(Processor::latency_samples(&p), 863);
    let p = identity_pipeline(Some(96)).unwrap();
    assert_eq!(Processor::latency_samples(&p), 768);
    assert!(identity_pipeline(Some(100)).is_err());
}

/// Worker + Forwarder 经同一条进程内总线：直通管线下输出 = 输入延迟 L_total（重采样有轻微纹波，按相关判）
#[test]
fn worker_and_forwarder_align_end_to_end() {
    // IO 块 480，降噪块 96（定块长管线）
    let (block, lead, dblock) = (480, 1440, 96);
    let mut w = Worker::new(identity_pipeline(Some(dblock)).unwrap(), dblock);
    // 预热不影响对齐（输出丢掉、之前的输入等于零）
    let wu = w.warm_up(16);
    assert_eq!(wu.blocks, 16);
    let l_total = w.l_total(lead);
    let bus = heap(l_total);
    w.prime(&bus, lead);
    let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
    let x = vdn_capture::testing::test_tone(IO_RATE, 200 * block, 0);
    let mut y = vec![0.0; x.len()];
    for (c, yc) in x.chunks(block).zip(y.chunks_mut(block)) {
        forward_same_bus(&mut f, &bus, c);
        while w.step(&bus) == Step::Block {}
        assert_eq!(bus.ring(RingId::Out).pop(yc), c.len());
    }
    assert_eq!(f.stats().proc_underrun, 0);
    assert!(f.is_fresh());
    // 相关（跳过开头 100 ms）
    let skip = 4800;
    let (a, b) = (&x[skip..x.len() - l_total], &y[skip + l_total..]);
    let dot: f64 = a
        .iter()
        .zip(b)
        .map(|(p, q)| f64::from(*p) * f64::from(*q))
        .sum();
    let na: f64 = a.iter().map(|p| f64::from(*p).powi(2)).sum();
    let nb: f64 = b.iter().map(|q| f64::from(*q).powi(2)).sum();
    let r = dot / (na * nb).sqrt();
    assert!(r > 0.999, "相关 {r}");
}
/// 实时路径不分配（R1）：真进程内总线 + 定块长直通管线，走遍 正常 → 降噪停（Raw 溢出、缺口、心跳过期）
/// → 恢复（迟到丢弃）→ 积压超限重新对账 这几条路径，Forwarder 与 Worker 都在 assert_no_alloc 里跑
#[test]
fn realtime_paths_do_not_allocate() {
    let (block, lead, dblock) = (480, 1440, 96);
    let mut w = Worker::new(identity_pipeline(Some(dblock)).unwrap(), dblock);
    let l_total = w.l_total(lead);
    let bus = heap(l_total);
    w.prime(&bus, lead);
    let mut f = Forwarder::new(ForwarderConfig::new(l_total, block));
    let x = vdn_capture::testing::test_tone(IO_RATE, block, 0);
    let mut y = vec![0.0f32; block];
    let junk = vec![0.5f32; 20 * block];
    // 热身
    for _ in 0..10 {
        forward_same_bus(&mut f, &bus, &x);
        while w.step(&bus) == Step::Block {}
        bus.ring(RingId::Out).pop(&mut y);
    }
    assert_no_alloc::assert_no_alloc(|| {
        for i in 0..400 {
            forward_same_bus(&mut f, &bus, &x);
            if !(100..200).contains(&i) {
                while w.step(&bus) == Step::Block {}
            }
            if i == 300 {
                bus.ring(RingId::Processed).push(&junk);
            }
            bus.ring(RingId::Out).pop(&mut y);
        }
    });
    let s = f.stats();
    assert!(s.raw_overflow > 0, "{s:?}");
    assert!(s.stale_switches >= 1, "{s:?}");
    assert!(s.discarded > 0, "{s:?}");
    assert!(s.resyncs >= 1, "{s:?}");
    assert!(s.cuts >= 1, "{s:?}");
}
