use Bitmap;
use canvas::Canvas;
use sprite::PointF;
use std::cell::RefCell;
use std::rc::Rc;

//一个Animation代表一行图片
//Sprite中有多行图片，对应多个Animation
pub struct Animation {
    bitmap: Rc<RefCell<Bitmap>>,
    src_x: u32,       //x坐标
    src_y: u32,       //y坐标
    frame_count: u32, //帧个数
    loops: i32, //0表示动画不执行，1表示动画执行一次,-1表示动画循环执行
    cur_frame: u32,
    check_frame: u32, //如果小于当前帧, check_frame+1并返回true, 如果等于当前帧返回false
    frame_width: u32,
    frame_height: u32,
    _duration: u32,   //动画时间 ms
    frame_delay: f64, //帧间隔 ms
    time_elapsed: f64,
    end: bool, //动画是否结束
    flip_horizontal: bool,
    flip_vertical: bool,
    translate: (f64, f64),
    tag: f64,
    tag_point: PointF,
}

impl Animation {
    pub fn new(
        bitmap: Rc<RefCell<Bitmap>>,
        src_x: u32,
        src_y: u32,
        frame_width: u32,
        frame_height: u32,
        frame_count: u32,
        duration: u32,
        loops: i32,
    ) -> Animation {
        Animation {
            bitmap,
            src_x,
            src_y,
            frame_width,
            frame_height,
            frame_count,
            _duration: duration,
            loops,
            cur_frame: 0,
            check_frame: 0,
            frame_delay: duration as f64 / frame_count as f64,
            time_elapsed: 0.0,
            end: false,
            flip_horizontal: false,
            flip_vertical: false,
            translate: (0.0, 0.0),
            tag: 0.0,
            tag_point: PointF::zero(),
        }
    }

    pub fn infinite(
        bitmap: Rc<RefCell<Bitmap>>,
        src_x: u32,
        src_y: u32,
        frame_width: u32,
        frame_height: u32,
        frame_count: u32,
        duration: u32,
    ) -> Animation {
        Animation::new(
            bitmap,
            src_x,
            src_y,
            frame_width,
            frame_height,
            frame_count,
            duration,
            -1,
        )
    }

    //单帧精灵
    pub fn single_frame(
        bitmap: Rc<RefCell<Bitmap>>,
        src_x: u32,
        src_y: u32,
        frame_width: u32,
        frame_height: u32,
    ) -> Animation {
        Animation::new(bitmap, src_x, src_y, frame_width, frame_height, 1, 0, 0)
    }

    //执行一遍
    pub fn on_cycle(
        bitmap: Rc<RefCell<Bitmap>>,
        src_x: u32,
        src_y: u32,
        frame_width: u32,
        frame_height: u32,
        frame_count: u32,
        duration: u32,
    ) -> Animation {
        Animation::new(
            bitmap,
            src_x,
            src_y,
            frame_width,
            frame_height,
            frame_count,
            duration,
            1,
        )
    }

    pub fn height(&self) -> u32 {
        self.frame_height
    }

    pub fn width(&self) -> u32 {
        self.frame_width
    }

    //动画是否结束
    pub fn end(&self) -> bool {
        self.end
    }

    //绘制当前帧
    pub fn draw(&self, x: i32, y: i32, context: &Canvas) {
        //context.console_log(&format!("cur_frame={} self.time_elapsed={}", self.cur_frame, self.time_elapsed));

        context.save();

        //检查是否反转
        let (mut x, mut y) = (x, y);
        let flip = self.flip_horizontal || self.flip_vertical;
        if flip {
            let mut scale_x = 1.0;
            let mut scale_y = 1.0;
            if self.flip_horizontal {
                scale_x = -1.0;
                x = -x - self.frame_width as i32;
            }
            if self.flip_vertical {
                scale_y = -1.0;
                y = -y - self.frame_height as i32;
            }
            context.scale(scale_x, scale_y);
        }
        context.translate(self.translate.0, self.translate.1);
        context.draw_image(
            &*self.bitmap.borrow(),
            (self.src_x + (self.cur_frame % self.frame_count) * self.frame_width) as i32,
            self.src_y as i32,
            self.frame_width as i32,
            self.frame_height as i32,
            x,
            y,
            self.frame_width as i32,
            self.frame_height as i32,
        );

        context.restore();
    }

    //初始化
    pub fn init(&mut self) {
        self.cur_frame = 0;
        self.time_elapsed = 0.0;
        self.check_frame = 0;
        if self.loops == 1 {
            self.end = false;
        }
    }

    //更新动画
    pub fn update(&mut self, elapsed_milis: f64) {
        self.time_elapsed += elapsed_milis;

        if !self.end {
            self.cur_frame = (self.time_elapsed / self.frame_delay) as u32;
            //动画执行到最后一帧标记结束
            if self.loops == 1 && self.cur_frame >= self.frame_count {
                self.end = true;
                self.cur_frame = self.frame_count - 1;
            }
        }
    }
    //设置图像反转
    pub fn set_flip(&mut self, flip_horizontal: bool, flip_vertical: bool) {
        self.flip_horizontal = flip_horizontal;
        self.flip_vertical = flip_vertical;
    }

    pub fn set_translate(&mut self, x: f64, y: f64) {
        self.translate.0 = x;
        self.translate.1 = y;
    }

    pub fn cur_frame(&self) -> u32 {
        self.cur_frame % self.frame_count
    }

    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }

    pub fn set_tag(&mut self, tag: f64) {
        self.tag = tag;
    }

    pub fn get_tag(&self) -> f64 {
        self.tag
    }

    pub fn add_tag(&mut self, val: f64) {
        self.tag += val;
    }

    pub fn set_tag_point(&mut self, x: f64, y: f64) {
        self.tag_point.x = x;
        self.tag_point.y = y;
    }

    pub fn get_tag_point(&self) -> &PointF {
        &self.tag_point
    }

    pub fn check_frame(&mut self) -> bool {
        if self.check_frame == self.cur_frame && self.end() {
            self.check_frame += 1;
            true
        } else if self.check_frame < self.cur_frame {
            self.check_frame += 1;
            true
        } else {
            false
        }
    }
}
