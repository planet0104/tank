extern crate ws;
extern crate tank;
extern crate uuid;
extern crate num;
use uuid::Uuid;
use ws::{WebSocket, CloseCode, Handler, Message, Result, Sender, Handshake};
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender as GameSender;
use tank::GAME;
use std::time::{ Duration, Instant};
//use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use tank::{
    SERVER_IP,
    KeyEvent,
    MSG_CONNECT,
    MSG_DISCONNECT,
    MSG_START,
    MSG_KEY_EVENT,
    SERVER_MSG_ERR,
    SERVER_MSG_EVENT,
    SERVER_MSG_UUID,
    SERVER_MSG_DATA
};
use tank::utils::{duration_to_milis};

// 服务器Web处理程序
struct Client {
    out: Sender,
    //i64 是玩家发送给服务器的消息ID, String是玩家的uuid, String是附加消息(如 keycode、鼠标坐标等等)
    /*
        client来的消息格式:
        MSG_ID\n内容

        server下发的消息格式:
        SERVER_MSG_ID\n内容
    */
    sender: GameSender<(Sender, i64, String, String)>,
    uuid: String //玩家连线以后，创建uuid，此uuid也用于玩家精灵的id
}

impl Client{}

impl Handler for Client {

    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("客户端连接:{:?}", shake.remote_addr());

        //玩家连线，从游戏拉去精灵数据，发送给客户端: SERVER_MSG_ID\nUUID
        let _ = self.out.send(Message::text(format!("{}\n{}", SERVER_MSG_UUID, self.uuid)));
        let _ = self.sender.send((self.out.clone(), MSG_CONNECT, self.uuid.clone(), "".to_string()));
        Ok(())
    }

    fn on_close(&mut self, _code: CloseCode, _reason: &str){
        //玩家下线
        let _ = self.sender.send((self.out.clone(), MSG_DISCONNECT, self.uuid.clone(), "".to_string()));
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        //println!("on_message:{:?}", msg);
        /*
            服务器端接收的消息:
                 玩家加入游戏=> MSG_START\nNAME
                 玩家键盘操作=> MSG_KEY_EVENT\nKeyEvent␟Key
        */
        if let Ok(text) = msg.into_text(){
            //分离消息ID
            if let Some(lf) = text.find('\n'){
                if let Some(msg_id) = text.get(0..lf){
                    if let Ok(msg_id) = msg_id.parse::<i64>(){
                        let data = String::from(text.get(lf+1..).unwrap_or(""));
                        let _ = self.sender.send((self.out.clone(), msg_id, self.uuid.clone(), data));
                        return Ok(());
                    }
                }
            }
        }
        self.out.send(Message::text(format!("{}\n消息格式错误", SERVER_MSG_ERR)))
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
        GAME.with(|game|{
            let mut total_frames = 0;
            let start_time = Instant::now();
            let mut last_time = start_time.elapsed();
            let mut game = game.borrow_mut();
            loop{
                let timestamp = start_time.elapsed();
                let elapsed_ms = timestamp-last_time;
                //println!("elapsed_ms={:?}", duration_to_milis(&elapsed_ms));
                //处理websocket传来的消息
                if let Ok((sender, msg_id, uuid, data)) = game_receiver.try_recv(){
                    match msg_id{
                        MSG_CONNECT => {
                            println!("玩家连接 {}", uuid);
                            /*
                                玩家连线，返回所有精灵列表
                                SERVER_MSG_ID\nID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame\n...
                            */
                            let sprites = game.sprites();
                            let mut msg = format!("{}\n", SERVER_MSG_DATA);
                            for sprite in sprites{
                                msg.push_str(&format!("{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}\n",
                                    sprite.id.clone(),
                                    sprite.bitmap().id(),
                                    sprite.position().left,
                                    sprite.position().top,
                                    sprite.position().right,
                                    sprite.position().bottom,
                                    sprite.velocity().x,
                                    sprite.velocity().y,
                                    sprite.current_frame(),
                                    sprite.name().clone(),
                                    sprite.score(),
                                    sprite.killer_name()
                                ));
                            }
                            //删掉最后一个换行键
                            let _ = msg.pop();
                            let _ = sender.send(Message::text(msg));
                        }

                        MSG_START => {
                            //玩家加入游戏
                            println!("join_game {} {}", uuid, data);
                            game.server_join_game(uuid, data);
                        }

                        MSG_DISCONNECT => {
                            //玩家断开连接
                            game.server_leave_game(&uuid)
                        }

                        MSG_KEY_EVENT => {
                            let slices:Vec<&str> = data.split("␟").collect();
                            //玩家上传按键事件
                            if slices.len() == 2{
                                if let Ok(event) = slices[0].parse::<i64>(){
                                    if let Ok(key) = slices[1].parse::<i32>(){
                                        //println!("key event {} {:?} {}", event, slices[1], uuid);
                                        game.server_on_key_event(KeyEvent::from_i64(event), key, &uuid);
                                    }
                                }
                            }
                        }

                        other => {
                            println!("未定义消息: id={}", other)
                        }
                    }
                }
                game.server_update(duration_to_milis(&elapsed_ms));

                /*
                    游戏更新以后，获取精更新、死亡、添加事件，分发到客户端
                    SERVER_MSG_ID\nEventId␟ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame\n...
                */
                {
                    let events = game.events();
                    if events.len()>0{
                        //println!("分发事件 {:?}", events);
                        let mut msg = format!("{}\n", SERVER_MSG_EVENT);
                        for event in events{
                            msg.push_str(&format!("{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}\n",
                                event.0.to_i64(),
                                event.1.id.clone(),
                                event.1.res_id,
                                event.1.position.left,
                                event.1.position.top,
                                event.1.position.right,
                                event.1.position.bottom,
                                event.1.velocity.x,
                                event.1.velocity.y,
                                event.1.current_frame,
                                event.1.name,
                                event.1.score,
                                event.1.killer_name
                            ));
                        }
                        //删掉最后一个换行键
                        let _ = msg.pop();
                        let _ = broadcaster.broadcast(Message::text(msg));
                    }
                }
                //清空事件
                game.events().clear();
                last_time = timestamp;
                thread::sleep(Duration::from_millis(20));
                total_frames += 1;
            }
        });
    });

    //启动websocket服务
    println!("游戏服务已启动: {}", SERVER_IP);
    ws.listen(SERVER_IP).unwrap();
    println!("游戏服务结束.");
}