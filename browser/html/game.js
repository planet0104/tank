//绘图画布
var canvas = document.getElementById('canvas');
var ctx = canvas.getContext("2d");

//加载图片资源 srcMap为json对象
function loadResources(srcMap, listener){
    var total = Object.keys(srcMap).length;
    var resMap = new Map();
    function check(listener){
        if(listener)
            listener(resMap, resMap.size, total);
    }
    for(var key in srcMap){
            var image = new Image();
            image.key = key;
            image.src = srcMap[key];
            image.onload = function(){
                resMap.set(this.key, this);
                check(listener);
            };
    }
}

//随机移动
function random_move(){
    //随机走一段距离
    if (Math.random()>0.75){
        var keys = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"];
        var idx = Math.randInt(0, 3);
        var event = new Event('keydown');
        event.key = keys[idx];
        document.dispatchEvent(event);
    
        setTimeout(() => {
            var event = new Event('keyup');
            event.key = keys[idx];
            document.dispatchEvent(event);
        }, Math.randInt(500, 3000));
    }
}

//随机发射炮弹
function random_fire(){
    if (Math.random()>0.75){
        var event = new Event('keydown');
        event.key = " ";
        document.dispatchEvent(event);

        var event = new Event('keyup');
        event.key = " ";
        document.dispatchEvent(event);
    }
}

function random_action(){
    random_move();
    random_fire();
}

function _start(){
    setInterval(() => {
        random_action();
    }, 400);
}

function ping(ip, callback) {
    var img = new Image();
    img.onload = function() {callback(Date.now()-this.start)};
    img.onerror = function() {callback(Date.now()-this.start)};
    img.start = Date.now();
    img.src = "http://"+ip;
}

//电脑版不显示游戏手柄
try{
    if (/Android|webOS|iPhone|iPod|BlackBerry/i.test(navigator.userAgent)) {
        document.getElementById("game_pad").style.display = 'block';   
    }
}catch(e){}

document.getElementById("game_pad").style.display = 'block'; 

ping("54.249.68.59", function(time){
        console.log("ping:"+time+"ms");
    });