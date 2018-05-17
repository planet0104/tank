#![feature(proc_macro)]

#[macro_use]
extern crate stdweb;
extern crate tank;
#[macro_use]
extern crate lazy_static;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use stdweb::js_export;
use stdweb::web::TypedArray;
use stdweb::serde::Serde;
use stdweb::unstable::TryInto;
use stdweb::console;
use stdweb::web::html_element::InputElement;
use stdweb::web::{
    document,
    window,
    HtmlElement,
    Date,
    WebSocket,
    IEventTarget,
    FileReader,
    FileReaderResult,
    IElement,
    IParentNode,
    IHtmlElement,
};
use stdweb::web::event::{
    KeyDownEvent,
    KeyUpEvent,
    SocketOpenEvent,
    SocketCloseEvent,
    SocketErrorEvent,
    SocketMessageEvent,
    ResizeEvent,
    LoadEndEvent,
    PointerMoveEvent,
    PointerDownEvent,
    PointerUpEvent,
    PointerOutEvent,
    IMouseEvent,
    ClickEvent,
    MouseMoveEvent,
    MouseDownEvent,
    MouseUpEvent,
    IMessageEvent, IKeyboardEvent, IEvent
};

use tank::engine::GameContext;
use std::cell::RefCell;
use tank::{ GAME, KEY_MAP, KeyEvent, VK_SPACE, VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref KEY_EVENTS: Arc<Mutex<Vec<(KeyEvent, i32)>>> = Arc::new(Mutex::new(vec![]));
    static ref MESSAGES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    static ref BINARY_MESSAGES: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![]));
    static ref SOCKET: Arc<Mutex<Option<WebSocket>>> = Arc::new(Mutex::new(None));
    static ref KEY_BOARD_STATUS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
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

const id_game_pad_button_a:i32 = 0;
const id_game_pad_button_b:i32 = 1;
const id_game_pad_direction:i32 = 3;
const id_touchstart:i32 = 0;
const id_touchend:i32 = 1;
const id_touchmove:i32 = 2;

fn connect(url: &str){
    if let Ok(mut socket) = SOCKET.lock() {
        let ws = WebSocket::new(url).unwrap();

        ws.add_event_listener(move |_: SocketOpenEvent| {
            JS.with(|e| {
                if let Some(callback) = e.borrow().on_connect_listener {
                    callback();
                }
            });
            
            //加入游戏
            join_game();
        });

        ws.add_event_listener(move |_: SocketErrorEvent| {
            js!{
                alert("连接失败，请重试");
            }
        });

        ws.add_event_listener(move |_event: SocketCloseEvent| {
            //output_msg(&format!("> Connection Closed: {}", event.reason()));
            JS.with(|e| {
                if let Some(callback) = e.borrow().on_close_listener {
                    callback();
                }
            });
        });

        ws.add_event_listener(move |event: SocketMessageEvent| {
            //output_msg(&event.data().into_text().unwrap());
            if let Some(blob) = event.data().into_blob(){
                let reader = FileReader::new();
                let reader_clone = reader.clone();
                reader.add_event_listener(move |_:LoadEndEvent|{
                    if let Ok(mut messages) = BINARY_MESSAGES.lock() {       
                        if let Some(result) = reader_clone.result(){
                            match result{
                                FileReaderResult::ArrayBuffer(buffer) => {
                                    messages.push(buffer.into());
                                }
                                _ => {}
                            }
                        }
                    }
                });
                let _ = reader.read_as_array_buffer(&blob);
            }else if let Some(text) = event.data().into_text(){
                console!(log, "text消息", text);
            }else if let Some(_buffer) = event.data().into_array_buffer(){
                console!(log, "buffer消息");
            }else{
                console!(log, "未知消息");
            }
        });

        *socket = Some(ws);
    }else{
        js!(alert("Socket创建失败"));
    }
}

pub struct JSGameContext {}

