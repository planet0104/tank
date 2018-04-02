
mod engine;
mod sprite; 
use engine::{GameEngine, GameEngineHandler};
//use std::time::{Duration, SystemTime};
use sprite::{GameContext, Sprite};

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


//游戏循环由服务器和客户端各自执行
//TankGame提供所有游戏更新方法

struct GameHandler{}
impl GameEngineHandler for GameHandler{
    fn sprite_dying(&mut self, sprite_dying: &Sprite){
        
    }

    fn sprite_collision(&self, sprite_hitter: &Sprite, sprite_hittee: &Sprite)->bool{
        //检测子弹是否和坦克碰撞
        false
    }
}

struct MyContext{

}
impl GameContext for MyContext{
fn random(&self)->f64{
    0.0
}
    fn rand_int(&self, start:i32, end:i32) ->i32{
        0
    }
    fn draw_image_at(&self, res_id:i32, x:i32, y:i32){

    }
    fn draw_image(&self, res_id:i32, source_x:i32, source_y:i32, source_width:i32, source_height:i32, dest_x:i32, dest_y:i32, dest_width:i32, dest_height:i32){

    }
    fn fill_style(&self, style: &str){
        
    }
    fn fill_rect(&self, x:i32, y:i32, width:i32, height:i32){
        
    }
    fn fill_text(&self, text: &str, x:i32, y:i32){
        
    }
    fn current_time_millis(&self) -> u64{
        0
    }
}

pub struct TankGame{
    engine: GameEngine,
}

impl TankGame{
    pub fn new()->TankGame{
        TankGame{
           engine: GameEngine::new(GameHandler{})
        }
    }

    pub fn update(&mut self){
        
    }
}

/*

//计时器
// pub trait Timer{
//     fn ready_for_next_frame(&mut self) -> bool;
// }

pub trait SysTime:Sized{
    fn current_time_millis(&self) -> u64;
}

pub struct ClientTimer<T: SysTime>{
    sys_time:T,
    fps:u64,
    frame_time:u64,
    start_time:u64,
    next_time:u64,
    current_time:u64,
}

impl <T: SysTime> ClientTimer<T>{
    pub fn new(fps:u64, sys_time: T)->ClientTimer<T>{
        let t = sys_time.current_time_millis();
        ClientTimer{
            sys_time: sys_time,
            fps:fps,
            frame_time: 1000 / fps,
            start_time: t,
            next_time: t,
            current_time: 0,
        }
    }

    pub fn fps(&self)->u64{
        self.fps
    }
}

impl <T: SysTime> Timer for ClientTimer<T>{
    fn ready_for_next_frame(&mut self)->bool{
        
	    //逝去的时间
        self.current_time = self.sys_time.current_time_millis() - self.start_time;
        
        if self.current_time > self.next_time {
            //更新时间
            self.next_time = self.current_time + self.frame_time;
            true
        }else{
            false
        }
    }
}

pub struct InstantTimer{
    frame_time:u64,
    start_time:SystemTime,
    next_time:Duration,
}

impl InstantTimer{
    pub fn new(fps:u64)->InstantTimer{
        InstantTimer{
            frame_time: 1000 / fps,
            start_time: SystemTime::now(),
            next_time: Duration::from_millis(0)
        }
    }

    pub fn _start(&mut self){
        //设置计数器起始值
        self.start_time = SystemTime::now();
        //更新时间在下一帧使用
        self.next_time = Duration::from_millis(0);
    }

    //逝去的毫秒数
    pub fn elapsed_secs(&self)->f64{
        let duration = self.start_time.elapsed().unwrap();
        duration.as_secs() as f64
           + duration.subsec_nanos() as f64 * 1e-9
    }
}

impl Timer for InstantTimer{
    fn ready_for_next_frame(&mut self)->bool{
        if self.start_time.elapsed().unwrap() > self.next_time {
            //更新时间
            self.next_time = self.start_time.elapsed().unwrap() + Duration::from_millis(self.frame_time);
            true
        }else{
            false
        }
    }
}

*/