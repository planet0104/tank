//参考: https://www.hellorust.com/setup/wasm-target/
#[macro_use]
extern crate json;

mod sprite;
mod timer;
mod engine;
use engine::{GameEngine, GameEngineHandler};
use sprite::{Sprite, Point, Rect, BA_BOUNCE, BA_DIE, BA_WRAP, BitmapRes};
use std::ptr;
use std::mem::transmute;

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

//导入的JS帮助函数
extern {
    pub fn log(text: *const u8, len:usize);
    pub fn current_time_millis()->f64;
    pub fn random()->f64;
    pub fn add_resource(res_id:i32, url: *const u8, len:usize);
    pub fn load_resource();
    pub fn request_animation_frame();
    pub fn window_width()->i32;
    pub fn window_height()->i32;
    pub fn set_canvas_size(width:i32, height:i32);
    pub fn set_canvas_margin(left:i32, top:i32, right:i32, bottom:i32);
    pub fn set_canvas_style_size(width:i32, height:i32);
    pub fn set_canvas_font(font: *const u8, len:i32);
    pub fn canvas_offset_left()->i32;
    pub fn fill_style_rgb(r:u8, g:u8, b:u8);
    pub fn fill_style(text: *const u8, len:usize);
    pub fn fill_rect(x:i32, y:i32, width:i32, height:i32);
    pub fn fill_text(text: *const u8, len:usize, x:i32, y:i32);
    pub fn draw_image_at(res_id:i32, x:i32, y:i32);
    pub fn draw_image(res_id:i32, source_x:i32, source_y:i32, source_width:i32, source_height:i32, dest_x:i32, dest_y:i32, dest_width:i32, dest_height:i32);
    pub fn send_message(text: *const u8, len:usize);
    pub fn ready();
}

//----------------------------------------------
//--------------以下为导出到JS的函数-------------
//----------------------------------------------

fn log_string(msg: &str){
    unsafe{
        log(msg.as_ptr(), msg.len());
    }
}

#[no_mangle]
pub unsafe fn run() {
    set_canvas_font("50px Arial".as_ptr(), 10);
    set_canvas_size(CLIENT_WIDTH, CLIENT_HEIGHT);
    on_window_resize();
    log_string("游戏启动...");
    let url = "tank.png";
    add_resource(RES_TANK_BITMAP, url.as_ptr(), url.len());
    let url = "missile.png";
    add_resource(RES_MISSILE_BITMAP, url.as_ptr(), url.len());
    let url = "sm_explosion.png";
    add_resource(RES_SM_EXPLOSION__BITMAP, url.as_ptr(), url.len());
    let url = "lg_explosion.png";
    add_resource(RES_LG_EXPLOSION_BITMAP, url.as_ptr(), url.len());
    load_resource();
}

#[no_mangle]
pub unsafe fn on_window_resize() {
    //调整画布大小
    let (window_width, window_height) = (window_width()-5, window_height()-5);
    let (canvas_style_width, canvas_style_height) = 
        if window_width < window_height{
            //竖屏
            (window_width, (window_width as f32/CLIENT_WIDTH as f32 * CLIENT_HEIGHT as f32) as i32)
        }else{
            ((window_height as f32/CLIENT_HEIGHT as f32 * CLIENT_WIDTH as f32) as i32, window_height)
        };
        set_canvas_style_size(canvas_style_width, canvas_style_height);
        //居中
        set_canvas_margin(
            (window_width-canvas_style_width)/2,
            (window_height-canvas_style_height)/2,
            0,
            0);
}

#[no_mangle]
pub unsafe fn on_load_resource_progress(current:i32, total:i32){
    log_string(&format!("资源加载中({}/{})...", current, total));
}

#[no_mangle]
pub unsafe fn on_resources_load() {
    //资源加载完成启动游戏
    log_string("资源加载完成");
    request_animation_frame();
    ready();
}

//游戏循环主函数(由window.requestAnimationFrame调用)
#[no_mangle]
pub unsafe fn draw_frame() {
    let game = game();
    if  game.engine.ready_for_next_frame(){
        fill_style("#2e6da3".as_ptr(), 7);
        fill_rect(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT);
        //更新精灵
        game.engine.update_sprites();
        //绘制精灵
        game.engine.draw_sprites();
    }
    request_animation_frame();
}

#[no_mangle]
pub fn on_touch_event(event:i32, x:i32, y:i32){
    //处理鼠标、触摸事件
}

