extern crate rand;
extern crate uuid;
pub mod utils;
pub mod engine;
pub mod sprite;
use uuid::Uuid;
use engine::{GameContext, GameEngine};
use sprite::{BitmapRes, Rect, Point, Sprite, BA_DIE, BA_WRAP};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use utils::Timer;
use std::fmt::Display;
use std::fmt::Debug;
//socket消息
pub const MSG_CONNECT: i64 = 1;
pub const MSG_DISCONNECT: i64 = 2;
pub const MSG_START: i64 = 3;
pub const MSG_KEY_EVENT: i64 = 4;
pub const MSG_MOUSE_EVENT: i64 = 5;

//server发送给客户端的消息
pub const SERVER_MSG_ERR: isize = 0;
pub const SERVER_MSG_EVENT: isize = 1;
pub const SERVER_MSG_UUID: isize = 2;
pub const SERVER_MSG_DATA: isize = 3;
//游戏宽高
pub const CLIENT_WIDTH: i32 = 600;
pub const CLIENT_HEIGHT: i32 = 850;
pub const FPS: u32 = 20;

pub const VK_SPACE:i32 = 32;
pub const VK_LEFT:i32 = 37;
pub const VK_RIGHT:i32 = 39;
pub const VK_UP:i32 = 38;
pub const VK_DOWN:i32 = 40;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const RES_TANK_BITMAP: i32 = 0;
pub const RES_MISSILE_BITMAP: i32 = 1;
pub const RES_LG_EXPLOSION_BITMAP: i32 = 2;
pub const RES_SM_EXPLOSION_BITMAP: i32 = 3;

pub const TANK_VELOCITY: i32 = 7;
pub const MISSILE_VELOCITY: i32 = 10;

//pub const SERVER_IP:&str = "127.0.0.1:8080";
pub const SERVER_IP:&str = "50.3.18.60:8080";

pub enum MouseEvent {
    MouseMove,
    MouseClick,
}

pub const GMAE_TITLE: &'static str = "Tank";

#[derive(Debug)]
pub enum KeyEvent {
    KeyDown,
    KeyUp,
}

impl KeyEvent{
    pub fn from_i64(num:i64) -> KeyEvent{
        match num{
            0 => KeyEvent::KeyDown,
            1 => KeyEvent::KeyUp,
            _ => KeyEvent::KeyUp
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self{
            &KeyEvent::KeyDown => 0,
            &KeyEvent::KeyUp => 1,
        }
    }
}

#[derive(Debug)]
pub enum SpriteEvent {
    Add,
    Update,
    Delete,
}

impl SpriteEvent{
    pub fn from_i64(num:i64) -> SpriteEvent{
        match num{
            0 => SpriteEvent::Add,
            1 => SpriteEvent::Update,
            2 => SpriteEvent::Delete,
            _ => SpriteEvent::Update
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self{
            &SpriteEvent::Add => 0,
            &SpriteEvent::Update => 1,
            &SpriteEvent::Delete => 2,
        }
    }
}

#[derive(Debug)]
pub struct SpriteInfo {
    pub id: String,
    pub res_id: i32, //资源ID
    pub position: Rect,
    pub velocity: Point,
    pub current_frame: i32, //当前帧
    pub name: String,
    pub score: i32,
    pub killer_name: String,
}

#[derive(Clone)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub killer_name: String,
}

/*
游戏循环由服务器和客户端各自执行(HTML5中游戏循环需要调用request_animation_frame)
TankGame提供所有游戏更新方法

服务端只update() 方法、键盘、鼠标事件处理， 处理完之后将会产生message，message被分发给改各个客户端
客户端调用 update_sprites(), draw() 方法, handle_event方法(处理添加精灵、更新精灵、删除精灵)； 键盘事件发送给websocket
(客户端不处理碰撞检测, 服务器检测到碰撞会自动将精灵状态下发到客户端)
*/

thread_local!{
    pub static GAME: RefCell<TankGame> = RefCell::new(TankGame {
            engine: GameEngine::new(),
            server_events: vec![],
            server_players: HashMap::new(),
            client_player: None,
            client_context: None,
            //client_messages: vec![],
            client_timer: Timer::new(20.0),
            //client_key_events: vec![],
            client_last_time: 0.0,
            client_dying_delay: 0,
        });
}

pub struct TankGame {
    pub engine: GameEngine,
    client_context: Option<Rc<Box<GameContext>>>,
    server_events: Vec<(SpriteEvent, SpriteInfo)>,
    server_players: HashMap<String, Player>,
    client_player: Option<Player>,
    //client_messages: Vec<String>,
    client_timer: Timer,
    client_last_time: f64,
    //client_key_events: Vec<(KeyEvent, i32)>,
    client_dying_delay: i32,
}

impl TankGame {
    pub fn set_game_context(&mut self, context: Box<GameContext>){
        self.client_context = Some(Rc::new(context));
    }

