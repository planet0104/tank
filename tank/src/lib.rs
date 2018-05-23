extern crate bincode;
pub extern crate engine;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
mod sprites;
use bincode::{deserialize, serialize};
use engine::canvas::Canvas;
use engine::sprite::{Entity, PointF, Rect, Sprite, BA_DIE, BA_WRAP};
use engine::utils::rand_int;
pub use engine::{LANDSCAPE, PORTRAIT, Bitmap, GameEngine, HtmlImage, UpdateCallback};
use sprites::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub trait Platform {
    fn set_canvas_style_margin(&self, left: i32, top: i32, right: i32, bottom: i32);
    fn set_canvas_style_width(&self, width: i32);
    fn set_canvas_style_height(&self, height: i32);
    fn set_canvas_width(&self, width: i32);
    fn set_canvas_height(&self, height: i32);
    fn alert(&self, msg: &str);
    fn load_resource(&self, json: String);
    fn window_inner_width(&self) -> i32;
    fn window_inner_height(&self) -> i32;
    fn set_on_window_resize_listener(&self, listener: fn());
    fn set_on_connect_listener(&self, listener: fn());
    fn set_on_resource_load_listener(&self, listener: fn(num: i32, total: i32));
    fn console_log(&self, msg: &str);
    fn send_message(&self, msg: &str);
    fn send_binary_message(&self, msg: &Vec<u8>);
    fn set_on_close_listener(&self, listener: fn());
    fn request_animation_frame(&self);
    fn connect(&self, url: &str);
    fn set_frame_callback(&self, callback: fn(f64));
    //fn set_on_message_listener(&self, callback: fn(&str));
    fn pick_key_events(&self) -> Vec<(KeyEvent, i32)>;
    fn pick_messages(&self) -> Vec<String>;
    fn pick_binary_messages(&self) -> Vec<Vec<u8>>;
    fn current_time_millis(&self) -> f64;
    fn set_orientation(&self, orientation:i32);
}

//socket消息
pub const MSG_DISCONNECT: u8 = 1;
pub const MSG_START: u8 = 2;
pub const MSG_KEY_EVENT: u8 = 3;
pub const MSG_SYNC_DATA: u8 = 4;
pub const MSG_CONNECT: u8 = 5;

//server发送给客户端的消息
pub const SERVER_MSG_ERR: u8 = 0; //错误
pub const SERVER_MSG_SYNC: u8 = 1; //数据同步
pub const SERVER_MSG_IP: u8 = 2; //IP地址
pub const SERVER_MSG_EVENT: u8 = 3; //事件
pub const SERVER_MSG_PLAYERS: u8 = 4; //用户上线发送玩家信息(name等), 为了节省流量，这些信息在SYNC中不发送

pub const DRIVE_THRESHOLD: i32 = 3;
//游戏宽高
pub const CLIENT_WIDTH: i32 = 1500;
pub const CLIENT_HEIGHT: i32 = 1500;

pub const VK_SPACE: i32 = 32;
pub const VK_LEFT: i32 = 37;
pub const VK_RIGHT: i32 = 39;
pub const VK_UP: i32 = 38;
pub const VK_DOWN: i32 = 40;

pub const SPRITE_UPDATE_FPS: u32 = 5;
pub const TANK_VELOCITY: f64 = 0.3;
pub const MISSILE_VELOCITY: f64 = 0.5;
pub const PLAYER_LIVES: u32 = 6; //生命值
pub const TANK_BITMAP_WIDTH: i32 = 57;
pub const TANK_BITMAP_HEIGHT: i32 = 57;
pub const SERVER_SYNC_DELAY: u64 = 100; //15帧刷新速度, 20人在线, 每次广播1K数据, 每秒广播15Kx20=300K数据,  100人1.5M/S?
pub const CLIENT_SYNC_DELAY: u64 = 100;

// pub const SERVER_IP: &str = "127.0.0.1:8080";
// pub const CLIENT_IP: &str = "127.0.0.1:8080";

// pub const SERVER_IP: &str = "192.168.192.122:8080";
// pub const CLIENT_IP: &str = "192.168.192.122:8080";

// pub const SERVER_IP:&str = "192.168.1.108:8080";
// pub const CLIENT_IP:&str = "192.168.1.108:8080";

pub const SERVER_IP: &str = "172.31.33.204:8414";
pub const CLIENT_IP: &str = "54.249.68.59:8414";

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

//服务器事件
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerEvent {
    //玩家死亡(dying_uid, dying_name, killer_name)
    PlayerDying(u32, String, String),
    //玩家加入(ip, uid, name)
    PlayerJoin(String, u32, String),
}

//精灵信息
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncData {
    pub id: u32,
    pub x: i16,
    pub y: i16,
    pub res: u8,
    pub frame: u8,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub extra: Option<ExtraData>,
}

