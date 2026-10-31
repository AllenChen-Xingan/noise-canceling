fn rt(buf: &mut Vec<f32>) {
    println!("x");
    let _s = format!("{}", 1);
    let mut v = vec![0.0f32; 4];
    v.push(1.0);
    buf.reserve(4);
    let _b = Box::new(1u8);
    let _m: std::sync::Mutex<u8> = std::sync::Mutex::new(0);
}
fn main() { let mut v = Vec::new(); rt(&mut v); }
