//! 研究 020：本机硬件峰值微基准（子命令 `hw-peak`）。下界分析用的每个常数都从这里来，不用网上数字。
//!
//! 量什么（都按纳秒报，不换算成周期——用户态读不到周期计数器）：
//! - `fma_tput_lanes_per_ns`：NEON 128 位 FMA 的吞吐，按 f32 乘加个数计；用 4–32 条独立累加链，取最大
//! - `fma_lat_ns`：一条依赖链上一条向量 FMA 的延迟；`fadd_lat_ns` 同理（向量加）
//! - `exp_fast_lat_ns` / `sig_fast_lat_ns` / `tanh_libm_lat_ns`：标量非线性一次的依赖延迟
//! - `exp_fast_tput_ns`：标量 exp_fast 在互不依赖的数组上每个的平均耗时（编译器可能自动向量化）
//! - `read_gbps[size]` / `write_gbps[size]`：顺序读 / 写带宽，缓冲 16 KB – 64 MB（覆盖 L1 / L2 / 系统缓存 / 内存）
//! 每项跑 `reps` 次（默认 9），报中位、最小、最大；单次约 10–30 ms。
//! 调度等级用 `--qos`：rt（实时策略，性能核）· bg（后台，系统倾向放能效核）。哪种核实际在跑，用不同 qos 下的数对比判断。

use core::arch::aarch64::*;
use std::hint::black_box;
use std::time::Instant;

fn stats(mut v: Vec<f64>) -> (f64, f64, f64) {
    v.sort_by(|a, b| a.total_cmp(b));
    (v[v.len() / 2], v[0], v[v.len() - 1])
}

fn js((med, lo, hi): (f64, f64, f64), n: usize) -> String {
    format!("{{\"median\":{med:.4},\"min\":{lo:.4},\"max\":{hi:.4},\"n\":{n}}}")
}

/// `chains` 条独立 FMA 累加链，每条 `iters` 次。返回耗时（ns）。
#[inline(never)]
fn fma_chains<const CH: usize>(iters: u64) -> f64 {
    let b = black_box(unsafe { vdupq_n_f32(1e-7) });
    let c = black_box(unsafe { vdupq_n_f32(0.999_9) });
    let mut a = [unsafe { vdupq_n_f32(0.0) }; CH];
    for (k, v) in a.iter_mut().enumerate() {
        *v = black_box(unsafe { vdupq_n_f32(k as f32) });
    }
    let t = Instant::now();
    for _ in 0..iters {
        for v in a.iter_mut() {
            *v = unsafe { vfmaq_f32(*v, b, c) };
        }
    }
    black_box(&a);
    t.elapsed().as_nanos() as f64
}

/// 一条依赖链：a = a·c + b（FMA）；返回每条的 ns
#[inline(never)]
fn fma_latency(iters: u64) -> f64 {
    let b = black_box(unsafe { vdupq_n_f32(1e-7) });
    let c = black_box(unsafe { vdupq_n_f32(0.999_9) });
    let mut a = black_box(unsafe { vdupq_n_f32(1.0) });
    let t = Instant::now();
    for _ in 0..iters {
        a = unsafe { vfmaq_f32(b, a, c) };
    }
    black_box(a);
    t.elapsed().as_nanos() as f64 / iters as f64
}

#[inline(never)]
fn fadd_latency(iters: u64) -> f64 {
    let b = black_box(unsafe { vdupq_n_f32(1e-7) });
    let mut a = black_box(unsafe { vdupq_n_f32(1.0) });
    let t = Instant::now();
    for _ in 0..iters {
        a = unsafe { vaddq_f32(a, b) };
    }
    black_box(a);
    t.elapsed().as_nanos() as f64 / iters as f64
}

/// 标量非线性的依赖延迟：x ← f(x)·k + d，保持在 [-4, 4] 附近
#[inline(never)]
fn nl_latency<F: Fn(f32) -> f32>(iters: u64, f: F) -> f64 {
    let k = black_box(0.9f32);
    let d = black_box(0.1f32);
    let mut x = black_box(0.3f32);
    let t = Instant::now();
    for _ in 0..iters {
        x = f(x) * k + d;
    }
    black_box(x);
    // 减去一次 FMA 的延迟太琐碎，这里不减，报的是「非线性 + 1 次标量乘加」
    t.elapsed().as_nanos() as f64 / iters as f64
}

