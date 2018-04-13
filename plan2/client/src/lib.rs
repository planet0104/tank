#[macro_use]
extern crate serde_json;
extern crate tank;
mod game;
use std::cell::RefCell;
use std::mem;

//导入的JS帮助函数
extern "C" {
    pub fn _console_log(text: *const u8, len: usize);
    pub fn _current_time_millis() -> f64;
    pub fn random() -> f64;
    pub fn _request_animation_frame();
    pub fn _window_inner_width() -> i32;
    pub fn _window_inner_height() -> i32;
    pub fn _set_canvas_style_margin(left: i32, top: i32, right: i32, bottom: i32);
    pub fn _set_canvas_style_width(width: i32);
    pub fn _set_canvas_style_height(height: i32);
    pub fn _set_canvas_width(width: i32);
    pub fn _set_canvas_height(height: i32);
    pub fn _set_canvas_font(font: *const u8, len: usize);
    pub fn _load_resource(json: *const u8, len: usize);
    pub fn canvas_offset_left() -> i32;
    pub fn _fill_style(text: *const u8, len: usize);
    pub fn _fill_rect(x: i32, y: i32, width: i32, height: i32);
    pub fn _fill_text(text: *const u8, len: usize, x: i32, y: i32);
    pub fn draw_image_at(res_id: i32, x: i32, y: i32);
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
    );
    pub fn send_message(text: *const u8, len: usize);
    pub fn ready();
}

struct JS {
    request_animation_frame_callback: Option<fn(f64)>,
    on_window_resize_listener: Option<fn()>,
    on_resource_load_listener: Option<fn(num: i32, total: i32)>,
    on_keyup_listener: Option<fn(key: String)>,
    on_keydown_listener: Option<fn(key: String)>,
}

thread_local!{
    static JS: RefCell<JS> = RefCell::new(JS{
        request_animation_frame_callback: None,
        on_window_resize_listener: None,
        on_resource_load_listener: None,
        on_keyup_listener: None,
        on_keydown_listener: None
    });
}

pub fn current_time_millis() -> u64 {
    unsafe { _current_time_millis() as u64 }
}

pub fn console_log(msg: &str) {
    unsafe {
        _console_log(msg.as_ptr(), msg.len());
    }
}

pub fn load_resource(map: serde_json::Value) {
    let json = serde_json::to_string(&map).unwrap();
    unsafe {
        _load_resource(json.as_ptr(), json.len());
    }
}

pub fn window_inner_width() -> i32 {
    unsafe { _window_inner_width() }
}

pub fn window_inner_height() -> i32 {
    unsafe { _window_inner_height() }
}

pub fn fill_style(style: &str){
    unsafe{
        _fill_style(style.as_ptr(), style.len());
    }
}

pub fn fill_rect(x: i32, y: i32, width: i32, height: i32){
    unsafe{
        _fill_rect(x, y, width, height);
    }
}

pub fn fill_text(text: &str, x: i32, y: i32){
    unsafe{
        _fill_text(text.as_ptr(), text.len(), x, y);
    }
}

pub fn set_canvas_font(font: &str){
    unsafe{
        _set_canvas_font(font.as_ptr(), font.len());
    }
}

pub fn set_canvas_style_margin(left: i32, top: i32, right: i32, bottom: i32) {
    unsafe { _set_canvas_style_margin(left, top, right, bottom) };
}
pub fn set_canvas_style_width(width: i32) {
    unsafe { _set_canvas_style_width(width) };
}
pub fn set_canvas_style_height(height: i32) {
    unsafe { _set_canvas_style_height(height) };
}
pub fn set_canvas_width(width: i32) {
    unsafe { _set_canvas_width(width) };
}
pub fn set_canvas_height(height: i32) {
    unsafe { _set_canvas_height(height) };
}

pub fn set_frame_callback(callback: fn(f64)) {
    JS.with(|e| {
        e.borrow_mut().request_animation_frame_callback = Some(callback);
    });
}

pub fn set_on_window_resize_listener(listener: fn()) {
    JS.with(|e| {
        e.borrow_mut().on_window_resize_listener = Some(listener);
    });
}

pub fn set_on_resource_load_listener(listener: fn(num: i32, total: i32)) {
    JS.with(|e| {
        e.borrow_mut().on_resource_load_listener = Some(listener);
    });
}

pub fn set_on_keyup_listener(listener: fn(key: String)) {
    JS.with(|e| {
        e.borrow_mut().on_keyup_listener = Some(listener);
    });
}

pub fn set_on_keydown_listener(listener: fn(key: String)) {
    JS.with(|e| {
        e.borrow_mut().on_keydown_listener = Some(listener);
    });
}

pub fn request_animation_frame() {
    unsafe {
        _request_animation_frame();
    }
}

#[no_mangle]
pub fn request_animation_frame_callback(timestamp: f64) {
    JS.with(|e| {
        if let Some(callback) = e.borrow().request_animation_frame_callback {
            callback(timestamp);
        }
    });
}

#[no_mangle]
pub fn on_window_resize() {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_window_resize_listener {
            callback();
        }
    });
}

#[no_mangle]
pub fn on_resource_load(num: i32, total: i32) {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_resource_load_listener {
            callback(num, total);
        }
    });
}

#[no_mangle]
pub unsafe fn on_keyup_event(key: *mut u8, length: usize) {
    let key = String::from_raw_parts(key, length, length);
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_keyup_listener {
            callback(key);
        }
    });
}

#[no_mangle]
pub unsafe fn on_keydown_event(key: *mut u8, length: usize) {
    let key = String::from_raw_parts(key, length, length);
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_keydown_listener {
            callback(key);
        }
    });
}

#[no_mangle]
pub fn alloc(size: usize) -> *const u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr;
}

#[no_mangle]
pub fn start() {
    game::start();
}