#[no_mangle]
pub unsafe fn on_keyup_event(keycode:i32){
    game().on_keyup_event(keycode);
}
#[no_mangle]
pub unsafe fn on_keydown_event(keycode:i32){
    game().on_keydown_event(keycode);
}
#[no_mangle]
pub unsafe fn on_connect(){
    log_string("on_connect..");
    //websocket连接成功
    //去服务器请求数据
    let msg_obj = object!{
        "i" => 4,
        "g" => "Tank",
    };
    let msg = json::stringify(msg_obj);
    log_string(&format!("on_connect send:{}", msg));
    send_message(msg.as_ptr(), msg.len());
}

#[no_mangle]
pub unsafe fn on_message(len:usize){
    //读取消息
    let msg = std::str::from_utf8(&game().message_buffer[0..len]).unwrap();
    
    let json = json::parse(msg);
    if json.is_err(){
        log_string("消息格式错误");
        return;
    }
    let json = json.unwrap();
    if let Some(msg_id) = json["i"].as_i32(){
        match msg_id{
            MSG_CREATE | MSG_UPDATE =>{
                log_string("message:创建/修改 精灵")
            },
            MSG_DELETE =>{
                log_string("message:精灵死亡")
            },
            MSG_QUERY =>{
                log_string("message:拉取数据");
                let game = game();
                //添加服务器端的精灵
                

                //加入游戏
                game.join_game();
                //发送消息
                let sprite = game.engine.get_sprite(game.current_player_id).unwrap();
                //发送上线消息
                let msg_obj = object!{
                    "i" => MSG_CREATE,  //消息ID
                    "g" => GMAE_TITLE,  //游戏名称
                    "s" => object!{
                        "i" => format!("{}", sprite.id()), //精灵ID
                        "v" => object!{
                            "b" => sprite.bitmap().id(), //资源ID
                            "l" => sprite.position().left,  //left
                            "t" => sprite.position().top,   //top
                            "x" => sprite.velocity().x,     //x速度
                            "y" => sprite.velocity().y,     //y速度
                            "n" => if let Some(name) = sprite.name(){ (*name).clone()  }else{ String::new() } //精灵label
                        }
                    }
                };
                let msg = json::stringify(msg_obj);
                send_message(msg.as_ptr(), msg.len());
            },
            _ => ()
        }
    }
}

#[no_mangle]
pub fn get_message_buffer()->*const u8{
    //字符串指针传递给javascript
    game().message_buffer.as_ptr()
}

//生成指定范围的随即整数
pub fn rand_int(l:i32, b:i32)->i32{
    unsafe{
        ((random()*(b as f64 - l as f64 + 1.0)).floor()+l as f64) as i32
    }
}

static mut GAME:*const Game = ptr::null_mut();

//获取全局的Game实例
fn game<'a>() -> &'a mut Game {
    unsafe {
        if GAME.is_null() {
            GAME = transmute(Box::new(Game::new()));
        }
        transmute(GAME)
    }
}

//游戏引擎回调函数
struct GameHandler{}
impl GameEngineHandler for GameHandler{
    fn sprite_dying(&mut self, sprite_dying:&Sprite){
        let game = game();
        let bitmap_id = sprite_dying.bitmap().id();
        match bitmap_id{
            //玩家死亡
            RES_TANK_BITMAP => {
                if sprite_dying.id() == game.current_player_id {
                    log_string("玩家死亡");
                    game.current_player_id = 0.0;
                }else{
                    log_string("敌人死亡");
                }
            }
            //子弹精灵死亡
            RES_MISSILE_BITMAP => {
                //在子弹位置创建一个小的爆炸精灵
                let mut sprite = Sprite::from_bitmap(
                    BitmapRes::new(RES_SM_EXPLOSION__BITMAP, 17, 136),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
                sprite.set_num_frames(8, true);
                sprite.set_position(sprite_dying.position().left, sprite_dying.position().top);
                game.engine.add_sprite(sprite);
            }
            _=> ()
        }
    }

    fn sprite_collision(&self, sprite_hitter:&Sprite, sprite_hittee:&Sprite)->bool{
        //检测子弹是否和坦克碰撞
        let hitter = sprite_hitter.bitmap().id();
        let hittee = sprite_hittee.bitmap().id();
        if hitter == RES_MISSILE_BITMAP && hittee == RES_TANK_BITMAP ||
           hitter == RES_TANK_BITMAP && hittee == RES_MISSILE_BITMAP{
            //杀死子弹和坦克
            game().engine.kill_sprite(sprite_hittee);
            game().engine.kill_sprite(sprite_hitter);

            //在坦克位置创建一个大的爆炸精灵
            let pos:&Rect = if hitter == RES_TANK_BITMAP{
                sprite_hitter.position()
            }else{
                sprite_hittee.position()
            };
            let mut sprite = Sprite::from_bitmap(
                BitmapRes::new(RES_LG_EXPLOSION_BITMAP, 33, 272),
                Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT));
            sprite.set_num_frames(8, true);
            sprite.set_position(pos.left, pos.top);
            game().engine.add_sprite(sprite);
        }

        //检测子弹和子弹是否碰撞
        if hitter == RES_MISSILE_BITMAP && hittee == RES_MISSILE_BITMAP{
            //杀死子弹
            game().engine.kill_sprite(sprite_hittee);
            game().engine.kill_sprite(sprite_hitter);
        }
        false
    }
}

