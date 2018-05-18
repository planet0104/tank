use canvas::Canvas;
use sprite::{Sprite, SA_KILL};
use std::cell::RefCell;
use std::rc::Rc;
use utils::Counter;
//GameEngine 绘制和更新精灵

pub trait UpdateCallback {
    fn on_sprite_dying(&mut self, engine: &mut GameEngine, idx_sprite_dying: usize);
    fn on_sprite_collision(
        &mut self,
        engine: &mut GameEngine,
        idx_sprite_hitter: usize,
        idx_sprite_hittee: usize,
    ) -> bool;
}

pub struct GameEngine {
    sprites: Vec<Box<Sprite>>,
    counter: Counter,
}

impl GameEngine {
    pub fn new() -> GameEngine {
        GameEngine {
            sprites: vec![],
            counter: Counter::new(),
        }
    }

    pub fn add_sprite(&mut self, sprite: Box<Sprite>) -> usize {
        if self.sprites.len() > 0 {
            for i in 0..self.sprites.len() {
                //根据z-order插入精灵到数组
                if sprite.z_order() < self.sprites[i].z_order() {
                    self.sprites.insert(i, sprite);
                    return i;
                }
            }
        }
        //精灵的zOrder是最高的，放入Vec的末尾
        self.sprites.push(sprite);
        self.sprites.len() - 1
    }

    pub fn draw_sprites(&self, context: &Canvas) {
        //绘制所有的精灵
        for sprite in &self.sprites {
            sprite.draw(context);
        }
    }

    pub fn update_sprites<C: UpdateCallback>(
        &mut self,
        elapsed_milis: f64,
        callback: Rc<RefCell<C>>,
    ) {
        //log_string(format!("sprites={}", self.sprites.len()).as_str().as_bytes());
        //更新所有精灵
        let mut sprites_to_kill: Vec<u32> = vec![];
        for i in 0..self.sprites.len() {
            //保存旧的精灵位置以防需要恢复
            let old_sprite_pos = *self.sprites[i].position();
            //更新精灵
            let sprite_action = self.sprites[i].update(elapsed_milis);

            //处理SA_ADDSPRITE
            // if sprite_action == SA_ADDSPRITE{
            //     //允许精灵添加它的精灵
            //     if let Some(sprite) = self.sprites[i].add_sprite(){
            //         self.add_sprite(sprite);
            //     }
            // }

            //处理 SA_KILL
            if sprite_action == SA_KILL {
                //通知游戏精灵死亡
                callback.borrow_mut().on_sprite_dying(self, i);
                //杀死精灵
                sprites_to_kill.push(self.sprites[i].id());
                continue;
            }

            if self.check_sprite_collision(i, &callback) {
                self.sprites[i].set_position_rect(old_sprite_pos);
            }
        }

        //删除死亡的精灵
        for sprite_id in sprites_to_kill {
            self.sprites.retain(|ref s| s.id() != sprite_id);
        }
    }

    pub fn check_sprite_collision<C: UpdateCallback>(
        &mut self,
        test_sprite_id: usize,
        callback: &Rc<RefCell<C>>,
    ) -> bool {
        //检查精灵是否和其他精灵相撞
        //let test_sprite = &self.sprites[test_sprite_id];
        for i in 0..self.sprites.len() {
            //不检查精灵自己
            if i == test_sprite_id {
                continue;
            }
            if self.sprites[test_sprite_id].test_collison(self.sprites[i].position()) {
                return callback
                    .borrow_mut()
                    .on_sprite_collision(self, i, test_sprite_id);
            }
        }
        return false;
    }

    pub fn clean_up_sprites(&mut self) {
        self.sprites.clear();
    }

    pub fn query_sprite(&mut self, id: u32) -> Option<&mut Box<Sprite>> {
        for sprite in &mut self.sprites {
            if sprite.id() == id {
                return Some(sprite);
            }
        }
        None
    }

    pub fn query_sprite_idx(&self, id: u32) -> Option<usize> {
        for i in 0..self.sprites.len() {
            if self.sprites[i].id() == id {
                return Some(i);
            }
        }
        None
    }

    // pub fn sprites(&self)->&Vec<Sprite>{
    //     &self.sprites
    // }

    pub fn sprites(&mut self) -> &mut Vec<Box<Sprite>> {
        &mut self.sprites
    }

    // pub fn kill_sprite(&mut self, sprite:&Sprite){
    //     if let Some(s) = self.query_sprite(&sprite.id){
    //         s.kill();
    //     }
    // }

    pub fn next_sprite_id(&mut self) -> u32 {
        self.counter.next().unwrap()
    }

    pub fn kill_sprite(&mut self, idx: usize) {
        self.sprites[idx].kill();
    }
}
