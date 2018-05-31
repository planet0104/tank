use canvas::Canvas;
//精灵代码
pub type SPRITEACTION = u32;
pub const SA_NONE: SPRITEACTION = 0;
pub const SA_KILL: SPRITEACTION = 1;
pub const SA_ADDSPRITE: SPRITEACTION = 2;

pub type BOUNDSACTION = u32;
pub const BA_STOP: BOUNDSACTION = 0;
pub const BA_WRAP: BOUNDSACTION = 1;
pub const BA_BOUNCE: BOUNDSACTION = 2;
pub const BA_DIE: BOUNDSACTION = 3;
use animation::Animation;

#[derive(Clone, Debug, Copy)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Rect {
    pub fn new(left: f64, top: f64, right: f64, bottom: f64) -> Rect {
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn zero() -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }

    /** 修改rect大小 */
    pub fn inflate(&mut self, dx: f64, dy: f64) {
        self.left -= dx;
        self.right += dx;
        self.top -= dy;
        self.bottom += dy;
    }

    pub fn offset(&mut self, dx: f64, dy: f64) {
        self.left += dx;
        self.right += dx;
        self.top += dy;
        self.bottom += dy;
    }

    pub fn contain(&self, x: f64, y: f64) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

#[derive(Clone, Debug, Copy)]
pub struct PointF {
    pub x: f64,
    pub y: f64,
}

impl PointF {
    pub fn new(x: f64, y: f64) -> PointF {
        PointF { x: x, y: y }
    }

    pub fn zero() -> PointF {
        PointF { x: 0.0, y: 0.0 }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new() -> Point {
        Point { x: 0, y: 0 }
    }
}

pub trait Sprite {
    fn draw(&self, context: &Canvas) {
        self.get_entity().draw(context);
    }
    fn z_order(&self) -> i32 {
        self.get_entity().z_order
    }
    fn position(&self) -> &Rect {
        &self.get_entity().position
    }
    fn left(&self) -> f64 {
        self.get_entity().position.left
    }
    fn top(&self) -> f64 {
        self.get_entity().position.top
    }
    fn update(&mut self, elapsed_milis: f64) -> SPRITEACTION {
        self.get_entity_mut().update(elapsed_milis)
    }
    fn id(&self) -> u32 {
        self.get_entity().id
    }
    fn add_followed_animation(&mut self, animation:Animation){
        self.get_entity_mut().add_followed_animation(animation);
    }
    fn set_position(&mut self, position: Rect) {
        self.get_entity_mut().position = position;
    }
    fn set_position_point(&mut self, x: f64, y: f64) {
        self.get_entity_mut().set_position(x, y);
    }
    //碰撞检测
    fn test_collison(&self, test: &Rect) -> bool {
        self.get_entity().test_collison(test)
    }
    fn kill(&mut self) {
        self.get_entity_mut().dying = true;
    }
    fn parent(&self) -> u32 {
        self.get_entity().parent
    }
    fn set_parent(&mut self, parent: u32) {
        self.get_entity_mut().parent = parent;
    }
    fn killer(&self) -> u32 {
        self.get_entity().killer
    }
    fn killer_name(&self) -> &String {
        &self.get_entity().killer_name
    }
    fn set_killer_name(&mut self, killer_name: String) {
        self.get_entity_mut().killer_name = killer_name;
    }
    fn name(&self) -> &String {
        &self.get_entity().name
    }
    fn set_name(&mut self, name: String) {
        self.get_entity_mut().name = name;
    }
    fn set_velocity(&mut self, x: f64, y: f64) {
        self.get_entity_mut().set_velocity(x, y);
    }
    fn add_score(&mut self) {
        self.get_entity_mut().score += 1;
    }
    fn set_lives(&mut self, lives: u32) {
        self.get_entity_mut().lives = lives;
    }
    fn lives(&self) -> u32 {
        self.get_entity().lives
    }
    fn set_score(&mut self, score: i32) {
        self.get_entity_mut().score = score;
    }
    fn score(&self) -> i32 {
        self.get_entity().score
    }
    fn set_killer(&mut self, killer: u32, killer_name: String) {
        self.get_entity_mut().set_killer(killer, killer_name);
    }
    fn cur_animation_index(&self) -> &[usize] {
        &self.get_entity().cur_animation
    }
    fn get_animation(&self, anim: usize) -> &Animation {
        &self.get_entity().animations[anim]
    }
    fn get_animation_mut(&mut self, anim: usize) -> &mut Animation{
        &mut self.get_entity_mut().animations[anim]
    }