#[inline(never)]
fn nl_tput<F: Fn(f32) -> f32>(buf: &mut [f32], rounds: usize, f: F) -> f64 {
    let t = Instant::now();
    for _ in 0..rounds {
        for v in buf.iter_mut() {
            *v = f(*v) * 0.9 + 0.1;
        }
        black_box(&mut *buf);
    }
    t.elapsed().as_nanos() as f64 / (rounds * buf.len()) as f64
}

/// 顺序读：每轮 16 次 128 位加载，先两两相加再进 8 个累加器（加法链不成为瓶颈）
#[inline(never)]
fn read_bw(buf: &[f32], passes: usize) -> f64 {
    let p = buf.as_ptr();
    let n = buf.len() / 64 * 64;
    let mut acc = [unsafe { vdupq_n_f32(0.0) }; 8];
    let t = Instant::now();
    for _ in 0..passes {
        let mut i = 0;
        while i < n {
            unsafe {
                for k in 0..8 {
                    let a = vld1q_f32(p.add(i + 8 * k));
                    let b = vld1q_f32(p.add(i + 8 * k + 4));
                    acc[k] = vaddq_f32(acc[k], vaddq_f32(a, b));
                }
            }
            i += 64;
        }
        black_box(&acc);
    }
    let ns = t.elapsed().as_nanos() as f64;
    (passes * n * 4) as f64 / ns // 字节 / ns = GB/s
}

/// 对照读核：16 个累加器，每次加载直接累加（无两两相加），查第一版读带宽偏低是不是核的写法问题
#[inline(never)]
fn read_bw16(buf: &[f32], passes: usize) -> f64 {
    let p = buf.as_ptr();
    let n = buf.len() / 64 * 64;
    let mut acc = [unsafe { vdupq_n_f32(0.0) }; 16];
    let t = Instant::now();
    for _ in 0..passes {
        let mut i = 0;
        while i < n {
            unsafe {
                for (k, a) in acc.iter_mut().enumerate() {
                    *a = vaddq_f32(*a, vld1q_f32(p.add(i + 4 * k)));
                }
            }
            i += 64;
        }
    }
    black_box(&acc);
    let ns = t.elapsed().as_nanos() as f64;
    (passes * n * 4) as f64 / ns
}

#[inline(never)]
fn write_bw(buf: &mut [f32], passes: usize) -> f64 {
    let p = buf.as_mut_ptr();
    let n = buf.len() / 64 * 64;
    let v = black_box(unsafe { vdupq_n_f32(1.0) });
    let t = Instant::now();
    for _ in 0..passes {
        let mut i = 0;
        while i < n {
            unsafe {
                for k in 0..16 {
                    vst1q_f32(p.add(i + 4 * k), v);
                }
            }
            i += 64;
        }
        black_box(&mut *buf);
    }
    let ns = t.elapsed().as_nanos() as f64;
    (passes * n * 4) as f64 / ns
}

// 泛型参数传入，编译器能内联（第一版用函数指针，量到的是调用开销，20261102-peak1 的非线性数作废）
#[inline(always)]
fn exp_fast(x: f32) -> f32 {
    crate::gtcrn_native::exp_fast_pub(x)
}
#[inline(always)]
fn sig_fast(x: f32) -> f32 {
    crate::gtcrn_native::sig_fast_pub(x)
}
#[inline(always)]
fn tanh_libm(x: f32) -> f32 {
    x.tanh()
}
#[inline(always)]
fn exp_libm(x: f32) -> f32 {
    x.exp()
}

