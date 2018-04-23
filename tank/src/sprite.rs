use std::cmp;
use engine::GameContext;
//use utils::js_rand_int as rand_int;
use utils::rand_int;
use std::rc::Rc;
//精灵代码

pub type SPRITEACTION = u32;
pub const SA_NONE: SPRITEACTION = 0;
pub const SA_KILL: SPRITEACTION = 1;
//pub const SA_ADDSPRITE:SPRITEACTION = 2;

pub type BOUNDSACTION = u32;
pub const BA_STOP: BOUNDSACTION = 0;
pub const BA_WRAP: BOUNDSACTION = 1;
pub const BA_BOUNCE: BOUNDSACTION = 2;
pub const BA_DIE: BOUNDSACTION = 3;

//导入的JS帮助函数
// use std::ffi::CString;
// use std::os::raw::c_char;
// extern "C" {
//     pub fn emscripten_console_log(text: *const c_char);
// }
// pub fn console_log(msg: &str) {
//     unsafe {
//         emscripten_console_log(CString::new(msg).unwrap().as_ptr());
//     }
// }

pub struct BitmapRes {
    id: i32,
    width: i32,
    height: i32,
}

impl BitmapRes {
    pub fn new(id: i32, width: i32, height: i32) -> BitmapRes {
        BitmapRes {
            id: id,
            width: width,
            height: height,
        }
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
        Rect {left, top, right, bottom}
    }