    pub fn client_on_connect(&mut self){
        let context = self.client_context.as_ref().unwrap();
        context.console_log("websocket 链接成功");
        //加入游戏
        let name = context.prompt("请输入你的名字", "未命名");
        context.console_log(&format!("玩家姓名:{}", name));
        self.client_player = Some(Player{
            uuid: String::new(),
            name: name.clone(),
            killer_name: String::new(),
        });
        context.send_message(&format!("{}\n{}", 3, name));
    }

    pub fn client_on_resource_load(&self, num: i32, total: i32){
        let context = self.client_context.as_ref().unwrap();
        let percent = num as f32 / total as f32;
        let bar_width = (CLIENT_WIDTH as f32 / 1.5) as i32;
        let bar_height = bar_width / 10;
        let bar_left = CLIENT_WIDTH / 2 - bar_width / 2;
        let bar_top = CLIENT_HEIGHT / 2 - bar_height / 2;
        context.fill_style("rgb(200, 200, 200)");
        context.fill_rect(bar_left, bar_top, bar_width, bar_height);
        context.fill_style("rgb(120, 120, 255)");
        context.fill_rect(
            bar_left,
            bar_top,
            (bar_width as f32 * percent) as i32,
            bar_height,
        );
        context.fill_style("#ff0");
        context.fill_text(
            &format!("资源加载中({}/{})", num, total),
            bar_left + bar_width / 3,
            bar_top + bar_height / 2 + 10,
        );
        if num == total {
            //资源加载完成, 启动游戏循环
            context.request_animation_frame();
            //connect("ws://50.3.18.60:8080");
            context.connect(&format!("ws://{}", SERVER_IP));
        }
    }

    pub fn client_start(&mut self) {
        let context = self.client_context.as_ref().unwrap();
        context.console_log("游戏启动!!!");
        context.set_canvas_width(CLIENT_WIDTH);
        context.set_canvas_height(CLIENT_HEIGHT);
        self.client_resize_window();
        context.set_canvas_font("24px 微软雅黑");

        context.set_on_window_resize_listener(|| {
             GAME.with(|game|{
                game.borrow().client_resize_window();
            });
        });

        context.set_on_connect_listener(||{
            GAME.with(|game|{
                game.borrow_mut().client_on_connect();
            });
        });
        context.set_on_close_listener(||{
            GAME.with(|game|{
                game.borrow().client_context.as_ref().unwrap().alert("网络已断开!");
            });
        });

        // context.set_on_key_up_listener(|key|{
        //     GAME.with(|game|{
        //         game.borrow_mut().client_key_events.push((KeyEvent::KeyUp, key));
        //     });
        // });

        // context.set_on_key_down_listener(|key|{
        //     GAME.with(|game|{
        //         game.borrow_mut().client_key_events.push((KeyEvent::KeyDown, key));
        //     });
        // });

        // context.set_on_message_listener(|msg|{
        //     GAME.with(|game|{
        //         game.borrow_mut().client_messages.push(String::from(msg));
        //     });
        // });

        //加载游戏资源
        context.set_on_resource_load_listener(|num: i32, total: i32| {
            GAME.with(|game|{
                game.borrow().client_on_resource_load(num, total);
            });
        });
        
        context.load_resource(format!(r#"{{"{}":"tank.png","{}":"missile.png","{}":"lg_explosion.png","{}":"sm_explosion.png"}}"#,
            RES_TANK_BITMAP,
            RES_MISSILE_BITMAP,
            RES_LG_EXPLOSION_BITMAP,
            RES_SM_EXPLOSION_BITMAP));

        //游戏循环
        context.set_frame_callback(|timestamp:f64| {
            GAME.with(|game|{
                game.borrow_mut().client_update(timestamp);        
            });
        });
    }

