extern crate tank;
use std::cell::RefCell;
use tank::{KeyEvent, TankGame, CLIENT_HEIGHT, CLIENT_WIDTH};
use tank::{RES_LG_EXPLOSION_BITMAP, RES_MISSILE_BITMAP, RES_SM_EXPLOSION__BITMAP, RES_TANK_BITMAP};
use tank::utils::Timer;
use tank::sprite::{Rect, Point};
use ::*;
use tank::{
    SpriteEvent,
    SpriteInfo,
    MSG_CONNECT,
    MSG_DISCONNECT,
    MSG_START,
    MSG_KEY_EVENT,
    MSG_MOUSE_EVENT,
    SERVER_MSG_ERR,
    SERVER_MSG_EVENT,
    SERVER_MSG_UUID,
    SERVER_MSG_DATA
};

struct Client {
    uuid: String,
    name: Option<String>,
    timer: Timer,
    game: TankGame,
    context: Context2D,
}

thread_local!{
    static CLIENT: RefCell<Client> = RefCell::new(Client{
        uuid: String::new(),
        name: None,
        timer:Timer::new(30),
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
        CLIENT.with(|c| {
            let mut client = c.borrow_mut();
            let name = format!("Player{}", random());
            client.name = Some(name.clone()); 
            send_message(&format!("{}\n{}", MSG_START, name));
        });
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
    
    load_resource(format!(r#"{{"{}":"tank.png","{}":"missile.png","{}":"lg_explosion.png","{}":"sm_explosion.png"}}"#,
        RES_TANK_BITMAP,
        RES_MISSILE_BITMAP,
        RES_LG_EXPLOSION_BITMAP,
        RES_SM_EXPLOSION__BITMAP));

    //游戏循环
    let frame_callback = |_timestamp| {
        CLIENT.with(|c| {
            let mut client = c.borrow_mut();
            if client.timer.ready_for_next_frame() {
                client.game.update_sprites();
                client.game.draw(&client.context);
            }
            request_animation_frame();
        });
    };

    //添加事件
    set_frame_callback(frame_callback);
}

//服务器下发的消息不用验证
fn handle_message(msg: &str){
    
    let lf = msg.find('\n');

    let msg_id = if lf.is_some() {
        msg.get(0..lf.unwrap()).unwrap().parse::<isize>().unwrap()
    }else{
        msg.parse::<isize>().unwrap()
    };
    let data = if lf.is_some() {
        msg.get(lf.unwrap()+1..).unwrap()
    }else{
        ""
    };

    CLIENT.with(|c| {
        let mut client = c.borrow_mut();
        match msg_id{
            SERVER_MSG_ERR => {
                console_log(&format!("服务器错误:{}", data));
            }
            SERVER_MSG_EVENT => {
                console_log("更新精灵");
                //更新精灵
                let events:Vec<&str> = data.split('\n').collect();
                for value in events{
                    //EventId␟ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame
                    let items:Vec<&str> = value.split('␟').collect();
                    let event = SpriteEvent::from_i64(items[0].parse::<i64>().unwrap());
                    let info = SpriteInfo{
                        id: items[1].to_string(),
                        res_id: items[2].parse::<i32>().unwrap(),
                        position: Rect{
                            left: items[3].parse::<i32>().unwrap(),
                            top: items[4].parse::<i32>().unwrap(),
                            right: items[5].parse::<i32>().unwrap(),
                            bottom: items[6].parse::<i32>().unwrap(),
                        },
                        velocity: Point{
                            x: items[7].parse::<i32>().unwrap(),
                            y: items[8].parse::<i32>().unwrap(),
                        },
                        current_frame: items[9].parse::<i32>().unwrap()
                    };

                    //检查玩家是否死亡
                    match event{
                        SpriteEvent::Delete => {
                            if info.id == client.uuid{
                                //alert(client.name.as_ref().unwrap());
                                alert("你死了!");
                            }
                        }
                        _ => {}
                    }

                    client.game.handle_server_event(event, info);
                }
            },
            SERVER_MSG_UUID => {
                client.uuid = data.to_string();
                console_log(&format!("client.uuid={}", client.uuid));
            },
            SERVER_MSG_DATA => {
                console_log("绘制所有精灵");
                //绘制所有精灵
                //ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame\n
                let sprites:Vec<&str> = data.split('\n').collect();
                for sprite in sprites{
                    let items:Vec<&str> = sprite.split('␟').collect();
                    let info = SpriteInfo{
                        id: items[0].to_string(),
                        res_id: items[1].parse::<i32>().unwrap(),
                        position: Rect{
                            left: items[2].parse::<i32>().unwrap(),
                            top: items[3].parse::<i32>().unwrap(),
                            right: items[4].parse::<i32>().unwrap(),
                            bottom: items[5].parse::<i32>().unwrap(),
                        },
                        velocity: Point{
                            x: items[6].parse::<i32>().unwrap(),
                            y: items[7].parse::<i32>().unwrap(),
                        },
                        current_frame: items[8].parse::<i32>().unwrap()
                    };
                    client.game.handle_server_event(SpriteEvent::Add, info);
                }
            }
            _ => {}
        }
    });
}

//处理按键事件
fn handle_key(event: KeyEvent, key: &str) {
    send_message(&format!("{}\n{}␟{}", MSG_KEY_EVENT, event.to_i64(), key));
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
