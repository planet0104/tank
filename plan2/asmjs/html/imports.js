//绘图画布
var canvas = document.getElementById('canvas');
var ctx = canvas.getContext("2d");

canvas.addEventListener("click", function(event){
    Module._on_click_event(event.clientX, event.clientY);
});

canvas.addEventListener("touchmove", function(event){
    Module._on_touch_move(event.touches[0].clientX, event.touches[0].clientY);
});

//下面是要导入webassembly的JS帮助函数
function console_log(str){
    console.log(str);
}
function current_time_millis(){
    return Date.now();
}
function random(){
    return Math.random();
}

function request_animation_frame(){
    window.requestAnimationFrame(Module._request_animation_frame_callback);
}

function load_resource(object){
    var urls = JSON.parse(object);
    loadResources(urls, function(map, num, total){
        window.resMap = map;
        Module._on_resource_load(num, total);
    });
}
function set_canvas_height(height){
    canvas.height = height;
}
function set_canvas_width(width){
    canvas.width = width;
}
function set_canvas_style_margin(left, top, right, bottom){
    canvas.style.marginLeft = left+'px';
    canvas.style.marginTop = top+'px';
    canvas.style.marginRight = right+'px';
    canvas.style.marginBottom = bottom+'px';
}
function set_canvas_style_width(width){
    canvas.style.width = width+'px';
}
function set_canvas_style_height(height){
    canvas.style.height = height+'px';
}
function set_canvas_font(font){
    ctx.font = font;
}
function fill_style(st){
    ctx.fillStyle = st;
}
function fill_rect(x, y, width, height){
    ctx.fillRect(x, y, width, height);
}
function fill_text(text, x, y){
    ctx.fillText(text, x, y);
}
function draw_image_at(resId, x, y){
    ctx.drawImage(window.resMap.get(resId+""), x, y);
}
function draw_image(resId, sourceX, sourceY, sourceWidth, sourceHeight, destX, destY, destWidth, destHeight){
    ctx.drawImage(window.resMap.get(resId+""), sourceX, sourceY, sourceWidth, sourceHeight, destX, destY, destWidth, destHeight);
}
function send_message(str){
    socket.send(str);
}
function connect(url){
    connect(url);
}

//从wasm内存读取字符串
//offset 指针
//len 长度
function read_string(offset, len){
    const string_buffer = new Uint8Array(exports.memory.buffer, offset, len);
    if (typeof TextDecoder === "function"){
        return new TextDecoder("UTF-8").decode(string_buffer);
    }else{
        return decode_utf8(string_buffer);
    }
}

//想Webassembly的memory.buffer写入utf8字符串，并返回该字符串的指针
//string 字符串
//buffer wasm的内存
//返回 字符串指针
function alloc_string(string){
    var encoded;
    if (typeof TextEncoder === "function"){
        encoded = new TextEncoder("UTF-8").encode(string);
    }else{
        encoded = encode_utf8(string);
    }
    var offset = Module._alloc(encoded.length);
    const bytes = new Uint8Array(exports.memory.buffer, offset, encoded.length);
    bytes.set(encoded);
    return { ptr:offset, len:bytes.length };
}

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


//源码:https://github.com/samthor/fast-text-encoding