    pub fn client_update(&mut self, timestamp:f64){
        let c = self.client_context.clone();
        let context = c.as_ref().unwrap();
        if self.client_timer.ready_for_next_frame(timestamp){
            //处理消息
            self.client_handle_message(context.pick_messages());
            //键盘事件
            let key_events = context.pick_key_events();
            for key_event in key_events{
                context.send_message(&format!("{}\n{}␟{}", 4, key_event.0.to_i64(), key_event.1));
            }

            //客户端不在update_sprites处理函数中做任何操作如:精灵死亡添加爆炸、碰撞检测杀死精灵等
            //客户端仅按帧更新精灵位置，所有精灵创建、更新都由服务器下发事件中处理
            self.engine.update_sprites(&mut |_, _| {}, |_, _, _| false);
            context.fill_style("#2e6da3");
            context.fill_rect(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);
            context.fill_style("#3e7daf");
            context.set_canvas_font("90px 微软雅黑");
            context.fill_text("坦克大战", CLIENT_WIDTH/2-185, CLIENT_HEIGHT/2-50);
            context.set_canvas_font("32px 微软雅黑");
            context.fill_text("↑ ↓ ← → ：移动  空格：开炮", 100, CLIENT_HEIGHT/2+30);
            //context.fill_text("(死了请刷新网页)", 180, CLIENT_HEIGHT/2+80);
            self.engine.draw_sprites(context.clone());
            if self.client_last_time > 0.0 {
                let frame_time = timestamp-self.client_last_time;
                context.fill_style("#fff");
                context.set_canvas_font("24px 微软雅黑");
                context.fill_text(&format!("FPS:{:0.1}", 1000.0/frame_time), 10, 30);
            }
            //死亡倒计时
            if self.client_dying_delay > 0{
                context.fill_style("#ddf");
                context.set_canvas_font("36px 微软雅黑");
                context.fill_text(&format!("被[{}]炸死", self.client_player.as_ref().unwrap().killer_name), CLIENT_WIDTH/2-185, CLIENT_HEIGHT/2-50);
                context.fill_text(&format!("{}秒之后重生", self.client_dying_delay/self.client_timer.fps() as i32), CLIENT_WIDTH/2-185, CLIENT_HEIGHT/2-10);
                self.client_dying_delay -= 1;
                if self.client_dying_delay <= 0{
                    //重新加入游戏
                    let player = self.client_player.as_mut().unwrap();
                    //player.uuid = String::new();
                    context.send_message(&format!("{}\n{}", 3, player.name));
                }
            }
            self.client_last_time = timestamp;
        }
        context.request_animation_frame(); 
    }

    //玩家加入游戏
    pub fn server_join_game(&mut self, id: String, name: String) {
        println!("join_game: {} {}", id, name.clone());
        //添加坦克精灵
        let sprite_index =
            TankGame::add_sprite(&mut self.engine, id.clone(), RES_TANK_BITMAP, true);
        //添加玩家信息
        self.engine.sprites()[sprite_index].set_name(name.clone());
        self.server_players.insert(
            self.engine.sprites()[sprite_index].id.clone(),
            Player {
                uuid: self.engine.sprites()[sprite_index].id.clone(),
                name: name.clone(),
                killer_name: String::new(),
            },
        );
        self.server_events.push(TankGame::get_event_info(SpriteEvent::Add, &self.engine.sprites()[sprite_index]));//添加事件
        println!("join_game {} {} 在线人数:{}", id, name, self.server_players.len());
    }

    //离开游戏/断线
    pub fn server_leave_game(&mut self, id: &String) {
        //查找玩家id对应的精灵, 将其删除
        self.server_players.remove(id);
        if let Some(index) = self.engine.query_sprite_idx(id) {
            self.server_events.push(TankGame::get_event_info(SpriteEvent::Delete, &self.engine.sprites()[index]));//事件
            self.engine.sprites().remove(index); //直接删除, 不kill
        }
        println!("leave_game {} 在线人数:{}", id, self.server_players.len());
    }