impl GameContext for JSGameContext {
    fn current_time_millis(&self) -> f64 {
        Date::now()
    }
    fn draw_image_repeat(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32) {
        js!{
            ctx.fillStyle = ctx.createPattern(window.resMap.get(@{res_id}+""), "repeat");
            ctx.fillRect(@{x}, @{y}, @{width}, @{height});
        }
    }
    fn draw_image_repeat_x(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32) {
        js!{
            // 平铺方式
            ctx.fillStyle = ctx.createPattern(window.resMap.get(@{res_id}+""), "repeat-x");
            ctx.fillRect(@{x}, @{y}, @{width}, @{height});
        }
    }
    fn draw_image_repeat_y(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32) {
        js!{
            // 平铺方式
            ctx.fillStyle = ctx.createPattern(window.resMap.get(@{res_id}+""), "repeat-y");
            ctx.fillRect(@{x}, @{y}, @{width}, @{height});
        }
    }
    fn draw_image_at(&self, res_id: i32, x: i32, y: i32) {
        js!{
            ctx.drawImage(window.resMap.get(@{res_id}+""), @{x}, @{y});
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
        js!{
            ctx.drawImage(window.resMap.get(@{res_id}+""), @{source_x}, @{source_y}, @{source_width}, @{source_height}, @{dest_x}, @{dest_y}, @{dest_width}, @{dest_height});
        }
    }

    fn fill_style(&self, style: &str) {
        js!{
            ctx.fillStyle = @{style};
        }
    }

    fn stroke_style(&self, style: &str) {
        js!{
            ctx.strokeStyle = @{style};
        }
    }

    fn set_canvas_font(&self, font: &str) {
        js!{
            ctx.font = @{font};
        }
    }

    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        js!{
            ctx.fillRect(@{x}, @{y}, @{width}, @{height});
        }
    }

    fn stroke_rect(&self, x: i32, y: i32, width: i32, height: i32) {
        js!{
            ctx.strokeRect(@{x}, @{y}, @{width}, @{height});
        }
    }

    fn fill_text(&self, text: &str, x: i32, y: i32) {
        js!{
            ctx.fillText(@{text}, @{x}, @{y});
        }
    }

    fn set_frame_callback(&self, callback: fn(f64)) {
        JS.with(|js| {
            js.borrow_mut().request_animation_frame_callback = Some(callback);
        });
    }

    fn set_on_window_resize_listener(&self, listener: fn()) {
        JS.with(|js| {
            js.borrow_mut().on_window_resize_listener = Some(listener);
        });
    }

    fn set_on_connect_listener(&self, listener: fn()) {
        JS.with(|js| {
            js.borrow_mut().on_connect_listener = Some(listener);
        });
    }

    fn set_on_close_listener(&self, listener: fn()) {
        JS.with(|js| {
            js.borrow_mut().on_close_listener = Some(listener);
        });
    }

    fn set_on_resource_load_listener(&self, listener: fn(num: i32, total: i32)) {
        JS.with(|js| {
            js.borrow_mut().on_resource_load_listener = Some(listener);
        });
    }

    fn pick_key_events(&self) -> Vec<(KeyEvent, i32)> {
        let mut events = vec![];
        if let Ok(mut e) = KEY_EVENTS.lock() {
            events.append(&mut e);
        }
        events
    }

    fn pick_messages(&self) -> Vec<String> {
        let mut msgs = vec![];
        if let Ok(mut m) = MESSAGES.lock() {
            msgs.append(&mut m);
        }
        msgs
    }

    fn pick_binary_messages(&self) -> Vec<Vec<u8>> {
        let mut msgs = vec![];
        if let Ok(mut m) = BINARY_MESSAGES.lock() {
            msgs.append(&mut m);
        }
        //console_log(&format!("wasm:pick_binary_messages {:?} len={}", msgs, msgs.len()));
        msgs
    }

    fn request_animation_frame(&self) {
        JS.with(|e| {
            if let Some(rust_callback) = e.borrow().request_animation_frame_callback {
                js!{
                    var callback = @{rust_callback};;
                    window.requestAnimationFrame(callback);
                }
            }
        });
    }

    fn console_log(&self, msg: &str) {
        js!{
            console.log(@{msg});
        }
    }

    fn alert(&self, msg: &str) {
        js!{
            alert(@{msg});
        }
    }

    fn line_width(&self, width: i32) {
        js!{
            ctx.lineWidth = @{width};
        }
    }

