extern crate rand;
use rand::Rng;
use std::time::{ Duration, SystemTime};

//导入的JS帮助函数
// extern "C" {
//     pub fn _random() -> f64;
//     pub fn _current_time_millis() -> f64;
// }

//返回[low, low] 区间的数
pub fn rand_int(low: i32, high: i32) -> i32 {
    rand::thread_rng().gen_range(low, high + 1)
}
/*
//生成指定范围的随即整数
pub fn js_rand_int(l:i32, b:i32)->i32{
    unsafe{
        ((_random()*(b as f64 - l as f64 + 1.0)).floor()+l as f64) as i32
    }
}

*/

pub struct Timer{
    frame_time:u64,
    start_time:SystemTime,
    next_time:Duration,
}

impl Timer{
    pub fn new(fps:u64)->Timer{
        Timer{
            frame_time: 1000 / fps,
            start_time: SystemTime::now(),
            next_time: Duration::from_millis(0)
        }
    }

    pub fn _start(&mut self){
        //设置计数器起始值
        self.start_time = SystemTime::now();
        //更新时间在下一帧使用
        self.next_time = Duration::from_millis(0);
    }

    //逝去的毫秒数
    pub fn elapsed_secs(&self)->f64{
        let duration = self.start_time.elapsed().unwrap();
        duration.as_secs() as f64
           + duration.subsec_nanos() as f64 * 1e-9
    }

    pub fn ready_for_next_frame(&mut self)->bool{
        if self.start_time.elapsed().unwrap() > self.next_time {
            //更新时间
            self.next_time = self.start_time.elapsed().unwrap() + Duration::from_millis(self.frame_time);
            true
        }else{
            false
        }
    }
}

/*

pub struct JSTimer{
    fps:u64,
    frame_time:u64,
    start_time:u64,
    //last_time:u64,
    next_time:u64,
    current_time:u64,
    //time_elapsed:u64,
}

impl JSTimer{
    pub fn new(fps:u64)->JSTimer{
        JSTimer{
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
        self.start_time = unsafe { _current_time_millis() as u64 };
        //lastTime 记录上一次的时间值
        //self.last_time = unsafe { current_time() as u64 } - self.start_time;
        
        //更新时间在下一帧使用
        //self.next_time = self.last_time;
        self.next_time = self.start_time;
    }

    pub fn ready_for_next_frame(&mut self)->bool{
        
	    //逝去的时间
        self.current_time = unsafe { _current_time_millis() as u64 } - self.start_time;
        
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

*/