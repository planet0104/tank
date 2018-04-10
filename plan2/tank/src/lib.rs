
extern crate uuid;
extern crate rand;
#[macro_use]
extern crate num_derive;
pub mod utils;
mod engine;
mod sprite; 
use engine::{GameEngine, CanvasContext};
use sprite::{BA_DIE, BA_WRAP, Sprite, BitmapRes, Rect };
use std::collections::HashMap;

//游戏宽高
pub const CLIENT_WIDTH:i32 = 1000;
pub const CLIENT_HEIGHT:i32 = 1000;

//--------------------------------------------
//-------------游戏资源ID----------------------
//--------------------------------------------
pub const RES_TANK_BITMAP:i32 = 0;
pub const RES_MISSILE_BITMAP:i32 = 1;
pub const RES_LG_EXPLOSION_BITMAP:i32 = 2;
pub const RES_SM_EXPLOSION__BITMAP:i32 = 3;

pub const TANK_VELOCITY:i32 = 6;
pub const MISSILE_VELOCITY:i32 = 2;

#[derive(FromPrimitive, ToPrimitive)]
pub enum MouseEvent{
    MOUSE_MOVE,
    MOUSE_CLICK,
}

pub const KEYCODE_LEFT:i32 = 37;
pub const KEYCODE_RIGHT:i32 = 39;
pub const KEYCODE_UP:i32 = 38;
pub const KEYCODE_DOWN:i32 = 40;
pub const KEYCODE_SPACE:i32 = 32;

pub const GMAE_TITLE:&'static str = "Tank";

#[derive(FromPrimitive, ToPrimitive)]
pub enum KeyEvent {
    KEY_DOWN,
    KEY_UP,
}

pub enum SpriteEvent{
    Add,
    Update,
    Delete
}

pub struct SpriteInfo{
    pub id: String,
    pub res: i32,//资源ID
    pub l: i32,
    pub t: i32,
    pub r: i32,
    pub b: i32,
    pub vx: i32,//x速度
    pub vy: i32,//y速度
    pub frame: i32//当前帧
}

pub struct Player{
    pub name: String
}

/*
游戏循环由服务器和客户端各自执行(HTML5中游戏循环需要调用request_animation_frame)
TankGame提供所有游戏更新方法

服务端只update() 方法、键盘、鼠标事件处理， 处理完之后将会产生message，message被分发给改各个客户端
客户端调用 update_sprites(), draw() 方法, handle_event方法(处理添加精灵、更新精灵、删除精灵)； 键盘事件发送给websocket
(客户端不处理碰撞检测, 服务器检测到碰撞会自动将精灵状态下发到客户端)
*/
pub struct TankGame{
    engine: GameEngine,
    events: Vec<(SpriteEvent, SpriteInfo)>,
    players: HashMap<String, Player>
}

impl TankGame{
    pub fn new()->TankGame{
        TankGame{
           engine: GameEngine::new(),
           events: vec![],
           players: HashMap::new()
        }
    }

    //创建游戏精灵
    pub fn add_sprite(engine:&mut GameEngine, id:Option<&str>, res:i32) -> usize {
        match res{
            RES_TANK_BITMAP => {
                //创建玩家坦克
                let mut tank_sprite = Sprite::with_bounds_action(
                                    BitmapRes::new(RES_TANK_BITMAP, 36, 144),
                                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_WRAP);
                tank_sprite.set_num_frames(4, false);
                tank_sprite.set_frame_delay(-1);
                tank_sprite.id = String::from(id.unwrap_or(&tank_sprite.id));
                engine.add_sprite(tank_sprite)
            }
            RES_MISSILE_BITMAP => {
                //创建一个新的子弹精灵
                let mut sprite = Sprite::with_bounds_action(
                    BitmapRes::new(RES_MISSILE_BITMAP, 17, 68),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_DIE);
                sprite.set_num_frames(4, false);
                sprite.set_frame_delay(-1);
                sprite.id = String::from(id.unwrap_or(&sprite.id));
                engine.add_sprite(sprite)
            },
            RES_SM_EXPLOSION__BITMAP => {
                //创建小的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    BitmapRes::new(RES_SM_EXPLOSION__BITMAP, 17, 136),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
                sprite.set_num_frames(8, true);
                sprite.id = String::from(id.unwrap_or(&sprite.id));
                engine.add_sprite(sprite)
            },
            RES_LG_EXPLOSION_BITMAP => {
                //创建一个大的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
                sprite.set_num_frames(8, true);
                sprite.id = String::from(id.unwrap_or(&sprite.id));
                engine.add_sprite(sprite)
            }
            _ => 0
        }
    }

