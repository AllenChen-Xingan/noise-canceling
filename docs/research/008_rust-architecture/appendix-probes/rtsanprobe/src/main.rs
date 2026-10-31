use rtsan_standalone::nonblocking;
use std::sync::Mutex;
static M: Mutex<i32> = Mutex::new(0);
#[nonblocking]
fn rt(kind: &str, buf: &mut [f32]) {
    match kind {
        "alloc" => { let v = vec![0.0f32; 256]; buf[0] = v[1]; }
        "lock" => { *M.lock().unwrap() += 1; }
        "sleep" => { std::thread::sleep(std::time::Duration::from_micros(10)); }
        "atomic" => { use std::sync::atomic::*; static A: AtomicU32 = AtomicU32::new(0); A.fetch_add(1, Ordering::Relaxed); }
        _ => { for x in buf.iter_mut() { *x *= 0.5; } }
    }
}
fn main() {
    let k = std::env::args().nth(1).unwrap_or_default();
    let mut b = [1.0f32; 64];
    rt(&k, &mut b);
    println!("ok {k}");
}
