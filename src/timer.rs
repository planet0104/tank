use ::{current_time_millis};

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
        self.start_time = unsafe { current_time_millis() as u64 };
        //lastTime 记录上一次的时间值
        //self.last_time = unsafe { current_time() as u64 } - self.start_time;
        
        //更新时间在下一帧使用
        //self.next_time = self.last_time;
        self.next_time = self.start_time;
    }

    pub fn ready_for_next_frame(&mut self)->bool{
        
	    //逝去的时间
        self.current_time = unsafe { current_time_millis() as u64 } - self.start_time;
        
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