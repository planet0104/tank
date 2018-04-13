extern crate rand;
use rand::Rng;

// extern "C" {
//     pub fn _random() -> f64;
// }

//返回[low, low] 区间的数
pub fn rand_int(low: i32, high: i32) -> i32 {
    rand::thread_rng().gen_range(low, high + 1)
}

// pub fn rand_int_js(l:i32, b:i32)->i32{
//     unsafe{
//         ((_random()*(b as f64 - l as f64 + 1.0)).floor()+l as f64) as i32
//     }
// }

pub struct Timer {
    frame_time: u64,
    next_time: u64,
    current_time: fn() -> u64,
}

impl Timer {
    pub fn new(fps: u64, current_time: fn() -> u64) -> Timer {
        Timer {
            frame_time: 1000 / fps,
            next_time: current_time() + 1000 / fps,
            current_time: current_time,
        }
    }

    pub fn ready_for_next_frame(&mut self) -> bool {
        let now = (self.current_time)();
        if now > self.next_time {
            //更新时间
            self.next_time = now + self.frame_time;
            true
        } else {
            false
        }
    }
}
