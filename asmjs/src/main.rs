extern crate tank;
#[macro_use]
extern crate lazy_static;
use tank::engine::GameContext;
use std::ffi::CString;
use std::os::raw::c_char;
use std::cell::RefCell;
use tank::GAME;
use tank::KeyEvent;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

//导入的JS帮助函数
#[link("-s"= "ASSERTIONS=1")]
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
    
    pub fn emscripten_stroke_style(text: *const c_char);
    pub fn emscripten_line_width(width:i32);
    pub fn emscripten_stroke_rect(x: i32, y: i32, width: i32, height: i32);
    pub fn emscripten_draw_image_repeat_y(resId: i32, x: i32, y: i32, width: i32, height: i32);
    pub fn emscripten_draw_image_repeat_x(resId: i32, x:i32 , y: i32, width: i32, height: i32);
    pub fn emscripten_draw_image_repeat(resId: i32, x:i32 , y: i32, width: i32, height: i32);

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
    pub fn emscripten_send_binary_message(data: *const u8, len: usize);
    pub fn emscripten_connect(url: *const c_char);
}

struct JS {
    request_animation_frame_callback: Option<fn(f64)>,
    on_window_resize_listener: Option<fn()>,
    on_resource_load_listener: Option<fn(num: i32, total: i32)>,
    on_connect_listener: Option<fn()>,
    on_close_listener: Option<fn()>,
    // on_message_listener: Option<fn(msg: &str)>,
    // on_key_up_listener: Option<fn(key: i32)>,
    // on_key_down_listener: Option<fn(key: i32)>,
}

lazy_static! {
    static ref KEY_EVENTS: Arc<Mutex<Vec<(KeyEvent, i32)>>> = Arc::new(Mutex::new(vec![]));
    static ref MESSAGES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    static ref BINARY_MESSAGES: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![]));
}

