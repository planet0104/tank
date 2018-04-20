extern crate rand;
extern crate uuid;
pub mod utils;
pub mod engine;
pub mod sprite;
use uuid::Uuid;
use engine::{CanvasContext, GameEngine};
use sprite::{BitmapRes, Rect, Point, Sprite, BA_DIE, BA_WRAP};
use std::collections::HashMap;
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
    pub score: i32
}

pub struct Player {
    pub name: String,
}

/*
游戏循环由服务器和客户端各自执行(HTML5中游戏循环需要调用request_animation_frame)
TankGame提供所有游戏更新方法

服务端只update() 方法、键盘、鼠标事件处理， 处理完之后将会产生message，message被分发给改各个客户端
客户端调用 update_sprites(), draw() 方法, handle_event方法(处理添加精灵、更新精灵、删除精灵)； 键盘事件发送给websocket
(客户端不处理碰撞检测, 服务器检测到碰撞会自动将精灵状态下发到客户端)
*/
pub struct TankGame {
    pub engine: GameEngine,
    events: Vec<(SpriteEvent, SpriteInfo)>,
    players: HashMap<String, Player>,
}

impl TankGame {
    pub fn new() -> TankGame {
        TankGame {
            engine: GameEngine::new(),
            events: vec![],
            players: HashMap::new(),
        }
    }

