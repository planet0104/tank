//绘图画布
var canvas = document.getElementById('canvas');
var ctx = canvas.getContext("2d");
var exports; //Webassembly

document.addEventListener("keyup", function(event){
    var str = alloc_string(event.key);
    exports.on_keyup_event(str.ptr, str.len);
});

document.addEventListener("keydown", function(event){
    var str = alloc_string(event.key);
    exports.on_keydown_event(str.ptr, str.len);
});

canvas.addEventListener("click", function(event){
    exports.on_click_event(event.clientX, event.clientY);
});

canvas.addEventListener("touchmove", function(event){
    exports.on_touch_move(event.touches[0].clientX, event.touches[0].clientY);
});

//下面是要导入webassembly的JS帮助函数
var imports = {
    env: {
        _console_log: function(str_ptr, len){
            console.log(read_string(str_ptr, len));
        },
        _current_time_millis: function(){
            return Date.now();
        },
        _random: function(){
            return Math.random();
        },
        _window_inner_width: function(){ return window.innerWidth; },
        _window_inner_height: function(){ return window.innerHeight; },
        _request_animation_frame: function(){
            window.requestAnimationFrame(exports.request_animation_frame_callback);
        },
        _load_resource: function(object, len){
            var urls = JSON.parse(read_string(object, len));
            loadResources(urls, function(map, num, total){
                window.resMap = map;
                exports.on_resource_load(num, total);
            });
        },
        _set_canvas_height: function(height){
            canvas.height = height;
        },
        _set_canvas_width: function(width){
            canvas.width = width;
        },
        _set_canvas_style_margin: function(left, top, right, bottom){
            canvas.style.marginLeft = left+'px';
            canvas.style.marginTop = top+'px';
            canvas.style.marginRight = right+'px';
            canvas.style.marginBottom = bottom+'px';
        },
        _set_canvas_style_width: function(width){
            canvas.style.width = width+'px';
        },
        _set_canvas_style_height: function(height){
            canvas.style.height = height+'px';
        },
        _set_canvas_font: function(font_ptr, len){
            var font = read_string(font_ptr, len);
            ctx.font = font;
        },
        _fill_style: function(str, len){
            ctx.fillStyle = read_string(str, len);
        },
        _fill_rect: function(x, y, width, height){
            ctx.fillRect(x, y, width, height);
        },
        _fill_text: function(text_ptr, len, x, y){
            ctx.fillText(read_string(text_ptr, len), x, y);
        },
        _draw_image_at: function(resId, x, y){
            ctx.drawImage(window.resMap.get(resId+""), x, y);
        },
        _draw_image: function(resId, sourceX, sourceY, sourceWidth, sourceHeight, destX, destY, destWidth, destHeight){
            ctx.drawImage(window.resMap.get(resId+""), sourceX, sourceY, sourceWidth, sourceHeight, destX, destY, destWidth, destHeight);
        },
        _send_message: function(str, len){
            socket.send(read_string(str, len));
        },
        _connect: function(url, len){
            connect(read_string(url, len));
        }
    }
};

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
    var offset = exports.alloc(encoded.length);
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

//连接websocket
function connect(url){
    socket = new WebSocket(url);
    console.log("连接服务器...");

    socket.onopen = function(event) {
        exports.on_connect();

        socket.onmessage = function(event){
            var msg = alloc_string(event.data);
            exports.on_message(msg.ptr, msg.len);
        };

        socket.onclose = function(event) {
            exports.on_close();
        };
    }

    socket.onerror = function(){
        alert("连接失败，请重试");
    }
}