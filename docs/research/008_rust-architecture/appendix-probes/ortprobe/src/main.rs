// 探针：GTCRN 流式 ONNX 在 ort rc.15 下，稳态每帧是否分配 / 调阻塞函数。
use ort::{session::Session, value::{Tensor, TensorRef}, AsPointer};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering::Relaxed};
use std::time::Instant;

struct Count;
static N: AtomicUsize = AtomicUsize::new(0);
static ON: AtomicBool = AtomicBool::new(false);
unsafe impl GlobalAlloc for Count {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 { if ON.load(Relaxed) { N.fetch_add(1, Relaxed); } System.alloc(l) }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) { if ON.load(Relaxed) { N.fetch_add(1, Relaxed); } System.dealloc(p, l) }
}
#[global_allocator] static A: Count = Count;

const SH: [&[usize]; 4] = [&[1,257,1,2], &[2,1,16,16,33], &[2,3,1,1,16], &[2,1,33,16]];
const IN: [&str; 4] = ["mix","conv_cache","tra_cache","inter_cache"];
const OUT: [&str; 4] = ["enh","conv_cache_out","tra_cache_out","inter_cache_out"];

#[rtsan_standalone::nonblocking]
fn rt_binding(s: &mut Session, b: &ort::session::IoBinding) { warm_binding(s, b) }
fn warm_binding(s: &mut Session, b: &ort::session::IoBinding) { let o = s.run_binding(b).unwrap(); drop(o); }
#[rtsan_standalone::nonblocking]
unsafe fn rt_raw(s: *mut ort::sys::OrtSession, b: *const ort::sys::OrtIoBinding) { warm_raw(s, b) }
unsafe fn warm_raw(s: *mut ort::sys::OrtSession, b: *const ort::sys::OrtIoBinding) {
    let f = ort::api().RunWithBinding; let st = f(s, std::ptr::null(), b); if !st.0.is_null() { std::process::abort() }
}
#[rtsan_standalone::nonblocking]
fn rt_plain(s: &mut Session, ins: &[Vec<f32>; 4], outs: &mut [Vec<f32>; 4]) {
    let i0 = TensorRef::from_array_view((SH[0].to_vec(), &ins[0][..])).unwrap();
    let i1 = TensorRef::from_array_view((SH[1].to_vec(), &ins[1][..])).unwrap();
    let i2 = TensorRef::from_array_view((SH[2].to_vec(), &ins[2][..])).unwrap();
    let i3 = TensorRef::from_array_view((SH[3].to_vec(), &ins[3][..])).unwrap();
    let o = s.run(ort::inputs!["mix"=>i0,"conv_cache"=>i1,"tra_cache"=>i2,"inter_cache"=>i3]).unwrap();
    for k in 0..4 { let (_, d) = o[OUT[k]].try_extract_tensor::<f32>().unwrap(); outs[k].copy_from_slice(d); }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let model = &a[1]; let mode = a[2].as_str(); let frames: usize = a[3].parse().unwrap();
    let env = ort::init().build().unwrap();
    let mut s = Session::builder(&env).unwrap()
        .with_intra_threads(1).unwrap().with_inter_threads(1).unwrap()
        .with_intra_op_spinning(false).unwrap().with_memory_pattern(true).unwrap()
        .commit_from_file(model).unwrap();
    let mut rng = 1u32; let mut rnd = || { rng = rng.wrapping_mul(1664525).wrapping_add(1013904223); (rng >> 9) as f32 / (1u32<<23) as f32 - 0.5 };
    let mut t = Vec::with_capacity(frames);
    let warm = 200;
    if mode == "plain" {
        let mut ins: [Vec<f32>;4] = std::array::from_fn(|k| vec![0.0; SH[k].iter().product()]);
        let mut outs: [Vec<f32>;4] = std::array::from_fn(|k| vec![0.0; if k==0 {514} else {SH[k].iter().product()}]);
        for f in 0..warm+frames {
            for x in ins[0].iter_mut() { *x = rnd() * 0.1; }
            if f == warm { N.store(0, Relaxed); ON.store(true, Relaxed); }
            let t0 = Instant::now(); rt_plain(&mut s, &ins, &mut outs); let dt = t0.elapsed();
            if f >= warm { t.push(dt.as_secs_f64()*1e3); }
            for k in 1..4 { let (a, b) = (&mut ins, &outs); a[k].copy_from_slice(&b[k]); }
        }
    } else {
        let alloc = s.allocator();
        let mut ti: Vec<Tensor<f32>> = (0..4).map(|k| Tensor::<f32>::new(alloc, SH[k].to_vec()).unwrap()).collect();
        let to: Vec<Tensor<f32>> = (0..4).map(|k| Tensor::<f32>::new(alloc, SH[k].to_vec()).unwrap()).collect();
        for k in 0..4 { ti[k].extract_tensor_mut().1.fill(0.0); }
        let mut b = s.create_binding().unwrap();
        for k in 0..4 { b.bind_input(IN[k], &ti[k]).unwrap(); }
        let mut to = to;
        for k in 0..4 { b.bind_output(OUT[k], &mut to[k]).unwrap(); }
        let mut enh_sum_prev = f32::NAN;
        for f in 0..warm+frames {
            { let d = ti[0].extract_tensor_mut().1; for x in d.iter_mut() { *x = rnd() * 0.1; } }
            if f == warm { N.store(0, Relaxed); ON.store(true, Relaxed); }
            let t0 = Instant::now();
            if f < warm { if mode == "binding" { warm_binding(&mut s, &b); } else { unsafe { warm_raw(s.ptr_mut(), b.ptr()); } } }
            else if mode == "binding" { rt_binding(&mut s, &b); } else { unsafe { rt_raw(s.ptr_mut(), b.ptr()); } }
            let dt = t0.elapsed();
            if f >= warm { t.push(dt.as_secs_f64()*1e3); }
            ON.store(false, Relaxed);
            for k in 1..4 { let src: Vec<f32> = to[k].extract_tensor().1.to_vec(); ti[k].extract_tensor_mut().1.copy_from_slice(&src); }
            if f == warm-1 { enh_sum_prev = to[0].extract_tensor().1.iter().sum(); }
            if f == warm { let s2: f32 = to[0].extract_tensor().1.iter().sum(); eprintln!("[check] enh changes w/o rebind: {}", s2 != enh_sum_prev); }
            if f >= warm { ON.store(true, Relaxed); }
        }
    }
    ON.store(false, Relaxed);
    let allocs = N.load(Relaxed);
    t.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let q = |p: f64| t[((t.len() as f64 - 1.0) * p) as usize];
    println!("mode={mode} frames={frames} rust_alloc+dealloc_in_window={allocs} per_frame={:.2} ms p50={:.3} p99={:.3} p99.9={:.3} max={:.3}", allocs as f64/frames as f64, q(0.5), q(0.99), q(0.999), t[t.len()-1]);
}
