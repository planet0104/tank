
extern crate uuid;
extern crate rand;
pub mod utils;
mod engine;
mod sprite; 
use engine::{GameEngine, CanvasContext};
use sprite::{BA_DIE, BA_WRAP, Sprite, BitmapRes, Rect };

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

//-----------------------------------
//-------------事件ID----------------
pub const EVENT_MOUSE_MOVE:i32 = 0;
pub const EVENT_MOUSE_CLICK:i32 = 1;
pub const EVENT_TOUCH_MOVE:i32 = 10;

pub const KEYCODE_LEFT:i32 = 37;
pub const KEYCODE_RIGHT:i32 = 39;
pub const KEYCODE_UP:i32 = 38;
pub const KEYCODE_DOWN:i32 = 40;
pub const KEYCODE_SPACE:i32 = 32;

pub const MSG_CREATE:i32 = 1;
pub const MSG_DELETE:i32 = 2;
pub const MSG_UPDATE:i32 = 3;
pub const MSG_QUERY:i32 = 4;
pub const GMAE_TITLE:&'static str = "Tank";

pub enum SpriteEvent{
    Add,
    Update,
    Delete
}

pub struct SpriteInfo{
    id: String,
    res: i32,//资源ID
    l: i32,
    t: i32,
    r: i32,
    b: i32,
    vx: i32,//x速度
    vy: i32,//y速度
    frame: i32//当前帧
}

/*
游戏循环由服务器和客户端各自执行(HTML5中游戏循环需要调用request_animation_frame)
TankGame提供所有游戏更新方法

服务端只调用 update() 方法、键盘、鼠标事件处理， 处理完之后将会产生message，message被分发给改各个客户端
客户端调用 update_sprites(), draw() 方法, handle_event方法(处理添加精灵、更新精灵、删除精灵)； 键盘事件发送给websocket
(客户端不处理碰撞检测, 服务器检测到碰撞会自动将精灵状态下发到客户端)
*/
pub struct TankGame{
    engine: GameEngine,
    events: Vec<(SpriteEvent, SpriteInfo)>
}

impl TankGame{
    pub fn new()->TankGame{
        TankGame{
           engine: GameEngine::new(),
           events: vec![]
        }
    }

    //创建游戏精灵
    pub fn add_sprite(engine:&mut GameEngine, res:i32) -> usize {
        match res{
            RES_TANK_BITMAP => {
                //创建玩家坦克
                let mut tank_sprite = Sprite::with_bounds_action(
                                    BitmapRes::new(RES_TANK_BITMAP, 36, 144),
                                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_WRAP);
                tank_sprite.set_num_frames(4, false);
                tank_sprite.set_frame_delay(-1);
                engine.add_sprite(tank_sprite)
            }
            RES_MISSILE_BITMAP => {
                //创建一个新的子弹精灵
                let mut sprite = Sprite::with_bounds_action(
                    BitmapRes::new(RES_MISSILE_BITMAP, 17, 68),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_DIE);
                sprite.set_num_frames(4, false);
                sprite.set_frame_delay(-1);
                engine.add_sprite(sprite)
            },
            RES_SM_EXPLOSION__BITMAP => {
                //创建小的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    BitmapRes::new(RES_SM_EXPLOSION__BITMAP, 17, 136),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
                sprite.set_num_frames(8, true);
                engine.add_sprite(sprite)
            },
            RES_LG_EXPLOSION_BITMAP => {
                //创建一个大的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
                sprite.set_num_frames(8, true);
                engine.add_sprite(sprite)
            }
            _ => 0
        }
    }

    //客户端接受到服务器发送来的消息，将消息传递给此方法，来更新渲染
    pub fn handle_event(&mut self, event: SpriteEvent, sprite_info: SpriteInfo){
        if let Some(sprite_idx) = match event{
            SpriteEvent::Add => {
                Some(TankGame::add_sprite(&mut self.engine, sprite_info.res))
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
        let mut events:Vec<(SpriteEvent, usize)> = vec![];
        self.engine.update_sprites(&mut |engine:&mut GameEngine, idx_sprite_dying|{

            events.push((SpriteEvent::Delete, idx_sprite_dying));//事件
            //精灵死亡
            let bitmap_id = engine.sprites()[idx_sprite_dying].bitmap().id();
            //在精灵位置创建不同的爆炸精灵
            let res =  match bitmap_id{
                RES_MISSILE_BITMAP => {
                    RES_SM_EXPLOSION__BITMAP
                }
                _ => RES_LG_EXPLOSION_BITMAP
            };
            let idx = TankGame::add_sprite(engine, res);
            let pos = *engine.sprites()[idx_sprite_dying].position();
            engine.sprites()[idx].set_position(pos.left, pos.top);
            events.push((SpriteEvent::Add, idx));//事件

        }, |engine, idx_sprite_hitter, idx_sprite_hittee|{

            //碰撞检测
            let hitter = engine.sprites()[idx_sprite_hitter].bitmap().id();
            let hittee = engine.sprites()[idx_sprite_hittee].bitmap().id();
            if hitter == RES_MISSILE_BITMAP && hittee == RES_TANK_BITMAP ||
            hitter == RES_TANK_BITMAP && hittee == RES_MISSILE_BITMAP{
                //杀死子弹和坦克
                engine.kill_sprite(idx_sprite_hittee);
                engine.kill_sprite(idx_sprite_hitter);
            }

            //检测子弹和子弹是否碰撞
            if hitter == RES_MISSILE_BITMAP && hittee == RES_MISSILE_BITMAP{
                //杀死子弹
                engine.kill_sprite(idx_sprite_hittee);
                engine.kill_sprite(idx_sprite_hitter);
            }
            true
        });
        
        //将事件存储的列表
        for event in events{
            self.add_sprite_event(event.0, event.1);
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
    pub fn on_keyup_event(&mut self, keycode:i32, sprite_id: &String){
        let idx = self.engine.query_sprite_idx(sprite_id).unwrap();
        match keycode{
            KEYCODE_SPACE=>{
                let tank_position = *(self.engine.sprites()[idx].position());
                //创建一个新的子弹精灵
                let idx = TankGame::add_sprite(&mut self.engine, RES_MISSILE_BITMAP);
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
}