    pub fn zero() -> Rect {
        Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }
    }

    /** 修改rect大小 */
    pub fn inflate(&mut self, dx: i32, dy: i32) {
        self.left -= dx;
        self.right += dx;
        self.top -= dy;
        self.bottom += dy;
    }

    pub fn offset(&mut self, dx: i32, dy: i32) {
        self.left += dx;
        self.right += dx;
        self.top += dy;
        self.bottom += dy;
    }

    pub fn contain(&self, x: i32, y: i32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point{
    pub fn new() -> Point{
        Point{
            x: 0, y: 0
        }
    }
}

pub struct Sprite {
    pub id: String,
    pub parent: Option<String>,
    bitmap: BitmapRes,
    num_frames: i32,
    cur_frame: i32,
    frame_delay: i32,
    frame_trigger: i32,
    position: Rect,
    bounds: Rect,
    velocity: Point,
    z_order: i32,
    collision: Rect,
    bounds_action: BOUNDSACTION,
    hidden: bool,
    dying: bool,
    one_cycle: bool,
    name: String,
    score: i32,
    killer: String,
    killer_name: String,
}

impl Sprite {
    pub fn new(
        id: String,
        bitmap: BitmapRes,
        position: Point,
        velocity: Point,
        z_order: i32,
        bounds: Rect,
        bounds_action: BOUNDSACTION,
    ) -> Sprite {
        let mut sprite = Sprite {
            id: id,
            parent: None,
            position: Rect::new(
                position.x,
                position.y,
                position.x + bitmap.width(),
                position.y + bitmap.height(),
            ),
            bitmap: bitmap,
            num_frames: 1,
            cur_frame: 0,
            frame_delay: 0,
            frame_trigger: 0,
            velocity: velocity,
            z_order: z_order,
            bounds: bounds,
            bounds_action: bounds_action,
            hidden: false,
            dying: false,
            one_cycle: false,
            name: "".to_string(),
            collision: Rect::zero(),
            score: 0,
            killer: String::new(),
            killer_name: String::new(),
        };
        sprite.calc_collision_rect();
        sprite
    }

    pub fn from_bitmap(id:String, bitmap: BitmapRes, bounds: Rect) -> Sprite {
        Sprite::new(
            id,
            bitmap,
            Point { x: 0, y: 0 },
            Point { x: 0, y: 0 },
            0,
            bounds,
            BA_STOP,
        )
    }

    pub fn with_bounds_action(
        id: String,
        bitmap: BitmapRes,
        bounds: Rect,
        bounds_action: BOUNDSACTION
    ) -> Sprite {
        //计算随即位置
        let x = rand_int(0, bounds.right - bounds.left);
        let y = rand_int(0, bounds.bottom - bounds.top);
        Sprite::new(
            id,
            bitmap,
            Point { x: x, y: y },
            Point { x: 0, y: 0 },
            0,
            bounds,
            bounds_action,
        )
    }

    pub fn with_bounds_action_norand(
        id: String,
        bitmap: BitmapRes,
        bounds: Rect,
        bounds_action: BOUNDSACTION,
    ) -> Sprite {
        Sprite::new(
            id,
            bitmap,
            Point { x: 0, y: 0 },
            Point { x: 0, y: 0 },
            0,
            bounds,
            bounds_action,
        )
    }

    fn calc_collision_rect(&mut self) {
        let x_shrink = (self.position.left - self.position.right) / 12;
        let y_shrink = (self.position.top - self.position.bottom) / 12;
        self.collision = self.position;
        self.collision.inflate(x_shrink, y_shrink);
    }

    //-----------------------------------------------------------------
    // Sprite General Methods
    //-----------------------------------------------------------------
    pub fn update(&mut self) -> SPRITEACTION {
        //let msg = format!("before update>position={:?}", self.position());
        //unsafe { log(msg.as_ptr(), msg.len()); }
        // See if the sprite needs to be killed
        if self.dying {
            return SA_KILL;
        }

        // Update the frame
        self.update_frame();

        // Update the position
        let mut new_position = Point { x: 0, y: 0 };
        let mut sprite_size = Point { x: 0, y: 0 };
        let mut bounds_size = Point { x: 0, y: 0 };
        new_position.x = self.position.left + self.velocity.x;
        new_position.y = self.position.top + self.velocity.y;
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
                new_position.x = cmp::max(
                    self.bounds.left,
                    cmp::min(new_position.x, self.bounds.right - sprite_size.x),
                );
                self.set_velocity(0, 0);
            }
            if new_position.y < self.bounds.top
                || new_position.y > (self.bounds.bottom - sprite_size.y)
            {
                new_position.y = cmp::max(
                    self.bounds.top,
                    cmp::min(new_position.y, self.bounds.bottom - sprite_size.y),
                );
                self.set_velocity(0, 0);
            }
        }
        self.set_position_point(&new_position);

        //let msg = format!("after update>position={:?}", self.position());
        //unsafe { log(msg.as_ptr(), msg.len()); }
        SA_NONE
    }

    pub fn draw(&self, context: Rc<Box<GameContext>>) {
        // Draw the sprite if it isn't hidden
        if !self.hidden {
            // Draw the appropriate frame, if necessary
            match self.num_frames {
                1 => context.draw_image_at(self.bitmap.id, self.position.left, self.position.top),
                _ => context.draw_image(
                    self.bitmap.id,
                    0,
                    self.cur_frame * self.height(),
                    self.width(),
                    self.height(),
                    self.position.left,
                    self.position.top,
                    self.width(),
                    self.height(),
                ),
            }
            context.fill_style("#ccccff");
            context.set_canvas_font("16px 微软雅黑");
            if self.name.len()>0&&self.score>=0{
                let score = &format!("({}分)", self.score);
                let w = self.name.len()*5+score.len()*5;
                let x = self.position.left+((self.position.right-self.position.left)/2-(w as i32/2));
                let y = self.position.bottom+20;
                context.fill_text(&format!("{}{}", self.name, score), x, y);
            }
        }
    }

    pub fn update_frame(&mut self) {
        self.frame_trigger -= 1;
        if (self.frame_delay >= 0) && (self.frame_trigger <= 0) {
            // Reset the frame trigger;
            self.frame_trigger = self.frame_delay;

            // Increment the frame
            self.cur_frame += 1;
            if self.cur_frame >= self.num_frames {
                // If it's a one-cycle frame animation, kill the sprite
                match self.one_cycle {
                    true => self.dying = true,
                    _ => self.cur_frame = 0,
                }
            }
        }
    }

    pub fn set_current_frame(&mut self, cur_frame: i32) {
        self.cur_frame = cur_frame;
    }

    pub fn current_frame(&self) -> i32 {
        self.cur_frame
    }

    pub fn set_frame_delay(&mut self, frame_delay: i32) {
        self.frame_delay = frame_delay;
    }

    pub fn set_velocity(&mut self, x: i32, y: i32) {
        self.velocity.x = x;
        self.velocity.y = y;
    }

    pub fn set_velocity_point(&mut self, velocity: &Point) {
        self.velocity.x = velocity.x;
        self.velocity.y = velocity.y;
    }

    pub fn velocity(&self) -> &Point {
        &self.velocity
    }

    pub fn set_position_point(&mut self, position: &Point) {
        let dx = position.x - self.position.left;
        let dy = position.y - self.position.top;
        self.position.offset(dx, dy);
        self.calc_collision_rect();
    }

    pub fn set_position(&mut self, x: i32, y: i32) {
        let x = x - self.position.left;
        let y = y - self.position.top;
        self.position.offset(x, y);
        self.calc_collision_rect();
    }

    pub fn set_position_rect(&mut self, position: Rect) {
        self.position = position;
    }

    pub fn test_collison(&self, test: &Rect) -> bool {
        self.collision.left <= test.right && test.left <= self.collision.right
            && self.collision.top <= test.bottom && test.top <= self.collision.bottom
    }

    pub fn is_point_inside(&self, x: i32, y: i32) -> bool {
        self.position.contain(x, y)
    }

    pub fn height(&self) -> i32 {
        if self.num_frames>0 {
            self.bitmap.height / self.num_frames
        }else{
            self.bitmap.height
        }
    }

    pub fn width(&self) -> i32 {
        self.bitmap.width
    }

    pub fn z_order(&self) -> i32 {
        self.z_order
    }

    pub fn bitmap(&self) -> &BitmapRes {
        &self.bitmap
    }

    pub fn position(&self) -> &Rect {
        &self.position
    }

    pub fn hidden(&self) -> bool {
        self.hidden
    }

    // pub fn id(&self)->Uuid{
    //     self.id
    // }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name:String){
        self.name = name;
    }

    pub fn set_killer(&mut self, killer:String, name:String){
        self.killer = killer;
        self.killer_name = name;
    }

    pub fn killer(&self) -> String{
        self.killer.clone()
    }

    pub fn killer_name(&self) -> &String{
        &self.killer_name
    }

    pub fn set_num_frames(&mut self, num_frames: i32, one_cycle: bool) {
        self.num_frames = num_frames;
        self.one_cycle = one_cycle;

        //重新计算位置
        self.position.bottom =
            self.position.top + (self.position.bottom - self.position.top) / self.num_frames;
    }

    pub fn kill(&mut self) {
        self.dying = true;
    }
    
    pub fn add_score(&mut self){
        self.score += 1;
    }

    pub fn set_score(&mut self, score:i32){
        self.score = score;
    }


    pub fn score(&self)->i32{
        self.score
    }
}
