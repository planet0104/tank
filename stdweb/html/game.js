//绘图画布
var canvas = document.getElementById('canvas');
var ctx = canvas.getContext("2d");

let game_pad = document.getElementById("game_pad");
let game_pad_direction = document.getElementById("game_pad_direction");
let game_pad_button_a = document.getElementById("game_pad_button_a");
let game_pad_button_b = document.getElementById("game_pad_button_b");
game_pad_direction.status = 0; // 0:未按, 1: Up, 2:Down, 3:Left, 4:Right
var game_pad_direction_active = function(event){
    event.preventDefault();
    //方向按钮按下 判断按钮方向
    let x = (event.type=="click"? event.clientX : event.touches[0].clientX) - game_pad.offsetLeft - game_pad_direction.offsetLeft;
    let y = (event.type=="click"? event.clientY : event.touches[0].clientY) - game_pad.offsetTop - game_pad_direction.offsetTop;
    let btn_width = game_pad_direction.clientWidth/3;
    if(x>=btn_width&&x<=btn_width*2&&y<=btn_width && game_pad_direction.status != 1){
        game_pad_direction.status = 1;
        Module.exports.on_keydown_event(VK_UP);
    }
    if(x>=btn_width&&x<btn_width*2&&y>=btn_width*2&&y<=btn_width*3 && game_pad_direction.status != 2){
        game_pad_direction.status = 2;
        Module.exports.on_keydown_event(VK_DOWN);
    }
    if(x<=btn_width&&y>=btn_width&&y<=btn_width*2 && game_pad_direction.status != 3){
        game_pad_direction.status = 3;
        Module.exports.on_keydown_event(VK_LEFT);
    }
    if(x>=btn_width*2&&y>=btn_width&&y<=btn_width*2 && game_pad_direction.status != 4){
        game_pad_direction.status = 4;
        Module.exports.on_keydown_event(VK_RIGHT);
    }
}

game_pad_direction.addEventListener("touchmove", game_pad_direction_active);
//game_pad_direction.addEventListener("click", game_pad_direction_active);
game_pad_direction.addEventListener("touchstart", game_pad_direction_active);
game_pad_direction.addEventListener("touchend", function(event){
    event.preventDefault();
    //方向按钮弹起
    Module.exports.on_keyup_event(VK_LEFT);
    game_pad_direction.status = 0;
});
game_pad_button_a.addEventListener("touchstart", function(event){
    event.preventDefault();
    Module.exports.on_keydown_event(VK_SPACE);
});
game_pad_button_b.addEventListener("touchstart", function(event){
    event.preventDefault();
    Module.exports.on_keydown_event(VK_SPACE);
});

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

ping("54.249.68.59", function(time){
        console.log("ping:"+time+"ms");
    });