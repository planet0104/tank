extern crate rand;
use rand::Rng;
use std::time::{SystemTime, Duration, UNIX_EPOCH};

//导入的JS帮助函数
// extern "C" {
//     pub fn _random() -> f64;
//     pub fn _current_time_millis() -> f64;
// }

pub fn current_time_millis()->f64{
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let in_ms = since_the_epoch.as_secs() as f64 * 1000.0 +
        since_the_epoch.subsec_nanos() as f64 / 1_000_000.0;
    in_ms
}

//返回[low, low] 区间的数
pub fn rand_int(low: i32, high: i32) -> i32 {
    rand::thread_rng().gen_range(low, high + 1)
}

// pub struct Timer{
//     fps: f64,
//     frame_time: f64,
//     next_time: f64,
//     current_time: f64,
//     start_time: f64,
//     last_time: f64,
// }

// impl Timer{
//     pub fn new(fps:f64)->Timer{
//         Timer{
//             frame_time: 1000.0 / fps,
//             next_time: 0.0,
//             fps: fps,
//             current_time: 0.0,
//             start_time: 0.0,
//             last_time: 0.0
//         }
//     }

//     pub fn frame_time(&self)->f64{
//         self.frame_time
//     }

//     pub fn next_time(&self)->f64{
//         self.next_time
//     }

//     pub fn current_time(&self)->f64{
//         self.current_time
//     }

//     pub fn ready_for_next_frame(&mut self, timestamp:f64)->bool{
//         if self.start_time == 0.0{
//             self.start_time = timestamp;
//             self.last_time = timestamp;
//             self.next_time = timestamp;
//         }
//         //逝去的时间
// 	    self.current_time = timestamp - self.start_time;

//         if self.current_time >= self.next_time {
//             //更新时间
//             self.last_time = self.current_time;
//             self.next_time = self.current_time + self.frame_time;
//             true
//         }else{
//             false
//         }
//     }
// }


// //计时器
// pub struct ServerTimer{
//     frame_time:Duration,
//     start_time:SystemTime,
//     next_time:Duration,
//     last_time: Duration,
//     delay: Duration,
// }

// impl ServerTimer{
//     pub fn new(fps:f64, delay: Duration)->ServerTimer{
//         ServerTimer{
//             frame_time: Duration::from_secs(1) / fps as u32,
//             start_time: SystemTime::now(),
//             next_time: Duration::from_millis(0),
//             last_time: Duration::from_millis(0),
//             delay: delay,
//         }
//     }

//     pub fn elapsed(&self) -> Duration{
//         self.start_time.elapsed().unwrap()
//     }

//     pub fn next_time(&self) -> Duration{
//         self.next_time
//     }

//     pub fn delay(&self) -> Duration{
//         self.delay
//     }

//     pub fn ready_for_next_frame(&mut self)->bool{
//         let elapsed = self.elapsed();
//         if elapsed > self.next_time {
//             //更新时间
//             let ft = elapsed-self.last_time;
//             println!("frame_time={:?}", ft.as_secs() as f64 * 1000.0 + ft.subsec_nanos() as f64 / 1_000_000.0);
//             self.last_time = elapsed;
//             self.next_time = elapsed + self.frame_time;
//             true
//         }else{
//             false
//         }
//     }

//     //逝去的毫秒数
//     // pub fn elapsed_secs(&self)->f64{
//     //     let duration = self.start_time.elapsed().unwrap();
//     //     duration.as_secs() as f64
//     //        + duration.subsec_nanos() as f64 * 1e-9
//     // }
// }

pub fn duration_to_milis(duration: &Duration) -> f64{
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}
pub type Id = u32;
pub struct Counter {
	count: Id,
}
impl Counter {
	pub fn new() -> Self {
		Counter { count: 0 }
	}
}

impl Iterator for Counter {
	type Item = Id;

	fn next(&mut self) -> Option<Id> {
		if self.count != Id::max_value() {
			self.count += 1;
			Some(self.count)
		} else {
			None
		}
	}
}

/*
//生成指定范围的随即整数
pub fn js_rand_int(l:i32, b:i32)->i32{
    unsafe{
        ((_random()*(b as f64 - l as f64 + 1.0)).floor()+l as f64) as i32
    }
}

*/

// pub struct Timer{
//     frame_time:u64,
//     start_time:SystemTime,
//     next_time:Duration,
//     last_time:f64,
// }

// impl Timer{
//     pub fn new(fps:u64)->Timer{
//         Timer{
//             frame_time: 1000 / fps,
//             start_time: SystemTime::now(),
//             next_time: Duration::from_millis(0),
//             last_time: 0.0,
//         }
//     }

//     pub fn _start(&mut self){
//         //设置计数器起始值
//         self.start_time = SystemTime::now();
//         //更新时间在下一帧使用
//         self.next_time = Duration::from_millis(0);
//     }

//     //逝去的毫秒数
//     pub fn elapsed_secs(&self)->f64{
//         let duration = self.start_time.elapsed().unwrap();
//         duration.as_secs() as f64
//            + duration.subsec_nanos() as f64 * 1e-9
//     }

//     pub fn last_time(&self)->f64{
//         self.last_time
//     }

//     pub fn save_last(&mut self, t:f64){
//         self.last_time = t;
//     }

//     pub fn next_time(&self)->f64{
//         self.next_time.as_secs() as f64
//            + self.next_time.subsec_nanos() as f64 * 1e-9
//     }

//     pub fn ready_for_next_frame(&mut self)->bool{
//         let duration = self.start_time.elapsed().unwrap();
//         if duration > self.next_time {
//             //更新时间
//             self.next_time = duration + Duration::from_millis(self.frame_time);
//             true
//         }else{
//             false
//         }
//     }
// }

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

pub struct Timer{
    fps:u64,
    frame_time:u64,
    start_time:u64,
    last_time:u64,
    next_time:u64,
    current_time:u64,
}

impl Timer{
    pub fn new(fps:u64)->Timer{
        let mut timer = Timer{
            fps:fps,
            frame_time: 1000 / fps,
            start_time: 0,
            last_time: 0,
            next_time: 0,
            current_time: 0,
        };
        timer.start();
        timer
    }

    pub fn fps(&self)->u64{
        self.fps
    }

    fn current_time_millis()->u64{
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let in_ms = since_the_epoch.as_secs() * 1000 +
            since_the_epoch.subsec_nanos() as u64 / 1_000_000;
        in_ms
    }

    fn start(&mut self){
        //设置计数器起始值
        self.start_time = Timer::current_time_millis();
        //更新时间在下一帧使用
        self.next_time = 0;
    }

    pub fn ready_for_next_frame(&mut self)->bool{
        
	    //逝去的时间
        self.current_time = Timer::current_time_millis() - self.start_time;
        if self.current_time >= self.next_time {
            //更新时间
            self.next_time = self.current_time + self.frame_time;
            true
        }else{
            false
        }
    }
}

*/