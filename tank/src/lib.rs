extern crate rand;
pub mod utils;
pub mod engine;
pub mod sprite;
pub mod vector_2d;
extern crate bincode;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use bincode::{deserialize, serialize};
use utils::rand_int;
use engine::{GameContext, GameEngine, UpdateCallback};
use sprite::{BitmapRes, PointF, Rect, Sprite, BA_DIE, BA_WRAP};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Debug;
use vector_2d::Vector2D;
use std::time::{Duration, Instant};

//socket消息
pub const MSG_DISCONNECT: u8 = 1;
pub const MSG_START: u8 = 2;
pub const MSG_KEY_EVENT: u8 = 3;

//server发送给客户端的消息
pub const SERVER_MSG_ERR: u8 = 0;
pub const SERVER_MSG_SYNC: u8 = 1;
pub const SERVER_MSG_UID: u8 = 2;

pub const DRIVE_THRESHOLD: i32 = 3;
//游戏宽高
pub const CLIENT_WIDTH: i32 = 600;
pub const CLIENT_HEIGHT: i32 = 900;

pub const VK_SPACE: i32 = 32;
pub const VK_LEFT: i32 = 37;
pub const VK_RIGHT: i32 = 39;
pub const VK_UP: i32 = 38;
pub const VK_DOWN: i32 = 40;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const RES_TANK_BITMAP: u8 = 0;
pub const RES_MISSILE_BITMAP: u8 = 1;
pub const RES_LG_EXPLOSION_BITMAP: u8 = 2;
pub const RES_SM_EXPLOSION_BITMAP: u8 = 3;
pub const RES_SM_GUN_BITMAP: u8 = 4;
pub const RES_NURSE_BITMAP: u8 = 5;

pub const SPRITE_UPDATE_FPS: u32 = 5;
pub const TANK_VELOCITY: f64 = 0.3;
pub const MISSILE_VELOCITY: f64 = 0.5;
pub const PLAYER_LIVES: u32 = 6; //生命值
pub const TANK_BITMAP_WIDTH: i32 = 57;
pub const TANK_BITMAP_HEIGHT: i32 = 57;
pub const SERVER_SYNC_DELAY: u64 = 66; //15帧刷新速度, 20人在线, 每次广播1K数据, 每秒广播15Kx20=300K数据,  100人1.5M/S?

// pub const SERVER_IP:&str = "127.0.0.1:8080";
// pub const CLIENT_IP:&str = "127.0.0.1:8080";

//pub const SERVER_IP:&str = "192.168.192.122:8080";

// pub const SERVER_IP:&str = "192.168.1.108:8080";
// pub const CLIENT_IP:&str = "192.168.1.108:8080";

pub const SERVER_IP: &str = "172.31.33.204:8414";
pub const CLIENT_IP: &str = "54.249.68.59:8414";

//pub const GMAE_TITLE: &'static str = "Tank";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KeyEvent {
    KeyDown,
    KeyUp,
}

