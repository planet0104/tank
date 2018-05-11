use sprite::{Sprite, SA_KILL};
use std::rc::Rc;
use std::cell::RefCell;
use KeyEvent;
use utils::Counter;
//GameEngine 绘制和更新精灵
pub trait GameContext {
    fn draw_image_repeat(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_repeat_x(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_repeat_y(&self, res_id: i32, x: i32, y: i32, width: i32, height: i32);
    fn draw_image_at(&self, res_id: i32, x: i32, y: i32);
    fn draw_image(
        &self,
        res_id: i32,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        dest_x: i32,
        dest_y: i32,
        dest_width: i32,
        dest_height: i32,
    );
    fn line_width(&self, width: i32);
    fn set_canvas_font(&self, font: &str);
    fn fill_style(&self, style: &str);
    fn stroke_style(&self, style: &str);
    fn fill_rect(&self, x: i32, y: i32, width: i32, height: i32);
    fn stroke_rect(&self, x: i32, y: i32, width: i32, height: i32);
    fn fill_text(&self, text: &str, x: i32, y: i32);
    fn console_log(&self, msg: &str);
    fn set_canvas_style_margin(&self, left: i32, top: i32, right: i32, bottom: i32);
    fn set_canvas_style_width(&self, width: i32);
    fn set_canvas_style_height(&self, height: i32);
    fn set_canvas_width(&self, width: i32);
    fn set_canvas_height(&self, height: i32);
    fn alert(&self, msg: &str);
    fn load_resource(&self, json: String);
    fn window_inner_width(&self) -> i32;
    fn window_inner_height(&self) -> i32;
    fn set_on_window_resize_listener(&self, listener: fn());
    fn set_on_connect_listener(&self, listener: fn());
    fn set_on_resource_load_listener(&self, listener: fn(num: i32, total: i32));
    //fn set_on_key_up_listener(&self, listener: fn(key: i32));
    //fn set_on_key_down_listener(&self, listener: fn(key: i32));
    fn send_message(&self, msg: &str);
    fn send_binary_message(&self, msg: &Vec<u8>);
    fn set_on_close_listener(&self, listener: fn());
    fn request_animation_frame(&self);
    fn connect(&self, url: &str);
    fn set_frame_callback(&self, callback: fn(f64));
    //fn set_on_message_listener(&self, callback: fn(&str));
    fn pick_key_events(&self) -> Vec<(KeyEvent, i32)>;
    fn pick_messages(&self) -> Vec<String>;
    fn pick_binary_messages(&self) -> Vec<Vec<u8>>;
    fn current_time_millis(&self) -> f64;
}

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
    sprites: Vec<Sprite>,
    counter: Counter,
}

impl GameEngine {
    pub fn new() -> GameEngine {
        GameEngine {
            sprites: vec![],
            counter: Counter::new(),
        }
    }

    pub fn add_sprite(&mut self, sprite: Sprite) -> usize {
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

    pub fn draw_sprites(&self, context: Rc<Box<GameContext>>) {
        //绘制所有的精灵
        for sprite in &self.sprites {
            sprite.draw(context.clone());
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
                sprites_to_kill.push(self.sprites[i].id.clone());
                continue;
            }

            if self.check_sprite_collision(i, &callback) {
                self.sprites[i].set_position_rect(old_sprite_pos);
            }
        }

        //删除死亡的精灵
        for sprite_id in sprites_to_kill {
            self.sprites.retain(|ref s| s.id != sprite_id);
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

    pub fn query_sprite(&mut self, id: u32) -> Option<&mut Sprite> {
        for sprite in &mut self.sprites {
            if sprite.id == id {
                return Some(sprite);
            }
        }
        None
    }

    pub fn query_sprite_idx(&self, id: u32) -> Option<usize> {
        for i in 0..self.sprites.len() {
            if self.sprites[i].id == id {
                return Some(i);
            }
        }
        None
    }

    // pub fn sprites(&self)->&Vec<Sprite>{
    //     &self.sprites
    // }

    pub fn sprites(&mut self) -> &mut Vec<Sprite> {
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
