extern crate ws;
#[macro_use]
extern crate json;

use ws::{listen, Handler, Message, Request, Response, Result, Sender};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

const MSG_CREATE:i32 = 1;
const MSG_DELETE:i32 = 2;
const MSG_UPDATE:i32 = 3;
const MSG_QUERY:i32 = 4;

// 服务器Web处理程序
struct Server {
    out: Sender,
    resources: HashMap<String, Vec<u8>>, //存放静态资源
    games: HashMap<String, HashMap<String, String>>, //游戏数据
}

impl Server{
    fn new(out :Sender) -> Server {
        let mut server = Server{ out: out, resources: HashMap::new() };
        server.load_resources();
        server
    }

    //加载静态文件资源
    fn load_resources(&mut self){
        let dir = Path::new("./html");
        for entry in fs::read_dir(dir).unwrap(){
            let path = entry.unwrap().path();
            let mut buffer = vec![];
            File::open(&path).unwrap().read_to_end(&mut buffer).unwrap();
            let mut file_name = String::from(path.file_name().unwrap().to_str().unwrap());
            file_name.insert_str(0, "/");
            self.resources.insert(file_name, buffer);
        }
    }
}

impl Handler for Server {

    fn on_request(&mut self, req: &Request) -> Result<(Response)> {
        //路由多处理
        match req.resource() {
            //默认 trait 实现
            "/ws" => Response::from_request(req),

            //分配静态文件
            file =>{
                let file = if file == "/"{ "/index.html" }else{ file };
                if self.resources.contains_key(file){
                    let content = self.resources.get(file).unwrap();
                    let content_length = content.len();
                    let mut response = Response::new(200, "OK", content.to_vec());
                    let content_type = match Path::new(file).extension().unwrap().to_str().unwrap(){
                        "html" => "text/html",
                        "js" => "text/javascript",
                        ".png" => "image/png",
                        _ => "application/octet-stream"
                    };
                    response.headers_mut().push((String::from("Content-type"), String::from(content_type).into_bytes()));
                    response.headers_mut().push((String::from("Content-Length"), format!("{}", content_length).into_bytes()));
                    Ok(response)
                }else{
                    Ok(Response::new(404, "文件未找到", self.resources.get("/404.html").unwrap().to_vec()))
                }
            }
        }
    }

    //处理websocket接收到的消息 (/ws)
    fn on_message(&mut self, msg: Message) -> Result<()> {
        
        if !msg.is_text(){
            return Ok();//非文本消息
        }

        let msg = msg.into_text().unwrap();
        let json = json::parse(msg);
        
        if json.is_err(){
            return Ok();//json解析失败
        }

        let json = json.unwrap();
        let msg_id = json["id"];
        let game = json["game"];
        let sprite = json["sprite"];

        if !msg_id.is_number() || !game.is_string() || !sprite.is_object(){
            return Ok(); //json结构错误
        }

        let msg_id = msg_id.as_number().unwrap();
        let game = game.as_str().unwrap();

        //创建游戏数据
        if(!GameMap.has(message.game)){
        GameMap.set(message.game, new Map());
        }

        match msg_id.into(){
            MSG_CREATE => {
                //添加(创建新的精灵)
            },
            MSG_DELETE => {
                //删除(精灵死亡)
            },
            MSG_UPDATE => {
                //修改
            },
            MSG_QUERY => {
                //查询
            },
            _ => ()
        }
        //self.out.broadcast(msg)
    }
}

fn main() {
    // Listen on an address and call the closure for each connection
    listen("127.0.0.1:8000", |out| Server::new(out)).unwrap()
}