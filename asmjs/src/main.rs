extern crate tank;
mod client;
use tank::engine::CanvasContext;
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
#[macro_use]
extern crate lazy_static;

//导入的JS帮助函数
extern "C" {
    pub fn emscripten_prompt(title: *const c_char, default_msg: *const c_char)->*mut c_char;
    pub fn emscripten_current_time_millis()->f64;
    pub fn emscripten_alert(text: *const c_char);
    pub fn emscripten_console_log(text: *const c_char);
    pub fn emscripten_random() -> f64;
    pub fn emscripten_request_animation_frame();
    pub fn emscripten_window_inner_width() -> i32;
    pub fn emscripten_window_inner_height() -> i32;
    pub fn emscripten_set_canvas_style_margin(left: i32, top: i32, right: i32, bottom: i32);
    pub fn emscripten_set_canvas_style_width(width: i32);
    pub fn emscripten_set_canvas_style_height(height: i32);
    pub fn emscripten_set_canvas_width(width: i32);
    pub fn emscripten_set_canvas_height(height: i32);
    pub fn emscripten_set_canvas_font(font: *const c_char);
    pub fn emscripten_load_resource(json: *const c_char);
    pub fn emscripten_fill_style(text: *const c_char);
    pub fn emscripten_fill_rect(x: i32, y: i32, width: i32, height: i32);
    pub fn emscripten_fill_text(text: *const c_char, x: i32, y: i32);
    pub fn emscripten_draw_image_at(res_id: i32, x: i32, y: i32);
    pub fn emscripten_draw_image(
        res_id: i32,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    );
    pub fn emscripten_send_message(text: *const c_char);
    pub fn emscripten_connect(url: *const c_char);
}

lazy_static! {
    static ref JSENV: Arc<Mutex<JS>> = Arc::new(Mutex::new(JS{
        request_animation_frame_callback: None,
        on_window_resize_listener: None,
        on_resource_load_listener: None,
        on_keyup_listener: None,
        on_keydown_listener: None,
        on_connect_listener: None,
        on_close_listener: None,
        on_message_listener: None,
        //on_prompt_listener: None,
    }));
}

// static mut JS:*const JS = ptr::null_mut();

// fn js<'a>() -> &'a mut JS {
//     unsafe {
//         if js..is_null() {
//             JS = transmute(Box::new(JS{
//                 request_animation_frame_callback: None,
//                 on_window_resize_listener: None,
//                 on_resource_load_listener: None,
//                 on_keyup_listener: None,
//                 on_keydown_listener: None,
//                 on_connect_listener: None,
//                 on_close_listener: None,
//                 on_message_listener: None,
//                 //on_prompt_listener: None,
//             }));
//         }
//         transmute(JS)
//     }
// }

struct JS {
    request_animation_frame_callback: Option<fn(f64)>,
    on_window_resize_listener: Option<fn()>,
    on_resource_load_listener: Option<fn(num: i32, total: i32)>,
    on_keyup_listener: Option<fn(key: &str)>,
    on_keydown_listener: Option<fn(key: &str)>,
    on_connect_listener: Option<fn()>,
    on_close_listener: Option<fn()>,
    on_message_listener: Option<fn(msg: &str)>,
    //on_prompt_listener: Option<fn(msg: &str)>,
}

// thread_local!{
//     static JS: RefCell<JS> = RefCell::new(JS{
//         request_animation_frame_callback: None,
//         on_window_resize_listener: None,
//         on_resource_load_listener: None,
//         on_keyup_listener: None,
//         on_keydown_listener: None,
//         on_connect_listener: None,
//         on_close_listener: None,
//         on_message_listener: None,
//     });
// }

pub fn random() -> f64 {
    unsafe{
        emscripten_random()
    }
}

pub fn console_log(msg: &str) {
    unsafe {
        if let Ok(string) = CString::new(msg){
            emscripten_console_log(string.as_ptr());
        }
    }
}

pub fn current_time_millis()->u64{
    unsafe{
        emscripten_current_time_millis() as u64
    }
}

pub fn alert(msg: &str) {
    unsafe {
        if let Ok(string) = CString::new(msg){
            emscripten_alert(string.as_ptr());
        }
    }
}

