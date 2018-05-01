extern crate websocket;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tank;

use tank::utils::{duration_to_milis};
use tank::{
    GAME,
    SERVER_IP,
    KeyEvent,
    MSG_PULL,
    MSG_DISCONNECT,
    MSG_START,
    MSG_KEY_EVENT,
    MSG_ID_ERR,
    SERVER_MSG_ERR,
    SERVER_MSG_EVENT,
    SERVER_MSG_UUID,
    SERVER_MSG_DATA
};

use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::{ Duration, Instant};
use std::thread;
use websocket::OwnedMessage;
use websocket::sync::Server;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
use websocket::result::WebSocketError;
use std::io::ErrorKind;

use env_logger::Builder;
use log::LevelFilter;

type Writer = websocket::sender::Writer<std::net::TcpStream>;

fn main() {
    let mut builder = Builder::from_default_env();

    builder.filter(None, LevelFilter::Info)
			.default_format_timestamp(true)
			.init();

    let (game_sender, game_receiver):(Sender<(i32, String, String)>, Receiver<(i32, String, String)>) = channel();
    let connections: Arc<RwLock<HashMap<String, Writer>>> =Arc::new(RwLock::new(HashMap::new()));

    //启动游戏线程
    let connections_clone = connections.clone();
    let builder = thread::Builder::new().name("tank_game".into());
    let _gs  = builder.spawn(move || {
        GAME.with(|game|{
            //let mut total_frames = 0;
            let start_time = Instant::now();
            let mut last_time = start_time.elapsed();
            let mut game = game.borrow_mut();
            loop{
                let timestamp = start_time.elapsed();
                let elapsed_ms = timestamp-last_time;
                //let now = Instant::now();
                //处理websocket传来的消息
                if let Ok((msg_id, uuid, data)) = game_receiver.try_recv(){
                    match msg_id{
                        MSG_PULL => {
                            //info!("玩家连接 {}", uuid);
                            /*
                                玩家连线，返回所有精灵列表
                                SERVER_MSG_ID\nID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame...\n...
                            */
                            let sprites = game.sprites();
                            let mut msg = format!("{}\n{}\n", SERVER_MSG_DATA, uuid);
                            for sprite in sprites{
                                msg.push_str(&format!("{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}\n",
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
                                    sprite.killer_name(),
                                    sprite.lives(),
                                ));
                            }
                            //删掉最后一个换行键
                            let _ = msg.pop();
                            
                            send_message(connections_clone.clone(), &uuid, msg);
                        }

                        MSG_START => {
                            //info!("join_game {} {}", uuid, data);
                            //玩家加入游戏
                            game.server_join_game(uuid, data);
                        }

                        MSG_DISCONNECT => {
                            info!("玩家离开游戏{}", uuid);
                            //玩家断开连接
                            game.server_leave_game(&uuid)
                        }

                        MSG_KEY_EVENT => {
                            let slices:Vec<&str> = data.split("␟").collect();
                            //玩家上传按键事件
                            if slices.len() == 2{
                                if let Ok(event) = slices[0].parse::<i64>(){
                                    if let Ok(key) = slices[1].parse::<i32>(){
                                        //info!("key event {} {:?} {}", event, slices[1], uuid);
                                        game.server_on_key_event(KeyEvent::from_i64(event), key, &uuid);
                                    }
                                }
                            }
                        }

                        MSG_ID_ERR => {
                            send_message(connections_clone.clone(), &uuid, format!("{}\n消息格式错误", SERVER_MSG_ERR));
                        }

                        other => {
                            info!("未定义消息: id={}", other)
                        }
                    }
                }
                game.server_update(duration_to_milis(&elapsed_ms));

                /*
                    游戏更新以后，获取精更新、死亡、添加事件，分发到客户端
                    SERVER_MSG_ID\nEventId␟ID␟RES␟Left␟Top␟Right␟Bottom␟VelocityX␟VelocityY␟Frame\n...
                */
                let game_events = game.events();
                {
                    let events = &*game_events.borrow_mut();
                    if events.len()>0{
                        //info!("分发事件 {:?}", events);
                        let mut msg = format!("{}\n", SERVER_MSG_EVENT);
                        for event in events{
                            msg.push_str(&format!("{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}␟{}\n",
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
                                event.1.killer_name,
                                event.1.lives
                            ));
                        }
                        //删掉最后一个换行键
                        let _ = msg.pop();

                        //广播
                        broad_cast_message(connections_clone.clone(), msg);
                    }
                }
                //清空事件
                game_events.borrow_mut().clear();
                last_time = timestamp;
                thread::park_timeout(Duration::from_millis(20));
                //total_frames += 1;
                // if total_frames%(50*60) == 0{
                //     info!("now={:?}", now.elapsed());
                // }
            }
        });
    });

    let server = Server::bind(SERVER_IP).unwrap();
	info!("服务器已启动 {}", SERVER_IP);
	for request in server.filter_map(Result::ok) {
        let game_sender_clone = game_sender.clone();
		let ws_connections = connections.clone();
		thread::spawn(move || {
			let client = request.accept().unwrap();
			let ip = client.peer_addr().unwrap();

			info!("创建连接: {}", ip);

			let (mut receiver, sender) = client.split().unwrap();
			if let Ok(mut map) = ws_connections.write(){
				map.insert(ip.to_string(), sender);
			}

			let connections_clone = ws_connections.clone();
			for message in receiver.incoming_messages() {
				if message.is_err(){
					info!("消息错误: {:?}", message.err());
					return;
				}
				let message = message.unwrap();

				match message {
					OwnedMessage::Text(text) =>{
						//info!("on text message:{}", text);
                        /*
                            服务器端接收的消息:
                                玩家加入游戏=> MSG_START\nNAME
                                玩家键盘操作=> MSG_KEY_EVENT\nKeyEvent␟Key
                        */
                        //分离消息ID，转发给游戏线程
                        if let Some(lf) = text.find('\n'){
                            if let Some(msg_id) = text.get(0..lf){
                                if let Ok(msg_id) = msg_id.parse::<i32>(){
                                    let data = String::from(text.get(lf+1..).unwrap_or(""));
                                    let _ = game_sender_clone.send((msg_id, ip.to_string(), data));
                                    continue;
                                }
                            }
                        }
                        let _ = game_sender_clone.send((MSG_ID_ERR, ip.to_string(), "".to_string()));
                        info!("on_message Ok.");
					}
					OwnedMessage::Close(_) => {
						//info!("Client {} Close断开连接", ip);
						break;
					}
					other => {
                        info!("other message {:?}", other);
                    }
				}
			}
            connections_clone.write().unwrap().remove(&ip.to_string());
            let _ = game_sender_clone.send((MSG_DISCONNECT, ip.to_string(), "".to_string()));
            info!("连接关闭: {}", ip);
		});
	}
    
    info!("游戏服务结束.");
}

