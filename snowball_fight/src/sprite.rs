use engine::Bitmap;
use engine::animation::Animation;
use engine::canvas::Canvas;
use engine::sprite::{Entity, PointF, Rect, Sprite, BA_STOP};
use std::cell::RefCell;
use std::rc::Rc;
use stdweb::web::html_element::ImageElement;
use {HEIGHT, WIDTH};
use std::f64::consts::PI;

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

const ANIM_IDLE: usize = 0;
const ANIM_WALK: usize = 1;
const ANIM_CROUCH: usize = 2;
const ANIM_STANDUP: usize = 3;
const ANIM_THROW_LEFT: usize = 4;
const ANIM_THROW_RIGHT: usize = 5;
const ANIM_HIT: usize = 6;
const ANIM_JUMP: usize = 7;
pub struct PersonSprite {
    bitmap: Rc<RefCell<Bitmap>>,
    entity: Entity,
    front: i32, //面向 0左, 1右
}

impl PersonSprite {
    pub fn new(bitmap: Rc<RefCell<Bitmap>>) -> PersonSprite {
        let mut anim_idle = Animation::infinite(bitmap.clone(), 0, 0, 100, 175, 30, 2500); //0.站立
        anim_idle.set_flip(true, false);
        let anim_walk = Animation::infinite(bitmap.clone(), 0, 175, 112, 175, 18, 600); //1.走路
        let anim_crouch = Animation::on_cycle(bitmap.clone(), 0, 175 * 2, 100, 175, 6, 80); //2.蹲下
        let anim_standup = Animation::on_cycle(bitmap.clone(), 0, 175 * 3, 100, 175, 6, 80); //3.起立
        let mut anim_throw_left = Animation::on_cycle(bitmap.clone(), 0, 175 * 4, 167, 175, 30, 600); //4.左投掷
        anim_throw_left.set_translate(-20.0, 0.0);
        let mut anim_throw_right = Animation::on_cycle(bitmap.clone(), 0, 175 * 5, 167, 175, 30, 600); //5.右投掷
        anim_throw_right.set_translate(-50.0, 0.0);
        let anim_hit = Animation::on_cycle(bitmap.clone(), 0, 175 * 6, 100, 175, 12, 300); //6.被击中
        let mut anim_jump = Animation::on_cycle(bitmap.clone(), 0, 175 * 7, 100, 189, 20, 760); //7.跳跃
        anim_jump.set_translate(-8.0, 0.0);

        //！！！ faceprint/dying 动画都可以作为人物的Animation，通过偏移可以方便设置位置 !!!
        
        //分离的动画帧:  跳跃腿、跳跃身体、投掷腿(两个方向)、投掷身体(两个方向)

        //动画组合
        // 跳跃动画: 跳腿+跳身
        // 跳跃时投掷: 跳腿+投身
        // (idle时投掷 或 walk时投掷): 投腿+投身
        // standup、hit、crouch(原游戏下蹲有单独投掷动画)时，不允许投掷

        let mut entity = Entity::new(
            0,
            vec![
                anim_idle,
                anim_walk,
                anim_crouch,
                anim_standup,
                anim_throw_left,
                anim_throw_right,
                anim_hit,
                anim_jump,
            ],
            PointF::zero(),
            112.0,
            175.0,
            Rect::new(0.0, 0.0, WIDTH as f64, HEIGHT as f64),
            BA_STOP,
            false,
        );
        entity.set_cur_animation(&[ANIM_IDLE]);
        PersonSprite { entity, front: 1, bitmap: bitmap }
    }

    //站立
    pub fn idle(&mut self) {
        //下蹲的时候不能起立
        if self.cur_animation_index() != &[ANIM_CROUCH] {
            self.set_cur_animation(&[ANIM_IDLE]);
        }
        self.flip_animation();
    }

    //走路 flip:是否横向反转(反转为向右, 不反转向左)
    fn walk(&mut self) {
        //下蹲的时候不能跑
        if self.cur_animation_index() != &[ANIM_CROUCH] && self.cur_animation_index() != &[ANIM_STANDUP] {
            self.set_cur_animation(&[ANIM_WALK]);
            self.flip_animation();
        }
    }

    pub fn walk_left(&mut self){
        self.front = 0;
        self.walk();
    }

    pub fn walk_right(&mut self){
        self.front = 1;
        self.walk();
    }

    pub fn walk_up(&mut self){
        self.walk();
    }

    pub fn walk_down(&mut self){
        self.walk();
    }

    //根据方向反转动画
    fn flip_animation(&mut self){
        let flip = if self.front == 0 {
            false
        } else {
            true
        };
        let anims:Vec<usize> = self.cur_animation_index().to_vec();
        for i in anims{
            self.get_animation_mut(i).set_flip(flip, false);
        }
    }