thread_local!{
    // static KEY_EVENTS: Rc<Vec<(KeyEvent, i32)>> = Rc::new(vec![]);
    // static MESSAGES: RefCell<Vec<String>> = RefCell::new(vec![]);
    static JS: RefCell<JS> = RefCell::new(JS{
        request_animation_frame_callback: None,
        on_window_resize_listener: None,
        on_resource_load_listener: None,
        on_connect_listener: None,
        on_close_listener: None,
        // on_message_listener: None,
        // on_key_up_listener: None,
        // on_key_down_listener: None
    });
    static CONTEXT: JSGameContext = JSGameContext{};
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn request_animation_frame_callback(timestamp: f64) {
    JS.with(|js|{
        if let Some(callback) = js.borrow().request_animation_frame_callback {
            callback(timestamp);
        }
    });
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_window_resize() {
    JS.with(|js|{
        if let Some(callback) = js.borrow().on_window_resize_listener {
            callback();
        }
    });
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_resource_load(num: i32, total: i32) {
    JS.with(|js|{
        if let Some(callback) = js.borrow().on_resource_load_listener {
            callback(num, total);
        }
    });
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_connect() {
    JS.with(|js|{
        if let Some(callback) = js.borrow().on_connect_listener {
            callback();
        }
    });
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_close() {
    JS.with(|js|{
        if let Some(callback) = js.borrow().on_close_listener {
            callback();
        }
    });
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_keyup_event(key: i32) {
    //console_log("on_keydown_up");
    if let Ok(mut events) = KEY_EVENTS.lock(){
        events.push((KeyEvent::KeyUp, key));
    }
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_keydown_event(key: i32) {
    //console_log("on_keydown");
    if let Ok(mut events) = KEY_EVENTS.lock(){
        events.push((KeyEvent::KeyDown, key));
    }
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn on_message(msg: *mut c_char) {
    let c_string = unsafe{ CString::from_raw(msg) };
    let s = c_string.to_str().unwrap_or("NULL");
    let s2 = s.clone();
    drop(s);
    //console_log("on_message 111");
    if !(s=="NULL") {
        if let Ok(mut messages) = MESSAGES.lock(){
            messages.push(s2.to_string());
        }
    }
    //console_log("on_message 222");
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub unsafe fn on_binary_message(msg: *mut u8, length: usize) {
    let msg = Vec::from_raw_parts(msg, length, length);
    let msg2 = msg.clone();
    drop(msg);
    //console_log(&format!("wasm:on_binary_message {:?} len={}", msg, msg.len()));
    if let Ok(mut messages) = BINARY_MESSAGES.lock() {
        messages.push(msg2);
    }
}

// #[no_mangle]
// pub fn on_message(msg: *mut c_char) {
//     let c_string = unsafe{ CString::from_raw(msg) };
//     let s = c_string.to_str().unwrap_or("NULL");
//     JS.with(|js|{
//         if let Some(callback) = js.borrow().on_message_listener {
//             callback(s);
//         }
//     });
// }

// #[no_mangle]
// pub fn on_keyup_event(key: i32) {
//     CONTEXT.with(|context|{
//             context.console_log("on_key_up 001");
//     });
//     JS.with(|js|{
//         if let Some(callback) = js.borrow().on_key_up_listener {
//             callback(key);
//         }
//     });
//     CONTEXT.with(|context|{
//             context.console_log("on_key_up 002");
//     });
// }

// #[no_mangle]
// pub fn on_keydown_event(key: i32) {
//     CONTEXT.with(|context|{
//             context.console_log("on_key_down 001");
//     });
//     JS.with(|js|{
//         if let Some(callback) = js.borrow().on_key_down_listener {
//             callback(key);
//         }
//     });
//     CONTEXT.with(|context|{
//             context.console_log("on_key_down 002");
//     });
// }

fn console_log( msg: &str) {
    unsafe {
        if let Ok(string) = CString::new(msg){
            emscripten_console_log(string.as_ptr());
        }
    }
}

#[link("-s"= "ASSERTIONS=1")]
#[no_mangle]
pub fn start() {
    GAME.with(|game|{
        let mut game = game.borrow_mut();
        game.set_game_context(Box::new(JSGameContext{}));
        game.client_start();
    });
}

pub struct JSGameContext {}

impl GameContext for JSGameContext {
    fn draw_image_repeat(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32){
        unsafe { emscripten_draw_image_repeat(res_id, x, y, width, height); }
    }
    fn draw_image_repeat_x(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32){
        unsafe { emscripten_draw_image_repeat_x(res_id, x, y, width, height); }
    }
    fn draw_image_repeat_y(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32){
        unsafe { emscripten_draw_image_repeat_y(res_id, x, y, width, height); }
    }

    fn draw_image_at(&self, res_id: i32, x: i32, y: i32) {
        unsafe {
            emscripten_draw_image_at(res_id, x, y);
        }
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

    fn stroke_style(&self, style: &str) {
        unsafe{
            if let Ok(style) = CString::new(style){
                emscripten_stroke_style(style.as_ptr());
            }
        }
    }
    fn stroke_rect(&self, x: i32, y: i32, width: i32, height: i32){
        unsafe { emscripten_stroke_rect(x, y, width, height); }
    }

    fn fill_style(&self, style: &str) {
        unsafe {
            if let Ok(string) = CString::new(style){
                emscripten_fill_style(string.as_ptr());
            }
        }
    }

    fn set_canvas_font(&self, font: &str) {
        unsafe {
            if let Ok(string) = CString::new(font){
                emscripten_set_canvas_font(string.as_ptr());
            }
        }
    }

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            emscripten_fill_rect(x, y, width, height);
        }
    }

    fn fill_text(&self, text: &str, x: i32, y: i32) {
        unsafe {
            if let Ok(string) = CString::new(text){
                emscripten_fill_text(string.as_ptr(), x, y);
            }
        }
    }

    fn set_frame_callback(&self, callback: fn(f64)) {
        JS.with(|js|{
            js.borrow_mut().request_animation_frame_callback = Some(callback);
        });
    }

    fn set_on_window_resize_listener(&self, listener: fn()) {
        JS.with(|js|{
            js.borrow_mut().on_window_resize_listener = Some(listener);
        });
    }

    fn set_on_connect_listener(&self, listener: fn()) {
        JS.with(|js|{
            js.borrow_mut().on_connect_listener = Some(listener);
        });
    }

    fn set_on_close_listener(&self, listener: fn()) {
        JS.with(|js|{
            js.borrow_mut().on_close_listener = Some(listener);
        });
    }

    fn set_on_resource_load_listener(&self, listener: fn(num: i32, total: i32)) {
        JS.with(|js|{
            js.borrow_mut().on_resource_load_listener = Some(listener);
        });
    }
    
    fn pick_key_events(&self)->Vec<(KeyEvent, i32)>{
        let mut events = vec![];
        //console_log(&format!("es_len={}", es.len()));
        if let Ok(mut e) = KEY_EVENTS.lock(){
            events.append(&mut e);
        }
        events
    }

    fn pick_messages(&self)->Vec<String>{
        let mut msgs = vec![];
        if let Ok(mut m) = MESSAGES.lock(){
            msgs.append(&mut m);
        }
        msgs
    }

    fn pick_binary_messages(&self) -> Vec<Vec<u8>> {
        let mut msgs = vec![];
        if let Ok(mut m) = BINARY_MESSAGES.lock() {
            msgs.append(&mut m);
        }
        msgs
    }

    fn request_animation_frame(&self) {
        unsafe {
            emscripten_request_animation_frame();
        }
    }

    // fn random(&self) -> f64 {
    //     unsafe{
    //         emscripten_random()
    //     }
    // }

    fn console_log(&self, msg: &str) {
        unsafe {
            if let Ok(string) = CString::new(msg){
                emscripten_console_log(string.as_ptr());
            }
        }
    }

    fn current_time_millis(&self, )->u64{
        unsafe{
            emscripten_current_time_millis() as u64
        }
    }

    fn alert(&self, msg: &str) {
        unsafe {
            if let Ok(string) = CString::new(msg){
                emscripten_alert(string.as_ptr());
            }
        }
    }

    fn load_resource(&self, json: String) {
        unsafe {
            if let Ok(string) = CString::new(json){
                emscripten_load_resource(string.as_ptr());
            }
        }
    }

    fn window_inner_width(&self, ) -> i32 {
        unsafe { emscripten_window_inner_width() }
    }

    fn window_inner_height(&self, ) -> i32 {
        unsafe { emscripten_window_inner_height() }
    }

    fn send_message(&self, msg: &str) {
        unsafe {
            if let Ok(string) = CString::new(msg){
                emscripten_send_message(string.as_ptr());
            }
        }
    }

    fn connect(&self, url: &str) {
        unsafe {
            if let Ok(string) = CString::new(url){
                emscripten_connect(string.as_ptr());
            }
        }
    }

    fn line_width(&self, width:i32){
        unsafe{
            emscripten_line_width(width);
        }
    }

    fn set_canvas_style_margin(&self, left: i32, top: i32, right: i32, bottom: i32) {
        unsafe { emscripten_set_canvas_style_margin(left, top, right, bottom) };
    }
    fn set_canvas_style_width(&self, width: i32) {
        unsafe { emscripten_set_canvas_style_width(width) };
    }
    fn set_canvas_style_height(&self, height: i32) {
        unsafe { emscripten_set_canvas_style_height(height) };
    }
    fn set_canvas_width(&self, width: i32) {
        unsafe { emscripten_set_canvas_width(width) };
    }
    fn set_canvas_height(&self, height: i32) {
        unsafe { emscripten_set_canvas_height(height) };
    }

    fn send_binary_message(&self, msg: &Vec<u8>) {
        //console_log(&format!("wasm:send_binary_message {:?} len={}", msg, msg.len()));
        unsafe {
            emscripten_send_binary_message(msg.as_ptr(), msg.len());
        }
    }

    fn prompt(&self, title:&str, default_msg:&str)->String{
        if let Ok(title) = CString::new(title){
            if let Ok(msg) = CString::new(default_msg){
                let c_string = unsafe{ CString::from_raw(emscripten_prompt(title.as_ptr(), msg.as_ptr())) };
                let name = c_string.to_str().unwrap_or("");
                return String::from(name.clone());
            }
        }
        String::new()
    }
}

#[link("-s"= "ASSERTIONS=1")]
fn main(){
    println!("main.");
    // let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    // let elapsed = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
    // println!("elapsed={}", elapsed);
}