    fn resotre_last_animation(&mut self) {
        self.get_entity_mut().resotre_last_animation();
    }

    fn set_cur_animation(&mut self, cur_animation: &[usize]) -> bool {
        self.get_entity_mut().set_cur_animation(cur_animation)
    }
    fn velocity(&self) -> &PointF {
        &self.get_entity().velocity
    }
    fn set_target_position(&mut self, target: PointF) {
        self.get_entity_mut().target_position = Some(target);
    }
    fn class(&self) -> i32;
    fn get_entity(&self) -> &Entity;
    fn get_entity_mut(&mut self) -> &mut Entity;
}

pub struct Entity {
    pub id: u32,
    pub parent: u32,
    pub animations: Vec<Animation>,
    pub cur_animation: Vec<usize>,
    pub last_animation: Vec<usize>,
    pub position: Rect,
    pub target_position: Option<PointF>,
    pub bounds: Rect,
    pub velocity: PointF,
    pub z_order: i32,
    pub collision: Rect,
    pub bounds_action: BOUNDSACTION,
    pub hidden: bool,
    pub dying: bool,
    pub one_cycle: bool,
    pub name: String,
    pub score: i32,
    pub killer: u32,
    pub killer_name: String,
    pub lives: u32,
    pub rotation: f64,
    pub followed_animations: Vec<Animation>,
}

impl Entity {
    pub fn new(
        id: u32,
        animations: Vec<Animation>,
        position: PointF,
        width: f64,
        height: f64,
        bounds: Rect,
        bounds_action: BOUNDSACTION,
        one_cycle: bool,
    ) -> Entity {
        let mut sprite = Entity {
            id: id,
            animations,
            followed_animations: vec![],
            cur_animation: vec![],
            last_animation: vec![],
            parent: 0,
            position: Rect::new(
                position.x,
                position.y,
                position.x + width,
                position.y + height,
            ),
            target_position: None,
            velocity: PointF::zero(),
            z_order: 0,
            bounds: bounds,
            bounds_action: bounds_action,
            hidden: false,
            dying: false,
            one_cycle: one_cycle,
            name: "".to_string(),
            collision: Rect::zero(),
            score: 0,
            killer: 0,
            killer_name: String::new(),
            lives: 0,
            rotation: 0.0,
        };
        sprite.calc_collision_rect();
        sprite
    }

    // pub fn from_bitmap(id: u32, bitmap: Box<Bitmap>, bounds: Rect) -> Entity {
    //     Entity::new(
    //         id,
    //         bitmap,
    //         PointF::zero(),
    //         PointF::zero(),
    //         0,
    //         bounds,
    //         BA_STOP,
    //     )
    // }

    // pub fn with_bounds_action(
    //     id: u32,
    //     bitmap: Box<Bitmap>,
    //     position: PointF,
    //     bounds: Rect,
    //     bounds_action: BOUNDSACTION,
    // ) -> Entity {
    //     Entity::new(
    //         id,
    //         bitmap,
    //         position,
    //         PointF::zero(),
    //         0,
    //         bounds,
    //         bounds_action,
    //     )
    // }

    // pub fn with_bounds_action_norand(
    //     id: u32,
    //     bitmap: BitmapRes,
    //     bounds: Rect,
    //     bounds_action: BOUNDSACTION,
    // ) -> Sprite {
    //     Sprite::new(
    //         id,
    //         bitmap,
    //         PointF::new(),
    //         PointF::new(),
    //         0,
    //         bounds,
    //         bounds_action,
    //     )
    // }

    fn calc_collision_rect(&mut self) {
        let x_shrink = (self.position.left - self.position.right) / 12.0;
        let y_shrink = (self.position.top - self.position.bottom) / 12.0;
        self.collision = self.position;
        self.collision.inflate(x_shrink, y_shrink);
    }