impl KeyEvent {
    pub fn from_i64(num: i64) -> KeyEvent {
        match num {
            0 => KeyEvent::KeyDown,
            1 => KeyEvent::KeyUp,
            _ => KeyEvent::KeyUp,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
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

impl SpriteEvent {
    pub fn from_i64(num: i64) -> SpriteEvent {
        match num {
            0 => SpriteEvent::Add,
            1 => SpriteEvent::Update,
            2 => SpriteEvent::Delete,
            _ => SpriteEvent::Update,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            &SpriteEvent::Add => 0,
            &SpriteEvent::Update => 1,
            &SpriteEvent::Delete => 2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncData {
    data: Vec<SData>,
    ext: Vec<PlayerData>,
}

//精灵信息
#[derive(Serialize, Deserialize, Debug)]
pub struct SData {
    pub id: u32,
    pub x: i16,
    pub y: i16,
    pub res: u8,
    pub frame: u8,
    pub velocity_x: f32,
    pub velocity_y: f32,
}

//精灵附加信息
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerData {
    pub id: u32,
    pub name: String,
    pub score: u16,
    pub killer_name: String,
    pub lives: u16,
}

#[derive(Clone)]
pub struct Player {
    pub id: u32,
    pub ip: String,
    pub name: String,
    pub killer_name: String,
    pub score: i32,
}

impl Player {
    pub fn empty() -> Player {
        Player {
            id: 0,
            ip: String::new(),
            name: String::new(),
            killer_name: String::new(),
            score: 0,
        }
    }
}

/*
游戏循环由服务器和客户端各自执行(HTML5中游戏循环需要调用request_animation_frame)
TankGame提供所有游戏更新方法

服务端只update() 方法、键盘、鼠标事件处理， 处理完之后将会产生message，message被分发给改各个客户端
客户端调用 update_sprites(), draw() 方法, handle_event方法(处理添加精灵、更新精灵、删除精灵)； 键盘事件发送给websocket
(客户端不处理碰撞检测, 服务器检测到碰撞会自动将精灵状态下发到客户端)
*/

thread_local!{
    pub static GAME: RefCell<TankGame> = RefCell::new(TankGame::new());
}
//客户端游戏更新(不做任何处理)
pub struct ClientUpdateCallback {}
impl UpdateCallback for ClientUpdateCallback {
    fn on_sprite_dying(&mut self, _engine: &mut GameEngine, _idx_sprite_dying: usize) {}
    fn on_sprite_collision(
        &mut self,
        engine: &mut GameEngine,
        idx_sprite_hitter: usize,
        idx_sprite_hittee: usize,
    ) -> bool {
        let (hitter_res, hitter_id, hitter_parent) = {
            let hitter = &engine.sprites()[idx_sprite_hitter];
            (hitter.bitmap().id(), hitter.id.clone(), hitter.parent_id)
        };
        let (hittee_res, hittee_id, hittee_parent) = {
            let hittee = &engine.sprites()[idx_sprite_hittee];
            (hittee.bitmap().id(), hittee.id.clone(), hittee.parent_id)
        };
        if hittee_res == RES_TANK_BITMAP && hitter_res == RES_TANK_BITMAP {
            //坦克之间不能互相穿过
            engine.sprites()[idx_sprite_hittee].set_velocity(0.0, 0.0);
            true
        }else if hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_TANK_BITMAP {
            //子弹碰撞到其他坦克 停止移动
            ! hitter_parent == hittee_id
        }else if hitter_res == RES_TANK_BITMAP && hittee_res == RES_MISSILE_BITMAP{
            ! hittee_parent == hitter_id
        }else if hitter_res == RES_NURSE_BITMAP && hittee_res == RES_MISSILE_BITMAP
            || hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_NURSE_BITMAP
        {
            //子弹和护士碰撞，停止走动
            true
        }
         else {
            false
        }
    }
}

//服务器端游戏更新
pub struct ServerUpdateCallback {
    extras: Rc<RefCell<Vec<(PlayerData)>>>,
}
impl UpdateCallback for ServerUpdateCallback {
    fn on_sprite_dying(&mut self, engine: &mut GameEngine, idx_sprite_dying: usize) {
        let bitmap_id = engine.sprites()[idx_sprite_dying].bitmap().id();
        //子弹精灵死亡添加小的爆炸精灵
        if bitmap_id == RES_MISSILE_BITMAP {
            let sid = engine.next_sprite_id();
            let pos = *engine.sprites()[idx_sprite_dying].position();
            TankGame::add_sprite(
                engine,
                sid,
                RES_SM_EXPLOSION_BITMAP,
                PointF {
                    x: pos.left,
                    y: pos.top,
                },
            );
        }
        //玩家死亡, 在update回调中添加ExtraData，此次更新完成后，精灵将会从列表删除，服务器将删除的精灵信息分发给客户端
        self.extras.borrow_mut().push({
            let sprite = &engine.sprites()[idx_sprite_dying];
            PlayerData {
                id: sprite.id,
                name: sprite.name().clone(),
                score: sprite.score() as u16,
                killer_name: sprite.killer_name().clone(),
                lives: sprite.lives() as u16,
            }
        });
        //坦克死亡添加大的爆炸精灵
        if bitmap_id == RES_TANK_BITMAP {

            let sid = engine.next_sprite_id();
            let pos = *engine.sprites()[idx_sprite_dying].position();
            let idx = TankGame::add_sprite(
                engine,
                sid,
                RES_LG_EXPLOSION_BITMAP,
                PointF {
                    x: pos.left,
                    y: pos.top,
                },
            );
            //增加凶手得分
            //let dying_name = engine.sprites()[idx_sprite_dying].name().clone();
            let killer = engine.sprites()[idx_sprite_dying].killer_id();
            if let Some(killer) = engine.query_sprite(killer) {
                killer.add_score();
            }
        }
        //护士死亡
        if bitmap_id == RES_NURSE_BITMAP {
            //子弹对应的玩家增加生命值
            let killer = engine.sprites()[idx_sprite_dying].killer_id();
            if let Some(killer) = engine.query_sprite(killer) {
                let lives = killer.lives();
                if lives < 6 {
                    killer.set_lives(lives + 1);
                }
            }
        }
    }

    fn on_sprite_collision(
        &mut self,
        engine: &mut GameEngine,
        idx_sprite_hitter: usize,
        idx_sprite_hittee: usize,
    ) -> bool {
        //此处杀死的精灵, 会在下次更新时，调用上边sprite_dying函数
        //碰撞检测

        let (hitter_res, hitter_id, hitter_parent) = {
            let hitter = &engine.sprites()[idx_sprite_hitter];
            (hitter.bitmap().id(), hitter.id.clone(), hitter.parent_id)
        };
        let (hittee_res, hittee_id, hittee_parent) = {
            let hittee = &engine.sprites()[idx_sprite_hittee];
            (hittee.bitmap().id(), hittee.id.clone(), hittee.parent_id)
        };
        if hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_TANK_BITMAP{
            //子弹碰撞坦克or坦克碰撞子弹
            let left_is_missile = hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_TANK_BITMAP;
            //玩家碰到自己发射的子弹不会爆炸
            if left_is_missile && hitter_parent == hittee_id {
                false
            } else if !left_is_missile && hittee_parent == hitter_id {
                false
            } else {
                //确定发子弹的人
                let killer = if left_is_missile {
                    hitter_parent
                } else {
                    hittee_parent
                };
                //死亡的玩家index
                let dying_idx = if left_is_missile {
                    idx_sprite_hittee
                } else {
                    idx_sprite_hitter
                };
                //确定子弹
                let missile_idx = if left_is_missile {
                    idx_sprite_hitter
                } else {
                    idx_sprite_hittee
                };

                //检查中弹玩家的生命值
                let lives = engine.sprites()[dying_idx].lives();
                if lives > 1 {
                    engine.sprites()[dying_idx].set_lives(lives - 1);
                    //杀死子弹
                    engine.kill_sprite(missile_idx);
                    false
                } else {
                    //子弹对应的玩家加分
                    let killer_name = if let Some(killer) = engine.query_sprite(killer) {
                        killer.name().clone()
                    } else {
                        String::new()
                    };
                    engine.sprites()[dying_idx].set_killer(killer, killer_name);
                    //杀死相撞的子弹和坦克
                    engine.kill_sprite(idx_sprite_hittee);
                    engine.kill_sprite(idx_sprite_hitter);
                    true
                }
            }
        } else if hitter_res == RES_NURSE_BITMAP && hittee_res == RES_MISSILE_BITMAP
            || hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_NURSE_BITMAP
        {
            //子弹和护士相撞, 玩家血量+1
            engine.sprites()[idx_sprite_hitter].kill();
            engine.sprites()[idx_sprite_hittee].kill();
            engine.sprites()[idx_sprite_hitter].set_killer(
                match hitter_res {
                    RES_NURSE_BITMAP => hittee_parent,
                    _ => hitter_parent,
                },
                String::new(),
            );
            true
        } else if hitter_res == RES_MISSILE_BITMAP && hittee_res == RES_MISSILE_BITMAP {
            //检测子弹和子弹是否碰撞
            //同一个玩家的子弹不会碰撞
            if hitter_parent != hittee_parent {
                engine.kill_sprite(idx_sprite_hittee);
                engine.kill_sprite(idx_sprite_hitter);
                true
            } else {
                false
            }
        } else if hittee_res == RES_TANK_BITMAP && hitter_res == RES_TANK_BITMAP {
            //坦克之间不能互相穿过
            engine.sprites()[idx_sprite_hittee].set_velocity(0.0, 0.0);
            true
        } else {
            false
        }
    }
}

pub struct TankGame {
    pub engine: GameEngine,
    client_context: Option<Rc<Box<GameContext>>>,
    server_extras: Rc<RefCell<Vec<PlayerData>>>,
    players: HashMap<u32, Player>,
    client_player: Option<Player>,
    client_dying_delay_ms: f64, //5秒重生
    leaders: Vec<(u32, i32)>,
    dying_players: Vec<(i32, String, String)>,
    server_update_callback: Rc<RefCell<ServerUpdateCallback>>,
    client_update_callback: Rc<RefCell<ClientUpdateCallback>>,
    last_timestamp: f64, //(client)上次绘制时间
    start_time_milis: f64, //(client)游戏开始时间
    time_elpased_ms: f64, //(server/client)游戏运行时间
    last_sync_time: f64,  //(client)上次数据同步时间
    next_nurse_time: f64, //(server)上次出现护士时间
}

impl TankGame {
    fn new() -> TankGame {
        let extras = Rc::new(RefCell::new(vec![]));
        TankGame {
            engine: GameEngine::new(),
            server_extras: extras.clone(),
            players: HashMap::new(),
            client_player: None,
            client_context: None,
            client_dying_delay_ms: 0.0,
            last_timestamp: 0.0,
            leaders: vec![],
            dying_players: vec![],
            server_update_callback: Rc::new(RefCell::new(ServerUpdateCallback { extras })),
            client_update_callback: Rc::new(RefCell::new(ClientUpdateCallback {})),
            next_nurse_time: 0.0,
            time_elpased_ms: 0.0,
            last_sync_time: 0.0,
            start_time_milis: 0.0
        }
    }

    pub fn set_game_context(&mut self, context: Box<GameContext>) {
        self.client_context = Some(Rc::new(context));
    }

    pub fn client_on_connect(&mut self) {
        let context = self.client_context.as_ref().unwrap();
        context.console_log("websocket 链接成功");
        //加入游戏
        let rand_name = {
            let t = format!("{}", context.current_time_millis() as u64 / 100);
            format!("{}", t[t.len() - 4..t.len()].to_string())
        };

        let name = context.prompt("输入4个字的大名", &rand_name);
        let name = if name.len() == 0 {
            rand_name
        } else {
            name.chars().take(4).collect::<String>()
        };

        context.console_log(&format!("客户端连接成功 玩家姓名:{}", name));
        self.client_player = Some(Player {
            id: 0,
            ip: String::new(),
            name: name.clone(),
            killer_name: String::new(),
            score: 0,
        });
        if let Ok(mut encoded) = serialize(&self.client_player.as_ref().unwrap().name) {
            encoded.insert(0, MSG_START);
            context.send_binary_message(&encoded);
        }
    }

    pub fn client_on_resource_load(&self, num: i32, total: i32) {
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
            context.connect(&format!("ws://{}", CLIENT_IP));
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
            GAME.with(|game| {
                game.borrow().client_resize_window();
            });
        });

        context.set_on_connect_listener(|| {
            GAME.with(|game| {
                game.borrow_mut().client_on_connect();
            });
        });
        context.set_on_close_listener(|| {
            GAME.with(|game| {
                game.borrow()
                    .client_context
                    .as_ref()
                    .unwrap()
                    .alert("网络已断开!");
            });
        });

        //加载游戏资源
        context.set_on_resource_load_listener(|num: i32, total: i32| {
            GAME.with(|game| {
                game.borrow().client_on_resource_load(num, total);
            });
        });

        context.load_resource(format!(r#"{{"{}":"tank.png","{}":"missile.png","{}":"lg_explosion.png","{}":"sm_explosion.png","{}":"gun.png","{}":"nurse.png"}}"#,
            RES_TANK_BITMAP,
            RES_MISSILE_BITMAP,
            RES_LG_EXPLOSION_BITMAP,
            RES_SM_EXPLOSION_BITMAP,
            RES_SM_GUN_BITMAP,
            RES_NURSE_BITMAP));

        //游戏循环
        context.set_frame_callback(|timestamp: f64| {
            GAME.with(|game| {
                game.borrow_mut().client_update(timestamp);
            });
        });
    }

    pub fn client_update(&mut self, timestamp: f64) {
        let c = self.client_context.clone();
        let context = c.as_ref().unwrap();
        if self.start_time_milis == 0.0{
            self.start_time_milis = timestamp;
        }
        self.time_elpased_ms = timestamp - self.start_time_milis;
        if self.last_timestamp == 0.0 {
            self.last_timestamp = timestamp;
        }
        let elapsed_ms = timestamp - self.last_timestamp;
        //self.console_log_1("elapsed_ms=", elapsed_ms);
        //let now = context.current_time_millis();
        //处理消息
        self.client_handle_message(context.pick_binary_messages());
        //键盘事件
        let key_events = context.pick_key_events();
        for key_event in key_events {
            //self.console_log_2("send_binary_message", MSG_KEY_EVENT, key_event.0.to_i64());
            self.client_on_key_event(key_event.0.clone(), key_event.1);
            if let Ok(mut encoded) = serialize(&(
                key_event.0,
                key_event.1,
                self.client_player.as_ref().unwrap().id,
            )) {
                encoded.insert(0, MSG_KEY_EVENT);
                context.send_binary_message(&encoded);
            }
        }

        //客户端不在update_sprites处理函数中做任何操作如:精灵死亡添加爆炸、碰撞检测杀死精灵等
        //客户端仅按帧更新精灵位置，所有精灵创建、更新都由服务器下发事件中处理
        self.engine
            .update_sprites(elapsed_ms, self.client_update_callback.clone());
        context.fill_style("#2e6da3");
        context.fill_rect(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);
        context.fill_style("#3e7daf");
        context.set_canvas_font("90px 微软雅黑");
        context.fill_text(
            "坦克大战",
            CLIENT_WIDTH / 2 - 185,
            CLIENT_HEIGHT / 2 - 50,
        );
        context.set_canvas_font("32px 微软雅黑");
        context.fill_text(
            "↑ ↓ ← → ：移动  空格：开炮",
            100,
            CLIENT_HEIGHT / 2 + 30,
        );
        context.set_canvas_font("29px 微软雅黑");
        context.fill_text(
            "源码:https://github.com/planet0104/tank",
            10,
            CLIENT_HEIGHT / 2 + 70,
        );
        //context.console_log(&format!("self.engine.sprites().len()={}", self.engine.sprites().len()));
        self.engine.draw_sprites(context.clone());
        //绘制树木
        //context.draw_image_repeat(RES_GEASS1_BITMAP, 0, 0, CLIENT_WIDTH, 30);
        //context.draw_image_repeat(RES_GEASS0_BITMAP, 0, CLIENT_HEIGHT-30, CLIENT_WIDTH, 30);
        context.stroke_style("#6efdef");
        context.line_width(2);
        context.stroke_rect(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);

        // if elapsed_ms>0.0{
        //     context.fill_style("#fff");
        //     context.set_canvas_font("24px 微软雅黑");
        //     context.fill_text(&format!("FPS:{}", 1000/elapsed_ms as i32), 10, 40);
        // }
        context.fill_style("#fff");
        context.set_canvas_font("22px 微软雅黑");
        context.fill_text(
            &format!(
                "{}在线玩家:{}",
                self.client_player.as_ref().unwrap_or(&Player::empty()).name,
                self.players.len()
            ),
            10,
            40,
        );

        //前三名
        let mut lead = 1;
        for player in &self.leaders {
            context.fill_style("#fff");
            context.set_canvas_font("22px 微软雅黑");
            context.fill_text(
                &format!(
                    "第{}名:{}",
                    lead,
                    self.players.get(&player.0).unwrap().name
                ),
                CLIENT_WIDTH - 260,
                lead * 40,
            );
            context.set_canvas_font("26px 微软雅黑");
            context.fill_style("#f00");
            context.fill_text(&format!("{}分", player.1), CLIENT_WIDTH - 90, lead * 40);
            lead += 1;
        }

        //死亡的玩家信息 (delay, killer_name, name)
        context.fill_style("#ff0");
        context.set_canvas_font("20px 微软雅黑");
        let mut di = 1;
        for d in &mut self.dying_players {
            let y = 40 + di * 50;
            context.fill_text(&d.1, 20, y);
            context.fill_text(&d.2, 170, y);
            context.draw_image_at(RES_SM_GUN_BITMAP as i32, 110, y - 40);
            di += 1;
            d.0 += 1;
        }
        //清除显示150帧以后的
        self.dying_players.retain(|d| d.0 < 150);

        //死亡倒计时
        if self.client_dying_delay_ms > 0.0 {
            context.fill_style("#FFC0CB");
            context.set_canvas_font("36px 微软雅黑");
            context.fill_text(
                &format!(
                    "被[{}]炸死",
                    self.client_player.as_ref().unwrap().killer_name
                ),
                CLIENT_WIDTH / 2 - 185,
                CLIENT_HEIGHT / 2 - 50,
            );
            context.fill_text(
                &format!(
                    "{}秒之后重生",
                    (self.client_dying_delay_ms as i32 / 1000) + 1
                ),
                CLIENT_WIDTH / 2 - 185,
                CLIENT_HEIGHT / 2 - 10,
            );
            self.client_dying_delay_ms -= elapsed_ms;
            if self.client_dying_delay_ms <= 0.0 {
                //重新加入游戏
                let player = self.client_player.as_mut().unwrap();
                context.console_log(&format!(
                    "重新加入游戏 MSG_ID={} player={}",
                    MSG_START, player.name
                ));
                if let Ok(mut encoded) = serialize(&player.name) {
                    encoded.insert(0, MSG_START);
                    context.send_binary_message(&encoded);
                }
            }
        }
        self.last_timestamp = timestamp;
        context.request_animation_frame();
    }

    //玩家加入游戏
    pub fn server_join_game(&mut self, ip: String, name: String) -> u32 {
        //println!("{}加入游戏 id:{}", name.clone(), id);
        //println!("{}加入游戏", name.clone());
        //添加坦克精灵
        let sid = self.engine.next_sprite_id();
        //计算随即位置
        let x = rand_int(0, CLIENT_WIDTH) as f64;
        let y = rand_int(0, CLIENT_HEIGHT) as f64;
        let sprite_index = TankGame::add_sprite(
            &mut self.engine,
            sid,
            RES_TANK_BITMAP,
            PointF { x: x, y: y },
        );
        //添加玩家信息
        self.engine.sprites()[sprite_index].set_name(name.clone());
        self.players.insert(
            sid,
            Player {
                id: self.engine.sprites()[sprite_index].id,
                ip: ip,
                name: name.clone(),
                killer_name: String::new(),
                score: 0,
            },
        );

        sid
    }

    //离开游戏/断线
    pub fn server_leave_game(&mut self, ip: String) {
        //找到对应的用户
        let mut laeve_uid = None;
        for (uid, player) in &self.players {
            if player.ip == ip {
                laeve_uid = Some(*uid);
                break;
            }
        }
        //将其删除
        if let Some(uid) = laeve_uid {
            self.players.remove(&uid);
            if let Some(index) = self.engine.query_sprite_idx(uid) {
                self.engine.sprites()[index].kill();
            }
        }
        //println!("leave_game {} 在线人数:{}", id, self.players.len());
    }

    //创建游戏精灵
    pub fn add_sprite(engine: &mut GameEngine, id: u32, res: u8, position: PointF) -> usize {
        match res {
            RES_TANK_BITMAP => {
                //创建玩家坦克
                let mut tank_sprite = Sprite::with_bounds_action(
                    id,
                    BitmapRes::new(RES_TANK_BITMAP, TANK_BITMAP_WIDTH, TANK_BITMAP_HEIGHT * 4),
                    position,
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                    BA_WRAP,
                );
                tank_sprite.set_num_frames(4, false);
                tank_sprite.set_frame_delay(-1);
                tank_sprite.set_lives(PLAYER_LIVES);
                engine.add_sprite(tank_sprite)
            }
            RES_NURSE_BITMAP => {
                //创建护士坦克
                let mut nurse = Sprite::with_bounds_action(
                    id,
                    BitmapRes::new(RES_NURSE_BITMAP, TANK_BITMAP_WIDTH, TANK_BITMAP_HEIGHT * 4),
                    position,
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                    BA_DIE,
                );
                nurse.set_num_frames(4, false);
                nurse.set_frame_delay(-1);
                engine.add_sprite(nurse)
            }
            RES_MISSILE_BITMAP => {
                //创建一个新的子弹精灵
                let mut sprite = Sprite::with_bounds_action(
                    id,
                    BitmapRes::new(RES_MISSILE_BITMAP, 20, 80),
                    position,
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                    BA_DIE,
                );
                sprite.set_num_frames(4, false);
                sprite.set_frame_delay(-1);
                engine.add_sprite(sprite)
            }
            RES_SM_EXPLOSION_BITMAP => {
                //创建小的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    id,
                    BitmapRes::new(RES_SM_EXPLOSION_BITMAP, 17, 136),
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                );
                sprite.set_num_frames(8, true);
                sprite.set_position(position.x, position.y);
                engine.add_sprite(sprite)
            }
            RES_LG_EXPLOSION_BITMAP => {
                //创建一个大的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    id,
                    BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                    Rect::new(0.0, 0.0, CLIENT_WIDTH as f64, CLIENT_HEIGHT as f64),
                );
                sprite.set_num_frames(8, true);
                sprite.set_position(position.x, position.y);
                engine.add_sprite(sprite)
            }
            _ => 0,
        }
    }

    //差值同步精灵位置
    pub fn synchronize_sprites_velocity(context: Rc<Box<GameContext>>, sdata:&SData, sprite:&mut Sprite){
        //设置目标位置
        sprite.set_target(PointF {
            x: sdata.x as f64,
            y: sdata.y as f64,
        });
        let distance = {
            let (dx, dy) = (sdata.x as f64 - sprite.position().left, sdata.y as f64 - sprite.position().top);
            (dx * dx + dy * dy).sqrt()
        };
        sprite.set_current_frame(sdata.frame as i32);

        if distance.abs()<1.0{
            //防止抖动
            sprite.set_velocity(0.0, 0.0);
        }else if distance.abs()>100.0{
            sprite.set_velocity(0.0, 0.0);
            sprite.set_position(sdata.x as f64, sdata.y as f64);
        }
        else{
            let (dx, dy) = (sdata.x as f64 - sprite.position().left, sdata.y as f64 - sprite.position().top);
            //context.console_log(&format!("distance:{} dx={},dy={} {:?} position={:?}", distance, dx, dy, sdata, sprite.position()));
            //context.console_log(&format!("uid={}, (dx, dy)={:?} time={} sprite.position()={:?}", sprite.id, (dx, dy), time, sprite.position()));
            //if time!=0.0{
                sprite.set_velocity(dx/SERVER_SYNC_DELAY as f64, dy/SERVER_SYNC_DELAY as f64);
            //}
        }
    }

    //Client接收服务器广播列表，对精灵当前位置和服务器位置的距离计算速度(时间为：1s/5、200ms)，精灵自动移动到下一个位置。
    pub fn client_synchronize_sprites(&mut self, sync_data: SyncData) {
        let context = self.client_context.as_ref().unwrap();
        //self.console_log_1("client_synchronize_sprites", &sync_data);
        if self.last_sync_time == 0.0{
            self.last_sync_time = self.time_elpased_ms;
        }
        let mut sync_data = sync_data;
        let time = self.time_elpased_ms - self.last_sync_time;
        //context.console_log(&format!("客户端精灵数量 {}", self.engine.sprites().len()));
        //更新每个精灵
        for sdata in sync_data.data {
            let mut ext_id = None;
            let mut ext_index:usize = 0;
            for e in &sync_data.ext{
                if e.id == sdata.id{
                    ext_id = Some(ext_index);
                    break;
                }
                ext_index += 1;
            }
            //context.console_log(&format!("查询ext_id结果：{:?}", ext_id));
            let sprite_id = self.engine.query_sprite_idx(sdata.id);
            if let Some(sprite_idx) = sprite_id {
                let mut sprite = &mut self.engine.sprites()[sprite_idx];
                
                //更新精灵
                match sdata.res {
                    RES_NURSE_BITMAP => {
                        TankGame::synchronize_sprites_velocity(context.clone(), &sdata, &mut sprite);
                    }
                    RES_TANK_BITMAP => {
                        //玩家自己的坦克不更新
                        if sdata.id != self.client_player.as_ref().unwrap().id{
                            TankGame::synchronize_sprites_velocity(context.clone(), &sdata, &mut sprite);
                        }
                        //更新除玩家以外的精灵
                        //if sdata.id != self.client_player.as_ref().unwrap().id{
                            //旋转坦克角度
                            // let new_look_at = Vector2D::normalize(Vector2D::new(
                            //     sdata.x as f64 - sprite.position().left,
                            //     sdata.y as f64 - sprite.position().top,
                            // ));
                            // let degree = Vector2D::dot(&sprite.look_at(), &new_look_at);
                            // sprite.set_rotation(degree);
                            //sprite.set_look_at(new_look_at);
                            
                            //sprite.set_position(sdata.x as f64, sdata.y as f64);
                            //计算达到目标位置的速度 (时间距离上次同步的时间)
                            //let time = self.time_elpased_ms - self.last_sync_time;
                            //context.console_log(&format!("time={}ms", time));
                            //let (dx, dy) = (sdata.x as f64 - sprite.position().left, sdata.y as f64 - sprite.position().top);
                            //context.console_log(&format!("(dx, dy)={:?}", (dx, dy)));
                            //sprite.set_velocity(dx/time, dy/time);
                        //}
                        
                        //更新得分
                        if ext_id.is_some() {
                            let ext = &sync_data.ext[ext_id.unwrap()];
                            sprite.set_score(ext.score as i32);
                            sprite.set_lives(ext.lives as u32);
                            sprite.set_name(ext.name.clone());
                            //更新玩家列表中的得分
                            if let Some(player) = self.players.get_mut(&sprite.id) {
                                player.score = ext.score as i32;
                            }
                        }
                    }
                    RES_MISSILE_BITMAP => {

                    }
                    _ => {}
                }
            } else {
                //创建精灵
                let sidx = TankGame::add_sprite(
                    &mut self.engine,
                    sdata.id,
                    sdata.res,
                    PointF {
                        x: sdata.x as f64,
                        y: sdata.y as f64,
                    },
                );
                let mut sprite = &mut self.engine.sprites()[sidx];
                sprite.set_current_frame(sdata.frame as i32);
                //context.console_log(&format!("创建精灵：{:?}", sdata));

                //子弹的初始速度
                if sdata.res == RES_MISSILE_BITMAP{
                    match sprite.current_frame() {
                        0 => sprite.set_velocity(0.0, -MISSILE_VELOCITY),
                        1 => sprite.set_velocity(0.0, MISSILE_VELOCITY),
                        2 => sprite.set_velocity(-MISSILE_VELOCITY, 0.0),
                        3 => sprite.set_velocity(MISSILE_VELOCITY, 0.0),
                        _ => {}
                    }
                    //context.console_log(&format!("子弹速度{:?}", sprite.position()));
                }

                //玩家的信息
                if ext_id.is_some() && sdata.res == RES_TANK_BITMAP {
                    let ext = &sync_data.ext[ext_id.unwrap()];
                    sprite.set_name(ext.name.clone());
                    sprite.set_score(ext.score as i32);
                    sprite.set_lives(ext.lives as u32);
                    self.players.insert(
                        ext.id,
                        Player {
                            id: ext.id,
                            ip: String::new(),
                            name: ext.name.clone(),
                            score: ext.score as i32,
                            killer_name: ext.killer_name.clone(),
                        },
                    );
                }
            }
            //删除ext中已经处理完毕的精灵数据
            if ext_id.is_some() {
                let _r = sync_data.ext.remove(ext_id.unwrap());
                //context.console_log(&format!("删除ext: {:?}", r));
            }
        }
        //删除剩下的在最新精灵列表中不存在的精灵
        //context.console_log(&format!("sync_data.ext={:?}", sync_data.ext));
        for data in sync_data.ext {
            if let Some(sprite_idx) = self.engine.query_sprite_idx(data.id) {
                let mut sprite = &mut self.engine.sprites()[sprite_idx];
                sprite.kill();
                if sprite.bitmap().id() == RES_TANK_BITMAP {
                    self.players.remove(&data.id);
                    //记录并显示死亡的玩家
                    self.dying_players
                        .push((0, data.killer_name.clone(), data.name));
                }
                //let now = Instant::now();
                //context.console_log(&format!("sync_data.nowext={:?}", sync_data.ext));
                //检查玩家是否死亡
                if data.id == self.client_player.as_ref().unwrap().id {
                    context.console_log(&format!("玩家死亡!! {}", data.id));
                    self.client_player.as_mut().unwrap().killer_name = data.killer_name.clone();
                    self.client_dying_delay_ms = 5000.0;
                }
            }
        }
        //按得分排序
        let mut players_score = vec![];
        for (id, player) in &self.players {
            players_score.push((*id, player.score));
        }
        players_score.sort_by_key(|p| -p.1);
        self.leaders = players_score
            .iter()
            .take(3)
            .map(|p| (*p).clone())
            .collect::<Vec<_>>();

        self.last_sync_time = self.time_elpased_ms;
    }

    //更新游戏
    pub fn server_update(&mut self, elapsed_milis: f64) {
        self.time_elpased_ms += elapsed_milis;
        //随机出现一个护士
        if self.next_nurse_time == 0.0 {
            self.next_nurse_time = self.time_elpased_ms + (rand_int(5, 10) * 1000) as f64;
        }
        if self.time_elpased_ms >= self.next_nurse_time {
            //有玩家的时候随机产生护士
            if self.players.len() > 1 {
                let sid = self.engine.next_sprite_id();
                let sprite_index = TankGame::add_sprite(
                    &mut self.engine,
                    sid,
                    RES_NURSE_BITMAP,
                    PointF { x: 0.0, y: 0.0 },
                );
                //随机速度 velocity = 0.05~0.2
                let velocity = rand_int(5, 20) as f64 / 100.0;
                match rand_int(0, 3) {
                    1 => {
                        //向下
                        self.engine.sprites()[sprite_index].set_velocity(0.0, velocity);
                        self.engine.sprites()[sprite_index].set_current_frame(1);
                        self.engine.sprites()[sprite_index].set_position(
                            rand_int(TANK_BITMAP_WIDTH, CLIENT_WIDTH - TANK_BITMAP_WIDTH) as f64,
                            -TANK_BITMAP_HEIGHT as f64,
                        );
                    }
                    2 => {
                        //向左
                        self.engine.sprites()[sprite_index].set_velocity(-velocity, 0.0);
                        self.engine.sprites()[sprite_index].set_current_frame(2);
                        self.engine.sprites()[sprite_index].set_position(
                            CLIENT_WIDTH as f64,
                            rand_int(TANK_BITMAP_HEIGHT, CLIENT_HEIGHT - TANK_BITMAP_HEIGHT) as f64,
                        );
                    }
                    3 => {
                        //向右
                        self.engine.sprites()[sprite_index]
                            .set_velocity(velocity, -TANK_BITMAP_WIDTH as f64);
                        self.engine.sprites()[sprite_index].set_current_frame(3);
                        self.engine.sprites()[sprite_index].set_position(
                            0.0,
                            rand_int(TANK_BITMAP_HEIGHT, CLIENT_HEIGHT - TANK_BITMAP_HEIGHT) as f64,
                        );
                    }
                    _ => {
                        //向上
                        self.engine.sprites()[sprite_index].set_velocity(0.0, -velocity);
                        self.engine.sprites()[sprite_index].set_current_frame(0);
                        self.engine.sprites()[sprite_index].set_position(
                            rand_int(TANK_BITMAP_WIDTH, CLIENT_WIDTH - TANK_BITMAP_WIDTH) as f64,
                            CLIENT_HEIGHT as f64,
                        );
                    }
                }
            }
            self.next_nurse_time = self.next_nurse_time + (rand_int(8, 15) * 1000) as f64;
        }

        self.engine
            .update_sprites(elapsed_milis, self.server_update_callback.clone());
    }

    //键盘按下，坦克移动、发射子弹
    pub fn server_on_key_event(&mut self, event: KeyEvent, key: i32, sprite_id: u32) {
        if let Some(idx) = self.engine.query_sprite_idx(sprite_id) {
            match event {
                KeyEvent::KeyDown => {
                    match key {
                        VK_SPACE => {
                            let tank_position = *(self.engine.sprites()[idx].position());
                            //创建一个新的子弹精灵
                            let sid = self.engine.next_sprite_id();
                            let missile_idx = TankGame::add_sprite(
                                &mut self.engine,
                                sid,
                                RES_MISSILE_BITMAP,
                                PointF { x: 0.0, y: 0.0 },
                            );

                            //子弹的方向同玩家的方向
                            let direction = self.engine.sprites()[idx].current_frame();
                            {
                                let mut missile = &mut self.engine.sprites()[missile_idx];
                                missile.set_current_frame(direction);
                                missile.parent_id = sprite_id; //记住玩家发射的子弹
                                match direction {
                                    0 => {
                                        //上
                                        missile.set_velocity(0.0, -MISSILE_VELOCITY);
                                        missile.set_position(
                                            tank_position.left
                                                + (tank_position.right - tank_position.left) / 2.0
                                                - 10.0,
                                            tank_position.top - 15.0,
                                        );
                                    }
                                    1 => {
                                        //下
                                        missile.set_velocity(0.0, MISSILE_VELOCITY);
                                        missile.set_position(
                                            tank_position.left
                                                + (tank_position.right - tank_position.left) / 2.0
                                                - 8.0,
                                            tank_position.bottom,
                                        );
                                    }
                                    2 => {
                                        //左
                                        missile.set_velocity(-MISSILE_VELOCITY, 0.0);
                                        missile.set_position(
                                            tank_position.left - 15.0,
                                            tank_position.top
                                                - (tank_position.top - tank_position.bottom) / 2.0
                                                - 8.0,
                                        );
                                    }
                                    3 => {
                                        //右
                                        missile.set_velocity(MISSILE_VELOCITY, 0.0);
                                        missile.set_position(
                                            tank_position.right,
                                            tank_position.top
                                                - (tank_position.top - tank_position.bottom) / 2.0
                                                - 8.0,
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        VK_LEFT => {
                            self.engine.sprites()[idx].set_current_frame(2);
                            self.engine.sprites()[idx].set_velocity(-TANK_VELOCITY, 0.0);
                        }
                        VK_RIGHT => {
                            self.engine.sprites()[idx].set_current_frame(3);
                            self.engine.sprites()[idx].set_velocity(TANK_VELOCITY, 0.0);
                        }
                        VK_UP => {
                            self.engine.sprites()[idx].set_current_frame(0);
                            self.engine.sprites()[idx].set_velocity(0.0, -TANK_VELOCITY);
                        }
                        VK_DOWN => {
                            self.engine.sprites()[idx].set_current_frame(1);
                            self.engine.sprites()[idx].set_velocity(0.0, TANK_VELOCITY);
                        }
                        _other => {
                            //println!("未定义按键 {}", other);
                        }
                    }
                }

                KeyEvent::KeyUp => {
                    //键盘弹起坦克停止走动
                    match key {
                        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN => {
                            self.engine.sprites()[idx].set_velocity(0.0, 0.0);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            //println!("没有找到ID {}", sprite_id);
        }
    }

    //键盘按下，坦克移动、发射子弹
    pub fn client_on_key_event(&mut self, event: KeyEvent, key: i32) {
        if self.client_player.is_none() {
            return;
        }
        if let Some(idx) = self.engine
            .query_sprite_idx(self.client_player.as_ref().unwrap().id)
        {
            match event {
                KeyEvent::KeyDown => {
                    match key {
                        VK_LEFT => {
                            self.engine.sprites()[idx].set_current_frame(2);
                            self.engine.sprites()[idx].set_velocity(-TANK_VELOCITY, 0.0);
                        }
                        VK_RIGHT => {
                            self.engine.sprites()[idx].set_current_frame(3);
                            self.engine.sprites()[idx].set_velocity(TANK_VELOCITY, 0.0);
                        }
                        VK_UP => {
                            self.engine.sprites()[idx].set_current_frame(0);
                            self.engine.sprites()[idx].set_velocity(0.0, -TANK_VELOCITY);
                        }
                        VK_DOWN => {
                            self.engine.sprites()[idx].set_current_frame(1);
                            self.engine.sprites()[idx].set_velocity(0.0, TANK_VELOCITY);
                        }
                        _other => {
                            //println!("未定义按键 {}", other);
                        }
                    }
                }

                KeyEvent::KeyUp => {
                    //键盘弹起坦克停止走动
                    match key {
                        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN => {
                            self.engine.sprites()[idx].set_velocity(0.0, 0.0);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            //println!("没有找到ID {}", sprite_id);
        }
    }

    //服务器同步数据时, 从这里获取附加数据
    pub fn get_sync_data(&mut self) -> SyncData {
        let mut ext = vec![];
        ext.append(&mut self.server_extras.borrow_mut());

        let mut data = vec![];
        for sprite in self.engine.sprites() {
            data.push(SData {
                id: sprite.id,
                frame: sprite.current_frame() as u8,
                x: sprite.position().left as i16,
                y: sprite.position().top as i16,
                res: sprite.bitmap().id(),
                velocity_x: sprite.velocity().x as f32,
                velocity_y: sprite.velocity().y as f32,
            });
            //玩家信息
            if sprite.bitmap().id() == RES_TANK_BITMAP{
                ext.push(PlayerData {
                    id: sprite.id,
                    name: sprite.name().clone(),
                    score: sprite.score() as u16,
                    killer_name: sprite.killer_name().clone(),
                    lives: sprite.lives() as u16,
                });
            }
        }

        SyncData {
            ext: ext,
            data: data,
        }
    }

    pub fn sprites(&mut self) -> &Vec<Sprite> {
        self.engine.sprites()
    }

    //窗口大小改变时，画布适应窗口
    fn client_resize_window(&self) {
        let context = self.client_context.as_ref().unwrap();
        let (width, height) = (
            context.window_inner_width() - 5,
            context.window_inner_height() - 5,
        );
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

    fn client_handle_message(&mut self, messages: Vec<Vec<u8>>) {
        let mut messages = messages;
        let c = self.client_context.clone();
        let context = c.as_ref().unwrap();
        for message in &mut messages {
            let msg_len = message.len() as f32/1024.0;
            let msg_id = message.remove(0);
            match msg_id {
                SERVER_MSG_ERR => {
                    //context.console_log(&format!("SERVER_MSG_ERR {:0.2}K", msg_len));
                    let r: Result<String, _> = deserialize(&message[..]);
                    if let Ok(msg) = r {
                        context.console_log(&format!("服务器错误:{}", msg));
                    } else {
                        context.console_log(&format!("SERVER_MSG_ERR 消息解析失败 {:?}", r.err()));
                    }
                }
                SERVER_MSG_UID => {
                    //context.console_log(&format!("SERVER_MSG_UID {:0.2}K", msg_len));
                    let r: Result<u32, _> = deserialize(&message[..]);
                    if let Ok(uid) = r {
                        self.console_log_1("SERVER_MSG_UID", uid);
                        self.client_player.as_mut().unwrap().id = uid;
                    } else {
                        context.console_log(&format!("SERVER_MSG_UUID 消息解析失败 {:?}", r.err()));
                    }
                }
                SERVER_MSG_SYNC => {
                    //context.console_log(&format!("SERVER_MSG_SYNC {:0.2}K {}", msg_len, context.current_time_millis()));
                    let r = deserialize(&message[..]);
                    if let Ok(msg) = r {
                        self.client_synchronize_sprites(msg);
                    } else {
                        context.console_log(&format!("SERVER_MSG_SYNC 消息解析失败 {:?}", r.err()));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn players(&self) -> &HashMap<u32,Player>{
        &self.players
    }

    fn console_log_1<A: Debug, B: Debug>(&self, msg: A, obj: B) {
        let msg = format!("{:?} {:?}", msg, obj);
        self.client_context.as_ref().unwrap().console_log(&msg);
    }

    fn console_log_2<A: Display + Debug, B: Display + Debug, C: Display + Debug>(
        &self,
        msg: A,
        obj: B,
        obj2: C,
    ) {
        let msg = format!("{:?} {:?} {:?}", msg, obj, obj2);
        self.client_context.as_ref().unwrap().console_log(&msg);
    }
}