    fn load_resource(&self, json: String) {
        let on_resource_load = |num: i32, total:i32|{
            JS.with(|e| {
                if let Some(callback) = e.borrow().on_resource_load_listener {
                    callback(num, total);
                }
            });
        };
        js!{
            var on_resource_load = @{on_resource_load};
            var urls = JSON.parse(@{json});
            loadResources(urls, function(map, num, total){
                window.resMap = map;
                on_resource_load(num, total);
            });
        }
    }

    fn window_inner_width(&self) -> i32 {
        window().inner_width()
    }

    fn window_inner_height(&self) -> i32 {
        window().inner_height()
    }

    fn send_message(&self, msg: &str) {
        if let Ok(mut socket) = SOCKET.lock() {
            let _ = socket.as_ref().unwrap().send_text(msg);
        }
    }

    fn send_binary_message(&self, msg: &Vec<u8>) {
        if let Ok(mut socket) = SOCKET.lock() {
            let _ = socket.as_ref().unwrap().send_bytes(msg.as_slice());
        }
    }

    fn connect(&self, url: &str) {
        connect(url);
    }

    fn set_canvas_style_margin(&self, left: i32, top: i32, right: i32, bottom: i32) {
        js!{
            canvas.style.marginLeft = @{left}+"px";
            canvas.style.marginTop = @{top}+"px";
            canvas.style.marginRight = @{right}+"px";
            canvas.style.marginBottom = @{bottom}+"px";
        }
    }
    fn set_canvas_style_width(&self, width: i32) {
        js!{
            canvas.style.width = @{width}+"px";
        }
    }
    fn set_canvas_style_height(&self, height: i32) {
        js!{
            canvas.style.height = @{height}+"px";
        }
    }
    fn set_canvas_width(&self, width: i32) {
        js!{
            canvas.width = @{width};
        }
    }
    fn set_canvas_height(&self, height: i32) {
        js!{
            canvas.height = @{height};
        }
    }
}

fn join_game(){
    //------------- 输入名字对话框 --------------
    let btn_start =  document().query_selector("#btn_start").unwrap().unwrap();
    js!(document.getElementById("input_name_dialog").style.display = "block"; );
    btn_start.add_event_listener(move |_: ClickEvent| {
        GAME.with(move |game| {
            let mut game = game.borrow_mut();
            let txt_user_name:InputElement = document().query_selector( "#txt_user_name" ).unwrap().unwrap().try_into().unwrap();
            game.player_join_game(&txt_user_name.raw_value());
            js!(document.getElementById("input_name_dialog").style.display = "none"; );
        });
    });
}

