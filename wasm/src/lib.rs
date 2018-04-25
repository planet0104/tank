extern crate tank;
use std::mem;
#[macro_use]
extern crate lazy_static;
use tank::engine::GameContext;
use std::cell::RefCell;
use tank::GAME;
use tank::KeyEvent;
use std::sync::{Arc, Mutex};

//导入的JS帮助函数
extern "C" {
    pub fn _alert(text: *const u8, len: usize);
    pub fn _prompt(t1: *const u8, len: usize, t2: *const u8, len: usize) -> usize;
    pub fn _get_prompt_ptr()->*mut u8;
    pub fn _console_log(text: *const u8, len: usize);
    pub fn _current_time_millis() -> f64;
    pub fn _random() -> f64;
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
    pub fn _fill_style(text: *const u8, len: usize);
    pub fn _stroke_style(text: *const u8, len: usize);
    pub fn _line_width(width:i32);
    pub fn _fill_rect(x: i32, y: i32, width: i32, height: i32);
    pub fn _stroke_rect(x: i32, y: i32, width: i32, height: i32);
    pub fn _fill_text(text: *const u8, len: usize, x: i32, y: i32);
    pub fn _draw_image_at(res_id: i32, x: i32, y: i32);
    pub fn _draw_image_repeat_y(resId: i32, x: i32, y: i32, width: i32, height: i32);
    pub fn _draw_image_repeat_x(resId: i32, x:i32 , y: i32, width: i32, height: i32);
    pub fn _draw_image_repeat(resId: i32, x:i32 , y: i32, width: i32, height: i32);
    pub fn _draw_image(
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
    pub fn _send_message(text: *const u8, len: usize);
    pub fn _connect(url: *const u8, len: usize);
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
}

pub fn random() -> f64 {
    unsafe{
        _random()
    }
}

pub fn prompt(title:&str, default_msg:&str) -> String{
    let len = unsafe{ _prompt(title.as_ptr(), title.len(), default_msg.as_ptr(), default_msg.len()) };
    let ptr = unsafe{ _get_prompt_ptr() };
    let prompt = unsafe{ String::from_raw_parts(ptr, len, len) };
    return String::from(prompt.clone());
}

pub fn current_time_millis() -> u64 {
    unsafe { _current_time_millis() as u64 }
}

pub fn console_log(msg: &str) {
    unsafe {
        _console_log(msg.as_ptr(), msg.len());
    }
}

pub fn alert(msg: &str) {
    unsafe {
        _alert(msg.as_ptr(), msg.len());
    }
}

pub fn load_resource(json: String) {
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

pub fn fill_style(style: &str) {
    unsafe {
        _fill_style(style.as_ptr(), style.len());
    }
}

pub fn stroke_style(style: &str) {
    unsafe {
        _stroke_style(style.as_ptr(), style.len());
    }
}

pub fn fill_rect(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        _fill_rect(x, y, width, height);
    }
}

pub fn stroke_rect(x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        _stroke_rect(x, y, width, height);
    }
}

pub fn fill_text(text: &str, x: i32, y: i32) {
    unsafe {
        _fill_text(text.as_ptr(), text.len(), x, y);
    }
}

pub fn set_canvas_font(font: &str) {
    unsafe {
        _set_canvas_font(font.as_ptr(), font.len());
    }
}

pub fn send_message(msg: &str) {
    unsafe {
        _send_message(msg.as_ptr(), msg.len());
    }
}

pub fn connect(url: &str) {
    unsafe {
        _connect(url.as_ptr(), url.len());
    }
}

pub fn line_width(width:i32){
    unsafe{
        _line_width(width);
    }
}

pub fn draw_image_at(res_id: i32, x: i32, y: i32) {
    unsafe {
        _draw_image_at(res_id, x, y);
    }
}

pub fn draw_image_repeat(res_id: i32, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        _draw_image_repeat(res_id, x, y, width, height);
    }
}

pub fn draw_image_repeat_x(res_id: i32, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        _draw_image_repeat_x(res_id, x, y, width, height);
    }
}

pub fn draw_image_repeat_y(res_id: i32, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
        _draw_image_repeat_y(res_id, x, y, width, height);
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
        _draw_image(
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
pub fn on_connect() {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_connect_listener {
            callback();
        }
    });
}

#[no_mangle]
pub fn on_close() {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_close_listener {
            callback();
        }
    });
}

#[no_mangle]
pub fn on_keyup_event(key: i32) {
    //console_log("on_keydown_up");
    //console_log(&format!("on_keyup_event={}", key));
    if let Ok(mut events) = KEY_EVENTS.lock(){
        events.push((KeyEvent::KeyUp, key));
    }
    //console_log(&format!("on_keyup_event push OK.={}", key));
}

