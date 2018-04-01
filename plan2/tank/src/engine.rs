use sprite::{Sprite, SA_KILL};
use std::time::{Duration, SystemTime};
use sprite::GameContext;

//GameEngine 负责创建游戏窗口、绘制和更新精灵

pub trait GameEngineHandler<T: SysTime, C: GameContext>:Sized{
    fn sprite_dying(&mut self, sprite_dying:&Sprite<T, C>);
    fn sprite_collision(&self, sprite_hitter:&Sprite<T, C>, sprite_hittee:&Sprite<T, C>)->bool;
}

pub struct GameEngine<T: SysTime, C: GameContext, H: GameEngineHandler<T, C>>{
    handler: H,
    sprites:Vec<Sprite<T, C>>
}

impl <T: SysTime, C: GameContext, H: GameEngineHandler<T, C>> GameEngine<T, C, H>{
    pub fn new(handler: H)->GameEngine<T, C, H>{
        GameEngine{
            handler: handler,
            sprites: vec![]
        }
    }

    pub fn add_sprite(&mut self, sprite:Sprite<T, C>){
        if self.sprites.len()>0 {
            for i in 0..self.sprites.len(){
                //根据z-order插入精灵到数组
                if sprite.z_order() < self.sprites[i].z_order(){
                    self.sprites.insert(i, sprite);
                    return;
                }
            }
        }
        //精灵的zOrder是最高的，放入Vec的末尾
        self.sprites.push(sprite);
    }

    pub fn draw_sprites(&self){
        //绘制所有的精灵
        for sprite in &self.sprites{
            sprite.draw();
        }
    }

    pub fn update_sprites(&mut self){
        //log_string(format!("sprites={}", self.sprites.len()).as_str().as_bytes());
        //更新所有精灵
        let mut sprites_to_kill:Vec<f64> = vec![];
        for i in 0..self.sprites.len(){
            //保存旧的精灵位置以防需要恢复
            let old_sprite_pos = *self.sprites[i].position();
            //更新精灵
            let sprite_action = self.sprites[i].update();

            //处理SA_ADDSPRITE
            // if sprite_action == SA_ADDSPRITE{
            //     //允许精灵添加它的精灵
            //     if let Some(sprite) = self.sprites[i].add_sprite(){
            //         self.add_sprite(sprite);
            //     }
            // }

            //处理 SA_KILL
            if sprite_action == SA_KILL{
                //通知游戏精灵死亡
                self.handler.sprite_dying(&self.sprites[i]);
                //杀死精灵
                sprites_to_kill.push(self.sprites[i].id());
                continue;
            }

            if self.check_sprite_collision(i){
                self.sprites[i].set_position_rect(old_sprite_pos);
            }
        }

        //删除死亡的精灵
        for sprite_id in sprites_to_kill{
            self.sprites.retain(|ref s|{
                s.id() != sprite_id
            });
        }
    }

    pub fn check_sprite_collision(&mut self, test_sprite_id:usize)->bool{
        //检查精灵是否和其他精灵相撞
        let test_sprite = &self.sprites[test_sprite_id];
        for i in 0..self.sprites.len(){
            //不检查精灵自己
            if i == test_sprite_id{
                continue;
            }
            if test_sprite.test_collison(self.sprites[i].position()){
                return self.handler.sprite_collision(&self.sprites[i], test_sprite);
            }
        }
        return false;
    }

    pub fn clean_up_sprites(&mut self){
        self.sprites.clear();
    }

    pub fn get_sprite(&mut self, id:f64)->Option<&mut Sprite<T, C>>{
        for sprite in &mut self.sprites{
            if sprite.id() == id{
                return Some(sprite);
            }
        }
        None
    }

    pub fn kill_sprite(&mut self, sprite:&Sprite<T, C>){
        if let Some(s) = self.get_sprite(sprite.id()){
            s.kill();
        }
    }
}


//计时器
pub trait Timer{
    fn ready_for_next_frame(&mut self) -> bool;
}

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