//精灵附加信息
#[derive(Serialize, Deserialize, Debug)]
pub struct ExtraData {
    pub score: u16,
    pub lives: u16,
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
    pub static KEY_MAP: HashMap<String, i32> = {
            [
                ("Left".to_string(), VK_LEFT),
                ("a".to_string(),VK_LEFT),
                ("ArrowLeft".to_string(), VK_LEFT),
                ("Right".to_string(), VK_RIGHT),
                ("d".to_string(), VK_RIGHT),
                ("ArrowRight".to_string(), VK_RIGHT),
                ("Up".to_string(), VK_UP),
                ("w".to_string(), VK_UP),
                ("ArrowUp".to_string(), VK_UP),
                ("Down".to_string(), VK_DOWN),
                ("s".to_string(), VK_DOWN),
                ("ArrowDown".to_string(), VK_DOWN),
                (" ".to_string(), VK_SPACE),
                ("j".to_string(), VK_SPACE),
                ("k".to_string(), VK_SPACE),
                ("l".to_string(), VK_SPACE)]
            .iter().cloned().collect()
    };
}
//客户端游戏更新(不做任何处理)
pub struct ClientUpdateCallback {
    player_id: u32,
}
impl UpdateCallback for ClientUpdateCallback {
    fn on_sprite_dying(&mut self, _engine: &mut GameEngine, _idx_sprite_dying: usize) {}
    fn on_sprite_collision(
        &mut self,
        engine: &mut GameEngine,
        idx_sprite_hitter: usize,
        idx_sprite_hittee: usize,
    ) -> bool {
        let (hitter_class, _hitter_id, _hitter_parent) = {
            let hitter = &engine.sprites()[idx_sprite_hitter].borrow();
            (hitter.class(), hitter.id(), hitter.parent())
        };
        let (hittee_class, hittee_id, _hittee_parent) = {
            let hittee = &engine.sprites()[idx_sprite_hittee].borrow();
            (hittee.class(), hittee.id(), hittee.parent())
        };
        if hittee_class == sprites::SPRITE_TANK && hitter_class == sprites::SPRITE_TANK {
            //坦克之间不能互相穿过
            if hittee_id == self.player_id {
                engine.sprites()[idx_sprite_hittee]
                    .borrow_mut()
                    .set_velocity(0.0, 0.0);
            }
            true
        } else {
            false
        }
    }
}

