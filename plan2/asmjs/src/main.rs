#[macro_use]
extern crate serde_json;
//#[macro_use]
//extern crate stdweb;
extern crate tank;
//use stdweb::unstable::TryInto;

mod game;
use std::cell::RefCell;
use std::mem;
use tank::engine::CanvasContext;

struct JS {
    request_animation_frame_callback: Option<fn(f64)>,
    on_resource_load_listener: Option<fn(num: i32, total: i32)>,
    on_connect_listener: Option<fn()>,
    on_close_listener: Option<fn()>,
    on_message_listener: Option<fn(msg: String)>,
}

thread_local!{
    static JS: RefCell<JS> = RefCell::new(JS{
        request_animation_frame_callback: None,
        on_resource_load_listener: None,
        on_connect_listener: None,
        on_close_listener: None,
        on_message_listener: None,
    });
}

pub fn random() -> f64 {
    //js!(random()).try_into().unwrap()
    0.0
}

pub fn current_time_millis() -> u64 {
    // js!({
    //     return current_time_millis();
    // }).try_into().unwrap()
    0
}

pub fn console_log(msg: &str) {
    //js!(console_log(@{msg}));
}

pub fn load_resource(map: serde_json::Value) {
    let json = serde_json::to_string(&map).unwrap();
    //js!(load_resource(@{json}));
}

pub fn send_json_message(json: serde_json::Value) {
    let json = serde_json::to_string(&json).unwrap();
    send_message(&json);
}

pub fn window_inner_width() -> i32 {
    // js!({
    //     return window.innerWidth;
    // }).try_into().unwrap()
    100
}

pub fn window_inner_height() -> i32 {
    // js!({
    //     return window.innerHeight;
    // }).try_into().unwrap()
    100
}

pub fn fill_style(style: &str) {
    //js!(fill_style(@{style}));
}

pub fn fill_rect(x: i32, y: i32, width: i32, height: i32) {
    //js!(fill_rect(@{x}, @{y}, @{width}, @{height}));
}

pub fn fill_text(text: &str, x: i32, y: i32) {
    //js!(fill_text(@{text}, @{x}, @{y}));
}

pub fn set_canvas_font(font: &str) {
    //js!(set_canvas_font(@{font}));
}

pub fn send_message(msg: &str) {
    //js!(send_message(@{msg}));
}

pub fn connect(url: &str) {
    //js!(connect(@{url}));
}

pub fn draw_image_at(res_id: i32, x: i32, y: i32) {
    //js!(draw_image_at(@{res_id}, @{x}, @{y}));
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
    // js!(draw_image(
    //         @{res_id},
    //         @{source_x},
    //         @{source_y},
    //         @{source_width},
    //         @{source_height},
    //         @{dest_x},
    //         @{dest_y},
    //         @{dest_width},
    //         @{dest_height},
    //     ));
}

pub fn set_canvas_style_margin(left: i32, top: i32, right: i32, bottom: i32) {
    //js!(set_canvas_style_margin(@{left}, @{top}, @{right}, @{bottom}));
}
pub fn set_canvas_style_width(width: i32) {
    //js!(set_canvas_style_width(@{width}));
}
pub fn set_canvas_style_height(height: i32) {
    //js!(set_canvas_style_height(@{height}));
}
pub fn set_canvas_width(width: i32) {
    //js!(set_canvas_width(@{width}));
}
pub fn set_canvas_height(height: i32) {
    //js!(set_canvas_height(@{height}));
}

pub fn set_frame_callback(callback: fn(f64)) {
    JS.with(|e| {
        e.borrow_mut().request_animation_frame_callback = Some(callback);
    });
}

pub fn set_on_window_resize_listener(listener: fn()) {
    // js!({
    //     var listener = @{listener};
    //     window.onresize = function(){ listener() };
    // });
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
    // js!({
    //     var listener = @{listener};
    //     document.addEventListener("keyup", function(event){
    //         listener(event.key);
    //     });
    // });
}

pub fn set_on_keydown_listener(listener: fn(key: String)) {
    // js!({
    //     var listener = @{listener};
    //     document.addEventListener("keydown", function(event){
    //         listener(event.key);
    //     });
    // });
}

pub fn set_on_message_listener(listener: fn(msg: String)) {
    JS.with(|e| {
        e.borrow_mut().on_message_listener = Some(listener);
    });
}

pub fn request_animation_frame() {
    //js!(request_animation_frame());
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
pub extern fn on_resource_load(num: i32, total: i32) {
    JS.with(|e| {
        if let Some(callback) = e.borrow().on_resource_load_listener {
            callback(num, total);
        }
    });
}

#[no_mangle]
pub extern fn on_connect() {
    let msg_listener = |msg:String|{
        JS.with(|e| {
            if let Some(callback) = e.borrow().on_message_listener {
                callback(msg);
            }
        });
    };

    // js!({
    //     var msg_listener = @{msg_listener};
    //     socket.onmessage = function(event){
    //         msg_listener(event.data);
    //     };
    // });

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
pub extern fn alloc(size: usize) -> *const u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr;
}

pub struct Context2D {}

impl CanvasContext for Context2D {
    fn draw_image_at(&self, res_id: i32, x: i32, y: i32) {
        //draw_image_at(res_id, x, y);
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
        // draw_image(
        //     res_id,
        //     source_x,
        //     source_y,
        //     source_width,
        //     source_height,
        //     dest_x,
        //     dest_y,
        //     dest_width,
        //     dest_height,
        // );
    }

    fn fill_style(&self, style: &str) {
        //fill_style(style);
    }

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        //fill_rect(x, y, width, height);
    }

    fn fill_text(&self, text: &str, x: i32, y: i32) {
        //fill_text(text, x, y);
    }
}


/*
https://users.rust-lang.org/t/compiling-to-the-web-with-rust-and-emscripten/7627/31

http://floooh.github.io/2016/08/27/asmjs-diet.html

Yes, correct.

Basically you have this boilerplate right now:

#[link_args = "-s EXPORTED_FUNCTIONS=['_hello_world']"]
extern {}

fn main() {}

#[no_mangle]
pub extern fn hello_world(n: c_int) -> c_int {
    n + 1
}

Then you can use this in your javascript to access and call the function:

var hello_world = cwrap('hello_world', 'number', ['number']);

console.log(hello_world(41));

*/

//导入的JS帮助函数
extern "C" {
    pub fn alertTest();
}

fn main(){
    //stdweb::initialize();
    // js!({
    //     console.log(Module);
    // });
    unsafe{ alertTest(); }
    game::start();

    //stdweb::event_loop();
}