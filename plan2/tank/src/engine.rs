use sprite::{Sprite, SA_KILL};

//GameEngine 绘制和更新精灵
pub trait CanvasContext{
    fn draw_image_at(&self, res_id:i32, x:i32, y:i32);
    fn draw_image(&self, res_id:i32, source_x:i32, source_y:i32, source_width:i32, source_height:i32, dest_x:i32, dest_y:i32, dest_width:i32, dest_height:i32);
    fn fill_style(&self, style: &str);
    fn fill_rect(&self, x:i32, y:i32, width:i32, height:i32);
    fn fill_text(&self, text: &str, x:i32, y:i32);
}

pub struct GameEngine{
    sprites: Vec<Sprite>
}

impl GameEngine{
    pub fn new()->GameEngine{
        GameEngine{
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

    pub fn draw_sprites(&self, context: &CanvasContext){
        //绘制所有的精灵
        for sprite in &self.sprites{
            sprite.draw(context);
        }
    }

    pub fn update_sprites<D: Fn(&mut GameEngine, usize), C: Fn(&mut GameEngine, usize, usize)->bool>(&mut self, sprite_dying: D, sprite_collision: C){
        //log_string(format!("sprites={}", self.sprites.len()).as_str().as_bytes());
        //更新所有精灵
        let mut sprites_to_kill:Vec<String> = vec![];
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
                sprite_dying(self, i);
                //杀死精灵
                sprites_to_kill.push(self.sprites[i].id.clone());
                continue;
            }

            if self.check_sprite_collision(i, &sprite_collision){
                self.sprites[i].set_position_rect(old_sprite_pos);
            }
        }

        //删除死亡的精灵
        for sprite_id in sprites_to_kill{
            self.sprites.retain(|ref s|{
                s.id != sprite_id
            });
        }
    }

    pub fn check_sprite_collision<C: Fn(&mut GameEngine, usize, usize)->bool>(&mut self, test_sprite_id:usize, sprite_collision: &C)->bool{
        //检查精灵是否和其他精灵相撞
        //let test_sprite = &self.sprites[test_sprite_id];
        for i in 0..self.sprites.len(){
            //不检查精灵自己
            if i == test_sprite_id{
                continue;
            }
            if self.sprites[test_sprite_id].test_collison(self.sprites[i].position()){
                return sprite_collision(self, i, test_sprite_id);
            }
        }
        return false;
    }

    pub fn clean_up_sprites(&mut self){
        self.sprites.clear();
    }

    pub fn query_sprite(&mut self, id: &String)->Option<&mut Sprite>{
        for sprite in &mut self.sprites{
            if sprite.id == id.as_ref(){
                return Some(sprite);
            }
        }
        None
    }

    pub fn sprites(&self)->&Vec<Sprite>{
        &self.sprites
    }

    // pub fn kill_sprite(&mut self, sprite:&Sprite){
    //     if let Some(s) = self.query_sprite(&sprite.id){
    //         s.kill();
    //     }
    // }

    pub fn kill_sprite(&mut self, idx: usize){
        self.sprites[idx].kill();
    }
}