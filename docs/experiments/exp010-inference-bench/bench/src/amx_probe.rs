//! 小矩阵上 Accelerate（`cblas_sgemm`，在 Apple Silicon 上走 AMX / SME 矩阵单元）对手写循环的每次调用耗时。
//! 形状取自 GTCRN 每帧真实用到的矩阵乘（C[M×N] = A[M×K]·B[K×N]，行主序）。
//! 只回答「GTCRN 这个量级值不值得把矩阵乘交给 Accelerate」，不进主基准。输出每个形状一行 JSON。

use std::time::Instant;

#[link(name = "Accelerate", kind = "framework")]
unsafe extern "C" {
    fn cblas_sgemm(
        order: i32,
        ta: i32,
        tb: i32,
        m: i32,
        n: i32,
        k: i32,
        alpha: f32,
        a: *const f32,
        lda: i32,
        b: *const f32,
        ldb: i32,
        beta: f32,
        c: *mut f32,
        ldc: i32,
    );
}
unsafe extern "C" {
    // vDSP：C[M×N] = A[M×P]·B[P×N]，行主序，步长 1
    fn vDSP_mmul(
        a: *const f32,
        ia: isize,
        b: *const f32,
        ib: isize,
        c: *mut f32,
        ic: isize,
        m: usize,
        n: usize,
        p: usize,
    );
}
const ROW_MAJOR: i32 = 101;
const NO_TRANS: i32 = 111;

fn hand(a: &[f32], b: &[f32], c: &mut [f32], m: usize, n: usize, k: usize) {
    // 与 gtcrn_native::Point::run 同写法：逐输出行，内层沿 N 连续累加（可自动向量化）
    for i in 0..m {
        let ci = &mut c[i * n..(i + 1) * n];
        ci.fill(0.0);
        for kk in 0..k {
            let w = a[i * k + kk];
            for (o, &x) in ci.iter_mut().zip(&b[kk * n..(kk + 1) * n]) {
                *o += w * x;
            }
        }
    }
}

/// 调一次 f，返回这一次里全进程 malloc 的次数（malloc_logger 钩子）
fn allocs_of(f: &mut dyn FnMut()) -> u64 {
    let a = crate::proc_allocs();
    f();
    crate::proc_allocs() - a
}

fn print_stacks() {
    use std::sync::atomic::Ordering::Relaxed;
    let n = (crate::STACK_COUNT.load(Relaxed) as usize).min(crate::STACK_N);
    for i in 0..n {
        let st = unsafe { (*std::ptr::addr_of!(crate::STACKS))[i] };
        let mut names = Vec::new();
        for &pc in st.iter().take_while(|p| **p != 0) {
            let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
            let name = if unsafe { libc::dladdr(pc as *const _, &mut info) } != 0
                && !info.dli_sname.is_null()
            {
                unsafe { std::ffi::CStr::from_ptr(info.dli_sname) }
                    .to_string_lossy()
                    .into_owned()
            } else {
                format!("{pc:#x}")
            };
            let lib = if info.dli_fname.is_null() {
                String::new()
            } else {
                unsafe { std::ffi::CStr::from_ptr(info.dli_fname) }
                    .to_string_lossy()
                    .rsplit('/')
                    .next()
                    .unwrap_or("")
                    .to_string()
            };
            names.push(format!("{name}@{lib}").replace('"', "'"));
        }
        println!(
            "{{\"probe\":\"sgemm_malloc_stack\",\"i\":{i},\"frames\":{:?}}}",
            names
        );
    }
}