//游戏主结构体
pub struct Game{
    engine: GameEngine,
    current_player_id: f64,
    message_buffer: [u8; 1024]
}

impl Game{
    fn new()->Game{
        Game{
            engine: GameEngine::new(30, CLIENT_WIDTH, CLIENT_HEIGHT, GameHandler{}),
            current_player_id: 0.0,
            message_buffer: [0; 1024]
        }
    }

    //新用户加入游戏
    pub fn join_game(&mut self) {
        //创建玩家坦克
        let mut tank_sprite = Sprite::with_bounds_action(
                            BitmapRes::new(RES_TANK_BITMAP, 36, 144),
                            Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_WRAP);
        self.current_player_id = tank_sprite.id();
        tank_sprite.set_num_frames(4, false);
        tank_sprite.set_frame_delay(-1);
        self.engine.add_sprite(tank_sprite);
    }

    pub fn on_keyup_event(&mut self, keycode:i32){
        if self.current_player_id == 0.0 {
            return;
        }
        match keycode{
            KEYCODE_SPACE=>{
                let tank_position = *(self.engine.get_sprite(self.current_player_id).unwrap().position());
                //创建一个新的子弹精灵
                let mut sprite = Sprite::with_bounds_action(
                    BitmapRes::new(RES_MISSILE_BITMAP, 17, 68),
                    Rect::new(0, 0, CLIENT_WIDTH, CLIENT_HEIGHT), BA_DIE);
                sprite.set_num_frames(4, false);
                sprite.set_frame_delay(-1);
                //子弹的方向同玩家的方向
                let direction = self.engine.get_sprite(self.current_player_id).unwrap().current_frame();
                sprite.set_current_frame(direction);
                match direction{
                    0 => {
                        sprite.set_velocity(0, -MISSILE_VELOCITY);
                        sprite.set_position(tank_position.left+(tank_position.right-tank_position.left)/2-8, tank_position.top-17);
                    }
                    1 => {
                        sprite.set_velocity(0, MISSILE_VELOCITY);
                        sprite.set_position(tank_position.left+(tank_position.right-tank_position.left)/2-8, tank_position.bottom);
                    }
                    2 => {
                        sprite.set_velocity(-MISSILE_VELOCITY, 0);
                        sprite.set_position(tank_position.left-17, tank_position.top-(tank_position.top-tank_position.bottom)/2-8);
                    }
                    3 => {
                        sprite.set_velocity(MISSILE_VELOCITY, 0);
                        sprite.set_position(tank_position.right, tank_position.top-(tank_position.top-tank_position.bottom)/2-8);
                    }
                    _=> ()
                }
                self.engine.add_sprite(sprite);
            }
            KEYCODE_LEFT | KEYCODE_RIGHT | KEYCODE_UP | KEYCODE_DOWN =>{
                let tank = self.engine.get_sprite(self.current_player_id).unwrap();
                tank.set_velocity(0, 0);
            }
            _ => ()
        }
    }

    pub fn on_keydown_event(&mut self, keycode:i32){
        if self.current_player_id == 0.0 {
            return;
        }
         match keycode{
            KEYCODE_LEFT => {
                let tank = self.engine.get_sprite(self.current_player_id).unwrap();
                tank.set_current_frame(2);
                tank.set_velocity(-TANK_VELOCITY, 0);
            }
            KEYCODE_RIGHT => {
                let tank = self.engine.get_sprite(self.current_player_id).unwrap();
                tank.set_current_frame(3);
                tank.set_velocity(TANK_VELOCITY, 0);
            }
            KEYCODE_UP => {
                let tank = self.engine.get_sprite(self.current_player_id).unwrap();
                tank.set_current_frame(0);
                tank.set_velocity(0, -TANK_VELOCITY);
            }
            KEYCODE_DOWN => {
                let tank = self.engine.get_sprite(self.current_player_id).unwrap();
                tank.set_current_frame(1);
                tank.set_velocity(0, TANK_VELOCITY);
            }
            _ => ()
        }
    }
}