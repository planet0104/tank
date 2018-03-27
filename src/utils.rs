use ::{random, current_time_millis};

#[derive(Debug)]
pub struct Point{
    pub x: i32,
    pub y: i32
}

impl Point {
    pub fn new() -> Point {
        Point{ x: 0, y: 0}
    }
}

impl Clone for Point {
    fn clone(&self) -> Point {
        Point{ x: self.x, y: self.y }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
    }
}

#[derive(Debug)]
pub struct PointF{
    pub x: f64,
    pub y: f64
}

impl PointF {
    pub fn new() -> PointF {
        PointF{ x: 0.0, y: 0.0}
    }

    pub fn from(x: f64, y: f64) -> PointF {
        PointF{ x: x, y: y}
    }
}

impl Clone for PointF {
    fn clone(&self) -> PointF {
        PointF{ x: self.x, y: self.y }
    }

    fn clone_from(&mut self, source: &Self) {
        self.x = source.x;
        self.y = source.y;
    }
}

//计时器
pub struct Timer{
    fps:u64,
    frame_time:u64,
    start_time:u64,
    //last_time:u64,
    next_time:u64,
    current_time:u64,
    //time_elapsed:u64,
}

impl Timer{
    pub fn new(fps:u64)->Timer{
        Timer{
            fps:fps,
            frame_time: 1000 / fps,
            start_time: 0,
            //last_time: 0,
            next_time: 0,
            current_time: 0,
            //time_elapsed: 0
        }
    }

    pub fn fps(&self)->u64{
        self.fps
    }

    pub fn start(&mut self){
        //设置计数器起始值
        self.start_time = unsafe { current_time_millis() };
        //lastTime 记录上一次的时间值
        //self.last_time = unsafe { current_time_millis() } - self.start_time;
        
        //更新时间在下一帧使用
        //self.next_time = self.last_time;
        self.next_time = self.start_time;
    }

    pub fn ready_for_next_frame(&mut self)->bool{
        
	    //逝去的时间
        self.current_time = unsafe { current_time_millis() } - self.start_time;
        
        if self.current_time > self.next_time {
            //逝去的时间
            //self.time_elapsed = (self.current_time - self.last_time) / 1000;
            //self.last_time = self.current_time;
            //更新时间
            self.next_time = self.current_time + self.frame_time;
            true
        }else{
            false
        }
    }
}

//返回-1 <n <1范围内的随机浮点数
pub fn random_clamped() -> f64{
    unsafe{ random() - random() }
}

//生成指定范围的随即整数
pub fn rand_int(l:i32, b:i32)->i32{
    unsafe{
        ((random()*(b as f64 - l as f64 + 1.0)).floor()+l as f64) as i32
    }
}

pub fn clamp(arg: &mut f64, min: f64, max: f64){
    if *arg < min {
        *arg = min;
    }
    if *arg > max {
        *arg = max;
    }
}

pub fn rgb(red: i32, green: i32, blue: i32) ->u32 {
    red as u32 | (green as u32)<<8 | (blue as u32)<<16
}