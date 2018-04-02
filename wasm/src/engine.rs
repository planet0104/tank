use sprite::{Sprite, SA_ADDSPRITE, SA_KILL};
use timer::Timer;

//GameEngine 负责创建游戏窗口、绘制和更新精灵
pub trait GameEngineHandler{
    fn sprite_dying(&mut self, sprite_dying:&Sprite);
    fn sprite_collision(&self, sprite_hitter:&Sprite, sprite_hittee:&Sprite)->bool;
}

pub struct GameEngine{
    handler:Box<GameEngineHandler>,
    timer:Timer,
    _width:i32,
    _height:i32,
    sprites:Vec<Sprite>
}

impl GameEngine{
    pub fn new<T: GameEngineHandler + 'static>(fps:u64, width:i32, height:i32, handler:T)->GameEngine{
        GameEngine{
            handler:Box::new(handler),
            timer:Timer::new(fps),
            _width: width,
            _height: height,
            sprites: vec![]
        }
    }

    pub fn add_sprite(&mut self, sprite:Sprite){
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

    pub fn get_sprite(&mut self, id:f64)->Option<&mut Sprite>{
        for sprite in &mut self.sprites{
            if sprite.id() == id{
                return Some(sprite);
            }
        }
        None
    }

    pub fn ready_for_next_frame(&mut self)->bool{
        self.timer.ready_for_next_frame()
    }

    pub fn kill_sprite(&mut self, sprite:&Sprite){
        if let Some(s) = self.get_sprite(sprite.id()){
            s.kill();
        }
    }
}