use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    pub fn emscripten_console_log(msg: *const c_char);
}

#[no_mangle]
pub fn on_message(ptr: *mut c_char){
    let c_string = unsafe{ CString::from_raw(ptr) };
    println!("on_message=>{:?}", c_string.to_str());
}

fn main() {
    println!("start...");
    let url = CString::new("你好 asm.js!").unwrap();
    let ptr = url.as_ptr();
    unsafe{emscripten_console_log(ptr); }
}