pub fn load_resource(json: String) {
    unsafe {
        if let Ok(string) = CString::new(json){
            emscripten_load_resource(string.as_ptr());
        }
    }
}

pub fn window_inner_width() -> i32 {
    unsafe { emscripten_window_inner_width() }
}

pub fn window_inner_height() -> i32 {
    unsafe { emscripten_window_inner_height() }
}

pub fn fill_style(style: &str) {
    unsafe {
        if let Ok(string) = CString::new(style){
            emscripten_fill_style(string.as_ptr());
        }
    }
}

pub fn fill_rect(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        emscripten_fill_rect(x, y, width, height);
    }
}

pub fn fill_text(text: &str, x: i32, y: i32) {
    unsafe {
        if let Ok(string) = CString::new(text){
            emscripten_fill_text(string.as_ptr(), x, y);
        }
    }
}

pub fn set_canvas_font(font: &str) {
    unsafe {
        if let Ok(string) = CString::new(font){
            emscripten_set_canvas_font(string.as_ptr());
        }
    }
}

pub fn send_message(msg: &str) {
    unsafe {
        if let Ok(string) = CString::new(msg){
            emscripten_send_message(string.as_ptr());
        }
    }
}

pub fn connect(url: &str) {
    unsafe {
        if let Ok(string) = CString::new(url){
            emscripten_connect(string.as_ptr());
        }
    }
}

pub fn draw_image_at(res_id: i32, x: i32, y: i32) {
    unsafe {
        emscripten_draw_image_at(res_id, x, y);
    }
}
pub fn draw_image(
    res_id: i32,
    source_x: i32,
    source_y: i32,
    source_width: i32,
    source_height: i32,
    dest_x: i32,
    dest_y: i32,
    dest_width: i32,
    dest_height: i32,
) {
    unsafe {
        emscripten_draw_image(
            res_id,
            source_x,
            source_y,
            source_width,
            source_height,
            dest_x,
            dest_y,
            dest_width,
            dest_height,
        );
    }
}

pub fn set_canvas_style_margin(left: i32, top: i32, right: i32, bottom: i32) {
    unsafe { emscripten_set_canvas_style_margin(left, top, right, bottom) };
}
pub fn set_canvas_style_width(width: i32) {
    unsafe { emscripten_set_canvas_style_width(width) };
}
pub fn set_canvas_style_height(height: i32) {
    unsafe { emscripten_set_canvas_style_height(height) };
}
pub fn set_canvas_width(width: i32) {
    unsafe { emscripten_set_canvas_width(width) };
}
pub fn set_canvas_height(height: i32) {
    unsafe { emscripten_set_canvas_height(height) };
}

pub fn prompt(title:&str, default_msg:&str)->String{
    if let Ok(title) = CString::new(title){
        if let Ok(msg) = CString::new(default_msg){
            let c_string = unsafe{ CString::from_raw(emscripten_prompt(title.as_ptr(), msg.as_ptr())) };
            return String::from(c_string.to_str().unwrap_or(""));
        }
    }
    String::new()
}

pub fn set_frame_callback(callback: fn(f64)) {
    if let Ok(mut js) = JSENV.lock(){
        js.request_animation_frame_callback = Some(callback);
    }
}

// pub fn set_on_prompt_listener(listener: fn(&str)) {
//     js..on_prompt_listener = Some(listener);
// }

pub fn set_on_window_resize_listener(listener: fn()) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_window_resize_listener = Some(listener);
    }
}

pub fn set_on_connect_listener(listener: fn()) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_connect_listener = Some(listener);
    }
}

pub fn set_on_close_listener(listener: fn()) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_close_listener = Some(listener);
    }
}

pub fn set_on_resource_load_listener(listener: fn(num: i32, total: i32)) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_resource_load_listener = Some(listener);
    }
}

pub fn set_on_keyup_listener(listener: fn(key: &str)) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_keyup_listener = Some(listener);
    }else{
        console_log("JSENV.lock()失败.")
    }
}

pub fn set_on_keydown_listener(listener: fn(key: &str)) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_keydown_listener = Some(listener);
    }else{
        console_log("JSENV.lock()失败.")
    }
}

pub fn set_on_message_listener(listener: fn(msg: &str)) {
    if let Ok(mut js) = JSENV.lock(){
        js.on_message_listener = Some(listener);
    }
}


