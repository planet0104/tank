//加载图片资源
function loadResources(srcMap, cb, listener){
    var resMap = new Map();
    function check(listener){
        if(listener)
            listener(resMap.size, srcMap.size);
        if(srcMap.size == resMap.size){
            cb(resMap);
        }
    }
    for(var [key, url] of srcMap.entries()){
            var image = new Image();
            image.key = key;
            image.src = url;
            image.onload = function(){
                resMap.set(this.key, this);
                check(listener);
            };
    }
}

//从wasm内存读取字符串
//offset 指针
//len 长度
function read_string(buffer, offset, len){
    const string_buffer = new Uint8Array(buffer, offset, len);
    if (typeof TextDecoder === "function"){
        return new TextDecoder("UTF-8").decode(string_buffer);
    }else{
        return decode_utf8(string_buffer);
    }
}

//想Webassembly的memory.buffer的offset位置上写入utf8字符串
//string 字符串
//buffer wasm的内存
//offset 指针
//返回 byteLength
function write_string(string, buffer, offset){
    var encoded;
    if (typeof TextEncoder === "function"){
        encoded = new TextEncoder("UTF-8").encode(string);
    }else{
        encoded = encode_utf8(string);
    }
    const bytes = new Uint8Array(buffer, offset, encoded.length);
    bytes.set(encoded);
    return bytes.byteLength;
}

//是否支持webassemboy
function support_webassembly(){
    try {
        if (typeof WebAssembly === "object"
            && typeof WebAssembly.instantiate === "function") {
            const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
            if (module instanceof WebAssembly.Module)
                return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
        }
    } catch (e) {
    }
    return false;
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