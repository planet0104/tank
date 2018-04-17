extern crate ws;
extern crate tank;
extern crate uuid;
extern crate num;
use uuid::Uuid;
use ws::{WebSocket, CloseCode, Handler, Message, Result, Sender, Handshake};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as GameSender;
use std::time::Duration;
use tank::TankGame;
use tank::utils::Timer;
use std::thread;
use tank::{
    KeyEvent,
    MSG_CONNECT,
    MSG_DISCONNECT,
    MSG_START,
    MSG_KEY_EVENT,
    MSG_MOUSE_EVENT,
    SERVER_MSG_EVENT,
    SERVER_MSG_UUID,
    SERVER_MSG_DATA
};

// 服务器Web处理程序
struct Client {
    out: Sender,
    //i64 是玩家发送给服务器的消息ID, String是玩家的uuid, String是附加消息(如 keycode、鼠标坐标等等)
    /*
        client来的消息格式:
        MSG_ID␊内容

        server下发的消息格式:
        SERVER_MSG_ID␊内容
    */
    sender: GameSender<(Sender, i64, String, Option<String>)>,
    uuid: String //玩家连线以后，创建uuid，此uuid也用于玩家精灵的id
}

impl Client{}

impl Handler for Client {

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("客户端连接:{:?}", shake.remote_addr());

        //玩家连线，从游戏拉去精灵数据，发送给客户端: SERVER_MSG_ID␊UUID
        let _ = self.out.send(Message::text(format!("{}␊{}", SERVER_MSG_UUID, self.uuid)));
        let _ = self.sender.send((self.out.clone(), MSG_CONNECT, self.uuid.clone(), None));
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str){
        //玩家下线
        let _ = self.sender.send((self.out.clone(), MSG_DISCONNECT, self.uuid.clone(), None));
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("on_message:{:?}", msg);
        /*
            服务器端接收的消息:
                 玩家加入游戏=> MSG_START␊NAME
                 玩家键盘操作=> MSG_KEY_EVENT␊KeyEvent␟Key
        */
        if let Ok(text) = msg.into_text(){
            //分离消息ID
            if let Some(lf) = text.find('␊'){
                if let Some(msg_id) = text.get(0..lf){
                    let data = if let Some(d) = text.get(lf..){
                        Some(d.to_string())
                    }else{ None};
                    let _ = self.sender.send((self.out.clone(), format!("{}", msg_id), self.uuid.clone(), data));
                    return  Ok(());
                }
            }
        }
        return self.out.send(Message::text("消息格式错误"));
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

        let mut timer = Timer::new(2);
        let mut game = TankGame::new();
        loop{
            //处理websocket传来的消息
            if let Ok((sender, msg_id, uuid, data)) = game_receiver.try_recv(){
                match msg_id{
                    MSG_CONNECT => {
                        println!("玩家连接 {}", uuid);
                        /*
                            玩家连线，返回所有精灵列表
                            SERVER_MSG_ID␊ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame␊...
                        */
                        let sprites = game.sprites();
                        let mut msg = format!("{}␊", SERVER_MSG_DATA);
                        for sprite in sprites{
                            msg.push_str(&format!("{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␊",
                                sprite.id.clone(),
                                sprite.bitmap().id(),
                                sprite.position().left,
                                sprite.position().top,
                                sprite.position().right,
                                sprite.position().bottom,
                                sprite.velocity().x,
                                sprite.velocity().y,
                                sprite.current_frame()
                            ));
                        }
                        //删掉最后一个换行键
                        let _ = msg.pop();
                        let _ = sender.send(Message::text(msg));
                    }

                    MSG_START => {
                        //玩家加入游戏
                        game.join_game(uuid, data.unwrap_or(String::from("无名氏")));
                    }

                    MSG_DISCONNECT => {
                        //玩家断开连接
                        game.leave_game(&uuid)
                    }

                    MSG_KEY_EVENT => {
                        let slices:Vec<&str> = data.unwrap_or("".to_string()).split("␟").collect();
                        //玩家上传按键事件
                        if slices.len() == 2{
                            if let Ok(event) = slices[0].parse::<i64>(){
                                game.on_key_event(KeyEvent::from_i64(event), slices[1], &uuid);
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

                /*
                    游戏更新以后，获取精更新、死亡、添加事件，分发到客户端
                    SERVER_MSG_ID␊EventId␟ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame␊...
                */
                {
                    let events = game.events();
                    if events.len()>0{
                        let mut msg = format!("{}␊", SERVER_MSG_EVENT);
                        for event in events{
                            msg.push_str(&format!("{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␊",
                                event.0.to_i64(),
                                event.1.id.clone(),
                                event.1.res_id,
                                event.1.position.left,
                                event.1.position.top,
                                event.1.position.right,
                                event.1.position.bottom,
                                event.1.velocity.x,
                                event.1.velocity.y,
                                event.1.current_frame
                            ));
                        }
                        //删掉最后一个换行键
                        let _ = msg.pop();
                        let _ = broadcaster.broadcast(Message::text(msg));
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