use engine::Bitmap;
use engine::sprite::{Entity, PointF, Rect, Sprite, BA_STOP};
use engine::animation::Animation;
use stdweb::web::html_element::ImageElement;
use std::rc::Rc;
use std::cell::RefCell;
use {WIDTH, HEIGHT};

pub struct Image {
    pub image: ImageElement,
    pub url: String,
}

impl Bitmap for Image {
    fn width(&self) -> i32 {
        self.image.width() as i32
    }
    fn height(&self) -> i32 {
        self.image.height() as i32
    }
    fn id(&self) -> u8 {
        0
    }
    fn url(&self) -> &str {
        &self.url
    }
}

impl Clone for Image {
    fn clone(&self) -> Image {
        Image {
            image: self.image.clone(),
            url: self.url.clone(),
        }
    }
}

pub struct PersonSprite {
    entity: Entity,
    front: i32, //面向 0左, 1右
}

impl PersonSprite {
    pub fn new(bitmap: Rc<RefCell<Bitmap>>) -> PersonSprite {
        let mut anim_idle = Animation::infinite(bitmap.clone(), 0, 0, 167, 220, 30, 2500); //0.站立
        anim_idle.set_flip(true, false);
        let anim_walk = Animation::infinite(bitmap.clone(), 0, 220, 167, 220, 18, 600); //1.走路
        let anim_crouch = Animation::on_cycle(bitmap.clone(), 0, 440, 167, 220, 6, 80); //2.蹲下
        let anim_standup = Animation::on_cycle(bitmap.clone(), 0, 660, 167, 220, 6, 80);//3.起立
        let mut anim_throw = Animation::on_cycle(bitmap, 0, 880, 167, 220, 30, 600); //4.投掷
        anim_throw.set_translate(30.0, 16.0);

        let entity = Entity::new(
            0,
            vec![
                anim_idle,
                anim_walk,
                anim_crouch,
                anim_standup,
                anim_throw
            ],
            PointF::zero(),
            Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
            BA_STOP,
            false
        );
        PersonSprite { entity, front: 1 }
    }

    //站立
    pub fn idle(&mut self){
        //下蹲的时候不能起立
        if self.cur_animation_index() != 2{
            self.set_cur_animation(0);
        }
        let flip = self.is_flip();
        self.cur_animation().set_flip(flip, false);
    }

    //走路 flip:是否横向反转(反转为向右, 不反转向左)
    pub fn walk(&mut self) {
        //下蹲的时候不能跑
        if self.cur_animation_index() != 2 && self.cur_animation_index() != 3{
            self.set_cur_animation(1);
            let flip = self.is_flip();
            self.cur_animation().set_flip(flip, false);
        }
    }

    fn is_flip(&self) ->bool{
        if self.front == 0{ false}else{true}
    }

    //投雪球
    fn throw(&mut self, front: i32){
        //蹲下的时候不能投掷
        if self.cur_animation_index() != 2 && self.cur_animation_index()!=4{
            self.front = front;
            //投掷
            if self.set_cur_animation(4){
                self.cur_animation().init();
            }
            let flip = self.is_flip();
            self.cur_animation().set_flip(flip, false);
        }
    }

    pub fn throw_left(&mut self){
        self.throw(0);
    }

    pub fn throw_right(&mut self){
        self.throw(1);
    }

    //起立
    pub fn standup(&mut self){
        if self.set_cur_animation(3){
            //如果上一个动画不是此动画, 初始化第一帧
            self.cur_animation().init();
        }
        let flip = self.is_flip();
        self.cur_animation().set_flip(flip, false);
    }

    //下蹲
    pub fn crouch(&mut self){
        if self.set_cur_animation(2){
            //如果上一个动画不是此动画, 初始化第一帧
            self.cur_animation().init();
        }
        let flip = self.is_flip();
        self.cur_animation().set_flip(flip, false);
    }
}

impl Sprite for PersonSprite {
    fn update(&mut self, elapsed_milis:f64) -> u32{
        let action = self.entity.update(elapsed_milis);
        //standup/throw 动画结束, 切换到 idle
        if (self.cur_animation_index() == 3 || self.cur_animation_index()==4)
            && self.cur_animation().end(){
            self.idle();
        }
        action
    }

    fn class(&self) -> i32 {
        0
    }
    fn get_entity(&self) -> &Entity {
        &self.entity
    }
    fn get_entity_mut(&mut self) -> &mut Entity {
        &mut self.entity
    }
}
