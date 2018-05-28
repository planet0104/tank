use canvas::Canvas;
use Bitmap;
use std::cell::RefCell;
use std::rc::Rc;

//一个Animation代表一行图片
//Sprite中有多行图片，对应多个Animation
pub struct Animation{
    bitmap: Rc<RefCell<Bitmap>>,
    src_x:u32,//x坐标
    src_y:u32,//y坐标
    frame_count:u32,//帧个数
    loops: i32,//0表示动画不执行，1表示动画执行一次,-1表示动画循环执行
    cur_frame: u32,
    frame_width: u32,
    frame_height: u32,
    _duration: u32,//动画时间 ms
    frame_delay: f64,//帧间隔 ms
    time_elapsed: f64,
}

impl Animation{
    pub fn new(bitmap:Rc<RefCell<Bitmap>>, src_x:u32, src_y:u32, frame_width:u32, frame_height:u32, frame_count:u32, duration: u32, loops: i32) ->Animation{
        Animation{bitmap, src_x, src_y, frame_width, frame_height, frame_count, _duration: duration, loops, cur_frame: 0,
        frame_delay: duration as f64/frame_count as f64,
        time_elapsed: 0.0}
    }

    pub fn infinite(bitmap:Rc<RefCell<Bitmap>>, src_x:u32, src_y:u32, frame_width:u32, frame_height:u32, frame_count:u32, duration: u32) -> Animation{
        Animation::new(bitmap, src_x, src_y, frame_width, frame_height, frame_count, duration, -1)
    }

    //单帧精灵
    pub fn single_frame(bitmap:Rc<RefCell<Bitmap>>, src_x:u32, src_y:u32, frame_width:u32, frame_height:u32) -> Animation{
        Animation{bitmap, src_x, src_y, frame_width, frame_height, frame_count:1, frame_delay:0.0, loops:0, cur_frame: 0, _duration: 0,
        time_elapsed: 0.0}
    }

    //执行一遍
    pub fn on_cycle(bitmap:Rc<RefCell<Bitmap>>, src_x:u32, src_y:u32, frame_width:u32, frame_height:u32, frame_count:u32, duration: u32) -> Animation{
        Animation::new(bitmap, src_x, src_y, frame_width, frame_height, frame_count, duration, 1)
    }

    pub fn height(&self) -> u32{
        self.frame_height
    }

    pub fn width(&self) -> u32{
        self.frame_width
    }

    //动画是否结束
    pub fn end(&self) -> bool{
        self.loops == 1 && self.cur_frame>=self.frame_count
    }

    //绘制当前帧
    pub fn draw(&self, x:i32, y:i32, context: &Canvas){
        //context.console_log(&format!("cur_frame={} self.time_elapsed={}", self.cur_frame, self.time_elapsed));
        context.draw_image(
            &*self.bitmap.borrow(),
            (self.src_x+(self.cur_frame%self.frame_count) * self.frame_width) as i32,
            self.src_y as i32,
            self.frame_width as i32,
            self.frame_height as i32,
            x,
            y,
            self.frame_width as i32,
            self.frame_height as i32,
        );
    }

    //初始化
    pub fn init(&mut self){
        self.cur_frame = 0;
        self.time_elapsed = 0.0;
    }

    //更新动画
    pub fn update(&mut self, elapsed_milis: f64){
        self.time_elapsed += elapsed_milis;

        if self.loops!=0 && self.frame_delay>0.0 && !self.end(){
            self.cur_frame = (self.time_elapsed/self.frame_delay) as u32;
        }
    }
}