//服务器端游戏更新
pub struct ServerUpdateCallback {
    events: Rc<RefCell<Vec<ServerEvent>>>,
}
impl UpdateCallback for ServerUpdateCallback {
    fn on_sprite_dying(&mut self, engine: &mut GameEngine, idx_sprite_dying: usize) {
        let (class, dying_position, killer) = {
            let sprite_dying = engine.sprites()[idx_sprite_dying].borrow();
            (
                sprite_dying.class(),
                *sprite_dying.position(),
                sprite_dying.killer(),
            )
        };
        //子弹精灵死亡添加小的爆炸精灵
        if class == sprites::SPRITE_MISSILE {
            let sid = engine.next_sprite_id();
            engine.add_sprite(Rc::new(RefCell::new(SMExplosionSprite::new(
                sid,
                PointF {
                    x: dying_position.left,
                    y: dying_position.top,
                },
            ))));
        }
        //坦克死亡添加大的爆炸精灵
        if class == sprites::SPRITE_TANK {
            let sid = engine.next_sprite_id();
            engine.add_sprite(Rc::new(RefCell::new(LGExplosionSprite::new(
                sid,
                PointF {
                    x: dying_position.left,
                    y: dying_position.top,
                },
            ))));
            //增加凶手得分
            //let dying_name = engine.sprites()[idx_sprite_dying].name().clone();
            if let Some(killer) = engine.query_sprite(killer) {
                killer.borrow_mut().add_score();
            }

            //玩家死亡事件
            let sprite_dying = engine.sprites()[idx_sprite_dying].borrow();
            self.events.borrow_mut().push({
                ServerEvent::PlayerDying(
                    sprite_dying.id(),
                    sprite_dying.name().clone(),
                    sprite_dying.killer_name().clone(),
                )
            });
        }
        //护士死亡
        if class == sprites::SPRITE_NURSE {
            //子弹对应的玩家增加生命值
            if let Some(killer) = engine.query_sprite(killer) {
                let lives = killer.borrow().lives();
                if lives < 6 {
                    killer.borrow_mut().set_lives(lives + 1);
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
        //碰撞检测
        //此处杀死的精灵, 会在下次更新时，调用上边sprite_dying函数

        let (_hitter_id, hitter_parent, hitter_class) = {
            let hitter = engine.sprites()[idx_sprite_hitter].borrow();
            (hitter.id(), hitter.parent(), hitter.class())
        };
        let (hittee_id, hittee_parent, hittee_class) = {
            let hittee = engine.sprites()[idx_sprite_hittee].borrow();
            (hittee.id(), hittee.parent(), hittee.class())
        };

        if hitter_class == sprites::SPRITE_MISSILE && hittee_class == sprites::SPRITE_TANK {
            let killer_name = if let Some(killer) = engine.query_sprite(hitter_parent) {
                killer.borrow().name().clone()
            } else {
                String::new()
            };
            //玩家碰到自己发射的子弹不会爆炸
            if hitter_parent == hittee_id {
                false
            } else {
                //检查中弹玩家的生命值
                let lives = engine.sprites()[idx_sprite_hittee].borrow().lives();
                if lives > 1 {
                    engine.sprites()[idx_sprite_hittee]
                        .borrow_mut()
                        .set_lives(lives - 1);
                    //杀死子弹
                    engine.sprites()[idx_sprite_hitter].borrow_mut().kill();
                    false
                } else {
                    //子弹对应的玩家加分
                    engine.sprites()[idx_sprite_hittee]
                        .borrow_mut()
                        .set_killer(hitter_parent, killer_name);
                    //杀死相撞的子弹和坦克
                    engine.sprites()[idx_sprite_hittee].borrow_mut().kill();
                    engine.sprites()[idx_sprite_hitter].borrow_mut().kill();
                    true
                }
            }
        } else if hitter_class == sprites::SPRITE_MISSILE && hittee_class == sprites::SPRITE_NURSE {
            //子弹和护士相撞, 玩家血量+1
            engine.sprites()[idx_sprite_hittee].borrow_mut().kill();
            engine.sprites()[idx_sprite_hitter].borrow_mut().kill();
            engine.sprites()[idx_sprite_hittee]
                .borrow_mut()
                .set_killer(hitter_parent, String::new());
            true
        } else if hitter_class == sprites::SPRITE_MISSILE && hittee_class == sprites::SPRITE_MISSILE
        {
            //检测子弹和子弹是否碰撞
            //同一个玩家的子弹不会碰撞
            if hitter_parent != hittee_parent {
                engine.sprites()[idx_sprite_hittee].borrow_mut().kill();
                engine.sprites()[idx_sprite_hitter].borrow_mut().kill();
                true
            } else {
                false
            }
        } else if hitter_class == sprites::SPRITE_TANK && hittee_class == sprites::SPRITE_TANK {
            //坦克之间不能互相穿过
            engine.sprites()[idx_sprite_hittee]
                .borrow_mut()
                .set_velocity(0.0, 0.0);
            true
        } else {
            false
        }
    }
}

pub struct Context {
    canvas: Rc<Box<Canvas>>,
    platform: Rc<Box<Platform>>,
}

impl Context {
    pub fn new(canvas: Box<Canvas>, platform: Box<Platform>) -> Context {
        Context {
            canvas: Rc::new(canvas),
            platform: Rc::new(platform),
        }
    }
}

pub struct TankGame {
    pub engine: GameEngine,
    server_events: Rc<RefCell<Vec<ServerEvent>>>,
    client_context: Option<Context>,
    players: HashMap<u32, Rc<RefCell<TankSprite>>>,
    clinet_player_names: HashMap<u32, (String, String)>,
    current_player: Option<Rc<RefCell<TankSprite>>>,
    current_player_ip: String,
    current_player_id: u32,
    current_player_name: String,
    client_dying_delay_ms: f64, //5秒重生
    leaders: Vec<(u32, i32)>,
    dying_players: Vec<(i32, String, String)>,
    server_update_callback: Rc<RefCell<ServerUpdateCallback>>,
    client_update_callback: Rc<RefCell<ClientUpdateCallback>>,
    last_timestamp: f64,        //(client)上次绘制时间
    start_time_milis: f64,      //(client)游戏开始时间
    time_elpased_ms: f64,       //(server/client)游戏运行时间
    last_sync_time: f64,        //(client)上次数据同步时间
    next_nurse_time: f64,       //(server)上次出现护士时间
    client_last_sync_time: f64, //(client)上次数据同步时间
    client_binary_messages: Vec<Vec<u8>>,
    orientation: i32, //0 竖屏 1横屏
}

impl TankGame {
    fn new() -> TankGame {
        let server_events = Rc::new(RefCell::new(vec![]));
        TankGame {
            engine: GameEngine::new(),
            server_events: server_events.clone(),
            players: HashMap::new(),
            clinet_player_names: HashMap::new(),
            current_player: None,
            current_player_id: 0,
            current_player_ip: String::new(),
            current_player_name: String::new(),
            client_context: None,
            client_dying_delay_ms: 0.0,
            last_timestamp: 0.0,
            leaders: vec![],
            dying_players: vec![],
            server_update_callback: Rc::new(RefCell::new(ServerUpdateCallback {
                events: server_events,
            })),
            client_update_callback: Rc::new(RefCell::new(ClientUpdateCallback { player_id: 0 })),
            next_nurse_time: 0.0,
            time_elpased_ms: 0.0,
            last_sync_time: 0.0,
            start_time_milis: 0.0,
            client_last_sync_time: 0.0,
            client_binary_messages: vec![],
            orientation: PORTRAIT
        }
    }

    pub fn set_game_context(&mut self, context: Context) {
        self.client_context = Some(context);
    }

    pub fn clone_context(&self) -> (Rc<Box<Canvas>>, Rc<Box<Platform>>) {
        (
            self.client_context.as_ref().unwrap().canvas.clone(),
            self.client_context.as_ref().unwrap().platform.clone(),
        )
    }

    pub fn player_join_game(&mut self, name: &str) {
        let platform = self.client_context.as_ref().unwrap().platform.clone();
        platform.console_log(&format!("{}你好， 正在加入...", name));
        //加入游戏
        let rand_name = {
            let t = format!("{}", platform.current_time_millis() as u64 / 100);
            format!("{}", t[t.len() - 4..t.len()].to_string())
        };

        let name = if name.len() == 0 {
            rand_name
        } else {
            name.chars().take(4).collect::<String>()
        };
        //保存用户姓名, 上线后设置
        self.current_player_name = name;
        self.client_binary_messages.push(vec![MSG_CONNECT]);
    }

    pub fn client_on_resource_load(&self, num: i32, total: i32) {
        let (canvas, platform) = self.clone_context();
        let percent = num as f32 / total as f32;
        let bar_width = (platform.window_inner_width() as f32 / 1.5) as i32;
        let bar_height = bar_width / 10;
        let bar_left = platform.window_inner_width() / 2 - bar_width / 2;
        let bar_top = platform.window_inner_height() / 2 - bar_height / 2;
        canvas.fill_style("rgb(200, 200, 200)");
        canvas.fill_rect(bar_left, bar_top, bar_width, bar_height);
        canvas.fill_style("rgb(120, 120, 255)");
        canvas.fill_rect(
            bar_left,
            bar_top,
            (bar_width as f32 * percent) as i32,
            bar_height,
        );
        canvas.fill_style("#ff0");
        canvas.fill_text(
            &format!("资源加载中({}/{})", num, total),
            bar_left + bar_width / 3,
            bar_top + bar_height / 2 + 10,
        );
        if num == total {
            //资源加载完成, 启动游戏循环
            platform.request_animation_frame();
            platform.connect(&format!("ws://{}", CLIENT_IP));
        }
    }

    pub fn client_start(&mut self) {
        let (canvas, platform) = self.clone_context();
        platform.console_log("游戏启动!!!");
        platform.set_canvas_width(platform.window_inner_width());
        platform.set_canvas_height(platform.window_inner_height());
        self.client_resize_window();
        canvas.set_font("24px 微软雅黑");

        platform.set_on_window_resize_listener(|| {
            GAME.with(|game| {
                game.borrow_mut().client_resize_window();
            });
        });

        // context.set_on_connect_listener(|| {
        //     GAME.with(|game| {
        //         game.borrow_mut().client_on_connect();
        //     });
        // });
        platform.set_on_close_listener(|| {
            GAME.with(|game| {
                game.borrow()
                    .client_context
                    .as_ref()
                    .unwrap()
                    .platform
                    .alert("网络已断开!");
            });
        });

        //加载游戏资源
        platform.set_on_resource_load_listener(|num: i32, total: i32| {
            GAME.with(|game| {
                game.borrow().client_on_resource_load(num, total);
            });
        });

        platform.load_resource(format!(r#"{{"{}":"tank.png","{}":"missile.png","{}":"lg_explosion.png","{}":"sm_explosion.png","{}":"gun.png","{}":"nurse.png","{}":"bg.jpg"}}"#,
            RES_TANK_BITMAP,
            RES_MISSILE_BITMAP,
            RES_LG_EXPLOSION_BITMAP,
            RES_SM_EXPLOSION_BITMAP,
            RES_SM_GUN_BITMAP,
            RES_NURSE_BITMAP,
            RES_BG_BITMAP));

        //游戏循环
        platform.set_frame_callback(|timestamp: f64| {
            GAME.with(|game| {
                game.borrow_mut().client_update(timestamp);
            });
        });
    }

    pub fn client_update(&mut self, timestamp: f64) {
        if self.start_time_milis == 0.0 {
            self.start_time_milis = timestamp;
        }
        self.time_elpased_ms = timestamp - self.start_time_milis;
        if self.last_timestamp == 0.0 {
            self.last_timestamp = timestamp;
        }
        let elapsed_ms = timestamp - self.last_timestamp;

        let (canvas, platform) = self.clone_context();

        //5帧的速度广播
        if timestamp >= self.client_last_sync_time {
            if let Some(current_player) = &self.current_player {
                let current_player = current_player.borrow();
                //上传玩家数据
                let data = SyncData {
                    id: current_player.id(),
                    frame: current_player.cur_frame() as u8,
                    x: current_player.position().left as i16,
                    y: current_player.position().top as i16,
                    res: current_player.bitmap().id(),
                    velocity_x: current_player.velocity().x as f32,
                    velocity_y: current_player.velocity().y as f32,
                    extra: None,
                };

                if let Ok(mut encoded) = serialize(&data) {
                    encoded.insert(0, MSG_SYNC_DATA);
                    self.client_binary_messages.push(encoded);
                }
            }

            //统一发送所有消息
            let mut messages = vec![];
            messages.append(&mut self.client_binary_messages);
            if messages.len() > 0 {
                if let Ok(encoded) = serialize(&messages) {
                    platform.send_binary_message(&encoded);
                }
            }
            self.client_last_sync_time = timestamp + CLIENT_SYNC_DELAY as f64;
        }

        //self.console_log_1("elapsed_ms=", elapsed_ms);
        //let now = context.current_time_millis();
        //处理消息
        let messages = platform.pick_binary_messages();
        for msgs in messages {
            //每一条消息都是一个消息集合
            let r: Result<Vec<Vec<u8>>, _> = deserialize(&msgs[..]);
            if let Ok(m) = r {
                self.client_handle_message(m);
            } else {
                platform.console_log(&format!(
                    "client_handle_message 消息解析失败 {:?}",
                    r.err()
                ));
            }
        }

        //键盘事件
        let mut key_events = platform.pick_key_events();
        //如果是竖屏,对事件修改: 上=>左; 下=>右;  左=>下; 右=>上;
        if self.orientation == PORTRAIT{
            for event in &mut key_events{
                let etype = event.0.clone();
                match event.1{
                    VK_UP => *event = (etype, VK_LEFT),
                    VK_DOWN => *event = (etype, VK_RIGHT),
                    VK_LEFT => *event = (etype, VK_DOWN),
                    VK_RIGHT => *event = (etype, VK_UP),
                    _ => {}
                }
            }
        }
        if self.current_player_id != 0 {
            for key_event in key_events {
                self.client_on_key_event(key_event.0.clone(), key_event.1);
                if let Ok(mut encoded) =
                    serialize(&(key_event.0, key_event.1, self.current_player_id))
                {
                    encoded.insert(0, MSG_KEY_EVENT);
                    self.client_binary_messages.push(encoded);
                }
            }
        }

        //客户端不在update_sprites处理函数中做任何操作如:精灵死亡添加爆炸、碰撞检测杀死精灵等
        //客户端仅按帧更新精灵位置，所有精灵创建、更新都由服务器下发事件中处理
        self.engine
            .update_sprites(elapsed_ms, self.client_update_callback.clone());

        //清空屏幕
        canvas.fill_style("#666666");
        canvas.fill_rect(0, 0, platform.window_inner_width(), platform.window_inner_height());

        //---------------------------------------------------------------------
        //-------------- 平移，使当前玩家处于屏幕中央 ---------------------------
        //---------------------------------------------------------------------
        canvas.save();
        let mut window_width = platform.window_inner_width();
        let mut window_height = platform.window_inner_height();
        //检查是否竖屏
        if self.orientation == PORTRAIT{
            let w = window_width;
            window_width = window_height;
            window_height = w;

            canvas.translate(platform.window_inner_width(), 0);
            canvas.rotate(std::f64::consts::PI/2.0);
        }

        canvas.save();

        if let Some(current_player) = &self.current_player {
            let current_player = current_player.borrow();
            let (cw, ch) = (window_width, window_height);
            canvas.translate(-current_player.position().left as i32 + cw/2 - TANK_BITMAP_WIDTH/2, -current_player.position().top as i32 + ch/2 - TANK_BITMAP_HEIGHT/2);
        }

        //背景边框和颜色
        canvas.draw_image_repeat(&HtmlImage {
            id: RES_BG_BITMAP,
            width: 64,
            height: 64,
        }, 0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);

        //背景文字
        canvas.fill_style("#3e7daf");
        canvas.set_font("90px 微软雅黑");
        canvas.fill_text(
            "坦克大战",
            CLIENT_WIDTH / 2 - 185,
            CLIENT_HEIGHT / 2 - 50,
        );

        self.engine.draw_sprites((*canvas).as_ref());

        canvas.restore();
        //-----------------------------------------------------------------------------
        //-----------------------------------------------------------------------------

        //前三名
        let mut lead = 1;
        for player in &self.leaders {
            canvas.fill_style("#fff");
            canvas.set_font("18px 微软雅黑");
            canvas.fill_text(
                &format!(
                    "第{}名:{}",
                    lead,
                    self.players
                        .get(&player.0)
                        .and_then(|player| Some(player.borrow().name().clone()))
                        .or_else(|| Some(String::new()))
                        .unwrap()
                ),
                window_width - 260,
                lead * 40,
            );
            canvas.set_font("18px 微软雅黑");
            canvas.fill_style("#f00");
            canvas.fill_text(&format!("{}分", player.1), window_width - 90, lead * 40);
            lead += 1;
        }

        //死亡的玩家信息 (delay, killer_name, name)
        canvas.fill_style("#ff0");
        canvas.set_font("18px 微软雅黑");
        let mut di = 1;
        for d in &mut self.dying_players {
            let y = 40 + di * 50;
            canvas.fill_text(&d.1, 20, y);
            canvas.fill_text(&d.2, 170, y);
            canvas.draw_image_at(&SM_GUN_BITMAP, 110, y - 40);
            di += 1;
            d.0 += 1;
        }
        //清除显示150帧以后的
        self.dying_players.retain(|d| d.0 < 150);

        //死亡倒计时
        if self.client_dying_delay_ms > 0.0 {
            let current_player = self.current_player.as_ref().unwrap();
            let current_player = current_player.borrow();
            canvas.fill_style("#FFC0CB");
            canvas.set_font("36px 微软雅黑");
            canvas.fill_text(
                &format!("被[{}]炸死", current_player.killer_name()),
                window_width / 2 - 185,
                window_height / 2 - 50,
            );
            canvas.fill_text(
                &format!(
                    "{}秒之后重生",
                    (self.client_dying_delay_ms as i32 / 1000) + 1
                ),
                window_width / 2 - 185,
                window_height / 2 - 10,
            );
            self.client_dying_delay_ms -= elapsed_ms;
            if self.client_dying_delay_ms <= 0.0 {
                //重新加入游戏
                platform.console_log(&format!(
                    "重新加入游戏 MSG_ID={} player={}",
                    MSG_START,
                    current_player.name()
                ));
                if let Ok(mut encoded) = serialize(current_player.name()) {
                    encoded.insert(0, MSG_START);
                    self.client_binary_messages.push(encoded);
                }
            }
        }

        //显示玩家名字和在线玩家数量
        canvas.fill_style("#fff");
        canvas.set_font("18px 微软雅黑");
        canvas.fill_text(
            &format!(
                "{}在线玩家:{}",
                self.current_player_name,
                self.players.len()
            ),
            10,
            40,
        );

        //使用说明
        // canvas.set_font("18px 微软雅黑");
        // canvas.fill_text(
        //     "↑ ↓ ← → ：移动  空格：开炮",
        //     150,
        //     platform.window_inner_height() - 60,
        // );
        canvas.set_font("15px 微软雅黑");
        canvas.fill_style("rgba(0, 0, 0, 0.35)");
        canvas.fill_text(
            "源码:https://github.com/planet0104/tank",
            10,
            window_height - 20,
        );
        canvas.restore();
        self.last_timestamp = timestamp;
        platform.request_animation_frame();
    }

    pub fn server_update_player(&mut self, _ip: String, data: SyncData) {
        if let Some(player_sprite) = self.engine.query_sprite(data.id) {
            let mut player_sprite = player_sprite.borrow_mut();
            player_sprite.set_position_point(data.x as f64, data.y as f64);
            player_sprite.set_velocity(data.velocity_x as f64, data.velocity_y as f64);
            player_sprite.set_cur_frame(data.frame as i32);
        }
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
        let tank_sprite = Rc::new(RefCell::new(TankSprite::new(
            ip.clone(),
            sid,
            PointF::new(x, y),
        )));
        self.engine.add_sprite(tank_sprite.clone());
        //检查新生玩家和其他玩家的距离，距离太近重新生成位置
        let mut check_count = 0;
        let mut born_position = PointF::new(
            tank_sprite.borrow().position().left,
            tank_sprite.borrow().position().top,
        );
        loop {
            //最多检查5次重叠位置
            if check_count >= 5 {
                break;
            }
            let mut overlap = false;
            for sprite in self.engine.sprites() {
                let sprite = sprite.borrow();
                if sprite.class() == sprites::SPRITE_TANK {
                    let (dx, dy) = (
                        born_position.x - sprite.position().left,
                        born_position.y - sprite.position().top,
                    );
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < sprite.bitmap().width() as f64 {
                        overlap = true;
                        break;
                    }
                }
            }
            if !overlap {
                break;
            }
            born_position.x = rand_int(0, CLIENT_WIDTH) as f64;
            born_position.y = rand_int(0, CLIENT_HEIGHT) as f64;
            check_count += 1;
        }
        tank_sprite
            .borrow_mut()
            .set_position_point(born_position.x, born_position.y);

        //添加玩家信息
        tank_sprite.borrow_mut().set_name(name.clone());

        self.server_events.borrow_mut().push({
            ServerEvent::PlayerJoin(
                ip,
                tank_sprite.borrow().id(),
                tank_sprite.borrow().name().clone(),
            )
        });

        self.players.insert(sid, tank_sprite);

        sid
    }

    //离开游戏/断线
    pub fn server_leave_game(&mut self, ip: String) {
        //找到对应的用户
        let mut laeve_uid = None;
        for (uid, player) in &self.players {
            let player = player.borrow();
            if player.ip == ip {
                laeve_uid = Some(*uid);
                break;
            }
        }
        //将其删除
        if let Some(uid) = laeve_uid {
            if let Some(player) = self.players.get(&uid) {
                player.borrow_mut().kill();
            }
            self.players.remove(&uid);
        }
        //println!("leave_game {} 在线人数:{}", id, self.players.len());
    }

    //Client接收服务器广播列表，对精灵当前位置和服务器位置的距离计算速度(时间为：1s/5、200ms)，精灵自动移动到下一个位置。
    pub fn client_synchronize_sprites(&mut self, sync_data: Vec<SyncData>) {
        let platform = self.client_context.as_ref().unwrap().platform.clone();
        //let context = self.client_context.as_ref().unwrap();
        //self.console_log_1("client_synchronize_sprites", &sync_data);
        if self.last_sync_time == 0.0 {
            self.last_sync_time = self.time_elpased_ms;
        }
        //let time = self.time_elpased_ms - self.last_sync_time;

        //删掉列表中不存在的精灵
        let server_ids = sync_data.iter().map(|d| d.id).collect::<Vec<u32>>();
        self.engine
            .sprites()
            .retain(|sprite| server_ids.contains(&sprite.borrow().id()));
        self.players()
            .retain(|id, _player| server_ids.contains(&id));

        //更新每个精灵
        for sdata in sync_data {
            let platform = self.client_context.as_ref().unwrap().platform.clone();
            let sprite_id = self.engine.query_sprite_idx(sdata.id);
            if let Some(sprite_idx) = sprite_id {
                //platform.console_log("更新精灵");
                let mut sprite = self.engine.sprites()[sprite_idx].borrow_mut();
                if sdata.id != self.current_player_id {
                    //sprite.set_position(sdata.x as f64, sdata.y as f64);
                    sprite.set_target_position(PointF::new(sdata.x as f64, sdata.y as f64));
                    sprite.set_velocity(sdata.velocity_x as f64, sdata.velocity_y as f64);
                    sprite.set_cur_frame(sdata.frame as i32);
                }
                //更新精灵
                if sdata.res == RES_NURSE_BITMAP {

                } else if sdata.res == RES_TANK_BITMAP {
                    //更新得分
                    if let Some(extra) = sdata.extra {
                        sprite.set_score(extra.score as i32);
                        sprite.set_lives(extra.lives as u32);
                    }
                } else if sdata.res == RES_MISSILE_BITMAP {

                }
            } else {
                //platform.console_log("创建精灵");
                //创建精灵
                let mut tank = None;
                let sidx = self.engine.add_sprite({
                    let pos = PointF {
                        x: sdata.x as f64,
                        y: sdata.y as f64,
                    };
                    match sdata.res {
                        RES_LG_EXPLOSION_BITMAP => {
                            Rc::new(RefCell::new(LGExplosionSprite::new(sdata.id, pos)))
                        }
                        RES_TANK_BITMAP => {
                            let tank_sprite = Rc::new(RefCell::new(TankSprite::new(
                                String::new(),
                                sdata.id,
                                pos,
                            )));
                            tank = Some(tank_sprite.clone());
                            tank_sprite
                        }
                        RES_MISSILE_BITMAP => {
                            Rc::new(RefCell::new(MissileSprite::new(sdata.id, pos)))
                        }
                        RES_NURSE_BITMAP => {
                            Rc::new(RefCell::new(NruseSprite::new(sdata.id, Some(pos))))
                        }
                        _ => Rc::new(RefCell::new(SMExplosionSprite::new(sdata.id, pos))),
                    }
                });
                
                //本地创建的坦克玩家
                if let Some(tank) = tank {
                    //判断并保存当前玩家
                    if sdata.id == self.current_player_id {
                        self.current_player = Some(tank.clone());
                        platform.console_log(&format!("玩家精灵创建 id:{}", sdata.id));
                    }
                    //添加到本地players
                    if !self.players.contains_key(&sdata.id) {
                        platform.console_log(&format!(
                            "添加了本地的玩家: {}",
                            sdata.id
                        ));
                        self.players.insert(sdata.id, tank);
                    };
                }

                let mut sprite = &mut self.engine.sprites()[sidx].borrow_mut();
                sprite.set_cur_frame(sdata.frame as i32);
                sprite.set_velocity(sdata.velocity_x as f64, sdata.velocity_y as f64);
                //platform.console_log(&format!("创建精灵：{:?}", sdata));

                if let Some((_ip, name)) = self.clinet_player_names.get(&sdata.id) {
                    sprite.set_name(name.clone());
                }

                //玩家的信息
                if let Some(extra) = sdata.extra {
                    sprite.set_score(extra.score as i32);
                    sprite.set_lives(extra.lives as u32);
                }
            }
        }

        //按得分排序
        let mut players_score = vec![];
        for (id, player) in &self.players {
            players_score.push((*id, player.borrow().score()));
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
                let nurse_sprite = NruseSprite::new(self.engine.next_sprite_id(), None);
                self.engine.add_sprite(Rc::new(RefCell::new(nurse_sprite)));
            }
            self.next_nurse_time = self.next_nurse_time + (rand_int(8, 15) * 1000) as f64;
        }

        self.engine
            .update_sprites(elapsed_milis, self.server_update_callback.clone());
        //检查是否有玩家死亡
        let events: &Vec<ServerEvent> = &self.server_events.borrow_mut();
        for event in events {
            match event {
                &ServerEvent::PlayerDying(ref uid, ref _name, ref _killer_name) => {
                    //删除服务器端死去的玩家
                    self.players.remove(uid);
                }
                _ => {}
            }
        }
    }

    //键盘按下，坦克移动、发射子弹
    pub fn server_on_key_event(&mut self, event: KeyEvent, key: i32, sprite_id: u32) {
        if let Some(idx) = self.engine.query_sprite_idx(sprite_id) {
            match event {
                KeyEvent::KeyDown => {
                    match key {
                        VK_SPACE => {
                            let (tank_position, direction) = {
                                let mut tank_sprite = self.engine.sprites()[idx].borrow_mut();
                                (*tank_sprite.position(), tank_sprite.cur_frame())
                            };
                            //创建一个新的子弹精灵
                            let sid = self.engine.next_sprite_id();
                            let missile_sprite =
                                Rc::new(RefCell::new(MissileSprite::new(sid, PointF::zero())));
                            self.engine.add_sprite(missile_sprite.clone());

                            //子弹的方向同玩家的方向
                            let mut missile = missile_sprite.borrow_mut();
                            missile.set_cur_frame(direction);
                            missile.set_parent(sprite_id); //记住玩家发射的子弹
                            match direction {
                                0 => {
                                    //上
                                    missile.set_velocity(0.0, -MISSILE_VELOCITY);
                                    missile.set_position_point(
                                        tank_position.left
                                            + (tank_position.right - tank_position.left) / 2.0
                                            - 10.0,
                                        tank_position.top - 15.0,
                                    );
                                }
                                1 => {
                                    //下
                                    missile.set_velocity(0.0, MISSILE_VELOCITY);
                                    missile.set_position_point(
                                        tank_position.left
                                            + (tank_position.right - tank_position.left) / 2.0
                                            - 8.0,
                                        tank_position.bottom,
                                    );
                                }
                                2 => {
                                    //左
                                    missile.set_velocity(-MISSILE_VELOCITY, 0.0);
                                    missile.set_position_point(
                                        tank_position.left - 15.0,
                                        tank_position.top
                                            - (tank_position.top - tank_position.bottom) / 2.0
                                            - 8.0,
                                    );
                                }
                                3 => {
                                    //右
                                    missile.set_velocity(MISSILE_VELOCITY, 0.0);
                                    missile.set_position_point(
                                        tank_position.right,
                                        tank_position.top
                                            - (tank_position.top - tank_position.bottom) / 2.0
                                            - 8.0,
                                    );
                                }
                                _ => {}
                            }
                        }
                        VK_LEFT => {
                            let mut tank_sprite = self.engine.sprites()[idx].borrow_mut();
                            tank_sprite.set_cur_frame(2);
                            tank_sprite.set_velocity(-TANK_VELOCITY, 0.0);
                        }
                        VK_RIGHT => {
                            let mut tank_sprite = self.engine.sprites()[idx].borrow_mut();
                            tank_sprite.set_cur_frame(3);
                            tank_sprite.set_velocity(TANK_VELOCITY, 0.0);
                        }
                        VK_UP => {
                            let mut tank_sprite = self.engine.sprites()[idx].borrow_mut();
                            tank_sprite.set_cur_frame(0);
                            tank_sprite.set_velocity(0.0, -TANK_VELOCITY);
                        }
                        VK_DOWN => {
                            let mut tank_sprite = self.engine.sprites()[idx].borrow_mut();
                            tank_sprite.set_cur_frame(1);
                            tank_sprite.set_velocity(0.0, TANK_VELOCITY);
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
                            self.engine.sprites()[idx]
                                .borrow_mut()
                                .set_velocity(0.0, 0.0);
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
        if let Some(current_player) = self.current_player.as_ref() {
            let mut player = current_player.borrow_mut();
            match event {
                KeyEvent::KeyDown => {
                    if let Some((velocity, frame)) = {
                        match key {
                            VK_LEFT => Some((PointF::new(-TANK_VELOCITY, 0.0), 2)),
                            VK_RIGHT => Some((PointF::new(TANK_VELOCITY, 0.0), 3)),
                            VK_UP => Some((PointF::new(0.0, -TANK_VELOCITY), 0)),
                            VK_DOWN => Some((PointF::new(0.0, TANK_VELOCITY), 1)),
                            _other => None,
                        }
                    } {
                        player.set_cur_frame(frame);
                        player.set_velocity(velocity.x, velocity.y);
                    };
                }

                KeyEvent::KeyUp => {
                    //键盘弹起坦克停止走动
                    match key {
                        VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN => {
                            player.set_velocity(0.0, 0.0);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            let platform = self.client_context.as_ref().unwrap().platform.clone();
            platform.console_log("用户不存在");
        }
    }

    pub fn get_server_events(&mut self) -> Vec<ServerEvent> {
        let mut events = vec![];
        events.append(&mut self.server_events.borrow_mut());
        events
    }

    //服务器同步数据时, 从这里获取附加数据
    pub fn get_sync_data(&mut self) -> Vec<SyncData> {
        let mut data = vec![];
        for sprite in self.engine.sprites() {
            let sprite = sprite.borrow();
            let mut extra = None;
            //玩家信息
            if sprite.bitmap().id() == RES_TANK_BITMAP {
                extra = Some(ExtraData {
                    score: sprite.score() as u16,
                    lives: sprite.lives() as u16,
                });
            }

            data.push(SyncData {
                id: sprite.id(),
                frame: sprite.cur_frame() as u8,
                x: sprite.position().left as i16,
                y: sprite.position().top as i16,
                res: sprite.bitmap().id(),
                velocity_x: sprite.velocity().x as f32,
                velocity_y: sprite.velocity().y as f32,
                extra: extra,
            });
        }

        data
    }

    pub fn sprites(&mut self) -> &Vec<Rc<RefCell<Sprite>>> {
        self.engine.sprites()
    }

    //窗口大小改变时，画布适应窗口
    fn client_resize_window(&mut self) {
        let platform = self.client_context.as_ref().unwrap().platform.clone();
        if platform.window_inner_width()<platform.window_inner_height(){
            //竖屏
            self.orientation = PORTRAIT;
        }else{
            //横屏
            self.orientation = LANDSCAPE;
        }
        platform.set_orientation(self.orientation);
        platform.set_canvas_width(platform.window_inner_width());
        platform.set_canvas_height(platform.window_inner_height());
    }

    fn client_handle_message(&mut self, messages: Vec<Vec<u8>>) {
        let platform = self.client_context.as_ref().unwrap().platform.clone();
        let mut messages = messages;
        for message in &mut messages {
            //context.console_log(&format!("message {:?}", message));
            let msg_id = message.remove(0);
            match msg_id {
                SERVER_MSG_ERR => {
                    //context.console_log(&format!("SERVER_MSG_ERR {:0.2}K", msg_len));
                    let r: Result<String, _> = deserialize(&message[..]);
                    if let Ok(msg) = r {
                        platform.console_log(&format!("服务器错误:{}", msg));
                    } else {
                        platform.console_log(&format!(
                            "SERVER_MSG_ERR 消息解析失败 {:?}",
                            r.err()
                        ));
                    }
                }
                SERVER_MSG_PLAYERS => {
                    //MSG_START发送之后，收到此消息(此时已经有current_player_name和current_player_ip)
                    //用户信息
                    let r: Result<Vec<(u32, String, String)>, _> = deserialize(&message[..]);
                    if let Ok(players) = r {
                        for (uid, ip, name) in players {
                            //保存当前玩家id
                            if ip == self.current_player_ip {
                                platform.console_log(&format!("current_player_ip={}", uid));
                                self.current_player_id = uid;
                            }
                            self.clinet_player_names.insert(uid, (ip, name.clone()));

                            if let Some(player) = self.engine.query_sprite(uid) {
                                let mut player = player.borrow_mut();
                                platform.console_log(&format!(
                                    "设置了玩家姓名: {}-{}",
                                    uid, name
                                ));
                                player.set_name(name.clone());
                            }
                        }
                    } else {
                        platform.console_log(&format!(
                            "SERVER_MSG_PLAYERS 消息解析失败 {:?}",
                            r.err()
                        ));
                    }
                }
                SERVER_MSG_IP => {
                    //context.console_log(&format!("SERVER_MSG_UID {:0.2}K", msg_len));
                    let r: Result<String, _> = deserialize(&message[..]);
                    if let Ok(ip) = r {
                        platform.console_log(&format!("你的IP:{}", ip));
                        self.current_player_ip = ip;
                        //收到IP以后开始游戏
                        if let Ok(mut encoded) = serialize(&self.current_player_name) {
                            encoded.insert(0, MSG_START);
                            self.client_binary_messages.push(encoded);
                        }
                    } else {
                        platform.console_log(&format!(
                            "SERVER_MSG_UUID 消息解析失败 {:?}",
                            r.err()
                        ));
                    }
                }
                SERVER_MSG_SYNC => {
                    //context.console_log(&format!("SERVER_MSG_SYNC {:0.2}K {}", msg_len, context.current_time_millis()));
                    let r = deserialize(&message[..]);
                    if let Ok(msg) = r {
                        self.client_synchronize_sprites(msg);
                    } else {
                        platform.console_log(&format!(
                            "SERVER_MSG_SYNC 消息解析失败 {:?}",
                            r.err()
                        ));
                    }
                }
                SERVER_MSG_EVENT => {
                    let r: Result<Vec<ServerEvent>, _> = deserialize(&message[..]);
                    if let Ok(events) = r {
                        for event in events {
                            match event {
                                ServerEvent::PlayerDying(uid, name, killer_name) => {
                                    self.players.remove(&uid);
                                    self.leaders.retain(|&(luid, _rank)| uid != luid);
                                    //记录并显示死亡的玩家
                                    self.dying_players.push((0, killer_name.clone(), name));

                                    //检查当前玩家是否死亡
                                    if let Some(current_player) = self.current_player.as_ref() {
                                        let mut current_player = current_player.borrow_mut();
                                        if uid == current_player.id() {
                                            current_player.set_killer_name(killer_name);
                                            self.client_dying_delay_ms = 5000.0;
                                        }
                                    }
                                }

                                ServerEvent::PlayerJoin(ip, uid, name) => {
                                    //玩家上线
                                    platform.console_log(&format!(
                                        "玩家上线:{}-{}-{}",
                                        ip, uid, name
                                    ));
                                }
                            }
                        }
                    } else {
                        platform.console_log(&format!(
                            "SERVER_MSG_EVENT 消息解析失败 {:?}",
                            r.err()
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    pub fn players(&mut self) -> &mut HashMap<u32, Rc<RefCell<TankSprite>>> {
        &mut self.players
    }
}