    //投雪球
    fn throw(&mut self, front: i32) {
        //蹲下的时候不能投掷
        if self.cur_animation_index() != &[ANIM_CROUCH] && self.cur_animation_index() != &[ANIM_THROW_LEFT]
            && self.cur_animation_index()!=&[ANIM_THROW_RIGHT]
        {
            self.front = front;
            let anim = if front==0{ANIM_THROW_LEFT}else{ANIM_THROW_RIGHT};
            //投掷
            if self.set_cur_animation(&[anim]) {
                self.get_animation_mut(anim).init();
            }
            //throw不需要反转
        }
    }

    pub fn throw_left(&mut self) {
        self.throw(0);
    }

    pub fn throw_right(&mut self) {
        self.throw(1);
    }

    //起立
    pub fn standup(&mut self) {
        if self.cur_animation_index() != &[ANIM_JUMP] {
            if self.set_cur_animation(&[ANIM_STANDUP]) {
                //如果上一个动画不是此动画, 初始化第一帧
                self.get_animation_mut(ANIM_STANDUP).init();
            }
            self.flip_animation();
        }
    }

    //下蹲
    pub fn crouch(&mut self) {
        if self.cur_animation_index() != &[ANIM_JUMP] {
            if self.set_cur_animation(&[ANIM_CROUCH]) {
                //如果上一个动画不是此动画, 初始化第一帧
                self.get_animation_mut(ANIM_CROUCH).init();
            }
            self.flip_animation();
        }
    }

    //被击中
    pub fn hit(&mut self) {
        //只有idle的时候，执行被击中动画
        if self.cur_animation_index() == &[ANIM_IDLE] {
            self.set_cur_animation(&[ANIM_HIT]);
            self.get_animation_mut(ANIM_HIT).init();
            self.flip_animation();
        }
    }

    pub fn jump(&mut self) {
        if self.set_cur_animation(&[ANIM_JUMP]) {
            self.get_animation_mut(ANIM_JUMP).init();
            self.flip_animation();
        }
    }
}

impl Sprite for PersonSprite {
    fn update(&mut self, elapsed_milis: f64) -> u32 {
        let action = self.entity.update(elapsed_milis);
        //standup动画结束以后切换到IDLE
        if self.cur_animation_index() == &[ANIM_STANDUP] && self.get_animation(ANIM_STANDUP).end(){
            self.set_cur_animation(&[ANIM_IDLE]);
            self.flip_animation();
        }
        //throw/hit 动画结束, 切换回上次动画
        if self.cur_animation_index() == &[ANIM_THROW_LEFT] || self.cur_animation_index() == &[ANIM_THROW_RIGHT]
            || self.cur_animation_index() == &[ANIM_HIT]
        {
            let anim = self.cur_animation_index()[0];
            if self.get_animation(anim).end(){
                self.resotre_last_animation();
                self.flip_animation();
            }
        }
        //jump结束恢复上次动画
        if self.cur_animation_index() == &[ANIM_JUMP] {
            let (x, y) = {
                //记录起跳位置
                if self.get_animation(ANIM_JUMP).cur_frame() == 0{
                    let x = self.left();
                    let y = self.top();
                    self.get_animation_mut(ANIM_JUMP).set_tag_point(x, y);
                    self.get_animation_mut(ANIM_JUMP).set_tag(0.0);
                }
                //起跳以后按照 加速-减速-加速调整精灵位置
                let cur_animation = self.get_animation_mut(ANIM_JUMP);
                let angle_step = PI/cur_animation.frame_count() as f64;
                //每一帧变化以后增加角度
                if cur_animation.check_frame(){
                    cur_animation.add_tag(angle_step);
                }
                let mut dy = cur_animation.get_tag().sin()*100.0;
                
                //判断动画是否结束,结束以后归位
                if (cur_animation.get_tag()-PI).abs()<=0.0000001{
                    dy = 0.0;
                }
                (cur_animation.get_tag_point().x, cur_animation.get_tag_point().y-dy)
            };
            
            self.set_position_point(x, y);

            if self.get_animation(ANIM_JUMP).end() {
                self.resotre_last_animation();
            }
        }
        action
    }

    fn draw(&self, context: &Canvas) {

        let pos = if self.cur_animation_index() == &[ANIM_JUMP]{
            let pos = self.get_animation(ANIM_JUMP).get_tag_point();
            (pos.x, pos.y)
        }else{
            (self.left(), self.top())
        };

        //绘制影子
        context.draw_image(
            &*self.bitmap.borrow(),
            3360, 0,
            70,
            39,
            pos.0 as i32 + 15,
            pos.1 as i32 + 144,
            70,
            39,
        );
        self.entity.draw(context);
        
        //绘制rect
        let pos = self.position();
        context.stroke_style("#f00");
        context.stroke_rect(
            pos.left,
            pos.top,
            pos.right - pos.left,
            pos.bottom - pos.top,
        );
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
