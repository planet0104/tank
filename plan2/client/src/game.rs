extern crate serde_json;
extern crate tank;
use std::cell::RefCell;
use tank::{KeyEvent, TankGame, CLIENT_HEIGHT, CLIENT_WIDTH};
use tank::{RES_LG_EXPLOSION_BITMAP, RES_MISSILE_BITMAP, RES_SM_EXPLOSION__BITMAP, RES_TANK_BITMAP};
use tank::utils::Timer;
use serde_json::Value;
use ::*;
use tank::{
    SpriteEvent,
    SpriteInfo,
    MSG_CONNECT,
    MSG_DISCONNECT,
    MSG_START,
    MSG_KEY_EVENT,
    MSG_MOUSE_EVENT,
    SERVER_MSG_EVENT,
    SERVER_MSG_UUID
};

struct Client {
    uuid: String,
    timer: Timer,
    game: TankGame,
    context: Context2D,
}

thread_local!{
    static CLIENT: RefCell<Client> = RefCell::new(Client{
        uuid: String::new(),
        timer:Timer::new(1, ||{ current_time_millis() }),
        game:TankGame::new(),
        context: Context2D{},
    });
}

pub fn start() {
    console_log("游戏启动!!!");
    set_canvas_width(CLIENT_WIDTH);
    set_canvas_height(CLIENT_HEIGHT);
    resize_window();
    set_canvas_font("32px 微软雅黑");

    set_on_window_resize_listener(|| {
        resize_window();
    });
    set_on_keydown_listener(|key| handle_key(KeyEvent::KeyDown, key));
    set_on_keyup_listener(|key| handle_key(KeyEvent::KeyUp, key));
    set_on_connect_listener(||{
        console_log("websocket 链接成功");
        //加入游戏
        send_json_message(json!([
            MSG_START,
            json!({"name" : "planet"})
        ]));
    });
    set_on_close_listener(||{
        console_log("websocket 链接断开");
    });
    set_on_message_listener(|msg| { handle_message(msg); });

    //加载游戏资源
    set_on_resource_load_listener(|num: i32, total: i32| {
        let percent = num as f32 / total as f32;
        let bar_width = (CLIENT_WIDTH as f32 / 1.5) as i32;
        let bar_height = bar_width / 10;
        let bar_left = CLIENT_WIDTH / 2 - bar_width / 2;
        let bar_top = CLIENT_HEIGHT / 2 - bar_height / 2;
        fill_style("rgb(200, 200, 200)");
        fill_rect(bar_left, bar_top, bar_width, bar_height);
        fill_style("rgb(120, 120, 255)");
        fill_rect(
            bar_left,
            bar_top,
            (bar_width as f32 * percent) as i32,
            bar_height,
        );
        fill_style("#ff0");
        fill_text(
            &format!("资源加载中({}/{})", num, total),
            bar_left + bar_width / 3,
            bar_top + bar_height / 2 + 10,
        );
        if num == total {
            //资源加载完成, 启动游戏循环
            request_animation_frame();
            connect("ws://127.0.0.1:8080");
        }
    });

    load_resource(json!({
        RES_TANK_BITMAP.to_string() : "tank.png",
        RES_MISSILE_BITMAP.to_string() : "missile.png",
        RES_LG_EXPLOSION_BITMAP.to_string() : "lg_explosion.png",
        RES_SM_EXPLOSION__BITMAP.to_string() : "sm_explosion.png"
    }));

    //游戏循环
    let frame_callback = |timestamp| {
        CLIENT.with(|c| {
            let mut client = c.borrow_mut();
            if client.timer.ready_for_next_frame() {
                client.game.update_sprites();
                client.game.draw(&client.context);
                console_log(&format!("游戏循环 {}", timestamp));
            }
            request_animation_frame();
        });
    };

    //添加事件
    set_frame_callback(frame_callback);
}

fn handle_message(msg:String){
    console_log(&format!("on_message:{}", msg));
    let value:Value = serde_json::from_str(&msg).unwrap();
    let array = value.as_array().unwrap();
    let msg_id = array[0].as_i64().unwrap() as isize;
    CLIENT.with(|c| {
        let mut client = c.borrow_mut();
        match msg_id{
            SERVER_MSG_EVENT => {
                //更新精灵
                let event = SpriteEvent::from_i64(array[1]["event"].as_i64().unwrap());
                let info:SpriteInfo = serde_json::from_value(array[1]["info"].clone()).unwrap();
                client.game.handle_server_event(event, info);
            },
            SERVER_MSG_UUID => {
                client.uuid = array[1].as_str().unwrap().to_string();
            }
            _ => {}
        }
    });
}

//处理按键事件
fn handle_key(event: KeyEvent, key: String) {
    console_log(&format!("event={:?} key={}", event, key));
}

//窗口大小改变时，画布适应窗口
fn resize_window() {
    let (width, height) = (window_inner_width() - 5, window_inner_height() - 5);
    let (cwidth, cheight) = if width < height {
        //竖屏
        (
            width,
            (width as f32 / CLIENT_WIDTH as f32 * CLIENT_HEIGHT as f32) as i32,
        )
    } else {
        //横屏
        (
            (height as f32 / CLIENT_HEIGHT as f32 * CLIENT_WIDTH as f32) as i32,
            height,
        )
    };

    set_canvas_style_width(cwidth);
    set_canvas_style_height(cheight);
    set_canvas_style_margin((width - cwidth) / 2, (height - cheight) / 2, 0, 0);
}
