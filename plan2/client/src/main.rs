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
use std::cell::RefCell;
use tank::{TankGame, KeyEvent, CLIENT_WIDTH, CLIENT_HEIGHT};
use tank::{RES_TANK_BITMAP, RES_MISSILE_BITMAP, RES_LG_EXPLOSION_BITMAP, RES_SM_EXPLOSION__BITMAP};
use tank::utils::Timer;
use tank::engine::CanvasContext;

// Shamelessly stolen from webplatform's TodoMVC example.
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

struct Env{
    timer: Timer,
    game: TankGame,
    frame_callback: fn(f64)
}

struct Context2D{
    context: CanvasRenderingContext2d
}

impl CanvasContext for Context2D{
    fn draw_image_at(&self, res_id:i32, x:i32, y:i32){
        //ctx.drawImage(res_map.get(resId), x, y);
    }

    fn draw_image(&self, res_id:i32, source_x:i32, source_y:i32, source_width:i32, source_height:i32, dest_x:i32, dest_y:i32, dest_width:i32, dest_height:i32){

    }

    fn fill_style(&self, style: &str){

    }

    fn fill_rect(&self, x:i32, y:i32, width:i32, height:i32){

    }

    fn fill_text(&self, text: &str, x:i32, y:i32){

    }
}

thread_local!{
    static ENV: RefCell<Env> = RefCell::new(Env{
        timer:Timer::new(1),
        game:TankGame::new(),

        //游戏循环
        frame_callback: |timestamp| {
            ENV.with(|e| {
                let mut env = e.borrow_mut();
                if env.timer.ready_for_next_frame(){
                    env.game.update_sprites();
                    let text = document().query_selector( "#console" ).unwrap().unwrap();
                    js!( @{text}.innerText = @{format!("frame_callback {}", timestamp)}; );
                }
                window().request_animation_frame(env.frame_callback);
            });
        }
    });
}

fn main(){
    stdweb::initialize();

    //let window = window();
    let canvas: CanvasElement = document().query_selector("#canvas").unwrap().unwrap().try_into().unwrap();
    //let context: CanvasRenderingContext2d = canvas.get_context().unwrap();
    //canvas.set_width(CLIENT_WIDTH as u32);
    //canvas.set_height(CLIENT_HEIGHT as u32);
    //resize_window(canvas.clone());

    // //添加事件
    // window.add_event_listener( enclose!( (canvas) move |_: ResizeEvent| { resize_window(canvas.clone()); }));
    // window.add_event_listener(|event: KeyDownEvent| { handle_key(KeyEvent::KeyDown, event.key()); });
    // window.add_event_listener(|event: KeyUpEvent| { handle_key(KeyEvent::KeyUp, event.key()); });

    // //加载图片资源

    // //启动游戏循环
    // ENV.with(|e| {
    //     window.request_animation_frame(e.borrow().frame_callback);
    // });

    //加载游戏资源
    // let mut urls = HashMap::new();
    // urls.insert(RES_TANK_BITMAP, "tank.png");
    // urls.insert(RES_MISSILE_BITMAP, "missile.png");
    // urls.insert(RES_LG_EXPLOSION_BITMAP, "lg_explosion.png");
    // urls.insert(RES_SM_EXPLOSION__BITMAP, "sm_explosion.png");
    // js!{
    //     var urls = JSON.parse(@{serde_json::to_string(&urls).unwrap()});
    //     loadResources(urls, function(map, num, total){
    //         if(num == total){
    //             //资源加载完成 启动游戏循环
    //             window.resMap = map;
    //             window.fcb = function(){
    //                 @{frame_callback}();
    //             };
    //             window.requestAnimationFrame(window.fcb);
    //         }
    //     });
    // }

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