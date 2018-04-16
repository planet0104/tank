#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate stdweb;
extern crate tank;
mod game;
use std::cell::RefCell;
use std::mem;
use tank::engine::CanvasContext;

struct JS {
    request_animation_frame_callback: Option<fn(f64)>,
    on_window_resize_listener: Option<fn()>,
    on_resource_load_listener: Option<fn(num: i32, total: i32)>,
    on_keyup_listener: Option<fn(key: String)>,
    on_keydown_listener: Option<fn(key: String)>,
    on_connect_listener: Option<fn()>,
    on_close_listener: Option<fn()>,
    on_message_listener: Option<fn(msg: String)>,
}

thread_local!{
    static JS: RefCell<JS> = RefCell::new(JS{
        request_animation_frame_callback: None,
        on_window_resize_listener: None,
        on_resource_load_listener: None,
        on_keyup_listener: None,
        on_keydown_listener: None,
        on_connect_listener: None,
        on_close_listener: None,
        on_message_listener: None,
    });
}

pub fn random() -> f64 {
    js!(_random())
}

pub fn current_time_millis() -> u64 {
    js!(_current_time_millis())
}

pub fn console_log(msg: &str) {
    js!(_console_log(msg.as_ptr(), msg.len()));
}

pub fn load_resource(map: serde_json::Value) {
    let json = serde_json::to_string(&map).unwrap();
    js!(_load_resource(json.as_ptr(), json.len()));
}

pub fn send_json_message(json: serde_json::Value) {
    let json = serde_json::to_string(&json).unwrap();
    send_message(&json);
}

pub fn window_inner_width() -> i32 {
    js!(_window_inner_width())
}

pub fn window_inner_height() -> i32 {
    js!(_window_inner_height())
}

pub fn fill_style(style: &str) {
    js!(_fill_style(style.as_ptr(), style.len()));
}

pub fn fill_rect(x: i32, y: i32, width: i32, height: i32) {
    js!(_fill_rect(x, y, width, height));
}

pub fn fill_text(text: &str, x: i32, y: i32) {
    js!(_fill_text(text.as_ptr(), text.len(), x, y));
}

pub fn set_canvas_font(font: &str) {
    js!(_set_canvas_font(font.as_ptr(), font.len()));
}

pub fn send_message(msg: &str) {
    js!(_send_message(msg.as_ptr(), msg.len()));
}

pub fn connect(url: &str) {
    js!(_connect(url.as_ptr(), url.len()));
}

pub fn draw_image_at(res_id: i32, x: i32, y: i32) {
    js!(_draw_image_at(res_id, x, y));
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
    js!(_draw_image(
            res_id,
            source_x,
            source_y,
            source_width,
            source_height,
            dest_x,
            dest_y,
            dest_width,
            dest_height,
        ));
}

pub fn set_canvas_style_margin(left: i32, top: i32, right: i32, bottom: i32) {
    js!(_set_canvas_style_margin(left, top, right, bottom));
}
pub fn set_canvas_style_width(width: i32) {
    js!(_set_canvas_style_width(width));
}
pub fn set_canvas_style_height(height: i32) {
    js!(_set_canvas_style_height(height));
}
pub fn set_canvas_width(width: i32) {
    js!(_set_canvas_width(width));
}
pub fn set_canvas_height(height: i32) {
    js!(_set_canvas_height(height));
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

pub fn set_on_connect_listener(listener: fn()) {
    JS.with(|e| {
        e.borrow_mut().on_connect_listener = Some(listener);
    });
}

pub fn set_on_close_listener(listener: fn()) {
    JS.with(|e| {
        e.borrow_mut().on_close_listener = Some(listener);
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

pub fn set_on_message_listener(listener: fn(msg: String)) {
    JS.with(|e| {
        e.borrow_mut().on_message_listener = Some(listener);
    });
}


pub fn request_animation_frame() {
    js!(_request_animation_frame());
}

#[no_mangle]
pub extern fn request_animation_frame_callback(timestamp: f64) {
    JS.with(|e| {
        if let Some(callback) = e.borrow().request_animation_frame_callback {
            callback(timestamp);
        }
    });
}

#[no_mangle]
pub extern fn on_window_resize() {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_window_resize_listener {
            callback();
        }
    });
}

#[no_mangle]
pub extern fn on_resource_load(num: i32, total: i32) {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_resource_load_listener {
            callback(num, total);
        }
    });
}

#[no_mangle]
pub extern fn on_connect() {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_connect_listener {
            callback();
        }
    });
}

#[no_mangle]
pub extern fn on_close() {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_close_listener {
            callback();
        }
    });
}

#[no_mangle]
pub extern fn on_message(msg: *mut u8, length: usize) {
    let msg = unsafe { String::from_raw_parts(msg, length, length) };
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_message_listener {
            callback(msg);
        }
    });
}

#[no_mangle]
pub extern fn on_keyup_event(key: *mut u8, length: usize) {
    let key = unsafe { String::from_raw_parts(key, length, length) };
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_keyup_listener {
            callback(key);
        }
    });
}

#[no_mangle]
pub extern fn on_keydown_event(key: *mut u8, length: usize) {
    let key = unsafe { String::from_raw_parts(key, length, length) };
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_keydown_listener {
            callback(key);
        }
    });
}

#[no_mangle]
pub extern fn alloc(size: usize) -> *const u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr;
}

#[no_mangle]
pub extern fn start() {
    game::start();
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

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        fill_rect(x, y, width, height);
    }

    fn fill_text(&self, text: &str, x: i32, y: i32) {
        fill_text(text, x, y);
    }
}

fn main(){
    println!("hello!");
}