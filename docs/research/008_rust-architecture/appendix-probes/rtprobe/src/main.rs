use assert_no_alloc::*;
#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

#[inline(never)]
fn rt(v: &mut Vec<f32>) { v.push(1.0); }

#[cfg(target_arch = "aarch64")]
fn fpcr() -> u64 { let r: u64; unsafe { core::arch::asm!("mrs {}, fpcr", out(reg) r, options(nomem, nostack, preserves_flags)); } r }
#[cfg(target_arch = "aarch64")]
fn set_fpcr(v: u64) { unsafe { core::arch::asm!("msr fpcr, {}", in(reg) v, options(nomem, nostack, preserves_flags)); } }

fn denorm_bench(label: &str) {
    let mut x = vec![1.0e-38f32; 4096];
    let t = std::time::Instant::now();
    for _ in 0..2000 { for v in x.iter_mut() { *v = std::hint::black_box(*v) * 0.5 + 1.0e-45; } }
    println!("{label}: {:?} sample={:e}", t.elapsed(), x[0]);
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "ftz" => {
            println!("fpcr before = {:#x}", fpcr());
            denorm_bench("no-FTZ");
            set_fpcr(fpcr() | (1 << 24));
            println!("fpcr after  = {:#x}", fpcr());
            denorm_bench("FTZ");
        }
        "alloc" => { let mut v = Vec::with_capacity(0); assert_no_alloc(|| rt(&mut v)); println!("survived, len={}", v.len()); }
        "ffi" => { extern "C" fn cb() { panic!("boom"); } cb(); }
        _ => {}
    }
}