    //-----------------------------------------------------------------
    // Sprite General Methods
    //-----------------------------------------------------------------
    pub fn update(&mut self, elapsed_milis: f64) -> SPRITEACTION {
        // See if the sprite needs to be killed
        if self.dying {
            return SA_KILL;
        }

        // Update the animation
        for anim in &self.cur_animation {
            self.animations[*anim].update(elapsed_milis);
            //执行一遍的动画结束后杀死精灵
            if self.one_cycle && self.animations[*anim].end() {
                self.dying = true;
            }
        }
        for anim in &mut self.followed_animations{
            anim.update(elapsed_milis);
        }

        //检查是否到达目标位置
        if let Some(target) = self.target_position {
            if self.velocity.x == 0.0 && self.velocity.y == 0.0 {
                if target.x != self.position.left || target.y != self.position.top {
                    self.set_position_point(&PointF {
                        x: target.x,
                        y: target.y,
                    });
                }
            }
        }

        // if let Some((target, velocity)) = self.target{
        //     let mut tmp_position = PointF{
        //         x: self.position.left,
        //         y: self.position.top,
        //     };
        //     self.velocity.x = velocity.x;
        //     self.velocity.y = velocity.y;
        //     if self.velocity.x != 0.0 && self.velocity.y != 0.0{
        //         self.last_velocity  = Some(velocity);
        //     }
        //     //由于每次绘制已经过去几十ms, 精灵有可能越过目标点, 所以这里进一步计算
        //     let mut distance = 0.0;
        //     for _ in 0..elapsed_milis as u32{
        //         tmp_position.x += self.velocity.x;
        //         tmp_position.y += self.velocity.y;
        //         let (dx, dy) = (target.x - tmp_position.x, target.y - tmp_position.y);
        //         distance =  (dx * dx + dy * dy).sqrt();
        //         //达到目标点(这里的1.0是假设游戏中最快的精灵速度不超过1.0)
        //         if distance.abs()<1.0{
        //             self.velocity.x = 0.0;
        //             self.velocity.y = 0.0;
        //             break;
        //         }else if distance.abs()>100.0{
        //             //正常情况下延迟不会导致距离差距到100
        //             //精灵穿越墙的时候，会导致服务器和客户端距离为整个屏幕的宽度或者高度，这时候不进行移动，直接跳过去
        //             self.velocity.x = 0.0;
        //             self.velocity.y = 0.0;
        //             self.set_position_point(&PointF{
        //                 x: target.x,
        //                 y: target.y,
        //             });
        //             break;
        //         }
        //     }
        //     //如果距离仍然很大，但是速度为零，这时候也直接将精灵移动过去
        //     if velocity.x == 0.0 && velocity.y == 0.0 && distance>1.0{
        //         if self.last_velocity.is_none(){
        //             self.set_position_point(&PointF{
        //                 x: target.x,
        //                 y: target.y,
        //             });
        //         }else{
        //             //如果存在上次移动的速度，按照最后一次速度移动
        //             self.velocity = self.last_velocity.unwrap();
        //         }
        //     }
        // }

        //Update the position
        let mut new_position = PointF::zero();
        let mut sprite_size = PointF::zero();
        let mut bounds_size = PointF::zero();

        new_position.x = self.position.left + self.velocity.x * elapsed_milis;
        new_position.y = self.position.top + self.velocity.y * elapsed_milis;
        sprite_size.x = self.position.right - self.position.left;
        sprite_size.y = self.position.bottom - self.position.top;
        bounds_size.x = self.bounds.right - self.bounds.left;
        bounds_size.y = self.bounds.bottom - self.bounds.top;

        // Check the bounds
        // Wrap?
        if self.bounds_action == BA_WRAP {
            if (new_position.x + sprite_size.x) < self.bounds.left {
                new_position.x = self.bounds.right;
            } else if new_position.x > self.bounds.right {
                new_position.x = self.bounds.left - sprite_size.x;
            }
            if (new_position.y + sprite_size.y) < self.bounds.top {
                new_position.y = self.bounds.bottom;
            } else if new_position.y > self.bounds.bottom {
                new_position.y = self.bounds.top - sprite_size.y;
            }
        }
        // Bounce?
        else if self.bounds_action == BA_BOUNCE {
            let mut bounce = false;
            let mut new_velocity = self.velocity;
            if new_position.x < self.bounds.left {
                bounce = true;
                new_position.x = self.bounds.left;
                new_velocity.x = -new_velocity.x;
            } else if (new_position.x + sprite_size.x) > self.bounds.right {
                bounce = true;
                new_position.x = self.bounds.right - sprite_size.x;
                new_velocity.x = -new_velocity.x;
            }
            if new_position.y < self.bounds.top {
                bounce = true;
                new_position.y = self.bounds.top;
                new_velocity.y = -new_velocity.y;
            } else if (new_position.y + sprite_size.y) > self.bounds.bottom {
                bounce = true;
                new_position.y = self.bounds.bottom - sprite_size.y;
                new_velocity.y = -new_velocity.y;
            }
            if bounce {
                self.velocity = new_velocity;
            }
        }
        // Die?
        else if self.bounds_action == BA_DIE {
            if (new_position.x + sprite_size.x) < self.bounds.left
                || new_position.x > self.bounds.right
                || (new_position.y + sprite_size.y) < self.bounds.top
                || new_position.y > self.bounds.bottom
            {
                return SA_KILL;
            }
        }
        // Stop (default)
        else {
            if new_position.x < self.bounds.left
                || new_position.x > (self.bounds.right - sprite_size.x)
            {
                new_position.x = f64::max(
                    self.bounds.left,
                    f64::min(new_position.x, self.bounds.right - sprite_size.x),
                );
                self.set_velocity(0.0, 0.0);
            }
            if new_position.y < self.bounds.top
                || new_position.y > (self.bounds.bottom - sprite_size.y)
            {
                new_position.y = f64::max(
                    self.bounds.top,
                    f64::min(new_position.y, self.bounds.bottom - sprite_size.y),
                );
                self.set_velocity(0.0, 0.0);
            }
        }
        self.set_position_point(&new_position);

        //let msg = format!("after update>position={:?}", self.position());
        //unsafe { log(msg.as_ptr(), msg.len()); }
        SA_NONE
    }

