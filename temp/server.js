// annrobot server demo

//https://www.npmjs.com/package/ws

const http = require('http');
const WebSocket = require('ws');
const url = require('url');
const fs = require('fs');

var index = fs.readFileSync('index.html');
const server = http.createServer((req, res) => {
    res.end(index);
});
server.on('clientError', (err, socket) => {
  socket.end('HTTP/1.1 400 Bad Request\r\n\r\n');
});

var port = process.env.PORT || process.env.OPENSHIFT_NODEJS_PORT || 8080,
    ip   = process.env.IP   || process.env.OPENSHIFT_NODEJS_IP || '0.0.0.0';

const wss = new WebSocket.Server({ server });

//服务器维护一个列表，可以 添加、删除、修改、查询精灵
//添加: 玩家加入游戏、玩家发射子弹(更改服务器列表且发送广播)  MessageID=1
//删除: 玩家死亡、子弹死亡(更改服务器列表且发送广播)         MessageID=2
//修改: 玩家按键以后(更改服务器列表且发送广播)              MessageID=3
//查询: 玩家连接,返回游戏对应的所有精灵数组(下发)           MessageID=4
//消息结构
/*
  {id:1, game:Tank, sprite:}
 */

const MSG_CREATE = 1;
const MSG_DELETE = 2;
const MSG_UPDATE = 3;
const MSG_QUERY = 4;

var GameMap = new Map();

wss.broadcast = function broadcast(data) {
  wss.clients.forEach(function each(client) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(data);
    }
  });
};

wss.on('connection', function connection(ws, req) {
  const location = url.parse(req.url, true);
  console.log("客户端链接");
  ws.on('message', function incoming(data) {
    var message = JSON.parse(data);
    if(!message.id || !message.game){
      console.log("无效消息", message);
      return;
    }
    console.log(message);
    //创建游戏数据
    if(!GameMap.has(message.game)){
      GameMap.set(message.game, new Map());
    }
    var dataMap = GameMap.get(message.game);
    switch (message.id){
      case MSG_CREATE:
      //添加(创建新的精灵)
      dataMap.set(message.sprite.id, message.sprite.value);
      wss.broadcast(message);
      break;
      case MSG_DELETE:
      //删除(精灵死亡)
      dataMap.delete(message.sprite);
      wss.broadcast(message);
      break;
      case MSG_UPDATE:
      //修改
      dataMap.set(message.sprite.id, message.sprite.value);
      wss.broadcast(message);
      break;
      case MSG_QUERY:
      //查询
      ws.send(JSON.stringify({
        id: message.id,
        game: message.game,
        data: dataMap
      }));
      break;
    }
  });
});

console.log('Server running on http://%s:%s', ip, port);
server.listen(port, ip);