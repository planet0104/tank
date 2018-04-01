
mod engine;
mod sprite; 
use engine::{Timer, GameEngine, GameEngineHandler, SysTime};
use sprite::{GameContext, Sprite};

//游戏宽高
pub const CLIENT_WIDTH:i32 = 1000;
pub const CLIENT_HEIGHT:i32 = 1000;

struct GameHandler{}
impl <T: SysTime, C: GameContext> GameEngineHandler<T, C> for GameHandler{
    fn sprite_dying(&mut self, sprite_dying:&Sprite<T, C>){
        
    }

    fn sprite_collision(&self, sprite_hitter:&Sprite<T, C>, sprite_hittee:&Sprite<T, C>)->bool{
        //检测子弹是否和坦克碰撞
        false
    }
}

pub struct TankGame<I, T, C>
    where I :Timer, T: SysTime, C: GameContext{
    timer: I,
    engine: GameEngine<T, C, GameHandler>,
}

impl <I, T, C> TankGame<I, T, C>
    where I :Timer, T: SysTime, C: GameContext{
    pub fn new(timer: I)->TankGame<I, T, C>{
        TankGame{
            timer: timer,
            engine: GameEngine::new(GameHandler{}),
        }
    }

    pub fn ready_for_next_frame(&mut self) -> bool{
        self.timer.ready_for_next_frame()
    }

    pub fn update(&mut self){
        
    }
}