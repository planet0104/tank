extern crate ws;
#[macro_use]
extern crate json;
extern crate tank;
use ws::{listen, Handler, Message, Request, Response, Result, Sender, Handshake};
use json::JsonValue;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as ThreadOut;
use std::time::Duration;
use tank::{ TankGame, SpriteEvent };
use tank::utils::Timer;
use std::thread;

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
    let _ws = thread::spawn(move || listen(address, |out| Player { out }).unwrap());
    
    //启动一个线程以30帧的速度进行游戏逻辑更新
    let gs  = thread::spawn(move || {
        let delay_ms = Duration::from_millis(10);
        let mut timer = Timer::new(30);
        let mut game = TankGame::new();
        loop{
            //处理websocket传来的消息: 玩家上线、键盘输入
            
            if timer.ready_for_next_frame(){
                game.update();

                //游戏更新以后，获取精灵死亡、添加事件，分发到客户端
                {
                    let mut events = game.events();
                    for event in events{
                        match event.0{
                            SpriteEvent::Add => {
                                
                            }
                            SpriteEvent::Update => {
                                
                            },
                            SpriteEvent::Delete => {
                                
                            }
                        }
                    }
                }
                //清空事件
                game.events().clear();
            }
            //给一些延迟, 降低CPU使用率
            thread::sleep(delay_ms);
        }
    });
    
    println!("游戏服务已启动: {}", address);
    let res = gs.join();
    println!("游戏服务结束 {:?}", res);
}