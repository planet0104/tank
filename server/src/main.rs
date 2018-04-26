extern crate websocket;
extern crate futures;
extern crate tokio_core;
extern crate tank;
extern crate uuid;
extern crate num;
extern crate futures_cpupool;

use websocket::message::OwnedMessage;
use websocket::server::InvalidConnection;
use websocket::async::Server;

use futures_cpupool::CpuPool;
use futures_cpupool::CpuFuture;
use futures::future::{self, Loop};
use tokio_core::reactor::{Handle, Remote, Core};
use futures::{Future, Sink, Stream};
use futures::sync::mpsc;
use futures::sync::mpsc::UnboundedSender;
use std::fmt::Debug;
use uuid::Uuid;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use tank::GAME;
use std::time::{ Duration, Instant};
//use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use std::sync::{RwLock, Arc};
use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;
use tank::utils::{ Id, Counter};

type SinkContent = websocket::client::async::Framed<tokio_core::net::TcpStream,
                                                    websocket::async::MessageCodec<OwnedMessage>>;
type SplitSink = futures::stream::SplitSink<SinkContent>;

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

fn main() {
    let connections: Arc<RwLock<HashMap<Id, SplitSink>>> =Arc::new(RwLock::new(HashMap::new()));
    let conn_id = Rc::new(RefCell::new(Counter::new()));

    // client来的消息格式:
    // MSG_ID\n内容
    // server下发的消息格式:
    // SERVER_MSG_ID\n内容

    //i32 是玩家发送给服务器的消息ID, u32是玩家的uuid, String是附加消息(如 keycode、鼠标坐标等等)
    //<(i32, u32, String)>
    let (game_sender, game_receiver):(Sender<(i32, u32, String)>, Receiver<(i32, u32, String)>) = channel();
    let (send_channel_out, send_channel_in) = mpsc::unbounded();

    //启动一个线程以50帧的速度进行游戏逻辑更新
    let connections_bc  =connections.clone();
    let builder = thread::Builder::new().name("tank_game".into());
    let send_channel_out_clone = send_channel_out.clone();
    let send_channel_out_clone_2 = send_channel_out.clone();
    let connections_inner = connections.clone();
    let _gs  = builder.spawn(move || {
        GAME.with(|game|{
            let mut total_frames = 0;
            let start_time = Instant::now();
            let mut last_time = start_time.elapsed();
            let mut game = game.borrow_mut();
            loop{
                let send_channel_out_clone_clone = send_channel_out_clone.clone();
                let send_channel_out_clone_clone2 = send_channel_out_clone.clone();
                let timestamp = start_time.elapsed();
                let elapsed_ms = timestamp-last_time;
                let now = Instant::now();
                //处理websocket传来的消息
                if let Ok((msg_id, uid, data)) = game_receiver.try_recv(){
                    println!("game_receiver.try_recv {:?}, {:?}, {:?}", msg_id, uid, data);
                    match msg_id{
                        MSG_CONNECT => {
                            //玩家连线，从游戏拉去精灵数据，发送给客户端: SERVER_MSG_ID\nUserID
                            //将用户ID回复给客户端
                            send_message(connections_inner.clone(), uid, format!("{}\n{}", SERVER_MSG_UUID, uid));
                            //let _ = send_channel_out_clone_clone2.send((uid, format!("{}\n{}", SERVER_MSG_UUID, uid)));
                            //println!("玩家连接 {}", uid);
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
                            println!(">>>这里>>>>10101010101010010");
                            //send_message(connections_inner.clone(), &uid, msg);
                            let _ = send_channel_out_clone_clone.send((uid, msg));
                        }

                        MSG_START => {
                            //玩家加入游戏
                            //println!("join_game {} {}", uuid, data);
                            game.server_join_game(format!("{}", uid), data);
                        }

                        MSG_DISCONNECT => {
                            //玩家断开连接
                            game.server_leave_game(&format!("{}", uid))
                        }

                        MSG_KEY_EVENT => {
                            let slices:Vec<&str> = data.split("␟").collect();
                            //玩家上传按键事件
                            if slices.len() == 2{
                                if let Ok(event) = slices[0].parse::<i64>(){
                                    if let Ok(key) = slices[1].parse::<i32>(){
                                        //println!("key event {} {:?} {}", event, slices[1], uuid);
                                        game.server_on_key_event(KeyEvent::from_i64(event), key, &format!("{}", uid));
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
                        
                        let map = connections_bc.read().unwrap();
                        let keys = map.keys().clone();
                        let send_channel_out_clone_2 = send_channel_out_clone_2.clone();
                        for key in keys{
                            //send_message(connections_bc_clone.clone(), &format!("{}", key), msg.clone());
                            let _ = send_channel_out_clone_2.clone().send((*key, msg.clone()));
                        }
                    }
                }
                //清空事件
                game.events().clear();
                last_time = timestamp;
                thread::park_timeout(Duration::from_millis(20));
                total_frames += 1;
                if total_frames%(50*60) == 0{
                    println!("now={:?}", now.elapsed());
                }
            }
        });
    });

    //启动websocket服务
    let mut core = Core::new().unwrap();
	let handle = core.handle();
    let remote = core.remote();
    let (receive_channel_out, receive_channel_in) = mpsc::unbounded();
	//绑定服务器
    let server = Server::bind(SERVER_IP, &handle).unwrap();
    let pool = Rc::new(CpuPool::new_num_cpus());
    println!("游戏服务已启动: {}", SERVER_IP);
    let connections_inner = connections.clone();
    let game_sender_clone = game_sender.clone();
	//接入服务器的stream
	let connection_handler = server.incoming()
    //过滤无效的连接
    .map_err(|InvalidConnection { error, .. }| error) //这句需要 futures = "0.1.21"
    .for_each(|(upgrade, addr)| {
        let channel = receive_channel_out.clone();
        let connections_inner = connections_inner.clone();
        let game_sender_clone_inner = game_sender_clone.clone();
        //接受ws连接的请求
        println!("Got a connection from: {} key={:?}", addr, upgrade.key());
        let handle_inner = handle.clone();
        let conn_id = conn_id.clone();
        let f = upgrade
            .accept()
            .and_then(move |(framed, _)| {
                    let id = conn_id
                        .borrow_mut()
                        .next()
                        .expect("maximum amount of ids reached");
                    let (sink, stream) = framed.split();
                    let f = channel.send((id, stream));
                    spawn_future(f, "Senk stream to connection pool", &handle_inner);
                    connections_inner.write().unwrap().insert(id, sink);
                    let _ = game_sender_clone_inner.send((MSG_CONNECT, id, "".to_string()));
                    Ok(())
            });
            // .and_then(move |(framed, _)| {
            //         let id = conn_id
            //             .borrow_mut()
            //             .next()
            //             .expect("maximum amount of ids reached");
            //         let (sink, stream) = framed.split();
            //         let connections_inner3 = connections_inner.clone();
            //         connections_inner.write().unwrap().insert(id, sink);
            //         //let _ = game_sender.send((MSG_CONNECT, format!("{}", id), "".to_string()));
            //         println!(">>>这里>>>>00000000000000000");
            //         stream
            //         .take_while(|m| Ok(!m.is_close()))
            //         //接收消息
            //         .filter_map(move |m| {
            //             println!(">>>这里>>>>1111111111111111");
            //             match m {
            //                 OwnedMessage::Text(text) =>{
            //                     println!(">>>这里>>>>2222222222222222222");
            //                     println!("OwnedMessage {}", text);
            //                     //         //println!("on_message:{:?}", msg);
            //                     /*
            //                         服务器端接收的消息:
            //                             玩家加入游戏=> MSG_START\nNAME
            //                             玩家键盘操作=> MSG_KEY_EVENT\nKeyEvent␟Key
            //                     */
            //                     //分离消息ID
            //                     if let Some(lf) = text.find('\n'){
            //                         if let Some(msg_id) = text.get(0..lf){
            //                             if let Ok(msg_id) = msg_id.parse::<i64>(){
            //                                 let data = String::from(text.get(lf+1..).unwrap_or(""));
            //                                 let _ = game_sender.send((msg_id, format!("{}", id), data));
            //                                 return None;
            //                             }
            //                         }
            //                     }
            //                     send_message(connections_inner2.clone(), &format!("{}", id), format!("{}\n消息格式错误", SERVER_MSG_ERR));
            //                     None
            //                 }
            //                 OwnedMessage::Close(_) => {
            //                     //玩家下线
            //                     let _ = game_sender.send((MSG_DISCONNECT, format!("{}", id), "".to_string()));
            //                     None
            //                 }
            //                 OwnedMessage::Ping(p) => Some(OwnedMessage::Pong(p)),
            //                 OwnedMessage::Pong(_) => None,
            //                 _ => Some(m),
            //             }
            //         }).forward(connections_inner3.write()
            //         .unwrap()
            //         .remove(&id)
            //         .expect("发送到无效的客户端ID",))
            // });

            handle.spawn(f.map_err(move |e| println!("{}: '{:?}'", addr, e))
                        .map(move |_| println!("{} closed.", addr)));
        Ok(())
    }).map_err(|_| ());

    // Handle receiving messages from a client
	let remote_inner = remote.clone();
    let game_sender_inner = game_sender.clone();
    //let connections_rh = connections.clone();
	let receive_handler = pool.spawn_fn(|| {
        receive_channel_in.for_each(move |(id, stream)| {
            let game_sender_clone = game_sender_inner.clone();
            //let connections_rh_clone = connections_rh.clone();
            remote_inner.spawn(move |_| {
                                stream.for_each(move |msg| {
                                                    process_message(id, msg, game_sender_clone.clone());
                                                    Ok(())
                                                    })
                                        .map_err(|_| ())
                                });
            Ok(())
        })
    });

	// Handle sending messages to a client
	let connections_inner = connections.clone();
	let remote_inner = remote.clone();
	let send_handler = pool.spawn_fn(move || {
		let connections = connections_inner.clone();
		let remote = remote_inner.clone();
		send_channel_in.for_each(move |(id, msg): (Id, String)| {
			let connections = connections.clone();
			let sink = connections.write()
			                      .unwrap()
			                      .remove(&id)
			                      .expect("Tried to send to invalid client id",);

			println!("Sending message '{}' to id {}", msg, id);
			let f = sink.send(OwnedMessage::Text(msg))
			            .and_then(move |sink| {
				                      connections.write().unwrap().insert(id, sink);
				                      Ok(())
				                     });
			remote.spawn(move |_| f.map_err(|_| ()));
			Ok(())
		})
		               .map_err(|_| ())
	});

    // Main 'logic' loop
	// let main_loop:CpuFuture<UnboundedSender<()>, String> = pool.spawn_fn(move || {
    //         future::loop_fn(send_channel_out, move |send_channel_out| {
    //             thread::sleep(Duration::from_millis(100));

    //             // let should_continue = update(connections.clone(), send_channel_out.clone(), &remote);
    //             // match should_continue {
    //             //     Ok(true) => Ok(Loop::Continue(send_channel_out)),
    //             //     Ok(false) => Ok(Loop::Break(())),
    //             //     Err(()) => Err(()),
    //             // }
    //             Ok(Loop::Continue(send_channel_out))
    //         })
    // });

    let handlers = connection_handler.select2(receive_handler.select(send_handler));
    core.run(handlers).map_err(|_| println!("Error while running core loop")).unwrap();
    println!("游戏服务结束.");
}

fn spawn_future<F, I, E>(f: F, desc: &'static str, handle: &Handle)
	where F: Future<Item = I, Error = E> + 'static,
	      E: Debug
{
	handle.spawn(f.map_err(move |e| println!("Error in {}: '{:?}'", desc, e))
	              .map(move |_| println!("{}: Finished.", desc)));
}

fn process_message(id: Id, msg: OwnedMessage, game_sender:Sender<(i32, Id, String)>) {
    match msg {
        OwnedMessage::Text(text) =>{
            println!("OwnedMessage {}", text);
            //         //println!("on_message:{:?}", msg);
            /*
                服务器端接收的消息:
                    玩家加入游戏=> MSG_START\nNAME
                    玩家键盘操作=> MSG_KEY_EVENT\nKeyEvent␟Key
            */
            //分离消息ID
            if let Some(lf) = text.find('\n'){
                if let Some(msg_id) = text.get(0..lf){
                    if let Ok(msg_id) = msg_id.parse::<i32>(){
                        let data = String::from(text.get(lf+1..).unwrap_or(""));
                        let _ = game_sender.send((msg_id, id, data));
                        return;
                    }
                }
            }
            //send_message(connections, &format!("{}", id), format!("{}\n消息格式错误", SERVER_MSG_ERR));
            //send_channel_out_clone.send((uid, msg));
        }
        OwnedMessage::Close(_) => {
            //玩家下线
            let _ = game_sender.send((MSG_DISCONNECT, id, "".to_string()));
        }
        OwnedMessage::Ping(_p) => {},
        OwnedMessage::Pong(_) => {},
        _ => {},
    }
}

fn send_message(connections: Arc<RwLock<HashMap<Id, SplitSink>>>, id:u32, msg:String){
    let connections_clone = connections.clone();
    let sink = connections.write()
                    .unwrap()
                    .remove(&id)
                    .expect("发送到无效的客户端ID",);
    let _f = sink.send(OwnedMessage::Text(msg))
                                .and_then(move |sink| {
                                            connections_clone.write().unwrap().insert(id, sink);
                                            Ok(())
                                            });
    //remote.spawn(move |_| f.map_err(|_| ()));
}