/// `hw-peak [--qos rt|ui|bg|default] [--reps N]`：输出一行 JSON
pub fn run(args: &[String]) -> Result<(), crate::Err> {
    let qos = crate::arg_str(args, "--qos", "rt");
    let reps = crate::arg_usize(args, "--reps", 9).max(3);
    crate::set_thread_qos(qos, 16.0)?;
    // 先空转 200 ms，让频率爬上来
    let t = Instant::now();
    let mut s = 0u64;
    while t.elapsed().as_millis() < 200 {
        s = black_box(s.wrapping_add(1));
    }
    const IT: u64 = 2_000_000;
    let tput = |ns: f64, ch: usize| (IT * ch as u64 * 4) as f64 / ns; // f32 乘加 / ns
    let mut best = (0usize, 0.0f64, Vec::new());
    let mut sweep = Vec::new();
    macro_rules! chains {
        ($($c:literal),*) => {$(
            let v: Vec<f64> = (0..reps).map(|_| tput(fma_chains::<$c>(IT), $c)).collect();
            let st = stats(v.clone());
            sweep.push(format!("\"{}\":{}", $c, js(st, reps)));
            if st.0 > best.1 { best = ($c, st.0, v); }
        )*};
    }
    chains!(4, 8, 12, 16, 20, 24, 32);
    let fma_lat = stats((0..reps).map(|_| fma_latency(IT * 4)).collect());
    let fadd_lat = stats((0..reps).map(|_| fadd_latency(IT * 4)).collect());
    let e_lat = stats((0..reps).map(|_| nl_latency(IT, exp_fast)).collect());
    let s_lat = stats((0..reps).map(|_| nl_latency(IT, sig_fast)).collect());
    let t_lat = stats((0..reps).map(|_| nl_latency(IT, tanh_libm)).collect());
    let el_lat = stats((0..reps).map(|_| nl_latency(IT, exp_libm)).collect());
    let mut buf = vec![0.5f32; 4096];
    let e_tput = stats((0..reps).map(|_| nl_tput(&mut buf, 500, exp_fast)).collect());
    let el_tput = stats((0..reps).map(|_| nl_tput(&mut buf, 500, exp_libm)).collect());
    let mut rd = Vec::new();
    let mut rd16 = Vec::new();
    let mut wr = Vec::new();
    for kb in [16usize, 64, 96, 256, 1024, 4096, 8192, 12288, 32768, 65536] {
        let n = kb * 1024 / 4;
        let mut b: Vec<f32> = (0..n).map(|i| (i % 7) as f32).collect();
        let passes = (256 * 1024 * 1024 / (kb * 1024)).max(4);
        let r = stats((0..reps).map(|_| read_bw(&b, passes)).collect());
        let r16 = stats((0..reps).map(|_| read_bw16(&b, passes)).collect());
        rd16.push(format!("\"{kb}KB\":{}", js(r16, reps)));
        let w = stats((0..reps).map(|_| write_bw(&mut b, passes)).collect());
        rd.push(format!("\"{kb}KB\":{}", js(r, reps)));
        wr.push(format!("\"{kb}KB\":{}", js(w, reps)));
    }
    println!(
        "{{\"qos\":\"{qos}\",\"reps\":{reps},\"fma_tput_lanes_per_ns\":{},\"fma_best_chains\":{},\"fma_sweep\":{{{}}},\"fma_lat_ns\":{},\"fadd_lat_ns\":{},\"exp_fast_lat_ns\":{},\"sig_fast_lat_ns\":{},\"tanh_libm_lat_ns\":{},\"exp_libm_lat_ns\":{},\"exp_fast_tput_ns\":{},\"exp_libm_tput_ns\":{},\"read_gbps\":{{{}}},\"read16_gbps\":{{{}}},\"write_gbps\":{{{}}}}}",
        js(stats(best.2), reps),
        best.0,
        sweep.join(","),
        js(fma_lat, reps),
        js(fadd_lat, reps),
        js(e_lat, reps),
        js(s_lat, reps),
        js(t_lat, reps),
        js(el_lat, reps),
        js(e_tput, reps),
        js(el_tput, reps),
        rd.join(","),
        rd16.join(","),
        wr.join(",")
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 微基准本身算得对：FMA 链的结果与标量公式一致；带宽函数读完整个缓冲（不测速度）
    #[test]
    fn kernels_compute_what_they_claim() {
        assert!(fma_latency(1000) > 0.0 && fadd_latency(1000) > 0.0);
        assert!(fma_chains::<4>(1000) > 0.0);
        assert!(nl_latency(1000, exp_fast) > 0.0);
        let buf = vec![1.0f32; 4096];
        assert!(read_bw(&buf, 2) > 0.0);
        let mut w = vec![0.0f32; 4096];
        write_bw(&mut w, 1);
        assert!(w.iter().all(|v| *v == 1.0), "write_bw 要写满缓冲");
        let s = stats(vec![3.0, 1.0, 2.0]);
        assert_eq!(s, (2.0, 1.0, 3.0));
    }
}
