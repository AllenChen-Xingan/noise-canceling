#[allow(unsafe_code)]
pub fn f()->u8{ unsafe { *(&0u8 as *const u8) } }
