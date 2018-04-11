#[macro_use]
extern crate stdweb;
extern crate tank;

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{
    document,
    window,
    CanvasRenderingContext2d
};

use stdweb::web::event::{
    MouseMoveEvent,
    LoadEndEvent,
    KeyDownEvent,
    ResourceLoadEvent,
    ReadyStateChangeEvent,
    ProgressLoadEvent,
    KeyUpEvent,
    ResizeEvent,
};

use stdweb::web::html_element::CanvasElement;

use tank::{KeyEvent, CLIENT_WIDTH, CLIENT_HEIGHT};
use tank::{RES_TANK_BITMAP, RES_MISSILE_BITMAP, RES_LG_EXPLOSION_BITMAP, RES_SM_EXPLOSION__BITMAP};

// Shamelessly stolen from webplatform's TodoMVC example.
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn main(){
    stdweb::initialize();
    let canvas: CanvasElement = document().query_selector( "#canvas" ).unwrap().unwrap().try_into().unwrap();
    let context: CanvasRenderingContext2d = canvas.get_context().unwrap();
    canvas.set_width(CLIENT_WIDTH as u32);
    canvas.set_height(CLIENT_HEIGHT as u32);
    resize_window(canvas.clone());

    //添加事件
    window().add_event_listener( enclose!( (canvas) move |_: ResizeEvent| { resize_window(canvas.clone()); }));
    window().add_event_listener(|event: KeyDownEvent| { handle_key(KeyEvent::KeyDown, event.key()); });
    window().add_event_listener(|event: KeyUpEvent| { handle_key(KeyEvent::KeyUp, event.key()); });

    //加载游戏资源
    js!{
        var map = new Map();
        map.set(@{RES_TANK_BITMAP}, "tank.png");
        map.set(@{RES_MISSILE_BITMAP}, "missile.png");
        map.set(@{RES_LG_EXPLOSION_BITMAP}, "lg_explosion.png");
        map.set(@{RES_SM_EXPLOSION__BITMAP}, "sm_explosion.png");
        loadResources(map, function(map){
            res_map = map;
            exports.on_resources_load();
        }, function(n1, n2){
            exports.on_load_resource_progress(n1, n2);
        });
    }

    stdweb::event_loop();
}

//处理按键事件
fn handle_key(event: KeyEvent, key: String){
    console!(log, format!("event={:?} key={}", event, key));
}

//窗口大小改变时，画布适应窗口
fn resize_window(canvas: CanvasElement){
    console!(log, "resize window!");
    let (width, height) = (window().inner_width(), window().inner_height());
    let (cwidth, cheight) = 
        if width < height{//竖屏
            (width, (width as f32/CLIENT_WIDTH as f32 * CLIENT_HEIGHT as f32) as i32)
        }else{//横屏
            ((height as f32/CLIENT_HEIGHT as f32 * CLIENT_WIDTH as f32) as i32, height)
        };

    js!{
        var canvas = @{canvas};
        canvas.style.width = @{cwidth}+"px";
        canvas.style.height = @{cheight}+"px";
        canvas.style.marginLeft = @{(width-cwidth)/2}+"px";
        canvas.style.marginTop = @{(height-cheight)/2}+"px";
    }
}