pub fn request_animation_frame() {
    unsafe {
        emscripten_request_animation_frame();
    }
}

#[no_mangle]
pub fn request_animation_frame_callback(timestamp: f64) {
    if let Ok(js) = JSENV.lock(){
        if let Some(callback) = js.request_animation_frame_callback {
            callback(timestamp);
        }
    }
}

#[no_mangle]
pub fn on_window_resize() {
    if let Ok(js) = JSENV.lock(){
        if let Some(callback) = js.on_window_resize_listener {
            callback();
        }
    }
}

#[no_mangle]
pub fn on_resource_load(num: i32, total: i32) {
    if let Ok(js) = JSENV.lock(){
        if let Some(callback) = js.on_resource_load_listener {
            callback(num, total);
        }
    }
}

#[no_mangle]
pub fn on_connect() {
    if let Ok(js) = JSENV.lock(){
        if let Some(callback) = js.on_connect_listener {
            callback();
        }
    }
}

#[no_mangle]
pub fn on_close() {
    if let Ok(js) = JSENV.lock(){
        if let Some(callback) = js.on_close_listener {
            callback();
        }
    }
}

#[no_mangle]
pub fn on_message(msg: *mut c_char) {
    console_log("on_message>>>110");
    let c_string = unsafe{ CString::from_raw(msg) };
    console_log("on_message>>>111");
    let js_lock= JSENV.lock();
    console_log("on_message>>>112");
    if let Ok(js) = js_lock{
        if let Some(callback) = js.on_message_listener {
            if let Ok(string) = c_string.to_str(){
                callback(string);
            }else{
                console_log("CString to str失败")
            }
        }else{
            console_log("没有callback.")
        }
    }else{
        console_log(&format!("JSENV.lock()失败. {:?}", js_lock.err()));
    }
}

#[no_mangle]
pub fn on_keyup_event(key: *mut c_char) {
    let key = unsafe{ CString::from_raw(key) };
    console_log(&format!("on_keyup_event>>>{:?}", key));
    let js_lock= JSENV.lock();
    if let Ok(js) = js_lock{
        if let Some(callback) = js.on_keyup_listener {
            if let Ok(key) = key.to_str(){
                callback(key);
            }else{
                console_log("CString to str失败")
            }
        }else{
            console_log("没有callback.")
        }
    }else{
        console_log(&format!("JSENV.lock()失败. {:?}", js_lock.err()));
    }
}

#[no_mangle]
pub fn on_keydown_event(key: *mut c_char) {
    let key = unsafe{ CString::from_raw(key) };
    console_log(&format!("on_keydown_event>>>{:?}", key));
    let js_lock= JSENV.lock();
    if let Ok(js) = js_lock{
        if let Some(callback) = js.on_keydown_listener {
            if let Ok(key) = key.to_str(){
                callback(key);
            }else{
                console_log("CString to str失败")
            }
        }else{
            console_log("没有callback.")
        }
    }else{
        console_log(&format!("JSENV.lock()失败. {:?}", js_lock.err()));
    }
}

// #[no_mangle]
// pub fn on_prompt(value: *mut c_char) {
//     let c_string = unsafe{ CString::from_raw(value) };
//     if let Some(callback) = js..on_message_listener {
//         callback(c_string.to_str().unwrap());
//     }
// }


#[no_mangle]
pub fn start() {
    client::start();
}

pub struct Context2D {}

impl CanvasContext for Context2D {
    fn draw_image_at(&self, res_id: i32, x: i32, y: i32) {
        draw_image_at(res_id, x, y);
    }

    fn draw_image(
        &self,
        res_id: i32,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    ) {
        draw_image(
            res_id,
            source_x,
            source_y,
            source_width,
            source_height,
            dest_x,
            dest_y,
            dest_width,
            dest_height,
        );
    }

    fn fill_style(&self, style: &str) {
        fill_style(style);
    }

    fn set_canvas_font(&self, font: &str) {
        set_canvas_font(font);
    }

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        fill_rect(x, y, width, height);
    }

    fn fill_text(&self, text: &str, x: i32, y: i32) {
        fill_text(text, x, y);
    }
}

fn main(){
    println!("main.");
    // let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    // let elapsed = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
    // println!("elapsed={}", elapsed);
}