//一些浏览器不支持TextEncoder,使用此方法代替
function encode_utf8(string) {
    let pos = 0;
    const len = string.length;
    const out = [];
  
    let at = 0;  // output position
    let tlen = Math.max(32, len + (len >> 1) + 7);  // 1.5x size
    let target = new Uint8Array((tlen >> 3) << 3);  // ... but at 8 byte offset
  
    while (pos < len) {
      let value = string.charCodeAt(pos++);
      if (value >= 0xd800 && value <= 0xdbff) {
        // high surrogate
        if (pos < len) {
          const extra = string.charCodeAt(pos);
          if ((extra & 0xfc00) === 0xdc00) {
            ++pos;
            value = ((value & 0x3ff) << 10) + (extra & 0x3ff) + 0x10000;
          }
        }
        if (value >= 0xd800 && value <= 0xdbff) {
          continue;  // drop lone surrogate
        }
      }
  
      // expand the buffer if we couldn't write 4 bytes
      if (at + 4 > target.length) {
        tlen += 8;  // minimum extra
        tlen *= (1.0 + (pos / string.length) * 2);  // take 2x the remaining
        tlen = (tlen >> 3) << 3;  // 8 byte offset
  
        const update = new Uint8Array(tlen);
        update.set(target);
        target = update;
      }
  
      if ((value & 0xffffff80) === 0) {  // 1-byte
        target[at++] = value;  // ASCII
        continue;
      } else if ((value & 0xfffff800) === 0) {  // 2-byte
        target[at++] = ((value >>  6) & 0x1f) | 0xc0;
      } else if ((value & 0xffff0000) === 0) {  // 3-byte
        target[at++] = ((value >> 12) & 0x0f) | 0xe0;
        target[at++] = ((value >>  6) & 0x3f) | 0x80;
      } else if ((value & 0xffe00000) === 0) {  // 4-byte
        target[at++] = ((value >> 18) & 0x07) | 0xf0;
        target[at++] = ((value >> 12) & 0x3f) | 0x80;
        target[at++] = ((value >>  6) & 0x3f) | 0x80;
      } else {
        // FIXME: do we care
        continue;
      }
  
      target[at++] = (value & 0x3f) | 0x80;
    }
  
    return target.slice(0, at);
  }

//一些浏览器不支持TextDecoder,使用此方法代替
function decode_utf8(bytes) {
    //const bytes = new Uint8Array(buffer);
    let pos = 0;
    const len = bytes.length;
    const out = [];
  
    while (pos < len) {
      const byte1 = bytes[pos++];
      if (byte1 === 0) {
        break;  // NULL
      }
    
      if ((byte1 & 0x80) === 0) {  // 1-byte
        out.push(byte1);
      } else if ((byte1 & 0xe0) === 0xc0) {  // 2-byte
        const byte2 = bytes[pos++] & 0x3f;
        out.push(((byte1 & 0x1f) << 6) | byte2);
      } else if ((byte1 & 0xf0) === 0xe0) {
        const byte2 = bytes[pos++] & 0x3f;
        const byte3 = bytes[pos++] & 0x3f;
        out.push(((byte1 & 0x1f) << 12) | (byte2 << 6) | byte3);
      } else if ((byte1 & 0xf8) === 0xf0) {
        const byte2 = bytes[pos++] & 0x3f;
        const byte3 = bytes[pos++] & 0x3f;
        const byte4 = bytes[pos++] & 0x3f;
  
        // this can be > 0xffff, so possibly generate surrogates
        let codepoint = ((byte1 & 0x07) << 0x12) | (byte2 << 0x0c) | (byte3 << 0x06) | byte4;
        if (codepoint > 0xffff) {
          // codepoint &= ~0x10000;
          codepoint -= 0x10000;
          out.push((codepoint >>> 10) & 0x3ff | 0xd800)
          codepoint = 0xdc00 | codepoint & 0x3ff;
        }
        out.push(codepoint);
      } else {
        // FIXME: we're ignoring this
      }
    }
    return String.fromCharCode.apply(null, out);
  }

var socket;

mergeInto(LibraryManager.library, {
    my_js: function() {
        alert('hi');
    },
});

//连接websocket
function connect(url){
    socket = new WebSocket(url);
    socket.onopen = function(event) {
        Module._on_connect();
        socket.onclose = function(event) {
            Module._on_close();
        };
    }

    socket.onerror = function(){
        alert("连接失败，请重试");
    }
}

mergeInto(LibraryManager.library, {
    test_alert: function() {
        window.alert("hello!");
    },
});