#[no_mangle]
pub fn on_keydown_event(key: i32) {
    //console_log(&format!("on_keydown_event={}", key));
    if let Ok(mut events) = KEY_EVENTS.lock(){
        events.push((KeyEvent::KeyDown, key));
    }
    //console_log(&format!("on_keydown_event push OK.={}", key));
}

#[no_mangle]
pub fn on_message(msg: *mut u8, length: usize) {
    let msg = unsafe{ String::from_raw_parts(msg, length, length) };
    //console_log(&format!("wasm_on_message = {:?}", msg));
    if let Ok(mut messages) = MESSAGES.lock(){
        messages.push(msg);
    }
    //console_log("on_message 222");
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
    GAME.with(|game|{
        let mut game = game.borrow_mut();
        game.set_game_context(Box::new(JSGameContext{}));
        game.client_start();
    });
}

pub struct JSGameContext {}

impl GameContext for JSGameContext {
    fn current_time_millis(&self) -> u64{
        current_time_millis()
    }
    fn draw_image_repeat(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32){
        draw_image_repeat(res_id, x, y, width, height);
    }
    fn draw_image_repeat_x(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32){
        draw_image_repeat_x(res_id, x, y, width, height);
    }
    fn draw_image_repeat_y(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32){
        draw_image_repeat_y(res_id, x, y, width, height);
    }
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
        draw_image(res_id, source_x, source_y, source_width, source_height, dest_x, dest_y, dest_width, dest_height);
    }

    fn fill_style(&self, style: &str) {
        fill_style(style);
    }

    fn stroke_style(&self, style: &str) {
        stroke_style(style);
    }

    fn set_canvas_font(&self, font: &str) {
        set_canvas_font(font);
    }

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        fill_rect(x, y, width, height);
    }

    fn stroke_rect(&self, x: i32, y: i32, width: i32, height: i32){
        stroke_rect(x, y, width, height);
    }

    fn fill_text(&self, text: &str, x: i32, y: i32) {
        fill_text(text, x, y);
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

    // fn set_on_message_listener(&self, listener: fn(msg: &str)) {
    //     JS.with(|js|{
    //         js.borrow_mut().on_message_listener = Some(listener);
    //     });
    // }

    // fn set_on_key_up_listener(&self, listener: fn(key: i32)) {
    //     JS.with(|js|{
    //         js.borrow_mut().on_key_up_listener = Some(listener);
    //     });
    // }

    // fn set_on_key_down_listener(&self, listener: fn(key: i32)) {
    //     JS.with(|js|{
    //         js.borrow_mut().on_key_down_listener = Some(listener);
    //     });
    // }
    
    fn pick_key_events(&self)->Vec<(KeyEvent, i32)>{
        let mut events = vec![];
        //console_log(&format!("es_len={}", es.len()));
        if let Ok(mut e) = KEY_EVENTS.lock(){
            events.append(&mut e);
            //console_log(&format!("pick_key_events={:?}", events));
        }
        events
    }

    fn pick_messages(&self)->Vec<String>{
        let mut msgs = vec![];
        if let Ok(mut m) = MESSAGES.lock(){
            msgs.append(&mut m);
            //console_log(&format!("pick_messages={:?}", msgs));
        }
        msgs
    }

    fn request_animation_frame(&self) {
        request_animation_frame();
    }

    fn console_log(&self, msg: &str) {
        console_log(msg);
    }

    fn alert(&self, msg: &str) {
        alert(msg);
    }

    fn line_width(&self, width:i32){
        line_width(width);
    }

    fn load_resource(&self, json: String) {
        load_resource(json);
    }

    fn window_inner_width(&self, ) -> i32 {
        window_inner_width()
    }

    fn window_inner_height(&self, ) -> i32 {
        window_inner_height()
    }

    fn send_message(&self, msg: &str) {
        send_message(msg);
    }

    fn connect(&self, url: &str) {
        connect(url);
    }

    fn set_canvas_style_margin(&self, left: i32, top: i32, right: i32, bottom: i32) {
        set_canvas_style_margin(left, top, right, bottom);
    }
    fn set_canvas_style_width(&self, width: i32) {
        set_canvas_style_width(width);
    }
    fn set_canvas_style_height(&self, height: i32) {
        set_canvas_style_height(height);
    }
    fn set_canvas_width(&self, width: i32) {
        set_canvas_width(width);
    }
    fn set_canvas_height(&self, height: i32) {
        set_canvas_height(height);
    }

    fn prompt(&self, title:&str, default_msg:&str)->String{
        prompt(title, default_msg)
    }
}