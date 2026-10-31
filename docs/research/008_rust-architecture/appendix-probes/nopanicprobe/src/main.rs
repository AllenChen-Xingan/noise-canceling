use no_panic::no_panic;
#[cfg(feature_bad)]
#[no_panic]
#[inline(never)]
fn gain_bad(buf: &mut [f32], g: &[f32]) { for i in 0..buf.len() { buf[i] *= g[i]; } }
#[no_panic]
#[inline(never)]
fn gain_ok(buf: &mut [f32], g: &[f32]) { for (x, y) in buf.iter_mut().zip(g) { *x *= *y; } }
fn main() { let n: usize = std::env::args().count() * 100;
    let mut b = vec![1.0f32; 480]; let g = vec![0.5f32; n];
    gain_ok(&mut b, &g);
    #[cfg(feature_bad)] gain_bad(&mut b, &g);
    println!("{}", b[0]);
}
