extern crate ws;
#[macro_use]
extern crate json;
extern crate tank;
extern crate uuid;
extern crate num;
use uuid::Uuid;
use ws::{WebSocket, CloseCode, Handler, Message, Result, Sender, Handshake};
use json::JsonValue;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as GameSender;
use std::time::Duration;
use tank::{TankGame, KeyEvent, SpriteEvent};
use tank::utils::Timer;
use std::thread;

const MSG_CONNECT: isize = 1;
const MSG_DISCONNECT: isize = 2;
const MSG_START: isize = 3;
const MSG_KEY_EVENT: isize = 4;
const MSG_MOUSE_EVENT: isize = 5;


// 服务器Web处理程序
struct Client {
    out: Sender,
    //isize 是玩家发送给服务器的消息ID, String是玩家的uuid, JsonValue是附加消息(如 keycode、鼠标坐标等等)
    sender: GameSender<(Sender, isize, String, JsonValue)>,
    uuid: String //玩家连线以后，创建uuid，此uuid也用于玩家精灵的id
}

impl Client{}

impl Handler for Client {

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("客户端连接:{:?}", shake.remote_addr());

        //玩家连线，从游戏拉去精灵数据，发送给客户端
        let _ = self.sender.send((self.out.clone(), MSG_CONNECT, self.uuid.clone(), JsonValue::Null));
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str){
        //玩家下线
        let _ = self.sender.send((self.out.clone(), MSG_DISCONNECT, self.uuid.clone(), JsonValue::Null));
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("on_message:{:?}", msg);
        //服务器端接收的消息，只有两种 1、玩家加入游戏， 2、玩家键盘操作
        if msg.is_text(){
            if let Ok(json) = json::parse(&msg.into_text().unwrap()){
                if json.is_array(){
                    //将消息转换成整数数组
                    if let Some(msg_id) = json[0].as_isize(){
                        //玩家开始游戏，通知游戏添加精灵，然后广播
                        //玩家键盘操作，通知游戏更新，然后广播
                        let _ = self.sender.send((self.out.clone(), msg_id, self.uuid.clone(), json[1].clone()));
                        return  Ok(());
                    }
                }
            }
        }
        return self.out.send(Message::text("JSON格式错误"));
    }
}

fn main() {
    let (game_sender, game_receiver) = channel();

    let ws = WebSocket::new(|out| Client{
        out: out,
        sender: game_sender.clone(),
        uuid: Uuid::new_v4().hyphenated().to_string()
    }).unwrap();
    let broadcaster = ws.broadcaster();

    //启动一个线程以30帧的速度进行游戏逻辑更新
    let _gs  = thread::spawn(move || {
        let delay_ms = Duration::from_millis(10);
        let mut timer = Timer::new(30);
        let mut game = TankGame::new();
        loop{
            //处理websocket传来的消息
            if let Ok((sender, msg_id, uuid, json)) = game_receiver.try_recv(){
                match msg_id{
                    MSG_CONNECT => {
                        //玩家连线，返回所有精灵列表
                        let sprites = game.sprites();
                        let mut array = vec![];
                        for sprite in sprites{
                            array.push(
                                object!{
                                    "id" => sprite.id.clone(),
                                    "res" => sprite.bitmap().id(),
                                    "l" => sprite.position().left,
                                    "t" => sprite.position().top,
                                    "r" => sprite.position().right,
                                    "b" => sprite.position().bottom,
                                    "vx" => sprite.velocity().x,
                                    "vy" => sprite.velocity().y,
                                    "frame" => sprite.current_frame()
                                }
                            );
                        }
                        let _ = sender.send(Message::text(json::stringify(array)));
                    }

                    MSG_START => {
                        //玩家加入游戏
                        game.join_game(&uuid, json["name"].as_str());
                    }

                    MSG_DISCONNECT => {
                        //玩家断开连接
                        game.leave_game(&uuid)
                    }

                    MSG_KEY_EVENT => {
                        //玩家上传按键事件
                        let event = json[0].as_i32();
                        let key_code = json[1].as_i32();
                        if event.is_some() && key_code.is_some(){
                            if let Some(event) = num::FromPrimitive::from_i32(event.unwrap()){
                                game.on_key_event(event, key_code.unwrap(), &uuid);
                            }
                        }
                    }

                    MSG_MOUSE_EVENT => {
                        //玩家上传鼠标事件
                    }

                    _ => {}
                }
            }
            
            if timer.ready_for_next_frame(){
                game.update();

                //游戏更新以后，获取精更新、死亡、添加事件，分发到客户端
                {
                    let events = game.events();
                    for event in events{
                        let _ = broadcaster.broadcast(Message::text(json::stringify(
                            object!{
                                "event" => num::ToPrimitive::to_i32(&event.0).unwrap(),
                                "msg" => object!{
                                            "id" => event.1.id.clone(),
                                            "res" => event.1.res,
                                            "l" => event.1.l,
                                            "t" => event.1.t,
                                            "r" => event.1.r,
                                            "b" => event.1.b,
                                            "vx" => event.1.vx,
                                            "vy" => event.1.vy,
                                            "frame" => event.1.frame
                                        }
                            }
                        )));
                    }
                }
                //清空事件
                game.events().clear();
            }
            //给一些延迟, 降低CPU使用率
            thread::sleep(delay_ms);
        }
    });

    //启动websocket服务
    let address = "127.0.0.1:8080";

    println!("游戏服务已启动: {}", address);
    ws.listen(address).unwrap();
    println!("游戏服务结束.");
}