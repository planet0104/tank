extern crate ws;
#[macro_use]
extern crate json;
extern crate tank;
use ws::{listen, Handler, Message, Request, Response, Result, Sender, Handshake};
use json::JsonValue;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;
use std::time::Duration;
use tank::TankGame;

const MSG_CREATE:i32 = 1;
const MSG_DELETE:i32 = 2;
const MSG_UPDATE:i32 = 3;
const MSG_QUERY:i32 = 4;

// 服务器Web处理程序
struct Player {
    out: Sender,
}

impl Player{}

impl Handler for Player {

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("客户端连接:{:?}", shake.remote_addr());
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        self.out.send(Message::text("未定义消息类型"))
    }
}

fn main() {


    //启动websocket服务
    let address = "127.0.0.1:8080";
    let ws =
        thread::spawn(move || listen(address, |out| Player { out }).unwrap());
    
    //启动一个线程以30帧的速度进行游戏逻辑更新
    let gs  = thread::spawn(move || {
        let delay_ms = time::Duration::from_millis(10);
        let timer = InstantTimer::new(30);
        let engine = GameEngine::new(30, CLIENT_WIDTH, CLIENT_HEIGHT, GameHandler{});
        loop{
            
            if timer.ready_for_next_frame(){

            }
            //给一些延迟, 降低CPU使用率
            thread::sleep(delay_ms);
        }
    });
    
    println!("游戏服务已启动: {}", address)
    let res = gs.join();
    println!("游戏服务结束 {:?}", res);
}