    //创建游戏精灵
    pub fn add_sprite(engine: &mut GameEngine, id: String, res: i32, rand_pos:bool) -> usize {
        match res {
            RES_TANK_BITMAP => {
                //创建玩家坦克
                let mut tank_sprite = 
                if rand_pos{
                    Sprite::with_bounds_action(
                        id,
                        BitmapRes::new(RES_TANK_BITMAP, 36, 144),
                        Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                        BA_WRAP
                    )
                }else{
                    Sprite::with_bounds_action_norand(
                        id,
                        BitmapRes::new(RES_TANK_BITMAP, 36, 144),
                        Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                        BA_WRAP
                    )
                };
                tank_sprite.set_num_frames(4, false);
                tank_sprite.set_frame_delay(-1);
                engine.add_sprite(tank_sprite)
            }
            RES_MISSILE_BITMAP => {
                //创建一个新的子弹精灵
                let mut sprite =
                if rand_pos{
                    Sprite::with_bounds_action(
                        id,
                        BitmapRes::new(RES_MISSILE_BITMAP, 17, 68),
                        Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                        BA_DIE
                    )
                }else{
                    Sprite::with_bounds_action_norand(
                        id,
                        BitmapRes::new(RES_MISSILE_BITMAP, 17, 68),
                        Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                        BA_DIE
                    )
                };
                sprite.set_num_frames(4, false);
                sprite.set_frame_delay(-1);
                engine.add_sprite(sprite)
            }
            RES_SM_EXPLOSION_BITMAP => {
                //创建小的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    id,
                    BitmapRes::new(RES_SM_EXPLOSION_BITMAP, 17, 136),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT)
                );
                sprite.set_num_frames(8, true);
                engine.add_sprite(sprite)
            }
            RES_LG_EXPLOSION_BITMAP => {
                //创建一个大的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    id,
                    BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                );
                sprite.set_num_frames(8, true);
                engine.add_sprite(sprite)
            }
            _ => 0,
        }
    }

    //客户端接受到服务器发送来的消息，将消息传递给此方法，来更新渲染
    pub fn client_handle_server_event(&mut self, event: SpriteEvent, sprite_info: SpriteInfo) {
        if let Some(sprite_idx) = match event {
            SpriteEvent::Add => Some(TankGame::add_sprite(
                &mut self.engine,
                sprite_info.id,
                sprite_info.res_id,
                false
            )),
            SpriteEvent::Update => self.engine.query_sprite_idx(&sprite_info.id),
            SpriteEvent::Delete => {
                if let Some(sprite) = self.engine.query_sprite(&sprite_info.id) {
                    sprite.kill();
                }
                None
            }
        } {
            //设置精灵信息
            let mut sprite = &mut self.engine.sprites()[sprite_idx];
            sprite.set_position_rect(sprite_info.position);
            sprite.set_velocity_point(&sprite_info.velocity);
            sprite.set_current_frame(sprite_info.current_frame);
            sprite.set_name(sprite_info.name);
            sprite.set_score(sprite_info.score);
        }
    }

    //更新游戏
    pub fn server_update(&mut self) {
        let mut sprites_dying_events = vec![];
        let mut sprites_add_events = vec![];

        //更新游戏，并处理精灵死亡、碰撞检测回调
        self.engine.update_sprites(
            &mut |engine: &mut GameEngine, idx_sprite_dying| {
                sprites_dying_events.push(TankGame::get_event_info(SpriteEvent::Delete, &engine.sprites()[idx_sprite_dying]));
                let bitmap_id = engine.sprites()[idx_sprite_dying].bitmap().id();
                //子弹精灵死亡添加小的爆炸精灵
                if bitmap_id == RES_MISSILE_BITMAP{
                    let idx = TankGame::add_sprite(engine, Uuid::new_v4().hyphenated().to_string(), RES_SM_EXPLOSION_BITMAP, true);
                    let pos = *engine.sprites()[idx_sprite_dying].position();
                    engine.sprites()[idx].set_position(pos.left, pos.top);
                    sprites_add_events.push(TankGame::get_event_info(SpriteEvent::Add, &engine.sprites()[idx]));
                }
                //坦克死亡添加大的爆炸精灵
                if bitmap_id == RES_TANK_BITMAP{
                    let idx = TankGame::add_sprite(engine, Uuid::new_v4().hyphenated().to_string(), RES_LG_EXPLOSION_BITMAP, true);
                    let pos = *engine.sprites()[idx_sprite_dying].position();
                    engine.sprites()[idx].set_position(pos.left, pos.top);
                    sprites_add_events.push(TankGame::get_event_info(SpriteEvent::Add, &engine.sprites()[idx]));
                    //增加凶手得分
                    let killer = engine.sprites()[idx_sprite_dying].killer();
                    if let Some(killer) = engine.query_sprite(&killer){
                        killer.add_score();
                        sprites_add_events.push(TankGame::get_event_info(SpriteEvent::Update, &killer));
                    }
                }
            },
            |engine, idx_sprite_hitter, idx_sprite_hittee| {
                //此处杀死的精灵, 会在下次更新时，调用上边sprite_dying函数
                //碰撞检测
                
                let (hitter_res, hitter_id, hitter_parent) = {
                    let hitter = &engine.sprites()[idx_sprite_hitter];
                    (hitter.bitmap().id(), hitter.id.clone(), hitter.parent.clone().unwrap_or("".to_string()))
                };
                let (hittee_res, hittee_id, hittee_parent) = {
                    let hittee = &engine.sprites()[idx_sprite_hittee];
                    (hittee.bitmap().id(), hittee.id.clone(), hittee.parent.clone().unwrap_or("".to_string()))
                };
                if hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_TANK_BITMAP {
                    //玩家碰到自己发射的子弹不会爆炸
                    if &hitter_parent == &hittee_id{
                        false
                    }else{
                        //子弹对应的玩家加分
                        let killer_name = if let Some(killer) = engine.query_sprite(&hitter_parent){
                            killer.name().clone()
                        }else{
                            String::new()
                        };
                        engine.sprites()[idx_sprite_hittee].set_killer(hitter_parent, killer_name);
                        //杀死相撞的子弹和坦克
                        engine.kill_sprite(idx_sprite_hittee);
                        engine.kill_sprite(idx_sprite_hitter);
                        true
                    }
                } else if hitter_res == RES_TANK_BITMAP && hittee_res == RES_MISSILE_BITMAP {
                    //玩家碰到自己发射的子弹不会爆炸
                    if &hittee_parent == &hitter_id{
                        false
                    }else{
                        //子弹对应的玩家加分
                        let killer_name = if let Some(killer) = engine.query_sprite(&hittee_parent){
                            killer.name().clone()
                        }else{
                            String::new()
                        };
                        engine.sprites()[idx_sprite_hitter].set_killer(hittee_parent, killer_name);
                        //杀死相撞的子弹和坦克
                        engine.kill_sprite(idx_sprite_hittee);
                        engine.kill_sprite(idx_sprite_hitter);
                        true
                    }
                } else if hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_MISSILE_BITMAP {
                    //检测子弹和子弹是否碰撞
                    //同一个玩家的子弹不会碰撞
                    if &hitter_parent != &hittee_parent{
                        engine.kill_sprite(idx_sprite_hittee);
                        engine.kill_sprite(idx_sprite_hitter);
                        true
                    }else{
                        false
                    }
                } else {
                    false
                }
            },
        );

        //添加精灵事件
        for e in sprites_add_events {
            self.server_events.push(e);
        }
        //删除精灵事件
        for e in sprites_dying_events {
            //坦克死亡将玩家删除
            if e.1.res_id == RES_TANK_BITMAP {
                self.server_players.remove(&e.1.id);
            }
            self.server_events.push(e);
        }
    }

    //添加要分发的事件
    fn get_event_info(event: SpriteEvent, sprite: &Sprite) -> (SpriteEvent, SpriteInfo) {
        (   event,
            SpriteInfo {
                id: sprite.id.clone(),
                res_id: sprite.bitmap().id(),
                position: sprite.position().clone(),
                velocity: sprite.velocity().clone(),
                current_frame: sprite.current_frame(),
                name: sprite.name().clone(),
                score: sprite.score(),
                killer_name: sprite.killer_name().clone()
            }
        )
    }

    //键盘按下，坦克移动、发射子弹
    pub fn on_key_event(&mut self, event: KeyEvent, key: i32, sprite_id: &String) {
        if let Some(idx) = self.engine.query_sprite_idx(sprite_id) {
            match event {
                KeyEvent::KeyDown => {
                    match key {
                        VK_SPACE => {
                            let tank_position = *(self.engine.sprites()[idx].position());
                            //创建一个新的子弹精灵
                            let missile_idx =
                                TankGame::add_sprite(&mut self.engine, Uuid::new_v4().hyphenated().to_string(), RES_MISSILE_BITMAP, true);
                            
                            //子弹的方向同玩家的方向
                            let direction = self.engine.sprites()[idx].current_frame();
                            {let mut missile = &mut self.engine.sprites()[missile_idx];
                            missile.set_current_frame(direction);
                            missile.parent = Some(sprite_id.clone());//记住玩家发射的子弹
                            match direction {
                                0 => {//上
                                    missile.set_velocity(0, -MISSILE_VELOCITY);
                                    missile.set_position(
                                        tank_position.left
                                            + (tank_position.right - tank_position.left) / 2
                                            - 8,
                                        tank_position.top - 15,
                                    );
                                }
                                1 => {//下
                                    missile.set_velocity(0, MISSILE_VELOCITY);
                                    missile.set_position(
                                        tank_position.left
                                            + (tank_position.right - tank_position.left) / 2
                                            - 8,
                                        tank_position.bottom,
                                    );
                                }
                                2 => {//左
                                    missile.set_velocity(-MISSILE_VELOCITY, 0);
                                    missile.set_position(
                                        tank_position.left - 15,
                                        tank_position.top
                                            - (tank_position.top - tank_position.bottom) / 2
                                            - 8,
                                    );
                                }
                                3 => {//右
                                    missile.set_velocity(MISSILE_VELOCITY, 0);
                                    missile.set_position(
                                        tank_position.right,
                                        tank_position.top
                                            - (tank_position.top - tank_position.bottom) / 2
                                            - 8,
                                    );
                                }
                                _ => {}
                            }}
                            self.server_events.push(TankGame::get_event_info(SpriteEvent::Add, &self.engine.sprites()[missile_idx]));
                        }
                        VK_LEFT => {
                            self.engine.sprites()[idx].set_current_frame(2);
                            self.engine.sprites()[idx].set_velocity(-TANK_VELOCITY, 0);
                            self.server_events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        VK_RIGHT => {
                            self.engine.sprites()[idx].set_current_frame(3);
                            self.engine.sprites()[idx].set_velocity(TANK_VELOCITY, 0);
                            self.server_events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        VK_UP => {
                            self.engine.sprites()[idx].set_current_frame(0);
                            self.engine.sprites()[idx].set_velocity(0, -TANK_VELOCITY);
                            self.server_events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        VK_DOWN => {
                            self.engine.sprites()[idx].set_current_frame(1);
                            self.engine.sprites()[idx].set_velocity(0, TANK_VELOCITY);
                            self.server_events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        other => {
                            println!("未定义按键 {}", other);
                        }
                    }
                }

                KeyEvent::KeyUp => {
                    //键盘弹起坦克停止走动
                    let do_update = {
                        match key {
                            VK_LEFT
                            | VK_RIGHT
                            | VK_UP
                            | VK_DOWN => {
                                self.engine.sprites()[idx].set_velocity(0, 0);
                                true
                            }
                            _ => false,
                        }
                    };
                    if do_update {
                        self.server_events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                    }
                }
            }
        }else{
            println!("没有找到ID {}", sprite_id);
        }
    }

    pub fn events(&mut self) -> &mut Vec<(SpriteEvent, SpriteInfo)> {
        &mut self.server_events
    }

    pub fn sprites(&mut self) -> &Vec<Sprite> {
        self.engine.sprites()
    }

    //窗口大小改变时，画布适应窗口
    fn client_resize_window(&self) {
        let context = self.client_context.as_ref().unwrap();
        let (width, height) = (context.window_inner_width() - 5, context.window_inner_height() - 5);
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

        context.set_canvas_style_width(cwidth);
        context.set_canvas_style_height(cheight);
        context.set_canvas_style_margin((width - cwidth) / 2, (height - cheight) / 2, 0, 0);
    }

    fn client_handle_message(&mut self, messages:Vec<String>){
        for msg in messages{
            let c = self.client_context.clone();
            let context = c.as_ref().unwrap();
            //console_log(&format!("handle_message {}", msg));
            
            let lf = msg.find('\n');
            if !lf.is_some(){
                context.console_log("lf空");
                return;
            }
            let lf = lf.unwrap();
            let msg_id = msg.get(0..lf);
            if msg_id.is_none(){
                context.console_log("msg_id空");
                return;
            }
            let msg_id = msg_id.unwrap().parse::<isize>();
            if msg_id.is_err() {
                context.console_log("msg_id解析失败");
                return;
            }
            let msg_id = msg_id.unwrap();
            let data = msg.get(lf+1..).unwrap_or("");
            //console_log(&format!("handle_message  msg_id={} data={}", msg_id, data));

            match msg_id{
                SERVER_MSG_ERR => {
                    context.console_log(&format!("服务器错误:{}", data));
                }
                SERVER_MSG_EVENT => {
                    //console_log("lock 成功>>>31.");
                    //更新精灵
                    let events:Vec<&str> = data.clone().split('\n').collect();
                // console_log("lock 成功>>>32.");
                    for value in events{
                        //EventId␟ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame
                        let items:Vec<&str> = value.split('␟').collect();
                        if items.len() != 13{ return; }
                        if let Ok(event_id) = items[0].parse::<i64>(){
                            let event = SpriteEvent::from_i64(event_id);
                            let info = SpriteInfo{
                                id: items[1].to_string(),
                                res_id: items[2].parse::<i32>().unwrap_or(-1),
                                position: Rect{
                                    left: items[3].parse::<i32>().unwrap_or(0),
                                    top: items[4].parse::<i32>().unwrap_or(0),
                                    right: items[5].parse::<i32>().unwrap_or(0),
                                    bottom: items[6].parse::<i32>().unwrap_or(0),
                                },
                                velocity: Point{
                                    x: items[7].parse::<i32>().unwrap_or(0),
                                    y: items[8].parse::<i32>().unwrap_or(0),
                                },
                                current_frame: items[9].parse::<i32>().unwrap_or(0),
                                name: String::from(items[10]),
                                score: items[11].parse::<i32>().unwrap_or(0),
                                killer_name: items[12].to_string()
                            };

                            //检查玩家是否死亡
                            match event{
                                SpriteEvent::Delete => {
                                    let mut player = self.client_player.as_mut().unwrap();
                                    if info.id == player.uuid{
                                        player.killer_name = info.killer_name.clone();
                                        //alert(client.name.as_ref().unwrap());
                                        //alert("你死了!");
                                        self.client_dying_delay = 100;
                                    }
                                }
                                _ => {}
                            }

                            self.client_handle_server_event(event, info);
                        }
                    }
                    //console_log("lock 成功>>>33.");
                    //console_log("更新精灵-2");
                },
                SERVER_MSG_UUID => {
                    self.client_player.as_mut().unwrap().uuid = data.to_string();
                    context.console_log(&format!("client.uuid={}", self.client_player.as_ref().unwrap().uuid));
                },
                SERVER_MSG_DATA => {
                    context.console_log("绘制所有精灵");
                    //绘制所有精灵
                    //ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame\n
                    let sprites:Vec<&str> = data.split('\n').collect();
                    for sprite in sprites{
                        let items:Vec<&str> = sprite.split('␟').collect();
                        if items.len() != 12{ return; }
                        let info = SpriteInfo{
                            id: items[0].to_string(),
                            res_id: items[1].parse::<i32>().unwrap_or(0),
                            position: Rect{
                                left: items[2].parse::<i32>().unwrap_or(0),
                                top: items[3].parse::<i32>().unwrap_or(0),
                                right: items[4].parse::<i32>().unwrap_or(0),
                                bottom: items[5].parse::<i32>().unwrap_or(0),
                            },
                            velocity: Point{
                                x: items[6].parse::<i32>().unwrap_or(0),
                                y: items[7].parse::<i32>().unwrap_or(0),
                            },
                            current_frame: items[8].parse::<i32>().unwrap_or(0),
                            name: String::from(items[9]),
                            score: items[10].parse::<i32>().unwrap_or(0),
                            killer_name: items[11].to_string()
                        };
                        self.client_handle_server_event(SpriteEvent::Add, info);
                    }
                }
                _ => {}
            }
        }
    }

    fn console_log_1<A:Display+Debug, B:Display+Debug>(&self, msg: A, obj:B){
        let msg = format!("{:?} {:?}", msg, obj);
        self.client_context.as_ref().unwrap().console_log(&msg);
    }

    fn console_log_2<A:Display+Debug, B:Display+Debug, C:Display+Debug>(&self, msg: A, obj:B, obj2:C) {
        let msg = format!("{:?} {:?} {:?}", msg, obj, obj2);
        self.client_context.as_ref().unwrap().console_log(&msg);
    }
}