    //玩家加入游戏
    pub fn join_game(&mut self, id:Option<&str>, name:Option<&str>){
        //添加坦克精灵
        let sprite_index = TankGame::add_sprite(&mut self.engine, id, RES_TANK_BITMAP);
        self.add_sprite_event(SpriteEvent::Add, sprite_index);//添加事件
        let sprite = &mut self.engine.sprites()[sprite_index];
        //添加玩家信息
        self.players.insert(sprite.id.clone(), Player{
            name: String::from(name.unwrap_or(""))
        });
    }

    //离开游戏/断线
    pub fn leave_game(&mut self, id:&String){
        //查找玩家id对应的精灵, 将其删除
        self.players.remove(id);
        if let Some(index) = self.engine.query_sprite_idx(id){
            self.add_sprite_event(SpriteEvent::Delete, index);//事件
            self.engine.sprites()[index].kill();
        }
    }

    //客户端接受到服务器发送来的消息，将消息传递给此方法，来更新渲染
    pub fn handle_server_event(&mut self, event: SpriteEvent, sprite_info: SpriteInfo){
        if let Some(sprite_idx) = match event{
            SpriteEvent::Add => {
                Some(TankGame::add_sprite(&mut self.engine, Some(&sprite_info.id), sprite_info.res))
            }
            SpriteEvent::Update => {
                self.engine.query_sprite_idx(&sprite_info.id)
            },
            SpriteEvent::Delete => {
                if let Some(sprite) = self.engine.query_sprite(&sprite_info.id){
                    sprite.kill();
                }
                None
            }
        }{
            //设置精灵信息
            let mut sprite = &mut self.engine.sprites()[sprite_idx];
            sprite.set_position_rect(Rect::new(sprite_info.l, sprite_info.t, sprite_info.r, sprite_info.b));
            sprite.set_velocity(sprite_info.vx, sprite_info.vy);
            sprite.set_current_frame(sprite_info.frame);
        }
    }

    //客户端不在update_sprites处理函数中做任何操作如:精灵死亡添加爆炸、碰撞检测杀死精灵等
    //客户端仅按帧更新精灵位置，所有精灵创建、更新都由服务器下发事件中处理
    pub fn update_sprites(&mut self){
        self.engine.update_sprites(&mut |_,_|{}, |_,_,_|{false});
    }

    //更新游戏
    pub fn update(&mut self){
        let mut sprites_dying = vec![];
        let mut sprites_add = vec![];

        //更新游戏，并处理精灵死亡、碰撞检测回调
        self.engine.update_sprites(&mut |engine:&mut GameEngine, idx_sprite_dying|{

            sprites_dying.push(idx_sprite_dying);
            //精灵死亡
            let bitmap_id = engine.sprites()[idx_sprite_dying].bitmap().id();
            //在精灵位置创建不同的爆炸精灵
            let res =  match bitmap_id{
                RES_MISSILE_BITMAP => {
                    RES_SM_EXPLOSION__BITMAP
                }
                _ => RES_LG_EXPLOSION_BITMAP
            };
            let idx = TankGame::add_sprite(engine, None, res);
            let pos = *engine.sprites()[idx_sprite_dying].position();
            engine.sprites()[idx].set_position(pos.left, pos.top);
            sprites_add.push(idx);

        }, |engine, idx_sprite_hitter, idx_sprite_hittee|{
            //此处杀死的精灵, 会在下次更新时，调用上边sprite_dying函数
            //碰撞检测
            let hitter = engine.sprites()[idx_sprite_hitter].bitmap().id();
            let hittee = engine.sprites()[idx_sprite_hittee].bitmap().id();
            if hitter == RES_MISSILE_BITMAP && hittee == RES_TANK_BITMAP ||
            hitter == RES_TANK_BITMAP && hittee == RES_MISSILE_BITMAP{
                //杀死相撞的子弹和坦克
                engine.kill_sprite(idx_sprite_hittee);
                engine.kill_sprite(idx_sprite_hitter);
                true
            }else if hitter == RES_MISSILE_BITMAP && hittee == RES_MISSILE_BITMAP{
                //检测子弹和子弹是否碰撞
                engine.kill_sprite(idx_sprite_hittee);
                engine.kill_sprite(idx_sprite_hitter);
                true
            }else{
                false
            }
        });
        
        //添加精灵事件
        for idx in sprites_add{
            self.add_sprite_event(SpriteEvent::Add, idx);
        }
        //删除精灵事件
        for idx in sprites_dying{
            self.add_sprite_event(SpriteEvent::Delete, idx);
            //坦克死亡将玩家删除
            let sprite = &self.engine.sprites()[idx];
            if sprite.bitmap().id() == RES_TANK_BITMAP{
                self.players.remove(&sprite.id);
            }
        }
    }