    //创建游戏精灵
    pub fn add_sprite(engine: &mut GameEngine, id: String, res: i32) -> usize {
        match res {
            RES_TANK_BITMAP => {
                //创建玩家坦克
                let mut tank_sprite = Sprite::with_bounds_action(
                    id,
                    BitmapRes::new(RES_TANK_BITMAP, 36, 144),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                    BA_WRAP
                );
                tank_sprite.set_num_frames(4, false);
                tank_sprite.set_frame_delay(-1);
                engine.add_sprite(tank_sprite)
            }
            RES_MISSILE_BITMAP => {
                //创建一个新的子弹精灵
                let mut sprite = Sprite::with_bounds_action(
                    id,
                    BitmapRes::new(RES_MISSILE_BITMAP, 17, 68),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT),
                    BA_DIE
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

    //玩家加入游戏
    pub fn join_game(&mut self, id: String, name: String) {
        println!("join_game: {} {}", id, name.clone());
        //添加坦克精灵
        let sprite_index =
            TankGame::add_sprite(&mut self.engine, id.clone(), RES_TANK_BITMAP);
        //添加玩家信息
        self.engine.sprites()[sprite_index].set_name(name.clone());
        self.players.insert(
            self.engine.sprites()[sprite_index].id.clone(),
            Player { name: name.clone() },
        );
        self.events.push(TankGame::get_event_info(SpriteEvent::Add, &self.engine.sprites()[sprite_index]));//添加事件
        println!("join_game {} {} 在线人数:{}", id, name, self.players.len());
    }

    //离开游戏/断线
    pub fn leave_game(&mut self, id: &String) {
        //查找玩家id对应的精灵, 将其删除
        self.players.remove(id);
        if let Some(index) = self.engine.query_sprite_idx(id) {
            self.events.push(TankGame::get_event_info(SpriteEvent::Delete, &self.engine.sprites()[index]));//事件
            self.engine.sprites().remove(index); //直接删除, 不kill
        }
        println!("leave_game {} 在线人数:{}", id, self.players.len());
    }

    //客户端接受到服务器发送来的消息，将消息传递给此方法，来更新渲染
    pub fn handle_server_event(&mut self, event: SpriteEvent, sprite_info: SpriteInfo) {
        if let Some(sprite_idx) = match event {
            SpriteEvent::Add => Some(TankGame::add_sprite(
                &mut self.engine,
                sprite_info.id,
                sprite_info.res_id
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

    //客户端不在update_sprites处理函数中做任何操作如:精灵死亡添加爆炸、碰撞检测杀死精灵等
    //客户端仅按帧更新精灵位置，所有精灵创建、更新都由服务器下发事件中处理
    pub fn update_sprites(&mut self) {
        self.engine.update_sprites(&mut |_, _| {}, |_, _, _| false);
    }

    //更新游戏
    pub fn update(&mut self) {
        let mut sprites_dying_events = vec![];
        let mut sprites_add_events = vec![];

        //更新游戏，并处理精灵死亡、碰撞检测回调
        self.engine.update_sprites(
            &mut |engine: &mut GameEngine, idx_sprite_dying| {
                sprites_dying_events.push(TankGame::get_event_info(SpriteEvent::Delete, &engine.sprites()[idx_sprite_dying]));
                let bitmap_id = engine.sprites()[idx_sprite_dying].bitmap().id();
                //子弹精灵死亡添加小的爆炸精灵
                if bitmap_id == RES_MISSILE_BITMAP{
                    let idx = TankGame::add_sprite(engine, Uuid::new_v4().hyphenated().to_string(), RES_SM_EXPLOSION_BITMAP);
                    let pos = *engine.sprites()[idx_sprite_dying].position();
                    engine.sprites()[idx].set_position(pos.left, pos.top);
                    sprites_add_events.push(TankGame::get_event_info(SpriteEvent::Add, &engine.sprites()[idx]));
                }
                //坦克死亡添加大的爆炸精灵
                if bitmap_id == RES_TANK_BITMAP{
                    let idx = TankGame::add_sprite(engine, Uuid::new_v4().hyphenated().to_string(), RES_LG_EXPLOSION_BITMAP);
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
                        engine.sprites()[idx_sprite_hittee].set_killer(hitter_parent);
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
                        engine.sprites()[idx_sprite_hitter].set_killer(hittee_parent);
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
            self.events.push(e);
        }
        //删除精灵事件
        for e in sprites_dying_events {
            //坦克死亡将玩家删除
            if e.1.res_id == RES_TANK_BITMAP {
                self.players.remove(&e.1.id);
            }
            self.events.push(e);
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
                score: sprite.score()
            }
        )
    }

    //绘制游戏
    pub fn draw(&self, context: &CanvasContext) {
        context.fill_style("#2e6da3");
        context.fill_rect(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);
        context.fill_style("#3e7daf");
        context.set_canvas_font("90px 微软雅黑");
        context.fill_text("坦克大战", CLIENT_WIDTH/2-185, CLIENT_HEIGHT/2-50);
        context.set_canvas_font("32px 微软雅黑");
        context.fill_text("↑ ↓ ← → ：移动  空格：开炮", 100, CLIENT_HEIGHT/2+30);
        context.fill_text("(死了请刷新网页)", 180, CLIENT_HEIGHT/2+80);
        self.engine.draw_sprites(context);
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
                                TankGame::add_sprite(&mut self.engine, Uuid::new_v4().hyphenated().to_string(), RES_MISSILE_BITMAP);
                            
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
                            self.events.push(TankGame::get_event_info(SpriteEvent::Add, &self.engine.sprites()[missile_idx]));
                        }
                        VK_LEFT => {
                            self.engine.sprites()[idx].set_current_frame(2);
                            self.engine.sprites()[idx].set_velocity(-TANK_VELOCITY, 0);
                            self.events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        VK_RIGHT => {
                            self.engine.sprites()[idx].set_current_frame(3);
                            self.engine.sprites()[idx].set_velocity(TANK_VELOCITY, 0);
                            self.events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        VK_UP => {
                            self.engine.sprites()[idx].set_current_frame(0);
                            self.engine.sprites()[idx].set_velocity(0, -TANK_VELOCITY);
                            self.events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                        }
                        VK_DOWN => {
                            self.engine.sprites()[idx].set_current_frame(1);
                            self.engine.sprites()[idx].set_velocity(0, TANK_VELOCITY);
                            self.events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
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
                        self.events.push(TankGame::get_event_info(SpriteEvent::Update, &self.engine.sprites()[idx]));
                    }
                }
            }
        }else{
            println!("没有找到ID {}", sprite_id);
        }
    }

    pub fn events(&mut self) -> &mut Vec<(SpriteEvent, SpriteInfo)> {
        &mut self.events
    }

    pub fn sprites(&mut self) -> &Vec<Sprite> {
        self.engine.sprites()
    }
}