#[cfg(all(target_arch = "asmjs", target_os = "emscripten"))]
#[no_mangle]
pub fn on_touch_event(target:i32, event_type: i32, client_x:i32, client_y:i32){
    handle_touch_event(target, event_type, client_x, client_y);
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[js_export]
pub fn on_touch_event(target:i32, event_type: i32, client_x:i32, client_y:i32){
    handle_touch_event(target, event_type, client_x, client_y);
}

fn handle_touch_event(target:i32, event_type: i32, client_x:i32, client_y:i32){
    if let Ok(mut events) = KEY_EVENTS.lock() {
        match target{
            id_game_pad_button_a | id_game_pad_button_b => {
                match event_type{
                    id_touchstart => events.push((KeyEvent::KeyDown, VK_SPACE)),
                    id_touchend => (),
                    id_touchmove => (),
                    _ => ()
                }
            },
            id_game_pad_direction => {
                let game_pad_direction:HtmlElement = document().query_selector("#game_pad_direction").unwrap().unwrap().try_into().unwrap();
                match event_type{
                    id_touchend => {
                        let _ = game_pad_direction.set_attribute("status", "0");
                        events.push((KeyEvent::KeyUp, VK_LEFT));
                    },
                    id_touchmove | id_touchstart => {
                        //方向按钮按下 判断按钮方向
                        let offset_left:i32 =  js!(return game_pad.offsetLeft + game_pad_direction.offsetLeft).try_into().unwrap();
                        let offset_top:i32 =  js!(return game_pad.offsetTop + game_pad_direction.offsetTop).try_into().unwrap();
                        //方向按钮按下 判断按钮方向
                        let x = client_x - offset_left;
                        let y = client_y - offset_top;
                        let offset_width:i32 = js!(return game_pad_direction.offsetWidth).try_into().unwrap();
                        let btn_width = offset_width/3;
                        let direction_status = game_pad_direction.get_attribute("status").unwrap_or("0".to_string()).parse::<i32>().unwrap();

                        if x>=btn_width&&x<=btn_width*2&&y<=btn_width && direction_status != 1 {
                            let _ = game_pad_direction.set_attribute("status", "1");
                            events.push((KeyEvent::KeyDown, VK_UP));
                        }

                        if x>=btn_width&&x<btn_width*2&&y>=btn_width*2&&y<=btn_width*3 && direction_status != 2 {
                            let _ = game_pad_direction.set_attribute("status", "2");
                            events.push((KeyEvent::KeyDown, VK_DOWN));
                        }

                        if x<=btn_width&&y>=btn_width&&y<=btn_width*2 && direction_status != 3 {
                            let _ = game_pad_direction.set_attribute("status", "3");
                            events.push((KeyEvent::KeyDown, VK_LEFT));
                        }

                        if x>=btn_width*2&&y>=btn_width&&y<=btn_width*2 && direction_status != 4 {
                            let _ = game_pad_direction.set_attribute("status", "4");
                            events.push((KeyEvent::KeyDown, VK_RIGHT));
                        }
                    },
                    _ => ()
                }
            }
            _ => ()
        }   
    }
}

fn main() {
    stdweb::initialize();

    //------------- 键盘事件 -----------------------------------

    window().add_event_listener(|_: ResizeEvent| {
        JS.with(|e| {
            if let Some(callback) = e.borrow().on_window_resize_listener {
                callback();
            }
        });
    });

    window().add_event_listener(|event: KeyUpEvent| {
        let key = event.key();
        KEY_MAP.with(|key_map|{
            if key_map.contains_key(&key){
                //event.prevent_default();
                if let Ok(mut status) = KEY_BOARD_STATUS.lock(){
                    //按键弹起删除状态
                    let ke:&str = key.as_ref();
                    status.retain(|ref k|{
                        let k:&str = k.as_ref();
                        k != ke
                    });
                    if let Ok(mut events) = KEY_EVENTS.lock() {
                        events.push((KeyEvent::KeyUp, *key_map.get(&key).unwrap()));
                    }else{
                        console!(log, "KEY_EVENTS lock失败");
                    }
                }else{
                    console!(log, "KEY_BOARD_STATUS lock失败");
                }
            }else{
                console!(log, "未定义按键", event.key());
            }
        });
    });

    window().add_event_listener(|event: KeyDownEvent| {
        let key = event.key();
        KEY_MAP.with(|key_map|{
            if key_map.contains_key(&key){
                //event.prevent_default();
                if let Ok(mut status) = KEY_BOARD_STATUS.lock(){
                    if !status.contains(&key){
                        status.push(event.key());
                        if let Ok(mut events) = KEY_EVENTS.lock() {
                            events.push((KeyEvent::KeyDown, *key_map.get(&key).unwrap()));
                        }else{
                            console!(log, "KEY_EVENTS lock失败");
                        }
                    }
                }else{
                    console!(log, "KEY_BOARD_STATUS lock失败");
                }
            }else{
                console!(log, "未定义按键", event.key());
            }
        });
    });

    js!{
        window.id_game_pad_button_a = @{id_game_pad_button_a};
        window.id_game_pad_button_b = @{id_game_pad_button_b};
        window.id_game_pad_direction = @{id_game_pad_direction};
        window.id_touchstart = @{id_touchstart};
        window.id_touchend = @{id_touchend};
        window.id_touchmove = @{id_touchmove};
    }

    //------------- 启动游戏 -----------------------------------

    GAME.with(|game| {
        let mut game = game.borrow_mut();
        game.set_game_context(Box::new(JSGameContext {}));
        game.client_start();
    });

    stdweb::event_loop();
}