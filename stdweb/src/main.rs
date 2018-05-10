#[macro_use]
extern crate stdweb;
extern crate tank;
#[macro_use]
extern crate lazy_static;

use tank::engine::GameContext;
use std::cell::RefCell;
use tank::GAME;
use tank::KeyEvent;
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref KEY_EVENTS: Arc<Mutex<Vec<(KeyEvent, i32)>>> = Arc::new(Mutex::new(vec![]));
    static ref MESSAGES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    static ref BINARY_MESSAGES: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![]));
}

struct JS {
    request_animation_frame_callback: Option<fn(f64)>,
    on_window_resize_listener: Option<fn()>,
    on_resource_load_listener: Option<fn(num: i32, total: i32)>,
    on_connect_listener: Option<fn()>,
    on_close_listener: Option<fn()>,
}

thread_local!{
    static JS: RefCell<JS> = RefCell::new(JS{
        request_animation_frame_callback: None,
        on_window_resize_listener: None,
        on_resource_load_listener: None,
        on_connect_listener: None,
        on_close_listener: None,
    });
}

fn main() {
    stdweb::initialize();

    let message = "Hello, 世界!";
    js! {
        alert( @{message} );
    }

    stdweb::event_loop();
}