    //添加要分发的事件
    fn add_sprite_event(&mut self, event: SpriteEvent, sprite_idx:usize){
        let sprite = &self.engine.sprites()[sprite_idx];
        self.events.push((event, SpriteInfo{
            id: sprite.id.clone(),
            res: sprite.bitmap().id(),
            l: sprite.position().left,
            t: sprite.position().top,
            r: sprite.position().right,
            b: sprite.position().bottom,
            vx: sprite.velocity().x,
            vy: sprite.velocity().y,
            frame: sprite.current_frame()
        }));
    }

    //绘制游戏
    pub fn draw(&self, context: &CanvasContext){
        context.fill_style("#2e6da3");
        context.fill_rect(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);
        self.engine.draw_sprites(context);
    }

    //键盘按下，坦克移动、发射子弹
    pub fn on_keyup_event(&mut self, event: &KeyEvent, keycode:i32, sprite_id: &String){
        let idx = self.engine.query_sprite_idx(sprite_id).unwrap();

        match event{
            KeyEvent::KEY_UP{

            }
            
            KeyEvent::KEY_DOWN{

            }
        }

        match keycode{
            KEYCODE_SPACE=>{
                let tank_position = *(self.engine.sprites()[idx].position());
                //创建一个新的子弹精灵
                let idx = TankGame::add_sprite(&mut self.engine, None, RES_MISSILE_BITMAP);
                //子弹的方向同玩家的方向
                let direction = self.engine.query_sprite(sprite_id).unwrap().current_frame();
                self.engine.sprites()[idx].set_current_frame(direction);
                match direction{
                    0 => {
                        self.engine.sprites()[idx].set_velocity(0, -MISSILE_VELOCITY);
                        self.engine.sprites()[idx].set_position(tank_position.left+(tank_position.right-tank_position.left)/2-8, tank_position.top-17);
                    }
                    1 => {
                        self.engine.sprites()[idx].set_velocity(0, MISSILE_VELOCITY);
                        self.engine.sprites()[idx].set_position(tank_position.left+(tank_position.right-tank_position.left)/2-8, tank_position.bottom);
                    }
                    2 => {
                        self.engine.sprites()[idx].set_velocity(-MISSILE_VELOCITY, 0);
                        self.engine.sprites()[idx].set_position(tank_position.left-17, tank_position.top-(tank_position.top-tank_position.bottom)/2-8);
                    }
                    3 => {
                        self.engine.sprites()[idx].set_velocity(MISSILE_VELOCITY, 0);
                        self.engine.sprites()[idx].set_position(tank_position.right, tank_position.top-(tank_position.top-tank_position.bottom)/2-8);
                    }
                    _=> {}
                }   
                self.add_sprite_event(SpriteEvent::Update, idx);
            }
            KEYCODE_LEFT | KEYCODE_RIGHT | KEYCODE_UP | KEYCODE_DOWN =>{
                self.engine.sprites()[idx].set_velocity(0, 0);
                self.add_sprite_event(SpriteEvent::Update, idx);
            }
            _ => {}
        }
    }

    //键盘弹起坦克停止走动
    pub fn on_keydown_event(&mut self, keycode:i32, sprite_id: &String){
        let idx = self.engine.query_sprite_idx(sprite_id).unwrap();
         match keycode{
            KEYCODE_LEFT => {
                self.engine.sprites()[idx].set_current_frame(2);
                self.engine.sprites()[idx].set_velocity(-TANK_VELOCITY, 0);
                self.add_sprite_event(SpriteEvent::Update, idx);
            }
            KEYCODE_RIGHT => {
                self.engine.sprites()[idx].set_current_frame(3);
                self.engine.sprites()[idx].set_velocity(TANK_VELOCITY, 0);
                self.add_sprite_event(SpriteEvent::Update, idx);
            }
            KEYCODE_UP => {
                self.engine.sprites()[idx].set_current_frame(0);
                self.engine.sprites()[idx].set_velocity(0, -TANK_VELOCITY);
                self.add_sprite_event(SpriteEvent::Update, idx);
            }
            KEYCODE_DOWN => {
                self.engine.sprites()[idx].set_current_frame(1);
                self.engine.sprites()[idx].set_velocity(0, TANK_VELOCITY);
                self.add_sprite_event(SpriteEvent::Update, idx);
            }
            _ => {}
        }
    }

    pub fn events(&mut self) -> &mut Vec<(SpriteEvent, SpriteInfo)> {
        &mut self.events
    }

    pub fn sprites(&mut self) -> &Vec<Sprite> {
        self.engine.sprites()
    }
}