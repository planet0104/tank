#[macro_use]
extern crate stdweb;
extern crate rand;
extern crate uuid;
use uuid::Uuid;
use rand::Rng;
use std::time::{ Duration, SystemTime};

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

    fn ready_for_next_frame(&mut self)->bool{
        if self.start_time.elapsed().unwrap() > self.next_time {
            //更新时间
            self.next_time = self.start_time.elapsed().unwrap() + Duration::from_millis(self.frame_time);
            true
        }else{
            false
        }
    }
}

static mut TIMER: Option<Timer> = None;

// cargo build --target wasm32-unknown-emscripten --release
//https://github.com/raphamorim/wasm-and-rust
fn main(){
    stdweb::initialize();
    //call_js();

    js! {
        var myCanvas = document.getElementById("draw");
        var ctx = myCanvas.getContext("2d");
        ctx.fillText("你好世界", 20, 20);
        console.log("你好世界!");
    }
    
    let my_uuid = Uuid::new_v4();
    println!("哈喽{}", my_uuid);
    let tuple = rand::random::<(f64, char)>();
    println!("random={:?}", tuple);

    let r = rand_int(1, 10);
    println!("rand_int={}", r);

    unsafe { TIMER = Some(Timer::new(30)); }
    let elapsed_secs = || {
        let elap = unsafe { TIMER.as_mut().unwrap().elapsed_secs() };
        js!{
            console.log(@{elap});
        }
    };
    js!{
        var elapsed_secs = @{elapsed_secs};
        setTimeout(elapsed_secs, 5000);
    }
}

//返回[low, low] 区间的数
pub fn rand_int(low: i32, high: i32) -> i32{
    rand::thread_rng().gen_range(low, high+1)
}