    pub fn draw(&self, context: &Canvas) {
        // Draw the sprite if it isn't hidden
        if !self.hidden {
            // Draw the appropriate frame, if necessary
            for anim in &self.followed_animations{
                anim.draw(self.position.left as i32, self.position.top as i32, context);
            }

            for anim in &self.cur_animation {
                self.animations[*anim].draw(
                    self.position.left as i32,
                    self.position.top as i32,
                    context,
                );
            }
            context.fill_style("#ccccff");
            context.set_font("16px 微软雅黑");
            if self.name.len() > 0 && self.score >= 0 {
                let score = &format!("({}分)", self.score);
                let w = self.name.len() * 5 + score.len() * 5;
                let x = self.position.left as i32
                    + ((self.position.right - self.position.left) as i32 / 2 - (w as i32 / 2));
                let y = self.position.bottom as i32 + 20;
                context.fill_text(&format!("{}{}", self.name, score), x, y);
            }
            //绘制坦克生命值
            let mut lives = String::new();
            for _ in 0..self.lives {
                //lives.push_str("❤️");
                lives.push_str("♡");
            }
            context.fill_style(if self.lives > 3 { "#ffff00" } else { "#ff0000" });
            context.fill_text(
                &lives,
                self.position.left as i32,
                self.position.bottom as i32 + 40,
            );
        }
    }

    pub fn set_velocity(&mut self, x: f64, y: f64) {
        self.velocity.x = x;
        self.velocity.y = y;
    }

    pub fn set_position_point(&mut self, position: &PointF) {
        let dx = position.x - self.position.left;
        let dy = position.y - self.position.top;
        self.position.offset(dx, dy);
        self.calc_collision_rect();
    }

    pub fn set_position(&mut self, x: f64, y: f64) {
        let x = x - self.position.left;
        let y = y - self.position.top;
        self.position.offset(x, y);
        self.calc_collision_rect();
    }

    pub fn test_collison(&self, test: &Rect) -> bool {
        self.collision.left <= test.right && test.left <= self.collision.right
            && self.collision.top <= test.bottom && test.top <= self.collision.bottom
    }

    pub fn is_point_inside(&self, x: f64, y: f64) -> bool {
        self.position.contain(x, y)
    }

    pub fn set_cur_animation(&mut self, cur_animation: &[usize]) -> bool {
        if self.cur_animation != cur_animation {
            self.last_animation.clear();
            self.last_animation.append(&mut self.cur_animation);
            self.cur_animation.clear();
            self.cur_animation.append(&mut cur_animation.to_vec());
            true
        } else {
            false
        }
        //更新位置?
    }

    // pub fn set_num_frames(&mut self, num_frames: i32, one_cycle: bool) {
    //     self.num_frames = num_frames;
    //     self.one_cycle = one_cycle;

    //     //重新计算位置
    //     self.position.bottom =
    //         self.position.top + (self.position.bottom - self.position.top) / self.num_frames as f64;
    // }

    pub fn set_killer(&mut self, killer: u32, killer_name: String) {
        self.killer = killer;
        self.killer_name = killer_name;
    }

    pub fn resotre_last_animation(&mut self) {
        self.cur_animation.clear();
        self.cur_animation.append(&mut self.last_animation);
    }

    pub fn add_followed_animation(&mut self, animation: Animation){
        self.followed_animations.push(animation);
    }
}
