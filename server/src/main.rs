extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate tank;
extern crate hyper;
mod ws_server;
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

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use ws_server::WsServer;
use std::sync::mpsc::{Sender, Receiver, channel};

use tokio_core::reactor::{Core, Remote};
use futures::{Future, Sink, Stream};
use std::time::{ Duration, Instant};

use std::thread;
use std::sync::{RwLock, Arc};
use std::collections::HashMap;
type SinkContent = websocket::client::async::Framed<tokio_core::net::TcpStream,
                                                    websocket::async::MessageCodec<OwnedMessage>>;
type SplitSink = futures::stream::SplitSink<SinkContent>;

fn main() {
    let (game_sender, game_receiver):(Sender<(i32, String, String)>, Receiver<(i32, String, String)>) = channel();
    let connections: Arc<RwLock<HashMap<String, SplitSink>>> =Arc::new(RwLock::new(HashMap::new()));
	let mut core = Core::new().unwrap();
    //event loop 自身的 handle
	let handle = core.handle();
    //其他线程的 handle
    let remote = core.remote();
	// 绑定服务器
	let server = WsServer::bind(SERVER_IP, &handle).unwrap();

    //启动游戏线程
    let remote_clone = remote.clone();
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
                            //println!("玩家连接 {}", uuid);
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
                            send_message(connections_clone.clone(), &remote_clone, uuid, msg);
                        }

                        MSG_START => {
                            //玩家加入游戏
                            game.server_join_game(uuid, data);
                        }

                        MSG_DISCONNECT => {
                            println!("玩家断开连接{}", uuid);
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

                        MSG_ID_ERR => {
                            send_message(connections_clone.clone(), &remote_clone, uuid, format!("{}\n消息格式错误", SERVER_MSG_ERR));
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
                let game_events = game.events();
                {
                    let events = &*game_events.borrow_mut();
                    if events.len()>0{
                        //println!("分发事件 {:?}", events);
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
                        let mut keys = vec![];
                        for key in connections_clone.read().unwrap().keys(){   
                            keys.push(key.clone());
                        }

                        for key in keys{
                            send_message(connections_clone.clone(), &remote_clone, key, msg.clone());
                        }
                    }
                }
                //清空事件
                game_events.borrow_mut().clear();
                last_time = timestamp;
                thread::park_timeout(Duration::from_millis(20));
                //total_frames += 1;
                // if total_frames%(50*60) == 0{
                //     println!("now={:?}", now.elapsed());
                // }
            }
        });
    });
    
    //构建服务器的future
    //这将是一个包含服务器将要做的所有事情的结构
    //传入连接的流
	let f = server.incoming()
        .map_err(|InvalidConnection { error, .. }|{ error})
        .for_each(|(upgrade, _addr)| {
            if upgrade.key().is_none(){
                return Ok(());
            }

            //根据key生成uuid
            let uuid = {
                let key = upgrade.key().unwrap();
                format!("{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                 key[0],key[1],key[1],key[3],key[4],key[5],key[6],key[7],key[9],key[9],key[10],key[11],key[12],key[13],key[14],key[15])
            };
            println!("客户端连接. UUID={}", uuid);
            //let uuid_map = uuid.clone();

            let handle_clone = handle.clone();
            let connections_clone = connections.clone();
            let game_sender_clone = game_sender.clone();
            let f = upgrade
                .accept()
                // 发送uuid到客户端
                // .and_then(move |(s, _)|{
                //     println!("下发UUID {}", uuid);
                //     s.send(OwnedMessage::Text(format!("{}\n{}", SERVER_MSG_UUID, uuid)))
                // })
                // 处理客户端发来的消息
                .and_then(move |(s, _)| {
                    let (sink, stream) = s.split();
                    //将sink存入hahsmap
                    connections_clone.write().unwrap().insert(uuid.clone(), sink);
                    //用户上线
                    //let _ = game_sender_clone.send((MSG_CONNECT, uuid.clone(), "".to_string()));
                    handle_clone.spawn(
                        stream.for_each(move |msg| {
                            let uuid = uuid.clone();
                            match msg {
                                OwnedMessage::Text(text) =>{
                                    //println!("on_message:{}", text);
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
                                                let _ = game_sender_clone.send((msg_id, uuid, data));
                                                return Ok(());
                                            }
                                        }
                                    }
                                    let _ = game_sender_clone.send((MSG_ID_ERR, uuid, "".to_string()));
                                }
                                OwnedMessage::Close(_) => {
                                   //玩家下线
                                    let _ = game_sender_clone.send((MSG_DISCONNECT, uuid, "".to_string()));
                                }
                                _ => {},
                            }
                            Ok(())
                        }).map_err(|_| ())
                    );
                    Ok(())
                });

            handle.spawn(f.map_err(move |e| println!("{}: '{:?}'", "Client Status", e))
	              .map(move |_| println!("{}: Finished.", "Client Status")));
            Ok(())
        });

    println!("游戏服务已启动: {}", SERVER_IP);
	core.run(f).unwrap();
    println!("游戏服务结束.");
}

fn send_message(connections: Arc<RwLock<HashMap<String, SplitSink>>>, remote: &Remote, uuid:String, msg:String){
    let sink = connections.write()
                        .unwrap()
                        .remove(&uuid)
                        .expect("无效连接, 消息发送失败",);

    let f = sink.send(OwnedMessage::Text(msg))
                .and_then(move |sink| {
                                connections.write().unwrap().insert(uuid, sink);
                                Ok(())
                                });
    remote.spawn(move |_| f.map_err(|_| ()));
}