fn send_message(connections: Arc<RwLock<HashMap<String, Writer>>>, uuid:&String, message:String){
    let mut connections = connections.write().unwrap();
    if !connections.contains_key(uuid){
        info!("uuid不存在 {}", uuid);
        return;
    }
    if let Err(err) = connections.get_mut(uuid).unwrap().send_message(&OwnedMessage::Text(message)){
        info!("消息发泄失败: {:?}", err);
        match err{
            WebSocketError::IoError(err) => {
                if err.kind() == ErrorKind::ConnectionAborted{
                    connections.remove(uuid);
                }
            },
            _ => {}
        }
    }
}

fn broad_cast_message(connections: Arc<RwLock<HashMap<String, Writer>>>, message:String){
    let mut aborted_connections = vec![];
    let message = OwnedMessage::Text(message);
    for (addr, sender) in connections.write().unwrap().iter_mut(){
        if let Err(err) = sender.send_message(&message){
            info!("消息发泄失败: {:?}", err);
            match err{
                WebSocketError::IoError(err) => {
                    if err.kind() == ErrorKind::ConnectionAborted{
                        aborted_connections.push(addr.clone());
                    }
                },
                _ => {}
            }
        }
    }
    let mut connections = connections.write().unwrap();
    for aborted_addr in aborted_connections{
        connections.remove(&aborted_addr);
    }
}

// fn send_message(connections: Arc<RwLock<HashMap<String, SplitSink>>>, remote: &Remote, uuid:String, msg:String){

//     let connections_clone = connections.clone();
//     let sink = connections_clone.write();
//     if sink.is_err(){
//         return;
//     }
//     let sink = sink.unwrap()
//                         .remove(&uuid)
//                         .expect("无效连接, 消息发送失败",);
    
//     let f = sink.send(OwnedMessage::Text(msg))
//                 .and_then(move |sink| {
//                                 connections.write().unwrap().insert(uuid, sink);
//                                 Ok(())
//                                 });
//     remote.spawn(move |_| f.map_err(|_| ()));
// }