pub fn run() {
    crate::install_malloc_counter();
    // (名字, M, N, K, 每帧调用次数)
    let shapes: [(&str, usize, usize, usize, usize); 5] = [
        ("point_conv1 24→16 @33", 16, 33, 24, 6),
        ("point_conv2 16→8 @33", 8, 33, 16, 6),
        ("dpgrnn_fc 16→16 ×33 频点", 33, 16, 16, 4),
        ("en0 im2col 45→16 @65", 16, 65, 45, 1),
        ("gru_in_proj 8→12 ×33 频点", 33, 12, 8, 8),
    ];
    let reps = 200_000;
    for (name, m, n, k, per_frame) in shapes {
        let a: Vec<f32> = (0..m * k)
            .map(|i| ((i * 7919) % 101) as f32 / 101.0 - 0.5)
            .collect();
        let b: Vec<f32> = (0..k * n)
            .map(|i| ((i * 104_729) % 97) as f32 / 97.0 - 0.5)
            .collect();
        let mut c1 = vec![0.0f32; m * n];
        let mut c2 = vec![0.0f32; m * n];
        let mut time = |f: &mut dyn FnMut()| {
            for _ in 0..2000 {
                f();
            }
            let t = Instant::now();
            for _ in 0..reps {
                f();
            }
            t.elapsed().as_secs_f64() * 1e9 / reps as f64
        };
        let ns_hand = time(&mut || {
            hand(std::hint::black_box(&a), &b, &mut c1, m, n, k);
            std::hint::black_box(&c1);
        });
        let ns_blas = time(&mut || {
            unsafe {
                cblas_sgemm(
                    ROW_MAJOR,
                    NO_TRANS,
                    NO_TRANS,
                    m as i32,
                    n as i32,
                    k as i32,
                    1.0,
                    std::hint::black_box(a.as_ptr()),
                    k as i32,
                    b.as_ptr(),
                    n as i32,
                    0.0,
                    c2.as_mut_ptr(),
                    n as i32,
                )
            };
            std::hint::black_box(&c2);
        });
        let mut c3 = vec![0.0f32; m * n];
        let ns_vdsp = time(&mut || {
            unsafe {
                vDSP_mmul(
                    std::hint::black_box(a.as_ptr()),
                    1,
                    b.as_ptr(),
                    1,
                    c3.as_mut_ptr(),
                    1,
                    m,
                    n,
                    k,
                )
            };
            std::hint::black_box(&c3);
        });
        let al_hand = allocs_of(&mut || hand(&a, &b, &mut c1, m, n, k));
        let al_sgemm = allocs_of(&mut || unsafe {
            cblas_sgemm(
                ROW_MAJOR,
                NO_TRANS,
                NO_TRANS,
                m as i32,
                n as i32,
                k as i32,
                1.0,
                a.as_ptr(),
                k as i32,
                b.as_ptr(),
                n as i32,
                0.0,
                c2.as_mut_ptr(),
                n as i32,
            )
        });
        let al_vdsp = allocs_of(&mut || unsafe {
            vDSP_mmul(a.as_ptr(), 1, b.as_ptr(), 1, c3.as_mut_ptr(), 1, m, n, k)
        });
        if al_sgemm > 0 && crate::STACK_COUNT.load(std::sync::atomic::Ordering::Relaxed) == 0 {
            crate::STACK_ARMED.store(true, std::sync::atomic::Ordering::Relaxed);
            unsafe {
                cblas_sgemm(
                    ROW_MAJOR,
                    NO_TRANS,
                    NO_TRANS,
                    m as i32,
                    n as i32,
                    k as i32,
                    1.0,
                    a.as_ptr(),
                    k as i32,
                    b.as_ptr(),
                    n as i32,
                    0.0,
                    c2.as_mut_ptr(),
                    n as i32,
                )
            };
            crate::STACK_ARMED.store(false, std::sync::atomic::Ordering::Relaxed);
        }
        let vdsp_diff = c1
            .iter()
            .zip(&c3)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        let max_diff = c1
            .iter()
            .zip(&c2)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max);
        println!(
            "{{\"probe\":\"amx\",\"shape\":\"{name}\",\"m\":{m},\"n\":{n},\"k\":{k},\"per_frame\":{per_frame},\"hand_ns\":{ns_hand:.1},\"accelerate_ns\":{ns_blas:.1},\"per_frame_hand_us\":{:.2},\"per_frame_accelerate_us\":{:.2},\"max_abs_diff\":{max_diff:.2e},\"vdsp_ns\":{ns_vdsp:.1},\"per_frame_vdsp_us\":{:.2},\"vdsp_max_abs_diff\":{vdsp_diff:.2e},\"allocs_per_call_hand\":{al_hand},\"allocs_per_call_sgemm\":{al_sgemm},\"allocs_per_call_vdsp\":{al_vdsp}}}",
            ns_hand * per_frame as f64 / 1e3,
            ns_blas * per_frame as f64 / 1e3,
            ns_vdsp * per_frame as f64 / 1e3
        );
    }
    print_stacks();
}
